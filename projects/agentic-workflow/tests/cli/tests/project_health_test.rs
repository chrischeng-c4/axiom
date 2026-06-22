// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/generate/project-health-test-source.md#source
// CODEGEN-BEGIN
use std::collections::BTreeMap;

use agentic_workflow::cli::cb::{CbCodegenOriginSummary, CbColdVerifySummary, CbVerifySummary};
use agentic_workflow::cli::project::{
    project_health_compact_summary_with_payload_path, project_health_section_summary,
    project_health_summary, project_health_summary_with_payload_path, ProjectEcCommandReport,
    ProjectEcGateReport, ProjectEcGateStatus, ProjectHealthReport, ProjectHealthSection,
    ProjectHealthStatus, ProjectTestCommandReport, ProjectTestCommandStatus, ProjectTestGateReport,
    ProjectTestGateStatus,
};
use agentic_workflow::cli::regenerability_policy::RegenerabilityAuthority;
use agentic_workflow::cli::standardize::{
    DependencyPolicyFinding, GeneratorPrimitiveGap, MarkerCounts, RegenerabilityCoverage,
    SemanticCoverage, SemanticGap, StackMigrationCoverage, StandardizationCoverage,
    TraceabilityBlocker, TraceabilityBlockerKind, TraceabilityCoverage, WorkspaceStackMigration,
};
use agentic_workflow::cli::Commands;
use agentic_workflow::models::{
    default_preflight_gates, ArtifactKind, PreFlightEvidence, PreFlightEvidenceKind,
    PreFlightEvidenceStatus, PreFlightGate, PreFlightGateReport, PreFlightGateSeverity,
};
use clap::Parser;

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn parse_cli<const N: usize>(argv: [&str; N]) -> Commands {
    let parsed = Cli::try_parse_from(argv).expect("health command parses");
    parsed.command
}

fn parse_health_args<const N: usize>(
    argv: [&str; N],
) -> agentic_workflow::cli::project::ProjectHealthArgs {
    let command = parse_cli(argv);
    match command {
        Commands::Health(args) => args,
        _ => panic!("expected health command"),
    }
}

#[test]
fn project_option_is_health_only_project_selector() {
    let args = parse_health_args(["aw", "health", "--project", "demo", "regenerable"]);
    assert_eq!(args.project, "demo");
    assert_eq!(args.section, Some(ProjectHealthSection::Regenerable));

    let args = parse_health_args(["aw", "health", "regenerable", "--project", "demo"]);
    assert_eq!(args.project, "demo");
    assert_eq!(args.section, Some(ProjectHealthSection::Regenerable));
    assert!(!args.verbose);

    let args = parse_health_args(["aw", "health", "--project", "demo", "--verbose"]);
    assert_eq!(args.project, "demo");
    assert_eq!(args.section, None);
    assert!(args.verbose);
}

