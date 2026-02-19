#[cfg(target_os = "linux")]
mod utils;

#[cfg(target_os = "linux")]
#[rocket::main]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
    use std::path::PathBuf;

    use rocket::{routes, Config};
    use utils::{CameraMode, H264CameraReader, WebcamManager};
    use v4l::Device;

    use crate::api::AppState;

    let mut available_camera_paths: Vec<PathBuf> = Vec::new();

    // Only add cameras that are streamable with h264.
    // available_camera_paths is only updated at initial start, however, this could be better changed to run when a camera is plugged in or removed.
    for node in v4l::context::enum_devices() {
        let mut device = Device::new(node.index())?;
        let Ok(modes) = CameraMode::fetch_all(&device) else {
            continue;
        };
        let initial_mode: CameraMode = *modes.first().ok_or(
            "Error creating a `CameraThreadHandle`: \
            Failed to initialize camera, as no valid camera operating modes \
            were provided by Video4Linux. \
            (Check the camera, as this was an OS-level issue!)",
        )?;

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
                api::get_available_cameras,
                api::get_camera_feed,
                api::get_camera_mode,
                api::get_camera_modes,
                api::put_camera_mode_set
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

#[cfg(not(target_os = "linux"))]
fn main() -> Result<(), u8> {
    eprintln!(
        "The camera backend is unsupported on non-Linux machines. \
        For testing, please use the `fake_backend` instead. \
        Otherwise, please run the real backend on the Rover -- it isn't \
        supposed to run on your personal device!"
    );
    Err(1)
}

#[cfg(target_os = "linux")]
mod api {
    use std::{collections::HashMap, path::PathBuf};

    use super::utils::WebcamManager;
    use rocket::{get, http::Status, post, put, serde::json::Json, State};
    use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

    pub struct AppState {
        pub webcam_manager: WebcamManager,
        pub available_camera_paths: Vec<PathBuf>,
    }

    // Fetch all the available v4l cameras in the system
    #[get("/cameras")]
    pub async fn get_available_cameras(
        state: &State<AppState>,
    ) -> Result<Json<Vec<PathBuf>>, (Status, &'static str)> {
        Ok(Json(state.available_camera_paths.clone()))
    }

    // Start a WebRTC stream by creating and returning an offer
    #[post("/cameras/<camera_path>/start", data = "<offer>")]
    pub async fn get_camera_feed(
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
    pub async fn get_camera_mode(
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
    pub async fn get_camera_modes(
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
    pub async fn put_camera_mode_set(
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
}
