---
change: lens-full-upgrade
group: disk-cache
date: 2026-03-13
---

# Requirements

Persistent AST index cache: serialize SemanticModel + diagnostics to .cclab/lens/cache/ using bincode. Manifest for staleness check (content hash). Warm load on daemon startup. Background write after check_file. Flush on shutdown. Plan exists at graceful-juggling-locket.md.
