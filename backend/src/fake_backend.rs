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
    use std::{collections::HashMap, io::Cursor, path::PathBuf, sync::Arc, time::Duration};

    use rocket::{get, http::Status, post, put, serde::json::Json, tokio::time, State};
    use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
    use webrtc::{
        api::{
            interceptor_registry,
            media_engine::{MediaEngine, MIME_TYPE_H264},
            APIBuilder,
        },
        interceptor::registry::Registry,
        media::Sample,
        peer_connection::configuration::RTCConfiguration,
        rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
        track::track_local::{track_local_static_sample::TrackLocalStaticSample, TrackLocal},
    };

    use crate::AppState;

    struct FakeVideoSample {
        bytes: Vec<u8>,
        duration: Duration,
    }

    struct Mp4BitstreamConverter {
        length_size: u8,
        sps: Vec<Vec<u8>>,
        pps: Vec<Vec<u8>>,
    }

    impl Mp4BitstreamConverter {
        fn for_mp4_track(track: &mp4::Mp4Track) -> Result<Self, Status> {
            let avcc = &track
                .trak
                .mdia
                .minf
                .stbl
                .stsd
                .avc1
                .as_ref()
                .ok_or(Status::InternalServerError)?
                .avcc;

            Ok(Self {
                length_size: avcc.length_size_minus_one + 1,
                sps: avcc
                    .sequence_parameter_sets
                    .iter()
                    .map(|v| v.bytes.clone())
                    .collect(),
                pps: avcc
                    .picture_parameter_sets
                    .iter()
                    .map(|v| v.bytes.clone())
                    .collect(),
            })
        }

        fn convert_packet(&self, packet: &[u8], out: &mut Vec<u8>) {
            out.clear();
            let mut stream = packet;
            let mut should_prefix_sps_pps = false;

            while !stream.is_empty() {
                let mut nal_size: usize = 0;
                for _ in 0..self.length_size {
                    if stream.is_empty() {
                        return;
                    }
                    nal_size = (nal_size << 8) | usize::from(stream[0]);
                    stream = &stream[1..];
                }

                if nal_size == 0 || stream.len() < nal_size {
                    return;
                }

                let nal = &stream[..nal_size];
                stream = &stream[nal_size..];

                if !nal.is_empty() && (nal[0] & 0x1F) == 5 {
                    should_prefix_sps_pps = true;
                }

                out.extend([0, 0, 0, 1]);
                out.extend(nal);
            }

            if should_prefix_sps_pps {
                let mut with_params = Vec::with_capacity(out.len() + 128);
                for sps in &self.sps {
                    with_params.extend([0, 0, 0, 1]);
                    with_params.extend(sps);
                }
                for pps in &self.pps {
                    with_params.extend([0, 0, 0, 1]);
                    with_params.extend(pps);
                }
                with_params.extend(out.iter().copied());
                *out = with_params;
            }
        }
    }

    fn load_fake_video_samples(path: PathBuf) -> Result<Vec<FakeVideoSample>, Status> {
        let mp4_bytes = std::fs::read(path).map_err(|_| Status::InternalServerError)?;
        let mut mp4_reader =
            mp4::Mp4Reader::read_header(Cursor::new(&mp4_bytes), mp4_bytes.len() as u64)
                .map_err(|_| Status::InternalServerError)?;

        let (_, track) = mp4_reader
            .tracks()
            .iter()
            .find(|(_, t)| t.media_type().ok() == Some(mp4::MediaType::H264))
            .ok_or(Status::InternalServerError)?;

        let track_id = track.track_id();
        let timescale = track.timescale();
        let sample_count = track.sample_count();
        let converter = Mp4BitstreamConverter::for_mp4_track(track)?;
        let mut converted = Vec::new();
        let mut out = Vec::new();

        for i in 1..=sample_count {
            let Some(sample) = mp4_reader
                .read_sample(track_id, i)
                .map_err(|_| Status::InternalServerError)?
            else {
                continue;
            };

            converter.convert_packet(&sample.bytes, &mut out);
            if out.is_empty() {
                continue;
            }

            let duration_ms = ((u64::from(sample.duration) * 1_000) / u64::from(timescale)).max(1);
            converted.push(FakeVideoSample {
                bytes: out.clone(),
                duration: Duration::from_millis(duration_ms),
            });
        }

        if converted.is_empty() {
            return Err(Status::InternalServerError);
        }

        Ok(converted)
    }

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
        // warning: slop code below

        if state
            .cameras
            .lock()
            .expect("not poisoned")
            .iter()
            .all(|c| c.path.to_string_lossy() != camera_path)
        {
            return Err(Status::NotFound);
        }

        let mut media_engine = MediaEngine::default();
        media_engine
            .register_default_codecs()
            .map_err(|_| Status::InternalServerError)?;
        let mut registry = Registry::new();
        registry = interceptor_registry::register_default_interceptors(registry, &mut media_engine)
            .map_err(|_| Status::InternalServerError)?;
        let rtc_api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();

        let peer_connection = Arc::new(
            rtc_api
                .new_peer_connection(RTCConfiguration::default())
                .await
                .map_err(|_| Status::InternalServerError)?,
        );
        let video_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                ..Default::default()
            },
            "video".to_owned(),
            "webrtc".to_owned(),
        ));
        let rtp_sender = peer_connection
            .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await
            .map_err(|_| Status::InternalServerError)?;

        rocket::tokio::spawn(async move {
            let mut rtcp_buf = vec![0u8; 1500];
            while rtp_sender.read(&mut rtcp_buf).await.is_ok() {}
        });

        let samples = load_fake_video_samples(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fake_stream.mp4"),
        )?;
        let notify_tx = Arc::new(rocket::tokio::sync::Notify::new());
        let notify_rx = notify_tx.clone();

        rocket::tokio::spawn(async move {
            notify_rx.notified().await;
            loop {
                for sample in &samples {
                    if video_track
                        .write_sample(&Sample {
                            data: sample.bytes.clone().into(),
                            duration: sample.duration,
                            ..Default::default()
                        })
                        .await
                        .is_err()
                    {
                        return;
                    }
                    time::sleep(sample.duration).await;
                }
            }
        });

        peer_connection.on_ice_connection_state_change(Box::new(
            move |connection_state: webrtc::ice_transport::ice_connection_state::RTCIceConnectionState| {
                if connection_state
                    == webrtc::ice_transport::ice_connection_state::RTCIceConnectionState::Connected
                {
                    notify_tx.notify_waiters();
                }

                Box::pin(async {})
            },
        ));

        peer_connection
            .set_remote_description(offer.into_inner())
            .await
            .map_err(|_| Status::InternalServerError)?;
        let answer = peer_connection
            .create_answer(None)
            .await
            .map_err(|_| Status::InternalServerError)?;
        let mut ice_gather_rx = peer_connection.gathering_complete_promise().await;
        peer_connection
            .set_local_description(answer)
            .await
            .map_err(|_| Status::InternalServerError)?;
        ice_gather_rx.recv().await;

        Ok(Json(
            peer_connection
                .local_description()
                .await
                .ok_or(Status::InternalServerError)?,
        ))
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