#[test]
fn project_option_propagates_to_nested_project_commands() {
    let command = parse_cli(["aw", "capability", "--project", "demo", "report"]);
    match command {
        Commands::Capability(args) => {
            assert_eq!(args.project.as_deref(), Some("demo"));
            assert!(matches!(
                args.command,
                agentic_workflow::cli::capability::CapabilityCommand::Report(_)
            ));
        }
        _ => panic!("expected capability command"),
    }

    let command = parse_cli(["aw", "capability", "report", "--project", "demo"]);
    match command {
        Commands::Capability(args) => {
            assert_eq!(args.project.as_deref(), Some("demo"));
            assert!(matches!(
                args.command,
                agentic_workflow::cli::capability::CapabilityCommand::Report(_)
            ));
        }
        _ => panic!("expected capability command"),
    }

    let command = parse_cli(["aw", "standardize", "managed", "run", "--project", "demo"]);
    match command {
        Commands::Standardize(args) => {
            assert_eq!(args.project.as_deref(), Some("demo"));
            assert!(matches!(
                args.command,
                Some(agentic_workflow::cli::standardize::StandardizeCommand::Managed(_))
            ));
        }
        _ => panic!("expected standardize command"),
    }

    let command = parse_cli(["aw", "standardize", "--project", "demo", "managed", "run"]);
    match command {
        Commands::Standardize(args) => {
            assert_eq!(args.project.as_deref(), Some("demo"));
            assert!(matches!(
                args.command,
                Some(agentic_workflow::cli::standardize::StandardizeCommand::Managed(_))
            ));
        }
        _ => panic!("expected standardize command"),
    }

    let command = parse_cli(["aw", "generator", "--project", "demo", "check"]);
    match command {
        Commands::Generator(args) => {
            assert_eq!(args.project.as_deref(), Some("demo"));
            assert!(matches!(
                args.command,
                agentic_workflow::cli::generator::GeneratorCommand::Check(_)
            ));
        }
        _ => panic!("expected generator command"),
    }

    let command = parse_cli(["aw", "generator", "check", "--project", "demo"]);
    match command {
        Commands::Generator(args) => {
            assert_eq!(args.project.as_deref(), Some("demo"));
            assert!(matches!(
                args.command,
                agentic_workflow::cli::generator::GeneratorCommand::Check(_)
            ));
        }
        _ => panic!("expected generator command"),
    }

    let command = parse_cli(["aw", "ec", "--project", "demo", "doc", "gen"]);
    match command {
        Commands::Ec(args) => {
            assert_eq!(args.project.as_deref(), Some("demo"));
            assert!(matches!(
                args.command,
                agentic_workflow::cli::ec::EcCommand::Doc(_)
            ));
        }
        _ => panic!("expected ec command"),
    }

    let command = parse_cli(["aw", "ec", "doc", "gen", "--project", "demo"]);
    match command {
        Commands::Ec(args) => {
            assert_eq!(args.project.as_deref(), Some("demo"));
            assert!(matches!(
                args.command,
                agentic_workflow::cli::ec::EcCommand::Doc(_)
            ));
        }
        _ => panic!("expected ec command"),
    }

    let command = parse_cli(["aw", "td", "--project", "demo", "lock"]);
    match command {
        Commands::Td(args) => {
            assert_eq!(args.project.as_deref(), Some("demo"));
            assert!(matches!(
                args.command,
                agentic_workflow::cli::td::TdCommand::Lock(_)
            ));
        }
        _ => panic!("expected td command"),
    }

    let command = parse_cli(["aw", "td", "lock", "--project", "demo"]);
    match command {
        Commands::Td(args) => {
            assert_eq!(args.project.as_deref(), Some("demo"));
            assert!(matches!(
                args.command,
                agentic_workflow::cli::td::TdCommand::Lock(_)
            ));
        }
        _ => panic!("expected td command"),
    }
}

