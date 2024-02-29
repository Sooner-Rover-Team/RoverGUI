import React, { useState } from 'react';
import ReactPlayer from 'react-player';
import './App.css';

//filepath for testing (DELETE LATER): ../../../GitHub/Automomous/examples/ARTrackerTest/videos
function App() {
  const [sliderValue, setSliderValue] = useState(50); // Initial slider value
  const [videoFilePath] = useState("https://youtu.be/WLw4mNk6fTk?si=9-LhGRexKvEPWMQ1"); // Initial video file path

  return (
    <div className="App">
      <header className="App-header">
      <ReactPlayer url={videoFilePath} width="90%" height="90%" playing={true} volume={0} controls={true} />
        <input
          type="range"
          min="1"
          max="100"
          value={sliderValue}
          onChange={(e) => setSliderValue(e.target.value)}
        />
        <p>Slider Value: {sliderValue}</p>
      </header>
    </div>
  );
}

export default App;