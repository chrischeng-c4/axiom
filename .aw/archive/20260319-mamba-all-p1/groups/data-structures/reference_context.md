---
change: mamba-all-p1
group: data-structures
date: 2026-03-19
written_by: claude-revision
review_verdict: ADDRESSED
---

# Reference Context

| Spec | Relevance | Key Requirements |
|------|-----------|------------------|
| `cclab-mamba/runtime/list-ops.md` | high | Directly covers list_ops.rs (Q1 answer lines 132-176): R4 (Subscript and Slicing Operations — mb_list_slice for list[start:stop:step], mb_list_getitem, negative index handling, out-of-range IndexError), R1 (Mutation Methods — append, extend, insert, remove, pop, clear, reverse, sort for regression coverage), R2 (Query Methods — count, index). Primary spec for Q1 list slicing. Implementers need this spec contract to validate slice implementation against conformance tests. |
| `cclab-mamba/runtime/dict-ops.md` | high | Directly covers dict_ops.rs (Q2 answer lines 320-352): R5 (Dict Merge for Unpacking — mb_dict_merge(target, source) merges key-value pairs with source overriding target on conflict, implementing {**d1, **d2} semantics for the \| operator), R3 (Mutation Methods — update, pop, popitem, clear, setdefault for regression coverage), R4 (Subscript Operations — dict getitem, setitem, delitem). Primary runtime spec for Q2 dict merge via \| operator. Implementers need this spec contract for merge semantics validation. |
| `cclab-mamba/runtime/string-ops.md` | high | Directly covers string_ops.rs (Q3 answer, dispatch table lines 848-901): 39 implemented methods. R1 covers 23 core string methods (upper, lower, capitalize, title, swapcase, strip, lstrip, rstrip, find, rfind, count, startswith, endswith, replace, split, splitlines, join, center, ljust, rjust, zfill, maketrans, translate, expandtabs), R2 covers 6 predicate methods (isdigit, isalpha, isalnum, isspace, isupper, islower, istitle, isidentifier, isprintable, isnumeric, isdecimal, casefold). Q3 notes 9 missing methods including maketrans, translate, expandtabs which ARE in spec but missing from implementation — revealing spec-vs-implementation gaps. Primary spec for Q3 conformance test failures. |
| `cclab-mamba/all-mamba-p0.md` | high | R5 (Builtins Conformance) directly addresses Python 3.12 validation for sequence, numeric, and string operations: R5.1 (migrate 108 builtin tests to CPython 3.12 conformance), R5.2 (achieve 100% compatibility for numeric, sequence, string, and type functions). Directly governs the conformance test failures cited in Q3 for list, dict, and string operations. Correctly includes R5 only (NOT R3 BigInt which is unrelated to list slicing, dict merge, or string methods). |
| `cclab-mamba/parser/ast.md` | medium | Covers ast.rs (Q1 at lines 298-303 for slice AST nodes): R1 (Expr enum including Slice variant — the AST node produced for list[start:stop:step] with 3-arg start/stop/step structure). The AST contract for slice representation is the interface between parser (expr.rs:350-369) which produces the Slice node and runtime (list_ops.rs:132-176) which consumes it. Without this spec, the AST representation for Q1 is unspecified. |
| `cclab-mamba/runtime/builtins.md` | medium | Covers builtins.rs (Q2 at lines 576-591 for mb_bitor which routes Dict\|Dict to mb_dict_merge): R5 (Enhanced P1 Builtins — operator dispatch, including mb_bitor routing Dict\|Dict to dict_ops::mb_dict_merge()), R6 (Symbol Registration — mb_bitor registered with correct MirExtern declaration in symbols.rs:200). The \| operator dispatch for Dict types is part of the operator overloading infrastructure. Without this spec, the type-level dispatch for Q2 is unspecified. |
| `cclab-mamba/types/type-checker.md` | medium | Covers check_expr.rs (Q2 at lines 414-421 for Dict\|Dict type checking: returns Ty::Any for non-integer \| operands): R2 (Expression Type Inference — binary op type checking, including \| operator handling for Dict types which returns Ty::Any to allow PEP 584 runtime semantics). The type checker behavior for \| on Dict operands must be understood by implementers working on Q2 conformance. |
| `cclab-mamba/README.md` | low | Authoritative spec index for cclab-mamba mapping source files to specifications. Shows list_ops.rs→runtime/list-ops.md, dict_ops.rs→runtime/dict-ops.md, string_ops.rs→runtime/string-ops.md mapping. Value is spec discovery only, not implementable requirements — consistent with class-features review rating of README as 'low'. Primary specs are directly included above. |

