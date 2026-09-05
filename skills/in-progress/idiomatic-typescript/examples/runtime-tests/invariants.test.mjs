import assert from "node:assert/strict";
import test from "node:test";

import {
  isNonEmptyString,
  observeReadonlyAlias,
  parseUserId,
} from "../dist/index.js";

test("parseUserId establishes the property recorded by the brand", () => {
  assert.equal(parseUserId("usr_ada1"), "usr_ada1");
  assert.throws(() => parseUserId("ada"), /must match/);
});

test("the custom predicate has positive and negative behavior", () => {
  assert.equal(isNonEmptyString("Ada"), true);
  assert.equal(isNonEmptyString(""), false);
  assert.equal(isNonEmptyString(1), false);
});

test("a readonly view observes mutation through a writable alias", () => {
  assert.deepEqual(observeReadonlyAlias(), [1, 2]);
});
