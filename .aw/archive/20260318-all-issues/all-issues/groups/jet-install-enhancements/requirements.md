---
change: all-issues
group: jet-install-enhancements
date: 2026-03-17
---

# Requirements

Optimize jet install performance and resolver correctness:
- Target cold install time <= 3.0s (beating pnpm).
- Implement persistent disk metadata cache for registry responses.
- Enable HTTP/2 multiplexing and speculative prefetching of transitive dependencies.
- Fix resolver bugs: version conflict resolution (hoisted version wins), '||' range syntax, pre-release version matching, space-separated ranges, and 'npm:' alias handling.
- Implement platform-aware optional dependency resolution.
- Performance: level-by-level parallel metadata prefetching, symlink-based linking for node_modules, and smart skip marker based on dependencies hash.
