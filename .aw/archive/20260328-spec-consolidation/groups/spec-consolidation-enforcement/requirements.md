---
change: spec-consolidation
group: spec-consolidation-enforcement
date: 2026-03-23
---

# Requirements

Break the spec-scatter feedback loop in cclab-sdd across three coordinated layers. (1) Structure: define canonical inner-directory rules in `cclab/specs/crates/cclab-sdd/logic/spec-structure.md` (interfaces/{type}/, domain subdirs, no loose root files, one concept per file); add `cclab sdd scaffold-spec {folder}` CLI command that creates the canonical subdirectory skeleton for a crate; add `cclab sdd validate-spec-structure` command that lints any spec root against the rules. (2) Lifecycle enforcement: inject a spec directory tree listing (ASCII tree of cclab/specs/{crate}/) into the reference_context Create prompt so the agent sees existing structure before planning; add prompt guidance instructing the agent to prefer `action: modify` for existing spec files and only `action: create` for genuinely new subsystems; add `main_spec_ref` path validation in spec_plan phase — reject paths that write directly to spec root (no subfolder); add merge-time validation that rejects root-level main_spec_ref and logs create-vs-overwrite for audit. (3) Dead code removal: remove `merge_strategy` (enum MergeStrategy and all variants) from frontmatter.rs, spec_service.rs, common_change_spec.rs, strip_change_spec_fields, and the spec files change-spec.md, change-merge.md, artifact-tools.md — actual merge behavior is always replace (write to path, create if absent, overwrite if exists). All three layers must land together to prevent partial enforcement gaps.
