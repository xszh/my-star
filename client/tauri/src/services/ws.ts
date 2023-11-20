import {
  action,
  computed,
  makeObservable,
  observable,
  reaction,
  runInAction,
} from "mobx";
import { Service } from "./base";

export enum WSCommand {
  Connect,
  Disconnect,
  Ping,
  Pong,
}

export type WSDataMap = {
  [WSCommand.Connect]: {
    deviceId: string;
  };
  [WSCommand.Disconnect]: {};
  [WSCommand.Ping]: {};
  [WSCommand.Pong]: {};
};

export interface WSHeader {
  deviceId: string;
}

export interface WSData<C extends WSCommand> {
  command: C;
  data: WSDataMap[C];
}

export class WSService extends Service {
  private retryTimer?: number;

  @computed get did() {
    return this.context.get("auth").deviceId;
  }
  @observable private wss?: WebSocket;
  init(): void {
    this.reaction<{
      connected: boolean;
      did: string;
    }>(
      () => ({
        connected: this.connected,
        did: this.did,
      }),
      () => {
        this.retry();
      },
      {
        fireImmediately: true,
        equals: (a, b) => {
          return a.connected === b.connected && a.did === b.did;
        },
      }
    );

    this.reaction(
      () => this.connected,
      () => {},
      { fireImmediately: true }
    );
  }
  destroy(): void {
    this.clearClient();
    window.clearInterval(this.retryTimer);
  }

  private retry = () => {
    window.clearTimeout(this.retryTimer);
    this.clearClient();
    if (!this.connected && this.did) {
      this.bindClient();
      this.retryTimer = window.setTimeout(() => {
        this.retry();
      }, 1000);
    }
  };

  bindClient = () => {
    this.wss = new WebSocket("wss://www.miemie.tech/mystar/ws/");
    this.on("open", this.handleOpen);
    this.on("close", this.handleClose);
    this.on("message", this.handleMessage);
    this.on("error", this.handleError);
  };

  clearClient = () => {
    this.off("open", this.handleOpen);
    this.off("close", this.handleClose);
    this.off("message", this.handleMessage);
    this.off("error", this.handleError);
  };

  readonly on = <K extends keyof WebSocketEventMap>(
    type: K,
    listener: (this: WebSocket, ev: WebSocketEventMap[K]) => any,
    options?: boolean | AddEventListenerOptions
  ) => {
    this.wss?.addEventListener(type, listener, options);
  };

  readonly off = <K extends keyof WebSocketEventMap>(
    type: K,
    listener: (this: WebSocket, ev: WebSocketEventMap[K]) => any,
    options?: boolean | EventListenerOptions
  ) => {
    this.wss?.removeEventListener(type, listener, options);
  };

  send = <C extends WSCommand>(command: C, data: WSDataMap[C]) => {
    const header: WSHeader = {
      deviceId: this.did,
    };
    this.wss?.send(
      JSON.stringify({
        command,
        header,
        data,
      })
    );
  };

  @observable connected = false;

  @action
  handleOpen = () => {
    console.log("connect to wss");
    this.connected = true;
    this.send(WSCommand.Connect, {
      deviceId: this.context.get("auth").deviceId,
    });
  };

  @action
  handleClose = () => {
    this.connected = false;
    console.log("close wss");
  };

  @action
  handleMessage = (msg: MessageEvent) => {
    try {
      const payload = JSON.parse(msg.data);
      console.log(payload);
    } catch (error) {
      console.error("parse message fail", error);
    }
  };

  @action
  handleError = (event: WebSocketEventMap["error"]) => {
    console.error("wss error", event);
    this.connected = false;
  };

  constructor() {
    super();
    makeObservable(this);
  }
}
