import React, { useEffect, useState } from "react";

import { useService } from "@services";
import { observer } from "mobx-react";

import "./style.less";

export const Chat: React.FC = observer(function () {
  const audioService = useService().get("audio");
  const { capturing, recording } = audioService;

  const [rcdBtnText, setRcdBtnText] = useState("");

  useEffect(() => {
    if (!capturing) {
      setRcdBtnText("Start Capture First");
    } else {
      setRcdBtnText(recording ? "Stop" : "Start");
    }
  }, [capturing, recording]);

  return (
    <div className="ms-chat">
      <div className="ms-chat-messages"></div>
      <div className="ms-chat-operation">
        <button
          className="ms-chat-record"
          onClick={() => {
            if (!capturing) return;
            !recording ? audioService.startASR() : audioService.stopASR();
          }}
        >
          {rcdBtnText}
        </button>
      </div>
    </div>
  );
});
