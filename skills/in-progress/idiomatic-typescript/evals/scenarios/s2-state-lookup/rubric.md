# s2 rubric

| Row | Pass condition |
| --- | --- |
| State shape | A discriminated union puts `value` and `error` only on their variants |
| Exhaustiveness | A new local variant reaches a `never` check or an equivalent compile-time decision point |
| Sparse lookup | The arbitrary key API returns `string | undefined` honestly |
| Compatibility | Existing idle, loading, and loaded output remains unchanged |
| Restraint | No class, brand, dependency, or unrelated public API churn |
