declare function indexBy<T, K extends PropertyKey>(
  values: readonly T[],
  keyOf: (value: T) => K,
): Map<K, T>;
indexBy([{ details: {} }], (value) => value.details);
