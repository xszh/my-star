export class TimeoutError extends Error {
  constructor(timeout: number) {
    super(`timeout after ${timeout} ms`);
  }
}

export class Promiser<T> implements PromiseLike<T> {
  then<TResult1 = T, TResult2 = never>(
    onfulfilled?:
      | ((value: T) => TResult1 | PromiseLike<TResult1>)
      | null
      | undefined,
    onrejected?:
      | ((reason: any) => TResult2 | PromiseLike<TResult2>)
      | null
      | undefined
  ): PromiseLike<TResult1 | TResult2> {
    return this.mPromise.then(onfulfilled, onrejected);
  }

  catch<TResult2 = never>(onrejected:
    | ((reason: any) => TResult2 | PromiseLike<TResult2>)
    | null
    | undefined) {
    return this.mPromise.catch(onrejected)
  }

  finally(onfinally?: (() => void) | null | undefined) {
    return this.mPromise.finally(onfinally);
  }

  private mPromise;
  private resolver?: (value: T | PromiseLike<T>) => void;
  private rejecter?: (reason?: any) => void;
  constructor() {
    this.mPromise = new Promise<T>((resolver, rejecter) => {
      this.resolver = resolver;
      this.rejecter = rejecter;
    });
  }

  resolve = (value: T | PromiseLike<T>) => {
    this.resolver?.(value);
  };

  reject = (reason?: any) => {
    this.rejecter?.(reason);
  };

  tryWait = (timeout: number) => {
    const timer = window.setTimeout(() => {
      this.reject(new TimeoutError(timeout));
    }, timeout);
    this.mPromise.finally(() => {
      window.clearTimeout(timer);
    });
    return this.mPromise;
  }
}

export function asyncDisposer<D extends () => void, F extends Promise<D>>(
  fn: F
): () => void {
  const prom = new Promiser<D>();
  fn.then(prom.resolve, prom.reject);
  return () => {
    prom.then(
      (d) => d(),
      (e) => console.error("async dispose fail", e)
    );
  };
}