#[test]
fn capability_issue_inventory_flags_parse_for_report_next_run_and_check() {
    let command = parse_cli([
        "aw",
        "capability",
        "--project",
        "demo",
        "report",
        "--skip-issue-inventory",
    ]);
    match command {
        Commands::Capability(args) => {
            assert_eq!(args.project.as_deref(), Some("demo"));
            match args.command {
                agentic_workflow::cli::capability::CapabilityCommand::Report(report) => {
                    assert!(report.skip_issue_inventory);
                    assert!(!report.include_issue_inventory);
                }
                _ => panic!("expected capability report"),
            }
        }
        _ => panic!("expected capability command"),
    }

    let command = parse_cli([
        "aw",
        "capability",
        "--project",
        "demo",
        "next",
        "--skip-issue-inventory",
    ]);
    match command {
        Commands::Capability(args) => match args.command {
            agentic_workflow::cli::capability::CapabilityCommand::Next(next) => {
                assert!(next.skip_issue_inventory);
                assert!(!next.include_issue_inventory);
            }
            _ => panic!("expected capability next"),
        },
        _ => panic!("expected capability command"),
    }

    let command = parse_cli([
        "aw",
        "capability",
        "--project",
        "demo",
        "run",
        "--non-interactive",
        "--skip-issue-inventory",
    ]);
    match command {
        Commands::Capability(args) => match args.command {
            agentic_workflow::cli::capability::CapabilityCommand::Run(run) => {
                assert!(run.skip_issue_inventory);
                assert!(!run.include_issue_inventory);
            }
            _ => panic!("expected capability run"),
        },
        _ => panic!("expected capability command"),
    }

    let command = parse_cli([
        "aw",
        "capability",
        "--project",
        "demo",
        "check",
        "--include-issue-inventory",
    ]);
    match command {
        Commands::Capability(args) => match args.command {
            agentic_workflow::cli::capability::CapabilityCommand::Check(check) => {
                assert!(check.include_issue_inventory);
                assert!(!check.skip_issue_inventory);
            }
            _ => panic!("expected capability check"),
        },
        _ => panic!("expected capability command"),
    }
}

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
    let gap_files = (0..handwrite_files)
        .map(|idx| format!("projects/demo/src/handwrite_{idx}.rs"))
        .chain((0..unmarked_files).map(|idx| format!("projects/demo/src/unmarked_{idx}.rs")))
        .collect();
    RegenerabilityCoverage {
        scope: vec!["projects/demo/**".to_string()],
        total_files: 2,
        eligible_files: 2,
        codegen_files: 2 - handwrite_files - unmarked_files,
        handwrite_files,
        unmarked_files,
        unsupported_codegen_files: Vec::new(),
        non_replayable_codegen_files: Vec::new(),
        snapshot_codegen_files: Vec::new(),
        codegen_drift_evaluated: false,
        codegen_drift_files: Vec::new(),
        percent,
        gap_files,
        semantic_percent: 100.0,
        generator_primitive_gaps: 0,
        primitive_covered_files: 2 - handwrite_files - unmarked_files,
        missing_generator_primitive_gaps: 0,
        insufficient_td_section_gaps: 0,
        human_decision_required_gaps: 0,
        next_gap: None,
        authority_mode: RegenerabilityAuthority::ExternalAdvisory,
        required_for_production: false,
        authority_reason: "test fixture".to_string(),
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

fn traceability(blockers: Vec<TraceabilityBlocker>) -> TraceabilityCoverage {
    TraceabilityCoverage {
        project: "demo".to_string(),
        scope: vec!["projects/demo/**".to_string()],
        cap_path: "projects/demo/README.md".to_string(),
        total_td_files: 1,
        traceable_td_files: if blockers.is_empty() { 1 } else { 0 },
        traceability_percent: if blockers.is_empty() { 100.0 } else { 0.0 },
        internal_td_count: 0,
        orphan_td_count: blockers
            .iter()
            .filter(|blocker| blocker.kind == TraceabilityBlockerKind::TdNoCapabilityRef)
            .count(),
        source_edge_count: 1,
        cb_edge_count: 1,
        command_traceability:
            agentic_workflow::cli::standardize::CommandTraceabilityCoverage::ready_fixture(),
        blocker_count: blockers.len(),
        next_gap: blockers.first().cloned(),
        blockers,
    }
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
        ProjectTestGateReport::passed_fixture("true"),
    );

    assert_eq!(report.status, ProjectHealthStatus::Healthy);
    assert!(report.capability_ready);
    assert!(report.managed_ready);
    assert!(report.semantic_ready);
    assert!(report.traceability_ready);
    assert!(report.takeover_ready);
    assert!(report.generator_request_ready);
    assert!(report.production_ready);
    assert!(report.blockers.is_empty());
    assert!(report.capability.root_runner_ready);
    assert_eq!(report.capability.capability_count, 1);
    assert_eq!(report.capability.production_percent, 100.0);
    assert!(report.optional_regenerability_gaps.is_empty());
    assert!(report.cb_verify_clean);
    assert_eq!(report.public_api_covered, 4);
    assert!(report.cold_rebuild_evaluated);
    assert!(report.cold_rebuild_clean);
    assert_eq!(report.cold_rebuild_workspace_count, 1);
}

