export type LoadState =
  | { status: "idle" }
  | { status: "loaded"; value: string }
  | { status: "failed"; error: Error };

export function stateLabel(state: LoadState): string {
  switch (state.status) {
    case "idle":
      return "Idle";
    case "loaded":
      return state.value;
    case "failed":
      return state.error.message;
    default:
      return assertNever(state);
  }
}

export function assertNever(value: never): never {
  throw new Error(`unexpected state: ${JSON.stringify(value)}`);
}

export function lookupLabel(
  labels: Readonly<Record<string, string>>,
  key: string,
): string | undefined {
  return labels[key];
}
