use super::ast::*;
use super::Parser;
use crate::lexer::token::TokenKind;
use crate::source::span::{Span, Spanned};

impl<'a> Parser<'a> {
    /// Ternary: `body if condition else else_body`
    pub(crate) fn parse_ternary(
        &mut self,
        body: Spanned<Expr>,
    ) -> crate::error::Result<Spanned<Expr>> {
        let start = body.span.start;
        self.expect(TokenKind::If)?;
        let condition = self.parse_expr_bp(0)?;
        self.expect(TokenKind::Else)?;
        let else_body = self.parse_expr()?;
        let span = Span::new(self.file_id, start, else_body.span.end);
        Ok(Spanned::new(
            Expr::IfExpr {
                body: Box::new(body),
                condition: Box::new(condition),
                else_body: Box::new(else_body),
            },
            span,
        ))
    }

    /// Lambda: `lambda x, y: body` (Python-style) or `lambda x: int: body` (typed).
    /// Type annotations are optional — omitted params default to Any.
    pub(crate) fn parse_lambda(&mut self) -> crate::error::Result<Spanned<Expr>> {
        let (start, _) = self.advance(); // consume `lambda`
        let mut params: Vec<Param> = Vec::new();

        // No-param lambda: `lambda: body`
        // PEP 570/3102 markers are valid in lambda parameter lists too:
        // a `/` retroactively marks prior params positional-only, and a bare
        // `*` (or `*args`) makes the following params keyword-only.
        let mut seen_star = false;
        if self.peek_kind() != Some(TokenKind::Colon) {
            loop {
                let p_start = self.peek().map(|t| t.start).unwrap_or(0);

                // `/` positional-only separator (`lambda a, /, b: ...`).
                if self.peek_kind() == Some(TokenKind::Slash) {
                    self.advance();
                    for p in params.iter_mut() {
                        if p.kind == ParamKind::Regular {
                            p.pos_only = true;
                        }
                    }
                    if self.peek_kind() == Some(TokenKind::Comma) {
                        self.advance();
                        continue;
                    }
                    break;
                }

                // Handle **kwargs / *args / bare `*`
                let kind = if self.peek_kind() == Some(TokenKind::DoubleStar) {
                    self.advance();
                    ParamKind::DoubleStar
                } else if self.peek_kind() == Some(TokenKind::Star) {
                    self.advance();
                    // bare `*` (keyword-only separator, `lambda a, *, c: ...`)
                    // vs `*args`: a name must follow for the latter.
                    if !self.peek_kind().as_ref().map_or(false, Self::is_name_token) {
                        seen_star = true;
                        if self.peek_kind() == Some(TokenKind::Comma) {
                            self.advance();
                            continue;
                        }
                        break;
                    }
                    seen_star = true;
                    ParamKind::Star
                } else {
                    ParamKind::Regular
                };

                let (ns, ne) = self.expect_name()?;
                let name = self.text_at(ns, ne).to_string();

                // Optional type annotation: disambiguate `x: type` vs `x: body`.
                // `,` alone after the tentative type is NOT enough to confirm
                // (#1582) — `lambda x: x, R(...)` inside a call-arg list is a
                // valid Python idiom where the body is `x` and `, R(...)` is
                // the next call arg. Only confirm on `:` / `=`, or on `,`
                // followed by a depth-0 body-`:` further ahead (which marks
                // a real typed-lambda continuation `lambda x: T, y: U: body`).
                let ty = if kind == ParamKind::Regular && self.peek_kind() == Some(TokenKind::Colon)
                {
                    let saved = self.pos;
                    self.advance(); // tentatively consume :
                    if let Ok(type_expr) = self.parse_type_expr() {
                        let next = self.peek_kind();
                        let confirmed = match next {
                            Some(TokenKind::Colon) | Some(TokenKind::Eq) => true,
                            Some(TokenKind::Comma) => self.has_lambda_body_colon_ahead(),
                            _ => false,
                        };
                        if confirmed {
                            type_expr
                        } else {
                            self.pos = saved; // backtrack — : is body separator
                            Spanned::new(TypeExpr::Named("Any".into()), self.span_from(p_start))
                        }
                    } else {
                        self.pos = saved; // backtrack
                        Spanned::new(TypeExpr::Named("Any".into()), self.span_from(p_start))
                    }
                } else {
                    Spanned::new(TypeExpr::Named("Any".into()), self.span_from(p_start))
                };

                let default = if self.peek_kind() == Some(TokenKind::Eq) {
                    self.advance();
                    Some(self.parse_expr()?)
                } else {
                    None
                };

                params.push(Param {
                    name,
                    ty,
                    default,
                    kind,
                    pos_only: false,
                    kw_only: seen_star && kind == ParamKind::Regular,
                    span: self.span_from(p_start),
                });

                if self.peek_kind() == Some(TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::Colon)?;
        let body = self.parse_expr()?;
        let span = Span::new(self.file_id, start, body.span.end);
        Ok(Spanned::new(
            Expr::Lambda {
                params,
                body: Box::new(body),
            },
            span,
        ))
    }

    /// Yield: `yield expr` or `yield from expr`
    pub(crate) fn parse_yield_expr(&mut self) -> crate::error::Result<Spanned<Expr>> {
        let (start, _) = self.advance(); // consume `yield`
        if self.peek_kind() == Some(TokenKind::From) {
            self.advance(); // consume `from`
            let expr = self.parse_expr()?;
            let span = Span::new(self.file_id, start, expr.span.end);
            return Ok(Spanned::new(Expr::YieldFrom(Box::new(expr)), span));
        }
        if self.peek_kind() == Some(TokenKind::Newline)
            || self.peek_kind() == Some(TokenKind::Dedent)
            || self.peek_kind() == Some(TokenKind::Eof)
            || self.peek_kind() == Some(TokenKind::RParen)
        {
            return Ok(Spanned::new(Expr::Yield(None), self.span_from(start)));
        }
        let expr = self.parse_expr()?;
        let span = Span::new(self.file_id, start, expr.span.end);
        Ok(Spanned::new(Expr::Yield(Some(Box::new(expr))), span))
    }

    /// Await: `await expr`
    pub(crate) fn parse_await_expr(&mut self) -> crate::error::Result<Spanned<Expr>> {
        let (start, _) = self.advance(); // consume `await`
        let expr = self.parse_expr()?;
        let span = Span::new(self.file_id, start, expr.span.end);
        Ok(Spanned::new(Expr::Await(Box::new(expr)), span))
    }

    /// Parenthesized expr, tuple, or generator expression.
    pub(crate) fn parse_paren_expr(&mut self) -> crate::error::Result<Spanned<Expr>> {
        let start = self.peek().unwrap().start;
        self.advance(); // consume (

        if self.peek_kind() == Some(TokenKind::RParen) {
            self.advance();
            return Ok(Spanned::new(Expr::TupleLit(vec![]), self.span_from(start)));
        }

        let first = self.parse_expr()?;

        // Generator expression: (expr for ...)
        if self.peek_kind() == Some(TokenKind::For) {
            let generators = self.parse_comprehension_clauses()?;
            self.expect(TokenKind::RParen)?;
            return Ok(Spanned::new(
                Expr::GeneratorExpr {
                    element: Box::new(first),
                    generators,
                },
                self.span_from(start),
            ));
        }

        // Tuple: (expr, ...)
        if self.peek_kind() == Some(TokenKind::Comma) {
            let mut elems = vec![first];
            while self.peek_kind() == Some(TokenKind::Comma) {
                self.advance();
                if self.peek_kind() == Some(TokenKind::RParen) {
                    break;
                }
                elems.push(self.parse_expr()?);
            }
            self.expect(TokenKind::RParen)?;
            return Ok(Spanned::new(Expr::TupleLit(elems), self.span_from(start)));
        }

        self.expect(TokenKind::RParen)?;
        Ok(first)
    }

    /// List literal or list comprehension: `[...]`
    pub(crate) fn parse_list_or_comp(&mut self) -> crate::error::Result<Spanned<Expr>> {
        let start = self.peek().unwrap().start;
        self.advance(); // consume [

        if self.peek_kind() == Some(TokenKind::RBracket) {
            self.advance();
            return Ok(Spanned::new(Expr::ListLit(vec![]), self.span_from(start)));
        }

        let first = self.parse_expr()?;

        // List comprehension: [expr for ...]
        if self.peek_kind() == Some(TokenKind::For) {
            let generators = self.parse_comprehension_clauses()?;
            self.expect(TokenKind::RBracket)?;
            return Ok(Spanned::new(
                Expr::ListComp {
                    element: Box::new(first),
                    generators,
                },
                self.span_from(start),
            ));
        }

        // Regular list
        let mut elems = vec![first];
        while self.peek_kind() == Some(TokenKind::Comma) {
            self.advance();
            if self.peek_kind() == Some(TokenKind::RBracket) {
                break;
            }
            elems.push(self.parse_expr()?);
        }
        self.expect(TokenKind::RBracket)?;
        Ok(Spanned::new(Expr::ListLit(elems), self.span_from(start)))
    }

    /// Dict/Set literal or comprehension: `{...}`
    pub(crate) fn parse_brace_expr(&mut self) -> crate::error::Result<Spanned<Expr>> {
        let start = self.peek().unwrap().start;
        self.advance(); // consume {

        // Empty dict
        if self.peek_kind() == Some(TokenKind::RBrace) {
            self.advance();
            return Ok(Spanned::new(Expr::DictLit(vec![]), self.span_from(start)));
        }

        // Dict unpack entry `{**expr, ...}` (PEP 448 / #1014)
        if self.peek_kind() == Some(TokenKind::DoubleStar) {
            self.advance();
            let val = self.parse_expr()?;
            let mut entries: Vec<(Option<Spanned<Expr>>, Spanned<Expr>)> = vec![(None, val)];
            while self.peek_kind() == Some(TokenKind::Comma) {
                self.advance();
                if self.peek_kind() == Some(TokenKind::RBrace) {
                    break;
                }
                if self.peek_kind() == Some(TokenKind::DoubleStar) {
                    self.advance();
                    let v = self.parse_expr()?;
                    entries.push((None, v));
                } else {
                    let k = self.parse_expr()?;
                    self.expect(TokenKind::Colon)?;
                    let v = self.parse_expr()?;
                    entries.push((Some(k), v));
                }
            }
            self.expect(TokenKind::RBrace)?;
            return Ok(Spanned::new(Expr::DictLit(entries), self.span_from(start)));
        }

        let first = self.parse_expr()?;

        // Dict or dict comprehension: {key: value ...}
        if self.peek_kind() == Some(TokenKind::Colon) {
            self.advance();
            let value = self.parse_expr()?;

            // Dict comprehension: {k: v for ...}
            if self.peek_kind() == Some(TokenKind::For) {
                let generators = self.parse_comprehension_clauses()?;
                self.expect(TokenKind::RBrace)?;
                return Ok(Spanned::new(
                    Expr::DictComp {
                        key: Box::new(first),
                        value: Box::new(value),
                        generators,
                    },
                    self.span_from(start),
                ));
            }

            // Regular dict (possibly mixed with **unpack entries)
            let mut pairs: Vec<(Option<Spanned<Expr>>, Spanned<Expr>)> = vec![(Some(first), value)];
            while self.peek_kind() == Some(TokenKind::Comma) {
                self.advance();
                if self.peek_kind() == Some(TokenKind::RBrace) {
                    break;
                }
                if self.peek_kind() == Some(TokenKind::DoubleStar) {
                    self.advance();
                    let v = self.parse_expr()?;
                    pairs.push((None, v));
                } else {
                    let k = self.parse_expr()?;
                    self.expect(TokenKind::Colon)?;
                    let v = self.parse_expr()?;
                    pairs.push((Some(k), v));
                }
            }
            self.expect(TokenKind::RBrace)?;
            return Ok(Spanned::new(Expr::DictLit(pairs), self.span_from(start)));
        }

        // Set comprehension: {expr for ...}
        if self.peek_kind() == Some(TokenKind::For) {
            let generators = self.parse_comprehension_clauses()?;
            self.expect(TokenKind::RBrace)?;
            return Ok(Spanned::new(
                Expr::SetComp {
                    element: Box::new(first),
                    generators,
                },
                self.span_from(start),
            ));
        }

        // Set literal: {expr, ...}
        let mut elems = vec![first];
        while self.peek_kind() == Some(TokenKind::Comma) {
            self.advance();
            if self.peek_kind() == Some(TokenKind::RBrace) {
                break;
            }
            elems.push(self.parse_expr()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(Spanned::new(Expr::SetLit(elems), self.span_from(start)))
    }

    /// Parse comprehension clauses: `for x in iter if cond ...`
    ///
    /// Supports both single-variable (`for x in …`) and tuple-target
    /// (`for k, v in …`) loop variables.
    pub(crate) fn parse_comprehension_clauses(
        &mut self,
    ) -> crate::error::Result<Vec<Comprehension>> {
        let mut generators = Vec::new();
        while self.peek_kind() == Some(TokenKind::For) || self.peek_kind() == Some(TokenKind::Async)
        {
            let is_async = if self.peek_kind() == Some(TokenKind::Async) {
                self.advance();
                true
            } else {
                false
            };
            self.expect(TokenKind::For)?;

            // Parse one or more comma-separated target names (tuple target).
            let mut targets = Vec::new();
            let (ts, te) = self.expect_name()?;
            targets.push(self.text_at(ts, te).to_string());
            while self.peek_kind() == Some(TokenKind::Comma) {
                self.advance(); // consume ','
                                // Break if we've reached `in` (trailing comma not valid here,
                                // but be lenient to avoid a confusing error).
                if self.peek_kind() == Some(TokenKind::In) {
                    break;
                }
                let (ts, te) = self.expect_name()?;
                targets.push(self.text_at(ts, te).to_string());
            }

            self.expect(TokenKind::In)?;
            let iter = self.parse_expr_bp(0)?;
            let mut conditions = Vec::new();
            while self.peek_kind() == Some(TokenKind::If) {
                self.advance();
                conditions.push(self.parse_expr_bp(0)?);
            }
            generators.push(Comprehension {
                targets,
                iter,
                conditions,
                is_async,
            });
        }
        Ok(generators)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser;
    use crate::parser::ast::*;
    use crate::source::span::FileId;

    fn fid() -> FileId {
        FileId(0)
    }
    fn parse_expr(src: &str) -> Expr {
        let full = format!("{src}\n");
        let module = parser::parse(&full, fid()).expect("parse failed");
        match module.stmts.into_iter().next().unwrap().node {
            Stmt::ExprStmt(e) => e.node,
            other => panic!("expected ExprStmt, got {other:?}"),
        }
    }

    // --- Ternary ---

    #[test]
    fn test_ternary_expr() {
        match parse_expr("a if cond else b") {
            Expr::IfExpr {
                body,
                condition,
                else_body,
            } => {
                assert!(matches!(body.node, Expr::Ident(ref n) if n == "a"));
                assert!(matches!(condition.node, Expr::Ident(ref n) if n == "cond"));
                assert!(matches!(else_body.node, Expr::Ident(ref n) if n == "b"));
            }
            other => panic!("expected IfExpr, got {other:?}"),
        }
    }

    #[test]
    fn test_ternary_with_complex_cond() {
        match parse_expr("x if a > 0 else y") {
            Expr::IfExpr { condition, .. } => {
                assert!(matches!(condition.node, Expr::BinOp { op: BinOp::Gt, .. }));
            }
            other => panic!("expected IfExpr, got {other:?}"),
        }
    }

    // --- Lambda ---

    #[test]
    fn test_lambda_no_params() {
        match parse_expr("lambda: 42") {
            Expr::Lambda { params, body } => {
                assert!(params.is_empty());
                assert!(matches!(body.node, Expr::IntLit(42)));
            }
            other => panic!("expected Lambda, got {other:?}"),
        }
    }

    #[test]
    fn test_lambda_one_param() {
        match parse_expr("lambda x: x") {
            Expr::Lambda { params, body } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "x");
                assert!(matches!(body.node, Expr::Ident(ref n) if n == "x"));
            }
            other => panic!("expected Lambda, got {other:?}"),
        }
    }

    #[test]
    fn test_lambda_multiple_params() {
        match parse_expr("lambda a, b: a") {
            Expr::Lambda { params, .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "a");
                assert_eq!(params[1].name, "b");
            }
            other => panic!("expected Lambda, got {other:?}"),
        }
    }

    #[test]
    fn test_lambda_with_default() {
        match parse_expr("lambda x=1: x") {
            Expr::Lambda { params, .. } => {
                assert_eq!(params.len(), 1);
                assert!(params[0].default.is_some());
            }
            other => panic!("expected Lambda, got {other:?}"),
        }
    }

    // --- Yield ---

    #[test]
    fn test_yield_no_value() {
        // yield at end of line
        match parse_expr("yield") {
            Expr::Yield(None) => {}
            other => panic!("expected Yield(None), got {other:?}"),
        }
    }

    #[test]
    fn test_yield_with_value() {
        match parse_expr("yield 42") {
            Expr::Yield(Some(val)) => {
                assert!(matches!(val.node, Expr::IntLit(42)));
            }
            other => panic!("expected Yield(Some), got {other:?}"),
        }
    }

    #[test]
    fn test_yield_from() {
        match parse_expr("yield from gen") {
            Expr::YieldFrom(expr) => {
                assert!(matches!(expr.node, Expr::Ident(ref n) if n == "gen"));
            }
            other => panic!("expected YieldFrom, got {other:?}"),
        }
    }

    // --- Await ---

    #[test]
    fn test_await_expr() {
        match parse_expr("await coro()") {
            Expr::Await(inner) => {
                assert!(matches!(inner.node, Expr::Call { .. }));
            }
            other => panic!("expected Await, got {other:?}"),
        }
    }

    // --- Parenthesized ---

    #[test]
    fn test_empty_tuple() {
        match parse_expr("()") {
            Expr::TupleLit(elems) => assert!(elems.is_empty()),
            other => panic!("expected TupleLit([]), got {other:?}"),
        }
    }

    #[test]
    fn test_parenthesized_expr() {
        // `(42)` should unwrap to 42
        match parse_expr("(42)") {
            Expr::IntLit(42) => {}
            other => panic!("expected IntLit(42), got {other:?}"),
        }
    }

    #[test]
    fn test_tuple_two_elements() {
        match parse_expr("(1, 2)") {
            Expr::TupleLit(elems) => {
                assert_eq!(elems.len(), 2);
                assert!(matches!(elems[0].node, Expr::IntLit(1)));
                assert!(matches!(elems[1].node, Expr::IntLit(2)));
            }
            other => panic!("expected TupleLit, got {other:?}"),
        }
    }

    #[test]
    fn test_tuple_trailing_comma() {
        match parse_expr("(1,)") {
            Expr::TupleLit(elems) => assert_eq!(elems.len(), 1),
            other => panic!("expected TupleLit, got {other:?}"),
        }
    }

    // --- List ---

    #[test]
    fn test_empty_list() {
        match parse_expr("[]") {
            Expr::ListLit(elems) => assert!(elems.is_empty()),
            other => panic!("expected ListLit([]), got {other:?}"),
        }
    }

    #[test]
    fn test_list_trailing_comma() {
        match parse_expr("[1, 2,]") {
            Expr::ListLit(elems) => assert_eq!(elems.len(), 2),
            other => panic!("expected ListLit, got {other:?}"),
        }
    }

    // --- List comprehension ---

    #[test]
    fn test_list_comp() {
        match parse_expr("[x for x in items]") {
            Expr::ListComp {
                element,
                generators,
            } => {
                assert!(matches!(element.node, Expr::Ident(ref n) if n == "x"));
                assert_eq!(generators.len(), 1);
                assert_eq!(generators[0].targets, vec!["x"]);
                assert!(generators[0].conditions.is_empty());
            }
            other => panic!("expected ListComp, got {other:?}"),
        }
    }

    #[test]
    fn test_list_comp_with_condition() {
        match parse_expr("[x for x in items if x > 0]") {
            Expr::ListComp { generators, .. } => {
                assert_eq!(generators[0].conditions.len(), 1);
            }
            other => panic!("expected ListComp, got {other:?}"),
        }
    }

    #[test]
    fn test_list_comp_tuple_target() {
        match parse_expr("[k for k, v in items]") {
            Expr::ListComp { generators, .. } => {
                assert_eq!(generators[0].targets, vec!["k", "v"]);
            }
            other => panic!("expected ListComp, got {other:?}"),
        }
    }

    // REQ: tick-132 test-coverage — PEP 530 async-for list comprehension gating gap.
    // parse_comprehension_clauses has is_async=true handling (this file, line ~328)
    // but parse_list_or_comp's gating check (line ~190) only peeks TokenKind::For,
    // not Async — so `[x async for ...]` never reaches the is_async branch and the
    // parser errors with "expected ], got async". This test LOCKS that current
    // (buggy) behavior so a future fix widening the gating check must also update
    // this test. When fixed, replace with a positive assertion of is_async=true.
    #[test]
    fn test_list_comp_async_for_currently_rejected_by_gating_gap() {
        let src = "[x async for x in aiter]\n";
        let r = crate::parser::parse(src, fid());
        assert!(
            r.is_err(),
            "gating gap: async-for in list comp should currently error"
        );
    }

    // --- Dict ---

    #[test]
    fn test_empty_dict() {
        match parse_expr("{}") {
            Expr::DictLit(pairs) => assert!(pairs.is_empty()),
            other => panic!("expected DictLit([]), got {other:?}"),
        }
    }

    #[test]
    fn test_dict_literal() {
        match parse_expr("{1: 2, 3: 4}") {
            Expr::DictLit(pairs) => {
                assert_eq!(pairs.len(), 2);
            }
            other => panic!("expected DictLit, got {other:?}"),
        }
    }

    #[test]
    fn test_dict_comp() {
        match parse_expr("{k: v for k, v in items}") {
            Expr::DictComp { generators, .. } => {
                assert_eq!(generators.len(), 1);
                assert_eq!(generators[0].targets, vec!["k", "v"]);
            }
            other => panic!("expected DictComp, got {other:?}"),
        }
    }

    // --- Set ---

    #[test]
    fn test_set_literal() {
        match parse_expr("{1, 2, 3}") {
            Expr::SetLit(elems) => assert_eq!(elems.len(), 3),
            other => panic!("expected SetLit, got {other:?}"),
        }
    }

    #[test]
    fn test_set_comp() {
        match parse_expr("{x for x in items}") {
            Expr::SetComp { generators, .. } => {
                assert_eq!(generators.len(), 1);
            }
            other => panic!("expected SetComp, got {other:?}"),
        }
    }

    // --- Generator expression ---

    #[test]
    fn test_generator_in_parens() {
        match parse_expr("(x for x in items)") {
            Expr::GeneratorExpr {
                element,
                generators,
            } => {
                assert!(matches!(element.node, Expr::Ident(ref n) if n == "x"));
                assert_eq!(generators.len(), 1);
            }
            other => panic!("expected GeneratorExpr, got {other:?}"),
        }
    }

    // --- Walrus ---

    #[test]
    fn test_walrus() {
        match parse_expr("(x := 10)") {
            Expr::Walrus { target, value } => {
                assert_eq!(target, "x");
                assert!(matches!(value.node, Expr::IntLit(10)));
            }
            other => panic!("expected Walrus, got {other:?}"),
        }
    }

    // --- Nested comprehension ---

    #[test]
    fn test_nested_comprehension() {
        match parse_expr("[x for x in a for y in b]") {
            Expr::ListComp { generators, .. } => {
                assert_eq!(generators.len(), 2);
                assert_eq!(generators[0].targets, vec!["x"]);
                assert_eq!(generators[1].targets, vec!["y"]);
            }
            other => panic!("expected ListComp, got {other:?}"),
        }
    }

    // --- Dict trailing comma ---

    #[test]
    fn test_dict_trailing_comma() {
        match parse_expr("{1: 2,}") {
            Expr::DictLit(pairs) => assert_eq!(pairs.len(), 1),
            other => panic!("expected DictLit, got {other:?}"),
        }
    }

    // --- Set trailing comma ---

    #[test]
    fn test_set_trailing_comma() {
        match parse_expr("{1, 2,}") {
            Expr::SetLit(elems) => assert_eq!(elems.len(), 2),
            other => panic!("expected SetLit, got {other:?}"),
        }
    }

    // --- Lambda *args / **kwargs ---

    #[test]
    fn test_lambda_star_args() {
        match parse_expr("lambda *args: args") {
            Expr::Lambda { params, body } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "args");
                assert_eq!(params[0].kind, ParamKind::Star);
                assert!(matches!(body.node, Expr::Ident(ref n) if n == "args"));
            }
            other => panic!("expected Lambda, got {other:?}"),
        }
    }

