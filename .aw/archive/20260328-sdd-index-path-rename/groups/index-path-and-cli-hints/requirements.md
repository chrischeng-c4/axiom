---
change: sdd-index-path-rename
group: index-path-and-cli-hints
date: 2026-03-26
---

# Requirements

1. Rename `.cclab/lens/` to `cclab/.index/` across the codebase:
   - Update `storage.rs` path constants (lens_dir, daemon_pid_path, daemon_sock_path, cache_dir)
   - Update `daemon.rs` references in both cclab-sdd-cli and cclab-sdd/src/server/
   - Update `.gitignore` (replace `.cclab/lens/` with `cclab/.index/`)
   - Remove old `.cclab/lens/` directory from disk
2. Add code intelligence CLI hints to the implementation prompt in `create_change_impl.rs`:
   - In `build_implement_code_prompt()`, add a section listing available CLI commands (symbols, hover, references, impact, context)
   - Only add hints when executor is mainthread (Claude has bash access)
