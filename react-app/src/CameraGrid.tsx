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
        <div
          key={cam.id}
          className={`camera-tile ${cam.size}`}
        >
          {cam.name}
        </div>
      ))}
    </div>
  );
};

export default CameraGrid;