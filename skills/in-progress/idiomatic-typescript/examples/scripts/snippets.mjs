import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";

const examples = fileURLToPath(new URL("..", import.meta.url));
const skill = fileURLToPath(new URL("../..", import.meta.url));
const sources = readdirSync(`${examples}/src`, { recursive: true })
  .filter((name) => name.endsWith(".ts"))
  .map((name) => readFileSync(`${examples}/src/${name}`, "utf8"))
  .join("\n");

let count = 0;
for (const name of ["SKILL.md", "INVARIANTS.md", "RUNTIME.md"]) {
  const text = readFileSync(`${skill}/${name}`, "utf8");
  for (const match of text.matchAll(/```ts\n([\s\S]*?)\n```/g)) {
    count += 1;
    assert.ok(
      sources.includes(match[1]),
      `${name} contains a TypeScript block that is not a verbatim examples/src excerpt:\n${match[1]}`,
    );
  }
}
assert.ok(count > 0, "the guidance must contain checked TypeScript snippets");
console.log(`${count} guidance snippets are verbatim excerpts of examples/src`);
