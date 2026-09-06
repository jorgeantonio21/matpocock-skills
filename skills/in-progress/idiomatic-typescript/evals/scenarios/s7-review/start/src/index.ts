export interface Settings {
  mode: WireMode;
  endpoint: string;
}

// Values are persisted by external clients and cannot change.
export enum WireMode {
  Fast = "fast",
  Safe = "safe",
}

export function parseSettings(text: string): Settings {
  return JSON.parse(text) as Settings;
}

interface Endpoint {
  url: string;
}

export function endpointUrl(
  endpoints: Readonly<Record<string, Endpoint>>,
  name: string,
): string {
  return endpoints[name].url;
}

export function echo(value: string): string;
export function echo(value: Uint8Array): Uint8Array;
export function echo(value: string | Uint8Array): string | Uint8Array {
  return value;
}

const handlers = {
  start: () => "started",
  stop: () => "stopped",
} as const;

type EventName = keyof typeof handlers;

export function handlerFor<E extends EventName>(event: E): (typeof handlers)[E] {
  return handlers[event] as (typeof handlers)[E];
}
