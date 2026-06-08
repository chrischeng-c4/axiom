---
change: mamba-p1-bugfix
group: try-except-dict-set-parse
date: 2026-03-28
written_by: artifact_cli
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| tokens-and-indent | crates/mamba/lexer | high | R2 |
| statements | crates/mamba/parser | high | R2 |
| expressions | crates/mamba/parser | medium | R2, R5 |
| ast | crates/mamba/parser | low | R1 |
| exception | crates/mamba/runtime | low | R3 |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| tokens-and-indent | modify | crates/mamba/lexer/tokens-and-indent.md | overview, changes |
| statements | modify | crates/mamba/parser/statements.md | overview, changes |

