Tighten the public `indexBy` helper without changing its name or last-value-wins runtime behavior.

Requirements:
- infer the exact property-key type returned by the callback
- reject callbacks that return objects or other values unusable as property keys
- keep the input collection readonly
- use only type parameters that express a relationship in the signature
- keep the implementation small and add no dependency
- run the package's typecheck and tests
