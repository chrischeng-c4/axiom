---
number: 883
title: "jet-install: resolver bugs fixed — version conflicts, ||, pre-release, optional deps"
state: open
labels: [bug, crate:jet]
group: "jet-install"
---

# #883 — jet-install: resolver bugs fixed — version conflicts, ||, pre-release, optional deps

## Summary

Fixed 6 resolver bugs discovered while testing `jet install` on real-world React project (react-bench):

### Bugs Fixed

1. **Version conflict resolution** — Flat resolver errored on `semver@^6` vs `semver@^7` (same package, incompatible majors). Fix: hoisted version wins with warning (like pnpm).

2. **`||` OR range syntax** — `^1.0.0 || ^2.0.0-0` failed parsing. Fix: split on `||`, try each alternative, strip `-0` pre-release suffixes.

3. **Pre-release version matching** — `gensync@^1.0.0-beta.2` found no match because `find_best_version` skipped all pre-releases. Fix: fall back to pre-release if no stable version matches.

4. **Space-separated ranges** — `>=0.6.2 <2.0.0` failed (semver crate expects commas). Fix: `normalize_npm_range()` converts space-separated comparators to comma-separated.

5. **npm: alias in transitive deps** — `"string-width-cjs": "npm:string-width@^4.2.0"` in transitive deps wasn't handled. Fix: apply `resolve_alias()` to transitive and peer deps, not just direct deps.

6. **Optional dependencies** — `@rollup/rollup-darwin-arm64` (Vite's native rollup binary) wasn't installed. Fix: parse `optionalDependencies` from registry metadata, resolve with platform filtering via `should_skip_optional()`.

### Performance Improvements (same PR)

7. **Parallel metadata prefetch** — Resolution changed from sequential BFS (200 serial HTTP requests) to level-by-level parallel prefetch. Result: **57s → 6.7s** cold install.

8. **Symlink-based linking** — Changed from recursive hardlink (`walkdir` + `hard_link` per file) to single directory symlink. Result: warm install **3.6s → 0.11s**.

9. **Smart skip marker** — `.jet-marker` in node_modules stores deps hash. If unchanged, skip all I/O. Result: hot install **2.3s → 0.03s**.

## Files Changed

- `crates/cclab-jet/src/pkg_manager/resolver.rs` — parallel BFS, version conflict handling, range parsing, optional deps
- `crates/cclab-jet/src/pkg_manager/registry.rs` — added `optional_dependencies` field
- `crates/cclab-jet/src/pkg_manager/store.rs` — symlink instead of hardlink
- `crates/cclab-jet/src/pkg_manager/mod.rs` — smart skip, concurrency bump 16→50
- `crates/cclab-jet/src/pkg_manager/lockfile.rs` — `nested_in` field support

## Status

All fixes implemented and verified. 34 pkg_manager tests pass. 9/9 PM×bundler combinations work.
