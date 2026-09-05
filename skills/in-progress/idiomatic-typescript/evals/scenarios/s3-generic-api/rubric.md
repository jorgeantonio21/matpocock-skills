# s3 rubric

| Row | Pass condition |
| --- | --- |
| Relationship | The key type connects the callback result to the returned map |
| Constraint | The key type is constrained to `PropertyKey` or an equivalent exact contract |
| Inference | Literal or narrower key types survive in the returned type |
| Runtime | Last value wins for duplicate keys, as before |
| Restraint | No overload set, class, dependency, or unrelated helper |
