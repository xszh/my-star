import { observer } from "mobx-react";

import { Button } from "../button";

import { useService } from "@services";

export const AudioTab: React.FC = observer(function () {
  const audioService = useService().get("audio");
  const { capturing, toggleCapture } = audioService;
  const captureBtnTxt = capturing ? "Stop Capture" : "Start Capture";
  return (
    <div>
      <Button
        onClick={() => {
          toggleCapture();
        }}
      >
        {captureBtnTxt}
      </Button>
    </div>
  );
});
