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
  start: { kind: start, label: "agent reads Lumen's offline entry points" }
  convention: { kind: process, label: "CONTRIBUTING contract: llm [--topic <t>] [--format md|json]" }
  parser: { kind: process, label: "LlmArgs remains flag-only: --topic defaults to outline, --format defaults to md" }
  docs: { kind: process, label: "module docs, README, and llm_outline_md advertise the same --topic commands" }
  outline: { kind: process, label: "llm_outline_md lists workflow/integration/quickstart/auth/storage/recipes as `lumen llm --topic <topic>`" }
  readme: { kind: process, label: "README brief and Agent Offline Integration surfaces use --topic examples" }
  tests: { kind: process, label: "spec_cli guards canonical text; cli_convention executes each advertised topic command" }
  done: { kind: terminal, label: "copying the self-docs command shape succeeds against the binary parser" }
edges:
  - { from: start, to: convention }
  - { from: convention, to: parser }
  - { from: parser, to: docs }
  - { from: docs, to: outline }
  - { from: outline, to: readme }
  - { from: readme, to: tests }
  - { from: tests, to: done }
---
flowchart TD
    start([agent reads Lumen offline entry points]) --> convention[CLI convention: llm --topic topic]
    convention --> parser[LlmArgs remains flag-only]
    parser --> docs[self-docs advertise the parser shape]
    docs --> outline[outline lists detail topics with --topic]
    outline --> readme[README surfaces use --topic examples]
    readme --> tests[spec text + binary parser tests]
    tests --> done([copied self-doc commands parse])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-llm-topic-invocation-verification
requirements:
  outline_uses_topic_flag:
    id: R1
    text: "`projects/lumen/tests/spec_cli.rs::llm_outline_maps_agent_topics` asserts all detail-topic examples use `lumen llm --topic <topic>`."
    kind: functional
    risk: high
    verify: test
  outline_rejects_positional_docs:
    id: R2
    text: "`spec_cli` rejects the old `lumen llm workflow` / positional topic examples in `llm_outline_md()`."
    kind: regression
    risk: high
    verify: test
  advertised_commands_parse:
    id: R3
    text: "`projects/lumen/tests/cli_convention.rs` invokes the built lumen binary with each outline-advertised `--topic` command."
    kind: functional
    risk: high
    verify: test
---
flowchart TD
    r1[R1 canonical outline examples] --> spec_cli[cargo test -p lumen --test spec_cli]
    r2[R2 positional docs not reintroduced] --> spec_cli
    r3[R3 advertised commands parse] --> cli_convention[cargo test -p lumen --test cli_convention]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Change the module-level agent entry hint from `lumen llm outline` to `lumen llm --topic outline` and annotate it to this TD."
  - path: projects/lumen/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Update `llm_outline_md()` topic bullets and nearby cross-topic references from positional `lumen llm <topic>` to canonical `lumen llm --topic <topic>` text."
  - path: projects/lumen/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Update README brief and Agent Offline Integration surfaces to show `lumen llm --topic ...` examples."
  - path: projects/lumen/tests/spec_cli.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Add #824 `@spec` regression assertions for canonical outline topic examples and absence of rejected positional examples."
  - path: projects/lumen/tests/cli_convention.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Add #824 `@spec` binary smoke coverage that each advertised topic command parses through the built `lumen` binary."
```
