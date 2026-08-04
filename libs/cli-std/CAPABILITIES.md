# CLI Std Capabilities

## Brief

`cli-std` is the standard agent-facing surface every axiom tool ships. The
repository convention is that a CLI is not finished when it does its job — it
is finished when an agent can drive it: ask it what it does offline (`llm`),
keep it current (`upgrade`), and file a diagnostics-bearing report when it
misbehaves (`issue`). Rebuilding that triad once per tool guarantees a dozen
slightly different answers to the same question.

This crate owns those answers once. A binary supplies a single `ToolInfo` —
its project name, repository, target triple and build stamps — and inherits
the release-tag grammar, asset naming, credential resolution order, issue body
shape and topic-rendering rules that every other axiom tool already follows.
It is deliberately clap-agnostic: each CLI keeps its own argument
registration and calls these functions, so adopting the convention never
forces a parser rewrite.

Two modules ride along that are not subcommands. `chainable` is the
conformance check behind the ecosystem's stdout contract — a CLI's output must
tell the agent what happens next, or it is a defect. `artifact` owns the
presentation hygiene shared by every CLI that renders checked-in deployment
bytes.

It does not own argument parsing, a tool's domain commands, the content of any
tool's documentation topics, or the decision to publish a release.

## Capabilities

Every capability belongs to exactly one of two feature roots:

- **Core Features** are what makes a binary a standard axiom CLI at all: an
  agent can read it offline, chain its output, update it, and report against
  it.
- **Non-Core Features** keep that surface usable in the environments the
  ecosystem actually runs in — a developer machine authenticated only through
  `gh`, an egress-restricted network reaching GitHub through courier, a
  Kubernetes cluster whose tokens live in a Secret, and an agent that needs
  stronger machine semantics than prose. Non-core does not mean optional.

This file contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Chainable Output Conformance | 3379 | implemented | verified | smoke | ready | core; a CLI's stdout carries a runnable next command or an explicit terminal marker in one of four recognized shapes, and output carrying neither fails naming the shapes it looked for |
| Offline Agent Orientation | 3379 | implemented | verified | smoke | ready | core; `llm` renders a topic map and topic bodies with no network and no configuration, in Markdown or JSON, and computed sections resolve at render time rather than being frozen at compile time |
| Diagnostics-Bearing Issue Surface | 3379 | implemented | verified | smoke | ready | core; every issue this surface files carries the reporting binary's version, target, git sha, build time and platform, and enters the tracker's typed intake queue under canonical labels |
| Self-Update From Release Assets | 3379 | implemented | verified | smoke | ready | core; the tag grammar, asset name and inner binary path derive from one project string, version selection ignores prereleases unless pinned, and downloaded bytes are checked against a published digest before replacing the running binary |
| GitHub Credential Resolution Order | 3379 | implemented | verified | smoke | ready | non-core; a fixed precedence of `$GH_TOKEN`, `$GITHUB_TOKEN`, then the `gh` credential store, with blank values skipped rather than accepted |
| Courier Proxy Routing | 3379 | implemented | verified | smoke | ready | non-core; when a courier endpoint is configured the whole issue quartet routes through it under its own bearer token, and when it is not the direct GitHub path is unchanged |
| Typed Agent Protocol v2 | 3379 | implemented | verified | smoke | ready | non-core; a schema-described task/runbook manifest for tools needing navigation semantics rather than prose, validated at construction |
| Cluster Connect Adapter | 3379 | implemented | verified | smoke | ready | non-core; a port-forward child that cannot outlive its guard, and a bearer token selected from a registry Secret by role coverage over a named resource |
| Deployment Artifact Render Hygiene | 3379 | implemented | verified | smoke | ready | non-core; source-ownership markers never reach a user-applied artifact, namespace substitution touches only namespace fields, and output resolves to a file or stdout under one rule |

### Core Features

#### Chainable Output Conformance

