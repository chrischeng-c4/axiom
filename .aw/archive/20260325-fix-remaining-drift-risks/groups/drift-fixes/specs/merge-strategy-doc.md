---
id: merge-strategy-doc
type: spec
title: "Change Merge — Logic"
version: 3
files:
  - tools/change_merge/create.rs
  - workflow/merge.rs
  - prompts/review_archive.md
main_spec_ref: crates/cclab-sdd/logic/change-merge.md
merge_strategy: extend
fill_sections: [overview, changes]
---

# Change Merge

## Phase Transition

```yaml
from: ChangeImplementationReviewed (all approved)
to: ChangeArchived
executor: [mainthread]
crr: false  # programmatic merge, no CRR
```

## Merge Logic

`sdd_workflow_create_change_merge` is **fully programmatic** — no agent needed.

```mermaid
flowchart TD
    Start([workflow_create_change_merge]) --> FindSpecs[find specs in changes/{id}/groups/*/specs/]
    FindSpecs --> Empty{specs found?}
    Empty -->|no| ArchiveEmpty[archive with no merge]
    Empty -->|yes| Loop[for each spec file]
    Loop --> ReadFM[read frontmatter: main_spec_ref]
    ReadFM --> UsePath[target = cclab/specs/main_spec_ref]
    UsePath --> Strip[strip change-spec-only frontmatter fields]
    Strip --> Write[write to cclab/specs/{target}]
    Write --> Loop
    Loop --> Done[all merged]
    Done --> Archive[phase → ChangeArchived]
    Archive --> Move[rename changes/{id} → archive/{date}-{id}]
```

## Frontmatter Stripping

Change-spec-only fields removed before writing to main specs:

```yaml
stripped_fields:
  - main_spec_ref      # only used for merge routing
  - merge_strategy     # only used during merge
  - create_complete    # internal marker
  - fill_sections      # internal tracking
  - filled_sections    # internal tracking
```

## Merge Strategy

From spec frontmatter `merge_strategy`:

| Strategy | Behavior |
|----------|----------|
| `new` | Create new file at `cclab/specs/{main_spec_ref}` |
| `update` | Overwrite existing file at `cclab/specs/{main_spec_ref}` |
| `extend` | Append new requirements/scenarios to existing spec at `cclab/specs/{main_spec_ref}` — preserves existing content, adds change-spec sections at the end |

## Archive

After merge:
- Phase set to `ChangeArchived`
- Tool moves change dir to `cclab/archive/{YYYYMMDD}-{change_id}` programmatically via `std::fs::rename`
- Response includes `archive_path` for caller reference

## Side Effects

| Action | STATE.yaml change |
|--------|-------------------|
| Merge all specs | Spec files written to `cclab/specs/` |
| Complete | `phase → ChangeArchived` |
| No specs to merge | `phase → ChangeArchived` (skip merge) |


# Reviews
