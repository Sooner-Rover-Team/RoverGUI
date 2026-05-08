import { useState, useEffect } from "react";
import "./App.css";
import CameraGrid, { type CameraContainer } from "./CameraGrid";
import CameraToolbar from "./CameraToolbar";

//filepath for testing (DELETE LATER): ../../../GitHub/Automomous/examples/ARTrackerTest/videos
function App() {

  //Need to create a selection of camera names to choose from, and then pass that camera name
  //to the image source to get the video feed from the server

  const [cameras, setCameras] = useState<string[] | null>([]); //getting available devices from server

  const [cameraConnections, setCameraConnections] = useState<Map<string, RTCPeerConnection>>(new Map());

  // Fetch Camera(s) Information from Server
  useEffect(() => {
    (async () => {
      const response = await fetch("/stream/cameras");
      const cameras = await response.json();

      console.log(`Found cameras: ${cameras}`);

      setCameras(["", ...cameras]);
    })();
  }, []);

  //On change of the camera selection, add components for camera feed and sliders
  //to control the camera feed
  const [selectedCamera, setSelectedCamera] = useState("");
  const [selectedSize, setSelectedSize] = useState<"large" | "small">("large");

  const [cameraContainers, setCameraContainers] = useState<CameraContainer[]>([
    { id: '1', name: 'Camera 1', size: 'large', connection: null, stream: null },
    { id: '2', name: 'Camera 2', size: 'large', connection: null, stream: null },
    { id: '3', name: 'Camera 3', size: 'large', connection: null, stream: null },
    { id: '4', name: 'Camera 4', size: 'large', connection: null, stream: null },
    { id: '5', name: 'Camera 5', size: 'large', connection: null, stream: null },
  ]);

  const updateToolbar = (cameraPath: string) => {
    console.info(`stream: camera selection changed to: \`${cameraPath}\``);

    setSelectedCamera(cameraPath);

    const existingContainer = cameraContainers.find(container => container.name === cameraPath);

    setSelectedSize(existingContainer?.size || "large");
  };

  const handleRemoveCamera = () => {
    const connection = cameraConnections.get(selectedCamera);
    if (connection) {
      connection.close();
      const updatedConnections = new Map(cameraConnections);
      updatedConnections.delete(selectedCamera);
      setCameraConnections(updatedConnections);

      setCameraContainers((prev) =>
        prev.map((container) =>
          container.name === selectedCamera
            ? { ...container, stream: null, connection: null, name: '' }
            : container
        )
      );

      setSelectedCamera("");
    }
  };

  return (
    <div className="App">
      <div className="camera-select">
        <CameraToolbar
          cameras={cameras}
          selectedCamera={selectedCamera}
          selectedSize={selectedSize}
          cameraConnections={cameraConnections}
          cameraContainers={cameraContainers}
          setCameraContainers={setCameraContainers}
          setCameraConnections={setCameraConnections}
          setSelectedSize={setSelectedSize}
          updateToolbar={updateToolbar}
        />
      </div>
      <div className="camera-content">
        <div className="camera-grid">
          <CameraGrid cameras={cameraContainers} onRemoveCamera={handleRemoveCamera} selectedCamera={selectedCamera} setSelectedCamera={setSelectedCamera} updateToolbar={updateToolbar} />
        </div>
      </div>
    </div>
  );
}

export default App;
