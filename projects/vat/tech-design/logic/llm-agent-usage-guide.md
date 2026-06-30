---
id: vat-llm-agent-usage-guide
summary: Add `vat llm [--topic <t>] [--format md|json]` as the offline agent-facing usage contract.
fill_sections: [scenarios, cli, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: agent-legible-state-and-diff-surface
    claim: agent-legible-state-and-diff-surface
    coverage: partial
    rationale: "The LLM guide makes vat's agent-facing command, evidence, and non-Docker contract directly discoverable from the CLI."
---

# Vat LLM Agent Usage Guide

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: vat-llm-agent-usage-guide-scenarios
scenarios:
  - id: llm_reads_compact_usage_contract
    given:
      - "an agent needs to learn how to use vat from the CLI"
    when:
      - "the agent runs `vat llm --topic guide`"
    then:
      - "vat prints a compact markdown guide"
      - "the guide explains `vat run <runner-id> --json` for vat.toml runner mode"
      - "the guide explains `vat run -- <command>` for direct command mode"
      - "the guide names `vat state`, `vat diff`, and `vat logs` as evidence commands"
      - "the guide states vat is not Docker, OCI, Compose, a Linux runtime, a VM, a daemon, or a long-lived process manager"
  - id: help_points_to_llm_guide
    given:
      - "an agent starts with ordinary command help"
    when:
      - "the agent runs `vat --help`"
    then:
      - "the help output lists `llm` as a command"
      - "the help output points agents to `vat llm` for the usage contract"
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat llm
    behavior:
      - "Follow the repo-wide convention: default `--topic outline`, `--format md`, with JSON available via `--format json`."
      - "`--topic guide` prints the stable detailed guide for LLM/tool agents."
      - "Exit successfully without requiring vat.toml or VAT_HOME."
      - "Mention runner mode, direct command mode, evidence commands, retention, and non-Docker boundaries."
  - name: vat --help
    behavior:
      - "Remain the clap-generated flag and command reference."
      - "Point agents to `vat llm` for the compact usage contract."
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-llm-agent-usage-guide
    name: "vat llm agent usage guide"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: agent-legible-state-and-diff-surface
    contract_id: agent-legible-state-and-diff-surface
    category: behavior
    command: "cargo test -p vat --test vat_toml_runner -- --nocapture"
    assertions:
      - "`vat llm --topic guide` exits successfully."
      - "The guide mentions vat.toml runner mode and direct command mode."
      - "The guide mentions state, diff, and logs evidence commands."
      - "The guide preserves non-Docker and non-daemon boundaries."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/commands/llm.rs
    action: create
    section: scenarios
    impl_mode: hand-written
    reason: "Implements the `llm_reads_compact_usage_contract` scenario with a stable stdout guide."
  - path: projects/vat/src/cli.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Register `vat llm` and make root help point agents to it."
  - path: projects/vat/src/commands/mod.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Expose the new llm command module."
  - path: projects/vat/src/commands/llm.rs
    action: create
    section: source
    impl_mode: hand-written
    reason: "Print the stable LLM usage guide."
  - path: projects/vat/tests/vat_toml_runner.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "Add a binary smoke test for the LLM guide contract."
  - path: projects/vat/tests/vat_toml_runner.rs
    action: validate
    section: e2e-test
    impl_mode: hand-written
    reason: "Verifies the `vat llm --topic guide` guide mentions the core agent commands and non-Docker boundaries."
  - path: projects/vat/README.md
    action: modify
    section: cli
    impl_mode: hand-written
    reason: "Document `vat llm` for operators and agents."
```
