import assert from "node:assert/strict";
import test from "node:test";

import { describeState, labelForStatus } from "../dist/index.js";

test("failed state carries and describes its error", () => {
  assert.equal(describeState({ status: "failed", error: new Error("offline") }), "offline");
});

test("unknown status labels stay absent", () => {
  assert.equal(labelForStatus("future-state"), undefined);
});
