Implement `runBatch` as the owner of a bounded asynchronous batch.

Requirements:
- preserve input order in the returned values
- never run more than `limit` operations at once
- reject a non-positive or non-integer limit
- pass the supplied `AbortSignal` to every operation and stop scheduling after abort
- propagate operation rejection to the caller
- wait for the work owned by the batch rather than detaching it
- add no dependency and run the package's typecheck and tests
