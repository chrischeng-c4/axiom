---
change: jet-pnpm-parity
group: pnpm-parity
date: 2026-03-11
---

# Requirements

Achieve pnpm feature parity for jet pkg_manager. All features in crates/cclab-jet/src/pkg_manager/.

### Milestone 1: Core Features (P0/P1)
- Frozen lockfile (--frozen-lockfile): hash package.json deps, compare vs lockfile header, fail on drift
- .npmrc support: parse registry, auth tokens, scoped registries, proxy; apply to reqwest
- Optional dependencies: parse optionalDependencies, check os/cpu/libc platform fields
- Overrides/resolutions: package.json overrides to force versions across dep tree
- Update command: jet update [pkg], resolve latest versions, rewrite lockfile
- Alias deps: npm: protocol support (foo: npm:bar@^1.0)

### Milestone 2: Store & Maintenance
- Content-addressable dedup: hash individual files for cross-version dedup
- Store GC: jet store prune — scan lockfiles, remove unreferenced packages
- Lockfile import: migrate from package-lock.json / yarn.lock
- Patching: jet patch <pkg> — edit in patches/ dir, auto-apply on install
- Audit: jet audit — npm advisory API, report CVEs with severity

### Milestone 3: Workspace / Monorepo
- Workspace discovery: package.json workspaces or jet-workspace.yaml
- Workspace protocol: workspace:*, workspace:^, workspace:~ → local symlinks
- Shared lockfile at workspace root
- --filter <pattern> for selective operations
- Recursive commands: jet -r install/run
- Hoisting strategies: shamefully-hoist, public-hoist-pattern
- Catalog: shared version definitions
- Deploy pruning: jet deploy for production node_modules

### Milestone 4: Publishing
- jet publish with workspace protocol resolution
- jet pack — create tarball
- Changeset integration for version bumping