ID: chainable-output-conformance
Root WI: 3379
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
An agent driving a CLI needs to know what to run next. The convention is that
stdout says so, and `assert_chainable` is the check that a tool honours it.
Four shapes are recognized, and they are the real shapes in use rather than an
idealized one: the `aw.cli.v1` dispatch envelope carrying a runnable step at
`invoke.command`; the loop-driver envelope carrying it at `next.command`; the
lightweight single top-level JSON `next` field; and, for tools that emit plain
text, a fixed trailing `next: <cmd>` line. A terminal state is equally valid
and is expressed by an explicit marker — `completion.workflow_complete`,
`status: "done"`, or `next: done` — because "there is nothing left to do" is
an answer, while silence is not.

Output matching none of these is a defect, and the violation names the shapes
that were checked and not found, so a failing test tells the author what to
add rather than only that something is wrong. Empty output is rejected first
and separately: it is the one case where no shape could possibly be present,
and diagnosing it as "not valid JSON" would misdirect.

`assert_command_chainable` wraps invoking a binary under test around the same
check and returns the captured stdout, so a conformance test and the
assertions that follow it read as one sequence.

Surfaces:
- `chainable::{assert_chainable, assert_command_chainable, ChainableViolation}`

Rust internal:
- `libs/cli-std/src/chainable.rs`

EC Dimensions:
- behavior: each of the four accepted shapes passes; a terminal envelope with
  no next command passes on its marker alone; JSON with neither a command nor
  a marker fails, as does plain text whose last line is not a `next:` line.
- security: empty output is never treated as conforming, so a CLI that crashes
  before printing cannot pass the check by producing nothing.

| Gate | Evidence |
|---|---|
| The four accepted shapes | `cargo test -p cli-std --all-features --lib chainable` — `real_aw_dispatch_envelope_is_chainable`, `real_aw_run_continue_envelope_is_chainable`, `real_aw_run_done_envelope_is_chainable_via_terminal_marker`, `lightweight_json_next_field_is_chainable`, `trailing_stdout_line_is_chainable` |
| Non-conforming output is rejected | `cargo test -p cli-std --all-features --lib chainable` — `empty_output_fails`, `next_less_json_fails_with_useful_message`, `lightweight_json_next_missing_fails`, `plain_text_without_next_line_fails` |
| Process wrapper | `cargo test -p cli-std --all-features --lib chainable::tests::assert_command_chainable_wraps_process_output`, `chainable::tests::assert_command_chainable_surfaces_violation` |
| Documented examples execute | `cargo test -p cli-std --all-features --doc` (2 passed) |
| The issue surface's own output conforms | `cargo test -p cli-std --all-features --lib issue::tests::representative_issue_outputs_are_chainable` |

#### Offline Agent Orientation

ID: offline-agent-orientation
Root WI: 3379
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
`<tool> llm` answers "how do I drive this?" with no network, no configuration
and no cluster. Each CLI supplies its own topic list as the single in-code
source of truth for its agent-facing documentation; this crate renders the
uniform shapes around it. The default `outline` topic prints the topic map
plus the standard-command footer, so an agent that knows nothing about a tool
still learns the two things it needs: which topics exist, and that the `llm`
/ `upgrade` / `issue` triad is available. `--format json` renders the same
content machine-readably, and an unrecognized format falls back to Markdown
rather than failing — a malformed flag should not deny an agent its
documentation.

An unknown topic is an error, and the error lists every valid topic id. That
is the difference between a dead end and a redirect.

A static topic body is a string frozen when it was written, which is exactly
how command inventories and configuration surfaces drift away from the truth.
A `SectionedTopic` composes its body from ordered sections, and a `Generated`
section is called at render time, so the emitted content reports what is true
now. `assert_topics_render` is the conformance check a tool runs over its own
registry: it catches a generated section that renders empty, which is the
failure mode where documentation silently disappears instead of going stale
visibly.

Surfaces:
- `llm::{Topic, SectionedTopic, TopicSection, RenderedSection, RenderableTopic}`
- `llm::{render, render_sectioned, assert_topics_render}`, `llm::Format::parse`

Rust internal:
- `libs/cli-std/src/llm.rs`

EC Dimensions:
- behavior: `outline` lists every topic and the standard-command footer; an
  unknown topic names the valid ids; `Format::parse` is case-insensitive and
  defaults to Markdown; a sectioned topic renders prose before generated
  content in declaration order.