    #[test]
    fn test_lambda_double_star_kwargs() {
        match parse_expr("lambda **kwargs: kwargs") {
            Expr::Lambda { params, body } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "kwargs");
                assert_eq!(params[0].kind, ParamKind::DoubleStar);
                assert!(matches!(body.node, Expr::Ident(ref n) if n == "kwargs"));
            }
            other => panic!("expected Lambda, got {other:?}"),
        }
    }

    // ── R7: Dict/set literal parsing in compound expressions ──────────────

    /// Parse `{1: 'a', 2: 'b'}` as a dict literal with integer keys.
    #[test]
    fn test_dict_literal_int_keys() {
        match parse_expr("{1: 'a', 2: 'b'}") {
            Expr::DictLit(entries) => {
                assert_eq!(entries.len(), 2);
                // Each entry has (Some(key), value)
                assert!(entries[0].0.is_some());
                assert!(entries[1].0.is_some());
            }
            other => panic!("expected DictLit, got {other:?}"),
        }
    }

    /// Parse `{1, 2, 3}` as a set literal.
    #[test]
    fn test_set_literal_in_compound_context() {
        match parse_expr("{1, 2, 3}") {
            Expr::SetLit(items) => {
                assert_eq!(items.len(), 3);
                assert!(matches!(items[0].node, Expr::IntLit(1)));
                assert!(matches!(items[1].node, Expr::IntLit(2)));
                assert!(matches!(items[2].node, Expr::IntLit(3)));
            }
            other => panic!("expected SetLit, got {other:?}"),
        }
    }

    /// Parse `{'key': True}` as a single-entry dict literal.
    #[test]
    fn test_dict_literal_single_entry() {
        match parse_expr("{'key': True}") {
            Expr::DictLit(entries) => {
                assert_eq!(entries.len(), 1);
                let (ref key, ref val) = entries[0];
                assert!(key.is_some());
                assert!(matches!(val.node, Expr::BoolLit(true)));
            }
            other => panic!("expected DictLit, got {other:?}"),
        }
    }

    /// Parse dict comprehension `{k: v for k, v in items}`.
    #[test]
    fn test_dict_comp_in_compound() {
        match parse_expr("{k: v for k, v in items}") {
            Expr::DictComp { .. } => {}
            other => panic!("expected DictComp, got {other:?}"),
        }
    }

    /// Parse set comprehension `{x for x in items}`.
    #[test]
    fn test_set_comp_in_compound() {
        match parse_expr("{x for x in items}") {
            Expr::SetComp { .. } => {}
            other => panic!("expected SetComp, got {other:?}"),
        }
    }
}
