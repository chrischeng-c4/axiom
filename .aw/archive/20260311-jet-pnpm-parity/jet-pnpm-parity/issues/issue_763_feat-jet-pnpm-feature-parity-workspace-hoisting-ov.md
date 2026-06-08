---
number: 763
title: "feat(jet): pnpm feature parity — workspace, hoisting, overrides, .npmrc, audit"
state: open
labels: [enhancement, P1, crate:jet]
group: "pnpm-parity"
---

# #763 — feat(jet): pnpm feature parity — workspace, hoisting, overrides, .npmrc, audit

## Goal

Achieve feature parity with pnpm for package management. Jet currently has: install/add/remove, global store with hardlinks, lockfile fast-path, peer deps, bin scripts, lifecycle hooks, shasum verification, parallel I/O.

## Milestone 1: Core Missing Features

| Feature | pnpm equivalent | Priority |
|---------|----------------|----------|
| **Workspace support** | `pnpm-workspace.yaml`, `--filter`, cross-package linking | P0 |
| **Frozen lockfile** | `--frozen-lockfile` for CI determinism | P0 |
| **.npmrc support** | Registry auth, scoped registries, proxy config | P0 |
| **Optional dependencies** | Platform-specific deps (`os`, `cpu`, `libc` fields) | P1 |
| **Overrides / resolutions** | Force specific versions across dep tree | P1 |
| **Update command** | `jet update [pkg]`, interactive mode | P1 |

## Milestone 2: Advanced Features

| Feature | pnpm equivalent | Priority |
|---------|----------------|----------|
| **Content-addressable dedup** | Deduplicate identical files across versions | P1 |
| **Store GC** | `jet store prune` — remove unreferenced packages | P1 |
| **Lockfile import** | Migrate from `package-lock.json` / `yarn.lock` | P2 |
| **Alias deps** | `foo: npm:bar@^1.0` syntax | P2 |
| **Patching** | `jet patch <pkg>` — modify installed dep | P2 |
| **Audit** | `jet audit` — security vulnerability scan via npm advisory API | P2 |

## Milestone 3: Monorepo / Workspace

| Feature | Description |
|---------|-------------|
| **Workspace protocol** | `workspace:*`, `workspace:^` in deps |
| **Filtering** | `--filter <pattern>` for selective operations |
| **Recursive commands** | `jet -r install`, `jet -r run build` |
| **Shared lockfile** | Single `jet-lock.yaml` for entire workspace |
| **Hoisting strategies** | `shamefully-hoist`, `public-hoist-pattern` |
| **Catalog** | Shared version definitions across packages |
| **Deploy pruning** | `jet deploy` — production-only node_modules |

## Milestone 4: Publishing

| Feature | Description |
|---------|-------------|
| **Publish** | `jet publish` with workspace protocol resolution |
| **Changeset integration** | Version bumping, changelog generation |
| **Pack** | `jet pack` — create tarball for publishing |

## Architecture Notes

- Workspace config: `jet-workspace.yaml` (or reuse `package.json.workspaces`)
- .npmrc parser: support `registry`, `//registry:_authToken`, `@scope:registry`
- Store GC: reference-count based, scan all project lockfiles
- Frozen lockfile: compare `package.json` deps hash vs lockfile header

## Acceptance Criteria

- [ ] `jet install` works in monorepo with workspace linking
- [ ] `--frozen-lockfile` fails on any lockfile drift
- [ ] `.npmrc` auth tokens work for private registries
- [ ] `jet audit` reports known CVEs
- [ ] `jet store prune` reclaims disk space
