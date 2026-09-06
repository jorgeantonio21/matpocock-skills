import assert from "node:assert/strict";
import test from "node:test";

import { echo, endpointUrl, handlerFor } from "../dist/index.js";

test("established APIs work for valid input", () => {
  assert.equal(echo("hello"), "hello");
  assert.equal(endpointUrl({ api: { url: "https://example.test" } }, "api"), "https://example.test");
  assert.equal(handlerFor("start")(), "started");
});
