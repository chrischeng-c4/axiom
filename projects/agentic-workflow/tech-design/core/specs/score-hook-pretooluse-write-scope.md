---
id: score-hook-pretooluse-write-scope
fill_sections: [logic, cli, test-plan, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# Score Hook PreToolUse Write-Scope

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: score-hook-pretooluse-write-scope-logic
entry: read_stdin
nodes:
  read_stdin:
    kind: start
    label: "read PreToolUse JSON from stdin"
  parse_payload:
    kind: process
    label: "parse JSON; on error exit 0 (fail-open)"
  extract_target:
    kind: process
    label: "extract tool_input.file_path; if empty exit 0"
  resolve_repo_root:
    kind: process
    label: "git -C <closest-existing-ancestor> rev-parse --show-toplevel; on failure exit 0"
  resolve_branch:
    kind: process
    label: "git -C <root> branch --show-current; on failure or detached HEAD exit 0"
  match_branch:
    kind: decision
    label: "branch matches ^project-(.+)$ ?"
  exit_allow_non_project:
    kind: terminal
    label: "exit 0 (non-project branch — issue-*/td-*/cb-*/main all unscoped)"
  load_config:
    kind: process
    label: "load <root>/.aw/config.toml via projects/agentic-workflow/src/services/path_scope::load_scope; on missing exit 0; on parse error exit 0 + stderr fail-open"
  resolve_project:
    kind: decision
    label: "[[projects]] entry with name == captured group exists ?"
  block_no_project_entry:
    kind: terminal
    label: "exit 2 + {decision:block, reason:'branch project-<name> has no matching [[projects]] entry'}"
  build_scope:
    kind: process
    label: "scope = path_scope::build_allowed_set(project) — collects path/, td_path/, workspaces[].paths globs"
  match_target_in_scope:
    kind: decision
    label: "target path (relative to repo root) matches any prefix or glob in scope ?"
  exit_allow_in_scope:
    kind: terminal
    label: "exit 0 (within project scope)"
  block_out_of_scope:
    kind: terminal
    label: "exit 2 + {decision:block, reason:'branch project-<name> restricts edits to <scope>; got: <rel>'}"
edges:
  - {from: read_stdin, to: parse_payload}
  - {from: parse_payload, to: extract_target}
  - {from: extract_target, to: resolve_repo_root}
  - {from: resolve_repo_root, to: resolve_branch}
  - {from: resolve_branch, to: match_branch}
  - {from: match_branch, to: exit_allow_non_project, label: "no"}
  - {from: match_branch, to: load_config, label: "yes"}
  - {from: load_config, to: resolve_project}
  - {from: resolve_project, to: block_no_project_entry, label: "no"}
  - {from: resolve_project, to: build_scope, label: "yes"}
  - {from: build_scope, to: match_target_in_scope}
  - {from: match_target_in_scope, to: exit_allow_in_scope, label: "yes"}
  - {from: match_target_in_scope, to: block_out_of_scope, label: "no"}
---
flowchart TD
    in[stdin: PreToolUse JSON] --> parse[parse JSON]
    parse --> target[extract tool_input.file_path]
    target --> root[resolve repo root]
    root --> br[resolve current branch]
    br --> isproj{branch ~= project-(.+) ?}
    isproj -- no --> okmain([exit 0 — unscoped branch])
    isproj -- yes --> cfg[load .aw/config.toml]
    cfg --> proj{[[projects]] entry has name ?}
    proj -- no --> blockno([exit 2 — no matching project entry])
    proj -- yes --> scope[build allowed prefix+glob set]
    scope --> match{target in scope ?}
    match -- yes --> okscope([exit 0 — in scope])
    match -- no --> blockout([exit 2 — out of scope])
```

The dispatcher (`aw hook <event> <kind>`) is a pure CLI shell: parse two
positional args, dispatch to the matching handler, propagate the handler's
exit code. The first handler shipped is `pretooluse write-scope`, which
absorbs the `.claude/hooks/pretooluse-project-branch-scope.py` decision tree
byte-for-byte in *observable* behavior (same stdin, same stdout, same exit
code, same stderr prefix on fail-open).

Fail-open is enforced at the binary boundary: any `panic!` caught by a
top-level `std::panic::catch_unwind` produces `exit 0` plus a single
`aw-hook: <reason>` stderr line. The hook never blocks edits because
*it* is broken; it only blocks edits because the *branch + path* are
out of scope.

The R5 reviewer note ("specify whether write-scope extracts a new shared
helper or threads through both call sites") is resolved by extracting a
**new** module `projects/agentic-workflow/src/services/path_scope.rs` that owns:
1. `load_scope(root: &Path) -> Result<ScoreScope>` — TOML load via the same
   parser already used by `services/project_registry.rs`
2. `build_allowed_set(project: &Project) -> AllowedScope` — collects
   `path`, `td_path`, and `workspaces[].paths` into a single matcher
3. `AllowedScope::contains(rel: &Path) -> bool` — uses `globset` (already
   a dependency via `workflow/test_gate.rs`)

The existing `workflow/test_gate.rs` glob-match call site is **not** retrofitted
in this slice — it has different semantics (test-target globs, not
edit-scope globs). `path_scope.rs` is a new owner; `test_gate.rs` keeps its
local `globset` use. Both depend on `globset`; neither depends on the other.

## CLI
<!-- type: cli lang: yaml -->

```yaml
new_public_entrypoints:
  hook_dispatcher: aw hook <event> <kind>
  pretooluse_write_scope: aw hook pretooluse write-scope
removed_public_entrypoints: []
namespace_contract:
  registration: "top-level aw CLI enum in projects/agentic-workflow/src/cli/commands.rs"
  module: "projects/agentic-workflow/src/cli/hook.rs"
  service_dependency: "projects/agentic-workflow/src/services/path_scope.rs"
io_contract:
  stdin: "Claude Code PreToolUse hook payload JSON: { tool_name, tool_input: { file_path, ... }, ... }"
  stdout_allow: "(empty)"
  stdout_block: "{\"decision\":\"block\",\"reason\":\"<single line>\"}\n"
  stderr_fail_open: "aw-hook: <single line reason>\n"
  exit_codes:
    "0": "allow (in-scope, unscoped branch, or fail-open)"
    "2": "block (out-of-scope or no matching [[projects]] entry)"
extension_points:
  events:
    pretooluse: "shipped"
    posttooluse: "deferred — R9 follow-up issue absorbs hook1-post-apply-validate.sh"
    sessionstart: "deferred — R9 follow-up issue absorbs hook5-session-start-idle.sh"
    subagentstart: "deferred"
    subagentstop: "deferred"
    userpromptsubmit: "deferred"
  kinds_under_pretooluse:
    write-scope: "shipped (this slice)"
```

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: score-hook-pretooluse-write-scope-test-plan
requirements:
  in_scope_path:
    id: TP-1
    text: "project-<name> branch + path under [[projects]].path → exit 0"
    risk: high
    verifymethod: test
  in_scope_td_path:
    id: TP-2
    text: "project-<name> branch + path under [[projects]].td_path → exit 0"
    risk: high
    verifymethod: test
  in_scope_workspace_glob:
    id: TP-3
    text: "project-<name> branch + path matches workspaces[].paths glob → exit 0"
    risk: high
    verifymethod: test
  out_of_scope_blocks:
    id: TP-4
    text: "project-<name> branch + path outside path/, td_path/, and workspaces globs → exit 2 + {decision:block} with explicit reason"
    risk: high
    verifymethod: test
  main_branch_unscoped:
    id: TP-5
    text: "branch == main → exit 0 regardless of path"
    risk: high
    verifymethod: test
  tracking_branches_unscoped:
    id: TP-6
    text: "branch matches issue-* / td-* / cb-* → exit 0 regardless of path"
    risk: high
    verifymethod: test
  no_matching_project_entry_blocks:
    id: TP-7
    text: "branch == project-<name> but no [[projects]] entry has that name → exit 2 with reason 'no matching project entry'"
    risk: high
    verifymethod: test
  malformed_config_fails_open:
    id: TP-8
    text: "branch == project-<name> + .aw/config.toml is malformed TOML → exit 0 + single-line stderr 'aw-hook: <reason>'"
    risk: high
    verifymethod: test
  detached_head_fails_open:
    id: TP-9
    text: "detached HEAD → exit 0 (no branch to scope against)"
    risk: medium
    verifymethod: test
  cwd_outside_repo_fails_open:
    id: TP-10
    text: "cwd outside any git repo → exit 0 (no repo to scope against)"
    risk: medium
    verifymethod: test
  panic_caught_at_boundary:
    id: TP-11
    text: "induced panic in handler → caught by top-level catch_unwind, exit 0 + stderr 'aw-hook: panic: <msg>'"
    risk: medium
    verifymethod: test
elements:
  path_scope_unit_tests:
    type: "cargo test -p agentic-workflow path_scope"
  hook_cli_unit_tests:
    type: "cargo test -p agentic-workflow-hook-cli"
  hook_cli_integration_tests:
    type: "cargo test -p agentic-workflow-hook-cli --test pretooluse_write_scope"
relations:
  - from: path_scope_unit_tests
    to: in_scope_path
    kind: verifies
  - from: path_scope_unit_tests
    to: in_scope_td_path
    kind: verifies
  - from: path_scope_unit_tests
    to: in_scope_workspace_glob
    kind: verifies
  - from: path_scope_unit_tests
    to: out_of_scope_blocks
    kind: verifies
  - from: hook_cli_integration_tests
    to: main_branch_unscoped
    kind: verifies
  - from: hook_cli_integration_tests
    to: tracking_branches_unscoped
    kind: verifies
  - from: hook_cli_integration_tests
    to: no_matching_project_entry_blocks
    kind: verifies
  - from: hook_cli_integration_tests
    to: malformed_config_fails_open
    kind: verifies
  - from: hook_cli_integration_tests
    to: detached_head_fails_open
    kind: verifies
  - from: hook_cli_integration_tests
    to: cwd_outside_repo_fails_open
    kind: verifies
  - from: hook_cli_unit_tests
    to: panic_caught_at_boundary
    kind: verifies
---
requirementDiagram
    requirement in_scope_path {
        id: TP-1
        text: "project-<name> branch + path under [[projects]].path → exit 0"
        risk: high
        verifymethod: test
    }
    requirement in_scope_td_path {
        id: TP-2
        text: "project-<name> branch + path under [[projects]].td_path → exit 0"
        risk: high
        verifymethod: test
    }
    requirement in_scope_workspace_glob {
        id: TP-3
        text: "project-<name> branch + path matches workspaces[].paths glob → exit 0"
        risk: high
        verifymethod: test
    }
    requirement out_of_scope_blocks {
        id: TP-4
        text: "project-<name> branch + path outside scope → exit 2 with reason"
        risk: high
        verifymethod: test
    }
    requirement main_branch_unscoped {
        id: TP-5
        text: "branch == main → exit 0"
        risk: high
        verifymethod: test
    }
    requirement tracking_branches_unscoped {
        id: TP-6
        text: "issue-* / td-* / cb-* → exit 0"
        risk: high
        verifymethod: test
    }
    requirement no_matching_project_entry_blocks {
        id: TP-7
        text: "project-<name> with no [[projects]] entry → exit 2"
        risk: high
        verifymethod: test
    }
    requirement malformed_config_fails_open {
        id: TP-8
        text: "malformed TOML → exit 0 + stderr"
        risk: high
        verifymethod: test
    }
    requirement detached_head_fails_open {
        id: TP-9
        text: "detached HEAD → exit 0"
        risk: medium
        verifymethod: test
    }
    requirement cwd_outside_repo_fails_open {
        id: TP-10
        text: "cwd outside any git repo → exit 0"
        risk: medium
        verifymethod: test
    }
    requirement panic_caught_at_boundary {
        id: TP-11
        text: "induced panic → caught + exit 0 + stderr"
        risk: medium
        verifymethod: test
    }
    element path_scope_unit_tests {
        type: "cargo test -p agentic-workflow path_scope"
    }
    element hook_cli_unit_tests {
        type: "cargo test -p agentic-workflow-hook-cli"
    }
    element hook_cli_integration_tests {
        type: "cargo test -p agentic-workflow-hook-cli --test pretooluse_write_scope"
    }
    path_scope_unit_tests - verifies -> in_scope_path
    path_scope_unit_tests - verifies -> in_scope_td_path
    path_scope_unit_tests - verifies -> in_scope_workspace_glob
    path_scope_unit_tests - verifies -> out_of_scope_blocks
    hook_cli_integration_tests - verifies -> main_branch_unscoped
    hook_cli_integration_tests - verifies -> tracking_branches_unscoped
    hook_cli_integration_tests - verifies -> no_matching_project_entry_blocks
    hook_cli_integration_tests - verifies -> malformed_config_fails_open
    hook_cli_integration_tests - verifies -> detached_head_fails_open
    hook_cli_integration_tests - verifies -> cwd_outside_repo_fails_open
    hook_cli_unit_tests - verifies -> panic_caught_at_boundary
```

The Scope/R3 reviewer contradiction is resolved here: the `.py` stopgap is
**deleted** in the cut-over commit (R3). No byte-for-byte regression test
against the `.py` exists; instead, each layer is independently property-tested
at the path_scope unit level (TP-1..4) and the hook CLI integration level
(TP-5..11). This is the "lean on independent property tests of each layer"
option from the review note.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/path_scope.rs
    section: source
    action: create
    impl_mode: hand-written
    description: New module owning ScoreScope load plus AllowedScope build/match for project branch edit scope.

  - path: projects/agentic-workflow/src/services/mod.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: Register path_scope service module.

  - path: projects/agentic-workflow/Cargo.toml
    section: manifest
    action: modify
    impl_mode: hand-written
    description: Keep globset as a direct dependency for path_scope and test gate matching.

  - path: projects/agentic-workflow/src/cli/hook.rs
    section: source
    action: create
    impl_mode: hand-written
    description: Implement aw hook dispatcher, pretooluse write-scope handler, fail-open panic boundary, and hook decision output contract.

  - path: projects/agentic-workflow/src/cli/mod.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: Register the hook CLI module.

  - path: projects/agentic-workflow/src/cli/commands.rs
    section: source
    action: modify
    impl_mode: hand-written
    description: Add the aw hook namespace to the top-level CLI.

  - path: projects/agentic-workflow/tests/cli/tests/hook_pretooluse_write_scope.rs
    section: e2e-test
    action: create
    impl_mode: hand-written
    description: Integration tests for write-scope branch/path decisions using real git repos and .aw/config.toml fixtures.

  - path: AGENTS.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Document the aw hook namespace.

  - path: CLAUDE.md
    action: modify
    section: cli
    impl_mode: hand-written
    description: Document the aw hook namespace.

  - path: projects/agentic-workflow/tech-design/semantic/agentic-workflow-cli.md
    section: changes
    action: modify
    impl_mode: hand-written
    description: Attach aw hook command refs to the active CLI capability traceability surface.
  - action: annotate
    section: logic
    impl_mode: hand-written
    description: "Traceability metadata edge for the logic section."

  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```

# Reviews

### Review 1
**Verdict:** approved

- [changes] Resolved by keeping `aw hook` in the main Agentic Workflow CLI module instead of a separate wrapper crate. The `catch_unwind` boundary wraps the handler dispatch in `projects/agentic-workflow/src/cli/hook.rs`, preserving allow/block exit-code semantics.
- [changes] Resolved by making `globset` a direct `projects/agentic-workflow` dependency.
- [logic] Fail-open edges stay in prose and test coverage; TP-8 through TP-11 verify malformed config, detached HEAD, outside-repo, and panic behavior.
