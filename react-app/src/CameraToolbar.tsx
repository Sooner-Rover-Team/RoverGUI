import { useEffect, useState } from "react";
import { type CameraContainer } from "./CameraGrid";
import "./CameraToolbar.css";

interface CameraToolbarProps {
  cameras: string[] | null;
  cameraConnections: Map<string, RTCPeerConnection>;
  cameraContainers: CameraContainer[];
  selectedCamera: string;
  selectedSize: "large" | "small";
  setCameraContainers: React.Dispatch<React.SetStateAction<CameraContainer[]>>;
  setCameraConnections: React.Dispatch<React.SetStateAction<Map<string, RTCPeerConnection>>>;
  setSelectedSize: React.Dispatch<React.SetStateAction<"large" | "small">>;
  updateToolbar: (cameraPath: string) => void;
}

const CameraToolbar: React.FC<CameraToolbarProps> = ({
  cameras,
  cameraConnections,
  cameraContainers,
  selectedCamera,
  selectedSize,
  setCameraContainers,
  setCameraConnections,
  setSelectedSize,
  updateToolbar,
}) => {
  const [fpsSlider, setFpsSlider] = useState<number>(50); // Initial fps slider value
  const [resolutionSlider, setResolutionSlider] = useState<number>(50); // Initial resolution slider value

  const  [isStreamActive, setIsStreamActive] = useState<boolean>(false);

  useEffect(() => {
    if (selectedCamera === "") {
      setIsStreamActive(false);
      return;
    }

    const connection = cameraConnections.get(selectedCamera);
    setIsStreamActive(connection?.connectionState === "connected" || connection?.connectionState === "connecting" || connection?.iceConnectionState === "failed");
  }, [selectedCamera, cameraConnections]);

  const handleSizeChange = (size: "large" | "small") => {
    setSelectedSize(size);

    // Update the size of the currently selected camera container
    setCameraContainers((prev) =>
      prev.map((container) =>
        container.name === selectedCamera
          ? { ...container, size }
          : container
      )
    );
  };

  const handleCameraChange = async (event: React.ChangeEvent<HTMLSelectElement>) => {
    const selectedCameraPath = event.target.value;
    updateToolbar(selectedCameraPath);
  };

  const throwCameraError = (connection: RTCPeerConnection, errorMessage: string) => {
    const cameraId = Array.from(cameraConnections.entries()).find(([_, conn]) => conn === connection)?.[0];

    alert(`stream: camera connection error\nCamera ID: ${cameraId}\nError: ${errorMessage}`);

    connection.close();

    setCameraContainers((prev) =>
      prev.map((container) =>
        container.name === cameraId
          ? { ...container, connStream: null, error: errorMessage }
          : container
      )
    );
  };

  const handleLaunchStream = async (cameraConnection?: CameraContainer) => {
    if (selectedCamera === "") {
      alert("stream: no camera selected; cannot add camera.");
      return;
    }

    if(!cameraConnection) {
      // find the first available container
      const availableContainer = cameraContainers.find(
        (container) => container.stream === null && container.connection === null
      );
      if (!availableContainer) {
        alert("stream: no available containers for camera");
        return;
      }
      cameraConnection = availableContainer;
    }

    const existingContainer = cameraContainers.find(container => container.name === selectedCamera);

    if (existingContainer && existingContainer.id !== cameraConnection.id) {
      alert("stream: camera already exists in grid; cannot add camera.");
      return;
    }

    const cameraId = selectedCamera;

    const peerConnection = new RTCPeerConnection();

    setCameraContainers((prev) =>
      prev.map((container) =>
        container.id === cameraConnection.id
          ? {
              ...container,
              name: cameraId,
              size: selectedSize,
              connection: peerConnection,
            }
          : container
      )
    );

    peerConnection.onconnectionstatechange = () => {
      console.info("stream: peer connection change", {
        cameraId,
        state: peerConnection.connectionState,
      });

      if (peerConnection.connectionState === "failed") {
        throwCameraError(peerConnection, `Peer connection state is ${peerConnection.connectionState}`);
      }
    };

    peerConnection.oniceconnectionstatechange = () => {
      console.info("stream: ice connection state changed", {
        cameraId,
        state: peerConnection.iceConnectionState,
      });
      if (peerConnection.connectionState === "failed") {
        throwCameraError(peerConnection, `Peer connection state is ${peerConnection.connectionState}`);
      }
    };

    peerConnection.ontrack = (e) => {
      console.debug("stream: received track event on peer connection", {
        cameraId,
        trackKind: e.track.kind,
        streamIds: e.streams.map((s) => s.id),
      });

      if (e.track.kind === "video" && e.streams.length > 0) {
        const stream = e.streams[0];

        setCameraContainers((prev) => {
          return prev.map((container) =>
            container.id === cameraConnection.id
              ? {
                  ...container,
                  name: cameraId,
                  stream: stream || null,
                  connection: peerConnection,
                }
              : container
          );
        });

        console.debug("stream: created video element for track event", {
          cameraId,
          trackKind: e.track.kind,
          streamIds: e.streams.map((s) => s.id),
        });
      }
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
        }
      );
      if (!response.ok) {
        throw new Error(`failed to start stream! http err: ${response.status}`);
      }
      const remoteOffer = await response.json();
      await peerConnection.setRemoteDescription(new RTCSessionDescription(remoteOffer));
      setCameraConnections((prev) => new Map(prev).set(cameraId, peerConnection));
    } catch (error) {
      throwCameraError(peerConnection, error instanceof Error ? error.message : String(error));
    }
  };

  const handleRelaunchStream = async () => {
    if (selectedCamera === "") {
      alert("stream: no camera selected; cannot re-launch stream.");
      return;
    }

    const cameraConnection = cameraConnections.get(selectedCamera);
    const cameraContainer = cameraContainers.find(container => container.name === selectedCamera);

    if (!cameraConnection) {
      alert("stream: selected camera does not have an active connection; cannot re-launch stream.");
      return;
    }

    cameraConnection.close();

    setCameraConnections((prev) => {
      const updated = new Map(prev);
      updated.delete(selectedCamera);
      return updated;
    });

    setCameraContainers((prev) =>
      prev.map((container) =>
        container.name === selectedCamera
          ? { ...container, stream: null, connection: null }
          : container
      )
    );

    if (!cameraContainer) {
      alert("stream: selected camera does not have an available container; cannot re-launch stream.");
      return;
    }

    await handleLaunchStream(cameraContainer);
  }

  return (
    <div className="toolbar-container">
      <div className="size-group">
        <label className="label">
          <input
            type="radio"
            value="large"
            checked={selectedSize === "large"}
            onChange={() => handleSizeChange("large")}
          />
          Large
        </label>

        <label className="label">
          <input
            type="radio"
            value="small"
            checked={selectedSize === "small"}
            onChange={() => handleSizeChange("small")}
          />
          Small
        </label>
      </div>

      <div className="slider-group">
        <label>Compression Quality</label>
        <input
          type="range"
          min="0"
          max="100"
          value={fpsSlider}
          onChange={(e) => setFpsSlider(Number(e.target.value))}
          className="slider"
        />
      </div>

      <div className="slider-group">
        <label>Resolution</label>
        <input
          type="range"
          min="0"
          max="100"
          value={resolutionSlider}
          onChange={(e) => setResolutionSlider(Number(e.target.value))}
          className="slider"
        />
      </div>

      <select
        value={selectedCamera}
        onChange={handleCameraChange}
        className="dropdown"
      >
        {cameras?.map((camera, index) => (
          <option key={camera || `empty-${index}`} value={camera}>
            {camera || "Select Camera"}
          </option>
        ))}
      </select>
      <button className="button" onClick={() => {
        if (selectedCamera === "") {
          alert("stream: no camera selected; cannot launch stream.");
          return;
        }

        // find camera container for selected camera
        const cameraContainer = cameraContainers.find(container => container.name === selectedCamera);

        handleLaunchStream(cameraContainer);
      }} disabled={isStreamActive}>
        Launch Stream
      </button>

      <button className="button" onClick={handleRelaunchStream}>
        Re-Launch
      </button>
    </div>
  );
};

export default CameraToolbar;