use std::{
    error::Error,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self},
    time::Duration,
};

use jpeg_decoder::Decoder;
use openh264::{
    encoder::Encoder,
    formats::{RgbSliceU8, YUVBuffer},
};
use rocket::tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex, Notify,
    },
    task, time,
};
use v4l::{
    buffer::Type,
    frameinterval::{FrameIntervalEnum, Stepwise},
    io::traits::CaptureStream,
    prelude::MmapStream,
    video::{capture::Parameters, Capture},
    Device, Format, FourCC, Fraction,
};
use webrtc::{
    api::{
        interceptor_registry,
        media_engine::{MediaEngine, MIME_TYPE_H264},
        APIBuilder, API,
    },
    ice_transport::ice_connection_state::RTCIceConnectionState,
    interceptor::registry::Registry,
    media::Sample,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
    },
    rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
    track::track_local::{track_local_static_sample::TrackLocalStaticSample, TrackLocal},
};

// Struct used for reading a camera's MJPG frames to H264 buffers.
pub struct H264CameraReader<'a> {
    stream: MmapStream<'a>,
    camera_mode: CameraMode,
    h264_encoder: Encoder,
}

// Heavily inspired by https://github.com/D1plo1d/h264_webcam_stream
impl<'a> H264CameraReader<'a> {
    pub fn new(device: &mut Device, mode: CameraMode) -> Result<Self, Box<dyn Error>> {
        let format = Format::new(mode.width, mode.height, FourCC::new(b"MJPG"));
        Capture::set_format(device, &format)?;

        let parameters = Parameters::new(mode.frame_interval);
        Capture::set_params(device, &parameters)?;

        let stream = MmapStream::with_buffers(device, Type::VideoCapture, 4)?;
        let encoder = Encoder::new()?;

        Ok(Self {
            stream,
            camera_mode: mode,
            h264_encoder: encoder,
        })
    }

    pub fn read(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let (buffer, _) = self.stream.next()?;
        let mut jpg = Decoder::new(buffer);
        let pixels = jpg.decode()?;

        let yuv_buffer = YUVBuffer::from_rgb_source(RgbSliceU8::new(
            &pixels[..],
            (
                self.camera_mode.width as usize,
                self.camera_mode.height as usize,
            ),
        ));
        let bitstream = self.h264_encoder.encode(&yuv_buffer)?;

        Ok(bitstream.to_vec())
    }
}

// CameraMode(s) are used for controlling what resolution and frame rate the H264CameraReader operates at.
#[derive(Debug, Clone, Copy)]
pub struct CameraMode {
    width: u32,
    height: u32,
    frame_interval: Fraction, // 1/30 is 30 fps, 1/15 is 15fps, etc.
}

impl core::fmt::Display for CameraMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}x{} @{}fps",
            self.width, self.height, self.frame_interval.denominator
        )
    }
}

impl CameraMode {
    // Fetch all possible CameraMode(s) of the Device camera.
    pub fn fetch_all(device: &Device) -> Result<Vec<CameraMode>, Box<dyn Error>> {
        let mut configs: Vec<CameraMode> = Vec::new();

        let discrete_iter = Capture::enum_framesizes(device, FourCC::new(b"MJPG"))?
            .into_iter()
            .flat_map(|framesize| framesize.size.to_discrete());

        for discrete_frame_size in discrete_iter {
            for interval in Capture::enum_frameintervals(
                device,
                FourCC::new(b"MJPG"),
                discrete_frame_size.width,
                discrete_frame_size.height,
            )? {
                let fraction = match interval.interval {
                    FrameIntervalEnum::Discrete(fraction) => fraction,
                    FrameIntervalEnum::Stepwise(Stepwise { min, .. }) => min,
                };

                configs.push(CameraMode {
                    width: discrete_frame_size.width,
                    height: discrete_frame_size.height,
                    frame_interval: fraction,
                });
            }
        }

        Ok(configs)
    }
}

/// Handles communication between CameraThreadHandle(s) and WebRTC clients.
pub struct WebcamManager {
    camera_handles: Arc<Mutex<Vec<CameraThreadHandle>>>,
    rtc_api: API,
}

