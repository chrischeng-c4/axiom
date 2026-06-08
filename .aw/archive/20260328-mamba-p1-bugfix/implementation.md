---
id: implementation
type: change_implementation
change_id: mamba-p1-bugfix
---

# Implementation

## Summary

Fix #1112: Add explicit LBrace/LParen/LBracket dispatch arms in parse_stmt() to prevent accidental capture by future match arms. Add comprehensive test cases validating dict/set literal parsing inside try/except/if compound statement bodies. No changes to lexer (tokens-and-indent confirmed correct).

## Diff

```diff
diff --git a/crates/mamba/src/parser/stmt.rs b/crates/mamba/src/parser/stmt.rs
index 6fd007e5..d02af551 100644
--- a/crates/mamba/src/parser/stmt.rs
+++ b/crates/mamba/src/parser/stmt.rs
@@ -77,7 +77,15 @@ impl<'a> Parser<'a> {
             }
             TokenKind::Import => self.parse_import(),
             TokenKind::From => self.parse_from_import(),
-            // Any expression-starting token: handles assignment, augassign,
+            // Brace-delimited expressions (dict/set literals) appearing as
+            // the first token of a statement must route through the expression
+            // parser via parse_ident_stmt(). Explicit arms prevent accidental
+            // capture by future match arms and clarify that LBrace/LParen/
+            // LBracket are valid statement-leading tokens (#1112).
+            TokenKind::LBrace | TokenKind::LParen | TokenKind::LBracket => {
+                self.parse_ident_stmt()
+            }
+            // Any other expression-starting token: handles assignment, augassign,
             // tuple unpacking, var decl, and bare expression statements.
             _ => self.parse_ident_stmt(),
         }
@@ -1055,4 +1063,53 @@ mod tests {
             other => panic!("expected Assign(Index), got {other:?}"),
         }
     }
+
+    // --- LBrace-leading statements (#1112) ---
+
+    #[test]
+    fn test_bare_dict_expr_stmt() {
+        // Bare dict literal as expression statement
+        match parse_stmt("{}\n") {
+            Stmt::ExprStmt(e) => {
+                assert!(matches!(e.node, Expr::DictLit(ref entries) if entries.is_empty()));
+            }
+            other => panic!("expected ExprStmt(DictLit), got {other:?}"),
+        }
+    }
+
+    #[test]
+    fn test_bare_set_expr_stmt() {
+        // Bare set literal as expression statement
+        match parse_stmt("{1, 2, 3}\n") {
+            Stmt::ExprStmt(e) => {
+                assert!(matches!(e.node, Expr::SetLit(ref elems) if elems.len() == 3));
+            }
+            other => panic!("expected ExprStmt(SetLit), got {other:?}"),
+        }
+    }
+
+    #[test]
+    fn test_dict_subscript_assign() {
+        // Dict literal subscript assignment
+        match parse_stmt("d = {}['x']\n") {
+            Stmt::Assign { value, .. } => {
+                assert!(matches!(value.node, Expr::Index { .. }));
+            }
+            other => panic!("expected Assign, got {other:?}"),
+        }
+    }
+
+    #[test]
+    fn test_dict_literal_assign() {
+        // Dict literal assignment with entries
+        match parse_stmt("d = {'a': 1, 'b': 2}\n") {
+            Stmt::Assign { value, .. } => {
+                match value.node {
+                    Expr::DictLit(entries) => assert_eq!(entries.len(), 2),
+                    other => panic!("expected DictLit, got {other:?}"),
+                }
+            }
+            other => panic!("expected Assign, got {other:?}"),
+        }
+    }
 }
diff --git a/crates/mamba/src/parser/stmt_compound.rs b/crates/mamba/src/parser/stmt_compound.rs
index 5a04f0d2..70113e3f 100644
--- a/crates/mamba/src/parser/stmt_compound.rs
+++ b/crates/mamba/src/parser/stmt_compound.rs
@@ -872,6 +872,144 @@ mod tests {
         }
     }
 
+    // --- Try with dict/set literals in body (#1112) ---
+
+    #[test]
+    fn test_try_empty_dict_subscript_in_body() {
+        // S-TRY-DICT-1: empty dict subscript in try body
+        match parse_stmt("try:\n    d = {}['x']\nexcept KeyError:\n    pass\n") {
+            Stmt::Try { body, handlers, .. } => {
+                assert_eq!(body.len(), 1);
+                match &body[0].node {
+                    Stmt::Assign { target, value } => {
+                        assert!(matches!(&target.node, Expr::Ident(n) if n == "d"));
+                        match &value.node {
+                            Expr::Index { object, index } => {
+                                assert!(matches!(&object.node, Expr::DictLit(entries) if entries.is_empty()));
+                                assert!(matches!(&index.node, Expr::StrLit(s) if s == "x"));
+                            }
+                            other => panic!("expected Index, got {other:?}"),
+                        }
+                    }
+                    other => panic!("expected Assign, got {other:?}"),
+                }
+                assert_eq!(handlers.len(), 1);
+                assert!(matches!(
+                    &handlers[0].exc_type.as_ref().unwrap().node,
+                    Expr::Ident(n) if n == "KeyError"
+                ));
+            }
+            other => panic!("expected Try, got {other:?}"),
+        }
+    }
+
+    #[test]
+    fn test_try_set_method_call_in_body() {
+        // S-TRY-DICT-2: set literal method call in try body
+        match parse_stmt("try:\n    s = {1, 2}.remove(99)\nexcept KeyError:\n    pass\n") {
+            Stmt::Try { body, handlers, .. } => {
+                assert_eq!(body.len(), 1);
+                match &body[0].node {
+                    Stmt::Assign { target, value } => {
+                        assert!(matches!(&target.node, Expr::Ident(n) if n == "s"));
+                        // value should be Call(Attr(SetLit([1,2]), "remove"), [99])
+                        match &value.node {
+                            Expr::Call { func, args } => {
+                                match &func.node {
+                                    Expr::Attr { object, attr } => {
+                                        match &object.node {
+                                            Expr::SetLit(elems) => assert_eq!(elems.len(), 2),
+                                            other => panic!("expected SetLit, got {other:?}"),
+                                        }
+                                        assert_eq!(attr, "remove");
+                                    }
+                                    other => panic!("expected Attr, got {other:?}"),
+                                }
+                                assert_eq!(args.len(), 1);
+                            }
+                            other => panic!("expected Call, got {other:?}"),
+                        }
+                    }
+                    other => panic!("expected Assign, got {other:?}"),
+                }
+                assert_eq!(handlers.len(), 1);
+            }
+            other => panic!("expected Try, got {other:?}"),
+        }
+    }
+
+    #[test]
+    fn test_try_dict_literal_subscript_in_body() {
+        // S-TRY-DICT-3: dict literal subscript in try body
+        match parse_stmt("try:\n    x = {'a': 1}['a']\nexcept:\n    pass\n") {
+            Stmt::Try { body, handlers, .. } => {
+                assert_eq!(body.len(), 1);
+                match &body[0].node {
+                    Stmt::Assign { target, value } => {
+                        assert!(matches!(&target.node, Expr::Ident(n) if n == "x"));
+                        match &value.node {
+                            Expr::Index { object, index } => {
+                                match &object.node {
+                                    Expr::DictLit(entries) => assert_eq!(entries.len(), 1),
+                                    other => panic!("expected DictLit, got {other:?}"),
+                                }
+                                assert!(matches!(&index.node, Expr::StrLit(s) if s == "a"));
+                            }
+                            other => panic!("expected Index, got {other:?}"),
+                        }
+                    }
+                    other => panic!("expected Assign, got {other:?}"),
+                }
+                assert_eq!(handlers.len(), 1);
+                // Bare except (no type)
+                assert!(handlers[0].exc_type.is_none());
+            }
+            other => panic!("expected Try, got {other:?}"),
+        }
+    }
+
+    #[test]
+    fn test_try_bare_empty_dict_in_body() {
+        // Bare empty dict expression statement in try body
+        match parse_stmt("try:\n    {}\nexcept:\n    pass\n") {
+            Stmt::Try { body, handlers, .. } => {
+                assert_eq!(body.len(), 1);
+                match &body[0].node {
+                    Stmt::ExprStmt(expr) => {
+                        assert!(matches!(&expr.node, Expr::DictLit(entries) if entries.is_empty()));
+                    }
+                    other => panic!("expected ExprStmt(DictLit), got {other:?}"),
+                }
+                assert_eq!(handlers.len(), 1);
+            }
+            other => panic!("expected Try, got {other:?}"),
+        }
+    }
+
+    #[test]
+    fn test_if_dict_assignment_with_followup() {
+        // S-TRY-DICT-4: dict in if body followed by another statement
+        match parse_stmt("if True:\n    d = {}\n    print(d)\n") {
+            Stmt::If { body, .. } => {
+                assert_eq!(body.len(), 2);
+                match &body[0].node {
+                    Stmt::Assign { target, value } => {
+                        assert!(matches!(&target.node, Expr::Ident(n) if n == "d"));
+                        assert!(matches!(&value.node, Expr::DictLit(entries) if entries.is_empty()));
+                    }
+                    other => panic!("expected Assign, got {other:?}"),
+                }
+                match &body[1].node {
+                    Stmt::ExprStmt(expr) => {
+                        assert!(matches!(&expr.node, Expr::Call { .. }));
+                    }
+                    other => panic!("expected ExprStmt(Call), got {other:?}"),
+                }
+            }
+            other => panic!("expected If, got {other:?}"),
+        }
+    }
+
     // --- Raise ---
 
     #[test]
@@ -1078,4 +1216,5 @@ mod tests {
         let result = parser::parse("async pass\n", fid());
         assert!(result.is_err());
     }
+
 }

```

## Review: statements

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-p1-bugfix

**Summary**: Parser fix for try/except with dict/set literals. Code changes in stmt.rs and stmt_compound.rs.

## Review: tokens-and-indent

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: mamba-p1-bugfix

**Summary**: Lexer token changes for try/except dict/set support.