- security: rendering reads no environment, opens no socket and touches no
  file, so `llm` discloses only what the binary was compiled with.

| Gate | Evidence |
|---|---|
| Outline and topic rendering | `cargo test -p cli-std --all-features --lib llm::tests` — `outline_lists_topics_and_standard_commands`, `topic_body_and_unknown`, `static_topic_unchanged_behavior` |
| Machine-readable shapes | `cargo test -p cli-std --all-features --lib llm::tests` — `json_outline_shape`, `sectioned_json_format_shape` |
| Render-time section resolution | `cargo test -p cli-std --all-features --lib llm::tests::sectioned_topic_renders_prose_then_generated_in_order` |
| An empty generated section is caught | `cargo test -p cli-std --all-features --lib llm::tests` — `conformance_helper_catches_empty_generated_section`, `conformance_helper_passes_healthy_sectioned_topics` |

#### Diagnostics-Bearing Issue Surface

ID: diagnostics-bearing-issue-surface
Root WI: 3379
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
A bug report without provenance costs a round trip to establish what was
running. Every issue this surface files carries a diagnostics block assembled
from the reporting binary's own `ToolInfo` — version, target triple, git sha,
build timestamp and the host's OS/architecture — appended below the user's
message under a separator, so the human-authored part stays first and intact.
A blank or whitespace-only message yields diagnostics alone rather than a
leading empty section.

Created issues always carry the canonical labels `app:<project>` and
`type:report`, added only when a caller has not already supplied them, so an
issue filed from any tool lands in the tracker's typed intake queue instead of
an unlabelled pile. Callers may add domain labels; they cannot drop the two
that make the issue routable.

When no credential is available the surface does not fail — it emits a
pre-filled `issues/new` URL with the title, body and labels percent-encoded,
so the report survives the trip through a browser with its labels intact. The
comment path is the counterpart for post-closure verification failures: it
reopens the issue it comments on, because a closed issue that was not actually
fixed is worse than an open one, and it supplies a default message when the
caller has none.

Surfaces:
- `issue::{render_diagnostics, assemble_body, resolve_repo, issue_payload}`
- `issue::{report_labels, comment_payload, followup_comment_body, prefilled_url}`
- `issue::{create, comment, search, view}`, `issue::{CreateOptions, CommentOptions, SearchOptions}`

Rust internal:
- `libs/cli-std/src/issue.rs`

EC Dimensions:
- behavior: the body is message, separator, then diagnostics, and diagnostics
  alone when the message is blank; `report_labels` adds each canonical label
  only when absent; the payload omits `labels` entirely when empty; the
  pre-filled URL percent-encodes the title, body and comma-joined labels.
- security: `--repo` overrides the default target explicitly rather than
  implicitly, and no credential is ever written into the issue body or the
  pre-filled URL.

| Gate | Evidence |
|---|---|
| Diagnostics and body assembly | `cargo test -p cli-std --all-features --lib issue::tests::diagnostics_and_body` |
| Payload, labels, repo and pre-filled URL | `cargo test -p cli-std --all-features --lib issue::tests::url_and_repo_and_payload` |
| Follow-up comment and reopen body | `cargo test -p cli-std --all-features --lib issue::tests::comment_payload_and_followup_body` |
| Output is agent-chainable | `cargo test -p cli-std --all-features --lib issue::tests::representative_issue_outputs_are_chainable` |

#### Self-Update From Release Assets

ID: self-update-from-release-assets
Root WI: 3379
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
One project string determines the whole release grammar: the tag prefix
`<project>@`, the asset name `<project>-<target>.tar.gz`, and the path of the
binary inside that tarball, `<project>-<target>/<project>`. A tool that fills
in `ToolInfo` correctly cannot name its own release artifacts inconsistently,
and the release-tag helper accepts a bare version or an already-prefixed one
without double-prefixing it.

Version selection is deliberately conservative. Without a pin, `upgrade`
selects the highest stable release and ignores prereleases entirely, so a
published release candidate never upgrades anyone who did not ask for it.
With a pin, the exact version is selected — accepting either `X.Y.Z` or the
fully prefixed tag — and no highest-version logic runs at all. Tags belonging
to another tool in the same repository are not candidates: they fail the
prefix test before semver parsing is attempted.

