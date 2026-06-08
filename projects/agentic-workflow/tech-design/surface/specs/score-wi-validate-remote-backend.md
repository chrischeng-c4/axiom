---
id: score-wi-validate-remote-backend
summary: Route aw wi validate through configured issue backend and materialize remote CRRR issues.
fill_sections: [logic, cli, test-plan, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# WI Validate Remote Backend

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: score-wi-validate-remote-backend-logic
entry: start
nodes:
  start: { kind: start, label: "aw wi validate <ref>" }
  resolve_backend: { kind: process, label: "resolve --backend/--repo or config backend" }
  fetch_issue: { kind: process, label: "backend.get(input_ref)" }
  found: { kind: decision, label: "issue found?" }
  canonicalize: { kind: process, label: "args.slug = issue.slug canonical id" }
  branch_exists: { kind: decision, label: "issue-<id> exists?" }
  remote_crrr: { kind: decision, label: "remote backend and issue has CRRR phase?" }
  switch_branch: { kind: process, label: "switch to issue-<id>" }
  create_branch: { kind: process, label: "provision issue-<id>" }
  materialize: { kind: process, label: "write remote issue into local cache on issue branch" }
  worktree_validate: { kind: process, label: "run existing LocalBackend worktree validate" }
  legacy_validate: { kind: process, label: "run legacy quality validate through selected backend" }
  not_found: { kind: terminal, label: "not found error" }
  done: { kind: terminal, label: "done or dispatch envelope" }
edges:
  - { from: start, to: resolve_backend, label: "invoke" }
  - { from: resolve_backend, to: fetch_issue, label: "backend ready" }
  - { from: fetch_issue, to: found, label: "lookup result" }
  - { from: found, to: not_found, label: "no" }
  - { from: found, to: canonicalize, label: "yes" }
  - { from: canonicalize, to: branch_exists, label: "numeric id or local slug" }
  - { from: branch_exists, to: switch_branch, label: "yes" }
  - { from: branch_exists, to: remote_crrr, label: "no" }
  - { from: remote_crrr, to: create_branch, label: "yes" }
  - { from: remote_crrr, to: legacy_validate, label: "no" }
  - { from: switch_branch, to: materialize, label: "if local cache missing and selected backend is remote" }
  - { from: create_branch, to: materialize, label: "new branch" }
  - { from: materialize, to: worktree_validate, label: "checkout-hosted issue body" }
  - { from: switch_branch, to: worktree_validate, label: "local cache present" }
  - { from: worktree_validate, to: done, label: "commit + push-through + envelope" }
  - { from: legacy_validate, to: done, label: "quality result" }
---
flowchart TD
    start([aw wi validate ref]) --> resolve_backend[resolve backend]
    resolve_backend --> fetch_issue[backend.get ref]
    fetch_issue --> found{issue found}
    found -- no --> not_found([not found])
    found -- yes --> canonicalize[canonicalize args.slug]
    canonicalize --> branch_exists{issue branch exists}
    branch_exists -- yes --> switch_branch[switch branch]
    branch_exists -- no --> remote_crrr{remote + CRRR phase}
    remote_crrr -- yes --> create_branch[provision issue branch]
    remote_crrr -- no --> legacy_validate[legacy validate selected backend]
    switch_branch --> materialize[materialize remote issue if local cache missing]
    create_branch --> materialize
    materialize --> worktree_validate[local worktree validate]
    switch_branch --> worktree_validate
    worktree_validate --> done([done])
    legacy_validate --> done
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - path: [wi, validate]
    aliases:
      - [issues, validate]
      - [iss, validate]
    args:
      - name: slug
        kind: positional
        required: true
      - name: json
        kind: flag
        long: json
      - name: backend
        kind: option
        long: backend
        values: [local, github, gitlab, jira]
      - name: repo
        kind: option
        long: repo
    behavior:
      backend_resolution: "same as wi list/show"
      canonical_slug: "backend issue slug wins after lookup"
      remote_crrr_materialization: "remote issue with phase label enters issue-<id> branch and local worktree validate"
```

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: score-wi-validate-remote-backend-tests
requirements:
  validate_accepts_backend_flags:
    id: WI-VAL-1
    text: "aw wi validate accepts --backend and --repo like list/show"
    kind: interface
    risk: high
    verify: test
  validate_local_regression:
    id: WI-VAL-2
    text: "local-only validate behavior remains unchanged"
    kind: functional
    risk: high
    verify: test
  validate_remote_materialization:
    id: WI-VAL-3
    text: "remote CRRR issue lookup canonicalizes id and materializes onto issue branch"
    kind: functional
    risk: high
    verify: test
elements:
  score_process_tests:
    kind: test
    type: "rs/cargo-test"
relations:
  - { from: score_process_tests, verifies: validate_accepts_backend_flags }
  - { from: score_process_tests, verifies: validate_local_regression }
  - { from: score_process_tests, verifies: validate_remote_materialization }
---
requirementDiagram
    requirement validate_accepts_backend_flags {
        id: WI-VAL-1
        text: "aw wi validate accepts --backend and --repo like list/show"
        risk: high
        verifymethod: test
    }
    requirement validate_local_regression {
        id: WI-VAL-2
        text: "local-only validate behavior remains unchanged"
        risk: high
        verifymethod: test
    }
    requirement validate_remote_materialization {
        id: WI-VAL-3
        text: "remote CRRR issue lookup canonicalizes id and materializes onto issue branch"
        risk: high
        verifymethod: test
    }
    element score_process_tests {
        type: "rs/cargo-test"
    }
    score_process_tests - verifies -> validate_accepts_backend_flags
    score_process_tests - verifies -> validate_local_regression
    score_process_tests - verifies -> validate_remote_materialization
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/issues.rs
    action: modify
    section: cli
    impl_mode: hand-written
    description: Resolve validate backend from CLI/config, canonicalize remote slugs, and materialize remote CRRR issues before worktree validate.
  - path: projects/agentic-workflow/tests/inplace_mode_test.rs
    action: modify
    section: test-plan
    impl_mode: hand-written
    description: Add local-regression coverage for validate backend flag parsing and issue branch behavior.
  - action: annotate
    section: logic
    impl_mode: hand-written
    description: "Traceability metadata edge for the logic section."

```

# Reviews

### Review 1
**Verdict:** approved

- [logic] The design preserves the existing checkout-local commit path while allowing GitHub/GitLab to be the source lookup and label update backend.
- [cli] Adding `--backend` and `--repo` to validate aligns it with list/show without changing existing positional usage.
- [test-plan] Tests should cover local regression and at least the parse/materialization branch selection without relying on live GitHub in unit tests.
