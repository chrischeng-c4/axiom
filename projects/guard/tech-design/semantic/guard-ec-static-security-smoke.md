---
id: guard-ec-static-security-smoke
summary: External-contract smoke evidence for guard's static security report.
capability_refs:
  - id: static-security-scan
    role: primary
    gap: compass-backed-diagnostic-scan
    claim: compass-backed-diagnostic-scan
    coverage: full
    rationale: "The EC case proves guard consumes compass diagnostics as its static security engine."
  - id: static-security-scan
    role: primary
    gap: json-report-envelope
    claim: json-report-envelope
    coverage: full
    rationale: "The EC case proves guard emits a clean guard.report/1 envelope for the project source tree."
  - id: security-policy-profile
    role: primary
    gap: baseline-static-policy
    claim: baseline-static-policy
    coverage: full
    rationale: "The EC case proves the baseline static policy maps compass output into guard status and summary fields."
  - id: security-policy-profile
    role: primary
    gap: security-lint-policy
    claim: security-lint-policy
    coverage: full
    rationale: "The EC case proves the security-lint profile folds security-impacting lint into guard policy findings."
  - id: security-policy-profile
    role: primary
    gap: cli-module-registration
    claim: cli-module-registration
    coverage: full
    rationale: "The EC case proves the guard CLI module stays registered in the cclab command surface."
  - id: security-ec-profile
    role: primary
    gap: aw-health-security-metric
    claim: aw-health-security-metric
    coverage: full
    rationale: "The EC case is the AW health security metric that consumes guard.report/1 as first-class security evidence."
  - id: security-ec-profile
    role: primary
    gap: ec-security-evidence-command
    claim: ec-security-evidence-command
    coverage: full
    rationale: "The EC case defines the executable guard security evidence command used by AW health."
  - id: dynamic-security-evidence
    role: primary
    gap: vat-isolated-security-runner
    claim: vat-isolated-security-runner
    coverage: full
    rationale: "The EC case proves guard can fold a vat-isolated security runner into its report."
  - id: dynamic-security-evidence
    role: primary
    gap: rig-exploit-journey-bridge
    claim: rig-exploit-journey-bridge
    coverage: full
    rationale: "The EC case proves guard can fold rig scenario evidence into its report."
  - id: dynamic-security-evidence
    role: primary
    gap: meter-dos-resource-evidence-bridge
    claim: meter-dos-resource-evidence-bridge
    coverage: full
    rationale: "The EC case proves guard can fold meter resource evidence into its report."
fill_sections: [e2e-test]
---

# Guard Static Security EC

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: guard-static-scan-clean-report
    capability_id: static-security-scan
    claim_id: json-report-envelope
    contract_id: guard-report-clean-static-scan
    category: security
    command: "target/debug/guard scan projects/guard --compact --no-persist"
    assertions:
      - command exits zero
      - stdout is a guard.report/1 JSON envelope
      - summary.security_findings is zero for the guard source tree
      - integrations.static_engine is compass
```

## Compass Backed Diagnostic Scan EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: guard-compass-backed-diagnostic-scan
    capability_id: static-security-scan
    claim_id: compass-backed-diagnostic-scan
    contract_id: compass-backed-diagnostic-scan
    category: security
    command: "CC=/usr/bin/cc PATH=\"$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin\" cargo test -p guard scan::tests::detects_javascript_eval_as_security_finding"
    assertions:
      - "compass-backed scan tests detect security diagnostics"
      - "guard preserves the static security engine integration"
```

## Security Policy Profile EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: guard-baseline-static-policy
    capability_id: security-policy-profile
    claim_id: baseline-static-policy
    contract_id: baseline-static-policy
    category: security
    command: "CC=/usr/bin/cc PATH=\"$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin\" cargo test -p guard detects_javascript_eval_as_security_finding"
    assertions:
      - "baseline static policy maps compass diagnostics into guard findings"
      - "policy severity normalization remains covered by guard tests"

  - id: guard-security-lint-policy
    capability_id: security-policy-profile
    claim_id: security-lint-policy
    contract_id: security-lint-policy
    category: security
    command: "target/debug/guard scan projects/guard --profile security-lint --compact --no-persist"
    assertions:
      - "security-lint profile runs on the guard source tree"
      - "security-impacting lint remains part of the guard policy profile"

  - id: guard-cli-module-registration
    capability_id: security-policy-profile
    claim_id: cli-module-registration
    contract_id: cli-module-registration
    category: behavior
    command: "CC=/usr/bin/cc PATH=\"$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin\" cargo test -p guard-cli registered_in_slice"
    assertions:
      - "guard-cli registers through the CLI distributed slice"
      - "the guard command remains discoverable by the root binary"
```

## Security EC Profile EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: guard-aw-health-security-metric
    capability_id: security-ec-profile
    claim_id: aw-health-security-metric
    contract_id: aw-health-security-metric
    category: security
    command: "./target/debug/aw ec check --project guard"
    assertions:
      - "AW EC check consumes guard report evidence as a first-class security metric"
      - "guard EC inventory remains generated and drift-free"

  - id: guard-ec-security-evidence-command
    capability_id: security-ec-profile
    claim_id: ec-security-evidence-command
    contract_id: ec-security-evidence-command
    category: security
    command: "target/debug/guard scan projects/guard --profile security-lint --compact --no-persist --vat-runner guard-security-smoke --rig-scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml --meter-target projects/guard"
    assertions:
      - "guard scan runs the full configured EC evidence command"
      - "vat, rig, and meter evidence adapters fold into guard.report/1"
```

## Dynamic Security Evidence EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: guard-vat-isolated-security-runner
    capability_id: dynamic-security-evidence
    claim_id: vat-isolated-security-runner
    contract_id: vat-isolated-security-runner
    category: security
    command: "target/debug/guard scan projects/guard --compact --no-persist --vat-runner guard-security-smoke"
    assertions:
      - "guard can fold a vat-isolated runner into its report"
      - "isolated evidence is visible without persisting guard state"

  - id: guard-rig-exploit-journey-bridge
    capability_id: dynamic-security-evidence
    claim_id: rig-exploit-journey-bridge
    contract_id: rig-exploit-journey-bridge
    category: security
    command: "target/debug/guard scan projects/guard --compact --no-persist --rig-scenario projects/guard/tests/rig/scenarios/security/guard_self_scan.toml"
    assertions:
      - "guard can fold rig scenario evidence into its report"
      - "exploit journey evidence remains executable through the guard bridge"

  - id: guard-meter-dos-resource-evidence-bridge
    capability_id: dynamic-security-evidence
    claim_id: meter-dos-resource-evidence-bridge
    contract_id: meter-dos-resource-evidence-bridge
    category: security
    command: "target/debug/guard scan projects/guard --compact --no-persist --meter-target projects/guard"
    assertions:
      - "guard can fold meter resource evidence into its report"
      - "resource-abuse evidence remains visible in guard.report/1"
```
