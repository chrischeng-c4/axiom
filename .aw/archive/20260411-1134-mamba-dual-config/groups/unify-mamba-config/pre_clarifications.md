---
change: 1134-mamba-dual-config
group: unify-mamba-config
date: 2026-04-10
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Which format should become canonical?
- **Answer**: The richer format from config/schema.rs — with [project] table, CrateEntry enum (Version or Config), CrateConfig with path/version/expose/module, and backward-compatible flat format support.

### Q2: General
- **Question**: Should backward compatibility with the flat format be maintained?
- **Answer**: Yes. The flat format (entry_point at root, crates as HashMap<String, String>) must remain valid via the CrateEntry::Version variant and top-level expose HashMap.

