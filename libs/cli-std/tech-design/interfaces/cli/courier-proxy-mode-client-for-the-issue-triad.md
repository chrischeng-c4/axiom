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
  resolvers: { kind: start, label: "lib.rs adds resolve_courier_url()/resolve_courier_token(): first non-blank AXIOM_COURIER_URL/AXIOM_COURIER_TOKEN, trimmed; blank counts as unset; else None -- mirrors resolve_github_token()'s env-resolution pattern" }
  op_select: { kind: decision, label: "issue.rs online search/view/create/comment: which op" }
  search_check: { kind: decision, label: "search: resolve_courier_url() Some(url)?" }
  search_courier: { kind: process, label: "GET {courier_url}/v1/issues/{owner}/{name}?state=&q=&limit= with Authorization: Bearer <resolve_courier_token()>" }
  search_direct: { kind: process, label: "unchanged: GET api.github.com/search/issues via crate::github_get + resolve_github_token()" }
  view_check: { kind: decision, label: "view: resolve_courier_url() Some(url)?" }
  view_courier: { kind: process, label: "GET {courier_url}/v1/issues/{owner}/{name}/{number} with Authorization: Bearer <resolve_courier_token()>" }
  view_direct: { kind: process, label: "unchanged: GET api.github.com/repos/{repo}/issues/{number} via crate::github_get + resolve_github_token()" }
  create_check: { kind: decision, label: "create: resolve_courier_url() Some(url)?" }
  create_courier: { kind: process, label: "POST {courier_url}/v1/issues/{owner}/{name} (issue_payload body) with Authorization: Bearer <resolve_courier_token()>" }
  create_direct: { kind: process, label: "unchanged: POST api.github.com/repos/{repo}/issues via submit_issue + resolve_github_token()" }
  comment_check: { kind: decision, label: "comment: resolve_courier_url() Some(url)?" }
  comment_courier: { kind: process, label: "POST {courier_url}/v1/issues/{owner}/{name}/{number}/comments (comment_payload body) with Authorization: Bearer <resolve_courier_token()> -- courier reopens then comments server-side, one round trip" }
  comment_direct: { kind: process, label: "unchanged: PATCH+POST api.github.com reopen_issue then post_issue_comment via resolve_github_token()" }
  out: { kind: terminal, label: "print result / next: done" }
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
    resolvers([lib.rs adds resolve_courier_url resolve_courier_token first non-blank env trimmed else None mirrors resolve_github_token]) --> op_select{issue.rs online search view create comment which op}
    op_select -->|search| search_check{search resolve_courier_url Some}
    op_select -->|view| view_check{view resolve_courier_url Some}
    op_select -->|create| create_check{create resolve_courier_url Some}
    op_select -->|comment| comment_check{comment resolve_courier_url Some}
    search_check -->|Some| search_courier[GET courier_url v1 issues owner name state q limit Bearer courier token]
    search_check -->|None| search_direct[unchanged GET api github com search issues via github_get resolve_github_token]
    view_check -->|Some| view_courier[GET courier_url v1 issues owner name number Bearer courier token]
    view_check -->|None| view_direct[unchanged GET api github com repos issues number via github_get resolve_github_token]
    create_check -->|Some| create_courier[POST courier_url v1 issues owner name issue_payload body Bearer courier token]
    create_check -->|None| create_direct[unchanged POST api github com repos issues via submit_issue resolve_github_token]
    comment_check -->|Some| comment_courier[POST courier_url v1 issues owner name number comments comment_payload body Bearer courier token]
    comment_check -->|None| comment_direct[unchanged PATCH POST api github com reopen_issue then post_issue_comment via resolve_github_token]
    search_courier --> out([print result next done])
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
id: cli-std-courier-proxy-mode-client-verification
requirements:
  comment_courier_routing:
    id: R2d
    text: "issue::comment() routes to POST {courier_url}/v1/issues/{owner}/{name}/{number}/comments with Authorization: Bearer <courier token> when resolve_courier_url() is Some"
    kind: functional
    risk: high
    verify: comment_routes_through_courier_when_url_configured
  create_courier_routing:
    id: R2c
    text: "issue::create() routes to POST {courier_url}/v1/issues/{owner}/{name} with Authorization: Bearer <courier token> when resolve_courier_url() is Some"
    kind: functional
    risk: high
    verify: create_routes_through_courier_when_url_configured
  fallback_byte_identical:
    id: R2e
    text: "when resolve_courier_url() is None, search/view/create/comment execute the pre-existing direct api.github.com code path completely unchanged"
    kind: regression
    risk: high
    verify: issue_ops_fall_back_to_direct_github_when_courier_url_unset
  resolve_courier_token_set:
    id: R1c
    text: "resolve_courier_token() returns Some(trimmed value) when AXIOM_COURIER_TOKEN is set to a non-blank string"
    kind: functional
    risk: medium
    verify: resolve_courier_token_returns_some_when_env_set
  resolve_courier_token_unset:
    id: R1d
    text: "resolve_courier_token() returns None when AXIOM_COURIER_TOKEN is unset or blank"
    kind: functional
    risk: medium
    verify: resolve_courier_token_returns_none_when_env_unset_or_blank
  resolve_courier_url_set:
    id: R1a
    text: "resolve_courier_url() returns Some(trimmed value) when AXIOM_COURIER_URL is set to a non-blank string"
    kind: functional
    risk: medium
    verify: resolve_courier_url_returns_some_when_env_set
  resolve_courier_url_unset:
    id: R1b
    text: "resolve_courier_url() returns None when AXIOM_COURIER_URL is unset or blank, mirroring resolve_github_token()'s blank-counts-as-unset convention"
    kind: functional
    risk: medium
    verify: resolve_courier_url_returns_none_when_env_unset_or_blank
  search_courier_routing:
    id: R2a
    text: "issue::search() routes to GET {courier_url}/v1/issues/{owner}/{name}?state=&q=&limit= with Authorization: Bearer <courier token> when resolve_courier_url() is Some"
    kind: functional
    risk: high
    verify: search_routes_through_courier_when_url_configured
  view_courier_routing:
    id: R2b
    text: "issue::view() routes to GET {courier_url}/v1/issues/{owner}/{name}/{number} with Authorization: Bearer <courier token> when resolve_courier_url() is Some"
    kind: functional
    risk: high
    verify: view_routes_through_courier_when_url_configured
