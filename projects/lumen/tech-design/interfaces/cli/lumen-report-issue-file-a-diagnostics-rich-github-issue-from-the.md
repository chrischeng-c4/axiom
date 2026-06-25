---
id: lumen-cli-report-issue
summary: >
  Add a top-level `lumen report-issue` subcommand that assembles a diagnostics
  block (lumen version, target triple, git sha, built-at, OS/arch, and an
  optional running-node `/version`+`/healthz` snapshot via `--url`) together with
  the operator's free-text description, then files a GitHub issue via the REST
  API (`POST /repos/{repo}/issues`) using `GITHUB_TOKEN` and prints the created
  issue URL. Without a token (or when built without the `report-issue` feature)
  it falls back to printing a pre-filled `issues/new` URL plus the body — never a
  silent failure. `--dry-run` assembles and prints without submitting. Reuses the
  GitHub-API + reqwest-gated pattern established by `lumen upgrade`.
capability_refs:
  - id: "cli-interface"
    role: primary
    claim: "service-process-interface"
    coverage: partial
    rationale: >
      Extends lumen's command surface with an operator-facing diagnostics +
      issue-filing path, turning a problem into a well-formed GitHub issue
      without leaving the binary.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-report-issue-dispatch
entry: start
nodes:
  start:    { kind: start,    label: "lumen report-issue --title T [--message M] [--url U] [--repo R] [--label L] [--dry-run] [-y]" }
  diag:     { kind: process,  label: "gather diagnostics: version, target, git sha, built-at, OS/arch" }
  node:     { kind: decision, label: "--url given?" }
  fetch:    { kind: process,  label: "GET node /version + /healthz (degrade to 'unreachable' on error)" }
  body:     { kind: process,  label: "assemble body = message + diagnostics block" }
  dry:      { kind: decision, label: "--dry-run?" }
  print:    { kind: terminal, label: "print title + repo + body; submit nothing; exit 0" }
  token:    { kind: decision, label: "GITHUB_TOKEN set and feature built in?" }
  confirm:  { kind: decision, label: "tty and not -y -> confirm file to {repo}?" }
  abort:    { kind: terminal, label: "'aborted'; exit 0" }
  post:     { kind: process,  label: "POST /repos/{repo}/issues {title, body, labels}" }
  created:  { kind: terminal, label: "print created issue URL; exit 0" }
  fallback: { kind: terminal, label: "print pre-filled issues/new URL + body; exit 0" }
edges:
  - { from: start,   to: diag }
  - { from: diag,    to: node }
  - { from: node,    to: fetch,    label: "yes" }
  - { from: node,    to: body,     label: "no" }
  - { from: fetch,   to: body }
  - { from: body,    to: dry }
  - { from: dry,     to: print,    label: "yes" }
  - { from: dry,     to: token,    label: "no" }
  - { from: token,   to: confirm,  label: "yes" }
  - { from: token,   to: fallback, label: "no" }
  - { from: confirm, to: abort,    label: "declined" }
  - { from: confirm, to: post,     label: "yes/-y" }
  - { from: post,    to: created }
---
flowchart TD
    start([lumen report-issue]) --> diag[gather diagnostics]
    diag --> node{--url given?}
    node -->|yes| fetch[GET node /version + /healthz]
    node -->|no| body[assemble body]
    fetch --> body
    body --> dry{--dry-run?}
    dry -->|yes| print([print title + repo + body])
    dry -->|no| token{token and feature?}
    token -->|yes| confirm{confirm unless -y?}
    token -->|no| fallback([print prefilled issues/new URL])
    confirm -->|declined| abort([aborted])
    confirm -->|yes| post[POST issues]
    post --> created([print created issue URL])
```