## Coverage Analysis

**Pre-Clarifications Scope Coverage:**

| Question | Area | Files | Covered By | Status |
|----------|------|-------|-----------|--------|
| Q1: List Slicing | AST, parser, and list slice operations (mb_list_slice_full with positive/negative steps, clamping) | ast.rs:298-303, expr.rs:350-369, list_ops.rs:132-176 | parser/ast.md R1 (Slice variant), runtime/list-ops.md R4 (Subscript and Slicing — mb_list_slice_full, negative indices, out-of-range) | ✅ Fully covered: Both parser AST contract and list slicing implementation specs present |
| Q2: Dict Merge via \| Operator | Type checking, builtin operator dispatch, and dict merge semantics (mb_dict_merge source overrides target) | check_expr.rs:414-421, builtins.rs:576-591, dict_ops.rs:320-352, symbols.rs:200 | types/type-checker.md R2 (binary op type inference), runtime/builtins.md R5-R6 (mb_bitor dispatch and symbol registration), runtime/dict-ops.md R5 (mb_dict_merge for {**d1, **d2} unpacking) | ✅ Fully covered: Type checking, operator dispatch, and runtime semantics all specified |
| Q3: String Methods | 39 implemented methods with dispatch table, 9 missing from CPython 47+ (maketrans, translate, expandtabs, etc.), conformance test focus | string_ops.rs:848-901, all 39 methods | runtime/string-ops.md R1-R2 (core and predicate string methods, including maketrans/translate/expandtabs in spec but missing from implementation) | ✅ Fully covered: All string methods and predicates specified; spec-vs-implementation gap identified (expandtabs in spec but not implemented) |

**Gap Analysis:**

| Gap | Severity | Details | Recommendation |
|-----|----------|---------|-----------------|
| Missing HIGH-relevance primary specs for Q1/Q2/Q3 | CRITICAL (RESOLVED) | Original context was missing runtime/list-ops.md, runtime/dict-ops.md, runtime/string-ops.md which directly implement Q1/Q2/Q3. pattern-matching.md was included at LOW despite being entirely irrelevant (no connection to list slicing, dict merge, or string methods). | RESOLVED: Added three HIGH-relevance primary specs (list-ops.md, dict-ops.md, string-ops.md). Removed pattern-matching.md entirely. |
| Missing compiler infrastructure specs for Q1/Q2 | HIGH (RESOLVED) | Q1 needs parser AST contract (ast.rs:298-303 in ast.md); Q2 needs type-checker contract (check_expr.rs:414-421) and builtin dispatch specs (builtins.rs:576-591). These were absent. | RESOLVED: Added parser/ast.md at MEDIUM for Q1 AST contract. Added runtime/builtins.md and types/type-checker.md at MEDIUM for Q2 infrastructure. |
| all-mamba-p0.md lists wrong key requirement (R3 BigInt) | MEDIUM (RESOLVED) | all-mamba-p0.md R3 (BigInt Fallback for integer overflow) has zero connection to list slicing (Q1), dict merge (Q2), or string method conformance (Q3). Correct key requirement is R5 (Builtins Conformance) which governs conformance test failures. | RESOLVED: Changed key requirement from 'R3: BigInt Fallback for integer overflow' to 'R5: Builtins Conformance (sequence, string, numeric validation against CPython 3.12)'. |
| README.md rated MEDIUM but has no implementable requirements | LOW (RESOLVED) | README.md is a spec-index table with no R1-Rn requirements or acceptance criteria. class-features review for same change rated README.md LOW with note 'Value is spec discovery only, not implementable requirements' — MEDIUM is inconsistent. | RESOLVED: Downgraded README.md from MEDIUM to LOW, consistent with class-features review. |
| pattern-matching.md included at LOW despite zero relevance | LOW (RESOLVED) | pattern-matching.md covers PEP 634 control-flow dispatch (parser patterns, HIR nodes, MIR decision trees, Cranelift codegen). None of its R1-R8 requirements touch list slicing, dict merge via \| operator, or string method conformance. Inclusion inverts correct priority. | RESOLVED: Removed pattern-matching.md entirely from this group's context. |

