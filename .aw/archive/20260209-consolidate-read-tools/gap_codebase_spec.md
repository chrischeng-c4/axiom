---
change_id: consolidate-read-tools
type: gap_codebase_spec
created_at: 2026-02-09T07:10:06.052147+00:00
updated_at: 2026-02-09T07:10:06.052147+00:00
---

# Gap Analysis: Codebase vs Spec\n\n## Code without matching spec\n\n| Code | Severity | Note |\n|------|----------|------|\n| `read.rs` — genesis_read_file scope prefix routing | medium | No spec describes the extended file parameter syntax (knowledge:, main_spec:, list:, requirements) |\n| `file_service.rs` — well-known file dispatch table | low | Implementation detail, no spec needed |\n| `knowledge.rs` / `main_spec.rs` — standalone tools | low | Currently implemented but will be removed by this change |\n\n## Spec without matching implementation\n\n| Spec | Severity | Note |\n|------|----------|------|\n| run-change spec — references genesis_list_main_specs as separate tool in Action table | medium | Spec describes 6 separate tools that will be consolidated; spec update needed post-implementation |\n| run-change spec — 37 phases (outdated, now 48 with gap analysis) | low | Pre-existing gap, not caused by this change |\n\n## Summary\n\nMain gap: the run-change spec references tool names that will change. The spec should be updated after implementation to reflect the consolidated genesis_read_file API. No blocking gaps for implementation."