impl WebcamManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()?;

        let mut registry = Registry::new();
        registry =
            interceptor_registry::register_default_interceptors(registry, &mut media_engine)?;

        let rtc_api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();

        Ok(Self {
            camera_handles: Arc::new(Mutex::new(Vec::new())),
            rtc_api,
        })
    }

    pub fn camera_handles(&self) -> Arc<Mutex<Vec<CameraThreadHandle>>> {
        self.camera_handles.clone()
    }

    /// Add a WebRTC "Client". A client in this sense is simply a WebRTC Connection created from an RTC Offer or [`RTCSessionDescription`].
    /// This connection serves an H264 encoded stream from the corresponding camera to the `camera_path` argument. "Client" is just used here
    /// to describe the connection itself and all its respective mpsc channels and tokio tasks needed to serve the required stream.
    /// **IMPORTANT: Everything in here (within the client) runs within the tokio runtime!**
    pub async fn add_client(
        &self,
        camera_path: String,
        rtc_offer: RTCSessionDescription,
    ) -> Result<RTCSessionDescription, Box<dyn Error>> {
        let peer_connection = self
            .rtc_api
            .new_peer_connection(RTCConfiguration::default())
            .await?;
        let peer_connection = Arc::new(peer_connection);
        let peer_disconnected = Arc::new(AtomicBool::new(false));
        let notify_tx = Arc::new(Notify::new());
        let notify_rx = notify_tx.clone();

        let video_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                ..Default::default()
            },
            // These are arbitrary values. I'm sure that in WebRTC scenarios that require more defined control, the id and stream_id would be important.
            // However, this is not the case as we are simply serving a video stream!
            "video".to_owned(),
            "webrtc".to_owned(),
        ));

        let rtp_sender = peer_connection
            .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await?;

        // Get a Receiver from the CameraThreadHandle thread. Additionally, create a CameraThreadHandle if one isn't active for the current path.
        let mut rx = {
            let camera_handles = &mut self.camera_handles.lock().await;
            match camera_handles
                .iter()
                .find(|handle| handle.camera_path == camera_path)
            {
                Some(handle) => handle.enroll_rx().await,
                None => {
                    let handle = CameraThreadHandle::start_camera_thread(
                        &camera_path,
                        self.camera_handles.clone(),
                    )
                    .unwrap();
                    let rx = handle.enroll_rx().await;

                    camera_handles.push(handle);
                    rx
                }
            }
        };

        // Should drop if connection is canceled
        task::spawn(async move {
            let mut rtcp_buf = vec![0u8; 1500];
            while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
        });

        let c_peer_disconnected = peer_disconnected.clone();
        let c_peer_connection = peer_connection.clone();
        task::spawn(async move {
            // We must wait for the ice connection status state to be Connected before sending bytes to the WebRTC client.
            if time::timeout(Duration::from_millis(5000), notify_rx.notified())
                .await
                .is_err()
            {
                return;
            }

            // Read h264 bytes from the CameraThreadHandle using mpsc.
            while let Some(bytes) = rx.recv().await {
                if c_peer_disconnected.load(Ordering::SeqCst) {
                    let _ = c_peer_connection.close().await;
                    return;
                }

                video_track
                    .write_sample(&Sample {
                        data: bytes.into(),
                        duration: Duration::from_millis(20),
                        ..Default::default()
                    })
                    .await
                    .unwrap();
            }
        });

        peer_connection.on_ice_connection_state_change(Box::new(
            move |connection_state: RTCIceConnectionState| {
                if connection_state == RTCIceConnectionState::Connected {
                    notify_tx.notify_waiters();
                }

                Box::pin(async {})
            },
        ));

        peer_connection.on_peer_connection_state_change(Box::new(
            move |state: RTCPeerConnectionState| {
                if state == RTCPeerConnectionState::Disconnected {
                    peer_disconnected.store(true, Ordering::SeqCst);
                }

                Box::pin(async {})
            },
        ));

        // WebRTC connection processes
        peer_connection.set_remote_description(rtc_offer).await?;
        let answer = peer_connection.create_answer(None).await?;
        let mut ice_gather_rx = peer_connection.gathering_complete_promise().await;
        peer_connection.set_local_description(answer).await?;
        ice_gather_rx.recv().await;

        Ok(peer_connection
            .local_description()
            .await
            .ok_or("Failed to Generate Description")?)
    }
}

/*
    How CameraThreadHandle Works:

    This struct manages a "reader" thread that owns an H264CameraReader. Importantly, this struct sends WebRTC client Receivers
    to the reader thread. The reader thread then constantly loops and reads H264 bytes from H264CameraReader and then iterates
    through all the WebRTC client Receivers, sending each receiver a copy of the bytes read. If there aren't any more Receivers,
    then the reader thread is dropped and CameraThreadHandle is removed from WebcamManager.

    This could be thought of as an "event system" in which all the WebRTC clients on the tokio runtime receiver bytes from the reader thread.
*/

