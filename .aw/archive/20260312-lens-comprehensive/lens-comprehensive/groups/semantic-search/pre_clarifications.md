---
change: lens-comprehensive
group: semantic-search
date: 2026-03-12
status: answered
---

# Pre-Clarifications

### Q1: Search scope
- **Answer**: Implement all 6 search modes: CallHierarchy, Usages, ByTypeSignature, SimilarCode, DocumentationSearch, Implementations.

### Q2: Index persistence
- **Answer**: Persist search index to disk, similar to DiskCache pattern. Warm load on daemon start for fast search.

