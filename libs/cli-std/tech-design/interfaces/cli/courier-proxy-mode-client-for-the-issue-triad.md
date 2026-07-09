---
id: cli-std-courier-proxy-mode-client
summary: >
  Add an optional client-side courier proxy-mode to `libs/cli-std`'s
  `issue` triad. `resolve_courier_url()`/`resolve_courier_token()` in
  `lib.rs` resolve `AXIOM_COURIER_URL`/`AXIOM_COURIER_TOKEN`, mirroring
  `resolve_github_token()`'s env-resolution style. When a courier URL is
  configured, `issue::{search,view,create,comment}` route through the
  `courier` service's `/v1/issues/...` endpoints with `Authorization:
  Bearer <courier token>` instead of `api.github.com`; when unset, the
  existing direct-GitHub path runs unchanged (byte-identical fallback).
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: standard-agent-cli-commands
    role: primary
    claim: standard-agent-cli-commands-contract
    coverage: partial
    rationale: "Adds an optional courier-proxy transport to the shared issue triad so callers can share one server-side GitHub credential instead of each needing their own; the direct-GitHub path (the capability's existing evidence) stays byte-identical when courier is unconfigured."
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cli-std-courier-proxy-mode-client-contract
entry: resolvers
nodes:
  resolvers: { kind: start, label: "pub fn resolve_courier_url() -> Option<String> and pub fn resolve_courier_token() -> Option<String> in lib.rs, cfg online: read env var, trim, None if empty, else Some(trimmed) -- same shape as resolve_github_token() -> Option<String>" }
  op_select: { kind: decision, label: "issue.rs cfg online: search(tool,SearchOptions) view(tool,repo,number) create(tool,CreateOptions) comment(tool,repo,number,CommentOptions) -- which op" }
  search_check: { kind: decision, label: "search: crate::resolve_courier_url() Some(url)?" }
  search_courier: { kind: process, label: "http_client GET format(url v1 issues owner name) query state q limit header Authorization Bearer resolve_courier_token unwrap_or_default parse same JSON shape as api.github.com search response" }
  search_direct: { kind: process, label: "unchanged crate::github_get GET api.github.com search issues q resolve_github_token" }
  view_check: { kind: decision, label: "view: crate::resolve_courier_url() Some(url)?" }
  view_courier: { kind: process, label: "http_client GET format(url v1 issues owner name number) header Authorization Bearer resolve_courier_token unwrap_or_default parse same JSON shape as api.github.com issue response" }
  view_direct: { kind: process, label: "unchanged crate::github_get GET api.github.com repos issues number resolve_github_token" }
  create_check: { kind: decision, label: "create: crate::resolve_courier_url() Some(url)?" }
  create_courier: { kind: process, label: "http_client POST format(url v1 issues owner name) json issue_payload header Authorization Bearer resolve_courier_token unwrap_or_default parse same JSON shape as submit_issue" }
  create_direct: { kind: process, label: "unchanged submit_issue POST api.github.com repos issues resolve_github_token" }
  comment_check: { kind: decision, label: "comment: crate::resolve_courier_url() Some(url)?" }
  comment_courier: { kind: process, label: "http_client POST format(url v1 issues owner name number comments) json comment_payload header Authorization Bearer resolve_courier_token unwrap_or_default -- courier reopens then comments server side one round trip" }
  comment_direct: { kind: process, label: "unchanged reopen_issue then post_issue_comment PATCH POST api.github.com resolve_github_token" }
  out: { kind: terminal, label: "return parsed result to caller, identical Result and struct shapes on both branches" }
edges:
  - { from: resolvers, to: op_select }
  - { from: op_select, to: search_check, label: "search" }
  - { from: op_select, to: view_check, label: "view" }
  - { from: op_select, to: create_check, label: "create" }
  - { from: op_select, to: comment_check, label: "comment" }
  - { from: search_check, to: search_courier, label: "Some" }
  - { from: search_check, to: search_direct, label: "None" }
  - { from: view_check, to: view_courier, label: "Some" }
  - { from: view_check, to: view_direct, label: "None" }
  - { from: create_check, to: create_courier, label: "Some" }
  - { from: create_check, to: create_direct, label: "None" }
  - { from: comment_check, to: comment_courier, label: "Some" }
  - { from: comment_check, to: comment_direct, label: "None" }
  - { from: search_courier, to: out }
  - { from: search_direct, to: out }
  - { from: view_courier, to: out }
  - { from: view_direct, to: out }
  - { from: create_courier, to: out }
  - { from: create_direct, to: out }
  - { from: comment_courier, to: out }
  - { from: comment_direct, to: out }
---
flowchart TD
    resolvers([resolve_courier_url resolve_courier_token in lib.rs read env trim None if empty else Some same shape as resolve_github_token]) --> op_select{issue.rs search view create comment which op}
    op_select -->|search| search_check{search resolve_courier_url Some}
    op_select -->|view| view_check{view resolve_courier_url Some}
    op_select -->|create| create_check{create resolve_courier_url Some}
    op_select -->|comment| comment_check{comment resolve_courier_url Some}
    search_check -->|Some| search_courier[GET url v1 issues owner name query state q limit Bearer resolve_courier_token]
    search_check -->|None| search_direct[unchanged github_get GET api github com search issues resolve_github_token]
    view_check -->|Some| view_courier[GET url v1 issues owner name number Bearer resolve_courier_token]
    view_check -->|None| view_direct[unchanged github_get GET api github com repos issues number resolve_github_token]
    create_check -->|Some| create_courier[POST url v1 issues owner name issue_payload Bearer resolve_courier_token]
    create_check -->|None| create_direct[unchanged submit_issue POST api github com repos issues resolve_github_token]
    comment_check -->|Some| comment_courier[POST url v1 issues owner name number comments comment_payload Bearer resolve_courier_token]
    comment_check -->|None| comment_direct[unchanged reopen_issue post_issue_comment api github com resolve_github_token]
    search_courier --> out([return parsed result identical shape both branches])
    search_direct --> out
    view_courier --> out
    view_direct --> out
    create_courier --> out
    create_direct --> out
    comment_courier --> out
    comment_direct --> out
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: (fill: spec-id)-verification
requirements:
  example_requirement:
    id: R1
    text: "(fill: requirement text)"
    kind: functional
    risk: medium
    verify: (fill: concrete verification target, e.g. a test name)
---
flowchart TD
    r1[R1 example requirement] --> fill_concrete_verification_target_e_g_a_test_name[(fill: concrete verification target, e.g. a test name)]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/cli-std/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "gap=cli-std-logic-flowchart-patch-fn tracker=#1320 reason: the generic Mermaid-flowchart-to-Rust generator only synthesizes one brand-new function named after the diagram's entry node with todo!() bodies; it cannot target insertions into existing named functions or emit multiple precisely-named functions from one diagram. Hand-write resolve_courier_url() and resolve_courier_token() (both #[cfg(feature = \"online\")]), mirroring resolve_github_token()'s env-resolution pattern exactly: read AXIOM_COURIER_URL / AXIOM_COURIER_TOKEN, trim, return None when unset or blank (blank counts as unset), else Some(trimmed value). No fallback subprocess (unlike resolve_github_token()'s gh CLI fallback) -- courier has no local-CLI credential source."
  - path: libs/cli-std/src/issue.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "gap=cli-std-logic-flowchart-patch-fn tracker=#1320 reason: same generator gap as lib.rs above -- the flowchart generator cannot patch branches into the existing search/view/create/comment functions. Hand-write: in the #[cfg(feature = \"online\")] search/view/create/comment functions, branch on crate::resolve_courier_url(): when Some(url), build the request against courier's /v1/issues/{owner}/{name}[/{number}[/comments]] endpoints with header Authorization: Bearer <crate::resolve_courier_token() value> (search: GET {url}/v1/issues/{owner}/{name}?state=&q=&limit=; view: GET {url}/v1/issues/{owner}/{name}/{number}; create: POST {url}/v1/issues/{owner}/{name} with issue_payload() body; comment: POST {url}/v1/issues/{owner}/{name}/{number}/comments with comment_payload() body). When None, execute today's existing direct api.github.com code path completely unchanged. The #[cfg(not(feature = \"online\"))] offline stubs are untouched."
  - path: libs/cli-std/src/issue.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "gap=cli-std-unit-test-generator tracker=#1320 reason: the unit-test generator emits an empty CODEGEN block for this project (no test-body synthesis primitive yet). Hand-write #[cfg(test)] cases (using the existing http mock/test harness pattern in issue.rs's tests module) covering: proxy-mode URL/token resolution (Some/None for both AXIOM_COURIER_URL and AXIOM_COURIER_TOKEN, blank-as-unset), proxy-mode request routing for search/view/create/comment (correct courier method+path+Authorization: Bearer header), and fallback-to-direct-GitHub behavior byte-identical to pre-change when the courier env vars are unset."
  - path: libs/cli-std/tech-design/semantic/source/libs-cli-std-src-lib-rs.md
    action: modify
    section: source
    impl_mode: codegen
    description: "Regenerate the rust-source-unit mirror for lib.rs so it includes resolve_courier_url()/resolve_courier_token() after aw td gen/fill."
  - path: libs/cli-std/tech-design/semantic/source/libs-cli-std-src-issue-rs.md
    action: modify
    section: source
    impl_mode: codegen
    description: "Regenerate the rust-source-unit mirror for issue.rs so it includes the courier-proxy branches in search/view/create/comment after aw td gen/fill."
```