Downloaded bytes are verified against the published sha256 digest before
anything replaces the running binary, with case-insensitive comparison and
surrounding whitespace ignored so a digest copied from a checksum file is
usable as-is. Extraction looks for the exact expected inner path and fails
with a clear message when the tarball does not contain it, rather than
installing whatever the archive happened to hold first. When the selected
version already matches the running one the action is a no-op unless the
caller forces it, and `--check` reports the next command as an upgrade only
when a strictly newer version exists.

Surfaces:
- `ToolInfo::{issue_label, tag_prefix, asset_name, inner_binary_path}`
- `upgrade::{parse_tag, select_version, verify_sha256, extract_binary, decide_action, run}`
- `upgrade::{Action, Options}`, `artifact::release_tag`

Rust internal:
- `libs/cli-std/src/lib.rs`
- `libs/cli-std/src/upgrade.rs`

EC Dimensions:
- behavior: prereleases are excluded from unpinned selection; a pin accepts
  both bare and prefixed forms; another tool's tags never parse; `decide_action`
  is `UpToDate` only when the versions are equal and force is unset.
- security: a tarball whose bytes do not match the published digest is never
  installed, and only the exact expected inner path is extracted, so a release
  asset cannot smuggle a different binary into place.

| Gate | Evidence |
|---|---|
| Tag parsing and version selection | `cargo test -p cli-std --all-features --lib upgrade::tests::parse_and_select` |
| Digest verification and install decision | `cargo test -p cli-std --all-features --lib upgrade::tests::sha_and_action` |
| Exact inner-path extraction | `cargo test -p cli-std --all-features --lib upgrade::tests::extract_inner` |
| `--check` next-command routing | `cargo test -p cli-std --all-features --lib upgrade::tests::check_next_command_prefers_upgrade_only_when_newer` |
| Release-tag normalization | `cargo test -p cli-std --all-features --lib artifact::tests::release_tag_accepts_bare_prefixed_and_default_versions` |
| Identity derivations execute as documented | `cargo test -p cli-std --all-features --doc` (the `ToolInfo` example asserts `tag_prefix` and `asset_name`) |

### Non-Core Features

#### GitHub Credential Resolution Order

ID: github-credential-resolution-order
Root WI: 3379
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
The standard operations must work for someone already authenticated the
ordinary way. `gh` stores its credential in the OS keyring and exports no
environment variable, so a tool that reads only `$GITHUB_TOKEN` appears broken
on a machine that is in fact logged in. Resolution therefore follows a fixed
precedence: `$GH_TOKEN`, then `$GITHUB_TOKEN`, then the `gh` credential store.

A blank or whitespace-only environment value is skipped rather than accepted.
An exported-but-empty variable is a configuration accident, and treating it as
a credential would produce an unauthenticated request with a confusing 401
instead of falling through to a credential that works. Resolved values are
trimmed. When nothing is available anywhere the result is `None`, which the
issue surface turns into the pre-filled-URL fallback rather than an error.

The pure resolution takes its environment lookup and its `gh` fallback as
injected functions, so the precedence is testable without mutating process
environment and without requiring `gh` to be installed.

Surfaces:
- `resolve_github_token_from` (crate-internal; `resolve_github_token` is the
  ambient-environment wrapper)

Rust internal:
- `libs/cli-std/src/lib.rs`

EC Dimensions:
- behavior: `$GH_TOKEN` wins over `$GITHUB_TOKEN`; `$GITHUB_TOKEN` wins over
  the `gh` store; a blank value at any level falls through; absence everywhere
  yields `None`.
- security: a token is never logged, never placed in a URL query string, and
  never written into an issue body; the `gh` fallback shells out only after
  both environment variables are absent or blank.

| Gate | Evidence |
|---|---|
| Precedence order | `cargo test -p cli-std --all-features --lib token_tests` — `gh_token_takes_precedence`, `github_token_is_second` |
| Blank values fall through to `gh` | `cargo test -p cli-std --all-features --lib token_tests::falls_back_to_gh_cli_when_env_absent_or_blank` |
| Absence yields no credential | `cargo test -p cli-std --all-features --lib token_tests::none_when_no_credential_anywhere` |

