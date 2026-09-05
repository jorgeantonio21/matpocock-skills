export function startDetached(
  task: () => Promise<void>,
  report: (error: unknown) => void,
): void {
  void task().catch(report);
}

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

export async function withAbortHandler<T>(
  signal: AbortSignal,
  onAbort: () => void,
  operation: () => Promise<T>,
): Promise<T> {
  signal.addEventListener("abort", onAbort, { once: true });
  try {
    signal.throwIfAborted();
    return await operation();
  } finally {
    signal.removeEventListener("abort", onAbort);
  }
}

export async function recoverLocalFailure(
  operation: () => Promise<string>,
): Promise<string> {
  try {
    return await operation();
  } catch {
    return "recovered";
  }
}
