---
change: stream-resolver
group: stream-resolver
date: 2026-03-20
---

# Requirements

Rewrite resolver from BFS level-based to stream-based. Each resolved package immediately spawns fetches for its transitive deps via tokio::spawn, without waiting for the current BFS level to complete. Use a shared DashMap for resolved packages and an atomic counter to know when all tasks are done. Preserve version conflict detection and override support.
