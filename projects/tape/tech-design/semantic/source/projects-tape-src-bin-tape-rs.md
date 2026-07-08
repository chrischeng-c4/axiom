---
id: projects-tape-src-bin-tape-rs
coverage_kind: semantic
capability_refs:
  - id: "cli-interface"
    role: primary
    claim: "tape-cli-convention-and-replay-verbs"
    gap: "tape-cli-convention-and-replay-verbs"
    coverage: partial
    rationale: "The binary exposes append/replay/checkpoint/spec/llm/upgrade/issue commands."
  - id: "cli-standard-surface"
    role: primary
    claim: "shared-llm-upgrade-issue-surface"
    gap: "shared-llm-upgrade-issue-surface"
    coverage: partial
    rationale: "The binary delegates llm/upgrade/issue to cli-std."
  - id: "chainable-output-conformance"
    role: primary
    claim: "local-replay-command-next-markers"
    gap: "local-replay-command-next-markers"
    coverage: partial
    rationale: "Operational commands emit a next marker or terminal next: done."
fill_sections: [overview, logic, unit-test, changes]
---

# Tape CLI Surface

## Overview
<!-- type: overview lang: markdown -->

`projects/tape/src/bin/tape.rs` is the first agent-facing Tape CLI. It wraps the
local `TapeJournal` file store and exposes standard ecosystem commands through
`cli-std`.

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    cli["tape CLI"] --> append["append: load store -> append event -> save -> print next replay"]
    cli --> replay["replay: load store -> print events -> next: done"]
    cli --> checkpoint["checkpoint get|put: read or advance durable cursor"]
    cli --> spec["spec: print routes/openapi/schema"]
    cli --> std["llm/upgrade/issue: delegate to cli-std"]
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    test["cargo test -p tape --test cli_contract -- --nocapture"] --> help["help_ships_standard_and_replay_commands"]
    test --> roundtrip["append_replay_checkpoint_roundtrip"]
    help --> surface["standard and Tape-specific commands visible"]
    roundtrip --> workflow["local file-backed workflow passes"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/tape/src/bin/tape.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Initial Tape CLI over the local replay journal and shared cli-std commands."
  - path: projects/tape/tests/cli_contract.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Binary contract tests proving the Tape CLI workflow."
```
