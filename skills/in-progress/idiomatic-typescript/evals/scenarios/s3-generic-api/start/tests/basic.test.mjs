import assert from "node:assert/strict";
import test from "node:test";

import { indexBy } from "../dist/index.js";

test("indexBy keeps the last value for a duplicate key", () => {
  const result = indexBy(
    [{ id: "one", value: 1 }, { id: "one", value: 2 }],
    (value) => value.id,
  );
  assert.equal(result.get("one")?.value, 2);
});
