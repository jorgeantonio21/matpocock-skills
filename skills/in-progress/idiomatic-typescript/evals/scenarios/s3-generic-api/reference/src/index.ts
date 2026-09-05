export function indexBy<T, K extends PropertyKey>(
  values: readonly T[],
  keyOf: (value: T) => K,
): Map<K, T> {
  const result = new Map<K, T>();
  for (const value of values) {
    result.set(keyOf(value), value);
  }
  return result;
}
