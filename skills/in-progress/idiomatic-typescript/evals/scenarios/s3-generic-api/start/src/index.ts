export function indexBy<T, K>(
  values: readonly T[],
  keyOf: (value: T) => K,
): Map<unknown, T> {
  const result = new Map<unknown, T>();
  for (const value of values) {
    result.set(keyOf(value), value);
  }
  return result;
}
