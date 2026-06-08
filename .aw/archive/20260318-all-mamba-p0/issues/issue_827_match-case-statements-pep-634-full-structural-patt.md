---
number: 827
title: "Match/case statements (PEP 634) — full structural pattern matching"
state: open
labels: [enhancement, P0, crate:mamba]
group: "match-case"
---

# #827 — Match/case statements (PEP 634) — full structural pattern matching

## Summary

Implement full PEP 634 structural pattern matching (`match`/`case` statements), introduced in Python 3.10.

Currently the parser has only a skeleton — no type checking, lowering, or codegen support.

## Scope

- **Parser**: Complete `match`/`case` AST nodes with all pattern types
- **Patterns**: literal, capture, wildcard (`_`), sequence, mapping, class, OR (`|`), AS, guard (`if`)
- **Type checker**: Narrowing within `case` branches
- **HIR/MIR lowering**: Decision tree compilation
- **Codegen**: Cranelift IR emission for pattern dispatch

## Why P0

`match`/`case` is a core Python 3.10+ language feature. Without it, Mamba cannot run a large portion of modern Python code. Libraries like `dataclasses`, `pydantic`, and CLI tools increasingly use pattern matching.

## References

- [PEP 634 – Structural Pattern Matching](https://peps.python.org/pep-0634/)
- [PEP 635 – Motivation and Rationale](https://peps.python.org/pep-0635/)
- [PEP 636 – Tutorial](https://peps.python.org/pep-0636/)
