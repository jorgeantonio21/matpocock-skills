export interface RequestState {
  status: "idle" | "loading" | "loaded";
  value?: string;
}

export function describeState(state: RequestState): string {
  if (state.status === "loaded") {
    return state.value ?? "";
  }
  return state.status;
}

const labels: Record<string, string> = {
  idle: "Idle",
  loading: "Loading",
  loaded: "Loaded",
};

export function labelForStatus(status: string): string {
  return labels[status];
}
