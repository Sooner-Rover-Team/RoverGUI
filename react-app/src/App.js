import React, { useState, useEffect, useRef } from 'react';
import './App.css';

//filepath for testing (DELETE LATER): ../../../GitHub/Automomous/examples/ARTrackerTest/videos
function App() {
  const [fpsSlider, setFpsSlider] = useState(50); // Initial fps slider value
  const [resolutionSlider, setResolutionSlider] = useState(50); // Initial resolution slider value

  //Need to create a selection of camera names to choose from, and then pass that camera name
  //to the image source to get the video feed from the server

  const [cameras, setCameras] = useState([]); //getting available devices from server

  const connection = useRef(null);

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
      let response = await fetch("/stream/cameras");
      let cameras = await response.json();

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

  const handleCameraChange = async (event) => {
    let selectedCameraPath = event.target.value;

    if (connection.current !== null)
      connection.current.close();

    if (selectedCameraPath === "")
      return;

    let peerConnection = new RTCPeerConnection();
    peerConnection.ontrack = (e) => {
      var el = document.createElement(e.track.kind);
      el.srcObject = e.streams[0];
      el.autoplay = true;
      el.controls = true;
    
      document.getElementById("videoDiv").appendChild(el);
    };

    peerConnection.onicecandidate = async (e) => {
      if (e.candidate === null || connection.current !== null)
          return;
      connection.current = peerConnection;

      let response = await fetch(`/stream/cameras/${encodeURIComponent(selectedCameraPath)}/start`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify(peerConnection.localDescription)
      });
      let remoteOffer = await response.json();
      peerConnection.setRemoteDescription(new RTCSessionDescription(remoteOffer));

      setSelectedCamera(selectedCameraPath);

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

    peerConnection.addTransceiver("video", {"direction": "sendrecv"})
    peerConnection.addTransceiver("audio", {"direction": "sendrecv"})

    let offer = await peerConnection.createOffer();
    peerConnection.setLocalDescription(offer);
  };

  return (
    <div className="App">
      <div className="camera-select">
        <label>Select Camera: </label>
        <select value={selectedCamera} onChange={handleCameraChange}>
          {cameras.map((camera, index) => {
            return <option key={index} value={camera}>{camera}</option>
          })}
        </select>
        {selectedCamera && (
        <div>
          <div id="videoDiv">

          </div>
          {/* <div className="camera-feed">
            <img src={`/stream/video_feed/${selectedCamera}`} alt="Camera Frame" width="600" height="400" />
          </div> */}
          <div className="slider-container">
            <label htmlFor="fpsSlider">FPS:</label>
            <input
              id="fpsSlider"
              type="range"
              min="0"
              max="100"
              value={fpsSlider}
              onChange={(event) => setFpsSlider(event.target.value)}
            />
            <label htmlFor="resolutionSlider">Resolution:</label>
            <input
              id="resolutionSlider"
              type="range"
              min="0"
              max="100"
              value={resolutionSlider}
              onChange={(event) => setResolutionSlider(event.target.value)}
            />
          </div>
        </div>
      )}
      </div>
    </div>
  );
}

export default App;
