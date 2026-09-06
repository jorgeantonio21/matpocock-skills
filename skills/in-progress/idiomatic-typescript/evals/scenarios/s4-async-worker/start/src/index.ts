export async function runBatch<T, U>(
  values: readonly T[],
  limit: number,
  operation: (value: T, signal: AbortSignal) => Promise<U>,
  signal: AbortSignal,
): Promise<U[]> {
  void limit;
  return Promise.all(values.map((value) => operation(value, signal)));
}
