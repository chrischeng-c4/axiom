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
