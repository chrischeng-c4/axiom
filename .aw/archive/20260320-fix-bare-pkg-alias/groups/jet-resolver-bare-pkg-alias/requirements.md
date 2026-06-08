---
change: fix-bare-pkg-alias
group: jet-resolver-bare-pkg-alias
date: 2026-03-20
---

# Requirements

Extend `crates/cclab-jet/src/pkg_manager/resolver.rs` to handle bare package names as implicit npm alias version specs. In package.json / package-lock.json, a version field like `"@storybook/expect": "storybook-jest"` is a valid npm pattern meaning 'install storybook-jest instead' (equivalent to `npm:storybook-jest@latest`). Jet's current resolver only handles the explicit `npm:pkg@version` form via `resolve_alias()`; it errors with 'Failed to parse version range' when encountering a bare name. The fix must: (1) detect when a version spec fails semver parsing AND is a syntactically valid npm package name, (2) treat it as `npm:{name}@latest`, (3) delegate to the existing alias resolution path. This change is additive on top of the resolver improvements already in #883 (parallel BFS, conflict resolution, `||` ranges, pre-release, space-separated ranges, transitive npm aliases, optional deps).
