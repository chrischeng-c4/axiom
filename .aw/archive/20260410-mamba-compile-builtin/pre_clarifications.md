---
change: mamba-compile-builtin
date: 2026-04-10
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should the code object be a new ObjKind variant in ObjData, or a separate struct stored via an opaque handle (like iterators use integer handles)?
- **Answer**: Add a new ObjKind::CodeObject variant to ObjData. This is the cleanest approach matching how other heap objects (BigInt, Complex) are stored. The code object will wrap the parsed AST Module plus the filename and mode.

### Q2: General
- **Question**: For 'eval' mode, should multi-statement input (e.g. 'x = 1') raise SyntaxError, matching CPython behavior?
- **Answer**: Yes. In 'eval' mode the source must parse as a single expression. If the source is a statement sequence, raise SyntaxError('invalid syntax') just like CPython.

### Q3: General
- **Question**: For 'single' mode, should multi-statement input be rejected?
- **Answer**: Yes, 'single' mode accepts exactly one interactive statement. Multi-statement input raises SyntaxError('multiple statements found while compiling a single statement').

### Q4: General
- **Question**: How should filename threading work given that mb_compile is a pure runtime function with no CompilerSession access?
- **Answer**: Store the filename string directly in the CodeObject ObjData variant. When diagnostics are needed, reconstruct a SourceFile from the stored source and filename inline within mb_compile. The source map is local to the parse call.

### Q5: General
- **Question**: Should SyntaxError include lineno/offset attributes on the exception object like CPython?
- **Answer**: For R4: include lineno and column information in the SyntaxError message text. Full attribute threading on the exception Instance is deferred. Message text with 'line N col C' is sufficient for phase 1.

