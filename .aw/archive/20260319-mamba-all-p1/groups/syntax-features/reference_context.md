---
change: mamba-all-p1
group: syntax-features
date: 2026-03-19
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| types/generics-and-protocols.md | types | HIGH | R1, R2, R4 (PEP 695 generics, type param substitution, constraint checking) |
| parser/statements.md | parser | HIGH | R2, R3, R4 (with-statement, decorators, generic syntax parsing) |
| parser/ast.md | parser | HIGH | R2 (PEP 695 type parameter nodes) |
| parser/expressions.md | parser | MEDIUM | R1, R2, R4 (Pratt parser, dict literal parsing, starred expressions) |
| lower/hir-to-mir.md | lower | HIGH | R5, R6, R7 (f-string lowering, starred unpacking, dict unpacking) |
| lower/ast-to-hir.md | lower | MEDIUM | R1, R3 (decorator desugaring, with-statement desugaring) |
| runtime/closure.md | runtime | HIGH | R1, R2, R3 (free variable capture, cell variables, closure creation) |
| resolve/name-resolution.md | resolve | HIGH | R1, R2, R3, R4, R5 (global/nonlocal processing, cell creation, MIR extensions) |
| lexer/tokens-and-indent.md | lexer | HIGH | R4 (string literal lexing with prefix combos and escape sequences) |
| runtime/bytes-ops.md | runtime | HIGH | R1, R3 (immutable bytes, encoding/decoding) |
| runtime/dict-ops.md | runtime | HIGH | R5 (dict merge for unpacking) |

## Notes and Spec Extensions

### Coverage Summary

This reference context maps **11 core specs** across **6 subsystems** (types, parser, lower, runtime, resolve, lexer) to cover all **7 issues** in the syntax-features group:
- **#830** (PEP 695 generics): generics-and-protocols.md, parser/statements.md, parser/ast.md
- **#831** (dict unpacking `{**d}`): parser/expressions.md, lower/hir-to-mir.md, runtime/dict-ops.md
- **#832** (parenthesized `with`): parser/statements.md, lower/ast-to-hir.md
- **#845** (starred expressions): parser/expressions.md, lower/hir-to-mir.md
- **#846** (global/nonlocal): resolve/name-resolution.md, runtime/closure.md
- **#847** (decorators): parser/statements.md, lower/ast-to-hir.md
- **#848** (string literals): lexer/tokens-and-indent.md, runtime/bytes-ops.md

### Required Spec Extensions

The review identified **5 spec gaps** that must be addressed during implementation:

#### 1. **types/generics-and-protocols.md** — TypeVarTuple and ParamSpec (PEP 646/612)
- **Current state**: R1-R4 define PEP 695 `TypeVar` support only
- **Missing**: TypeVarTuple (`*Ts`) and ParamSpec (`**P`) variants for generic functions with variadic type parameters
- **Pre-clarifications flag**: Q1 explicitly marks as MISSING with "Recommend implementing all in this change"
- **Action**: Add R5 (TypeVarTuple support) and R6 (ParamSpec support) to spec before or during implementation

#### 2. **parser/expressions.md** — Dict unpacking syntax `{**expr}` (PEP 448)
- **Current state**: R2 specifies `DictLit` as `{k: v}` with simple key-value pairs; no unpacking markers
- **Missing**: Parser-level specification for `DictUnpack(Box<Expr>)` entry kind in dict literals
- **Pre-clarifications flag**: Q2 confirms "NOT IMPLEMENTED in parser — DictLit stores simple pairs only with no unpacking markers"
- **Action**: Add R5 to parser/expressions.md specifying: (a) new `DictUnpack` entry kind, (b) `**expr` parsing algorithm, (c) interaction with `{k: v}` pairs

#### 3. **lexer/tokens-and-indent.md** — String literal prefix combos and escape sequences
- **Current state**: R4 lists 5 forms (Regular, Triple, F-string, Raw, Byte) with generic "escape processing: Yes/No"
- **Missing**:
  - Prefix combos: `rb`/`br` (raw-bytes), `fr`/`rf` (raw-f-string), uppercase variants
  - Escape sequences: Unicode `\uXXXX`, `\UXXXXXXXX`, `\N{name}`; Hex `\xNN`; Octal `\NNN`
- **Pre-clarifications flag**: Q5 lists all forms as in-scope for #848
- **Action**: Extend R4 to enumerate all prefix combos and define complete escape processing rules

#### 4. **parser/statements.md** — Parenthesized with statement (PEP 617)
- **Current state**: R2 specifies `with expr as name, expr2: body` (comma-separated on one line)
- **Missing**: Parenthesized form `with (cm1 as a, cm2 as b,): body` with distinct parsing requirements (opening `(` recognition, trailing comma support)
- **Pre-clarifications note**: "parser-only change, rest of pipeline reuses existing with-handling"
- **Action**: Extend R2 to explicitly specify parenthesized form parsing, reference PEP 617

#### 5. **lower/ast-to-hir.md** — Starred assignment target lowering
- **Current state**: R1-R5 cover decorators, comprehensions, with-statement, for-else, augmented assignments
- **Missing**: Explicit requirement for starred assignment unpacking lowering (e.g., `a, *b, c = iterable`)
- **Pre-clarifications flag**: Q3 cites ast_to_hir.rs:1098 as implementation site; MIR side (hir-to-mir.md R6) is well-specified but HIR representation is not contractually defined
- **Action**: Add R6 to spec: starred assignment targets lowered to HIR, preserving target sequence for MIR lowering

