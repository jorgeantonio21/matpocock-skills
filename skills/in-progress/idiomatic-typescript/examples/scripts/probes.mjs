import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import ts from "typescript";

const root = mkdtempSync(join(tmpdir(), "idiomatic-typescript-probes-"));

function diagnostics(name, source, extra = {}) {
  const file = join(root, `${name}.ts`);
  writeFileSync(file, source);
  const program = ts.createProgram([file], {
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    noEmit: true,
    skipLibCheck: true,
    strict: true,
    target: ts.ScriptTarget.ES2022,
    ...extra,
  });
  return ts.getPreEmitDiagnostics(program).map((diagnostic) => diagnostic.code);
}

try {
  const indexed = "export {}; const xs: string[] = []; const x: string = xs[0];";
  assert.deepEqual(diagnostics("indexed-strict", indexed), []);
  assert.deepEqual(
    diagnostics("indexed-checked", indexed, { noUncheckedIndexedAccess: true }),
    [2322],
  );

  const optional = "export {}; const x: { name?: string } = { name: undefined };";
  assert.deepEqual(diagnostics("optional-strict", optional), []);
  assert.deepEqual(
    diagnostics("optional-exact", optional, { exactOptionalPropertyTypes: true }),
    [2375],
  );

  const erased = `export {};
const payload: unknown = { name: 42 };
const user = payload as { name: string };
function isString(value: unknown): value is string { return true; }
user.name.toUpperCase();
if (isString(payload)) payload.toUpperCase();`;
  assert.deepEqual(diagnostics("erased", erased), []);
  assert.throws(() => ({ name: 42 }).name.toUpperCase(), TypeError);

  const readonly = `export {};
const mutable = { count: 1 };
const view: Readonly<{ count: number }> = mutable;
mutable.count = 2;
void view;`;
  assert.deepEqual(diagnostics("readonly", readonly), []);

  const satisfies = `export {};
const constrained = { enabled: true } satisfies { enabled: boolean };
constrained.enabled = false;`;
  assert.deepEqual(diagnostics("satisfies", satisfies), [2322]);
  assert.deepEqual(
    diagnostics("satisfies-extra", "export {}; const x = { a: 1, b: 2 }; x satisfies { a: number };"),
    [],
  );

  assert.deepEqual(
    diagnostics(
      "predicate-inference",
      "export {}; const xs: number[] = [1, undefined, 0].filter(x => x !== undefined);",
    ),
    [],
  );

  console.log(`compiler probes pass (TypeScript ${ts.version}, 7 checks)`);
} finally {
  rmSync(root, { recursive: true, force: true });
}