#### Courier Proxy Routing

ID: courier-proxy-routing
Root WI: 3379
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
Not every environment that needs to file an issue can reach `api.github.com`
directly. When `$AXIOM_COURIER_URL` is configured, the whole issue quartet —
create, comment, search and view — routes through courier's `/v1/issues`
endpoints instead, authenticated with `$AXIOM_COURIER_TOKEN` as a bearer
token. The courier credential is deliberately distinct from the GitHub
credential: the caller proves itself to the proxy, and the proxy holds the
GitHub credential, so a workstation reaching GitHub only through courier never
needs a GitHub token at all.

The routing decision is per-invocation and total. Either every one of the four
verbs goes through courier or none does; there is no partial mode where a
search is proxied and a create is not. When the endpoint is unset — or set to
a blank value, which is treated as unset — each verb falls through to the
direct GitHub path it always used, unchanged.

URL construction for both branches is pure and separately named, so the direct
path's exact bytes stay verifiable without a network call. That is the
property that matters when adding a proxy to an existing surface: the fallback
must be provably identical to what it replaced.

Surfaces:
- `$AXIOM_COURIER_URL`, `$AXIOM_COURIER_TOKEN`
- `GET|POST {courier}/v1/issues/{owner}/{name}[/{number}]`

Rust internal:
- `libs/cli-std/src/lib.rs`
- `libs/cli-std/src/issue.rs`

EC Dimensions:
- behavior: each of create, comment, search and view routes through courier
  when the URL is configured, and all four fall back to the direct GitHub URLs
  when it is not; a trailing slash on the configured URL does not produce a
  doubled path separator.
- security: courier requests carry the courier bearer token and not the GitHub
  token, and a blank endpoint value never enables proxying by accident.

| Gate | Evidence |
|---|---|
| Endpoint and token resolution | `cargo test -p cli-std --all-features --lib courier_tests` — `resolve_courier_url_returns_some_when_env_set`, `resolve_courier_url_returns_none_when_env_unset_or_blank`, `resolve_courier_token_returns_some_when_env_set`, `resolve_courier_token_returns_none_when_env_unset_or_blank` |
| All four verbs route through courier | `cargo test -p cli-std --all-features --lib issue::courier_routing_tests` — `create_routes_through_courier_when_url_configured`, `comment_routes_through_courier_when_url_configured`, `search_routes_through_courier_when_url_configured`, `view_routes_through_courier_when_url_configured` |
| The unconfigured fallback is unchanged | `cargo test -p cli-std --all-features --lib issue::courier_routing_tests::issue_ops_fall_back_to_direct_github_when_courier_url_unset` |
| The courier token authenticates courier | `cargo test -p cli-std --all-features --lib issue::courier_routing_tests::courier_get_sets_bearer_auth_header_from_courier_token` |

#### Typed Agent Protocol v2

ID: typed-agent-protocol-v2
Root WI: 3379
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
Prose topics answer "what is this?"; some tools need to answer "what task am I
performing, what inputs does it need, and what is the next step?" The `v2`
protocol (`cclab.llm.v2`) is that typed form: a document of topics carrying
tasks, typed inputs, ordered steps, runbooks with explicit constraints, and a
risk classification, described by a JSON schema the tool emits alongside the
content so a consumer can validate what it received.

The invariants are enforced at construction rather than at render time. A step
that names an unbound command must supply a template, and a template must
reference each of its typed inputs at least once — an input the template never
mentions is either a documentation bug or an input the step does not actually
take, and both are worth failing on. Building the document is where a tool
learns its manifest is malformed, not the first time an agent asks for it.

The v2 JSON output retains the compatibility fields of the v1 shape and adds
the protocol marker, so a consumer written against the older form keeps
working while a newer one can detect and use the typed semantics.

Surfaces:
- `llm::v2::{PROTOCOL, json_schema, ProtocolDocument}`
- `llm::v2::{Topic, Task, Input, Step, Runbook, Risk}`

