---
id: project-health-test-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Project health generated surfaces implement standardization readiness reporting and gate evidence."
---

# Project Health Test Source Template

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/project_health_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/project_health_test.rs -->
```rust
use std::collections::BTreeMap;

use agentic_workflow::cli::cb::{CbCodegenOriginSummary, CbColdVerifySummary, CbVerifySummary};
use agentic_workflow::cli::project::{ProjectHealthReport, ProjectHealthStatus};
use agentic_workflow::cli::standardize::{
    DependencyPolicyFinding, GeneratorPrimitiveGap, MarkerCounts, RegenerabilityCoverage,
    SemanticCoverage, SemanticGap, StackMigrationCoverage, StandardizationCoverage,
    WorkspaceStackMigration,
};

fn managed(percent: f64, uncovered_files: Vec<String>) -> StandardizationCoverage {
    StandardizationCoverage {
        scope: vec!["projects/demo/**".to_string()],
        total_files: 2,
        managed_files: 2 - uncovered_files.len(),
        percent,
        by_language: BTreeMap::new(),
        by_marker: MarkerCounts {
            codegen: 2 - uncovered_files.len(),
            handwrite: 0,
        },
        uncovered_files,
    }
}

fn regenerable(
    percent: f64,
    handwrite_files: usize,
    unmarked_files: usize,
) -> RegenerabilityCoverage {
    RegenerabilityCoverage {
        scope: vec!["projects/demo/**".to_string()],
        total_files: 2,
        eligible_files: 2,
        codegen_files: 2 - handwrite_files - unmarked_files,
        fully_codegen_files: 2 - handwrite_files - unmarked_files,
        handwrite_files,
        unmarked_files,
        unsupported_codegen_files: Vec::new(),
        codegen_drift_files: Vec::new(),
        percent,
        gap_files: Vec::new(),
        semantic_percent: 100.0,
        generator_primitive_gaps: 0,
        primitive_covered_files: 2 - handwrite_files - unmarked_files,
        missing_generator_primitive_gaps: 0,
        insufficient_td_section_gaps: 0,
        human_decision_required_gaps: 0,
        next_gap: None,
    }
}

fn semantic(percent: f64, uncovered_files: Vec<String>) -> SemanticCoverage {
    SemanticCoverage {
        scope: vec!["projects/demo/**".to_string()],
        total_files: 2,
        source_units: 2,
        source_symbols: 0,
        claim_files: 0,
        semantic_files: 1,
        semantically_covered_files: 2 - uncovered_files.len(),
        percent,
        source_ir: Vec::new(),
        source_evidence_graph: None,
        frontend_ecosystem: None,
        coverage_map: Vec::new(),
        generator_primitive_gaps: Vec::new(),
        uncovered_files,
        next_gap: None,
        blocked_gap_count: 0,
        human_decision_required_count: 0,
    }
}

fn semantic_with_next_gap(primitive: &str) -> SemanticCoverage {
    let mut coverage = semantic(100.0, Vec::new());
    coverage.generator_primitive_gaps = vec![GeneratorPrimitiveGap {
        target: "projects/demo/build.sh".to_string(),
        primitive: primitive.to_string(),
        reason: "fixture gap".to_string(),
        human_decision_required: false,
    }];
    coverage.next_gap = Some(SemanticGap {
        target: "projects/demo/build.sh".to_string(),
        primitive: primitive.to_string(),
        reason: "fixture gap".to_string(),
        action: "extend_generator_primitive_or_keep_tracked_handwrite".to_string(),
    });
    coverage
}

fn cb_summary(clean: bool) -> CbVerifySummary {
    CbVerifySummary {
        clean,
        public_api_covered: 4,
        public_api_total: 4,
        semantic_review_required: 0,
        failures: if clean {
            Vec::new()
        } else {
            vec!["drift".to_string()]
        },
    }
}

fn cold_summary(clean: bool) -> Vec<CbColdVerifySummary> {
    vec![CbColdVerifySummary {
        workspace: Some("demo-backend".to_string()),
        clean,
        spec_count: 2,
        source_root_count: 1,
        generated_files: if clean { 2 } else { 1 },
        expected_files: 2,
        codegen_origin: CbCodegenOriginSummary {
            target_files: 2,
            td_ast_files: 2,
            artifact_replay_files: 0,
            source_template_files: 0,
        },
        failures: if clean {
            Vec::new()
        } else {
            vec!["src/lib.rs: differs after cold TD rebuild".to_string()]
        },
    }]
}

fn stack_migration(normalized: bool) -> StackMigrationCoverage {
    StackMigrationCoverage {
        project: "demo".to_string(),
        workspaces: vec![WorkspaceStackMigration {
            name: "demo-backend".to_string(),
            target: Some("python".to_string()),
            paths: vec!["projects/demo/**".to_string()],
            manifest_stacks: vec!["fastapi".to_string(), "sqlalchemy".to_string()],
            source_stacks: vec!["fastapi".to_string()],
            migration_state: "sqlalchemy_alloydb".to_string(),
            persistence_annotations: usize::from(normalized),
            dependency_policies: vec![DependencyPolicyFinding {
                dependency: "fastapi".to_string(),
                classification: "core_external".to_string(),
                action: "keep".to_string(),
                reason: "test fixture".to_string(),
            }],
            deployment_manifest_count: 0,
            deployment_facets: Vec::new(),
            unsupported_deployment_kinds: Vec::new(),
            normalized,
            notes: vec!["test fixture".to_string()],
        }],
        migration_normalized_percent: if normalized { 100.0 } else { 0.0 },
        incomplete_workspace_count: if normalized { 0 } else { 1 },
        dependency_policy_blockers: Vec::new(),
        deployment_policy_blockers: Vec::new(),
        blockers: if normalized {
            Vec::new()
        } else {
            vec!["demo-backend stack migration classification incomplete".to_string()]
        },
    }
}

fn stack_migration_with_dependency_blocker() -> StackMigrationCoverage {
    let mut coverage = stack_migration(true);
    coverage.dependency_policy_blockers = vec![
        "demo-frontend dependency `legacy-editor`: legacy editor should not be a target dependency"
            .to_string(),
    ];
    coverage.blockers = coverage.dependency_policy_blockers.clone();
    coverage
}

#[test]
fn clean_project_health_json_fields_are_healthy() {
    let report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
    );

    assert_eq!(report.status, ProjectHealthStatus::Healthy);
    assert!(report.production_ready);
    assert!(report.blockers.is_empty());
    assert!(report.optional_regenerability_gaps.is_empty());
    assert!(report.cb_verify_clean);
    assert_eq!(report.public_api_covered, 4);
    assert!(report.cold_rebuild_evaluated);
    assert!(report.cold_rebuild_clean);
    assert_eq!(report.cold_rebuild_workspace_count, 1);
}

#[test]
fn blocked_project_health_collects_governance_blockers() {
    let report = ProjectHealthReport::from_components(
        "demo",
        managed(50.0, vec!["projects/demo/src/lib.rs".to_string()]),
        semantic(50.0, vec!["projects/demo/src/lib.rs".to_string()]),
        regenerable(0.0, 1, 1),
        stack_migration(false),
        CbVerifySummary {
            clean: false,
            public_api_covered: 1,
            public_api_total: 4,
            semantic_review_required: 2,
            failures: vec!["byte drift".to_string()],
        },
        cold_summary(true),
    );

    assert_eq!(report.status, ProjectHealthStatus::Blocked);
    assert!(!report.production_ready);
    assert!(report.blockers.iter().any(|b| b.contains("unmanaged")));
    assert!(report
        .optional_regenerability_gaps
        .iter()
        .any(|b| b.contains("HANDWRITE")));
    assert!(report.blockers.iter().any(|b| b.contains("public API")));
    assert_eq!(report.semantic_review_required, 2);
    assert!(!report
        .blockers
        .iter()
        .any(|b| b.contains("semantic review")));
    assert!(report
        .blockers
        .iter()
        .any(|b| b.contains("stack migration")));
}

#[test]
fn regenerability_gaps_are_advisory_when_production_gates_clean() {
    let report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(50.0, 1, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
    );

    assert_eq!(report.status, ProjectHealthStatus::Healthy);
    assert!(report.production_ready);
    assert!(report.blockers.is_empty());
    assert_eq!(report.regenerable_percent, 50.0);
    assert!(report
        .optional_regenerability_gaps
        .iter()
        .any(|b| b.contains("HANDWRITE")));
}

#[test]
fn tracked_handwrite_generator_gap_does_not_block_semantic_readiness() {
    let report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic_with_next_gap("source_unit"),
        regenerable(90.0, 1, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
    );

    assert_eq!(report.status, ProjectHealthStatus::Healthy);
    assert!(report.semantic_ready);
    assert!(report.production_ready);
    assert!(report.blockers.is_empty());
    assert!(report
        .optional_regenerability_gaps
        .iter()
        .any(|gap| gap.contains("HANDWRITE")));
}

#[test]
fn semantic_review_required_is_reported_without_blocking_project_health() {
    let report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        CbVerifySummary {
            clean: true,
            public_api_covered: 4,
            public_api_total: 4,
            semantic_review_required: 2,
            failures: Vec::new(),
        },
        cold_summary(true),
    );

    assert_eq!(report.status, ProjectHealthStatus::Healthy);
    assert_eq!(report.semantic_review_required, 2);
    assert!(report.blockers.is_empty());
}

#[test]
fn dependency_policy_blockers_block_even_when_stack_migration_is_normalized() {
    let report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration_with_dependency_blocker(),
        cb_summary(true),
        cold_summary(true),
    );

    assert_eq!(report.status, ProjectHealthStatus::Blocked);
    assert!(report.blockers.iter().any(|b| b.contains("legacy-editor")));
}

#[test]
fn cold_rebuild_failures_block_project_health() {
    let report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(false),
    );

    assert_eq!(report.status, ProjectHealthStatus::Blocked);
    assert!(!report.cold_rebuild_clean);
    assert!(report
        .blockers
        .iter()
        .any(|b| b.contains("cold rebuild failed")));
    assert!(report.blockers.iter().any(|b| b.contains("demo-backend")));
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/project_health_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Source-template promotion for the project health aggregation tests.
      Replays the issue-2119 test implementation without the temporary HANDWRITE wrapper.
```
