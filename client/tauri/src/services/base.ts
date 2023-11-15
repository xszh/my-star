import { event } from "@tauri-apps/api";
import { Disposer } from "../utils/disposer";
import { reaction } from "mobx";

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
  tauriOn = (...args: Parameters<typeof event.listen>) => {
    event.listen(...args).then((unlisten) => {
      this.disposer.add(unlisten);
    });
  };
  reaction = (...args: Parameters<typeof reaction>) => {
    this.disposer.add(reaction(...args));
  };
}
