import assert from "node:assert/strict";
import test from "node:test";

import { effectiveTimeout } from "../dist/index.js";

test("zero timeout remains zero", () => {
  assert.equal(effectiveTimeout({ timeoutMs: 0 }, 100), 0);
});
