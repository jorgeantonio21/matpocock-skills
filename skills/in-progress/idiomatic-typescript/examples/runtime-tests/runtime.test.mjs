import assert from "node:assert/strict";
import test from "node:test";

import {
  mapConcurrent,
  recoverLocalFailure,
  startDetached,
  withAbortHandler,
} from "../dist/index.js";

test("mapConcurrent preserves order and bounds fan-out", async () => {
  let active = 0;
  let maximum = 0;
  const controller = new AbortController();
  const output = await mapConcurrent(
    [1, 2, 3, 4, 5],
    2,
    async (value) => {
      active += 1;
      maximum = Math.max(maximum, active);
      await Promise.resolve();
      active -= 1;
      return value * 2;
    },
    controller.signal,
  );
  assert.deepEqual(output, [2, 4, 6, 8, 10]);
  assert.equal(maximum, 2);
});

test("mapConcurrent waits for started siblings before rejecting", async () => {
  let release;
  const gate = new Promise((resolve) => {
    release = resolve;
  });
  const batch = mapConcurrent(
    [1, 2],
    2,
    async (value) => {
      if (value === 1) {
        throw new Error("failed");
      }
      await gate;
      return value;
    },
    new AbortController().signal,
  );
  let settled = false;
  void batch.catch(() => {}).then(() => {
    settled = true;
  });
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(settled, false);
  release();
  await assert.rejects(batch, /failed/);
});

test("mapConcurrent observes cancellation", async () => {
  const controller = new AbortController();
  controller.abort(new Error("stop"));
  await assert.rejects(
    mapConcurrent([1], 1, async (value) => value, controller.signal),
    /stop/,
  );
});

test("withAbortHandler removes its listener after success and failure", async () => {
  for (const operation of [
    async () => "done",
    async () => { throw new Error("failed"); },
  ]) {
    let added = 0;
    let removed = 0;
    const signal = {
      aborted: false,
      addEventListener() { added += 1; },
      removeEventListener() { removed += 1; },
      throwIfAborted() {},
    };
    await withAbortHandler(signal, () => {}, operation).catch(() => undefined);
    assert.equal(added, 1);
    assert.equal(removed, 1);
  }
});

test("recoverLocalFailure observes rejection inside its catch", async () => {
  assert.equal(
    await recoverLocalFailure(() => Promise.reject(new Error("failure"))),
    "recovered",
  );
});

test("startDetached reports a rejection", async () => {
  const reported = new Promise((resolve) => {
    startDetached(
      () => Promise.reject(new Error("detached failure")),
      resolve,
    );
  });
  assert.match(String(await reported), /detached failure/);
});

test("Promise.all rejection does not cancel a sibling", async () => {
  let release;
  let completed = false;
  const gate = new Promise((resolve) => {
    release = resolve;
  });
  const sibling = gate.then(() => {
    completed = true;
  });

  await assert.rejects(
    Promise.all([Promise.reject(new Error("failure")), sibling]),
    /failure/,
  );
  assert.equal(completed, false);
  release();
  await sibling;
  assert.equal(completed, true);
});
