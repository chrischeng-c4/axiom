---
id: projects-jet-logic-inspector-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# jet `--debug` + `page.pause()` (P4.2 MVP)

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/inspector.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# jet `--debug` + `page.pause()` (P4.2 MVP)

### Overview

Phase 7 P4.2 MVP. Adds two minimal hooks for interactive test debugging:

- `jet test --debug` flips the runner into headed + `workers=1` so the
  Chromium window is visible and only one spec runs at a time.
- `await page.pause()` in a spec blocks for up to 30 minutes (or until
  the test-level timeout) so the dev can poke at the page by hand in
  the open window.

Deferred (the "real" Playwright inspector):
- In-window step-through UI with "Resume / Step Over" buttons.
- `--debug` auto-injecting a breakpoint before the first action.
- Keyboard-driven resume via stdin.

Both would require wiring a dedicated control channel between the test
worker and the user's terminal, plus a UI panel injected into the
page — substantial work beyond this session's MVP.

### Design Contract

```mermaid
---
id: jet-inspector-requirements
entry: I1
---
requirementDiagram
    requirement I1 {
        id: I1
        text: jet test debug forces headed mode workers one and prints a one line notice
        risk: medium
        verifymethod: test
    }
    requirement I2 {
        id: I2
        text: page pause waits up to thirty minutes never throws and is no op in headless runs
        risk: medium
        verifymethod: test
    }
    requirement I3 {
        id: I3
        text: Test timeout still preempts page pause and reports timed out
        risk: medium
        verifymethod: test
    }
```

### Changes

```yaml
_sdd:
  id: inspector-changes
  refs:
    - $ref: "watch-mode"
changes:
  - path: crates/jet/src/cli.rs
    action: modify
    section: cli
    impl_mode: hand-written
    purpose: |
      Add `--debug` flag. In the test handler, when set, force
      cfg.headless=false + cfg.workers=1 and print a one-line notice.
  - path: crates/jet/runtime/test/page.js
    action: modify
    section: logic
    impl_mode: hand-written
    purpose: "Add Page.pause() — 30-minute setTimeout-based hold."
  - path: .aw/tech-design/crates/jet/logic/inspector.md
    action: create
    section: doc
    impl_mode: hand-written
    purpose: "This spec."
```