#[test]
fn project_health_summary_marks_complete_health_done() {
    let report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );

    let summary = project_health_summary(&report);

    assert_eq!(summary["schema_version"].as_str(), Some("aw.cli.v1"));
    assert_eq!(summary["event"].as_str(), Some("result"));
    assert_eq!(summary["status"].as_str(), Some("done"));
    assert_eq!(
        summary["completion"]["workflow_complete"].as_bool(),
        Some(true)
    );
    assert_eq!(summary["next"]["kind"].as_str(), Some("done"));
    assert_eq!(
        summary["readiness"]["production_ready"].as_bool(),
        Some(true)
    );
    assert_eq!(
        summary["axes"]["td_gen"]["generated_percent"].as_f64(),
        Some(100.0)
    );
    assert_eq!(
        summary["axes"]["td_gen"]["generated_units"].as_u64(),
        Some(2)
    );
    assert_eq!(
        summary["axes"]["td_gen"]["expected_units"].as_u64(),
        Some(2)
    );
    assert_eq!(
        summary["axes"]["td_gen"]["handwrite_units"].as_u64(),
        Some(0)
    );
    assert!(summary["axes"]["td_gen"].get("codegen_files").is_none());
    assert_eq!(summary["project"].as_str(), Some("demo"));
}

#[test]
fn project_health_summary_with_payload_path_is_bounded_result() {
    let mut report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        CbVerifySummary {
            clean: false,
            public_api_covered: 1,
            public_api_total: 4,
            semantic_review_required: 0,
            failures: (0..30).map(|idx| format!("drift {idx}")).collect(),
        },
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );
    report.blockers = (0..30).map(|idx| format!("blocker {idx}")).collect();
    report.test_gates = ProjectTestGateReport {
        evaluated: true,
        status: ProjectTestGateStatus::Failed,
        note: None,
        command_count: 1,
        passed_count: 0,
        failed_count: 1,
        skipped_count: 0,
        commands: vec![ProjectTestCommandReport {
            workspace: "demo".to_string(),
            command: "cargo test -p demo".to_string(),
            status: ProjectTestCommandStatus::Failed,
            exit_code: Some(101),
            duration_ms: 42,
            stdout_tail: "very long test stdout".to_string(),
            stderr_tail: "very long test stderr".to_string(),
        }],
    };
    report.ec = ProjectEcGateReport {
        evaluated: true,
        check_clean: true,
        verify_evaluated: true,
        status: ProjectEcGateStatus::Failed,
        note: None,
        inventory_path: "projects/demo/aw.toml".to_string(),
        expected_case_count: 1,
        case_count: 1,
        expected_tool_manifest_count: 0,
        tool_manifest_count: 0,
        command_count: 1,
        passed_count: 0,
        failed_count: 1,
        findings: vec!["ec failed".to_string()],
        commands: vec![ProjectEcCommandReport {
            case_id: "demo-case".to_string(),
            command: "cargo test -p demo demo_case".to_string(),
            status: ProjectTestCommandStatus::Failed,
            exit_code: Some(101),
            duration_ms: 43,
            stdout_tail: "very long ec stdout".to_string(),
            stderr_tail: "very long ec stderr".to_string(),
        }],
    };

    let summary =
        project_health_summary_with_payload_path(&report, "/tmp/aw/demo/health/report.json");

    assert_eq!(summary["event"].as_str(), Some("result"));
    assert_eq!(
        summary["payload_path"].as_str(),
        Some("/tmp/aw/demo/health/report.json")
    );
    assert!(summary.get("report").is_none());
    assert_eq!(summary["blockers"]["blocker_count"].as_u64(), Some(30));
    assert_eq!(
        summary["blockers"]["blockers_preview"]
            .as_array()
            .map(|items| items.len()),
        Some(5)
    );
    assert!(summary.get("test_gates").is_none());
    assert!(summary.get("ec").is_none());
    assert_eq!(summary["axes"]["ec"]["status"].as_str(), Some("failed"));
    assert_eq!(
        summary["axes"]["ec_gen"]["generated_units"].as_u64(),
        Some(1)
    );
    assert_eq!(
        summary["axes"]["ec_gen"]["expected_units"].as_u64(),
        Some(1)
    );
    assert_eq!(
        summary["axes"]["ec_gen"]["generated_percent"].as_f64(),
        Some(100.0)
    );
    assert!(summary["axes"]["ec_gen"].get("case_count").is_none());
    assert!(summary["axes"]["ec_gen"]
        .get("tool_manifest_count")
        .is_none());
    let rendered = serde_json::to_string(&summary).expect("summary renders");
    assert!(!rendered.contains("stdout_tail"));
    assert!(!rendered.contains("stderr_tail"));
    assert!(!rendered.contains("very long"));
}

