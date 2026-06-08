---
change: sdd-workflow-cleanup
group: config-unification
date: 2026-03-17
---

# Requirements

Restructure `config.toml` so each workflow step (e.g., `restructure_input`, `create_change_spec`) has its own section containing both `agents` and `label`. Introduce `label_prefix` in `[workflow]` to avoid repetition. Update `SddConfig` model in `crates/cclab-sdd/src/models/change.rs` and update agent resolution and migration logic.
