import { useState, useEffect, useRef } from "react";
import "./App.css";

//filepath for testing (DELETE LATER): ../../../GitHub/Automomous/examples/ARTrackerTest/videos
function App() {
  const [fpsSlider, setFpsSlider] = useState<number>(50); // Initial fps slider value
  const [resolutionSlider, setResolutionSlider] = useState<number>(50); // Initial resolution slider value

  //Need to create a selection of camera names to choose from, and then pass that camera name
  //to the image source to get the video feed from the server

  const [cameras, setCameras] = useState<string[] | null>([]); //getting available devices from server

  const connection = useRef<RTCPeerConnection | null>(null);

  /*//Effect to get the camera names from the server
  useEffect(() => {
    fetch('/video_feed/available_devices')
      .then(response => response.json())
      .then(data => {
        updateCameraNames(data)
      })
  }, [])*/

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

  const handleCameraChange = async (
    event: React.ChangeEvent<HTMLSelectElement>,
  ) => {
    const selectedCameraPath = event.target.value;
    console.info(
      `stream: camera selection changed to: \`${selectedCameraPath}\``,
    );

    if (connection.current !== null) {
      console.log("stream: closing current connection.", selectedCameraPath);
      connection.current.close();
    }

    if (selectedCameraPath === "") {
      console.log(
        "stream: cannot stream for empty path. early returning...",
        selectedCameraPath,
      );
      return;
    }

    const peerConnection = new RTCPeerConnection();
    peerConnection.onconnectionstatechange = () => {
      console.info("stream: peer connection change", {
        selectedCameraPath,
        state: peerConnection.connectionState,
      });
    };

    peerConnection.oniceconnectionstatechange = () => {
      console.info("stream: ice connection state changed", {
        selectedCameraPath,
        state: peerConnection.iceConnectionState,
      });
    };

    peerConnection.ontrack = (e) => {
      const el = document.createElement(e.track.kind) as HTMLMediaElement;
      el.srcObject = e.streams[0] ?? null;
      if (el.srcObject === null) {
        console.error(
          "stream: video `src` was set to `null` for path: ",
          selectedCameraPath,
        );
      }

      el.autoplay = true;
      el.controls = true;
      el.onerror = (error) => {
        console.error("stream: html media element err: ", {
          selectedCameraPath,
          kind: e.track.kind,
          error,
          mediaError: el.error,
        });
      };

      document.getElementById("videoDiv")?.appendChild(el);
    };

    peerConnection.onicecandidate = async (e) => {
      if (e.candidate === null || connection.current !== null) return;
      connection.current = peerConnection;

      try {
        const response = await fetch(
          `/stream/cameras/${encodeURIComponent(selectedCameraPath)}/start`,
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

        setSelectedCamera(selectedCameraPath);
      } catch (error) {
        console.error("stream: failed to create rtc stream session", {
          selectedCameraPath,
          error,
        });
      }

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
    } catch (error) {
      console.error("stream: failed to do offer/local desc", {
        selectedCameraPath,
        error,
      });
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
          {// TODO: Add an error display if cameras is null
          cameras?.map((camera, index) => {
            return (
              <option key={camera || `empty-${index}`} value={camera}>
                {camera}
              </option>
            );
          })}
        </select>
        {selectedCamera && (
          <div>
            <div id="videoDiv" />
            {/* <div className="camera-feed">
            <img src={`/stream/video_feed/${selectedCamera}`} alt="Camera Frame" width="600" height="400" />
          </div> */}
            <div className="slider-container">
              <label htmlFor="fpsSlider"> FPS: </label>
              <input
                id="fpsSlider"
                type="range"
                min="0"
                max="100"
                value={fpsSlider}
                onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
                  setFpsSlider(Number(event.target.value))
                }
              />
              <label htmlFor="resolutionSlider"> Resolution: </label>
              <input
                id="resolutionSlider"
                type="range"
                min="0"
                max="100"
                value={resolutionSlider}
                onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
                  setResolutionSlider(Number(event.target.value))
                }
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