#[test]
fn project_health_compact_summary_is_low_token_default() {
    let mut report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );
    report.blockers = (0..30).map(|idx| format!("blocker {idx}")).collect();

    let summary = project_health_compact_summary_with_payload_path(
        &report,
        "/tmp/aw/demo/health/report.json",
    );

    assert_eq!(summary["action"].as_str(), Some("health"));
    assert_eq!(
        summary["payload_path"].as_str(),
        Some("/tmp/aw/demo/health/report.json")
    );
    assert_eq!(
        summary["axes"]["td_gen"]["generated_percent"].as_f64(),
        Some(100.0)
    );
    assert!(summary.get("metrics").is_none());
    assert!(summary.get("gates").is_none());
    assert!(summary.get("report").is_none());
    assert_eq!(
        summary["blockers"]["blockers_preview"]
            .as_array()
            .map(|items| items.len()),
        Some(5)
    );
    let rendered = serde_json::to_string(&summary).expect("compact summary renders");
    assert!(!rendered.contains("blocker 29"));
}

#[test]
fn project_health_ec_gen_axis_is_not_configured_for_zero_expected_units() {
    let mut report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );
    report.ec = ProjectEcGateReport {
        evaluated: true,
        check_clean: true,
        verify_evaluated: false,
        status: ProjectEcGateStatus::NotConfigured,
        note: None,
        inventory_path: "projects/demo/aw.toml".to_string(),
        expected_case_count: 0,
        case_count: 0,
        expected_tool_manifest_count: 0,
        tool_manifest_count: 0,
        command_count: 0,
        passed_count: 0,
        failed_count: 0,
        findings: Vec::new(),
        commands: Vec::new(),
    };

    let summary = project_health_summary(&report);

    assert_eq!(
        summary["axes"]["ec_gen"]["status"].as_str(),
        Some("not_configured")
    );
    assert_eq!(
        summary["axes"]["ec_gen"]["expected_units"].as_u64(),
        Some(0)
    );
    assert_eq!(
        summary["axes"]["ec_gen"]["generated_units"].as_u64(),
        Some(0)
    );
}

#[test]
fn project_health_section_full_preserves_detailed_report() {
    let report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );

    let summary = project_health_section_summary(&report, ProjectHealthSection::Full);

    assert_eq!(summary["project"].as_str(), Some("demo"));
    assert_eq!(
        summary["axes"]["td_gen"]["generated_percent"].as_f64(),
        Some(100.0)
    );
}

#[test]
fn project_health_section_regenerable_is_focused() {
    let report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );

    let summary = project_health_section_summary(&report, ProjectHealthSection::Regenerable);

    assert_eq!(summary["section"].as_str(), Some("regenerable"));
    assert!(summary.get("report").is_none());
    assert_eq!(summary["data"]["codegen_percent"].as_f64(), Some(100.0));
    assert!(summary["data"].get("codegen_origin").is_some());
}

