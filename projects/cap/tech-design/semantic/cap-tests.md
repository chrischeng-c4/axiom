---
id: semantic-cap-tests
summary: Semantic coverage for "projects/cap/tests"
fill_sections: [schema, unit-test, changes]
capability_refs:
  - id: agent-hook-installation
    role: primary
    gap: hook-payload-rewrite-adapters
    claim: hook-payload-rewrite-adapters
    coverage: full
    rationale: "The cap tests validate hook installation, hook rewrite behavior, and same-name command rewrite compatibility."
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: full
    rationale: "The cap tests validate command lease behavior and command replacement parity for managed run surfaces."
  - id: daemon-lifecycle-and-status
    role: primary
    gap: daemon-process-lifecycle
    claim: daemon-process-lifecycle
    coverage: full
    rationale: "The cap tests validate daemon lifecycle and status behavior."
  - id: config-logging-and-reap-policy
    role: primary
    gap: configuration-defaults-and-compatibility
    claim: configuration-defaults-and-compatibility
    coverage: full
    rationale: "The cap tests validate configuration, logging, and reap policy behavior."
---

# Semantic TD: cap/tests

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "cap/tests"
  source_group: "projects/cap/tests"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/cap/tests/behavior_cap_config_logging_and_reap_policy.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "cap_config_logging_and_reap_policy"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/cap/tests"
      - path: "projects/cap/tests/behavior_cap_agent_hook_installation.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "cap_agent_hook_installation"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/cap/tests"
      - path: "projects/cap/tests/behavior_cap_command_lease_throttling.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "cap_command_lease_throttling"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/cap/tests"
      - path: "projects/cap/tests/behavior_cap_daemon_lifecycle_and_status.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "cap_daemon_lifecycle_and_status"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/cap/tests"
      - path: "projects/cap/tests/behavior_cap_hook_auto_command_optimizer_whitelist.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method", "test_case"]
        symbols:
          - name: "cap_hook_auto_command_optimizer_whitelist"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/cap/tests"
      - path: "projects/cap/tests/behavior_cap_command_replacement_parity.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["data_model", "service_method", "test_case"]
        symbols:
          - name: "active_replacements_match_success_and_error_behavior"
            kind: "function"
            public: false
          - name: "Case"
            kind: "struct"
            public: false
          - name: "new"
            kind: "function"
            public: false
          - name: "assert_success_parity"
            kind: "function"
            public: false
          - name: "assert_error_parity"
            kind: "function"
            public: false
          - name: "assert_quiet_nonzero_parity"
            kind: "function"
            public: false
          - name: "run"
            kind: "function"
            public: false
          - name: "exit_code"
            kind: "function"
            public: false
          - name: "build_cap_frontend"
            kind: "function"
            public: false
          - name: "compile_c"
            kind: "function"
            public: false
          - name: "cap_full_binary"
            kind: "function"
            public: false
          - name: "Fixture"
            kind: "struct"
            public: false
          - name: "create"
            kind: "function"
            public: false
          - name: "list_dir"
            kind: "function"
            public: false
          - name: "cat_file"
            kind: "function"
            public: false
          - name: "uniq_file"
            kind: "function"
            public: false
          - name: "find_root"
            kind: "function"
            public: false
          - name: "du_root"
            kind: "function"
            public: false
          - name: "sort_file"
            kind: "function"
            public: false
          - name: "sed_file"
            kind: "function"
            public: false
          - name: "grep_root"
            kind: "function"
            public: false
          - name: "path_string"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "unit-test"
          domain: "projects/cap/tests"
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: unit-test
coverage_kind: semantic
strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
evidence:
  source_tests:
    - path: "projects/cap/tests/behavior_cap_config_logging_and_reap_policy.rs"
    - path: "projects/cap/tests/behavior_cap_agent_hook_installation.rs"
    - path: "projects/cap/tests/behavior_cap_command_lease_throttling.rs"
    - path: "projects/cap/tests/behavior_cap_daemon_lifecycle_and_status.rs"
    - path: "projects/cap/tests/behavior_cap_hook_auto_command_optimizer_whitelist.rs"
    - path: "projects/cap/tests/behavior_cap_command_replacement_parity.rs"
---
requirementDiagram

element UT_SOURCE_TESTS {
  type: "TestEvidence"
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/cap/tests/behavior_cap_config_logging_and_reap_policy.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/cap/tests/behavior_cap_config_logging_and_reap_policy.rs"
    action: modify
    section: unit-test
    description: |
      The behavior test is executable evidence for config, logging, and reap
      policy semantics.
    impl_mode: hand-written
  - path: "projects/cap/tests/behavior_cap_agent_hook_installation.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/cap/tests/behavior_cap_agent_hook_installation.rs"
    action: modify
    section: unit-test
    description: |
      The behavior test is executable evidence for hook installation semantics.
    impl_mode: hand-written
  - path: "projects/cap/tests/behavior_cap_command_lease_throttling.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/cap/tests/behavior_cap_command_lease_throttling.rs"
    action: modify
    section: unit-test
    description: |
      The behavior test is executable evidence for command lease throttling
      semantics.
    impl_mode: hand-written
  - path: "projects/cap/tests/behavior_cap_daemon_lifecycle_and_status.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/cap/tests/behavior_cap_daemon_lifecycle_and_status.rs"
    action: modify
    section: unit-test
    description: |
      The behavior test is executable evidence for daemon lifecycle and status
      semantics.
    impl_mode: hand-written
  - path: "projects/cap/tests/behavior_cap_hook_auto_command_optimizer_whitelist.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/cap/tests/behavior_cap_hook_auto_command_optimizer_whitelist.rs"
    action: modify
    section: unit-test
    description: |
      The behavior test is executable evidence for same-name command rewrite
      and scout-only bypass semantics.
    impl_mode: hand-written
  - path: "projects/cap/tests/behavior_cap_command_replacement_parity.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-cap-tests-behavior-cap-command-replacement-parity-rs>"
  - path: "projects/cap/tests/behavior_cap_command_replacement_parity.rs"
    action: modify
    section: unit-test
    description: |
      The behavior test is executable evidence that active same-name
      replacements preserve successful output, missing-path diagnostics, and
      quiet nonzero behavior.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-cap-tests-behavior-cap-command-replacement-parity-rs>"
```