---
flowchart TD
    r1a[R1a resolve courier url set] --> resolve_courier_url_returns_some_when_env_set[resolve_courier_url_returns_some_when_env_set]
    r1b[R1b resolve courier url unset] --> resolve_courier_url_returns_none_when_env_unset_or_blank[resolve_courier_url_returns_none_when_env_unset_or_blank]
    r1c[R1c resolve courier token set] --> resolve_courier_token_returns_some_when_env_set[resolve_courier_token_returns_some_when_env_set]
    r1d[R1d resolve courier token unset] --> resolve_courier_token_returns_none_when_env_unset_or_blank[resolve_courier_token_returns_none_when_env_unset_or_blank]
    r2a[R2a search courier routing] --> search_routes_through_courier_when_url_configured[search_routes_through_courier_when_url_configured]
    r2b[R2b view courier routing] --> view_routes_through_courier_when_url_configured[view_routes_through_courier_when_url_configured]
    r2c[R2c create courier routing] --> create_routes_through_courier_when_url_configured[create_routes_through_courier_when_url_configured]
    r2d[R2d comment courier routing] --> comment_routes_through_courier_when_url_configured[comment_routes_through_courier_when_url_configured]
    r2e[R2e fallback byte identical] --> issue_ops_fall_back_to_direct_github_when_courier_url_unset[issue_ops_fall_back_to_direct_github_when_courier_url_unset]
```
