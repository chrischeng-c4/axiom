---
change: mamba-all-p1
group: syntax-features
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: PARTIALLY IMPLEMENTED. types/generic.rs has TypeVar struct, GenericParams, type variable resolution and unification. Parsed: def foo[T, U]() generic syntax (ast.rs:35 type_params). MISSING: TypeVarTuple (no matches), ParamSpec (no matches). Type alias statement (type X = ...) status unclear. Recommend implementing all in this change.

### Q2: General
- **Answer**: NOT APPLICABLE — dict unpacking {**d1, **d2} is NOT IMPLEMENTED in parser. DictLit stores simple key-value pairs only (Vec<(Expr, Expr)>) with no unpacking markers. Function call **kwargs IS supported via DoubleStarArg. Parser needs to be extended to support DictUnpack entries in dict literals before type inference becomes relevant.

### Q3: General
- **Answer**: Assignment unpacking IMPLEMENTED. Starred(Box<Expr>) and UnpackTarget in ast.rs:358-360. Pattern parsing with Star(Option<Name>) in pattern.rs:125-150. HIR lowering in ast_to_hir.rs:1098. MIR desugaring in hir_to_mir.rs:1615. f(*args, **kwargs) also works via existing call argument handling. Focus on conformance testing.

### Q4: General
- **Answer**: FULL CLOSURE MECHANISM IMPLEMENTED. global/nonlocal parsed as AST statements (ast.rs:140-142). runtime/closure.rs (123 lines) has MbClosure struct with captured variables, thread-local storage, mb_closure_new(), get/set by index. Not a flat variable model — proper upvalue cells with closure capture.

### Q5: General
- **Answer**: BYTES TYPE IMPLEMENTED as distinct type. Lexer supports b'' (token.rs:258 ByteStr). Runtime bytes_ops.rs (239 lines) has mb_bytes_new(), mb_bytearray_new(), getitem, etc. ObjData has Bytes(Vec<u8>) and ByteArray variants separate from strings. Scope: slicing, decode, hex, fromhex are in scope per requirements.

