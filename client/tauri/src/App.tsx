import { useEffect, useState } from "react";
import "./App.css";

import { emit, listen } from '@tauri-apps/api/event';

import {} from '@tauri-apps/api/app'
import { invoke } from "@tauri-apps/api";

function App() {
  const [recording, setRecording] = useState(false);

  useEffect(() => {
    invoke("is_recording").then(value => {
      setRecording(!!value);
    })
    // listen<string>("")
  }, []);



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