## Verification Summary

**Pre-clarifications Coverage:**
- ✅ Q1 (List slicing): Covered by runtime/list-ops.md R4 (mb_list_slice_full, negative indices, out-of-range) + parser/ast.md R1 (Slice variant) — Complete AST and runtime coverage
- ✅ Q2 (Dict merge via \|): Covered by runtime/dict-ops.md R5 (mb_dict_merge), runtime/builtins.md R5-R6 (mb_bitor dispatch), types/type-checker.md R2 (binary op type inference) — Complete type inference, dispatch, and runtime coverage
- ✅ Q3 (String methods): Covered by runtime/string-ops.md R1-R2 (23 core methods and 6 predicate methods, includes spec for maketrans/translate/expandtabs) — All 39 methods specified; spec gap identified (expandtabs in spec but not implemented)
- ✅ All affected crates/source files covered: ast.rs, expr.rs, list_ops.rs, check_expr.rs, builtins.rs, dict_ops.rs, symbols.rs, string_ops.rs

**Relevance Scores:**
- ✅ runtime/list-ops.md rated 'high' — directly implements Q1 list slicing (mb_list_slice_full), primary spec for Q1
- ✅ runtime/dict-ops.md rated 'high' — directly implements Q2 dict merge (mb_dict_merge), primary spec for Q2
- ✅ runtime/string-ops.md rated 'high' — directly implements Q3 string methods (39 methods), primary spec for Q3
- ✅ all-mamba-p0.md rated 'high' — R5 (Builtins Conformance) directly governs conformance test failures cited in Q3
- ✅ parser/ast.md rated 'medium' — AST contract (Slice variant) needed by Q1 parser implementation
- ✅ runtime/builtins.md rated 'medium' — operator dispatch (mb_bitor) needed by Q2 type-level routing
- ✅ types/type-checker.md rated 'medium' — binary op type inference needed by Q2 type checking
- ✅ README.md rated 'low' — spec index only, no implementable requirements

**Key Requirements Accuracy:**
- ✅ all-mamba-p0.md correctly lists R5 only (NOT R3 BigInt) — R5 Builtins Conformance directly addresses sequence, string, numeric validation against CPython 3.12
- ✅ runtime/list-ops.md lists correct key requirements: R4 (Subscript/Slicing), R1 (Mutation), R2 (Query)
- ✅ runtime/dict-ops.md lists correct key requirements: R5 (Dict Merge), R3 (Mutation), R4 (Subscript)
- ✅ runtime/string-ops.md lists correct key requirements: R1 (Core methods, including maketrans/translate/expandtabs), R2 (Predicate methods)
- ✅ parser/ast.md correctly references Slice variant in Expr enum
- ✅ runtime/builtins.md correctly references mb_bitor and symbol registration
- ✅ types/type-checker.md correctly references binary op type inference for \| operator

## Review Feedback Resolutions

### [HIGH] runtime/list-ops.md is absent
- **Status**: ✅ RESOLVED
- **Action**: Added runtime/list-ops.md at HIGH relevance. Key requirements: R4 (Subscript and Slicing Operations — mb_list_slice, negative indices, out-of-range IndexError), R1 (Mutation Methods), R2 (Query Methods).
- **Details**: This spec directly covers list_ops.rs:132-176 (mb_list_slice_full implementation). R4 specifies exactly the functions in scope for Q1: mb_list_slice with full start/stop/step support, negative index handling, and out-of-range errors. Without it, implementers have no spec contract for slice implementation validation.

### [HIGH] runtime/dict-ops.md is absent
- **Status**: ✅ RESOLVED
- **Action**: Added runtime/dict-ops.md at HIGH relevance. Key requirements: R5 (Dict Merge for Unpacking — mb_dict_merge with source override semantics), R3 (Mutation Methods), R4 (Subscript Operations).
- **Details**: This spec directly covers dict_ops.rs:320-352 (mb_dict_merge implementation). R5 specifies mb_dict_merge(target, source) which implements {**d1, **d2} semantics exactly as needed for Q2's \| operator routing. Without it, implementers have no spec contract for merge semantics validation.

