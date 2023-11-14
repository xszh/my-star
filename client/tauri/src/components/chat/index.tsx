import React, { useState } from "react";

import "./style.less";

export const Chat: React.FC = function () {
  const [rcdBtnText, setRcdBtnText] = useState("Press");

  return (
    <div className="ms-chat">
      <div className="ms-chat-messages"></div>
      <div className="ms-chat-operation">
        <div className="ms-chat-record">{rcdBtnText}</div>
      </div>
    </div>
  );
};
