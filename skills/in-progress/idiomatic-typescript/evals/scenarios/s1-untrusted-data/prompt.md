The package reads user configuration from an untrusted JSON boundary. Implement `parseUser` without changing its exported name or `User` shape.

Requirements:
- accept the boundary value as `unknown`
- reject malformed objects and wrong field types with a thrown error
- preserve valid empty names, zero retries, and `false` enabled values
- add no dependency and preserve the existing valid-input behavior
- run the package's typecheck and tests
