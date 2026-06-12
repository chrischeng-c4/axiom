---
id: projects-meter-examples-junit-reporter-example-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/examples/junit_reporter_example.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/examples/junit_reporter_example.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! JUnit XML Reporter Example
//!
//! This example demonstrates how to use the JUnit XML reporter
//! to generate test reports for CI/CD integration.
//!
//! Run this example with:
//! ```bash
//! cargo run --example junit_reporter_example -p meter
//! ```

use meter::reporter::{Reporter, TestReport};
use meter::runner::{TestMeta, TestResult, TestStatus, TestType};
use std::fs;
use std::io::Write;

fn main() {
    println!("=== JUnit XML Reporter Example ===\n");

    // Create test results with various statuses. CI can request a passing-only
    // sample so report publishing validates formatting without failing the job.
    let results = if std::env::var_os("CCLAB_JUNIT_EXAMPLE_PASSING").is_some() {
        create_passing_example_test_results()
    } else {
        create_example_test_results()
    };

    // Create a test report
    let report = TestReport::new("ExampleTestSuite", results);

    // Generate JUnit XML output
    let reporter = Reporter::junit();
    let junit_xml = reporter.generate(&report);

    // Print the generated XML to console
    println!("Generated JUnit XML:\n");
    println!("{}", junit_xml);
    println!("\n{}", "=".repeat(80));

    // Write to file
    let output_path = "test-results.xml";
    match fs::File::create(output_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(junit_xml.as_bytes()) {
                eprintln!("Failed to write to file: {}", e);
            } else {
                println!("\nJUnit XML report written to: {}", output_path);
                println!("You can now upload this file to your CI/CD system.");
            }
        }
        Err(e) => eprintln!("Failed to create file: {}", e),
    }

    // Display summary
    println!("\nTest Summary:");
    println!("  Total:   {}", report.summary.total);
    println!("  Passed:  {}", report.summary.passed);
    println!("  Failed:  {}", report.summary.failed);
    println!("  Errors:  {}", report.summary.errors);
    println!("  Skipped: {}", report.summary.skipped);
    println!("  Duration: {:.3}s", report.duration_ms as f64 / 1000.0);

    // Show how to use with other formats
    println!("\n=== Other Report Formats ===\n");
    demonstrate_other_formats(&report);
}

/// Create example test results demonstrating different test statuses
fn create_example_test_results() -> Vec<TestResult> {
    vec![
        // Passed test - basic unit test
        {
            let meta = TestMeta::new("test_database_connection").with_type(TestType::Unit);
            TestResult::passed(meta, 150)
        },
        // Passed test - with file location
        {
            let mut meta = TestMeta::new("test_query_execution");
            meta.test_type = TestType::Unit;
            meta.file_path = Some("tests/database_tests.rs".to_string());
            meta.line_number = Some(45);
            meta.full_name = "database_tests::test_query_execution".to_string();
            TestResult::passed(meta, 230)
        },
        // Failed test - assertion failure
        {
            let mut meta = TestMeta::new("test_invalid_input");
            meta.test_type = TestType::Unit;
            meta.file_path = Some("tests/validation_tests.rs".to_string());
            meta.line_number = Some(102);
            meta.full_name = "validation_tests::test_invalid_input".to_string();

            TestResult {
                meta,
                status: TestStatus::Failed,
                duration_ms: 75,
                error: Some("Expected validation error, but operation succeeded".to_string()),
                stack_trace: Some(
                    "at validation_tests.rs:102\n\
                     at test_runner::execute_test\n\
                     at main"
                        .to_string(),
                ),
                profile_metrics: None,
                started_at: chrono::Utc::now().to_rfc3339(),
            }
        },
        // Error test - runtime error
        {
            let mut meta = TestMeta::new("test_network_timeout");
            meta.test_type = TestType::Unit;
            meta.file_path = Some("tests/network_tests.rs".to_string());
            meta.line_number = Some(67);
            meta.full_name = "network_tests::test_network_timeout".to_string();

            TestResult {
                meta,
                status: TestStatus::Error,
                duration_ms: 5000,
                error: Some("Connection timeout after 5000ms".to_string()),
                stack_trace: Some(
                    "NetworkError: Connection timeout\n\
                     at network_tests.rs:67\n\
                     at tokio::runtime::block_on"
                        .to_string(),
                ),
                profile_metrics: None,
                started_at: chrono::Utc::now().to_rfc3339(),
            }
        },
        // Skipped test
        {
            let mut meta = TestMeta::new("test_experimental_feature");
            meta.test_type = TestType::Unit;
            meta.skip_reason = Some("Feature not yet implemented".to_string());
            meta.file_path = Some("tests/experimental_tests.rs".to_string());
            meta.line_number = Some(15);

            TestResult {
                meta,
                status: TestStatus::Skipped,
                duration_ms: 0,
                error: Some("Feature not yet implemented".to_string()),
                stack_trace: None,
                profile_metrics: None,
                started_at: chrono::Utc::now().to_rfc3339(),
            }
        },
        // Another passed test
        {
            let meta = TestMeta::new("test_data_serialization").with_type(TestType::Unit);
            TestResult::passed(meta, 95)
        },
        // Another passed test with tags
        {
            let meta = TestMeta::new("test_concurrent_access")
                .with_type(TestType::Unit)
                .with_tags(vec!["concurrency".to_string()]);
            TestResult::passed(meta, 3200)
        },
    ]
}

