import type React from "react";
import "./CameraGrid.css";

export interface CameraContainer {
  id: string;
  name: string;
  size: "large" | "small";
  connection: RTCPeerConnection | null;
  stream: MediaStream | null;
  error?: string | null;
}

export interface ConnectionAndStream {
    connection: RTCPeerConnection;
    stream: MediaStream;
}

interface CameraGridProps {
  cameras: CameraContainer[];
  onRemoveCamera: (cameraId: string) => void;
  selectedCamera: string;
  setSelectedCamera: (cameraId: string) => void;
}

const CameraGrid: React.FC<CameraGridProps> = ({
  cameras,
  selectedCamera,
  setSelectedCamera,
  onRemoveCamera,
}) => {
  return (
    <div className="grid-container">
      {cameras.map((cam) => (
        <button
          key={cam.id}
          type="button"
          className={`camera-tile ${cam.size} ${
            cam.name === selectedCamera ? "selected" : ""
          }`}
          onClick={() => setSelectedCamera(cam.name)}
          onKeyDown={() => setSelectedCamera(cam.name)}
        >
          <div className="camera-header">
            <div className="camera-title">{cam.name}</div>
            <button
              type="button"
              className="remove-button"
              onClick={() => {
                onRemoveCamera(cam.id);
              }}
            >
              Remove Camera
            </button>
          </div>
          <div className="camera-body">
            {cam.error ? (
              <div className="camera-error">
                <h1>CAMERA ERROR</h1>
                <h3>{cam.error}</h3>
              </div>
            ) : cam.connection && cam.stream ? (
              <video
                ref={(el) => {
                  if (el && el.srcObject !== cam.stream) {
                    el.srcObject = cam.stream;
                  }
                }}
                autoPlay
                muted
                playsInline
                poster="/loading_indicator.svg"
                className="camera-feed"
              />
            ) : (
              <div className="camera-placeholder">
                <h1>No Camera Added</h1>
                <h3>
                  Select a camera in camera dropdown and press "Add Camera" to
                  add new feed.
                </h3>
              </div>
            )}
          </div>
        </button>
      ))}
    </div>
  );
};

export default CameraGrid;
