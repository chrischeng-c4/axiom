---
id: lumen-llm-topic-invocation-docs
summary: >
  Align lumen's offline `llm` self-documentation with the repo-wide CLI
  convention so agents copy `lumen llm --topic <topic>` commands that the
  binary actually accepts.
capability_refs:
  - id: "cli-interface"
    role: primary
    gap: "lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes"
    claim: "lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes"
    coverage: partial
    rationale: "Keeps the offline llm topic surface copy-paste runnable through the standard `--topic` flag form."
  - id: "agent-offline-integration"
    role: primary
    gap: "lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes"
    claim: "lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes"
    coverage: partial
    rationale: "Ensures the self-onboarding path advertised to agents matches the parser and does not fail on the first copied command."
fill_sections: [logic, unit-test, changes]
---

# TD: Lumen llm topic invocation docs

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-llm-topic-invocation-contract
entry: start
nodes:
  start: { kind: start, label: "agent reads lumen offline self-docs" }
  convention: { kind: process, label: "repo CLI convention: lumen llm [--topic <t>] [--format md|json]" }
  docs: { kind: process, label: "module docs, README, and llm_outline_md publish runnable topic commands" }
  parse: { kind: process, label: "clap parses LlmArgs { --topic <enum>, --format <enum> }" }
  mismatch: { kind: decision, label: "published command shape matches parser?" }
  broken: { kind: terminal, label: "no: copied positional topic fails before agent can read detail topic" }
  canonical: { kind: process, label: "yes: every published topic command uses --topic <topic>" }
  gate: { kind: process, label: "test extracts topic commands from llm_outline_md() and verifies clap accepts them" }
  done: { kind: terminal, label: "agent self-onboarding commands are copy-paste runnable" }
edges:
  - { from: start, to: convention }
  - { from: convention, to: docs }
  - { from: docs, to: parse }
  - { from: parse, to: mismatch }
  - { from: mismatch, to: broken, label: "current: positional form advertised" }
  - { from: mismatch, to: canonical, label: "fixed: --topic form advertised" }
  - { from: canonical, to: gate }
  - { from: gate, to: done }
---
flowchart TD
    start([agent reads lumen offline self-docs]) --> convention[CLI convention: lumen llm --topic topic]
    convention --> docs[module docs, README, and llm_outline_md publish runnable commands]
    docs --> parse[clap parser accepts --topic enum]
    parse --> mismatch{docs match parser?}
    mismatch -->|current no| broken([copied positional topic errors])
    mismatch -->|fixed yes| canonical[canonicalize self-docs to --topic topic]
    canonical --> gate[test outline commands parse through clap]
    gate --> done([agent self-onboarding commands are runnable])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-llm-topic-invocation-verification
requirements:
  canonical_outline_commands:
    id: R1
    text: "`llm_outline_md()` advertises topic detail commands only in the convention-canonical `lumen llm --topic <topic>` form."
    kind: functional
    risk: high
    verify: test
  parser_accepts_advertised_commands:
    id: R2
    text: "Every `lumen llm --topic <topic>` command shown by `llm_outline_md()` parses successfully through the built lumen binary."
    kind: functional
    risk: high
    verify: test
  positional_form_not_reintroduced:
    id: R3
    text: "`llm_outline_md()` does not advertise positional `lumen llm <topic>` commands that clap rejects."
    kind: regression
    risk: high
    verify: test
---
flowchart TD
    r1[R1 outline publishes --topic commands] --> v1{all detail topics present with --topic?}
    r2[R2 advertised commands parse] --> v2{binary accepts each outline command?}
    r3[R3 no positional topic docs] --> v3{no rejected positional command shown?}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Update the agent-facing module documentation to point at the convention-canonical `lumen llm --topic outline` entry point."
  - path: projects/lumen/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Change `llm_outline_md()` and nearby cross-topic references so every advertised detail topic uses `lumen llm --topic <topic>`."
  - path: projects/lumen/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Update README capability surfaces from positional topic examples to `--topic` examples."
  - path: projects/lumen/tests/spec_cli.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Assert the outline publishes canonical `--topic` commands and does not advertise rejected positional topic commands."
  - path: projects/lumen/tests/cli_convention.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Run each topic command shape advertised by `llm_outline_md()` through the built lumen binary parser."
```
