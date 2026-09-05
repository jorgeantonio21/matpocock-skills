export interface TimeoutConfig {
  timeoutMs?: number;
}

export function effectiveTimeout(config: TimeoutConfig, fallbackMs: number): number {
  return config.timeoutMs ?? fallbackMs;
}
