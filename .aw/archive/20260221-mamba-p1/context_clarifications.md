---
change: mamba-p1
date: 2026-02-20
issues: [382, 383, 384, 385, 386, 387, 388, 405, 406, 407, 408, 409, 420, 421, 422, 423, 424]
---

# Context Clarifications

## Q1: Scope
- **Question**: 17 P1 crate:mamba issues found. Tackle all 17 or a subset?
- **Answer**: All 17 issues (#382-#388, #405-#409, #420-#424)
- **Rationale**: User wants to address the full P1 backlog for mamba in one change.

## Q2: Git Workflow
- **Question**: Which git workflow to use?
- **Answer**: in_place — work on current sdd-and-mamba branch
- **Rationale**: Already on a feature branch, no need for extra branching.

## Q3: Affected Modules
- **Question**: Which crates or paths will this change affect?
- **Answer**: crates/cclab-mamba — runtime features (isinstance, super, decorators, context managers, tuple/set, for-else, f-strings), type/protocol features (bytes, descriptors, metaclasses, reflection, unpacking), stdlib/builtins (map/filter/any/all, modules, assert/del, time, os.path)
- **Rationale**: All 17 issues target the mamba compiler crate exclusively.

## Q4: Implementation Approach
- **Question**: How should the 17 issues be ordered and implemented?
- **Answer**: Follow DAG topological order. No inter-dependencies detected among issues, so implement sequentially: runtime (#382-#388) → types/protocols (#405-#409) → stdlib/builtins (#420-#424). Each issue = codegen + runtime support.
- **Rationale**: Logical grouping by feature category for coherent implementation.

## Dependency Graph

| Order | Issue | Depends On |
|-------|-------|------------|
| 1 | #382 — feat(mamba): isinstance/issubclass and type narrowing | — |
| 2 | #383 — feat(mamba): super() runtime implementation | — |
| 3 | #384 — feat(mamba): property, classmethod, staticmethod decorators | — |
| 4 | #385 — feat(mamba): context manager protocol (with statement __enter__/__exit__) | — |
| 5 | #386 — feat(mamba): tuple methods, set type, and set operations | — |
| 6 | #387 — feat(mamba): for/while else clause | — |
| 7 | #388 — feat(mamba): f-string format specifiers (f"{x:.2f}", f"{x:>10}") | — |
| 8 | #405 — mamba: bytes/bytearray type and binary data operations | — |
| 9 | #406 — mamba: descriptor protocol (__get__/__set__/__delete__) | — |
| 10 | #407 — mamba: metaclasses and abc (Abstract Base Classes) | — |
| 11 | #408 — mamba: reflection builtins (hasattr/getattr/setattr/delattr/callable) | — |
| 12 | #409 — mamba: starred unpacking codegen (a, *b, c = iterable) | — |
| 13 | #420 — mamba: missing builtins — map, filter, any, all, round, divmod, format | — |
| 14 | #421 — mamba: module/package system (__init__.py, sys.path, relative imports) | — |
| 15 | #422 — mamba: assert and del statement codegen | — |
| 16 | #423 — mamba: time module | — |
| 17 | #424 — mamba: os.path and extended os module | — |

```mermaid
graph LR
    382["#382 feat(mamba): isinstance/issubclass and type narrowing"]
    383["#383 feat(mamba): super() runtime implementation"]
    384["#384 feat(mamba): property, classmethod, staticmethod decorators"]
    385["#385 feat(mamba): context manager protocol (with statement __enter__/__exit__)"]
    386["#386 feat(mamba): tuple methods, set type, and set operations"]
    387["#387 feat(mamba): for/while else clause"]
    388["#388 feat(mamba): f-string format specifiers (f"{x:.2f}", f"{x:>10}")"]
    405["#405 mamba: bytes/bytearray type and binary data operations"]
    406["#406 mamba: descriptor protocol (__get__/__set__/__delete__)"]
    407["#407 mamba: metaclasses and abc (Abstract Base Classes)"]
    408["#408 mamba: reflection builtins (hasattr/getattr/setattr/delattr/callable)"]
    409["#409 mamba: starred unpacking codegen (a, *b, c = iterable)"]
    420["#420 mamba: missing builtins — map, filter, any, all, round, divmod, format"]
    421["#421 mamba: module/package system (__init__.py, sys.path, relative imports)"]
    422["#422 mamba: assert and del statement codegen"]
    423["#423 mamba: time module"]
    424["#424 mamba: os.path and extended os module"]
```

