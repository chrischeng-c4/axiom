---
id: change-merge-update
type: spec
title: "Change Merge — Logic"
version: 3
files:
  - mcp/tools/change_merge/create.rs
  - workflow/merge.rs
  - prompts/review_archive.md
main_spec_ref: "crates/cclab-sdd/logic/change-merge.md"
merge_strategy: extend
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


## Changes

<!-- type: changes lang: markdown -->

## Changes

### Remove: merge_strategy Dead Code

The `merge_strategy` frontmatter field and its variants (`new`, `update`) are removed. Actual merge behavior is always: write to `cclab/specs/{main_spec_ref}` — create file if absent, overwrite if exists. No variant is needed.

Remove from `stripped_fields`:

```yaml
stripped_fields_remove:
  - merge_strategy
```

Remove the **Merge Strategy** table and its surrounding prose entirely from the spec.

### Update: Merge Logic Flowchart

Replace existing flowchart with validation and audit steps:

```mermaid
flowchart TD
    Start([workflow_create_change_merge]) --> FindSpecs[find specs in changes/{id}/groups/*/specs/]
    FindSpecs --> Empty{specs found?}
    Empty -->|no| ArchiveEmpty[archive with no merge]
    Empty -->|yes| Loop[for each spec file]
    Loop --> ReadFM[read frontmatter: main_spec_ref]
    ReadFM --> Validate{main_spec_ref has subfolder?}
    Validate -->|no| Error[hard error: root-level path rejected]
    Validate -->|yes| UsePath[target = cclab/specs/main_spec_ref]
    UsePath --> Strip[strip change-spec-only frontmatter fields]
    Strip --> Exists{file exists at target?}
    Exists -->|yes| LogOverwrite[audit log: overwrite {target}]
    Exists -->|no| LogCreate[audit log: create {target}]
    LogOverwrite --> Write[write to cclab/specs/{target}]
    LogCreate --> Write
    Write --> Loop
    Loop --> Done[all merged]
    Done --> Archive[phase → ChangeArchived]
    Archive --> Move[rename changes/{id} → archive/{date}-{id}]
```

### Add: Merge-Time Validation

| Check | Rule | Failure Mode |
|-------|------|--------------|
| Path depth | `main_spec_ref` must contain at least one `/` (i.e., reside in a subfolder) | Hard error — merge aborted, no files written |

### Add: Create-vs-Overwrite Audit Logging

| Condition | Audit Log Entry |
|-----------|----------------|
| Target file does not exist | `audit: create cclab/specs/{main_spec_ref}` |
| Target file already exists | `audit: overwrite cclab/specs/{main_spec_ref}` |

Audit entries are appended to the merge response and visible to the caller.