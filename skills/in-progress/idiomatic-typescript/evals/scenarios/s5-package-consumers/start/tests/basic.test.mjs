import assert from "node:assert/strict";
import test from "node:test";

test("package metadata declares ESM", async () => {
  const packageJson = await import("../package.json", { with: { type: "json" } });
  assert.equal(packageJson.default.type, "module");
});
