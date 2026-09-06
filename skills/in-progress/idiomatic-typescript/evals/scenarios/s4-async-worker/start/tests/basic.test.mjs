import assert from "node:assert/strict";
import test from "node:test";

import { runBatch } from "../dist/index.js";

test("runBatch maps values", async () => {
  const signal = new AbortController().signal;
  assert.deepEqual(await runBatch([1, 2], 2, async (value) => value * 2, signal), [2, 4]);
});