fn create_passing_example_test_results() -> Vec<TestResult> {
    vec![
        {
            let meta = TestMeta::new("test_database_connection").with_type(TestType::Unit);
            TestResult::passed(meta, 150)
        },
        {
            let mut meta = TestMeta::new("test_query_execution");
            meta.test_type = TestType::Unit;
            meta.file_path = Some("tests/database_tests.rs".to_string());
            meta.line_number = Some(45);
            meta.full_name = "database_tests::test_query_execution".to_string();
            TestResult::passed(meta, 230)
        },
        {
            let meta = TestMeta::new("test_data_serialization").with_type(TestType::Unit);
            TestResult::passed(meta, 95)
        },
        {
            let meta = TestMeta::new("test_concurrent_access")
                .with_type(TestType::Unit)
                .with_tags(vec!["concurrency".to_string()]);
            TestResult::passed(meta, 3200)
        },
    ]
}

/// Demonstrate other report formats available
fn demonstrate_other_formats(report: &TestReport) {
    // Markdown format
    println!("1. Markdown Format:");
    println!("   Usage: Reporter::markdown().generate(&report)");
    println!("   Output: Human-readable markdown file");
    println!("   File: test-report.md\n");

    // JSON format
    println!("2. JSON Format:");
    println!("   Usage: Reporter::json().generate(&report)");
    println!("   Output: Machine-parseable JSON");
    println!("   File: test-report.json\n");

    // HTML format
    println!("3. HTML Format:");
    println!("   Usage: Reporter::html().generate(&report)");
    println!("   Output: Interactive HTML report");
    println!("   File: test-report.html\n");

    // YAML format
    println!("4. YAML Format:");
    println!("   Usage: Reporter::yaml().generate(&report)");
    println!("   Output: Human-readable YAML");
    println!("   File: test-report.yaml\n");

    // Console format
    println!("5. Console Format:");
    println!("   Usage: Reporter::console().generate(&report)");
    println!("   Output: Colored terminal output\n");

    // Generate a sample markdown report
    let markdown = Reporter::markdown().generate(report);
    if let Ok(mut file) = fs::File::create("test-report.md") {
        let _ = file.write_all(markdown.as_bytes());
        println!("   Sample markdown report written to: test-report.md");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/examples/junit_reporter_example.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/examples/junit_reporter_example.rs` captured during meter full-codegen standardization.
```
