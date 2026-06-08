---
change: lens-comprehensive
group: semantic-search
date: 2026-03-12
status: clarified
---

# Post-Clarifications

## Questions

### Q1: Index + DiskCache coexistence
- **Question**: Should search index use the same DiskCache or a separate persistence layer?
- **Answer**: Separate persistence under .cclab/lens/search_index/. DiskCache is for analysis results (SemanticModel+Diagnostics), search index has different structure (inverted index).

