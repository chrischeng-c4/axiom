---
change_id: grid-merge-cells
type: gap_codebase_knowledge
created_at: 2026-02-10T03:39:16.853775+00:00
updated_at: 2026-02-10T03:39:16.853775+00:00
---

## Gap Analysis: Codebase vs Knowledge\n\n### Gaps\n\n1. **Command pattern not applied to merge range updates during row/col shifts**\n   - Knowledge documents describe command pattern for all mutations\n   - But insert_rows/delete_rows directly mutate sheet without updating merged_ranges\n   - Fix: Update merged_ranges in the same insert/delete methods, or add a separate adjustment step\n\n2. **Google Sheets merge behavior not documented**\n   - No knowledge doc describes expected Google Sheets merge behavior\n   - Needed: merge toggle, unmerge all, merge keeping values, horizontal merge, vertical merge\n\n### Aligned\n\n- Event emission pattern: MergeCellsEvent/UnmergeCellsEvent follow the established pattern\n- WASM bridge layer: merge functions follow same pattern as other bridge functions\n- Rendering pipeline: merge-aware rendering already integrated"