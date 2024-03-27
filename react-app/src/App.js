import React, { useState } from 'react';
import './App.css';

//filepath for testing (DELETE LATER): ../../../GitHub/Automomous/examples/ARTrackerTest/videos
function App() {
  const [sliderValue, setSliderValue] = useState(50); // Initial slider value
  const [frame] = useState('/stream') //calling stream route from server

//change this to a frame variable that contains an image that can be updated with each change
//of the state (as with any state variable) and then change the rendering to an image render
//instead of a video render
  return (
    <div className="App">
      <header className="App-header">
        <img src={frame} alt="Camera Frame" width="600" height="400" />
        <p>Slider Value: {sliderValue}</p>
      </header>
    </div>
  );
}

export default App;