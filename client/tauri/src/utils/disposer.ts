export class Disposer {
  private fns: (() => void)[] = [];
  constructor() {}
  add(fn: () => void) {
    this.fns.push(fn);
  }
  dispose() {
    this.fns.forEach((fn) => fn());
  }
}
