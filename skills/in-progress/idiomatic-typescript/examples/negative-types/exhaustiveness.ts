type State = { status: "idle" } | { status: "loaded" } | { status: "failed" };
function assertNever(value: never): never {
  throw new Error(String(value));
}
function label(state: State): string {
  switch (state.status) {
    case "idle": return "idle";
    case "loaded": return "loaded";
    default: return assertNever(state);
  }
}
void label;
