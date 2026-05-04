import React from "react";
import "./CameraGrid.css";

export interface CameraContainer {
  id: string;
  name: string;
  size: "large" | "small";
  connection: RTCPeerConnection | null;
  stream: MediaStream | null;
}

interface CameraGridProps {
  cameras: CameraContainer[];
  onRemoveCamera: (cameraId: string) => void;
  connections: Map<string, RTCPeerConnection>;
}

const CameraGrid: React.FC<CameraGridProps> = ({ cameras }) => {
    return (
      <div className="grid-container">
        {cameras.map((cam) => (
          <div key={cam.id} className={`camera-tile ${cam.size}`}>
            <div className="camera-title">{cam.name}</div>
            <div className="camera-body">
            {cam.connection && cam.stream ? (
              <video
              ref={(el) => {
                if (el && el.srcObject !== cam.stream) {
                  el.srcObject = cam.stream;
                }
              }}
              autoPlay
              playsInline
              className="camera-feed"
            />
            ) : (
                <div className="camera-placeholder">
                    <button className="add-camera-button">+ Add Camera</button>
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
    );
};

export default CameraGrid;