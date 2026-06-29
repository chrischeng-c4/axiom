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
