---
change: jet-workspace-protocol
group: jet-workspace-protocol
date: 2026-03-24
---

# Requirements

Extend `cclab-jet`'s package manager to support pnpm-style workspace monorepos. Two pieces of work: (1) Workspace discovery — extend `WorkspaceManager::load_config()` in `src/pkg_manager/workspace.rs` to detect and parse `pnpm-workspace.yaml` (pnpm's native format, containing a `packages:` glob array) alongside the existing `jet-workspace.yaml` and `package.json.workspaces` sources. (2) Workspace install — integrate `WorkspaceManager` into `PackageManager::install_with_options()` in `src/pkg_manager/mod.rs`: detect workspace mode at install time, identify which direct deps carry a `workspace:*` (or `workspace:^`, `workspace:~`) protocol spec, skip registry resolution and tarball download for those packages, and instead create a symlink from `node_modules/<pkg-name>` pointing to the workspace package's local directory. The existing `WorkspaceManager::resolve_workspace_protocol()` and `is_workspace_protocol()` utilities are already present and can be reused. Acceptance criteria: a project with `pnpm-workspace.yaml` and a package using `workspace:*` in its deps causes `jet install` to create a symlink in `node_modules/` rather than fetching from the npm registry.
