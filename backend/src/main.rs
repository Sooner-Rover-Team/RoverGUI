use std::{collections::HashMap, error::Error, path::PathBuf};

use rocket::{get, http::Status, post, put, routes, serde::json::Json, Config, State};
use utils::{CameraMode, H264CameraReader, WebcamManager};
use v4l::Device;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

mod utils;

// Fetch all the available v4l cameras in the system
#[get("/cameras")]
async fn get_available_cameras(
    state: &State<AppState>,
) -> Result<Json<Vec<PathBuf>>, (Status, &'static str)> {
    Ok(Json(state.available_camera_paths.clone()))
}

// Start a WebRTC stream by creating and returning an offer
#[post("/cameras/<camera_path>/start", data = "<offer>")]
async fn get_camera_feed(
    camera_path: &str,
    offer: Json<RTCSessionDescription>,
    state: &State<AppState>,
) -> Result<Json<RTCSessionDescription>, Status> {
    let webcam_manager = &state.webcam_manager;
    let local_description = webcam_manager
        .add_client(camera_path.to_owned(), offer.into_inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(Json(local_description))
}

// Get the current camera mode
#[get("/cameras/<camera_path>/modes/current")]
async fn get_camera_mode(
    camera_path: &str,
    state: &State<AppState>,
) -> Result<String, (Status, &'static str)> {
    let camera_handles_mutex = state.webcam_manager.camera_handles();
    let camera_handles = &mut *camera_handles_mutex.lock().await;
    let handle = camera_handles
        .iter()
        .find(|handle| handle.camera_path() == camera_path)
        .ok_or((Status::BadRequest, "Invalid / Inactive Camera Path"))?;

    Ok(handle.current_mode().to_string())
}

// Get all the available camera modes
#[get("/cameras/<camera_path>/modes")]
async fn get_camera_modes(
    camera_path: &str,
    state: &State<AppState>,
) -> Result<Json<HashMap<usize, String>>, (Status, &'static str)> {
    let camera_handles_mutex = state.webcam_manager.camera_handles();
    let camera_handles = &mut *camera_handles_mutex.lock().await;
    let handle = camera_handles
        .iter()
        .find(|handle| handle.camera_path() == camera_path)
        .ok_or((Status::BadRequest, "Invalid / Inactive Camera Path"))?;

    // Using a HashMap to make sure that all modes stay in their exact order represented in the Vec<CameraMode> of the handle.
    let mut mapped_modes: HashMap<usize, String> = HashMap::new();
    for i in 0..handle.camera_modes().len() {
        let mode = handle.camera_modes()[i];
        mapped_modes.insert(i, mode.to_string());
    }

    Ok(Json(mapped_modes))
}

// Set the current camera mode for the camera path
#[put("/cameras/<camera_path>/modes/set/<mode_id>")]
async fn put_camera_mode_set(
    camera_path: &str,
    mode_id: usize,
    state: &State<AppState>,
) -> Result<(), (Status, &'static str)> {
    let camera_handles_mutex = state.webcam_manager.camera_handles();
    let camera_handles = &mut *camera_handles_mutex.lock().await;
    let handle = camera_handles
        .iter_mut()
        .find(|handle| handle.camera_path() == camera_path)
        .ok_or((Status::BadRequest, "Invalid / Inactive Camera Path"))?;
    handle.update_camera_mode(mode_id).await.map_err(|_| {
        (
            Status::BadRequest,
            "Failed to Update Camera Mode; index may be invalid",
        )
    })?;

    Ok(())
}

struct AppState {
    webcam_manager: WebcamManager,
    available_camera_paths: Vec<PathBuf>,
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut available_camera_paths: Vec<PathBuf> = Vec::new();

    // Only add cameras that are streamable with h264.
    // available_camera_paths is only updated at initial start, however, this could be better changed to run when a camera is plugged in or removed.
    for node in v4l::context::enum_devices() {
        let mut device = Device::new(node.index())?;
        let Ok(modes) = CameraMode::fetch_all(&device) else {
            continue;
        };
        let initial_mode = *modes.first().ok_or("Error Creating a CameraThreadHandle: Failed to initialize camera as no valid Camera operating modes were provided by video4linux. (Check the camera as this was an OS-level issue!)")?;

        // If can't query or create a stream, then it can't be displayed.
        if H264CameraReader::new(&mut device, initial_mode).is_ok() {
            available_camera_paths.push(node.path().to_path_buf());
        }
    }

    // Create a Rocket instance with the default configuration.
    rocket::build()
        // This is arbitrary and can be changed at any time through a config file or it can be left hardcoded.
        .configure(Config::figment().merge(("port", 3600)))
        .mount(
            "/stream",
            routes![
                get_available_cameras,
                get_camera_feed,
                get_camera_mode,
                get_camera_modes,
                put_camera_mode_set
            ],
        )
        .manage(AppState {
            webcam_manager: WebcamManager::new().unwrap(),
            available_camera_paths,
        })
        .launch()
        .await?;

    Ok(())
}
