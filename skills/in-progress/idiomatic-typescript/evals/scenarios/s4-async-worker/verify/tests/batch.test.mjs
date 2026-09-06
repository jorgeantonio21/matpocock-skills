import assert from "node:assert/strict";
import test from "node:test";

import { runBatch } from "../dist/index.js";

test("runBatch preserves order and bounds concurrency", async () => {
  let active = 0;
  let maximum = 0;
  const signal = new AbortController().signal;
  const values = await runBatch(
    [1, 2, 3, 4, 5],
    2,
    async (value, receivedSignal) => {
      assert.equal(receivedSignal, signal);
      active += 1;
      maximum = Math.max(maximum, active);
      await Promise.resolve();
      active -= 1;
      return value * 10;
    },
    signal,
  );
  assert.deepEqual(values, [10, 20, 30, 40, 50]);
  assert.equal(maximum, 2);
});

test("runBatch rejects invalid limits and operation failures", async () => {
  const signal = new AbortController().signal;
  await assert.rejects(runBatch([1], 0, async (value) => value, signal), /limit/);
  await assert.rejects(
    runBatch([1], 1, async () => { throw new Error("operation failed"); }, signal),
    /operation failed/,
  );
});

test("runBatch waits for started work before propagating failure", async () => {
  let release;
  const gate = new Promise((resolve) => { release = resolve; });
  const batch = runBatch(
    [1, 2],
    2,
    async (value) => {
      if (value === 1) throw new Error("failed");
      await gate;
      return value;
    },
    new AbortController().signal,
  );
  let settled = false;
  void batch.catch(() => {}).then(() => { settled = true; });
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(settled, false);
  release();
  await assert.rejects(batch, /failed/);
});

test("runBatch observes a pre-aborted signal before scheduling", async () => {
  const controller = new AbortController();
  controller.abort(new Error("cancelled"));
  let calls = 0;
  await assert.rejects(
    runBatch([1, 2], 1, async (value) => { calls += 1; return value; }, controller.signal),
    /cancelled/,
  );
  assert.equal(calls, 0);
});