Rust internal:
- `libs/cli-std/src/llm/v2.rs`

EC Dimensions:
- behavior: construction rejects an unbound command with no template and a
  template that omits a declared input; the emitted schema covers both the
  manifest and the typed runbook envelope; JSON output carries the protocol
  marker alongside the v1-compatible fields.
- security: the protocol is rendered from compiled-in content only — no
  environment, file or network read participates in producing it.

| Gate | Evidence |
|---|---|
| Construction-time invariants | `cargo test -p cli-std --all-features --lib llm::v2::tests` — `unbound_commands_require_a_template`, `templates_must_name_each_typed_input_exactly_once_or_more` |
| Schema coverage | `cargo test -p cli-std --all-features --lib llm::v2::tests::schema_covers_manifest_and_typed_runbook_envelopes` |
| Backward-compatible JSON shape | `cargo test -p cli-std --all-features --lib llm::v2::tests::json_keeps_compatibility_fields_and_adds_protocol` |

#### Cluster Connect Adapter

ID: cluster-connect-adapter
Root WI: 3379
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
A k8s-native service CLI's `connect` verb does the same two things every time:
open a port-forward to the service, and find a bearer token that is allowed to
talk to it. Both live behind the `k8s` feature so consumers that never touch a
cluster do not pay for the dependencies.

The port-forward child is owned by a guard that kills it on drop. A CLI that
exits, panics or returns early must not leave a forwarding process bound to a
local port, because the next invocation would then either fail to bind or —
worse — connect to a stale tunnel. Readiness is established by actually
waiting for the local port to accept a connection within a timeout, not by
sleeping and hoping, and the timeout expires with an error rather than
proceeding against a port nothing is listening on.

Token resolution walks a deliberate precedence. An explicitly supplied token
always wins and short-circuits everything, so an operator can override cluster
state. Otherwise, when a namespace and Secret are both named, the Secret's
`token-registry.json` key is base64-decoded and a token is selected whose
claims cover the requested role for the requested resource, falling back to
the wildcard `*` grant only when no specific entry matches. Roles are ordered
— read, write, admin — and coverage means meeting or exceeding, so an admin
token satisfies a read requirement while a read token never satisfies a write
one. When nothing can be resolved the result is `None` rather than an error,
because an auth-disabled deployment is a legitimate configuration.

Surfaces:
- `connect::{ChildGuard, free_local_port, wait_for_local_port_ready}`
- `connect::{Role, TokenClaims, select_token, resolve_token}`
- `connect::{cr_tokens_secret, resolve_cr_tokens_secret, kubectl_get_json, secret_data_bytes}`
- `connect::TOKEN_REGISTRY_SECRET_KEY`

Rust internal:
- `libs/cli-std/src/connect.rs`

EC Dimensions:
- behavior: `Role::covers` is reflexive and ordered read < write < admin; an
  explicit token bypasses cluster lookup; a missing namespace or Secret yields
  `None`; a resource-specific grant is preferred over the wildcard; readiness
  waiting succeeds against a bound listener and times out against a closed
  port.
- security: the forwarding child is killed when its guard drops, so a
  port-forward cannot outlive the CLI that opened it; a token whose claims do
  not cover the requested role is never selected.

| Gate | Evidence |
|---|---|
| A forwarding child cannot outlive its guard | `cargo test -p cli-std --all-features --lib connect::tests` — `child_guard_kills_process_on_drop`, `child_guard_spawn_nonexistent_binary_errs` |
| Readiness is observed, not assumed | `cargo test -p cli-std --all-features --lib connect::tests` — `wait_for_local_port_ready_succeeds_against_bound_listener`, `wait_for_local_port_ready_times_out_against_closed_port` |
| Role coverage and token selection | `cargo test -p cli-std --all-features --lib connect::tests` — `role_covers_hierarchy`, `select_token_picks_token_covering_role_for_collection_or_wildcard`, `both_registry_shapes_resolve_the_same_token` |
| Resolution precedence | `cargo test -p cli-std --all-features --lib connect::tests` — `resolve_token_prefers_explicit_token`, `resolve_token_returns_none_without_namespace_or_secret` |
| Secret and CR decoding | `cargo test -p cli-std --all-features --lib connect::tests` — `cr_tokens_secret_reads_spec_field`, `secret_data_bytes_decodes_base64_field` |

