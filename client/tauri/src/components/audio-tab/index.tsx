import { observer } from "mobx-react";

import { Button } from "../button";

import { useService } from "@services";
import { tauri } from "@tauri-apps/api";

export const AudioTab: React.FC = observer(function () {
  const capturing = useService().get("audio").capturing;
  const captureBtnTxt = capturing ? "Stop Capture" : "Start Capture";
  return (
    <div>
      <Button
        onClick={() => {
          if (!capturing) {
            tauri.invoke("audio_open");
          } else {
            tauri.invoke("audio_close");
          }
        }}
      >
        {captureBtnTxt}
      </Button>
    </div>
  );
});
