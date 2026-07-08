---
id: tape-install-script
coverage_kind: semantic
capability_refs:
  - id: "cli-standard-surface"
    role: primary
    claim: "shared-llm-upgrade-issue-surface"
    gap: "shared-llm-upgrade-issue-surface"
    coverage: partial
    rationale: "The install script follows the release asset convention used by self-update."
fill_sections: [overview, logic, changes]
---

# Tape Install Script

## Overview
<!-- type: overview lang: markdown -->

`apps/tape/install.sh` downloads a `tape@*` release asset for the detected
target and installs it to `TAPE_INSTALL` or `$HOME/.local/bin`.

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-td-flow
---
flowchart TD
    start["apps/tape/install.sh"] --> target["detect target from uname"]
    target --> release["resolve tape@* release tag"]
    release --> download["download tape-<target>.tar.gz"]
    download --> checksum{"sha256 available?"}
    checksum -->|yes| verify["verify sha256"]
    checksum -->|no| extract["extract archive"]
    verify --> extract
    extract --> install["install tape to TAPE_INSTALL or ~/.local/bin"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/install.sh
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Project-local installer for Tape release assets."
```
