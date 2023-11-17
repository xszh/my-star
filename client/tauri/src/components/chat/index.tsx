import React, { useEffect, useRef, useState } from "react";

import { useService } from "@services";
import { observer } from "mobx-react";

import cx from "classnames";

import "./style.less";
import { Icon } from "../icon";
import { makeObservable, observable } from "mobx";

interface Message {
  sender: string,
  content: string,
  time: Date,
}

class ChatStore {
  @observable
  readonly messages: Message[] = [];

  push = (content: string, sender: string) => {
    this.messages.push({
      sender,
      content,
      time: new Date(),
    })
  }

  constructor() {
    makeObservable(this);
  }
}

export const Chat: React.FC = observer(function () {
  const audioService = useService().get("audio");
  const { capturing, recording } = audioService;

  const [mode, setMode] = useState<'Audio' | 'Text'>('Text');

  const chatStore = useRef(new ChatStore()).current;

  const onStart = () => {
    if (!capturing) return;
    if (recording) return;
    audioService.startASR();
  };

  const onStop = () => {
    if (!capturing) return;
    audioService.stopASR().then(res => {
      if (res) {
        chatStore.push(res, 'ASR');
      }
    });
  };

  useEffect(() => {
    if (!capturing) {
      setMode('Text');
    }
  }, [capturing]);

  return (
    <div className="ms-chat">
      <div className="ms-chat-messages">{
        chatStore.messages.map(msg => {
          return (<div key={msg.time.getTime()}>{msg.content}</div>)
        })
      }</div>
      <div className="ms-chat-operation">
        {
          <div className="ms-chat-mode" onClick={() => {
            if (capturing) {
              setMode(mode === 'Text' ? 'Audio' : 'Text')
            } else {
              setMode('Text');
            }
          }}>
            <Icon name={mode === 'Text' ? 'keyboard-26' : 'mic-player'} />
          </div>
        }
        {mode === 'Audio' && (
          <button
            className={cx("ms-chat-record")}
            onMouseDown={onStart}
            onMouseUp={onStop}
          >
            {"按住说话"}
          </button>
        )}
      </div>
    </div>
  );
});
