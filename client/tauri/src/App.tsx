import { useState } from "react";
import "./App.css";

function App() {
  const [recording, setRecording] = useState(false);

  const startRecording = () => {
    setRecording(true);
  };

  const stopRecording = () => {
    setRecording(false);
  };

  return (
    <>
      {!recording && <button onClick={startRecording}>Record</button>}
      {recording && <button onClick={stopRecording}>Stop</button>}
    </>
  );
}

export default App;
