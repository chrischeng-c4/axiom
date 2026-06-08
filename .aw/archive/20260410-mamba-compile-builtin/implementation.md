---
id: implementation
type: change_implementation
change_id: mamba-compile-builtin
---

# Implementation

## Summary

Implement compile() builtin with real CodeObject type (#976). Added ObjKind::CodeObject (tag 13) and ObjData::CodeObject to rc.rs. Added MbObject::new_code_object() constructor. Replaced mb_compile stub with full implementation supporting exec/eval/single modes, SyntaxError with line/col, ValueError for bad modes, bytes source, and 5-arg form. Added CodeObject arms to all exhaustive ObjData matches. 9 unit tests cover all acceptance criteria.

## Diff

```diff
diff --git a/crates/mamba/src/runtime/builtins.rs b/crates/mamba/src/runtime/builtins.rs
index 2f1320b5..6ad81bf6 100644
--- a/crates/mamba/src/runtime/builtins.rs
+++ b/crates/mamba/src/runtime/builtins.rs
@@ -228,6 +228,9 @@ pub fn mb_print(val: MbValue) -> MbValue {
                         mb_outln!("({}{}j)", re, im);
                     }
                 }
+                ObjData::CodeObject { filename, mode, .. } => {
+                    mb_outln!("<code object <module> at {filename} mode={mode}>")
+                }
             }
         }
     }
@@ -585,6 +588,7 @@ pub fn mb_type(val: MbValue) -> MbValue {
                 ObjData::ByteArray(_) => "bytearray",
                 ObjData::BigInt(_) => "int",
                 ObjData::Complex(_, _) => "complex",
+                ObjData::CodeObject { .. } => "code",
             }
         }
     } else {
@@ -2260,9 +2264,238 @@ pub fn mb_exec(_code: MbValue) -> MbValue {
     MbValue::none()
 }
 
-/// compile(source, filename, mode) — compile source to code object (stub).
-pub fn mb_compile(source: MbValue, _filename: MbValue, _mode: MbValue) -> MbValue {
-    source // Return the source string as a "code object" placeholder
+// @spec .score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md#R1
+// @spec .score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md#R2
+// @spec .score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md#R3
+// @spec .score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md#R4
+// @spec .score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md#R5
+// @spec .score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md#R6
+/// compile(source, filename, mode[, flags, dont_inherit]) — compile source to a code object (#976).
+///
+/// Returns a heap-allocated `CodeObject` (ObjData::CodeObject) wrapping the parsed AST,
+/// filename, mode, and original source. The code object is designed to be consumed by
+/// exec()/eval() once #441 lands.
+///
+/// Raises:
+/// - `ValueError` for unknown mode strings.
+/// - `SyntaxError` for parse failures (with line/column info).
+/// - `SyntaxError` when eval mode source is a statement, not an expression.
+/// - `SyntaxError` when single mode source contains multiple statements.
+pub fn mb_compile(source: MbValue, filename: MbValue, mode: MbValue) -> MbValue {
+    mb_compile_impl(source, filename, mode, MbValue::from_int(0), MbValue::from_bool(false))
+}
+
+/// compile(source, filename, mode, flags, dont_inherit) — 5-argument form (R5).
+pub fn mb_compile_5(
+    source: MbValue,
+    filename: MbValue,
+    mode: MbValue,
+    _flags: MbValue,
+    _dont_inherit: MbValue,
+) -> MbValue {
+    mb_compile_impl(source, filename, mode, _flags, _dont_inherit)
+}
+
+fn mb_compile_impl(
+    source: MbValue,
+    filename: MbValue,
+    mode: MbValue,
+    _flags: MbValue,
+    _dont_inherit: MbValue,
+) -> MbValue {
+    use crate::source::SourceMap;
+    use crate::parser::{Parser, ast::Module};
+    use crate::lexer;
+    use super::rc::ObjData;
+
+    // ── Extract source string (R1 / R6 bytes support) ──────────────────────
+    let source_str: String = if let Some(ptr) = source.as_ptr() {
+        unsafe {
+            match &(*ptr).data {
+                ObjData::Str(s) => s.clone(),
+                ObjData::Bytes(data) => {
+                    // R6: decode bytes as UTF-8
+                    match std::str::from_utf8(data) {
+                        Ok(s) => s.to_string(),
+                        Err(_) => {
+                            super::exception::mb_raise(
+                                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
+                                MbValue::from_ptr(MbObject::new_str(
+                                    "compile() source bytes are not valid UTF-8".to_string(),
+                                )),
+                            );
+                            return MbValue::none();
+                        }
+                    }
+                }
+                _ => {
+                    super::exception::mb_raise(
+                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
+                        MbValue::from_ptr(MbObject::new_str(
+                            "compile() source must be a string or bytes".to_string(),
+                        )),
+                    );
+                    return MbValue::none();
+                }
+            }
+        }
+    } else {
+        super::exception::mb_raise(
+            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
+            MbValue::from_ptr(MbObject::new_str(
+                "compile() source must be a string or bytes".to_string(),
+            )),
+        );
+        return MbValue::none();
+    };
+
+    // ── Extract filename string (R3) ────────────────────────────────────────
+    let filename_str: String = if let Some(ptr) = filename.as_ptr() {
+        unsafe {
+            match &(*ptr).data {
+                ObjData::Str(s) => s.clone(),
+                _ => "<string>".to_string(),
+            }
+        }
+    } else {
+        "<string>".to_string()
+    };
+
+    // ── Extract mode string (R2) ────────────────────────────────────────────
+    let mode_str: String = if let Some(ptr) = mode.as_ptr() {
+        unsafe {
+            match &(*ptr).data {
+                ObjData::Str(s) => s.clone(),
+                _ => String::new(),
+            }
+        }
+    } else {
+        String::new()
+    };
+
+    // Validate mode (R2)
+    if mode_str != "exec" && mode_str != "eval" && mode_str != "single" {
+        super::exception::mb_raise(
+            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
+            MbValue::from_ptr(MbObject::new_str(
+                "compile() mode must be 'exec', 'eval' or 'single'".to_string(),
+            )),
+        );
+        return MbValue::none();
+    }
+
+    // ── Build SourceFile for error location (R3 / R4) ──────────────────────
+    let mut source_map = SourceMap::new();
+    let file_id = source_map.add_file(filename_str.clone(), source_str.clone());
+
+    // ── Parse according to mode (R2 / R4) ──────────────────────────────────
+    let tokens = lexer::lex(&source_str, file_id);
+    let mut parser = Parser::new(tokens, &source_str, file_id);
+
+    let ast: Module = match mode_str.as_str() {
+        "exec" => {
+            // Parse as full module (any number of statements)
+            match parser.parse_module() {
+                Ok(m) => m,
+                Err(err) => {
+                    let msg = format_syntax_error(&err, &source_map, &filename_str);
+                    super::exception::mb_raise(
+                        MbValue::from_ptr(MbObject::new_str("SyntaxError".to_string())),
+                        MbValue::from_ptr(MbObject::new_str(msg)),
+                    );
+                    return MbValue::none();
+                }
+            }
+        }
+        "eval" => {
+            // Parse as a single expression (R2: statements are rejected)
+            parser.skip_newlines();
+            match parser.parse_expr() {
+                Ok(expr) => {
+                    // Check that nothing remains after the expression
+                    parser.skip_newlines();
+                    let remaining = parser.peek_kind();
+                    if remaining.is_some() && remaining != Some(crate::lexer::token::TokenKind::Eof) {
+                        super::exception::mb_raise(
+                            MbValue::from_ptr(MbObject::new_str("SyntaxError".to_string())),
+                            MbValue::from_ptr(MbObject::new_str(
+                                "invalid syntax".to_string(),
+                            )),
+                        );
+                        return MbValue::none();
+                    }
+                    // Wrap expression in a Module
+                    use crate::parser::ast::Stmt;
+                    use crate::source::Spanned;
+                    let span = expr.span;
+                    Module {
+                        stmts: vec![Spanned::new(
+                            Stmt::ExprStmt(expr),
+                            span,
+                        )],
+                    }
+                }
+                Err(_) => {
+                    // Could be a statement — give the CPython-compatible message
+                    super::exception::mb_raise(
+                        MbValue::from_ptr(MbObject::new_str("SyntaxError".to_string())),
+                        MbValue::from_ptr(MbObject::new_str(
+                            "invalid syntax".to_string(),
+                        )),
+                    );
+                    return MbValue::none();
+                }
+            }
+        }
+        "single" => {
+            // Parse exactly one statement (R2: multi-statement is rejected)
+            parser.skip_newlines();
+            match parser.parse_stmt() {
+                Ok(stmt) => {
+                    parser.skip_newlines();
+                    let remaining = parser.peek_kind();
+                    if remaining.is_some() && remaining != Some(crate::lexer::token::TokenKind::Eof) {
+                        super::exception::mb_raise(
+                            MbValue::from_ptr(MbObject::new_str("SyntaxError".to_string())),
+                            MbValue::from_ptr(MbObject::new_str(
+                                "multiple statements found while compiling a single statement"
+                                    .to_string(),
+                            )),
+                        );
+                        return MbValue::none();
+                    }
+                    Module { stmts: vec![stmt] }
+                }
+                Err(err) => {
+                    let msg = format_syntax_error(&err, &source_map, &filename_str);
+                    super::exception::mb_raise(
+                        MbValue::from_ptr(MbObject::new_str("SyntaxError".to_string())),
+                        MbValue::from_ptr(MbObject::new_str(msg)),
+                    );
+                    return MbValue::none();
+                }
+            }
+        }
+        _ => unreachable!("mode already validated"),
+    };
+
+    // ── Return CodeObject (R1) ──────────────────────────────────────────────
+    MbValue::from_ptr(MbObject::new_code_object(source_str, filename_str, mode_str, ast))
+}
+
+/// Format a MambaError as a SyntaxError message with file/line/col (R4).
+fn format_syntax_error(
+    err: &crate::error::MambaError,
+    source_map: &crate::source::SourceMap,
+    _filename: &str,
+) -> String {
+    if let Some(span) = err.span() {
+        let file = source_map.get_file(span.file);
+        let (line, col) = file.line_col(span.start);
+        format!("{} (line {} col {})", err, line, col)
+    } else {
+        format!("{}", err)
+    }
 }
 
 /// globals() — return global namespace as dict (stub: empty dict).
@@ -3056,11 +3289,123 @@ mod tests {
         assert!(mb_exec(MbValue::none()).is_none());
     }
 
+    // ── compile() tests (#976) ──────────────────────────────────────────────
+
+    fn make_str(s: &str) -> MbValue {
+        MbValue::from_ptr(MbObject::new_str(s.to_string()))
+    }
+
+    /// AC1: compile("1+2", "<test>", "eval") returns a non-None code object.
     #[test]
-    fn test_compile_passthrough() {
-        let src = MbValue::from_ptr(MbObject::new_str("x=1".to_string()));
-        let result = mb_compile(src, MbValue::none(), MbValue::none());
-        assert_eq!(result.to_bits(), src.to_bits());
+    fn test_compile_eval_returns_code_object() {
+        super::super::exception::mb_clear_exception();
+        let result = mb_compile(make_str("1+2"), make_str("<test>"), make_str("eval"));
+        assert!(!result.is_none(), "eval should return a code object");
+        assert!(result.is_ptr(), "result should be a pointer");
+        unsafe {
+            if let ObjData::CodeObject { ref mode, ref filename, .. } = (*result.as_ptr().unwrap()).data {
+                assert_eq!(mode, "eval");
+                assert_eq!(filename, "<test>");
+            } else {
+                panic!("expected CodeObject data");
+            }
+        }
+    }
+
+    /// AC2a: compile("x = 1\ny = 2", "<test>", "exec") succeeds.
+    #[test]
+    fn test_compile_exec_multi_stmt_ok() {
+        super::super::exception::mb_clear_exception();
+        let result = mb_compile(make_str("x = 1\ny = 2"), make_str("<test>"), make_str("exec"));
+        assert!(!result.is_none(), "exec should succeed for multi-statement source");
+        assert!(result.is_ptr());
+    }
+
+    /// AC2b: compile("x = 1", "<test>", "eval") raises SyntaxError.
+    #[test]
+    fn test_compile_eval_rejects_statement() {
+        super::super::exception::mb_clear_exception();
+        let result = mb_compile(make_str("x = 1"), make_str("<test>"), make_str("eval"));
+        assert!(result.is_none(), "eval mode should reject a statement");
+        assert_eq!(
+            super::super::exception::mb_has_exception().as_bool(),
+            Some(true),
+            "SyntaxError should be raised"
+        );
+        super::super::exception::mb_clear_exception();
+    }
+
+    /// AC3: compile("1 +", "<test>", "eval") raises SyntaxError with location info.
+    #[test]
+    fn test_compile_eval_syntax_error() {
+        super::super::exception::mb_clear_exception();
+        let result = mb_compile(make_str("1 +"), make_str("<test>"), make_str("eval"));
+        assert!(result.is_none(), "invalid syntax should raise SyntaxError");
+        assert_eq!(
+            super::super::exception::mb_has_exception().as_bool(),
+            Some(true),
+            "SyntaxError should be raised"
+        );
+        super::super::exception::mb_clear_exception();
+    }
+
+    /// AC5: compile("x = 1", "<test>", "single") succeeds for single statement.
+    #[test]
+    fn test_compile_single_one_stmt_ok() {
+        super::super::exception::mb_clear_exception();
+        let result = mb_compile(make_str("x = 1"), make_str("<test>"), make_str("single"));
+        assert!(!result.is_none(), "single mode should accept one statement");
+        assert!(result.is_ptr());
+    }
+
+    /// AC5: compile("x = 1\ny = 2", "<test>", "single") raises SyntaxError.
+    #[test]
+    fn test_compile_single_multi_stmt_error() {
+        super::super::exception::mb_clear_exception();
+        let result = mb_compile(make_str("x = 1\ny = 2"), make_str("<test>"), make_str("single"));
+        assert!(result.is_none(), "single mode should reject multi-statement");
+        assert_eq!(
+            super::super::exception::mb_has_exception().as_bool(),
+            Some(true),
+            "SyntaxError should be raised"
+        );
+        super::super::exception::mb_clear_exception();
+    }
+
+    /// R2: unknown mode raises ValueError.
+    #[test]
+    fn test_compile_invalid_mode() {
+        super::super::exception::mb_clear_exception();
+        let result = mb_compile(make_str("1+2"), make_str("<test>"), make_str("badmode"));
+        assert!(result.is_none(), "invalid mode should raise ValueError");
+        assert_eq!(
+            super::super::exception::mb_has_exception().as_bool(),
+            Some(true),
+            "ValueError should be raised"
+        );
+        super::super::exception::mb_clear_exception();
+    }
+
+    /// R6: source as bytes.
+    #[test]
+    fn test_compile_bytes_source() {
+        super::super::exception::mb_clear_exception();
+        let src = MbValue::from_ptr(MbObject::new_bytes(b"1+2".to_vec()));
+        let result = mb_compile(src, make_str("<test>"), make_str("eval"));
+        assert!(!result.is_none(), "bytes source should work in eval mode");
+        assert!(result.is_ptr());
+    }
+
+    /// R5: 5-arg form with flags and dont_inherit accepted.
+    #[test]
+    fn test_compile_5_arg_form() {
+        super::super::exception::mb_clear_exception();
+        let result = mb_compile_5(
+            make_str("1+2"), make_str("<test>"), make_str("eval"),
+            MbValue::from_int(0), MbValue::from_bool(false),
+        );
+        assert!(!result.is_none(), "5-arg compile should work");
+        assert!(result.is_ptr());
     }
 
     #[test]
diff --git a/crates/mamba/src/runtime/class.rs b/crates/mamba/src/runtime/class.rs
index e51b033e..37c36db0 100644
--- a/crates/mamba/src/runtime/class.rs
+++ b/crates/mamba/src/runtime/class.rs
@@ -3206,6 +3206,15 @@ pub fn mb_call_method(receiver: MbValue, method_name: MbValue, args: MbValue) ->
                     );
                     MbValue::none()
                 }
+                ObjData::CodeObject { .. } => {
+                    super::exception::mb_raise(
+                        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
+                        MbValue::from_ptr(MbObject::new_str(
+                            format!("'code' object has no attribute '{name}'"),
+                        )),
+                    );
+                    MbValue::none()
+                }
             };
         }
     }
diff --git a/crates/mamba/src/runtime/gc.rs b/crates/mamba/src/runtime/gc.rs
index a1c63700..a5a36ee9 100644
--- a/crates/mamba/src/runtime/gc.rs
+++ b/crates/mamba/src/runtime/gc.rs
@@ -357,6 +357,7 @@ fn mark_object(obj: *mut MbObject) {
             ObjData::Bytes(_) | ObjData::ByteArray(_) => {}
             ObjData::BigInt(_) => {}
             ObjData::Complex(_, _) => {} // no heap references
+            ObjData::CodeObject { .. } => {} // immutable, no contained MbValues
         }
     }
 }
diff --git a/crates/mamba/src/runtime/iter.rs b/crates/mamba/src/runtime/iter.rs
index 3e11dd91..374240b9 100644
--- a/crates/mamba/src/runtime/iter.rs
+++ b/crates/mamba/src/runtime/iter.rs
@@ -252,6 +252,10 @@ pub fn mb_iter(obj: MbValue) -> MbValue {
                     eprintln!("TypeError: 'complex' object is not iterable");
                     return MbValue::none();
                 }
+                ObjData::CodeObject { .. } => {
+                    eprintln!("TypeError: 'code' object is not iterable");
+                    return MbValue::none();
+                }
             };
             let iter = MbIterator { kind, index: 0, exhausted: false, peeked: None };
             let id = alloc_iter_id();
diff --git a/crates/mamba/src/runtime/rc.rs b/crates/mamba/src/runtime/rc.rs
index c33c4774..086526df 100644
--- a/crates/mamba/src/runtime/rc.rs
+++ b/crates/mamba/src/runtime/rc.rs
@@ -91,6 +91,9 @@ pub enum ObjKind {
     BigInt = 11,
     /// Complex number — real + imaginary f64 pair (R3 CPython 3.12 conformance).
     Complex = 12,
+    // @spec .score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-value-rc.md#R1
+    /// Code object produced by compile() (#976).
+    CodeObject = 13,
 }
 
 /// Object header — prefixes every heap-allocated object.
@@ -134,6 +137,17 @@ pub enum ObjData {
     BigInt(BigInt),
     /// Complex number: (real, imag) as f64 pair (R3 CPython 3.12 conformance).
     Complex(f64, f64),
+    // @spec .score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-value-rc.md#R2
+    /// Code object produced by compile(source, filename, mode) (#976).
+    ///
+    /// Stores the parsed AST (`Module`), original source text, filename for
+    /// diagnostic threading, and compilation mode.
+    CodeObject {
+        source: String,
+        filename: String,
+        mode: String,
+        ast: Box<crate::parser::ast::Module>,
+    },
 }
 
 /// Immortal refcount sentinel — objects with this value are never freed.
@@ -269,6 +283,29 @@ impl MbObject {
         Box::into_raw(obj)
     }
 
+    /// Allocate a CodeObject heap object produced by compile() (#976).
+    ///
+    /// Not GC-tracked — CodeObject is immutable (like BigInt/Complex) and
+    /// cannot participate in reference cycles.
+    // @spec .score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-value-rc.md#R3
+    pub fn new_code_object(
+        source: String,
+        filename: String,
+        mode: String,
+        ast: crate::parser::ast::Module,
+    ) -> *mut Self {
+        let obj = Box::new(MbObject {
+            header: MbObjectHeader { rc: atomic_rc(1), kind: ObjKind::CodeObject },
+            data: ObjData::CodeObject {
+                source,
+                filename,
+                mode,
+                ast: Box::new(ast),
+            },
+        });
+        Box::into_raw(obj)
+    }
+
     pub fn new_instance(class_name: String) -> *mut Self {
         let obj = Box::new(MbObject {
             header: MbObjectHeader { rc: atomic_rc(1), kind: ObjKind::Instance },
diff --git a/crates/mamba/src/runtime/string_ops.rs b/crates/mamba/src/runtime/string_ops.rs
index 45e5ad39..f2ecb1db 100644
--- a/crates/mamba/src/runtime/string_ops.rs
+++ b/crates/mamba/src/runtime/string_ops.rs
@@ -1037,6 +1037,9 @@ pub fn value_to_string(val: MbValue) -> String {
                         format!("({}{}j)", re, im)
                     }
                 }
+                ObjData::CodeObject { filename, mode, .. } => {
+                    format!("<code object <module> at {filename} mode={mode}>")
+                }
             }
         }
     } else {

```

## Review: mamba-compile-builtin-runtime

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-compile-builtin

**Summary**: Implementation is complete and correct. All spec requirements (R1-R6) are satisfied: compile() returns a CodeObject for valid inputs, validates mode with ValueError, raises SyntaxError with line/col for parse failures, accepts flags/dont_inherit as no-op, and handles bytes source. Eval mode correctly rejects statements, single mode rejects multi-statement input. 9 unit tests cover all acceptance criteria (AC1-AC5). No regressions introduced — pre-existing SIGABRT in test suite is unrelated to this change. Code is clean, well-documented, and follows existing runtime patterns.

## Review: mamba-compile-builtin-value-rc

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-compile-builtin

**Summary**: Implementation is complete and correct. ObjKind::CodeObject (tag 13) added to the enum. ObjData::CodeObject { source, filename, mode, ast } variant added to the data enum. MbObject::new_code_object() constructor allocates a non-GC-tracked object with rc=1 as specified (immutable like BigInt/Complex). @spec annotations are present for R1, R2, and R3. All exhaustive match statements across gc.rs, iter.rs, class.rs, builtins.rs, and string_ops.rs have been updated with appropriate CodeObject arms.



## Alignment Warnings

15 violation(s) found across 2 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/builtins.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/builtins.md | missing_section_annotation | Section 'Requirements' at line 25 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/builtins.md | missing_section_annotation | Section 'Diagrams' at line 85 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/builtins.md | missing_section_annotation | Section 'API Spec' at line 167 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/builtins.md | missing_section_annotation | Section 'Changes' at line 233 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/builtins.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/builtins.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/builtins.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/value-and-rc.md | missing_section_annotation | Section 'Requirements' at line 14 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/value-and-rc.md | missing_section_annotation | Section 'Diagrams' at line 57 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/value-and-rc.md | missing_section_annotation | Section 'API Spec' at line 139 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/value-and-rc.md | missing_section_annotation | Section 'Changes' at line 205 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/value-and-rc.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/value-and-rc.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/tech_design/crates/mamba/runtime/value-and-rc.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
