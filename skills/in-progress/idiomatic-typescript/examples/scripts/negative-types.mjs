import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";

import ts from "typescript";

const root = fileURLToPath(new URL("..", import.meta.url));
const cases = [
  ["unchecked-index.ts", 2322],
  ["exact-optional.ts", 2375],
  ["exhaustiveness.ts", 2345],
  ["brand-role.ts", 2345],
  ["generic-call.ts", 2322],
  ["satisfies-context.ts", 2322],
];

for (const [name, expectedCode] of cases) {
  const file = `${root}/negative-types/${name}`;
  const program = ts.createProgram([file], {
    exactOptionalPropertyTypes: true,
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    noEmit: true,
    noUncheckedIndexedAccess: true,
    skipLibCheck: true,
    strict: true,
    target: ts.ScriptTarget.ES2022,
  });
  const diagnostics = ts.getPreEmitDiagnostics(program);
  assert.deepEqual(
    diagnostics.map((diagnostic) => diagnostic.code),
    [expectedCode],
    `${name} must fail only with TS${expectedCode}: ${diagnostics
      .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"))
      .join("\n")}`,
  );
}

console.log(`negative type fixtures fail with their intended diagnostics (${cases.length} cases)`);
