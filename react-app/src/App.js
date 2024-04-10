import React, { useState, useEffect } from 'react';
import './App.css';

//filepath for testing (DELETE LATER): ../../../GitHub/Automomous/examples/ARTrackerTest/videos
function App() {
  const [sliderValue, setSliderValue] = useState(50); // Initial slider value

  //Need to create a selection of camera names to choose from, and then pass that camera name
  //to the image source to get the video feed from the server

  const [cameraNames, updateCameraNames] = useState([]) //getting available devices from server

  //Effect to get the camera names from the server
  useEffect(() => {
    fetch('/video_feed/available_devices')
      .then(response => response.json())
      .then(data => {
        updateCameraNames(data)
      })
  }, [])

  const [frame] = useState('/video_feed/[some_camera_name]') //calling stream route from server

//change this to a frame variable that contains an image that can be updated with each change
//of the state (as with any state variable) and then change the rendering to an image render
//instead of a video render
  return (
    <div className="App">
      <header className="App-header">
        Select Camera: 
        <select>
          {cameraNames.map((cameraName) => {
            return <option value={cameraName}>{cameraName}</option>
          })}
        </select>
      </header>
    </div>
  );
}

/*
      <img src={frame} alt="Camera Frame" width="600" height="400" />
      <p>Slider Value: {sliderValue}</p>
*/

export default App;