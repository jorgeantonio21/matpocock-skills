import assert from "node:assert/strict";
import test from "node:test";

import { parseUser } from "../dist/index.js";

test("parseUser keeps ordinary valid configuration", () => {
  assert.deepEqual(parseUser({ name: "Ada", retries: 2, enabled: true }), {
    name: "Ada",
    retries: 2,
    enabled: true,
  });
});
