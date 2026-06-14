// SPEC-MANAGED: projects/rig/tech-design/semantic/source/projects-rig-src-verdict-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Verdict vocabulary + expected-outcome bucketing.
//!
//! Inherited from mamba's harness: a RAW result (did the scenario's steps
//! hold?) is bucketed through the record's `expected` declaration. An
//! xfail that fails never gates; an xfail that PASSES is surfaced as a
//! graduate-to-pass signal (xpass), never silently absorbed.

use crate::scenario::ExpectedOutcome;

/// Final per-scenario verdict after bucketing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-verdict-rs.md#source
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
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-verdict-rs.md#source
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
// CODEGEN-END
