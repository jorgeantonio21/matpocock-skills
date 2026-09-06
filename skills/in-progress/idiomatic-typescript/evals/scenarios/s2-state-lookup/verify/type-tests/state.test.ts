import { type RequestState } from "../src/index.js";

const loaded: RequestState = { status: "loaded", value: "done" };
// @ts-expect-error: loaded state requires its value
const missingValue: RequestState = { status: "loaded" };
// @ts-expect-error: idle state cannot carry loaded data
const idleWithValue: RequestState = { status: "idle", value: "wrong" };

void loaded;
void missingValue;
void idleWithValue;
