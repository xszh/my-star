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
