---
id: vat-production-like-integration-scenarios
summary: Add first-class production-like integration scenarios for app plus dependencies plus SaaS mocks.
fill_sections: [logic, config, schema, cli, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This TD extends the local agent test runner protocol with app-under-test scenarios without introducing VM or Docker runner semantics."
---

# Vat Production-Like Integration Scenarios

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: production-like-integration-scenarios-applicability
entry: start
nodes:
  start: { kind: start, label: "evaluate WI #701" }
  schema: { kind: process, label: "extend vat.toml with optional scenarios" }
  cli: { kind: process, label: "route vat run --scenario <id>" }
  services: { kind: process, label: "reuse service lifecycle for app + deps" }
  evidence: { kind: process, label: "extend test-run topology evidence" }
  tests: { kind: process, label: "add focused scenario tests" }
  applicable: { kind: terminal, label: "applicable to vat runner protocol" }
edges:
  - { from: start, to: schema }
  - { from: schema, to: cli }
  - { from: cli, to: services }
  - { from: services, to: evidence }
  - { from: evidence, to: tests }
  - { from: tests, to: applicable }
---
flowchart TD
    start([evaluate WI #701]) --> schema[extend vat.toml with optional scenarios]
    schema --> cli[route vat run --scenario id]
    cli --> services[reuse service lifecycle for app + deps]
    services --> evidence[extend test-run topology evidence]
    evidence --> tests[add focused scenario tests]
    tests --> applicable([applicable to vat runner protocol])
```

## Config
<!-- type: config lang: yaml -->

```yaml
config_changes:
  - surface: "projects/vat/src/config.rs"
    change: "Add optional scenarios: Vec<ScenarioConfig> to VatConfig with serde default."
  - surface: "vat.toml"
    change: "Introduce compatible [[scenarios]] entries; existing files with only services/runners remain valid."
  - surface: "validation"
    change: "Validate unique scenario ids, known app service, known required services, known runner, and supported network mode."
scenario_shape:
  fields:
    id: "non-empty scenario id"
    app: "service id for the app under test"
    requires: "additional service ids for dependencies"
    runner: "runner id to execute after readiness"
    network: "open | hermetic; default open"
compatibility:
  existing_run_modes: "unchanged"
  scenario_optional: true
  service_schema_reuse: true
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
new_types:
  ScenarioConfig:
    fields:
      id: String
      app: String
      requires: Vec<String>
      runner: String
      network: ScenarioNetworkMode
  ScenarioNetworkMode:
    variants:
      - open
      - hermetic
  ScenarioRunRecord:
    fields:
      id: String
      app: String
      runner: String
      network: String
      services: Vec<String>
      routes: Vec<RouteRecord>
      hermetic: bool
  RouteRecord:
    fields:
      host: String
      target: String
      source: String
state_integration:
  TestRunEvidence:
    add_optional_field: "scenario: Option<ScenarioRunRecord>"
  ServiceRunRecord:
    reuse_existing_fields: ["id", "status", "preset", "port", "exported_env", "ready_duration_ms", "stdout_log", "stderr_log"]
compatibility:
  serde_defaults: "new fields default absent for existing metadata"
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: "vat run --scenario <id>"
    purpose: "Run a named production-like integration scenario."
    behavior:
      - "Load nearest vat.toml."
      - "Resolve scenario id."
      - "Resolve scenario runner."
      - "Start app plus dependency services."
      - "Wait for readiness before runner execution."
      - "Emit JSONL select/ready/runner/result events."
      - "Forward runner exit code."
compatibility:
  vat_run_default_runner: "unchanged"
  vat_run_runner_id: "unchanged"
  vat_run_multiple_runners: "unchanged"
  vat_run_direct_command: "unchanged"
errors:
  - code: "scenario_required"
    trigger: "unknown scenario id"
  - code: "scenario_hermetic_proxy_required"
    trigger: "scenario network hermetic but no http-mock service participates"
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: scenario-starts-app-deps-and-runner
    name: "Scenario starts app dependencies and runner"
    command: "cargo test -p vat scenario_run_starts_app_dependency_and_runner -- --nocapture"
    assertions:
      - "vat run --scenario prod-like exits with runner status"
      - "app service readiness is observed before the runner executes"
      - "state retains scenario/app/runner topology evidence"
  - id: scenario-failure-keeps-evidence
    name: "Scenario failure keeps topology and logs"
    command: "cargo test -p vat scenario_failure_keeps_topology_and_logs -- --nocapture"
    assertions:
      - "failed scenario run is retained under keep=failed"
      - "vat logs and vat state expose app and runner evidence"
  - id: scenario-hermetic-requires-http-mock
    name: "Scenario hermetic requires http mock service"
    command: "cargo test -p vat scenario_hermetic_requires_http_mock_service -- --nocapture"
    assertions:
      - "hermetic scenario without http-mock emits scenario_hermetic_proxy_required"
      - "no runner starts after the setup error"
regression:
  - "cargo test -p vat vat_toml_runner -- --nocapture"
  - "cargo test -p vat --test vat_concurrent_runners -- --nocapture"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - area: "config"
    files: ["projects/vat/src/config.rs"]
    summary: "Add ScenarioConfig and ScenarioNetworkMode plus validation helpers."
  - area: "cli"
    files: ["projects/vat/src/cli.rs"]
    summary: "Add --scenario to vat run and dispatch a scenario target."
  - area: "runner-orchestration"
    files: ["projects/vat/src/commands/run.rs"]
    summary: "Resolve scenario service union, require hermetic proxy when requested, and reuse service lifecycle."
  - area: "state"
    files: ["projects/vat/src/state.rs"]
    summary: "Persist scenario topology evidence under TestRunEvidence."
  - area: "tests"
    files: ["projects/vat/tests/vat_toml_runner.rs", "projects/vat/tests/vat_concurrent_runners.rs"]
    summary: "Add focused scenario execution and regression tests."
non_changes:
  - "No VM backend."
  - "No Dockerized runner."
  - "No service preset expansion."
```