### [HIGH] runtime/string-ops.md is absent
- **Status**: ✅ RESOLVED
- **Action**: Added runtime/string-ops.md at HIGH relevance. Key requirements: R1 (Core String Methods, including maketrans/translate/expandtabs), R2 (Predicate Methods).
- **Details**: This spec directly covers string_ops.rs:848-901 (dispatch table for all 39 methods). R1 covers all 23 core methods and R2 covers all 6 predicate methods. Notably, maketrans, translate, and expandtabs are in the spec but missing from Q3's implementation list — this identifies a spec-vs-implementation gap. Q3 explicitly focuses on conformance test failures, which this spec governs.

### [MEDIUM] parser/ast.md is absent
- **Status**: ✅ RESOLVED
- **Action**: Added parser/ast.md at MEDIUM relevance. Key requirement: R1 (Expr enum including Slice variant).
- **Details**: This spec covers ast.rs:298-303 (Slice AST node definition). The 3-arg start/stop/step structure is the contract that parser (expr.rs:350-369) produces and runtime (list_ops.rs:132-176) consumes. Without this spec, the AST representation for Q1 is unspecified in the reference context.

### [MEDIUM] runtime/builtins.md is absent
- **Status**: ✅ RESOLVED
- **Action**: Added runtime/builtins.md at MEDIUM relevance. Key requirements: R5 (Enhanced P1 Builtins — operator dispatch including mb_bitor), R6 (Symbol Registration).
- **Details**: This spec covers builtins.rs:576-591 (mb_bitor function routing Dict|Dict to mb_dict_merge) and symbols.rs:200 (symbol registration). The \| operator dispatch for Dict types is part of operator overloading infrastructure. Without this spec, the type-level dispatch for Q2 is unspecified.

### [MEDIUM] types/type-checker.md is absent
- **Status**: ✅ RESOLVED
- **Action**: Added types/type-checker.md at MEDIUM relevance. Key requirement: R2 (Expression Type Inference — binary ops).
- **Details**: This spec covers check_expr.rs:414-421 (Dict|Dict type checking returns Ty::Any for non-integer | operands). The type checker behavior for | on Dict operands (returning Ty::Any to allow PEP 584 runtime semantics) must be understood by implementers working on Q2 conformance.

### [MEDIUM] all-mamba-p0.md incorrectly lists R3 (BigInt)
- **Status**: ✅ RESOLVED
- **Action**: Changed key requirement from 'R3: BigInt Fallback for integer overflow' to 'R5: Builtins Conformance (sequence, string, numeric validation against CPython 3.12)'.
- **Details**: all-mamba-p0.md R3 covers overflow detection on add/sub/mul and heap-allocated BigInt promotion. None of the three pre-clarification questions (list slicing, dict merge, string methods) mention BigInt, integer overflow, or numeric precision. R3 is not relevant to data-structures. R5 (Builtins Conformance) is the correct key requirement: R5.1 migrates 108 builtin tests to CPython 3.12 conformance, R5.2 achieves 100% compatibility for sequence, string, numeric functions — directly addressing Q3's conformance test failures.

### [LOW] pattern-matching.md is irrelevant
- **Status**: ✅ RESOLVED
- **Action**: Removed pattern-matching.md from this group's reference context entirely.
- **Details**: pattern-matching.md covers PEP 634 control-flow dispatch (parser pattern types, AS patterns, mapping rest capture, case-branch type narrowing, HIR pattern nodes, AST→HIR lowering, nested patterns in MIR, Cranelift IR codegen). None of these requirements address list slicing (Q1), dict merge via the \| operator (Q2), or string method conformance gaps (Q3). While mapping patterns involve dicts and sequence patterns involve lists, the spec concerns control-flow dispatch semantics, not data structure operations. Including it inverted correct priority.

### [LOW] README.md is rated MEDIUM but has no implementable requirements
- **Status**: ✅ RESOLVED
- **Action**: Downgraded README.md from MEDIUM to LOW, consistent with class-features review.
- **Details**: README.md is a spec-index table with no R1-Rn requirements, no acceptance criteria, no behavior specification. The class-features review for this same change rated README.md LOW with note 'Value is spec discovery only, not implementable requirements' — MEDIUM was inconsistent. If README is included for navigation, the specs it points to (list-ops.md, dict-ops.md, string-ops.md, tuple-ops.md) must be directly included. The three primary data-structure specs are now directly included at HIGH relevance.
