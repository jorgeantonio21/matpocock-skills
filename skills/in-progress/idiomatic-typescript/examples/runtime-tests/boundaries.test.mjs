import assert from "node:assert/strict";
import test from "node:test";

import { parseApiUser } from "../dist/index.js";

test("parseApiUser preserves meaningful falsy values", () => {
  assert.deepEqual(parseApiUser({ name: "", loginCount: 0, enabled: false }), {
    name: "",
    loginCount: 0,
    enabled: false,
  });
});

test("parseApiUser rejects malformed input", () => {
  assert.throws(
    () => parseApiUser({ name: "Ada", loginCount: "0", enabled: true }),
    /loginCount must be an integer/,
  );
  assert.throws(() => parseApiUser(null), /must be an object/);
});
