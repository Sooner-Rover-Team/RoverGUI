import { useState, useEffect, useRef } from "react";
import "./App.css";
import CameraGrid, { CameraContainer } from "./CameraGrid";

//filepath for testing (DELETE LATER): ../../../GitHub/Automomous/examples/ARTrackerTest/videos
function App() {
  const [fpsSlider, setFpsSlider] = useState<number>(50); // Initial fps slider value
  const [resolutionSlider, setResolutionSlider] = useState<number>(50); // Initial resolution slider value

  //Need to create a selection of camera names to choose from, and then pass that camera name
  //to the image source to get the video feed from the server

  const [cameras, setCameras] = useState<string[] | null>([]); //getting available devices from server

  const [cameraConnections, setCameraConnections] = useState<Map<string, RTCPeerConnection>>(new Map());
  const videoElementsRef = useRef<Map<string, HTMLVideoElement>>(new Map());

  // Fetch Camera(s) Information from Server
  useEffect(() => {
    (async () => {
      const response = await fetch("/stream/cameras");
      const cameras = await response.json();

      console.log(`Found cameras: ${cameras}`);

      setCameras(["", ...cameras]);
    })();
  }, []);

  // //test data for camera names
  // useEffect(() => {
  //   setCameras([null, 'Camera 1', 'Camera 2', 'Camera 3'])
  // }, [])

  //On change of the camera selection, add components for camera feed and sliders
  //to control the camera feed
  const [selectedCamera, setSelectedCamera] = useState("");

  const [cameraContainers, setCameraContainers] = useState<CameraContainer[]>([
    { id: '1', name: 'Camera 1', size: 'large', connection: null, videoElement: null },
    { id: '2', name: 'Camera 2', size: 'large', connection: null, videoElement: null },
    { id: '3', name: 'Camera 3', size: 'small', connection: null, videoElement: null },
    { id: '4', name: 'Camera 4', size: 'small', connection: null, videoElement: null },
    { id: '5', name: 'Camera 5', size: 'small', connection: null, videoElement: null },
  ])

  const handleCameraChange = async (
    event: React.ChangeEvent<HTMLSelectElement>,
  ) => {
    const selectedCameraPath = event.target.value;
    console.info(
      `stream: camera selection changed to: \`${selectedCameraPath}\``,
    );

    setSelectedCamera(selectedCameraPath);
  };

  const handleAddCamera = async (event: React.MouseEvent) => {

    if (selectedCamera === "") {
      console.warn("stream: no camera selected; cannot add camera.");
      return;
    }

    if (cameraConnections.has(selectedCamera)) {
      console.warn(
        "stream: camera already has an active connection; cannot add camera.",
        selectedCamera,
      );
      return;
    }

    const cameraId = selectedCamera;

    // Find the first available container
    const availableContainer = cameraContainers.find(
      (container) => container.videoElement === null && container.connection === null
    );
 
    if (!availableContainer) {
      console.warn("stream: no available containers for camera");
      return;
    }

    const peerConnection = new RTCPeerConnection();

    peerConnection.onconnectionstatechange = () => {
      console.info("stream: peer connection change", {
        cameraId,
        state: peerConnection.connectionState,
      });
    };

    peerConnection.oniceconnectionstatechange = () => {
      console.info("stream: ice connection state changed", {
        cameraId,
        state: peerConnection.iceConnectionState,
      });
    };

    peerConnection.ontrack = (e) => {
      console.debug("stream: received track event on peer connection", {
        cameraId,
        trackKind: e.track.kind,
        streamIds: e.streams.map((s) => s.id),
      }
      );

      if(e.track.kind == "video" && e.streams.length > 0) {
        const videoEl = document.createElement("video");
        videoEl.autoplay = true;
        videoEl.playsInline = true;
        videoEl.controls = true;
        videoEl.srcObject = e.streams[0] ?? null;

        if (videoEl.srcObject === null) {
          console.error(
            "stream: video `src` was set to `null` for path: ",
            cameraId,
          );
        }

        setCameraContainers((prev) => {
          return prev.map((container) =>
            container.id === availableContainer.id
              ? {
                  ...container,
                  name: cameraId,
                  videoElement: videoEl,
                  connection: peerConnection,
                }
              : container
          );
        });

        // Store ref for cleanup
        videoElementsRef.current.set(cameraId, videoEl);

        console.debug("stream: created video element for track event", {
          cameraId,
          trackKind: e.track.kind,
          streamIds: e.streams.map((s) => s.id),
        });
      };
    };

    peerConnection.onicecandidate = async (e) => {
      console.debug("stream: ice candidate event", {
        cameraId,
        candidate: e.candidate,
      });

      // IMPORTANT: Calls to the API should only run after this point.

      /* API Requests Example
        // Get the current mode (ex. 1920x1080 @ 30fps)
        let modeResponse = await fetch(`/stream/cameras/${encodeURIComponent(selectedCameraPath)}/modes/current`);
        console.log(await modeResponse.text());

        // Get the possible modes for the camera (ex. { 0: "1920x1080 @ 30fps", .. })
        let modesResponse = await fetch(`/stream/cameras/${encodeURIComponent(selectedCameraPath)}/modes`);
        console.log(await modesResponse.json());

        // Set the current mode for the camera by the index found in the top api request
        let setResponse = await fetch(`/stream/cameras/${encodeURIComponent(selectedCameraPath)}/modes/set/${1}`, { method: "PUT" });
        console.log(setResponse.status);
      */
    };

    peerConnection.addTransceiver("video", { direction: "sendrecv" });
    peerConnection.addTransceiver("audio", { direction: "sendrecv" });

    try {
      const offer = await peerConnection.createOffer();
      await peerConnection.setLocalDescription(offer);

      const response = await fetch(
        `/stream/cameras/${encodeURIComponent(cameraId)}/start`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(peerConnection.localDescription),
        },
      );
      if (!response.ok) {
        throw new Error(
          `failed to start stream! http err: ${response.status}`,
        );
      }
      const remoteOffer = await response.json();
      await peerConnection.setRemoteDescription(
        new RTCSessionDescription(remoteOffer),
      );
      setCameraConnections((prev) => new Map(prev).set(cameraId, peerConnection));
    } catch (error) {
      console.error("stream: failed to do offer/local desc", {
        cameraId,
        error,
      });

      // Cleanup connection on failure
      peerConnection.close();

      videoElementsRef.current.delete(selectedCamera);

      setCameraContainers((prev) =>
        prev.map((c) =>
          c.name === selectedCamera
            ? { ...c, videoElement: null, connection: null }
            : c
        )
      );
    }
  };

  const handleRemoveCamera = () => {
    const connection = cameraConnections.get(selectedCamera);
    if (connection) {
      connection.close();
      const updatedConnections = new Map(cameraConnections);
      updatedConnections.delete(selectedCamera);
      setCameraConnections(updatedConnections);
    }
  };

  return (
    <div className="App">
      <div className="camera-select">
        <label htmlFor="cameraSelect"> Select Camera: </label>
        <select
          id="cameraSelect"
          value={selectedCamera}
          onChange={handleCameraChange}
        >
          {cameras?.map((camera, index) => (
            <option key={camera || `empty-${index}`} value={camera}>
              {camera}
            </option>
          ))}
        </select>
        {selectedCamera && <button onClick={handleAddCamera}>+ Add Camera</button>}
      </div>
      <div className="camera-content">
        <div className="camera-grid">
          <CameraGrid cameras={cameraContainers} onRemoveCamera={handleRemoveCamera} connections={cameraConnections} />
          {/* <div id="videoDiv" ref={videoDivRef} /> */}
        </div>
  
        {selectedCamera && (
          <div className="slider-container">
            <label htmlFor="fpsSlider"> FPS: </label>
            <input
              id="fpsSlider"
              type="range"
              min="0"
              max="100"
              value={fpsSlider}
              onChange={(e) => setFpsSlider(Number(e.target.value))}
            />
  
            <label htmlFor="resolutionSlider"> Resolution: </label>
            <input
              id="resolutionSlider"
              type="range"
              min="0"
              max="100"
              value={resolutionSlider}
              onChange={(e) => setResolutionSlider(Number(e.target.value))}
            />
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
