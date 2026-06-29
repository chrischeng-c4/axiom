---
id: lumen-cli-issue-group
summary: >
  Ship the standard top-level `lumen issue` command group required by the
  ecosystem CLI convention: `issue search [query]`, `issue view <n>`, and
  `issue create [--title <t>] [msg...]`. Search/view read `project:lumen`
  tracker issues through `cli_std::issue`; create assembles a diagnostics block
  (lumen version, target triple, git sha, built-at, OS/arch, and optional
  running-node `/version`+`/healthz` snapshot via `--url`) with the operator's
  message, tags the report `project:lumen`, and either files it through GitHub
  or prints a pre-filled fallback URL. The deprecated `lumen report-issue`
  command is removed from the command surface.
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
id: lumen-issue-group-contract
entry: start
nodes:
  start:    { kind: start,    label: "lumen issue <search|view|create>" }
  search:   { kind: terminal, label: "issue search [query] [--state open|closed|all] [--limit N] -> cli_std::issue::search(project:lumen)" }
  view:     { kind: terminal, label: "issue view <n> -> cli_std::issue::view" }
  create:   { kind: process,  label: "issue create [--title T] [msg...] [--url U] [--repo R] [--label L] [--dry-run] [-y]" }
  diag:     { kind: process,  label: "Diagnostics{version=CARGO_PKG_VERSION, target=LUMEN_TARGET, git_sha=LUMEN_GIT_SHA, built_at=LUMEN_BUILT_AT, os=consts::OS, arch=consts::ARCH}" }
  repo:     { kind: process,  label: "repo = --repo else DEFAULT_REPO (owner/name)" }
  node:     { kind: decision, label: "--url provided?" }
  fetch:    { kind: process,  label: "GET {url}/version + {url}/healthz; on error append 'node: unreachable ({url})'" }
  body:     { kind: process,  label: "body = [message?] + '\\n\\n---\\n' + render(diagnostics)" }
  dry:      { kind: decision, label: "--dry-run?" }
  print:    { kind: terminal, label: "print 'repo: {repo}', 'title: {title}', body; submit nothing; exit 0" }
  title:    { kind: process,  label: "title = --title else 'lumen: ' + first message line else 'lumen: issue report'" }
  labels:   { kind: process,  label: "labels = project:lumen + any --label values" }
  cansubmit: { kind: decision, label: "online build AND GitHub token available?" }
  confirm:  { kind: decision, label: "stdin tty AND not -y -> confirm 'file issue to {repo}?'" }
  abort:    { kind: terminal, label: "'aborted'; exit 0" }
  post:     { kind: process,  label: "POST https://api.github.com/repos/{repo}/issues {title, body, labels} (bearer token, UA)" }
  okstatus: { kind: decision, label: "2xx?" }
  err:      { kind: terminal, label: "bail with GitHub status/message; exit non-zero" }
  created:  { kind: terminal, label: "print created issue html_url; exit 0" }
  fallback: { kind: terminal, label: "print 'set GITHUB_TOKEN to file directly', pre-filled {repo}/issues/new?title=&body= (percent-encoded), and body; exit 0" }
edges:
  - { from: start,     to: search,    label: "search" }
  - { from: start,     to: view,      label: "view" }
  - { from: start,     to: create,    label: "create" }
  - { from: create,    to: title }
  - { from: title,     to: diag }
  - { from: diag,      to: labels }
  - { from: labels,    to: repo }
  - { from: repo,      to: node }
  - { from: node,      to: fetch,     label: "yes" }
  - { from: node,      to: body,      label: "no" }
  - { from: fetch,     to: body }
  - { from: body,      to: dry }
  - { from: dry,       to: print,     label: "yes" }
  - { from: dry,       to: cansubmit, label: "no" }
  - { from: cansubmit, to: confirm,   label: "yes" }
  - { from: cansubmit, to: fallback,  label: "no" }
  - { from: confirm,   to: abort,     label: "declined" }
  - { from: confirm,   to: post,      label: "yes/-y" }
  - { from: post,      to: okstatus }
  - { from: okstatus,  to: err,       label: "no" }
  - { from: okstatus,  to: created,   label: "yes" }
---
flowchart TD
    start([lumen issue]) --> search([search project:lumen issues])
    start --> view([view one issue])
    start --> create[create diagnostics-rich issue]
    create --> title[title from --title or message]
    title --> diag[gather Diagnostics]
    diag --> labels[labels = project:lumen + --label]
    labels --> repo[repo = --repo or DEFAULT_REPO]
    repo --> node{--url provided?}
    node -->|yes| fetch[GET /version + /healthz; degrade on error]
    node -->|no| body[body = message + diagnostics]
    fetch --> body
    body --> dry{--dry-run?}
    dry -->|yes| print([print repo+title+body])
    dry -->|no| cansubmit{feature + GITHUB_TOKEN?}
    cansubmit -->|yes| confirm{confirm unless -y?}
    cansubmit -->|no| fallback([print prefilled issues/new URL])
    confirm -->|declined| abort([aborted])
    confirm -->|yes| post[POST issues]
    post --> okstatus{2xx?}
    okstatus -->|no| err([bail with status])
    okstatus -->|yes| created([print issue html_url])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-issue-group-contract-verification
requirements:
  help_surface:
    id: R1
    text: "`lumen --help` lists llm, upgrade, and issue, and no longer lists report-issue"
    kind: functional
    risk: high
    verify: test
  issue_subcommands:
    id: R2
    text: "`lumen issue --help` lists search, view, and create"
    kind: functional
    risk: high
    verify: test
  search_filters_project:
    id: R3
    text: "issue search delegates to cli_std::issue::search with tool.project = lumen, so results filter to project:lumen"
    kind: functional
    risk: medium
    verify: test
  create_labels_project:
    id: R4
    text: "issue create always includes project:lumen before any user labels"
    kind: functional
    risk: high
    verify: test
  diagnostics_fallback:
    id: R5
    text: "issue create preserves diagnostics, --url enrichment, --repo, --dry-run, and no-token fallback through cli_std::issue::create"
    kind: functional
    risk: medium
    verify: test
---
flowchart TD
    r1[R1 help surface] --> v1{issue shown/report-issue absent?}
    r2[R2 issue subcommands] --> v2{search/view/create shown?}
    r3[R3 search] --> v3{project:lumen filter?}
    r4[R4 create labels] --> v4{project:lumen included?}
    r5[R5 create diagnostics] --> v5{cli_std create path?}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Replace the deprecated lumen report-issue command with the standard issue search/view/create group wired to cli_std::issue."
  - path: projects/lumen/tests/cli_convention.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Verify the standard issue group appears in help and deprecated report-issue is absent without relying on live GitHub reads/writes."
```

# Reviews

### Review 2
**Verdict:** approved

- [logic] Contract now matches the ecosystem CLI convention: Lumen exposes the
  `issue` group with read-only search/view and diagnostics-rich create. Create
  preserves the useful report path (`--url`, `--repo`, `--label`, `--dry-run`,
  `-y`) while delegating to `cli_std::issue::create` instead of the deprecated
  `report_issue` shim.
- [unit-test] Help-surface tests verify the agent-facing contract without
  network access: `llm`, `upgrade`, and `issue` appear, `report-issue` is gone,
  and `issue` lists `search`, `view`, and `create`.
