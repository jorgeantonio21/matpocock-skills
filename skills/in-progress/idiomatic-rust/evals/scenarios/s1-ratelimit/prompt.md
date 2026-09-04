Build a token-bucket rate limiter as a library crate in this directory. The crate is already initialised (`Cargo.toml` and an empty `src/lib.rs`); keep the name `ratelimit`.

Requirements:

- A limiter is configured with a bucket capacity (whole tokens) and a refill rate (tokens per second). Both arrive from a config file as plain numbers. A capacity of zero or a refill rate of zero must be impossible to construct.
- Buckets are per client. A client is identified by an opaque string taken from the request.
- `admit` decides whether a request that costs a given number of tokens is admitted now. When it is not, the caller needs to know how long to wait before retrying. A request whose cost exceeds the capacity can never be admitted, and the caller must be able to tell that case apart.
- A client idle for longer than a configured idle period is evicted, so the map does not grow forever. Provide an eviction operation the owner calls periodically; it returns how many clients it evicted.
- Everything must be testable without sleeping.
- Standard library only, plus `thiserror` if you want it (add it with `cargo add`).

Include unit tests. `cargo test` must pass when you finish. Write the code you would put in a pull request.
