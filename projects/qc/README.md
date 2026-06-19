# Qc

## Brief

Qc is the planned agent-facing quality-control CLI surface for structured
reports, security findings, and performance boundary-cost findings.

The current checkout contains EC placeholder tests and one last-report artifact,
but no `qc` or `qc-cli` Cargo package is present in the workspace. The
capability map therefore records intended command contracts as blocked and does
not claim an implemented CLI.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Agent Report And Spec CLI | - | planned | failing | smoke | not_ready | EC placeholders exist, but `qc` and `qc-cli` packages are absent from the workspace |
| Security Finding Checks | - | planned | failing | smoke | not_ready | security EC placeholders exist, but `qc` package is absent from the workspace |
| Performance Profile Checks | - | planned | failing | smoke | not_ready | last report artifact exists, but `qc-cli` package is absent from the workspace |

### Agent Report And Spec CLI

ID: agent-report-and-spec-cli
Type: DeveloperTool
Surfaces: CLI: `qc report` + `qc spec --json-schema --compact` - report envelope and offline self-description surfaces
EC Dimensions: behavior: `cargo test -p qc report::`; behavior: `cargo run -p qc-cli --bin qc -- spec --json-schema --compact` - currently blocked because `qc` and `qc-cli` packages are missing from the workspace
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Qc should provide an agent-facing CLI that emits machine-readable report envelopes, preserves delegated runner exit-code semantics, and describes its schema/catalog offline without network access.
Gate Inventory: projects/qc/tests/behavior_qc_json_default_report_envelope_and_findings.rs; projects/qc/tests/behavior_qc_delegated_runner_exit_code_contract.rs; projects/qc/tests/behavior_qc_offline_schema_and_catalog_self_description.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Agent report/spec CLI contract | epic | - | planned | failing | smoke | projects/qc/tests/behavior_qc_json_default_report_envelope_and_findings.rs; projects/qc/tests/behavior_qc_delegated_runner_exit_code_contract.rs; projects/qc/tests/behavior_qc_offline_schema_and_catalog_self_description.rs |

### Security Finding Checks

ID: security-finding-checks
Type: SecurityTool
Surfaces: CLI: `qc security` + `qc audit` - security finding and advisory detection surfaces
EC Dimensions: security: `cargo test -p qc security::`; security: `cargo test -p qc --test audit_trust_bug` - currently blocked because `qc` package is missing from the workspace
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Qc should turn security checks into structured findings for advisory detection, seeded fuzz cases, and injection evidence that agents can inspect and route.
Gate Inventory: projects/qc/tests/security_qc_cargo_audit_advisory_detection.rs; projects/qc/tests/security_qc_seeded_fuzz_and_injection_finding_generation.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Security finding checks contract | epic | - | planned | failing | smoke | projects/qc/tests/security_qc_cargo_audit_advisory_detection.rs; projects/qc/tests/security_qc_seeded_fuzz_and_injection_finding_generation.rs |

### Performance Profile Checks

ID: performance-profile-checks
Type: DeveloperTool
Surfaces: CLI: `qc profile --phases <breakdown.json>` - boundary-cost profiling report surface
EC Dimensions: efficiency: `cargo run -p qc-cli --bin qc -- profile --phases projects/qc/tests/fixtures/profile_phase_breakdown.json` - currently blocked because `qc-cli` package is missing from the workspace
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Qc should fold phase timing breakdowns into structured boundary-cost findings so agents can identify expensive execution phases and remediation prompts.
Gate Inventory: projects/qc/tests/performance_qc_profile_phase_boundary_cost_report.rs; projects/qc/qc-cli/.qc/last-report.json

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Performance profile report contract | epic | - | planned | failing | smoke | projects/qc/tests/performance_qc_profile_phase_boundary_cost_report.rs; projects/qc/qc-cli/.qc/last-report.json |
