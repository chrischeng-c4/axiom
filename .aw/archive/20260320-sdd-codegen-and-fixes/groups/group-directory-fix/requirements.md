---
change: sdd-codegen-and-fixes
group: group-directory-fix
date: 2026-03-20
---

# Requirements

Fix the file placement bug in multi-group changes where payloads/, prompts/ (spec-fill + implementation), and specs/ are written to the change root instead of under groups/{group}/. Expected structure: cclab/changes/{id}/groups/{group-id}/payloads/, groups/{group-id}/prompts/, groups/{group-id}/specs/ — mirroring the existing correct placement of requirements.md, pre_clarifications.md, post_clarifications.md, reference_context.md. Affected phases: (1) create_change_spec — currently writes specs and fill prompts to root; must write to groups/{group-id}/specs/ and groups/{group-id}/prompts/. (2) create_change_implementation — currently writes impl prompts to root; must write to groups/{group-id}/prompts/. (3) Mainthread payload writes — currently write to root payloads/; must route to groups/{group-id}/payloads/. The merge phase reads from changes/{id}/specs/ — update the merge reader to look in groups/{group-id}/specs/ per group, or establish a symlink/aggregation strategy. Ensure backward compatibility with existing changes that already have root-level specs/ (detect and handle both layouts).
