---
id: projects-meter-tests-junit-example-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/tests/junit_example.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/tests/junit_example.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/tests/junit_example.rs -->
````rust
use meter::reporter::{Reporter, TestReport};
use meter::runner::{TestMeta, TestResult, TestStatus};

#[test]
fn show_enhanced_junit_xml() {
    // Create test results with file_path and line_number
    let mut meta1 = TestMeta::new("test_example_pass");
    meta1.file_path = Some("src/lib.rs".to_string());
    meta1.line_number = Some(42);
    meta1.full_name = "my_module::test_example_pass".to_string();

    let mut meta2 = TestMeta::new("test_example_fail");
    meta2.file_path = Some("src/test.rs".to_string());
    meta2.line_number = Some(100);
    meta2.full_name = "my_module::test_example_fail".to_string();

    let mut meta3 = TestMeta::new("test_example_error");
    meta3.file_path = Some("src/error.rs".to_string());
    meta3.line_number = Some(200);
    meta3.full_name = "my_module::test_example_error".to_string();

    let results = vec![
        TestResult::passed(meta1, 150),
        TestResult::failed(meta2, 75, "Assertion failed: expected 5, got 3"),
        TestResult {
            meta: meta3,
            status: TestStatus::Error,
            duration_ms: 25,
            error: Some("Runtime error occurred".to_string()),
            stack_trace: Some("at line 200\nat line 195".to_string()),
            profile_metrics: None,
            stress_metrics: None,
            started_at: chrono::Utc::now().to_rfc3339(),
        },
    ];

    let report = TestReport::new("ExampleTestSuite", results);
    let reporter = Reporter::junit();
    let xml = reporter.generate(&report);

    println!("\n========== Enhanced JUnit XML Output ==========\n");
    println!("{}", xml);
    println!("\n===============================================\n");
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/tests/junit_example.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/tests/junit_example.rs` captured during meter full-codegen standardization.
```
