---
change: lens-dissolution
group: lens-dissolution
date: 2026-03-25
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should #1087 (dissolve lens/) be done FIRST or LAST?
- **Answer**: FIRST. Dissolve lens/ as the foundation so all subsequent features (#944, #946, #949) land directly in the new top-level structure.

### Q2: General
- **Question**: MCP tool exposure for #946 and #949?
- **Answer**: No MCP at all. Remove MCP entirely. #946 (agent context builder) and #949 (agent-optimized output) are CLI-only features exposed via cclab sdd CLI. Existing MCP tool interfaces in lens should also be removed as part of the dissolution.

### Q3: General
- **Question**: Implementation order?
- **Answer**: 1) #1087 dissolve lens/ into SDD top-level, 2) #944 wire cross-file type propagation, 3) #946 agent context builder (CLI-only), 4) #949 agent-optimized output (CLI-only)

