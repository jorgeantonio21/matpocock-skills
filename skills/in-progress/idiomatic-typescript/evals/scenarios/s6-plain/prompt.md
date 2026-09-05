Fix the timeout defect in this otherwise well-written module. A configured timeout of zero means no waiting and must remain zero. An omitted timeout uses the supplied fallback.

Preserve the exported function and types. Keep the change as small as the behavior requires, add no dependency, and run the package's typecheck and tests.
