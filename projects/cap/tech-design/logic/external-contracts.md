---
id: cap-external-contracts
summary: External contract gates for cap README capabilities.
fill_sections: [e2e-test]
capability_refs:
  - id: agent-hook-installation
    role: primary
    gap: claude-and-codex-hook-installation
    claim: claude-and-codex-hook-installation
    coverage: full
    rationale: "The EC gate verifies hook installation and hook payload rewrite behavior."
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: full
    rationale: "The EC gate verifies lease admission, pressure response, and sampler behavior."
  - id: daemon-lifecycle-and-status
    role: primary
    gap: daemon-process-lifecycle
    claim: daemon-process-lifecycle
    coverage: full
    rationale: "The EC gate verifies daemon lifecycle and CLI status surfaces."
  - id: config-logging-and-reap-policy
    role: primary
    gap: configuration-defaults-and-compatibility
    claim: configuration-defaults-and-compatibility
    coverage: full
    rationale: "The EC gate verifies configuration defaults, event logging, and reap policy."
---

# External Contracts: cap

## Agent Hook Installation EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-agent-hook-installation
    capability_id: agent-hook-installation
    contract_id: agent-hook-installation
    category: behavior
    command: "cargo test -p cap hook_install -- --nocapture && cargo test -p cap hook -- --nocapture"
    assertions:
      - "cap init hook installation is idempotent for Claude Code and Codex CLI"
      - "unrelated user hooks are preserved"
      - "hook payload adapters rewrite Bash commands without making cap a hard dependency"
```

## Command Lease Throttling EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-command-lease-throttling
    capability_id: command-lease-throttling
    contract_id: command-lease-throttling
    category: behavior
    command: "cargo test -p cap throttle -- --nocapture && cargo test -p cap sampler -- --nocapture"
    assertions:
      - "cap leases pause, resume, and kill under configured pressure thresholds"
      - "release outcomes preserve structured kill diagnostics"
      - "sampler output is stable enough for daemon pressure decisions"
```

## Daemon Lifecycle And Status EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-daemon-lifecycle-and-status
    capability_id: daemon-lifecycle-and-status
    contract_id: daemon-lifecycle-and-status
    category: behavior
    command: "cargo test -p cap daemon -- --nocapture && cargo test -p cap cli -- --nocapture"
    assertions:
      - "daemon lifecycle and liveness surfaces remain callable"
      - "status and wait CLI surfaces use the daemon contract without blocking agent commands"
      - "command leases stay isolated by process group"
```

## Config Logging And Reap Policy EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-config-logging-and-reap-policy
    capability_id: config-logging-and-reap-policy
    contract_id: config-logging-and-reap-policy
    category: behavior
    command: "cargo test -p cap config -- --nocapture && cargo test -p cap eventlog -- --nocapture && cargo test -p cap reap -- --nocapture"
    assertions:
      - "configuration defaults and compatibility rules stay stable"
      - "JSONL event log persistence roundtrips run records"
      - "reap allowlist policy stays bounded and excludes active cap leases"
```
