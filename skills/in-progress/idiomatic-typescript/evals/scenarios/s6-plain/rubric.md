# s6 rubric

| Row | Pass condition |
| --- | --- |
| Correctness | Zero remains zero and omission uses the fallback |
| Minimality | The implementation change is the direct nullish check or equivalent |
| API stability | `TimeoutConfig` and `effectiveTimeout` remain compatible |
| Generality | No brand, class, schema, generic, result type, or dependency appears |
| Diff size | Source change is ideally one operator and tests only cover the defect |