pub struct CameraThreadHandle {
    camera_path: String,
    cam_mode_tx: Sender<CameraMode>,
    current_mode: CameraMode,
    camera_modes: Vec<CameraMode>,
    manual_shutdown_needed: Arc<AtomicBool>,
    sink_flush_needed: Arc<AtomicBool>,
    tx_sink: Arc<Mutex<Vec<Sender<Vec<u8>>>>>,
}

impl CameraThreadHandle {
    fn start_camera_thread(
        camera_path: &str,
        camera_handles: Arc<Mutex<Vec<CameraThreadHandle>>>,
    ) -> Result<Self, Box<dyn Error>> {
        let camera_path = camera_path.to_owned();
        let (cam_mode_tx, mut cam_mode_rx) = mpsc::channel::<CameraMode>(1);
        let manual_shutdown_needed: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let sink_flush_needed: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let tx_sink: Arc<Mutex<Vec<Sender<Vec<u8>>>>> = Arc::new(Mutex::new(Vec::new()));

        let c_camera_path = camera_path.clone();
        let c_manual_shutdown_needed = manual_shutdown_needed.clone();
        let c_sink_flush_needed = sink_flush_needed.clone();
        let c_tx_sink = tx_sink.clone();

        let device_path = Path::new(&c_camera_path);
        let node = v4l::context::enum_devices()
            .into_iter()
            .find(|node| node.path() == device_path)
            .ok_or("V4l Node Not Found")?;
        let mut device = Device::new(node.index())?;
        let modes = CameraMode::fetch_all(&device)?;
        let initial_mode = *modes.last().ok_or(
            "Error creating a CameraThreadHandle: \
                Failed to initialize camera, as no valid camera operating \
                modes were provided by Video4Linux. \
                (Check the camera, as this was an OS-level issue!)",
        )?; // The last camera mode tends to be the one with the best resolution and fps.
        thread::spawn(move || {
            let mut reader = H264CameraReader::new(&mut device, initial_mode).unwrap();
            let mut rtc_txs: Vec<Sender<Vec<u8>>> = Vec::new();

            // Run until the CameraThreadHandle has been dropped from memory.
            while !c_manual_shutdown_needed.load(Ordering::SeqCst) {
                // Used to prevent having to constantly lock the c_tx_sink mutex.
                if c_sink_flush_needed.load(Ordering::SeqCst) {
                    let tx_sink = &mut *c_tx_sink.blocking_lock();

                    while let Some(tx) = tx_sink.pop() {
                        rtc_txs.push(tx);
                    }

                    c_sink_flush_needed.store(false, Ordering::SeqCst);
                }

                // Restart reader / stream with the new mode
                if let Ok(mode) = cam_mode_rx.try_recv() {
                    drop(reader);
                    reader = H264CameraReader::new(&mut device, mode).unwrap();
                }

                let bytes = reader.read().unwrap();
                rtc_txs.retain(|tx| tx.blocking_send(bytes.clone()).is_ok());

                if rtc_txs.is_empty() {
                    let camera_handles = &mut *camera_handles.blocking_lock();
                    camera_handles.retain(|handle| handle.camera_path != c_camera_path);

                    return;
                }
            }
        });

        Ok(CameraThreadHandle {
            camera_path,
            manual_shutdown_needed,
            cam_mode_tx,
            current_mode: *modes.last().ok_or(
                "Error creating a CameraThreadHandle: \
                    Failed to initialize camera, as no valid camera operating \
                    modes were provided by Video4Linux. \
                    (Check the camera, as this was an OS-level issue!)",
            )?,
            camera_modes: modes,
            sink_flush_needed,
            tx_sink,
        })
    }

    pub async fn update_camera_mode(&mut self, mode_index: usize) -> Result<(), Box<dyn Error>> {
        let mode = self
            .camera_modes
            .get(mode_index)
            .ok_or("Invalid Mode Index")?;
        self.cam_mode_tx.send(*mode).await?;

        Ok(())
    }

    pub fn camera_path(&self) -> &str {
        &self.camera_path
    }

    pub fn current_mode(&self) -> &CameraMode {
        &self.current_mode
    }

    pub fn camera_modes(&self) -> &[CameraMode] {
        &self.camera_modes
    }

    async fn enroll_rx(&self) -> Receiver<Vec<u8>> {
        let (tx, rx) = mpsc::channel::<Vec<u8>>(1);
        let tx_sink = &mut *self.tx_sink.lock().await;

        tx_sink.push(tx);
        self.sink_flush_needed.store(true, Ordering::SeqCst);

        rx
    }
}

// Signal the reader thread to stop running after drop.
impl Drop for CameraThreadHandle {
    fn drop(&mut self) {
        self.manual_shutdown_needed.store(true, Ordering::SeqCst);
    }
}
