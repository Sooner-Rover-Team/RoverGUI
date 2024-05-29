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

@app.get("/hello")
async def hello():
    return "Sup from backend"


@app.get("/stream/start/{camera_name}")
async def start_stream(camera_name: str) -> StreamingResponse:
    """
    Take in a camera name and start a video stream
    """
    try:
        camera = camera_manager.get_camera(camera_name)
        # If camera is already streaming, return bad response
        if camera.is_streaming:
            raise HTTPException(
                status_code=400, detail=f"{camera_name} is already streaming"
            )
        # Create video capture object for camera
        cap = cv2.VideoCapture(camera_manager.camera_path_from_name(camera_name))
        cap.set(cv2.CAP_PROP_BUFFERSIZE, 1)  # don't store extra frames
        camera.start(cap)

    except CameraNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    # Return streaming response
    return StreamingResponse(
        camera.stream(),
        media_type="multipart/x-mixed-replace; boundary=frame",
    )


@app.post("/stream/end/{camera_name}")
async def end_stream(camera_name: str) -> Response:
    """
    End a video a stream given a camera name
    """
    try:
        camera = camera_manager.get_camera(camera_name)
    except CameraNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))

    if camera.is_streaming:
        camera.end_stream()
        return Response(status_code=200)
    else:
        raise HTTPException(
            status_code=400, detail=f"{camera_name} is not currently streaming"
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
        camera_manager.get_camera(camera_name).set_fps(fps)
    except CameraNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    return Response(status_code=200)


@app.get("/stream/fps/{camera_name}")
async def get_camera_fps(camera_name: str) -> Response:
    """
    Get the fps on a camera given a camera name
    """
    try:
        fps = camera_manager.get_camera(camera_name).fps
    except CameraNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    return Response(status_code=200, content=str(fps))


@app.post("/stream/resolution/{camera_name}")
async def set_camera_resolution(camera_name: str, vertical: int, horizontal: int) -> Response:
    """
    Set the resolution on a camera given a camera name
    """
    try:
        logger.info(f"Changing {camera_name} resolution")
        resolution = Resolution(horizontal=horizontal, vertical=vertical)
        camera_manager.get_camera(camera_name).set_current_resolution(resolution)
        logger.info(f"Changed {camera_name} resolution")
    except CameraNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    return Response(status_code=200)


@app.get("/stream/resolution/{camera_name}")
async def get_camera_resolution(camera_name: str) -> Response:
    """
    Get the resolution on a camera given a camera name
    """
    try:
        resolution = camera_manager.get_camera(camera_name).resolution
    except CameraNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    return Response(status_code=200, content=str(resolution))


@app.post("/stream/image_quality/{camera_name}")
async def set_camera_image_quality(camera_name: str, quality: int) -> Response:
    """
    Change the JPEG image quality (a value from 0-100) given a camera name
    """
    try:
        camera_manager.get_camera(camera_name).set_image_quality(quality)
    except CameraNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    return Response(status_code=200)


@app.get("/stream/image_quality/{camera_name}")
async def get_camera_image_quality(camera_name: str) -> Response:
    """
    Get the image quality on a camera given a camera name
    """
    try:
        quality = camera_manager.get_camera(camera_name).image_quality
    except CameraNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    return Response(status_code=200, content=str(quality))
