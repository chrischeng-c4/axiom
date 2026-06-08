---
change: jet-remaining-issues
group: jet-install
date: 2026-03-19
---

# Requirements

Complete and verify the resolver bug fixes from #883 (version conflict resolution, || OR range syntax, pre-release version matching, space-separated ranges, npm: alias in transitive deps, optional dependencies — 34 pkg_manager tests pass). Then implement cold install performance optimizations from #881: (1) disk metadata cache at ~/.jet-store/.metadata/{name}.json with configurable TTL (default 5 min) so repeated installs skip HTTP, (2) HTTP/2 connection reuse via reqwest adaptive window for registry connections, (3) speculative prefetch of common transitive deps during BFS level resolution, (4) pipeline tarball downloads for already-resolved packages while remaining deps are still resolving. Target: cold install ≤ 3.0s. Additionally, remove the nx CLI dependency: jet must parse project.json directly in workspace discovery without spawning the nx binary. Affected subsystems: crates/cclab-jet/src/pkg_manager/{resolver,registry,store,lockfile,mod}.rs and any workspace-config reading code.
