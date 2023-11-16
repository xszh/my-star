import { action, makeObservable, observable } from "mobx";
import { Service } from "./base";

export class AudioManager extends Service {
  init(): void {
    console.log("init");
    this.tauriOn<boolean>("audio_capture", (e) => {
      this.setCaputring(e.payload);
    });
    this.tauriOn<boolean>("audio_record", (e) => {
      this.setRecording(e.payload);
    });
    this.invoke<boolean>("is_capturing").then((d) => {
      this.setCaputring(d as boolean);
    });
    this.invoke<boolean>("is_recording").then((d) => {
      this.setRecording(d);
    });
  }
  destroy(): void {}
  @observable capturing = false;
  @observable recording = false;

  @action
  setCaputring = (value: boolean) => {
    this.capturing = value;
  };

  @action
  setRecording = (value: boolean) => {
    this.recording = value;
  };

  startCapture = () => {
    if (!this.capturing) {
      this.invoke("audio_open");
    }
  };

  stopCapture = () => {
    if (this.capturing) {
      this.invoke("audio_close");
    }
  };

  toggleCapture = () => {
    this.capturing ? this.stopCapture() : this.startCapture();
  };

  startRecord = () => {
    if (this.capturing && !this.recording) {
      this.invoke("start_record");
    }
  };

  stopRecord = () => {
    if (this.capturing && this.recording) {
      this.invoke("stop_record");
    }
  };

  startASR = async () => {
    if (this.capturing && !this.recording) {
      this.invoke("start_asr");
    }
  };

  stopASR = async () => {
    if (this.capturing && this.recording) {
      this.invoke("stop_asr", {
        appId: "45KqS2ZhFlq6F9Sp",
        token: "05e133aacb21478d8767f726ddb7c983",
      });
    }
  };

  constructor() {
    super();
    makeObservable(this);
  }
}