#[test]
fn project_health_summary_points_to_full_verify_when_gates_are_missing() {
    let mut report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::not_evaluated("demo"),
    );
    report.traceability_evaluated = false;
    report.cb_verify_evaluated = false;
    report.cold_rebuild_evaluated = false;
    report.production_ready = false;
    report.status = ProjectHealthStatus::Blocked;

    let summary = project_health_summary(&report);

    assert_eq!(summary["status"].as_str(), Some("continue"));
    assert_eq!(
        summary["completion"]["workflow_complete"].as_bool(),
        Some(false)
    );
    assert_eq!(summary["next"]["kind"].as_str(), Some("run_command"));
    assert_eq!(
        summary["next"]["command"].as_str(),
        Some("aw health --project demo full")
    );
}

#[test]
fn project_health_summary_routes_managed_blockers_to_standardize() {
    let report = ProjectHealthReport::from_components(
        "demo",
        managed(50.0, vec!["projects/demo/src/lib.rs".to_string()]),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );

    let summary = project_health_summary(&report);

    assert_eq!(summary["status"].as_str(), Some("continue"));
    assert_eq!(summary["next"]["kind"].as_str(), Some("run_command"));
    assert_eq!(
        summary["next"]["command"].as_str(),
        Some("aw standardize managed run --project demo --non-interactive --max-ticks 1")
    );
}

#[test]
fn project_health_next_reason_matches_managed_route_when_ec_has_no_expected_units() {
    let mut report = ProjectHealthReport::from_components(
        "demo",
        managed(50.0, vec!["projects/demo/src/lib.rs".to_string()]),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );
    report.ec = ProjectEcGateReport {
        evaluated: true,
        check_clean: true,
        verify_evaluated: false,
        status: ProjectEcGateStatus::NotConfigured,
        note: Some("EC inventory has no cases".to_string()),
        inventory_path: "projects/demo/aw.toml".to_string(),
        expected_case_count: 0,
        case_count: 0,
        expected_tool_manifest_count: 0,
        tool_manifest_count: 0,
        command_count: 0,
        passed_count: 0,
        failed_count: 0,
        findings: Vec::new(),
        commands: Vec::new(),
    };

    let summary = project_health_summary(&report);

    assert_eq!(
        summary["next"]["command"].as_str(),
        Some("aw standardize managed run --project demo --non-interactive --max-ticks 1")
    );
    assert_eq!(
        summary["next"]["reason"].as_str(),
        Some("source ownership is incomplete; advance managed takeover")
    );
}

#[test]
fn no_cold_rebuild_workspace_keeps_specific_repair_route() {
    let mut report = ProjectHealthReport::from_components(
        "demo",
        managed(50.0, vec!["projects/demo/src/lib.rs".to_string()]),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        Vec::new(),
        ProjectTestGateReport::passed_fixture("true"),
    );
    report.cold_rebuild_evaluated = false;
    report.cold_rebuild_workspace_count = 0;
    report.cold_rebuild_clean = false;
    let note =
        "not evaluated; project `demo` has no workspace with `verify_cold = true`".to_string();
    report.cold_rebuild_note = Some(note.clone());
    report.blockers.push(note);

    let summary = project_health_summary(&report);

    assert_eq!(
        summary["next"]["command"].as_str(),
        Some("aw standardize managed run --project demo --non-interactive --max-ticks 1")
    );
    let missing = summary["completion"]["missing"].as_array().unwrap();
    assert!(missing
        .iter()
        .any(|value| value.as_str().unwrap_or("").contains("verify_cold")));
}

#[test]
fn unparseable_capability_map_blocks_without_invalid_next_command() {
    let mut report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );
    report.capability_ready = false;
    report.production_ready = false;
    report.status = ProjectHealthStatus::Blocked;
    report.capability.format = "unparseable".to_string();
    report.capability.root_runner_ready = false;
    report.capability.capability_count = 0;
    report.capability.blocker_count = 1;
    report.capability.blockers =
        vec!["capability document parse failed: no capability sections found".to_string()];
    report.blockers = report.capability.blockers.clone();

    let summary = project_health_summary(&report);

    assert_eq!(summary["next"]["kind"].as_str(), Some("blocked"));
    assert!(summary["next"].get("command").is_none());
    assert_eq!(
        summary["next"]["reason"].as_str(),
        Some("capability document parse failed: no capability sections found")
    );
}

