# s4 rubric

| Row | Pass condition |
| --- | --- |
| Ownership | `runBatch` returns only after its scheduled workers settle and propagates rejection |
| Bound | Observed concurrency never exceeds `limit` |
| Order | Result positions match input positions |
| Cancellation | The signal reaches operations and no new item is scheduled after abort is observed |
| Validation | Invalid limits reject before work starts |
| Restraint | No queue or promise utility dependency is added |
