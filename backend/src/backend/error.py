class CameraNotStartedError(Exception):
    def __init__(self, action_name: str, camera_name: str):
        self.message = f"Error while {action_name}: the camera with name \
            `{camera_name}` doesn't have a VideoCapture object to modify. Make \
            sure the camera has been started before continuing."
        super().__init__(self.message)


class CameraNotFoundError(Exception):
    def __init__(self, camera_name: str):
        self.message = f"Error: Camera with name {
            camera_name} cannot be found."
        super().__init__(self.message)


class VideoCaptureError(Exception):
    def __init__(self, camera_name: str):
        self.message = f"Error: Faiiled to start camera with name `{camera_name}`."
        super().__init__(self.message)
