---
change: all-mamba-p0
group: match-case
date: 2026-03-18
---

# Requirements

Implement full PEP 634 structural pattern matching: match/case AST nodes, all pattern types (literal, capture, wildcard _, sequence, mapping, class, OR |, AS, guard if), type narrowing in case branches, HIR/MIR lowering to decision tree, and Cranelift IR emission for pattern dispatch.
