import assert from "node:assert/strict";
import test from "node:test";

import { describeState, labelForStatus } from "../dist/index.js";

test("existing states retain their output", () => {
  assert.equal(describeState({ status: "idle" }), "idle");
  assert.equal(describeState({ status: "loading" }), "loading");
  assert.equal(describeState({ status: "loaded", value: "done" }), "done");
  assert.equal(labelForStatus("loaded"), "Loaded");
});
