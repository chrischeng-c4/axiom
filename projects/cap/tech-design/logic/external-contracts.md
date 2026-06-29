---
id: cap-external-contracts
summary: External contract gates for cap README capabilities.
fill_sections: [e2e-test]
capability_refs:
  - id: standard-agent-cli-operations
    role: primary
    gap: shared-standard-cli-commands
    claim: shared-standard-cli-commands
    coverage: full
    rationale: "The EC gate verifies cap's standard llm, upgrade, issue, and report-issue compatibility surface."
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

## Standard Agent CLI Operations EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-standard-agent-cli-operations
    capability_id: standard-agent-cli-operations
    claim_id: shared-standard-cli-commands
    contract_id: standard-agent-cli-operations
    category: behavior
    command: "cargo test -p cap --lib cli_std_convention -- --nocapture && cargo test -p cap installed_frontend_exposes_standard_agent_commands -- --nocapture && cargo build -p cap --features release"
    assertions:
      - "cap help lists llm, upgrade, issue, and report-issue compatibility commands"
      - "installed cap frontend delegates standard commands through the cap-full sibling"
      - "installed cap frontend preserves the caller environment for cap-full passthrough commands"
      - "cap llm renders cap-specific offline docs through cli-std"
      - "cap issue create and report-issue dry-run payloads carry project:cap diagnostics"
      - "release-feature builds enable cli-std online paths"
```

## Agent Hook Installation EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-agent-hook-installation
    capability_id: agent-hook-installation
    claim_id: claude-and-codex-hook-installation
    contract_id: agent-hook-installation
    category: behavior
    command: "cargo test -p cap hook_install -- --nocapture && cargo test -p cap hook -- --nocapture"
    assertions:
      - "cap init hook installation is idempotent for Claude Code and Codex CLI"
      - "unrelated user hooks are preserved"
      - "hook payload adapters rewrite Bash commands without making cap a hard dependency"
```

## Hook Payload Rewrite Adapters EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-hook-payload-rewrite-adapters
    capability_id: agent-hook-installation
    claim_id: hook-payload-rewrite-adapters
    contract_id: hook-payload-rewrite-adapters
    category: behavior
    command: "cargo test -p cap hook -- --nocapture"
    assertions:
      - "hook payload adapters rewrite Bash commands without making cap a hard dependency"
      - "recursive cap invocations and empty commands remain fail-open"
```

## Command Lease Throttling EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-command-lease-throttling
    capability_id: command-lease-throttling
    claim_id: lease-admission-and-process-supervision
    contract_id: command-lease-throttling
    category: behavior
    command: "cargo test -p cap throttle -- --nocapture && cargo test -p cap sampler -- --nocapture"
    assertions:
      - "cap leases pause, resume, and kill under configured pressure thresholds"
      - "release outcomes preserve structured kill diagnostics"
      - "sampler output is stable enough for daemon pressure decisions"
```

## Memory And CPU Pressure Sampling EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-memory-and-cpu-pressure-sampling
    capability_id: command-lease-throttling
    claim_id: memory-and-cpu-pressure-sampling
    contract_id: memory-and-cpu-pressure-sampling
    category: behavior
    command: "cargo test -p cap sampler -- --nocapture"
    assertions:
      - "sampler output is stable enough for daemon pressure decisions"
      - "memory and CPU pressure readings remain available to lease admission"
```

## Daemon Lifecycle And Status EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-daemon-lifecycle-and-status
    capability_id: daemon-lifecycle-and-status
    claim_id: daemon-process-lifecycle
    contract_id: daemon-lifecycle-and-status
    category: behavior
    command: "cargo test -p cap daemon -- --nocapture && cargo test -p cap cli -- --nocapture"
    assertions:
      - "daemon lifecycle and liveness surfaces remain callable"
      - "status and wait CLI surfaces use the daemon contract without blocking agent commands"
      - "command leases stay isolated by process group"
```

## CLI Status And Wait Surfaces EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-cli-status-and-wait-surfaces
    capability_id: daemon-lifecycle-and-status
    claim_id: cli-status-and-wait-surfaces
    contract_id: cli-status-and-wait-surfaces
    category: behavior
    command: "cargo test -p cap cli -- --nocapture"
    assertions:
      - "status and wait CLI surfaces use the daemon contract without blocking agent commands"
      - "command leases stay isolated by process group"
```

## Config Logging And Reap Policy EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-config-logging-and-reap-policy
    capability_id: config-logging-and-reap-policy
    claim_id: configuration-defaults-and-compatibility
    contract_id: config-logging-and-reap-policy
    category: behavior
    command: "cargo test -p cap config -- --nocapture && cargo test -p cap eventlog -- --nocapture && cargo test -p cap reap -- --nocapture"
    assertions:
      - "configuration defaults and compatibility rules stay stable"
      - "JSONL event log persistence roundtrips run records"
      - "reap allowlist policy stays bounded and excludes active cap leases"
```

## Run Log Persistence EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-run-log-persistence
    capability_id: config-logging-and-reap-policy
    claim_id: run-log-persistence
    contract_id: run-log-persistence
    category: behavior
    command: "cargo test -p cap eventlog -- --nocapture"
    assertions:
      - "JSONL event log persistence roundtrips run records"
      - "run evidence remains durable enough for agent diagnosis"
```

## Reap Allowlist Policy EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cap-reap-allowlist-policy
    capability_id: config-logging-and-reap-policy
    claim_id: reap-allowlist-policy
    contract_id: reap-allowlist-policy
    category: behavior
    command: "cargo test -p cap reap -- --nocapture"
    assertions:
      - "reap allowlist policy stays bounded"
      - "active cap leases are excluded from reap actions"
```
