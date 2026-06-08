---
id: project-config-and-prism-index
type: proposal
version: 1
created_at: 2026-02-10T06:08:46.090235+00:00
updated_at: 2026-02-10T06:08:46.090235+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add [project] config section with monorepo-aware language modules and relocate Prism index to ~/.cclab/projects/"
history:
  - timestamp: 2026-02-10T06:08:46.090235+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 8
  new_files: 0
affected_specs:
  - id: project-config
    path: specs/project-config.md
    depends: []
  - id: prism-index-storage
    path: specs/prism-index-storage.md
    depends: [project-config]
---

<proposal>

# Change: project-config-and-prism-index

## Summary

Add [project] config section with monorepo-aware language modules and relocate Prism index to ~/.cclab/projects/

## Why

The Genesis task generator currently detects project language by checking for files like Cargo.toml or pyproject.toml at the project root. This fails for monorepos (e.g., infohub has api/ in Python and frontend/ in TypeScript but no pyproject.toml at root) and produces incorrect file paths in generated tasks (e.g., .rs extensions for a Python project).

Additionally, Prism's code index is currently held in-memory with no persistent storage path. For monorepo support, each language module may need its own index, and indexes should persist across server restarts to avoid re-indexing delays.

This change introduces a [project] section in cclab/config.toml with per-directory language module mappings ([[project.modules]]), making both Genesis task generator and Prism able to read structured project metadata from a single source of truth. It also establishes ~/.cclab/projects/{path_hash}/prism/ as the persistent index location.

## What Changes

- Add ProjectConfig struct with modules field (path + language mapping) to GenesisConfig in models/change.rs
- Add [[project.modules]] TOML array support: path, language, optional framework field
- Update cclab init to auto-detect project languages and populate [project] section on fresh install
- Add config migration (migrate.rs) for existing projects to inject [project] section
- Update task_generator.rs to read config.project.modules instead of detect_project_language()
- Establish ~/.cclab/projects/{path_hash}/prism/ directory structure for persistent Prism index storage
- Update config.toml template with [project] section and documentation comments

## Impact

- **Scope**: minor
- **Affected Files**: ~8
- **New Files**: ~0
- Affected specs:
  - `project-config` (no dependencies)
  - `prism-index-storage` → depends on: `project-config`
- Affected code: `crates/cclab-genesis/src/models/change.rs`, `crates/cclab-genesis/src/cli/init.rs`, `crates/cclab-genesis/src/cli/migrate.rs`, `crates/cclab-genesis/src/services/task_generator.rs`, `crates/cclab-genesis/templates/config.toml`, `crates/cclab-prism/src/server/handler.rs`

</proposal>
