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


@dataclass(kw_only=True)
class Resolution:
    horizontal: int
    vertical: int

    def __str__(self) -> str:
        return f"{self.horizontal}x{self.vertical}"


@dataclass
class Camera:
    """
    A representation of camera that stores information like name, camera index, etc.
    """

    name: str
    path: str
    fps: int
    resolution: Resolution
    image_quality: int
    video_capture: cv2.VideoCapture | None
    is_streaming: bool = False
    is_capturing: bool = False

    def __init__(
        self,
        camera_name: str,
        camera_path: str,
        resolution: Resolution | None = Resolution(horizontal=640, vertical=480),
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
        self.is_streaming: bool = False
        self.is_capturing: bool = False

    def read(self) -> Union[Tuple[bool, cv2.Mat], None]:
        """
        Read a frame or return None if the camera isn't able to capture frames at the moment.
        """
        if self.is_capturing and self.video_capture is not None:
            return self.video_capture.read()
        else:
            return None


    def start(self, video_capture: cv2.VideoCapture):
        """
        Uses a given video capture object to start the camera.

        Raises a `VideoCaptureError` if OpenCV fails to get a lock on the camera.
        """
        logger.info(f"Starting camera `{self.name}`...")

        if self.video_capture is not None:
            self.video_capture.release()

        self.video_capture = video_capture
        self.is_capturing = True


    def reset(self):
        """
        Recreates the internal `VideoCapture` object using the internal state.

        This assumes that the camera is already started. If not, it will start it.

        Throws a `TODO Error` when the capture device fails to open given internal state.
        """
        # release the internal object
        logger.info("Destructing internal VideoCapture...")
        self.stop()
        logger.info("Destruction complete!")

        # make a new capture device and set its properties
        cap = cv2.VideoCapture(self.path)
        cap.set(cv2.CAP_PROP_FRAME_HEIGHT, self.resolution.vertical)
        cap.set(cv2.CAP_PROP_FRAME_WIDTH, self.resolution.horizontal)
        cap.set(cv2.CAP_PROP_FPS, self.fps)

        if not cap.isOpened():
            # return an error
            logger.error("hey the new capture device didn't open :(")
        else:
            self.start(cap)
            logger.info("Completed VideoCapture reset!")

    def stop(self):
        """
        Stops the camera. This releases the video capture device for other uses.
        """
        if self.video_capture is not None:
            self.video_capture.release()
        self.is_capturing = False
        self.video_capture = None
        logger.info(f"Camera `{self.name}` stopped successfully!")

    def stream(self):
        """
        A generator that will yield the frames for a given camera

        TODO: Instead of reading frames and yielding them directly, utilize multiprocessing
        so that each camera is running on its own process and then this loop is only grabbing from said processes queue.
        """
        self.is_streaming = True
        # Capture video frames at specified frame rate
        time_since_last_frame = 0
        while self.is_streaming:
            # Get the wait time based on fps
            wait_time = fps_to_ms(self.fps)


            # Check if it's time to send a frame
            # NOTE: The time function is in millis, so x1000 makes it in seconds
            if self.is_capturing and time.time() * 1000 - time_since_last_frame > wait_time:
                # Capture a video frame
                frame_read = self.read()
                if frame_read is None:  # If none, then camera is waiting
                    logger.info("Camera is currently not capturing frames...")
                    time.sleep(1)
                    continue

                # Ensure frame could be captured successfully
                success, frame = frame_read
                if not success:
                    logger.info("Couldn't capture frame")
                    continue

                # Get the encoding quality of camera
                ret, buffer = cv2.imencode(".jpg", frame, [ cv2.IMWRITE_JPEG_QUALITY, self.image_quality])
                frame = buffer.tobytes()

                # Yield the current frame
                yield (b"--frame\r\n" b"Content-Type: image/jpeg\r\n\r\n" + frame + b"\r\n")
                time_since_last_frame = time.time() * 1000
        self.stop()

    def end_stream(self):
        """
        End streaming for a camera
        """
        self.is_streaming = False

    def set_current_resolution(self, resolution: Resolution):
        """
        Sets the camera's current resolution.

        Raises a `CameraNotStartedError` if the camera hasn't been started yet.
        """
        if self.video_capture is not None:
            self.resolution = resolution
            self.reset()
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
            if not camera.is_streaming:
                available_cameras.append(camera.name)
        return available_cameras
