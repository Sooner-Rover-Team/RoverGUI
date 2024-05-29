import subprocess
from dataclasses import dataclass

import cv2


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


class Camera:
    """
    A representation of camera that stores information like name, camera index, etc.
    """

    def __init__(
        self,
        camera_name: str,
        camera_path: str,
        camera_fps: int = 30,
        resolution: Resolution = Resolution(640, 480),
    ):
        self.name: str = camera_name
        self.path: str = camera_path
        self.resolution: Resolution = resolution
        self.fps: int = camera_fps
        self.is_running: bool = False

        """
        Set the encoding quality for each frame that is sent to a client.
        Although this is a list, it represents a key-value pair (WHY OPENCV?!).
        90 is our default value, but it can range between 0-100 with 100 meaning that quality of the frame is maintained.
        TODO: Consider other ways to compress, maybe use WebP or Pillow library
        """
        self.encoding_params: list[int]


class CameraNotFoundError(Exception):
    def __init__(self, camera_name: str):
        self.message = f"Error: Camera with name {
            camera_name} cannot be found."
        super().__init__(self.message)


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

    def __get_camera(self, camera_name: str) -> Camera:
        """
        Get the Camera object associated with a given camera name

        Raises: CameraNotFoundError if specified camera cannot be found
        """
        for camera in self.cameras:
            if camera.name == camera_name:
                return camera

        # Raise error if camera not found
        raise CameraNotFoundError(camera_name)

    def get_camera_fps(self, camera_name: str) -> int:
        """
        Return the fps for a camera, given a camera name

        Raises: CameraNotFoundError if specified camera cannot be found
        """
        try:
            camera = self.__get_camera(camera_name)
        except CameraNotFoundError:
            raise
        return camera.fps

    def set_camera_fps(self, camera_name: str, fps: int):
        """
        Change the fps for a camera, given a camera name and fps

        Raises: CameraNotFoundError if specified camera cannot be found
        """
        try:
            camera = self.__get_camera(camera_name)
        except CameraNotFoundError:
            raise
        camera.fps = fps

    def get_camera_encoding_params(self, camera_name: str) -> list[int]:
        """
        Return the encoding parameters given a camera name
        Will always be in the form [Image Type (JPEG, PNG, etc.), quality of image (0-100)]

        Raises: CameraNotFoundError if specified camera cannot be found
        """
        try:
            camera = self.__get_camera(camera_name)
        except CameraNotFoundError:
            raise
        return camera.encoding_params

    def set_camera_encoding_params(self, camera_name: str, encoding_quality: int):
        """
        Set the camera encoding parameters given a camera name

        Raises: CameraNotFoundError if specified camera cannot be found
        """
        try:
            camera = self.__get_camera(camera_name)
        except CameraNotFoundError:
            raise

        # Change quality of captured frames
        camera.encoding_params[1] = encoding_quality

    def camera_is_running(self, camera_name: str) -> bool:
        """
        Return True if camera stream is being asked for and False if it has been ended

        Raises: CameraNotFoundError if specified camera cannot be found
        """
        try:
            camera = self.__get_camera(camera_name)
        except CameraNotFoundError:
            raise
        return camera.is_running

    def start_video_capture(self, camera_name: str) -> cv2.VideoCapture:
        """
        Given a camera name, return an openCV video capture object (to read frames)

        Raises: CameraNotFoundError if specified camera cannot be found

        """
        try:
            camera = self.__get_camera(camera_name)
        except CameraNotFoundError:
            raise

        # Create video capture device
        cap = cv2.VideoCapture(camera.path)
        camera.is_running = True
        return cap

    def end_video_capture(self, camera_name: str):
        """
        Set a camera to no longer run

        Raises: CameraNotFoundError if specified camera cannot be found
        """
        try:
            camera = self.__get_camera(camera_name)
        except CameraNotFoundError:
            raise
        camera.is_running = False

    def get_available_cameras(self) -> list[str]:
        """
        Return a list of cameras which are not currently being used
        """
        available_cameras = []
        for camera in self.cameras:
            if not camera.is_running:
                available_cameras.append(camera.name)
        return available_cameras
