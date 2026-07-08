---
id: projects-tape-build-rs
coverage_kind: semantic
capability_refs:
  - id: "cli-standard-surface"
    role: primary
    claim: "shared-llm-upgrade-issue-surface"
    gap: "shared-llm-upgrade-issue-surface"
    coverage: partial
    rationale: "Build stamping supplies ToolInfo provenance for shared cli-std diagnostics."
fill_sections: [overview, logic, changes]
---

# Tape Build Stamp

## Overview
<!-- type: overview lang: markdown -->

`projects/tape/build.rs` delegates to `libs/build-stamp` with the `TAPE` prefix,
emitting `TAPE_GIT_SHA`, `TAPE_BUILT_AT`, and `TAPE_TARGET`.

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    build["cargo builds tape"] --> stamp["build_stamp::stamp(TAPE)"]
    stamp --> env["TAPE_GIT_SHA / TAPE_BUILT_AT / TAPE_TARGET"]
    env --> toolinfo["cli_std::ToolInfo diagnostics for upgrade/issue"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/tape/build.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Tape build-stamp wiring for cli-std ToolInfo provenance."
```
