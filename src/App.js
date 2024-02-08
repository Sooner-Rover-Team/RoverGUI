import React, { useState } from 'react';
import './App.css';

function App() {
  const [sliderValue, setSliderValue] = useState(50); // Initial slider value

  return (
    <div className="App">
      <header className="App-header">
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



