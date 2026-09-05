import assert from "node:assert/strict";
import test from "node:test";

import { parseUser } from "../dist/index.js";

test("valid falsy values survive the boundary", () => {
  assert.deepEqual(parseUser({ name: "", retries: 0, enabled: false }), {
    name: "",
    retries: 0,
    enabled: false,
  });
});

test("malformed boundary values are rejected", () => {
  assert.throws(() => parseUser(null));
  assert.throws(() => parseUser({ name: "Ada", retries: "2", enabled: true }));
  assert.throws(() => parseUser({ name: "Ada", retries: 2, enabled: "yes" }));
});
