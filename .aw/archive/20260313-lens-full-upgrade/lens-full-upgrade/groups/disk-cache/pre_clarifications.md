---
change: lens-full-upgrade
group: disk-cache
date: 2026-03-13
status: answered
---

# Pre-Clarifications

### Q1: Cache eviction
- **Answer**: Hash mismatch only — 僅在檔案內容變更時丟棄舊 cache，不做 LRU/TTL eviction

