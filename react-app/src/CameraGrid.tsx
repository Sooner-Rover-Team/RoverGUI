import React from "react";
import "./CameraGrid.css";

export interface CameraContainer {
  id: string;
  name: string;
  size: "large" | "small";
}

interface CameraGridProps {
  cameras: CameraContainer[];
}

const CameraGrid: React.FC<CameraGridProps> = ({ cameras }) => {
    return (
      <div className="grid-container">
        {cameras.map((cam) => (
          <div key={cam.id} className={`camera-tile ${cam.size}`}>
            <div className="camera-title">{cam.name}</div>
            <div className="camera-body">
              {/* video goes here later */}
            </div>
          </div>
        ))}
      </div>
    );
};

export default CameraGrid;