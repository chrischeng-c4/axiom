---
id: finish-phase-c-in-place-unconditional
fill_sections: [logic, cli, test-plan, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# Current Checkout Branch Lifecycle

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: current-checkout-branch-lifecycle
entry: cli_invocation
nodes:
  cli_invocation:
    kind: start
    label: "aw <verb> <slug> invoked from CLI CWD"
  resolve_checkout_root:
    kind: process
    label: "find_project_root(): nearest ancestor with .aw/config.toml"
  resolve_target_branch:
    kind: process
    label: "branch = <kind>-<slug>"
  ensure_clean:
    kind: process
    label: "ensure_branch_clean(checkout_root)"
  switch_branch:
    kind: process
    label: "switch_or_create_branch(checkout_root, branch)"
  run_verb:
    kind: process
    label: "read/write .aw/ under checkout_root"
  done:
    kind: terminal
    label: "emit dispatch/done/error envelope"
edges:
  - {from: cli_invocation, to: resolve_checkout_root}
  - {from: resolve_checkout_root, to: resolve_target_branch}
  - {from: resolve_target_branch, to: ensure_clean}
  - {from: ensure_clean, to: switch_branch}
  - {from: switch_branch, to: run_verb}
  - {from: run_verb, to: done}
---
flowchart TD
    cli[score verb from CLI CWD] --> root[find_project_root]
    root --> branch[branch = kind-slug]
    branch --> clean[ensure clean checkout]
    clean --> switch[switch or create branch]
    switch --> verb[verb reads/writes checkout .aw]
    verb --> done([envelope])
```

The root rule is intentionally simple: Score walks from the CLI process CWD to
the nearest `.aw/config.toml` and treats that directory as the current
checkout root. It does not inspect shared git metadata to choose another
checkout, and it does not support an alternate per-slug Score workspace model.

## CLI
<!-- type: cli lang: yaml -->

```yaml
current_public_entrypoints:
  work_items: aw wi <verb>
  tech_design: aw td create|validate|review|revise|merge|claim
  code_artifact: aw cb gen|check|claim|fill|review|revise|arbitrate
removed_public_entrypoints:
  - aw td idle
  - aw cb idle
  - score migrate-worktrees
  - aw wi prune
  - aw td init
root_contract:
  checkout_root: "find_project_root() from CLI CWD"
  writes:
    - "<checkout_root>/.aw/issues/"
    - "<checkout_root>/.aw/payloads/"
    - "<checkout_root>/.aw/tech-design/"
```

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: current-checkout-branch-lifecycle-test-plan
requirements:
  score_lib:
    id: TP-1
    text: "cargo test -p agentic-workflow --lib passes"
    risk: high
    verifymethod: test
  score_full:
    id: TP-2
    text: "cargo test -p agentic-workflow passes"
    risk: high
    verifymethod: test
  wi_linked_checkout:
    id: TP-3
    text: "run aw wi update from a linked checkout and assert the primary checkout is unchanged"
    risk: high
    verifymethod: test
  td_linked_checkout:
    id: TP-4
    text: "run aw td create from a linked checkout and assert state writes stay in that checkout"
    risk: high
    verifymethod: test
elements:
  score_tests:
    type: "cargo test -p agentic-workflow"
relations:
  - from: score_tests
    to: score_lib
    kind: verifies
  - from: score_tests
    to: score_full
    kind: verifies
  - from: score_tests
    to: wi_linked_checkout
    kind: verifies
  - from: score_tests
    to: td_linked_checkout
    kind: verifies
---
requirementDiagram
    requirement score_lib {
        id: TP-1
        text: "cargo test -p agentic-workflow --lib passes"
        risk: high
        verifymethod: test
    }
    requirement score_full {
        id: TP-2
        text: "cargo test -p agentic-workflow passes"
        risk: high
        verifymethod: test
    }
    requirement wi_linked_checkout {
        id: TP-3
        text: "run aw wi update from a linked checkout and assert the primary checkout is unchanged"
        risk: high
        verifymethod: test
    }
    requirement td_linked_checkout {
        id: TP-4
        text: "run aw td create from a linked checkout and assert state writes stay in that checkout"
        risk: high
        verifymethod: test
    }
    element score_tests {
        type: "cargo test -p agentic-workflow"
    }
    score_tests - verifies -> score_lib
    score_tests - verifies -> score_full
    score_tests - verifies -> wi_linked_checkout
    score_tests - verifies -> td_linked_checkout
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/mod.rs
    action: keep find_project_root as current-checkout .aw/config.toml walk
    impl_mode: hand-written
    section: source
  - path: projects/agentic-workflow/src/cli/commands.rs
    action: remove retired migration command from public CLI
    impl_mode: hand-written
    section: source
  - path: projects/agentic-workflow/src/cli/issues.rs
    action: remove retired prune and idle recovery surfaces
    impl_mode: hand-written
    section: source
  - path: projects/agentic-workflow/src/cli/td.rs
    action: remove retired idle surface; keep td claim branch-based
    impl_mode: hand-written
    section: source
  - path: projects/agentic-workflow/src/cli/cb.rs
    action: remove retired idle surface; make cb claim checkout-based
    impl_mode: hand-written
    section: source
  - path: AGENTS.md and skill templates
    action: remove old per-slug Score workspace workflow instructions
    section: logic
    impl_mode: hand-written
  - action: annotate
    section: cli
    impl_mode: hand-written
    description: "Traceability metadata edge for the cli section."

  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```
