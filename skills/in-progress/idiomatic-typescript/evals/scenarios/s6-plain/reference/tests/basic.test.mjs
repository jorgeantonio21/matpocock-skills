import assert from "node:assert/strict";
import test from "node:test";

import { effectiveTimeout } from "../dist/index.js";

test("positive timeout overrides the fallback", () => {
  assert.equal(effectiveTimeout({ timeoutMs: 25 }, 100), 25);
});

test("omitted timeout uses the fallback", () => {
  assert.equal(effectiveTimeout({}, 100), 100);
});
