import { observable } from "mobx";
import { invoke } from "@tauri-apps/api";
import { Service } from "./base";

export class AudioManager extends Service {
  init(): void {
    this.reaction(
      () => this.capturing,
      () => console.log(this.capturing),
      { fireImmediately: true }
    );
    this.tauriOn("audio_capture", (e) => {
      console.log(e);
      this.capturing = e.payload as boolean;
    });
    this.tauriOn("audio_record", (e) => {
      console.log(e);
      this.recording = e.payload as boolean;
    });
    invoke("is_capturing").then((d) => (this.capturing = d as boolean));
    invoke("is_recording").then((d) => (this.recording = d as boolean));
  }
  destroy(): void {}
  @observable capturing = false;
  @observable recording = false;
}
