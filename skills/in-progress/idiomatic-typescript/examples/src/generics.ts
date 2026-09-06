export function indexBy<T, K extends PropertyKey>(
  values: readonly T[],
  keyOf: (value: T) => K,
): Map<K, T> {
  return new Map(values.map((value) => [keyOf(value), value]));
}

export const defaults = {
  enabled: true,
  retries: 0,
} satisfies { enabled: boolean; retries: number };
