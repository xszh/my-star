import { AudioManager } from "./audio";
import { useRef } from "react";
import { S_DESTROY, S_INIT, Service } from "./base";

console.log("service.ts");

const ServiceConfig = {
  audio: new AudioManager(),
};

type ServiceMap = typeof ServiceConfig;

class _ServiceManager {
  private services: Map<keyof ServiceMap, Service> = new Map();

  constructor() {
    Object.entries(ServiceConfig).forEach(([key, service]) => {
      this.services.set(key as keyof ServiceMap, service);
    });
  }

  init() {
    this.services.forEach((s) => s[S_INIT]());
  }

  destroy() {
    this.services.forEach((s) => s[S_DESTROY]());
  }

  get<K extends keyof ServiceMap>(key: K): ServiceMap[K] {
    return this.services.get(key) as ServiceMap[K];
  }
}

export const ServiceManager = new _ServiceManager();

export const useService = () => {
  return useRef(ServiceManager).current;
};
