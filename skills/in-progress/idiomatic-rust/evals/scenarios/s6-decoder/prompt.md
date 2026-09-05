Extend this frame codec crate.

- Add `Header::encode(self) -> [u8; Header::SIZE]`, the inverse of `Header::decode`.
- Add `pub fn load_capture(json: &str) -> Result<Vec<Frame>, CaptureError>`, which reads a JSON array of frames as the capture tool writes them: `[{"header": {"version": 1, "priority": 2, "payload_len": 3}, "payload": [1, 2, 3]}]`. Define `CaptureError`.
- Add `Delta::magnitude(self) -> u32`, the unsigned distance, and `Delta::checked_invert(self) -> Option<Delta>`, which returns `None` when the opposite delta does not fit in an `i32`.

The doc comments on `Header`, `Frame`, `Priority`, `PayloadLen`, and `Delta` state invariants. Every value the crate hands out must satisfy them, whichever route built it. Keep the existing public API. Add tests. `cargo test` must pass when you finish. Write the code you would put in a pull request.
