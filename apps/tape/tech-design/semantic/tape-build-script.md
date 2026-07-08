---
id: tape-build-script
coverage_kind: semantic
capability_refs:
  - id: "cli-interface"
    role: primary
    claim: "tape-cli-convention-and-replay-verbs"
    gap: "tape-cli-convention-and-replay-verbs"
    coverage: partial
    rationale: "The build script is the project-local build/install entrypoint for the Tape CLI."
fill_sections: [overview, logic, changes]
---

# Tape Build Script

## Overview
<!-- type: overview lang: markdown -->

`apps/tape/build.sh` provides the project-local debug/release build and
install entrypoint expected by service-shaped projects.

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    mode["apps/tape/build.sh <debug|release>"] --> debug{"debug?"}
    debug -->|yes| dbg["cargo build -p tape --bin tape"]
    debug -->|no| rel["cargo build --release -p tape --bin tape --features self-update issue"]
    dbg --> install["install tape to TAPE_INSTALL or ~/.cargo/bin"]
    rel --> install
    install --> verify["print tape --version verification hint"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/build.sh
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Project-local build/install wrapper for Tape."
```
