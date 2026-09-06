import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { rmSync, writeFileSync } from "node:fs";
import test from "node:test";

import { greet } from "idiomatic-typescript-eval-fixture";

test("Node imports the emitted output through package exports", () => {
  assert.equal(greet("Ada"), "Hello, Ada!");
});

for (const [module, resolution] of [["NodeNext", "NodeNext"], ["ESNext", "Bundler"]]) {
  test(`a ${resolution} consumer uses declarations through package exports`, () => {
    const consumer = `tests/generated-${resolution}-consumer.ts`;
    writeFileSync(
      consumer,
      `import { greet } from "idiomatic-typescript-eval-fixture";
const value: string = greet("Ada");
// @ts-expect-error: the exported API accepts a name string
void greet(42);
void value;
`,
    );
    try {
      execFileSync(
        "node_modules/.bin/tsc",
        [
          "--noEmit",
          "--strict",
          "--target", "ES2022",
          "--module", module,
          "--moduleResolution", resolution,
          consumer,
        ],
        { stdio: "pipe" },
      );
    } finally {
      rmSync(consumer, { force: true });
    }
  });
}
