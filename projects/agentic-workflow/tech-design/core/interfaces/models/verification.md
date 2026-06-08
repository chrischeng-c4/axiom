---
id: sdd-models-verification
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Verification

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/models/verification.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Task` | projects/agentic-workflow/src/models/verification.rs | struct | pub | 123 |  |
| `TaskStatus` | projects/agentic-workflow/src/models/verification.rs | enum | pub | 56 |  |
| `TestResult` | projects/agentic-workflow/src/models/verification.rs | struct | pub | 83 |  |
| `TestStatus` | projects/agentic-workflow/src/models/verification.rs | enum | pub | 12 |  |
| `Verification` | projects/agentic-workflow/src/models/verification.rs | struct | pub | 151 |  |
| `VerificationCoverage` | projects/agentic-workflow/src/models/verification.rs | struct | pub | 135 |  |
| `all_passed` | projects/agentic-workflow/src/models/verification.rs | function | pub | 215 | all_passed(&self) -> bool |
| `completion_percentage` | projects/agentic-workflow/src/models/verification.rs | function | pub | 220 | completion_percentage(&self) -> u8 |
| `emoji` | projects/agentic-workflow/src/models/verification.rs | function | pub | 30 | emoji(&self) -> &'static str |
| `emoji` | projects/agentic-workflow/src/models/verification.rs | function | pub | 71 | emoji(&self) -> &'static str |
| `from_markdown` | projects/agentic-workflow/src/models/verification.rs | function | pub | 189 | from_markdown(s: &str) -> Self |
| `name` | projects/agentic-workflow/src/models/verification.rs | function | pub | 40 | name(&self) -> &'static str |
| `new` | projects/agentic-workflow/src/models/verification.rs | function | pub | 103 | new(         name: impl Into<String>,         status: TestStatus,         requirement: impl Into<String>,         scenario: impl Into<String>,     ) -> Self |
| `new` | projects/agentic-workflow/src/models/verification.rs | function | pub | 169 | new(change_id: impl Into<String>) -> Self |
| `pass_rate` | projects/agentic-workflow/src/models/verification.rs | function | pub | 202 | pass_rate(&self) -> f64 |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  TestStatus:
    type: string
    enum:
      - Pass
      - Fail
      - Skip
      - Pending
    description: |
      Status of a single test result.
      Pass: test passed successfully.
      Fail: test failed.
      Skip: test was skipped.
      Pending: test has not yet run.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      variants:
        - name: Pass
          doc: "Test passed successfully."
        - name: Fail
          doc: "Test failed."
        - name: Skip
          doc: "Test was skipped."
        - name: Pending
          doc: "Test has not yet run."
    x-methods:
      - name: emoji
        returns: "&'static str"
        impl_mode: codegen
        doc: "Emoji symbol representing the test status."
        dispatch:
          - variant: Pass
            value: "✅"
          - variant: Fail
            value: "❌"
          - variant: Skip
            value: "⏭️"
          - variant: Pending
            value: "⏸️"
      - name: name
        returns: "&'static str"
        impl_mode: codegen
        doc: "Human-readable display name for the test status."
        dispatch:
          - variant: Pass
            value: "PASS"
          - variant: Fail
            value: "FAIL"
          - variant: Skip
            value: "SKIP"
          - variant: Pending
            value: "PENDING"

  TaskStatus:
    type: string
    enum:
      - Pending
      - InProgress
      - Completed
    description: |
      Implementation task status.
      Pending: not yet started.
      InProgress: currently being worked on.
      Completed: finished.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      variants:
        - name: Pending
          doc: "Not yet started."
        - name: InProgress
          doc: "Currently being worked on."
        - name: Completed
          doc: "Finished."
    x-methods:
      - name: emoji
        returns: "&'static str"
        impl_mode: codegen
        doc: "Emoji symbol representing the task status."
        dispatch:
          - variant: Pending
            value: "⬜"
          - variant: InProgress
            value: "🔄"
          - variant: Completed
            value: "✅"
      - name: from_markdown
        returns: TaskStatus
        impl_mode: hand-written
        doc: |
          Parse a markdown checkbox string into a TaskStatus.
          Recognises: "[ ]" → Pending, "[>]" → InProgress, "[x]"/"[X]" → Completed.
          Any unrecognised string maps to Pending.
          Hand-written: multi-arm string match with two aliases for Completed.
        args:
          - { name: s, rust_type: "&str" }

  TestResult:
    type: object
    required: [name, status, requirement, scenario]
    description: Result of running a single test case.
    properties:
      name:
        type: string
        description: "Name of the test."
      status:
        $ref: "#/definitions/TestStatus"
        description: "Test execution status."
      error:
        type: string
        description: "Error message if the test failed."
      duration_ms:
        type: integer
        minimum: 0
        description: "Test execution duration in milliseconds."
      requirement:
        type: string
        description: "Requirement ID this test validates."
      scenario:
        type: string
        description: "Scenario ID this test validates."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
    x-constructor:
      name: new
      doc: "Create a new test result."
      impl_mode: codegen
      args:
        - { name: name,        rust_type: "impl Into<String>", into: String }
        - { name: status,      rust_type: TestStatus }
        - { name: requirement, rust_type: "impl Into<String>", into: String }
        - { name: scenario,    rust_type: "impl Into<String>", into: String }
      init:
        error: "None"
        duration_ms: "None"
    x-builders:
      - name: with_error
        doc: "Set the error message."
        impl_mode: codegen
        args:
          - { name: error, rust_type: "impl Into<String>", into: String }
        sets:
          - field: error
            wrap: Some
      - name: with_duration
        doc: "Set the execution duration in milliseconds."
        impl_mode: codegen
        args:
          - { name: ms, rust_type: u64 }
        sets:
          - field: duration_ms
            wrap: Some

  Task:
    type: object
    required: [id, description, status]
    description: Implementation task tracking an item from the change plan.
    properties:
      id:
        type: string
        description: "Task ID (e.g., \"1.1\", \"2.3\")."
      description:
        type: string
        description: "Human-readable task description."
      status:
        $ref: "#/definitions/TaskStatus"
        description: "Current task status."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  VerificationCoverage:
    type: object
    required: [requirements_tested, requirements_total, scenarios_tested, scenarios_total, pass_rate]
    description: Coverage statistics for a verification run.
    properties:
      requirements_tested:
        type: integer
        minimum: 0
        description: "Number of requirements that have at least one test."
      requirements_total:
        type: integer
        minimum: 0
        description: "Total number of requirements tracked."
      scenarios_tested:
        type: integer
        minimum: 0
        description: "Number of scenarios that have at least one test."
      scenarios_total:
        type: integer
        minimum: 0
        description: "Total number of scenarios tracked."
      pass_rate:
        type: number
        minimum: 0.0
        maximum: 1.0
        description: "Fraction of tests that passed (0.0–1.0)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Verification:
    type: object
    required: [change_id, verified_at, tests, tasks, issues, coverage]
    description: Full verification report for a change, generated by Codex.
    properties:
      change_id:
        type: string
        description: "Slug of the change that was verified."
      verified_at:
        type: string
        description: "ISO 8601 timestamp when verification was run."
      tests:
        type: array
        items:
          $ref: "#/definitions/TestResult"
        description: "Individual test results."
      tasks:
        type: array
        items:
          $ref: "#/definitions/Task"
        description: "Task progress items."
      issues:
        type: array
        items:
          type: string
        description: "Free-text issues found during verification."
      coverage:
        $ref: "#/definitions/VerificationCoverage"
        description: "Requirement and scenario coverage statistics."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
    x-constructor:
      name: new
      doc: "Create a new verification report for the given change ID. Sets verified_at to the current local time and zeroes all counters."
      impl_mode: codegen
      args:
        - { name: change_id, rust_type: "impl Into<String>", into: String }
      init:
        verified_at: "chrono::Local::now().to_rfc3339()"
        tests: "Vec::new()"
        tasks: "Vec::new()"
        issues: "Vec::new()"
        coverage: "VerificationCoverage { requirements_tested: 0, requirements_total: 0, scenarios_tested: 0, scenarios_total: 0, pass_rate: 0.0 }"
    x-methods:
      - name: pass_rate
        returns: f64
        impl_mode: hand-written
        doc: |
          Fraction of tests with status Pass (0.0 if no tests).
          Hand-written: iterates self.tests, filters by TestStatus::Pass, divides.
      - name: all_passed
        returns: bool
        impl_mode: hand-written
        doc: |
          True iff every test has status Pass.
          Hand-written: uses Iterator::all over self.tests.
      - name: completion_percentage
        returns: u8
        impl_mode: hand-written
        doc: |
          Percentage of tasks with status Completed (0 if no tasks).
          Hand-written: counts Completed tasks, divides by total, multiplies by 100.
```
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/models/verification.rs -->
```rust
use serde::{Deserialize, Serialize};

/// Status of a single test result.
/// Pass: test passed successfully.
/// Fail: test failed.
/// Skip: test was skipped.
/// Pending: test has not yet run.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/verification.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestStatus {
    /// Test passed successfully.
    #[serde(rename = "Pass")]
    Pass,
    /// Test failed.
    #[serde(rename = "Fail")]
    Fail,
    /// Test was skipped.
    #[serde(rename = "Skip")]
    Skip,
    /// Test has not yet run.
    #[serde(rename = "Pending")]
    Pending,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/verification.md#schema.impls
impl TestStatus {
    /// Emoji symbol representing the test status.
    pub fn emoji(&self) -> &'static str {
        match self {
            TestStatus::Pass => "✅",
            TestStatus::Fail => "❌",
            TestStatus::Skip => "⏭️",
            TestStatus::Pending => "⏸️",
        }
    }

    /// Human-readable display name for the test status.
    pub fn name(&self) -> &'static str {
        match self {
            TestStatus::Pass => "PASS",
            TestStatus::Fail => "FAIL",
            TestStatus::Skip => "SKIP",
            TestStatus::Pending => "PENDING",
        }
    }
}

/// Implementation task status.
/// Pending: not yet started.
/// InProgress: currently being worked on.
/// Completed: finished.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/verification.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Not yet started.
    #[serde(rename = "Pending")]
    Pending,
    /// Currently being worked on.
    #[serde(rename = "InProgress")]
    InProgress,
    /// Finished.
    #[serde(rename = "Completed")]
    Completed,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/verification.md#schema.impls
impl TaskStatus {
    /// Emoji symbol representing the task status.
    pub fn emoji(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "⬜",
            TaskStatus::InProgress => "🔄",
            TaskStatus::Completed => "✅",
        }
    }
}

/// Result of running a single test case.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/verification.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Name of the test.
    pub name: String,
    /// Test execution status.
    pub status: TestStatus,
    /// Error message if the test failed.
    #[serde(default)]
    pub error: Option<String>,
    /// Test execution duration in milliseconds.
    #[serde(default)]
    pub duration_ms: Option<u64>,
    /// Requirement ID this test validates.
    pub requirement: String,
    /// Scenario ID this test validates.
    pub scenario: String,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/verification.md#schema.impls
impl TestResult {
    /// Create a new test result.
    pub fn new(
        name: impl Into<String>,
        status: TestStatus,
        requirement: impl Into<String>,
        scenario: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            status,
            requirement: requirement.into(),
            scenario: scenario.into(),
            error: None,
            duration_ms: None,
        }
    }
}

/// Implementation task tracking an item from the change plan.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/verification.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task ID (e.g., "1.1", "2.3").
    pub id: String,
    /// Human-readable task description.
    pub description: String,
    /// Current task status.
    pub status: TaskStatus,
}

/// Coverage statistics for a verification run.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/verification.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCoverage {
    /// Number of requirements that have at least one test.
    pub requirements_tested: u64,
    /// Total number of requirements tracked.
    pub requirements_total: u64,
    /// Number of scenarios that have at least one test.
    pub scenarios_tested: u64,
    /// Total number of scenarios tracked.
    pub scenarios_total: u64,
    /// Fraction of tests that passed (0.0–1.0).
    pub pass_rate: f64,
}

/// Full verification report for a change, generated by Codex.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/verification.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    /// Slug of the change that was verified.
    pub change_id: String,
    /// ISO 8601 timestamp when verification was run.
    pub verified_at: String,
    /// Individual test results.
    pub tests: Vec<TestResult>,
    /// Task progress items.
    pub tasks: Vec<Task>,
    /// Free-text issues found during verification.
    pub issues: Vec<String>,
    /// Requirement and scenario coverage statistics.
    pub coverage: VerificationCoverage,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/verification.md#schema.impls
impl Verification {
    /// Create a new verification report for the given change ID. Sets verified_at to the current local time and zeroes all counters.
    pub fn new(change_id: impl Into<String>) -> Self {
        Self {
            change_id: change_id.into(),
            verified_at: chrono::Local::now().to_rfc3339(),
            tests: Vec::new(),
            tasks: Vec::new(),
            issues: Vec::new(),
            coverage: VerificationCoverage {
                requirements_tested: 0,
                requirements_total: 0,
                scenarios_tested: 0,
                scenarios_total: 0,
                pass_rate: 0.0,
            },
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/verification.md#source
impl TaskStatus {
    pub fn from_markdown(s: &str) -> Self {
        match s.trim() {
            "[ ]" => TaskStatus::Pending,
            "[>]" => TaskStatus::InProgress,
            "[x]" | "[X]" => TaskStatus::Completed,
            _ => TaskStatus::Pending,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/verification.md#source
impl Verification {
    /// Calculate overall pass rate
    pub fn pass_rate(&self) -> f64 {
        if self.tests.is_empty() {
            return 0.0;
        }
        let passed = self
            .tests
            .iter()
            .filter(|t| t.status == TestStatus::Pass)
            .count();
        passed as f64 / self.tests.len() as f64
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.tests.iter().all(|t| t.status == TestStatus::Pass)
    }

    /// Get completion percentage (0-100)
    pub fn completion_percentage(&self) -> u8 {
        if self.tasks.is_empty() {
            return 0;
        }
        let completed = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        ((completed as f64 / self.tasks.len() as f64) * 100.0) as u8
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/verification.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete verification model module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 2
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [changes] Both Round 1 findings are resolved: `"impl Verification"` is now present in `replaces:`, and the NOTE explicitly names both `impl TaskStatus` and `impl Verification` as requiring prepare commits before gen-code runs. The hand-written second entry correctly documents the two surviving impl blocks outside CODEGEN-BEGIN/END. Spec is ready for implementation.

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** needs-revision

- [changes] `"impl Verification"` is absent from `replaces:`, but the actual source file has a single `impl Verification` block containing all four methods (`new`, `pass_rate`, `all_passed`, `completion_percentage`). Codegen will emit `impl Verification { new }` inside CODEGEN-BEGIN/END while the original combined block remains untouched (not listed in `replaces:`), resulting in `new` being defined twice — a Rust compile error. Fix: require a prepare commit that splits the source `impl Verification` into two separate blocks (`impl Verification { new }` for codegen and `impl Verification { pass_rate, all_passed, completion_percentage }` for hand-written), then add `"impl Verification"` to `replaces:` so the first occurrence (the codegen block) is targeted. The NOTE in the description already establishes this exact pattern for `impl TaskStatus`; extend it to cover `impl Verification` as well.
- [changes] The prepare-commit NOTE must be expanded to explicitly name both mixed-mode impl blocks requiring a pre-split: `impl TaskStatus` (already listed) and `impl Verification` (currently omitted). An implementer reading only the NOTE would split `impl TaskStatus` but leave `impl Verification` unsplit, causing the compile error described above.
