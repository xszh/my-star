import { event, invoke } from "@tauri-apps/api";
import { Disposer } from "../utils/disposer";
import { reaction } from "mobx";
import { EventCallback, EventName } from "@tauri-apps/api/event";
import { InvokeArgs } from "@tauri-apps/api/tauri";

export const S_INIT = Symbol("init");
export const S_DESTROY = Symbol("destroy");

export abstract class Service {
  abstract init(): void;
  abstract destroy(): void;

  private [S_INIT]() {
    this.init();
  }
  private [S_DESTROY]() {
    this.destroy();
    this.disposer.dispose();
  }

  private disposer = new Disposer();
  tauriOn = <T>(eventName: EventName, handler: EventCallback<T>) => {
    event
      .listen<T>(eventName, (e) => {
        console.log(`[TAURI::EVENT]["${eventName}"] - <${e.payload}>`, e);
        handler(e);
      })
      .then((unlisten) => {
        this.disposer.add(unlisten);
      });
  };
  invoke = async <T>(cmd: string, args?: InvokeArgs): Promise<T> => {
    console.log(`[TAURI::INVOKE_START]["${cmd}"] (${args})`);
    try {
      const res = await invoke<T>(cmd, args);
      console.log(`[TAURI::INVOKE_END]["${cmd}"] <${res}>`);
      return res;
    } catch (error) {
      console.error(`[TAURI::INVOKE_FAIL]["${cmd}"] <${error}>`);
      throw error;
    }
  };
  reaction = (...args: Parameters<typeof reaction>) => {
    this.disposer.add(reaction(...args));
  };
}
