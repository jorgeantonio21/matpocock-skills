# Review correctness and false positives

Required findings: `[1, 0]` is incorrectly changed to Bounded(1); `[2, 0]` is incorrectly accepted. Count these as two independent semantic regressions even if grouped in one finding. Accept an exact demonstration by test or clear input/output reasoning.

False positives: calling stdout, the two free-text parameters, the simple label function, explicit Result types, Box<dyn Error> in main, or synchronous execution a correctness defect. Optional suggestions are not false positives unless presented as mandatory fixes. Zero findings is not success: report recall (0..2) alongside false-positive count. Verify the tree is unchanged.

Record cost and latency separately. Lint compliance, citations of skill rules, and proposed newtypes do not increase correctness credit.
