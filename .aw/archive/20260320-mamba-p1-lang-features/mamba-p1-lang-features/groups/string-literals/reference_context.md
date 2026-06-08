---
change: mamba-p1-lang-features
group: string-literals
date: 2026-03-20
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| lexer/tokens-and-indent.md | string-literals | high | R4: ByteStr/RawStr/TripleStr prefix forms; NEW R5: Escape sequence processing (uXXXX, UXXXXXXXX, N{name}, xNN, 0NNN) per Q2, Q3 |
| parser/ast.md | string-literals | high | R1: Expr enum needs BytesLit variant (issue #848); distinguishes b'...' from str literals per Q1 |
| types/type-representations.md | string-literals | high | R1: Ty::Bytes primitive type; R5: bytes in built-in type registry — direct answer to Q1 type model |
| types/type-checker.md | string-literals | high | R2: Literal type inference must add BytesLit → Ty::Bytes; currently absent per Q1 type checking requirement |
| runtime/value-and-rc.md | string-literals | high | R3: ObjData::Bytes and ObjData::ByteArray heap variants (Vec<u8>); NaN-boxed representation per Q1 |
| runtime/bytes-ops.md | string-literals | high | Full CPython bytes API: construction, indexing, slicing, len, concatenation (R1); decode/hex/fromhex (R3); find, replace, split, join, strip, startswith, endswith, count (R4); NEW: repr in R1 |
| runtime/string-ops.md | string-literals | medium | R5: mb_string_encode(s, encoding) → ObjData::Bytes; str↔bytes encoding bridge supporting Q4 decode/encode path |
| runtime/symbols.md | string-literals | medium | R1-R2: Register mb_bytes_* symbol names and MirExtern declarations (mb_bytes_find, mb_bytes_replace, mb_bytes_decode, mb_bytes_hex, mb_bytes_split, mb_bytes_join, mb_bytes_startswith, mb_bytes_endswith, mb_bytes_strip, mb_bytes_count) |
| testing/test-harness.md | string-literals | medium | R1, R3: JIT fixtures for b'...' construction, b'hello'[0] indexing, raw bytes, escape sequences (uXXXX, N{...}, xNN), decode/encode per Q2, Q3, Q4 |

