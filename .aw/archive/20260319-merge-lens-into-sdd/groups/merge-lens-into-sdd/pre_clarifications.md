---
change: merge-lens-into-sdd
group: merge-lens-into-sdd
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: Flat mirror — place under crates/cclab-sdd/src/lens/ mirroring current cclab-lens/src/ structure. Preserves existing module paths.

### Q2: General
- **Answer**: No MCP tools for lens. Lens is CLI-only. No MCP tool migration needed.

### Q3: General
- **Answer**: Remove cclab-lens crate immediately in this change. No transition wrapper.

### Q4: General
- **Answer**: N/A — cclab-lens has no PyO3 bindings.

### Q5: General
- **Answer**: Keep SpecIR at top-level src/spec_ir/ (already exists in cclab-sdd). Both SDD pipeline and lens code reference it. After merge, lens code imports from the existing spec_ir module.

