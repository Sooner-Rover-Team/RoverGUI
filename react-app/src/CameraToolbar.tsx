import "./CameraToolbar.css";

interface CameraToolbarProps {
  cameras: string[] | null;
  selectedCamera: string;
  selectedSize: "large" | "small";
  fpsSlider: number;
  resolutionSlider: number;
  setSelectedCamera: (camera: string) => void;
  setSelectedSize: (size: "large" | "small") => void;
  setFpsSlider: (value: number) => void;
  setResolutionSlider: (value: number) => void;
  handleAddCamera: () => void;
}

function CameraToolbar(props: CameraToolbarProps) {
  function handleCameraChange(event: React.ChangeEvent<HTMLSelectElement>) {
    props.setSelectedCamera(event.target.value);
    
    const handleLaunchStream = (): void => { 
        console.log('Launch stream clicked'); 
    }; 
    const handleRelaunchStream = (): void => { 
        console.log('Re-launch stream clicked'); 
    }; 
    const handleAddCamera = (): void => { 
        console.log('Add camera clicked'); 
    };
  }

  return (
    <div className="toolbar-container">
      <button className="button" onClick={props.handleAddCamera}>
        Launch Stream
      </button>

      <button className="button" onClick={props.handleAddCamera}>
        Re-Launch
      </button>

      <div className="size-group">
        <label className="label">
          <input
            type="radio"
            value="large"
            checked={props.selectedSize === "large"}
            onChange={() => props.setSelectedSize("large")}
          />
          Large
        </label>

        <label className="label">
          <input
            type="radio"
            value="small"
            checked={props.selectedSize === "small"}
            onChange={() => props.setSelectedSize("small")}
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
          value={props.fpsSlider}
          onChange={(e) => props.setFpsSlider(Number(e.target.value))}
          className="slider"
        />
      </div>

      <div className="slider-group">
        <label>Resolution</label>
        <input
          type="range"
          min="0"
          max="100"
          value={props.resolutionSlider}
          onChange={(e) => props.setResolutionSlider(Number(e.target.value))}
          className="slider"
        />
      </div>

      <select
        value={props.selectedCamera}
        onChange={handleCameraChange}
        className="dropdown"
      >
        {props.cameras?.map((camera, index) => (
          <option key={camera || `empty-${index}`} value={camera}>
            {camera || "Select Camera"}
          </option>
        ))}
      </select>

      <button className="add-button" onClick={props.handleAddCamera}>
        + Add Camera
      </button>
    </div>
  );
}

export default CameraToolbar;