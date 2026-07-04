---
id: lumen-cli-issue-comment-follow-up
summary: >
  Extend the standard `lumen issue` group with `comment`, delegating reopen and
  diagnostics-rich follow-up posting to `cli_std::issue::comment` while keeping
  existing search, view, and create behavior unchanged.
capability_refs:
  - id: "cli-interface"
    role: primary
    gap: "service-process-interface"
    claim: "service-process-interface"
    coverage: partial
    rationale: >
      Completes the standard issue group follow-up path so agents can reopen
      and comment on Lumen tracker issues after user-side verification fails.
fill_sections: [logic, unit-test, changes]
---

# TD: Lumen CLI issue comment follow-up

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-issue-comment-applicability
entry: start
nodes:
  start: { kind: start, label: "lumen issue comment <number> [message...]" }
  parse: { kind: process, label: "Clap parses issue number, free-form follow-up message, --repo, --dry-run, --yes" }
  opts: { kind: process, label: "Build cli_std::issue::CommentOptions with TOOL metadata" }
  dry: { kind: decision, label: "--dry-run?" }
  preview: { kind: terminal, label: "cli-std prints repo, issue #, state: open, diagnostics comment; no network mutation" }
  online: { kind: process, label: "cli_std::issue::comment handles token lookup, reopen, and POST comment" }
  done: { kind: terminal, label: "existing search/view/create behavior unchanged" }
edges:
  - { from: start, to: parse }
  - { from: parse, to: opts }
  - { from: opts, to: dry }
  - { from: dry, to: preview, label: "yes" }
  - { from: dry, to: online, label: "no" }
  - { from: online, to: done }
  - { from: preview, to: done }
---
flowchart TD
    start([lumen issue comment number message]) --> parse[parse number/message/repo/dry-run/yes]
    parse --> opts[build cli_std CommentOptions with Lumen TOOL metadata]
    opts --> dry{--dry-run?}
    dry -->|yes| preview([print open-state diagnostics comment; no mutation])
    dry -->|no| online[cli_std handles credential lookup, reopen issue, post comment]
    online --> done([search/view/create unchanged])
    preview --> done
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-issue-comment-verification
requirements:
  help_surface:
    id: R1
    text: "`lumen issue --help` lists `comment` alongside search/view/create."
    kind: functional
    risk: medium
    verify: test
  dry_run_preview:
    id: R2
    text: "`lumen issue comment 123 --dry-run ...` exits successfully and prints repo, issue number, `state: open`, follow-up message, and diagnostics without network mutation."
    kind: functional
    risk: high
    verify: test
  shared_api:
    id: R3
    text: "Dispatch calls `cli_std::issue::comment` with `CommentOptions`; Lumen does not duplicate GitHub reopen/comment HTTP logic."
    kind: design
    risk: medium
    verify: code-review
  regression:
    id: R4
    text: "Existing search/view/create help and standard CLI convention tests keep passing."
    kind: regression
    risk: medium
    verify: test
---
flowchart TD
    r1[R1 help lists comment] --> cli_convention[cargo test -p lumen --test cli_convention]
    r2[R2 dry-run preview] --> cli_convention
    r4[R4 existing issue group unchanged] --> cli_convention
    r3[R3 shared cli_std dispatch] --> code_review[review projects/lumen/src/bin/lumen.rs dispatch]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add IssueCommand::Comment args and dispatch to cli_std::issue::comment with repo, dry-run, yes, and free-form message support."
  - path: projects/lumen/tests/cli_convention.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Assert `lumen issue --help` lists comment and `lumen issue comment ... --dry-run` prints the shared reopen/comment preview."
```
