export async function runBatch<T, U>(
  values: readonly T[],
  limit: number,
  operation: (value: T, signal: AbortSignal) => Promise<U>,
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
        results[index] = await operation(values[index]!, signal);
      } catch (error) {
        if (!failed) {
          failed = true;
          failure = error;
        }
      }
    }
  }

  await Promise.all(
    Array.from({ length: Math.min(limit, values.length) }, () => worker()),
  );
  if (failed) {
    throw failure;
  }
  return results;
}
