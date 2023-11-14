import { useEffect, useState } from "react";
import "./App.css";

import { UnlistenFn, emit, listen } from "@tauri-apps/api/event";

import {} from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api";
import { asyncDisposer } from "./utils/promise";

interface TextNode {
  success: boolean;
  text: string;
}

function App() {
  const [loading, setLoading] = useState(false);
  const [recording, setRecording] = useState(false);

  const [result, setResult] = useState<TextNode>();

  useEffect(() => {
    invoke("is_recording").then((value) => {
      console.log("is_recording", value);
      setRecording(!!value);
    });
    const disposer = asyncDisposer(
      listen<string>("recording", ({ payload }) => {
        console.log("is_recording", payload);
        setRecording(!!payload);
      })
    );
    return () => {
      disposer();
    };
  }, []);

  const startRecording = async () => {
    try {
      setLoading(true);
      await invoke("start_asr");
    } catch (error) {
      setResult({
        success: false,
        text: `${error}`,
      });
    } finally {
      setLoading(false);
    }
  };

  const stopRecording = async () => {
    try {
      setLoading(true);
      const data: string = await invoke("stop_asr", {
        appId: "45KqS2ZhFlq6F9Sp",
        token: "05e133aacb21478d8767f726ddb7c983",
      });
      setResult({
        success: true,
        text: data,
      });
    } catch (error) {
      setResult({
        success: false,
        text: `${error}`,
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      {!result?.success && <div>{result?.text}</div>}
      {result?.success && <div>{result?.text}</div>}
      {!recording && (
        <button onClick={startRecording}>{loading ? "..." : "Record"}</button>
      )}
      {recording && (
        <button onClick={stopRecording}>{loading ? "..." : "Stop"}</button>
      )}
    </>
  );
}

export default App;
