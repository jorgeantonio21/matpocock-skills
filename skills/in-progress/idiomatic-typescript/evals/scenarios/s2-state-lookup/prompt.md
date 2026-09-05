Extend the request state module with a failed state carrying an `Error`, and make arbitrary status-label lookup expose a missing key.

Requirements:
- preserve the exported `RequestState`, `describeState`, and `labelForStatus` names
- make each state's data available only on that state
- keep idle, loading, and loaded behavior unchanged
- describe a failed state with the error message
- return `undefined` for an unknown label key
- make adding another local state force `describeState` to make a decision
- run the package's typecheck and tests
