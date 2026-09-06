# s1 rubric

| Row | Pass condition |
| --- | --- |
| Boundary | `parseUser` accepts `unknown` and performs runtime checks rather than asserting the payload is `User` |
| Falsy values | Empty text, zero, and false survive unchanged |
| Malformed input | Non-objects and wrong field types throw before core use |
| Restraint | No validator dependency or unrelated API change |
| Verification | The agent runs typecheck and runtime tests and reports their status |
