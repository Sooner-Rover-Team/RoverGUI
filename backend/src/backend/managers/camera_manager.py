import subprocess
from dataclasses import dataclass

import cv2

from ..error import CameraNotFoundError, CameraNotStartedError, VideoCaptureError


def get_camera_name_and_paths() -> dict[str:str]:
    """
    Returns a dictionary containing the camera name and its path (`/dev/videox`, etc.)

    This only works on linux systems that have `v412-ctl` installed.

    TODO: Replace with a Video4Linux library rather than directly parsing text
    """

    # Try to read available camera streams
    # If no cameras are found, an error is raised which we ignore
    command = "v4l2-ctl --list-devices"
    output = ""
    try:
        output = subprocess.check_output(command, shell=True, text=True)
    except Exception as e:
        print(f"{e}")

    # Create a dictionary mapping camera name to device path
    cameras = {}
    lines = output.strip().split("\n")
    lines = [line.replace("\t", "") for line in lines]  # Remove tabs in output
    lines = [line for line in lines if line != ""]  # Remove empty lines
    curr_line = 0
    while curr_line < len(lines):
        # Read the camera name
        camera_name = lines[curr_line].split(":")[0]
        curr_line += 1

        # Read the camera file path
        camera_path = lines[curr_line]
        curr_line += 1

        # Add camera to dictionary
        cameras[camera_name] = camera_path

        # Skip over other device file paths
        while curr_line < len(lines) and lines[curr_line].startswith("/dev/"):
            curr_line += 1
    return cameras


@dataclass
class Resolution:
    verical: int
    horizontal: int

    def __str__(self) -> str:
        f"{self.verical}x{self.horizontal}"


@dataclass
class Camera:
    """
    A representation of camera that stores information like name, camera index, etc.
    """

    camera_name: str
    camera_path: str
    camera_fps: int
    resolution: Resolution
    image_quality: int
    video_capture: cv2.VideoCapture | None

    encoding_params: list[int] = [cv2.IMWRITE_JPEG_QUALITY, 90]

    def __init__(
        self,
        camera_name: str,
        camera_path: str,
        resolution: Resolution | None = Resolution(640, 480),
        camera_fps: int = 30,
        image_quality: int = 90,
        video_capture: cv2.VideoCapture | None = None,
    ):
        self.name: str = camera_name
        self.path: str = camera_path
        self.resolution: Resolution = resolution
        self.fps: int = camera_fps
        self.image_quality: int = image_quality
        self.video_capture: cv2.VideoCapture | None = video_capture
        self.is_running: bool = False

        """
        Set the encoding quality for each frame that is sent to a client.
        Although this is a list, it represents a key-value pair (WHY OPENCV?!).
        90 is our default value, but it can range between 0-100 with 100 meaning that quality of the frame is maintained.
        TODO: Consider other ways to compress, maybe use WebP or Pillow library
        """
        self.encoding_params: list[int]

    def start(self, video_capture: cv2.VideoCapture):
        """
        Uses a given video capture object to start the camera.

        Raises a `VideoCaptureError` if OpenCV fails to get a lock on the camera.
        """
        if self.video_capture.isOpened():
            self.video_capture = video_capture
            self.is_running = True
        else:
            raise VideoCaptureError(self.name)

    def stop(self):
        """
        Stops the camera. This releases the video capture device for other uses.
        """
        self.video_capture.release()
        self.video_capture = None
        self.is_running = False

    def set_current_resolution(self, resolution: Resolution):
        """
        Sets the camera's current resolution.

        Raises a `CameraNotStartedError` if the camera hasn't been started yet.
        """
        if self.video_capture is not None:
            self.video_capture.set(cv2.CAP_PROP_FRAME_WIDTH, resolution.horizontal)
            self.video_capture.set(cv2.CAP_PROP_FRAME_HEIGHT, resolution.vertical)
            self.resolution = resolution
        else:
            raise CameraNotStartedError("setting current resolution", self.name)

    def set_fps(self, fps: int):
        """
        Sets the camera's fps.

        Raises a `CameraNotStartedError` if the camera hasn't been started yet.
        """
        if self.video_capture is not None:
            self.video_capture.set(cv2.CAP_PROP_FPS, fps)
            self.fps = fps
        else:
            raise CameraNotStartedError("setting current fps", self.name)

    def set_image_quality(self, quality: int):
        """
        Sets the camera's image quality, a value ranging from 0 to 100
        representing the amount of image compression.

        Raises a `CameraNotStartedError` if the camera hasn't been started yet,
        or a `ValueError` if the quality isn't from 0 to 100.
        """
        if quality < 0 or quality > 100:
            raise ValueError("Image quality must be between 0 and 100.")

        if self.video_capture is not None:
            self.video_capture.set(cv2.IMWRITE_JPEG_QUALITY, quality)
            self.image_quality = quality
        else:
            raise CameraNotStartedError("setting current image quality", self.name)


class CameraManager:
    """
    Camera Manager manages a list of currently connected USB cameras (at instantiation)
    and keeps track of which cameras are being used by along with a list of their configurations (fps, etc.)
    """

    def __init__(self):
        # List of available cameras
        self.cameras: list[Camera] = self.__create_cameras()

    def __create_cameras(self) -> list[Camera]:
        """
        Create a list of cameras available on the computer
        NOTE: Will only work on Linux
        """
        camera_dict = get_camera_name_and_paths()
        cameras = []
        for camera_name, camera_path in camera_dict.items():
            cameras.append(Camera(camera_name, camera_path))
        return cameras

    def get_camera(self, camera_name: str) -> Camera:
        """
        Get the Camera object associated with a given camera name

        Raises: CameraNotFoundError if specified camera cannot be found
        """
        for camera in self.cameras:
            if camera.name == camera_name:
                return camera

        # Raise error if camera not found
        raise CameraNotFoundError(camera_name)

    def camera_path_from_name(self, camera_name: str) -> str:
        """
        Get the path to a camera given its name

        Raises: CameraNotFoundError if specified camera cannot be found
        """
        for camera in self.cameras:
            if camera.name == camera_name:
                return camera.path

        # Raise error if camera not found
        raise CameraNotFoundError(camera_name)

    def get_available_cameras(self) -> list[str]:
        """
        Return a list of cameras which are not currently being used
        """
        available_cameras = []
        for camera in self.cameras:
            if not camera.is_running:
                available_cameras.append(camera.name)
        return available_cameras
