import assert from "node:assert/strict";

import { parseApiUser, stateLabel } from "../../dist/index.js";

assert.equal(
  stateLabel({ status: "loaded", value: parseApiUser({ name: "Ada", loginCount: 0, enabled: true }).name }),
  "Ada",
);
