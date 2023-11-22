import {
  action,
  computed,
  makeObservable,
  observable,
  runInAction,
} from "mobx";
import { Service } from "./base";
import { Promiser } from "../utils/promise";

const HEARTBEAT_INTERVAL = 60_000;


class WSClient {
  @observable connected = false;
  private ws: WebSocket = new WebSocket("wss://www.miemie.tech/mystar/ws/");

  private connectPromiser = new Promiser<void>();
  connect = async () => {
    if (this.ws.readyState === WebSocket.OPEN) {
      return;
    }
    if (this.ws.readyState === WebSocket.CONNECTING) {
      return this.connectPromiser.tryWait(1000);
    }
    throw new Error("WebSocket Closing or Closed");
  }

  private disconnectPromiser = new Promiser<void>();
  disconnect = async (code?: number, reason?: string) => {
    runInAction(() => {
      this.connected = false;
    });
    if (this.ws.readyState < 2) {
      this.ws.close(code, reason);
    }
    await this.disconnectPromiser.tryWait(1000);
    this.dispose();
  }

  send = (...data: Parameters<WebSocket['send']>) => {
    if (this.connected) {
      this.ws.send(...data);
    }
  }

  private pingTimer?: number;
  constructor() {
    this.ws.onopen = () => {
      this.connectPromiser.resolve();
      runInAction(() => {
        this.connected = true;
      })
    }
    this.ws.onclose = () => {
      runInAction(() => {
        this.connected = false;
      });
      this.disconnectPromiser.resolve();
      this.disconnect();
    }
    this.ws.onerror = (err) => {
      const errMsg = `ws error ${err}`;
      this.disconnect(500, errMsg);
    }
    this.pingTimer = window.setInterval(() => {
      const now = new Date().getTime();
      if (this.lastPing && now - this.lastPing >= 5000) {
        this.disconnectPromiser.reject("heartbeat stop");
        this.disconnect(500, "heartbeat stop");
      }
    }, HEARTBEAT_INTERVAL);
    this.ws.onmessage = this.handleMessage;
    makeObservable(this);
  }

  onMessage?: (payload: any) => void;
  private lastPing?: number;
  private handleMessage = (ev: MessageEvent<any>) => {
    const data = ev.data.toString();
    if (data === 'ping') {
      console.info("incoming ping");
      this.lastPing = new Date().getTime();
    } else {
      try {
        const payload = JSON.parse(data);
        if (!payload) return;
        this.onMessage?.(payload);
      } catch (error) {
        console.error("incoming invalid message", ev.data);
      }
    }
  }

  private dispose() {
    window.clearInterval(this.pingTimer);
  }
}

export class WSService extends Service {
  init(): void {
    this.reaction(() => this.connected, this.handleConnected, {
      fireImmediately: true,
    });
  }
  destroy(): void {
  }
  @computed get did() {
    return this.context.get("auth").deviceId;
  }

  @observable private wsClient?: WSClient;

  @computed get connected() {
    return this.wsClient?.connected ?? false;
  }

  @action
  newClient = () => {
    this.wsClient = new WSClient();
    this.wsClient.onMessage = this.handleMessage;
    return this.wsClient;
  }
  
  connect = async () => {
    if (!this.did) {
      throw new Error(`invalid device id ${this.did}`);
    }
    this.wsClient?.disconnect().catch((e) => {
      console.error("disconnect failed", e);
    });
    return this.newClient().connect();
  }

  disconnect = async () => {
    return this.wsClient?.disconnect();
  }

  private genHeader = () => {
    return {
      deviceId: this.did
    }
  }

  send = (data: any) => {
    this.wsClient?.send(JSON.stringify(data))
  }

  private connectTimer?: number;
  handleConnected = () => {
    window.clearTimeout(this.connectTimer);
    if (this.connected) {
      this.send({
        command: 0,
        header: this.genHeader(),
      });
    } else {
      // this.startConnect();
    }
  }

  startConnect = async () => {
    try {
      await this.connect()
    } catch (error) {
      this.connectTimer = window.setTimeout(() => {
        this.startConnect();
      }, 1000);
    }
  }

  handleMessage = (payload: any) => {
    console.log(payload);
  }

  constructor() {
    super();
    makeObservable(this);
  }
}
