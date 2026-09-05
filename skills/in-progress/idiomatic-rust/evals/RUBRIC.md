# Independent evaluation dimensions

Score each arm and repetition separately. Read the scenario prompt as the contract, then its external `oracle.rs`. The agent receives only prompt and start tree; the scorer injects the oracle afterward into a disposable copy. Agent-written tests and compiler checks are separate evidence. Compilation failure or missing infrastructure is incomplete evidence, never zero findings or a pass.

| Dimension | Evidence | Score |
| --- | --- | --- |
| Independent correctness | External tests for accepted inputs, rejected inputs, boundaries, and historical behavior | Passed/total plus failures and log path |
| Maintainability | Diff scope, clarity of guarantees, error usefulness, API compatibility | Acceptable / concern, with concrete example |
| Review quality | Required semantic findings and invalid mandatory findings on valid code | Recall and false positives, separately |
| Generality | Ownership, dispatch, dependencies, and concurrency relative to each scenario's requirements | Appropriate / imposed architecture, with reason |
| Cost | Resolved model, wall seconds, turns, reported dollars from transcript | Numeric values; missing data remains unknown |
| Rule adherence | Lints, conventions, skill reads, formatting | Separate diagnostic data, never correctness credit |

For s6, check all constructor/parse/decoder routes and signed minimum arithmetic. Borrowed decoding with the existing Serde dependency is sufficient; no backend or runtime is needed. For s7, preserve previously admitted work through stricter and zero limits, return rejected input unchanged, and return authoritative removal data. Existing exclusive ownership is sufficient; extra shared-state or async infrastructure needs a reason. For s8, read literal historical bytes, preserve Unlimited, reject corrupt input, verify new writes and stdout. Preserve straightforward `label` and explicit result types if abstractions add no benefit. Keeping the three workloads architecturally different is a successful result.

The original s1 through s5 rubrics and s2 answer key record the historical **rule-adherence** probe. Their mandatory wrappers, ecosystem preferences, and architectural rewrites are not independent correctness criteria. Preserve those records for comparison; use this rubric for new conclusions. For s9, use its scenario rubric for false positives and recall.

## Real-ticket gate

Before promotion, choose a public Rust ticket with an agreed contract and reproducible start commit. Record its URL, revision, license, setup, independent acceptance tests, and supported toolchain. Keep source-specific research separate from shipped fixtures. Run bare, merged, and revised arms against the same ticket and model at least twice, and publish the same dimensions above with artifact hashes. A self-authored synthetic fixture is not a real-ticket probe. Until that evidence exists, promotion is not justified.