#[test]
fn traceability_blockers_make_project_health_not_ready() {
    let blocker = TraceabilityBlocker {
        kind: TraceabilityBlockerKind::TdNoCapabilityRef,
        target: "projects/demo/tech-design/app.md".to_string(),
        reason: "TD has no capability_refs".to_string(),
        source: None,
    };
    let report = ProjectHealthReport::from_components_with_traceability(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        traceability(vec![blocker]),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );

    assert_eq!(report.status, ProjectHealthStatus::Blocked);
    assert!(report.capability_ready);
    assert!(report.managed_ready);
    assert!(report.semantic_ready);
    assert!(!report.traceability_ready);
    assert!(!report.takeover_ready);
    assert!(!report.generator_request_ready);
    assert!(!report.production_ready);
    assert_eq!(report.traceability_blocker_count, 1);
    assert_eq!(report.traceability_orphan_td_count, 1);
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker.contains("traceability closure incomplete")));
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
        ProjectTestGateReport::passed_fixture("true"),
    );

    assert_eq!(report.status, ProjectHealthStatus::Blocked);
    assert!(report.capability_ready);
    assert!(!report.managed_ready);
    assert!(!report.semantic_ready);
    assert!(report.traceability_ready);
    assert!(!report.takeover_ready);
    assert!(!report.generator_request_ready);
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
        ProjectTestGateReport::passed_fixture("true"),
    );

    assert_eq!(report.status, ProjectHealthStatus::Healthy);
    assert!(report.production_ready);
    assert!(report.blockers.is_empty());
    assert_eq!(report.codegen_percent, 50.0);
    assert!(report
        .optional_regenerability_gaps
        .iter()
        .any(|b| b.contains("HANDWRITE")));
    assert_eq!(
        report.regenerability_authority.authority,
        RegenerabilityAuthority::ExternalAdvisory
    );
    assert!(!report.regenerability_authority.required_for_production);
    assert_eq!(report.regenerability_authority.gap_count, 1);
}

#[test]
fn codegen_metric_counts_only_ast_codegen_units() {
    let mut regenerable = regenerable(50.0, 1, 0);
    regenerable.non_replayable_codegen_files = vec!["projects/demo/src/build.rs".to_string()];

    let report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable,
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );

    assert_eq!(report.codegen_percent, 50.0);
    assert_eq!(report.codegen_files, 1);
    assert_eq!(report.cb_ownership.codegen_files, 1);
    assert_eq!(report.cb_ownership.handwrite_files, 1);
    assert!(report
        .optional_regenerability_gaps
        .iter()
        .any(|gap| gap.contains("mark HANDWRITE or implement AST codegen")));
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
        ProjectTestGateReport::passed_fixture("true"),
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
fn authoritative_regenerability_gaps_block_project_health() {
    let report = ProjectHealthReport::from_components(
        "agentic-workflow",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(50.0, 1, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );

    assert_eq!(report.status, ProjectHealthStatus::Blocked);
    assert!(!report.production_ready);
    assert_eq!(
        report.regenerability_authority.authority,
        RegenerabilityAuthority::GeneratorAuthoritative
    );
    assert!(report.regenerability_authority.required_for_production);
    assert_eq!(report.regenerability_authority.gap_count, 1);
    assert!(report.optional_regenerability_gaps.is_empty());
    assert!(report
        .blockers
        .iter()
        .any(|blocker| blocker.contains("regenerability required")));
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
        ProjectTestGateReport::passed_fixture("true"),
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
        ProjectTestGateReport::passed_fixture("true"),
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
        ProjectTestGateReport::passed_fixture("true"),
    );

    assert_eq!(report.status, ProjectHealthStatus::Blocked);
    assert!(!report.cold_rebuild_clean);
    assert!(report
        .blockers
        .iter()
        .any(|b| b.contains("cold rebuild failed")));
    assert!(report.blockers.iter().any(|b| b.contains("demo-backend")));
}

