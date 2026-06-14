---
id: projects-rig-src-verdict-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/verdict.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/verdict.rs`, captured as a rust-source-unit (td_ast) item-tree
during rig standardization onto the codegen ladder.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Verdict vocabulary + expected-outcome bucketing.
//!
//! Inherited from mamba's harness: a RAW result (did the scenario's steps
//! hold?) is bucketed through the record's `expected` declaration. An
//! xfail that fails never gates; an xfail that PASSES is surfaced as a
//! graduate-to-pass signal (xpass), never silently absorbed.

use crate::scenario::ExpectedOutcome;

/// Final per-scenario verdict after bucketing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Expected pass, passed.
    Pass,
    /// Expected pass, failed — gates the run (if `required`).
    Red,
    /// Expected fail, failed — reported, never gates.
    Xfail,
    /// Expected fail, PASSED — graduate-to-pass signal, never gates.
    Xpass,
    /// Structurally skipped before execution.
    Skip,
}

/// Bucket a raw pass/fail through the declared expectation.
pub fn bucket(expected: ExpectedOutcome, raw_passed: bool) -> Verdict {
    match (expected, raw_passed) {
        (ExpectedOutcome::Skip, _) => Verdict::Skip,
        (ExpectedOutcome::Pass, true) => Verdict::Pass,
        (ExpectedOutcome::Pass, false) => Verdict::Red,
        (ExpectedOutcome::Xfail, false) => Verdict::Xfail,
        (ExpectedOutcome::Xfail, true) => Verdict::Xpass,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucketing_matrix() {
        assert_eq!(bucket(ExpectedOutcome::Pass, true), Verdict::Pass);
        assert_eq!(bucket(ExpectedOutcome::Pass, false), Verdict::Red);
        assert_eq!(bucket(ExpectedOutcome::Xfail, false), Verdict::Xfail);
        assert_eq!(bucket(ExpectedOutcome::Xfail, true), Verdict::Xpass);
        assert_eq!(bucket(ExpectedOutcome::Skip, true), Verdict::Skip);
        assert_eq!(bucket(ExpectedOutcome::Skip, false), Verdict::Skip);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/verdict.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/verdict.rs` captured during rig
      standardization onto the codegen ladder.
```
