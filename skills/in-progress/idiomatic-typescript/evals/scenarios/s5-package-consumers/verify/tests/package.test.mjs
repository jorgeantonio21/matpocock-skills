import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { rmSync, writeFileSync } from "node:fs";
import test from "node:test";

import { greet } from "../dist/index.js";

test("Node imports the emitted package output", () => {
  assert.equal(greet("Ada"), "Hello, Ada!");
});

test("a bundler-mode consumer uses the emitted declarations", () => {
  const consumer = "tests/generated-consumer.ts";
  writeFileSync(
    consumer,
    'import { greet } from "../dist/index.js";\nconst value: string = greet("Ada");\nvoid value;\n',
  );
  try {
    execFileSync(
      "node_modules/.bin/tsc",
      [
        "--noEmit",
        "--strict",
        "--target", "ES2022",
        "--module", "ESNext",
        "--moduleResolution", "Bundler",
        consumer,
      ],
      { stdio: "pipe" },
    );
  } finally {
    rmSync(consumer, { force: true });
  }
});
