from fastapi import FastAPI, Response
from fastapi.responses import StreamingResponse, JSONResponse
from fastapi.exceptions import HTTPException
from fastapi.middleware.cors import CORSMiddleware
import cv2
import time

from backend.managers.camera_manager import CameraManager, CameraNotFoundError

# Create api
app = FastAPI()

# Configure CORS
origins = [
    "http://localhost:3000",  # React development server
]
app.add_middleware(
    CORSMiddleware, allow_origins=origins, allow_methods=["*"], allow_headers=["*"]
)

# Create camera manager
camera_manager = CameraManager()


def fps_to_ms(fps: int) -> float:
    """
    Convert frames per second to frames per miliseconds (time between each frame)
    """
    return (1 / fps) * 1000


def generate_frames(cap: cv2.VideoCapture, camera_name: str):
    """
    A generator that will yield the frames for a given camera

    TODO: Instead of reading frames and yielding them directly, utilize multiprocessing
    so that each camera is running on its own process and then this loop is only grabbing from said processes queue.
    """
    # Capture video frames at specified frame rate
    time_since_last_frame = 0
    while camera_manager.camera_is_running(camera_name):
        # Get the wait time based on fps
        wait_time = fps_to_ms(camera_manager.get_camera_fps(camera_name))

        # Capture a video frame
        success, frame = cap.read()
        if not success:
            break

        # Check if it's time to send a frame
        # NOTE: The time function is in millis, so x1000 makes it in seconds
        if time.time() * 1000 - time_since_last_frame > wait_time:
            # Get the encoding quality of camera
            encoding_params = camera_manager.get_camera_encoding_params(camera_name)

            # Encode frame and convert to byte string
            ret, buffer = cv2.imencode(".jpg", frame, encoding_params)
            frame = buffer.tobytes()

            # Yield the current frame
            yield (b"--frame\r\n" b"Content-Type: image/jpeg\r\n\r\n" + frame + b"\r\n")
            time_since_last_frame = time.time() * 1000
    cap.release()


@app.get("/hello")
async def hello():
    return "Sup from backend"


@app.get("/stream/start/{camera_name}")
async def start_stream(camera_name: str) -> StreamingResponse:
    """
    Take in a camera name and start a video stream
    """
    # If camera is already streaming, return bad response
    if camera_manager.camera_is_running(camera_name):
        raise HTTPException(
            status_code=400, detail=f"{camera_name} is already streaming"
        )
    try:
        # Create video capture object for camera
        cap = camera_manager.start_video_capture(camera_name)
        cap.set(cv2.CAP_PROP_BUFFERSIZE, 1)  # Set camera parameters.

    except CameraNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    # Return streaming response
    return StreamingResponse(
        generate_frames(cap, camera_name),
        media_type="multipart/x-mixed-replace; boundary=frame",
    )


@app.post("/stream/end/{camera_name}")
async def end_stream(camera_name: str) -> Response:
    """
    End a video a stream given a camera name
    """
    if camera_manager.camera_is_running(camera_name):
        camera_manager.end_video_capture(camera_name)
        return Response(status_code=200)
    else:
        raise HTTPException(
            status_code=400, detail=f"{camera_name} is already not streaming"
        )


@app.get("/stream/available_cameras")
async def get_available_cameras() -> JSONResponse:
    """
    Get list of cameras that aren't currently streaming
    """
    available_cameras = camera_manager.get_available_cameras()
    return JSONResponse(status_code=200, content=available_cameras)


@app.post("/stream/fps/{camera_name}")
async def set_camera_fps(camera_name: str, fps: int) -> Response:
    """
    Set the fps on a camera given a camera name
    """
    try:
        camera_manager.set_camera_fps(camera_name, fps)
    except CameraNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    return Response(status_code=200)


@app.get("/stream/fps/{camera_name}")
async def get_camera_fps(camera_name: str) -> Response:
    """
    Get the fps on a camera given a camera name
    """
    try:
        fps = camera_manager.get_camera_fps(camera_name)
    except CameraNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    return Response(status_code=200, content=fps)


@app.post("/stream/encoding_quality/{camera_name}")
async def set_camera_encoding_quality(
    camera_name: str, encoding_quality: int
) -> Response:
    """
    Change the JPEG image quality (a value from 0-100) given a camera name
    """
    try:
        camera_manager.set_camera_encoding_params(camera_name, encoding_quality)
    except CameraNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    return Response(status_code=200)
