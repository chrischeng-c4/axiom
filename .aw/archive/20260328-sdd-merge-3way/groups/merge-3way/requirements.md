---
change: sdd-merge-3way
group: merge-3way
date: 2026-03-27
---

# Requirements

Implement 3-way merge for spec files during change-merge to support parallel changes targeting the same main spec. (1) At change-spec creation time, when a spec references a main_spec_ref that already exists in cclab/specs/, snapshot the current main spec content as a base file (e.g. groups/{group}/specs/{name}.base.md). (2) At merge time in create_change_merge.rs, when a .base.md snapshot exists: use git merge-file to perform 3-way merge (current main spec as 'ours', base snapshot as 'base', change spec as 'theirs'). If clean merge, write the merged result. If conflicts, abort merge for that spec and report conflict details. (3) When no .base.md exists (new spec or legacy), fall back to current overwrite behavior. (4) Strip .base.md files from archive (they are merge artifacts, not spec content).
