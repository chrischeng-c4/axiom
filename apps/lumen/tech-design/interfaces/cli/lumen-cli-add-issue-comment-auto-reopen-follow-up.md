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
id: lumen-issue-comment-contract
entry: command
nodes:
  command: { kind: start, label: "Command: lumen issue comment <number> [message...]" }
  args: { kind: process, label: "Args: number required; message variadic; --repo optional; --dry-run; -y/--yes" }
  msg: { kind: process, label: "message = joined message args when non-empty; otherwise cli-std default follow-up text" }
  options: { kind: process, label: "CommentOptions { number, message, repo, dry_run, yes }" }
  delegate: { kind: process, label: "delegate to cli_std::issue::comment(&TOOL, options)" }
  dry: { kind: decision, label: "dry_run?" }
  preview: { kind: terminal, label: "stdout includes repo, issue #, state: open, message, diagnostics; exits 0" }
  mutate: { kind: terminal, label: "cli-std resolves token, reopens issue, posts comment; Lumen owns no HTTP code" }
edges:
  - { from: command, to: args }
  - { from: args, to: msg }
  - { from: msg, to: options }
  - { from: options, to: delegate }
  - { from: delegate, to: dry }
  - { from: dry, to: preview, label: "yes" }
  - { from: dry, to: mutate, label: "no" }
---
flowchart TD
    command([lumen issue comment]) --> args[number/message/repo/dry-run/yes]
    args --> msg[join message args or use cli-std default]
    msg --> options[build CommentOptions]
    options --> delegate[cli_std::issue::comment]
    delegate --> dry{dry_run?}
    dry -->|yes| preview([preview open-state diagnostics comment])
    dry -->|no| mutate([cli-std reopens issue and posts comment])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-issue-comment-contract-tests
requirements:
  issue_help_comment:
    id: R1
    text: "`issue_help_lists_search_view_create_comment` asserts `lumen issue --help` lists search, view, create, and comment."
    kind: functional
    risk: medium
    verify: test
  comment_help_flags:
    id: R2
    text: "`lumen issue comment --help` lists number/message usage plus `--repo`, `--dry-run`, and `--yes`."
    kind: functional
    risk: medium
    verify: test
  dry_run_body:
    id: R3
    text: "`lumen issue comment 123 --dry-run still broken` prints `state: open`, the message, and the shared diagnostics block."
    kind: functional
    risk: high
    verify: test
  no_network:
    id: R4
    text: "Dry-run mode succeeds without GitHub credentials and without requiring network I/O."
    kind: regression
    risk: high
    verify: test
---
flowchart TD
    r1[R1 issue help] --> cli[cargo test -p lumen --test cli_convention]
    r2[R2 comment help flags] --> cli
    r3[R3 dry-run diagnostics] --> cli
    r4[R4 no network dry-run] --> cli
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Extend IssueCommand with Comment, add IssueCommentArgs, and dispatch to cli_std::issue::comment using Lumen TOOL metadata."
  - path: apps/lumen/tests/cli_convention.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Cover issue comment help flags and dry-run output so the follow-up path remains offline-testable."
```
