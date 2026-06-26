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
id: lumen-report-issue-contract
entry: start
nodes:
  start:    { kind: start,    label: "lumen report-issue --title T [--message M] [--url U] [--repo R] [--label L] [--dry-run] [-y]" }
  diag:     { kind: process,  label: "Diagnostics{version=CARGO_PKG_VERSION, target=LUMEN_TARGET, git_sha=LUMEN_GIT_SHA, built_at=LUMEN_BUILT_AT, os=consts::OS, arch=consts::ARCH}" }
  repo:     { kind: process,  label: "repo = --repo else DEFAULT_REPO (owner/name)" }
  node:     { kind: decision, label: "--url provided?" }
  fetch:    { kind: process,  label: "GET {url}/version + {url}/healthz; on error append 'node: unreachable ({url})'" }
  body:     { kind: process,  label: "body = [message?] + '\\n\\n---\\n' + render(diagnostics)" }
  dry:      { kind: decision, label: "--dry-run?" }
  print:    { kind: terminal, label: "print 'repo: {repo}', 'title: {title}', body; submit nothing; exit 0" }
  cansubmit: { kind: decision, label: "feature report-issue built AND GITHUB_TOKEN set?" }
  confirm:  { kind: decision, label: "stdin tty AND not -y -> confirm 'file issue to {repo}?'" }
  abort:    { kind: terminal, label: "'aborted'; exit 0" }
  post:     { kind: process,  label: "POST https://api.github.com/repos/{repo}/issues {title, body, labels} (bearer token, UA)" }
  okstatus: { kind: decision, label: "2xx?" }
  err:      { kind: terminal, label: "bail with GitHub status/message; exit non-zero" }
  created:  { kind: terminal, label: "print created issue html_url; exit 0" }
  fallback: { kind: terminal, label: "print 'set GITHUB_TOKEN to file directly', pre-filled {repo}/issues/new?title=&body= (percent-encoded), and body; exit 0" }
edges:
  - { from: start,     to: diag }
  - { from: diag,      to: repo }
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
    start([lumen report-issue]) --> diag[gather Diagnostics]
    diag --> repo[repo = --repo or DEFAULT_REPO]
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
id: lumen-report-issue-contract-verification
requirements:
  render_diagnostics_fields:
    id: R1
    text: "render_diagnostics(d) contains the version, target, git sha, built-at, os and arch values from the Diagnostics struct"
    kind: functional
    risk: high
    verify: test
  assemble_body_order:
    id: R2
    text: "assemble_body(Some(msg), diag) puts msg first, then a '---' separator, then the diagnostics block; assemble_body(None, diag) emits just the diagnostics block"
    kind: functional
    risk: medium
    verify: test
  prefilled_url_encodes:
    id: R3
    text: "prefilled_url(repo, title, body) yields https://github.com/{repo}/issues/new?title=..&body=.. with title and body percent-encoded (spaces, newlines, & escaped)"
    kind: functional
    risk: high
    verify: test
  resolve_repo_default_override:
    id: R4
    text: "resolve_repo(None) == DEFAULT_REPO and resolve_repo(Some(\"o/n\")) == \"o/n\""
    kind: functional
    risk: medium
    verify: test
  issue_payload_shape:
    id: R5
    text: "issue_payload(title, body, labels) serializes to a JSON object with title, body, and a labels array (omitted/empty when no labels)"
    kind: functional
    risk: medium
    verify: test
---
flowchart TD
    r1[R1 render diagnostics] --> v1{all fields present?}
    r2[R2 assemble_body] --> v2{message then --- then diagnostics?}
    r3[R3 prefilled_url] --> v3{repo + percent-encoded title/body?}
    r4[R4 resolve_repo] --> v4{default vs override?}
    r5[R5 issue_payload] --> v5{title+body+labels json?}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Wire the lumen report-issue command, flags, project label, and cli_std report_issue execution path."
  - path: libs/cli-std/src/report_issue.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Shared report-issue diagnostics, body assembly, URL prefill, repo resolution, payload shaping, and pure tests."
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract pins the binding behavior: a `Diagnostics` struct sourced from the build-time stamps + runtime consts, repo resolution (`--repo` else `DEFAULT_REPO`), optional node enrichment that degrades to an "unreachable" note, the `--dry-run` print-only exit, a gated submit path (feature + `GITHUB_TOKEN`) with confirmation and 2xx handling, and a no-token pre-filled-`issues/new` fallback. No path silently fails.
- [unit-test] R1–R5 isolate the pure seams (`render_diagnostics`, `assemble_body`, `prefilled_url` percent-encoding, `resolve_repo`, `issue_payload` JSON) so behavior is verified without network or filesystem access — matching testability=required and scope_control=strict.
