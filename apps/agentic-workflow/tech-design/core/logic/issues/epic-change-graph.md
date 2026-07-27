---
id: aw-wi-epic-change-graph
fill_sections: [overview, logic, cli, e2e-test, changes]
capability_refs:
  - id: work-item-planning
    role: primary
    gap: epic-to-change-atomization
    claim: epic-to-change-atomization
    coverage: full
    rationale: "The issue platform is the durable source of epic ownership, ordering, dependency, duplication, and supersession state."
command_refs:
  - command: aw wi graph
---

# Issue-platform Epic/Change Graph

## Overview
<!-- type: overview lang: markdown -->

### Domain language

- **Epic** is the only aggregate root and the only work item allowed to own children.
- **Change** is an executable leaf. Every open change belongs to exactly one epic.
- **Project graph** is the deterministic read model built from the complete issue-platform inventory for one configured project label.
- **Explicit priority** is a `priority:p0..p3` label on the work item itself. A change without one inherits its epic priority.
- **Supersession** replaces one oversized or obsolete change with sibling changes under the same epic. The obsolete change never becomes a parent.

The issue platform remains the durable source of truth. `aw wi graph` is a pure
projection and validation command: it reads all issues to distinguish missing
from cross-project targets, computes `aw.wi.graph.v1`, and never creates,
updates, closes, labels, or comments on an issue.

### Canonical tracker labels

| Meaning | Canonical label |
|---|---|
| Change owner | `epic:<epic-id>` |
| Dependency | `depends-on:<change-id>` |
| Duplicate recommendation | `duplicate-of:<change-id>` |
| Replacement points to original | `supersedes:<change-id>` |
| Original points to replacement | `superseded-by:<change-id>` |

`parent-epic:<id>`, `parent:<id>`, `Parent Epic:`, `Parent WI:`, generic
`Parent:`, and the existing epic-targeting `related` / `implements` references
are decode-only migration inputs. After a parent prefix, the first `#<digits>`
reference is authoritative and trailing prose or later references are ignored.
When no hash reference exists, the first bare id, slug, or
`owner/repository/<id>` token remains supported; a prefix with no extractable
reference declares no parent. Labels use the same extraction rule. These forms
normalize to the same graph as `epic:<id>` and do not cause tracker writes.

Body dependency text is also decode-only migration input; the canonical
authoring form remains `depends-on:<change-id>`. Compatibility decoding accepts
only declaration-shaped lines whose normalized content begins with `Depends
on`, `Dependency`, `Dependencies`, `Blocked by`, or `Requires`, optionally
followed by a colon, and whose first non-whitespace suffix character is `#`.
Markdown list markers and bold field emphasis are ignored. Requirements,
reproductions, prose, headings, and backtick-delimited syntax examples do not
declare graph edges merely because they contain relation vocabulary and hash
references. In particular, an indented bare prefix is a Markdown continuation,
not a new declaration; indentation is accepted only when it precedes a list
marker. One valid declaration returns every hash reference in encounter order;
the graph's existing normalization then sorts and deduplicates them.
The executable grammar is
`src/agentic_workflow/work_items/dependency_reference_extraction.py`.

## Logic
<!-- type: logic lang: markdown -->

### Aggregate invariants

1. Every open change resolves to exactly one owning epic.
2. Only epics may own children.
3. Parent epics must exist and share the change's configured project label.
4. Every open epic declares exactly one `priority:p0..p3` label.
5. A change priority is explicit when its own label exists; otherwise it is inherited from its resolved epic; otherwise it is `unset`.
6. Re-atomization replacements and their superseded original are changes and siblings under the same epic.
7. Issue ids, child lists, relations, and diagnostics are sorted deterministically before hashing.
8. The digest is SHA-256 over the complete projection with only the digest field cleared. Re-reading unchanged tracker state produces the same bytes and digest.

The normalized graph contains:

- project name and canonical project label;
- epics with state, explicit priority, and child ids;
- changes with state, normalized parent, effective priority plus source,
  dependencies, duplicate recommendation, and bidirectional supersession;
- stable diagnostics containing a code, issue, related target when applicable,
  exact remediation target, and executable `aw wi show <id>` inspection command.
- `action=done` as the valid terminal marker, or `action=blocked` plus a
  top-level executable `next.command` copied from the first stable diagnostic.

Validation fails closed for unowned or multiply-owned open changes,
change-as-parent, missing parent, cross-project parent, invalid priority
cardinality, missing/non-change/cross-project dependency or supersession
targets, and non-sibling supersession.

## CLI
<!-- type: cli lang: yaml -->

```yaml
command: aw wi graph --project <project> [--json] [--repo <owner/name>]
read_set: complete configured issue-platform inventory
write_set: []
success:
  exit: 0
  action: done
  projection: aw.wi.graph.v1
invalid_graph:
  exit: non-zero
  action: blocked
  next: first stable diagnostic's executable inspection command
  stdout: complete aw.wi.graph.v1 projection with valid=false and diagnostics
  stderr: summary directing the agent to emitted remediation targets
```

## E2E Test
<!-- type: e2e-test lang: markdown -->

- A valid fixture proves canonical and legacy parent forms normalize to one graph.
- A child with its own priority proves explicit override; a sibling without one proves epic inheritance.
- Supersession proves the original and replacements remain siblings and the mapping is bidirectional.
- Invalid fixtures prove exact diagnostics for unowned, multiply-owned, missing, cross-project, change-as-parent, and unresolved dependency relations.
- Source invariants prove explicit legacy body dependency declarations remain
  readable while explanatory prose and syntax examples create no relation
  edges.
- Two unchanged reads prove byte-equivalent JSON and digest stability.
- Before/after issue-file snapshots prove the CLI performs no backend writes.

Gate: `cargo test -p agentic-workflow --test wi_epic_change_graph_cli_test -- --nocapture`

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/issues/graph.rs
    action: create
    section: logic
    impl_mode: handwrite
    description: "DDD issue graph aggregate, normalized relations, invariant diagnostics, and stable digest."
  - path: apps/agentic-workflow/src/cli/issues.rs
    action: modify
    section: cli
    impl_mode: codegen
    description: "Expose the read-only aw wi graph command."
  - path: apps/agentic-workflow/src/cli/run.rs
    action: modify
    section: logic
    impl_mode: codegen
    description: "Reuse canonical parent compatibility decoding in epic rollup."
  - path: apps/agentic-workflow/tests/wi_epic_change_graph_cli_test.rs
    action: create
    section: e2e-test
    impl_mode: handwrite
    description: "Compiled CLI behavior, deterministic digest, diagnostics, and no-write evidence."
```