#[test]
fn tests_not_evaluated_blocks_production_ready() {
    let report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::not_evaluated("demo"),
    );

    assert_eq!(report.status, ProjectHealthStatus::Blocked);
    assert!(!report.production_ready);
    assert_eq!(
        report.test_gates.status,
        ProjectTestGateStatus::NotEvaluated
    );
    assert!(report.blockers.iter().any(|b| b.contains("not evaluated")));
}

#[test]
fn missing_hard_preflight_gate_blocks_project_health() {
    let mut report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );
    let preflight = PreFlightGateReport::evaluate(
        "projects/demo/src/lib.rs",
        &[preflight_gate(PreFlightGateSeverity::Hard)],
        &[],
    );

    report.apply_preflight_gate_report(preflight);

    assert_eq!(report.status, ProjectHealthStatus::Blocked);
    assert!(!report.production_ready);
    assert!(report
        .production_blockers
        .iter()
        .any(|blocker| blocker.contains("missing test evidence")));
    assert!(report.optional_quality_warnings.is_empty());
}

#[test]
fn missing_frontend_preflight_evidence_blocks_project_health() {
    let mut report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );
    let preflight = PreFlightGateReport::evaluate(
        "projects/demo/frontend/src/App.tsx",
        &default_preflight_gates(ArtifactKind::FrontendPage),
        &[],
    );

    report.apply_preflight_gate_report(preflight);

    assert_eq!(report.status, ProjectHealthStatus::Blocked);
    assert!(!report.production_ready);
    assert!(report
        .production_blockers
        .iter()
        .any(|blocker| blocker.contains("frontend-page-viewport-screenshots")));
    assert!(report
        .production_blockers
        .iter()
        .any(|blocker| blocker.contains("frontend-page-interaction-smoke")));
}

#[test]
fn advisory_preflight_gate_warns_without_blocking_project_health() {
    let mut report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );
    let preflight = PreFlightGateReport::evaluate(
        "projects/demo/README.md",
        &[preflight_gate(PreFlightGateSeverity::Advisory)],
        &[],
    );

    report.apply_preflight_gate_report(preflight);

    assert_eq!(report.status, ProjectHealthStatus::Healthy);
    assert!(report.production_ready);
    assert!(report.production_blockers.is_empty());
    assert!(report
        .optional_quality_warnings
        .iter()
        .any(|warning| warning.contains("missing advisory test evidence")));
}

#[test]
fn accepted_preflight_evidence_keeps_project_health_ready() {
    let mut report = ProjectHealthReport::from_components(
        "demo",
        managed(100.0, Vec::new()),
        semantic(100.0, Vec::new()),
        regenerable(100.0, 0, 0),
        stack_migration(true),
        cb_summary(true),
        cold_summary(true),
        ProjectTestGateReport::passed_fixture("true"),
    );
    let evidence = PreFlightEvidence {
        gate_id: "code-artifact-test".to_string(),
        evidence_kind: PreFlightEvidenceKind::Test,
        source_ref: "cargo test -p demo".to_string(),
        status: PreFlightEvidenceStatus::Accepted,
    };
    let preflight = PreFlightGateReport::evaluate(
        "projects/demo/src/lib.rs",
        &[preflight_gate(PreFlightGateSeverity::Hard)],
        &[evidence],
    );

    report.apply_preflight_gate_report(preflight);

    assert_eq!(report.status, ProjectHealthStatus::Healthy);
    assert!(report.production_ready);
    assert!(report.production_blockers.is_empty());
    assert!(report.optional_quality_warnings.is_empty());
}

fn preflight_gate(severity: PreFlightGateSeverity) -> PreFlightGate {
    PreFlightGate {
        id: "code-artifact-test".to_string(),
        artifact_kind: ArtifactKind::CodeArtifact,
        severity,
        evidence_kind: PreFlightEvidenceKind::Test,
        description: "targeted test evidence".to_string(),
    }
}
// CODEGEN-END
