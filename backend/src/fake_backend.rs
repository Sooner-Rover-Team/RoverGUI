use std::{
    env::Args,
    error::Error,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use rocket::{routes, Config};

use crate::utils::{CameraMode, FakeCamera, Fraction};

struct AppState {
    cameras: Arc<Mutex<Vec<FakeCamera>>>,
}

mod utils {
    use std::path::PathBuf;

    #[derive(Debug, Clone, Copy)]
    pub struct CameraMode {
        pub width: u32,
        pub height: u32,
        pub frame_interval: Fraction,
    }

    impl core::fmt::Display for CameraMode {
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            write!(
                f,
                "{}x{} @{}fps",
                self.width, self.height, self.frame_interval.denominator
            )
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Fraction {
        #[expect(unused, reason = "buggy FPS impl in `Display`")]
        pub numerator: u32,
        pub denominator: u32,
    }

    /// A fake camera. Imitates a V4L2 camera.
    pub struct FakeCamera {
        /// The camera's capture settings.
        pub camera_mode: CameraMode,

        /// A list of all supported camera modes for this device.
        pub supported_camera_modes: Vec<CameraMode>,

        /// File representation on disk.
        ///
        /// Could become stale if unplugged.
        pub path: PathBuf,
    }
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let num_cameras: u8 = {
        // ensure args given correctly
        let mut args: Args = std::env::args();
        if args.len() != 2 {
            panic!(
                "You ran this script with {} argument(s)! \
                Please run like so: \
                `cargo run --bin fake_backend -- {{NUM_CAMERAS}}`",
                args.len()
            );
        }

        // skip binary name
        _ = args.next();

        // grab number of cameras
        str::parse(
            args.next()
                .expect("one argument found but couldn't access it")
                .as_str(),
        )
        .expect("number of cameras should be a value in [0, 255].")
    };

    // initialize the fake cameras
    let fake_cameras: Vec<FakeCamera> = Vec::from_iter((0..num_cameras).map(|n| {
        let camera_mode: CameraMode = CameraMode {
            width: 1_000 * (n + 1) as u32,
            height: 2_000 * (n + 1) as u32,
            frame_interval: Fraction {
                numerator: 1,
                denominator: (n + 1) as u32,
            },
        };

        FakeCamera {
            camera_mode,
            supported_camera_modes: vec![camera_mode],
            path: PathBuf::from(format!("/fake/camera{n}")),
        }
    }));

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
            cameras: Arc::new(Mutex::new(fake_cameras)),
        })
        .launch()
        .await?;

    Ok(())
}

mod api {
    use std::{collections::HashMap, path::PathBuf};

    use rocket::{get, http::Status, post, put, serde::json::Json, State};
    use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

    use crate::AppState;

    // Fetch all the available v4l cameras in the system
    #[get("/cameras")]
    pub async fn get_available_cameras(
        state: &State<AppState>,
    ) -> Result<Json<Vec<PathBuf>>, (Status, &'static str)> {
        Ok(Json(
            state
                .cameras
                .lock()
                .expect("not poisoned")
                .iter()
                .map(|c| c.path.clone())
                .collect(),
        ))
    }

    // Start a WebRTC stream by creating and returning an offer
    #[post("/cameras/<camera_path>/start", data = "<offer>")]
    pub async fn get_camera_feed(
        camera_path: &str,
        offer: Json<RTCSessionDescription>,
        state: &State<AppState>,
    ) -> Result<Json<RTCSessionDescription>, Status> {
        todo!()
    }

    // Get the current camera mode
    #[get("/cameras/<camera_path>/modes/current")]
    pub async fn get_camera_mode(
        camera_path: &str,
        state: &State<AppState>,
    ) -> Result<String, (Status, &'static str)> {
        // grab the mutex
        let locked_cameras = state.cameras.lock().expect("not poisoned");

        // find the camera; error if we don't find it
        let Some(camera) = locked_cameras
            .iter()
            .find(|c| c.path.to_string_lossy() == camera_path)
        else {
            return Err((Status::NotFound, "Camera with given value not found"));
        };

        Ok(camera.camera_mode.to_string())
    }

    // Get all the available camera modes
    #[get("/cameras/<camera_path>/modes")]
    pub async fn get_camera_modes(
        camera_path: &str,
        state: &State<AppState>,
    ) -> Result<Json<HashMap<usize, String>>, (Status, &'static str)> {
        // grab the mutex
        let locked_cameras = state.cameras.lock().expect("not poisoned");

        // find the camera; error if we don't find it
        let Some(camera) = locked_cameras
            .iter()
            .find(|c| c.path.to_string_lossy() == camera_path)
        else {
            return Err((Status::NotFound, "Camera with given value not found"));
        };

        // list all the supported camera modes in a hashmap
        Ok(Json(
            camera
                .supported_camera_modes
                .iter()
                .enumerate()
                .map(|(i, camera_mode)| (i, camera_mode.to_string()))
                .collect(),
        ))
    }

    // Set the current camera mode for the camera path
    #[put("/cameras/<camera_path>/modes/set/<mode_id>")]
    pub async fn put_camera_mode_set(
        camera_path: &str,
        mode_id: usize,
        state: &State<AppState>,
    ) -> Result<(), (Status, &'static str)> {
        // grab the mutex
        let mut locked_cameras = state.cameras.lock().expect("not poisoned");

        // find the requested camera
        let Some(camera) = locked_cameras
            .iter_mut()
            .find(|c| c.path.to_string_lossy() == camera_path)
        else {
            return Err((Status::NotFound, "Camera with given value not found"));
        };

        // find the requested camera mode
        let Some(new_camera_mode) = camera.supported_camera_modes.get(mode_id) else {
            return Err((
                Status::NotFound,
                "Camera mode at given index was not found.",
            ));
        };

        // swap to the new mode
        camera.camera_mode = *new_camera_mode;

        Ok(())
    }
}
