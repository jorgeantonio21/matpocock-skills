export type RequestState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "loaded"; value: string }
  | { status: "failed"; error: Error };

function assertNever(value: never): never {
  throw new Error(`unexpected request state: ${JSON.stringify(value)}`);
}

export function describeState(state: RequestState): string {
  switch (state.status) {
    case "idle":
    case "loading":
      return state.status;
    case "loaded":
      return state.value;
    case "failed":
      return state.error.message;
    default:
      return assertNever(state);
  }
}

const labels = new Map<string, string>([
  ["idle", "Idle"],
  ["loading", "Loading"],
  ["loaded", "Loaded"],
  ["failed", "Failed"],
]);

export function labelForStatus(status: string): string | undefined {
  return labels.get(status);
}
