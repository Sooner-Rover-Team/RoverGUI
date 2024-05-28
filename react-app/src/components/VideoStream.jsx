import React, { useEffect, useRef } from "react";
import axiosInstance from "../axiosInstance";

// Define a functional component called VideoStream
// TODO: use an arg for the cameraName
const VideoStream = () => {
  const cameraName = "Integrated_Webcam_FHD"; // Camera device name

  useEffect(() => {});

  return (
    <div>
      <h1>Video Stream for `{cameraName}`</h1>
      <img alt="Video Stream" style={{ width: "100%" }} />
    </div>
  );
};

export default VideoStream;
