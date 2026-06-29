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
