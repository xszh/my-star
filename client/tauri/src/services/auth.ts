import { dialog } from "@tauri-apps/api";
import axios from "axios";
import { Service } from "./base";
import { computed, makeObservable, observable, runInAction } from "mobx";

const LS_KEY_ALICLOUD_TOKEN = "ALICLOUD_TOKEN";

export interface AliCloudToken {
  UserId: string;
  Id: string;
  ExpireTime: number;
}

export async function getToken() {
  try {
    const token = JSON.parse(
      localStorage.getItem(LS_KEY_ALICLOUD_TOKEN) || ""
    ) as AliCloudToken;
    if (!token.Id || !token.ExpireTime || !token.ExpireTime) {
      throw new Error("invalid token");
    }
    if (token.ExpireTime * 1000 <= new Date().getTime() - 5 * 60 * 1000) {
      throw new Error("token expired");
    }
    return token.Id;
  } catch (error) {
    console.warn(`${error}`);
  }
  try {
    const { data } = await axios.get(
      "http://alicloute-token-rpc-shanghai-tnhzbmofml.cn-shanghai.fcapp.run"
    );
    localStorage.setItem(LS_KEY_ALICLOUD_TOKEN, JSON.stringify(data));
    return data.Id as string;
  } catch (error) {
    dialog.message("Fetch Token Fail");
    throw error;
  }
}

export class AuthService extends Service {
  @observable private mToken: string = "";
  @computed get token() {
    return this.mToken;
  }
  init(): void {
    getToken().then((t) => {
      runInAction(() => {
        this.mToken = t;
      });
    });
  }
  destroy(): void {
    throw new Error("Method not implemented.");
  }

  constructor() {
    super();
    makeObservable(this);
  }
}
