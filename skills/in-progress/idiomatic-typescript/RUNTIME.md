# TypeScript runtime control flow

Read this when code owns promises, callbacks, concurrency, cancellation, timers, listeners, streams, or cleanup. First name who owns completion, failure, and resource release.

## Own every promise

Await or return work whose result belongs to the operation. A deliberately detached task has a lifecycle owner and handles rejection at the detachment site. Prefixing a call with `void` communicates discarded completion to some linters, but it does not handle a rejection.

```ts
export function startDetached(
  task: () => Promise<void>,
  report: (error: unknown) => void,
): void {
  void task().catch(report);
}
```

A promise-returning callback passed to a `void` callback slot can lose its rejection. Wrap it so the promise is owned, or use an API whose callback contract awaits it. Typed lint rules for floating and misused promises detect different paths.

## Choose sequencing and fan-out

Use sequential iteration when order, a dependency's rate, or per-item state requires it. Use deliberate concurrency when tasks are independent. Bound fan-out when input size can exceed a dependency's capacity.

```ts
export async function mapConcurrent<T, U>(
  values: readonly T[],
  limit: number,
  map: (value: T, signal: AbortSignal) => Promise<U>,
  signal: AbortSignal,
): Promise<U[]> {
  if (!Number.isInteger(limit) || limit < 1) {
    throw new RangeError("limit must be a positive integer");
  }

  const results = new Array<U>(values.length);
  let nextIndex = 0;
  let failed = false;
  let failure: unknown;

  async function worker(): Promise<void> {
    while (!failed) {
      try {
        signal.throwIfAborted();
        const index = nextIndex;
        if (index >= values.length) {
          return;
        }
        nextIndex += 1;
        results[index] = await map(values[index]!, signal);
      } catch (error) {
        if (!failed) {
          failed = true;
          failure = error;
        }
      }
    }
  }

  const workerCount = Math.min(limit, values.length);
  await Promise.all(Array.from({ length: workerCount }, () => worker()));
  if (failed) {
    throw failure;
  }
  return results;
}
```

The non-null assertion is supported by the checked index range and the input's fixed length during the operation. The first failure stops new scheduling, all started workers settle, then the owner rejects with that failure. The readonly parameter blocks mutation through this interface, not through every alias. Copy the input first if stable membership is part of the contract.

`Promise.all` preserves input order in its result and rejects when an input rejects. It does not cancel sibling work. If siblings must stop, pass a shared cancellation signal to operations that honor it and decide whether to wait for their settlement before returning.

## Keep failure inside the observing scope

A surrounding `catch` or `finally` observes a returned promise only when the promise is awaited inside that scope.

```ts
export async function recoverLocalFailure(
  operation: () => Promise<string>,
): Promise<string> {
  try {
    return await operation();
  } catch {
    return "recovered";
  }
}
```

Outside an observing scope, `return promise` and `return await promise` can remain a repository style choice. Avoid old performance folklore as the reason to remove `return await`.

## Cancellation is cooperative

Pass an `AbortSignal` through cancelable I/O and check it at useful boundaries. A signal only affects code that observes it. Define whether cancellation rejects, returns a domain outcome, or leaves partial work, and match the surrounding API.

Use one controller when one owner cancels a group. Use composed or timeout signals only when the supported runtime provides the required behavior. Do not invent a boolean flag when downstream APIs already accept `AbortSignal`.

## Cleanup follows every exit

Put cleanup in the control flow that owns the resource. Use `finally` for timers, listeners, handles, temporary files, and subscriptions that must be released after success, failure, or cancellation. If cleanup is asynchronous, await it before leaving the owning scope.

`{ once: true }` removes an abort listener after abort fires. It does not remove that listener when the operation completes normally. Remove listeners on the normal path too, or use an API that owns that lifecycle.

For streams and event emitters, inspect the runtime's distinct failure channels. A thrown exception, rejected promise, callback error, and emitted `error` event are not interchangeable. Preserve the ecosystem's expected channel at the public surface.

## Review questions

- Who waits for completion?
- Who receives rejection?
- Can the callback contract observe a returned promise?
- Is concurrency sequential, unbounded, or bounded for a stated reason?
- What continues after the first failure?
- Which operations honor cancellation?
- Does every success, failure, and cancellation path release its resources?
