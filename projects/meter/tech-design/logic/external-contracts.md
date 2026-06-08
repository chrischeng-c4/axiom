---
id: meter-external-contracts
summary: External contract gates for meter README capabilities.
fill_sections: [e2e-test]
capability_refs:
  - id: runtime-resource-attribution
    role: primary
    gap: profile-phase-boundary-cost-report
    claim: profile-phase-boundary-cost-report
    coverage: full
    rationale: "The EC gate verifies meter's deterministic profile boundary-cost report."
  - id: runtime-resource-attribution
    role: primary
    gap: embedded-profiler-api
    claim: embedded-profiler-api
    coverage: full
    rationale: "The EC gate verifies meter's embedded profiler API."
  - id: runtime-resource-attribution
    role: primary
    gap: benchmark-regression-api
    claim: benchmark-regression-api
    coverage: full
    rationale: "The EC gate verifies meter's benchmark and regression API coverage."
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "The EC gate verifies meter's JSON report envelope and finding model."
  - id: agent-use-first-cli
    role: primary
    gap: offline-schema-and-catalog-self-description
    claim: offline-schema-and-catalog-self-description
    coverage: full
    rationale: "The EC gate verifies meter's offline schema self-description."
  - id: agent-use-first-cli
    role: primary
    gap: delegated-runner-exit-code-contract
    claim: delegated-runner-exit-code-contract
    coverage: full
    rationale: "The EC gate verifies meter's delegated runner exit-code contract."
  - id: agent-use-first-cli
    role: primary
    gap: llm-usage-guide
    claim: llm-usage-guide
    coverage: full
    rationale: "The EC gate verifies meter's offline LLM usage guide."
---

# External Contracts: meter

## Profile Phase Boundary Cost Report EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: meter-profile-phase-boundary-cost-report
    capability_id: runtime-resource-attribution
    contract_id: profile-phase-boundary-cost-report
    category: performance
    command: "cargo run -p meter-cli --bin meter -- profile --phases projects/meter/tests/fixtures/profile_phase_breakdown.json"
    assertions:
      - "meter profile emits ranked boundary-cost findings from a serialized PhaseBreakdown"
      - "the deterministic profile path remains agent-readable JSON without requiring sampler privileges"
```

## Embedded Profiler API EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: meter-embedded-profiler-api
    capability_id: runtime-resource-attribution
    contract_id: embedded-profiler-api
    category: performance
    command: "cargo test -p meter performance::profiler"
    assertions:
      - "embedded profiler tests pass"
      - "RSS snapshot, phase breakdown, and profile result contracts remain available"
```

## Benchmark Regression API EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: meter-benchmark-regression-api
    capability_id: runtime-resource-attribution
    contract_id: benchmark-regression-api
    category: performance
    command: "cargo test -p meter benchmark::"
    assertions:
      - "benchmark tests pass"
      - "adaptive benchmark and percentile contracts remain stable"
```

## JSON Default Report Envelope And Findings EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: meter-json-default-report-envelope-and-findings
    capability_id: agent-use-first-cli
    contract_id: json-default-report-envelope-and-findings
    category: behavior
    command: "cargo test -p meter report::"
    assertions:
      - "report envelope and finding model tests pass"
      - "default agent-facing report shape remains deterministic"
```

## Offline Schema And Catalog Self Description EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: meter-offline-schema-and-catalog-self-description
    capability_id: agent-use-first-cli
    contract_id: offline-schema-and-catalog-self-description
    category: behavior
    command: "cargo run -p meter-cli --bin meter -- spec --catalog --compact"
    assertions:
      - "meter spec emits the finding catalog without target setup"
      - "offline self-description remains available for agents"
```

## Delegated Runner Exit Code Contract EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: meter-delegated-runner-exit-code-contract
    capability_id: agent-use-first-cli
    contract_id: delegated-runner-exit-code-contract
    category: behavior
    command: "cargo test -p meter report::builder::tests::forward_exit_overrides_natural_code"
    assertions:
      - "delegated exit-code forwarding remains encoded in report builder behavior"
      - "meter-native status does not hide delegated runner exit semantics"
```

## LLM Usage Guide EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: meter-llm-usage-guide
    capability_id: agent-use-first-cli
    contract_id: llm-usage-guide
    category: behavior
    command: "cargo run -p meter-cli --bin meter -- llm guide"
    assertions:
      - "meter llm guide emits the agent-facing usage contract without target setup"
      - "the guide identifies meter as a resource measurement tool rather than a security scanner or test framework"
```
