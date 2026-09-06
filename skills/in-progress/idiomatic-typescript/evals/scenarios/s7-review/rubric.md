# s7 review rubric

## Required findings

1. **Unknown in, checked value out, requirement:** `parseSettings` asserts parsed JSON without runtime checks. The review must describe malformed input reaching the returned `Settings` and show a checked parser or existing-schema rewrite.
2. **Sparse lookup, requirement:** `endpointUrl` uses `Record<string, Endpoint>` as if every arbitrary key exists. The review must expose absence in the return or check it before `.url`.

## Valid alternatives that are not findings

- `interface Settings` is a normal public object contract.
- `enum WireMode` is an existing serialized wire contract, documented at the declaration.
- The overloads correlate string input with string output and byte input with byte output.
- The narrow assertion in `handlerFor` connects a generic key to a closed internal object after exact indexed access. It adds no runtime claim.

## Scoring

Count required findings found, false positives against the four valid alternatives, duplicate findings, and whether every finding includes a rule, kind, consequence, and rewrite. Any source edit fails review-only restraint.