#### Deployment Artifact Render Hygiene

ID: deployment-artifact-render-hygiene
Root WI: 3379
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
Every k8s-native service CLI renders checked-in bytes for a user to build or
apply — Dockerfiles, operator manifests, CRD and instance YAML. The service
owns the body; this crate owns the presentation rules that must not differ
between them.

Source-ownership markers are removed. `# SPEC-MANAGED:` and the codegen
sentinels are instructions to this repository's own tooling; emitting them
into an artifact a user applies to their cluster leaks a maintenance concern
into a deliverable and invites someone to preserve a marker that means nothing
on their side.

Namespace substitution rewrites only namespace-bearing fields — a `name:`
under a Namespace object and `namespace:` references — rather than replacing
every occurrence of the string. A blanket text replacement would also rewrite
image names, labels and comments that happen to contain the checked-in
namespace, producing a manifest that applies cleanly and is wrong.

Output resolution follows one rule everywhere: no output path streams to
stdout; a path with an extension is the file; a path without one is a
directory the default filename goes into, created if missing. The written path
is returned only when bytes actually reached disk, so a caller can append a
chainable `next:` line without re-deriving where the artifact went. Text
artifacts end with a trailing newline, added when absent and never doubled.

Surfaces:
- `artifact::{release_tag, strip_source_ownership_markers, replace_kubernetes_namespace}`
- `artifact::{ensure_trailing_newline, write_or_print}`

Rust internal:
- `libs/cli-std/src/artifact.rs`

EC Dimensions:
- behavior: markers are stripped; only `name:`/`namespace:` fields are
  rewritten; a directory argument resolves to the default filename; a trailing
  newline is added when absent and preserved when present.
- security: an artifact handed to a user carries no source-ownership marker
  that would invite them to edit generated bytes in place.

| Gate | Evidence |
|---|---|
| Marker stripping and scoped namespace substitution | `cargo test -p cli-std --all-features --lib artifact::tests::render_hygiene_strips_markers_and_replaces_only_namespace_fields` |
| Output path resolution | `cargo test -p cli-std --all-features --lib artifact::tests::write_or_print_resolves_a_directory_to_the_default_file` |
| A trailing newline is idempotent | `cargo test -p cli-std --all-features --lib artifact::tests::trailing_newline_is_preserved_or_added` |
| Release-tag normalization | `cargo test -p cli-std --all-features --lib artifact::tests::release_tag_accepts_bare_prefixed_and_default_versions` |

## Not Promised Here

- **Argument parsing.** This crate is clap-agnostic on purpose. Each CLI owns
  its own argument registration — derive or builder — and calls these
  functions. Adopting the convention never dictates a parser.
- **A tool's domain commands.** `llm`, `upgrade` and `issue` are the standard
  surface. Everything a tool actually does is the tool's own.
- **Documentation content.** Topic ids, summaries and bodies are supplied by
  the calling binary. This crate renders them and checks that they render; it
  does not write them and cannot tell whether they are accurate.
- **Publishing releases.** `upgrade` consumes releases. Building, tagging,
  uploading assets and computing published digests belong to the release
  pipeline, and this crate never creates a release.
- **Closing issues.** The surface files, comments on and reopens issues. It
  never closes one — a closure is a human or lifecycle decision.
- **Courier itself.** The proxy's availability, its GitHub credential and its
  authorization policy belong to courier. This crate decides only whether to
  route through it and how to authenticate to it.
- **Kubernetes access.** `connect` shells out to `kubectl` and inherits
  whatever kubeconfig, context and RBAC the caller already has. It grants no
  access and creates no credential.
- **`tests/behavior_standard_agent_cli_commands_contract.rs`.** This file is
  the AW external-contract harness stub and is `#[ignore]`d in a normal
  `cargo test` run. It is not a gate for any capability above and must not be
  cited as evidence; every gate in this document names a test that runs and
  passes in the default suite.
