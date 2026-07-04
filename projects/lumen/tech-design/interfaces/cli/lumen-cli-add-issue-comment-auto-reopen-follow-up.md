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
