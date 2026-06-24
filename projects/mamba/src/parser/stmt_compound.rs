use super::ast::*;
use super::Parser;
use crate::error::MambaError;
use crate::lexer::token::TokenKind;
use crate::source::span::{Span, Spanned};

/// Compound statement parsers: def, class, enum, if, while, for, match,
/// try, raise, with, assert, del, global, nonlocal, type alias, decorated.
impl<'a> Parser<'a> {
    /// Parse @decorator(s) followed by def/class.
    pub(crate) fn parse_decorated(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let start = self.peek().unwrap().start;
        let mut decorators = Vec::new();
        while self.peek_kind() == Some(TokenKind::At) {
            self.advance(); // consume @
            let expr = self.parse_expr()?;
            decorators.push(expr);
            self.skip_newlines();
        }
        match self.peek_kind() {
            Some(TokenKind::Def) => self.parse_fn_def(decorators),
            Some(TokenKind::Async) => {
                self.advance(); // consume async
                self.parse_async_fn_def(decorators, start)
            }
            Some(TokenKind::Class) => self.parse_class_def(decorators),
            _ => Err(MambaError::syntax(
                self.span_from(start),
                "expected 'def', 'async def', or 'class' after decorator",
            )),
        }
    }

    pub(crate) fn parse_fn_def(
        &mut self,
        decorators: Vec<Spanned<Expr>>,
    ) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance(); // consume `def`
        let (ns, ne) = self.expect_name()?;
        let name = self.text_at(ns, ne).to_string();
        let type_params = self.parse_optional_type_params()?;
        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;
        let return_ty = if self.peek_kind() == Some(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        Ok(Spanned::new(
            Stmt::FnDef {
                decorators,
                name,
                type_params,
                params,
                return_ty,
                body,
            },
            self.span_from(start),
        ))
    }

    pub(crate) fn parse_async_stmt(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let start = self.peek().unwrap().start;
        self.advance(); // consume `async`
        match self.peek_kind() {
            Some(TokenKind::Def) => self.parse_async_fn_def(vec![], start),
            Some(TokenKind::For) => self.parse_for(true),
            Some(TokenKind::With) => self.parse_with(true),
            _ => Err(MambaError::syntax(
                self.span_from(start),
                "expected 'def', 'for', or 'with' after 'async'",
            )),
        }
    }

    pub(crate) fn parse_async_fn_def(
        &mut self,
        decorators: Vec<Spanned<Expr>>,
        start: u32,
    ) -> crate::error::Result<Spanned<Stmt>> {
        self.expect(TokenKind::Def)?;
        let (ns, ne) = self.expect_name()?;
        let name = self.text_at(ns, ne).to_string();
        let type_params = self.parse_optional_type_params()?;
        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;
        let return_ty = if self.peek_kind() == Some(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        Ok(Spanned::new(
            Stmt::AsyncFnDef {
                decorators,
                name,
                type_params,
                params,
                return_ty,
                body,
            },
            self.span_from(start),
        ))
    }

    pub(crate) fn parse_class_def(
        &mut self,
        decorators: Vec<Spanned<Expr>>,
    ) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance(); // consume `class`
        let (ns, ne) = self.expect(TokenKind::Ident)?;
        let name = self.text_at(ns, ne).to_string();
        let type_params = self.parse_optional_type_params()?;
        let (bases, keyword_args) = if self.peek_kind() == Some(TokenKind::LParen) {
            self.advance();
            let mut bases = Vec::new();
            let mut keyword_args = Vec::new();
            while self.peek_kind() != Some(TokenKind::RParen)
                && self.peek_kind() != Some(TokenKind::Eof)
            {
                // CPython grammar permits the full call-argument shape in
                // a class header — positional, keyword, `*expr`, `**expr`
                // (#1674). HIR lowering's `filter_map` over bases /
                // `find_map` over keyword_args drops these silently, so
                // accepting the syntax unblocks module-load even before
                // proper runtime semantics for class-base unpacking
                // lands (a separate enhancement).
                match self.peek_kind() {
                    Some(TokenKind::DoubleStar) => {
                        self.advance(); // consume `**`
                        let value = self.parse_expr()?;
                        keyword_args.push(("**".to_string(), value));
                    }
                    Some(TokenKind::Star) => {
                        let (sp_start, _) = self.advance(); // consume `*`
                        let inner = self.parse_expr()?;
                        let span = self.span_from(sp_start);
                        bases.push(Spanned::new(Expr::Starred(Box::new(inner)), span));
                    }
                    _ => {
                        let expr = self.parse_expr()?;
                        // Handle keyword arg: `metaclass=Metaclass`, `name="alpha"`
                        if self.peek_kind() == Some(TokenKind::Eq) {
                            self.advance(); // consume =
                            let value = self.parse_expr()?;
                            if let Expr::Ident(key_name) = &expr.node {
                                keyword_args.push((key_name.clone(), value));
                            }
                        } else {
                            bases.push(expr);
                        }
                    }
                }
                if self.peek_kind() == Some(TokenKind::Comma) {
                    self.advance();
                }
            }
            self.expect(TokenKind::RParen)?;
            (bases, keyword_args)
        } else {
            (Vec::new(), Vec::new())
        };
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        Ok(Spanned::new(
            Stmt::ClassDef {
                decorators,
                name,
                type_params,
                bases,
                keyword_args,
                body,
            },
            self.span_from(start),
        ))
    }

    pub(crate) fn parse_enum_def(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance(); // consume `enum`
        let (ns, ne) = self.expect(TokenKind::Ident)?;
        let name = self.text_at(ns, ne).to_string();
        let type_params = self.parse_optional_type_params()?;
        self.expect(TokenKind::Colon)?;
        self.skip_newlines();
        self.expect(TokenKind::Indent)?;
        let mut variants = Vec::new();
        while self.peek_kind() != Some(TokenKind::Dedent)
            && self.peek_kind() != Some(TokenKind::Eof)
        {
            self.skip_newlines();
            if self.peek_kind() == Some(TokenKind::Dedent) {
                break;
            }
            let v_start = self.peek().map(|t| t.start).unwrap_or(0);
            let (vs, ve) = self.expect(TokenKind::Ident)?;
            let v_name = self.text_at(vs, ve).to_string();
            let fields = if self.peek_kind() == Some(TokenKind::LParen) {
                self.advance();
                let params = self.parse_params()?;
                self.expect(TokenKind::RParen)?;
                params
            } else {
                Vec::new()
            };
            variants.push(Variant {
                name: v_name,
                fields,
                span: self.span_from(v_start),
            });
            self.skip_newlines();
        }
        if self.peek_kind() == Some(TokenKind::Dedent) {
            self.advance();
        }
        Ok(Spanned::new(
            Stmt::EnumDef {
                name,
                type_params,
                variants,
            },
            self.span_from(start),
        ))
    }

    pub(crate) fn parse_if(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance();
        let condition = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        let mut elif_clauses = Vec::new();
        while self.peek_kind() == Some(TokenKind::Elif) {
            self.advance();
            let cond = self.parse_expr()?;
            self.expect(TokenKind::Colon)?;
            elif_clauses.push((cond, self.parse_block()?));
        }
        let else_body = if self.peek_kind() == Some(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Spanned::new(
            Stmt::If {
                condition,
                body,
                elif_clauses,
                else_body,
            },
            self.span_from(start),
        ))
    }

    pub(crate) fn parse_while(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance();
        let condition = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        let else_body = if self.peek_kind() == Some(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Spanned::new(
            Stmt::While {
                condition,
                body,
                else_body,
            },
            self.span_from(start),
        ))
    }

    pub(crate) fn parse_for(&mut self, is_async: bool) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance(); // consume `for`
                                         // Parse comma-separated target units: each unit is either a bare
                                         // name or a parenthesized group `(a, b, ...)`.
                                         //   #1590: outer parens around the *whole* target list (single
                                         //          group) flatten directly into `targets`.
                                         //   #1594: per-unit parenthesized groups in non-leading position
                                         //          (e.g. `for (a, b), c in ...` or `for (a, b), (c, d) in ...`)
                                         //          desugar into a fresh `__for_target_N_K__` flat target
                                         //          plus a prepended `(orig...) = __for_target_N_K__` in the
                                         //          body.
        let mut targets: Vec<String> = Vec::new();
        let mut prepends: Vec<Spanned<Stmt>> = Vec::new();
        let mut units: Vec<(Option<Vec<String>>, String)> = Vec::new();
        loop {
            if self.peek_kind() == Some(TokenKind::LParen) {
                let (lp_start, _) = self.advance();
                let mut group: Vec<String> = Vec::new();
                let (s, e) = self.expect_name()?;
                group.push(self.text_at(s, e).to_string());
                while self.peek_kind() == Some(TokenKind::Comma) {
                    self.advance();
                    if self.peek_kind() == Some(TokenKind::RParen) {
                        break;
                    }
                    let (s, e) = self.expect_name()?;
                    group.push(self.text_at(s, e).to_string());
                }
                self.expect(TokenKind::RParen)?;
                units.push((
                    Some(group),
                    format!("__for_target_{}_{}__", lp_start, units.len()),
                ));
            } else {
                let (s, e) = self.expect_name()?;
                units.push((None, self.text_at(s, e).to_string()));
            }
            if self.peek_kind() == Some(TokenKind::Comma) {
                self.advance();
                if self.peek_kind() == Some(TokenKind::In) {
                    break;
                }
            } else {
                break;
            }
        }
        // Single outer-paren group with no continuation → treat as flat.
        if units.len() == 1 {
            match units.into_iter().next().unwrap() {
                (Some(group), _) => {
                    targets = group;
                }
                (None, name) => {
                    targets.push(name);
                }
            }
        } else {
            for (group, flat_name) in units {
                targets.push(flat_name.clone());
                if let Some(names) = group {
                    let span = Span::dummy();
                    let tuple_elems: Vec<Spanned<Expr>> = names
                        .into_iter()
                        .map(|n| Spanned::new(Expr::Ident(n), span))
                        .collect();
                    let tuple_target = Spanned::new(Expr::TupleLit(tuple_elems), span);
                    let tmp_ident = Spanned::new(Expr::Ident(flat_name), span);
                    prepends.push(Spanned::new(
                        Stmt::Assign {
                            target: tuple_target,
                            value: tmp_ident,
                        },
                        span,
                    ));
                }
            }
        }
        // Optional type annotation (single target only)
        let var_ty = if targets.len() == 1 && self.peek_kind() == Some(TokenKind::Colon) {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };
        self.expect(TokenKind::In)?;
        let iter = self.parse_tuple_or_expr()?;
        self.expect(TokenKind::Colon)?;
        let mut body = self.parse_block()?;
        if !prepends.is_empty() {
            let mut new_body = prepends;
            new_body.append(&mut body);
            body = new_body;
        }
        let else_body = if self.peek_kind() == Some(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else {
            None
        };
        let stmt = if is_async {
            Stmt::AsyncFor {
                targets,
                var_ty,
                iter,
                body,
                else_body,
            }
        } else {
            Stmt::For {
                targets,
                var_ty,
                iter,
                body,
                else_body,
            }
        };
        Ok(Spanned::new(stmt, self.span_from(start)))
    }

    pub(crate) fn parse_match(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance();
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        self.skip_newlines();
        self.expect(TokenKind::Indent)?;
        let mut arms = Vec::new();
        while self.peek_kind() != Some(TokenKind::Dedent)
            && self.peek_kind() != Some(TokenKind::Eof)
        {
            self.skip_newlines();
            if self.peek_kind() == Some(TokenKind::Dedent) {
                break;
            }
            let arm_start = self.peek().map(|t| t.start).unwrap_or(0);
            self.expect(TokenKind::Case)?;
            let pattern = self.parse_pattern()?;
            let guard = if self.peek_kind() == Some(TokenKind::If) {
                self.advance();
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span: self.span_from(arm_start),
            });
        }
        if self.peek_kind() == Some(TokenKind::Dedent) {
            self.advance();
        }
        if let Err(msg) = validate_match_arms(&arms) {
            return Err(crate::error::MambaError::syntax(self.span_from(start), msg));
        }
        Ok(Spanned::new(
            Stmt::Match { expr, arms },
            self.span_from(start),
        ))
    }

    pub(crate) fn parse_try(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance(); // consume `try`
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        let mut handlers = Vec::new();
        while self.peek_kind() == Some(TokenKind::Except) {
            let h_start = self.peek().unwrap().start;
            self.advance(); // consume `except`
                            // Check for `except*` (PEP 654)
            let is_star = if self.peek_kind() == Some(TokenKind::Star) {
                self.advance(); // consume `*`
                true
            } else {
                false
            };
            let (exc_type, name) = if self.peek_kind() != Some(TokenKind::Colon) {
                let exc = self.parse_expr()?;
                let name = if self.peek_kind() == Some(TokenKind::As) {
                    self.advance();
                    let (ns, ne) = self.expect_name()?;
                    Some(self.text_at(ns, ne).to_string())
                } else {
                    None
                };
                (Some(exc), name)
            } else {
                (None, None)
            };
            self.expect(TokenKind::Colon)?;
            let h_body = self.parse_block()?;
            handlers.push(ExceptHandler {
                exc_type,
                name,
                body: h_body,
                is_star,
                span: self.span_from(h_start),
            });
        }
        let else_body = if self.peek_kind() == Some(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else {
            None
        };
        let finally_body = if self.peek_kind() == Some(TokenKind::Finally) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Spanned::new(
            Stmt::Try {
                body,
                handlers,
                else_body,
                finally_body,
            },
            self.span_from(start),
        ))
    }

    pub(crate) fn parse_raise(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance();
        if self.peek_kind() == Some(TokenKind::Newline) || self.peek_kind() == Some(TokenKind::Eof)
        {
            self.skip_newlines();
            return Ok(Spanned::new(
                Stmt::Raise {
                    value: None,
                    from: None,
                },
                self.span_from(start),
            ));
        }
        let value = self.parse_expr()?;
        let from = if self.peek_kind() == Some(TokenKind::From) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.skip_newlines();
        Ok(Spanned::new(
            Stmt::Raise {
                value: Some(value),
                from,
            },
            self.span_from(start),
        ))
    }

    pub(crate) fn parse_with(&mut self, is_async: bool) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance(); // consume `with`

        // PEP 617: parenthesized with-statement `with (ctx1 as a, ctx2 as b):` (#1014)
        let parenthesized =
            self.peek_kind() == Some(TokenKind::LParen) && self.is_parenthesized_with();
        if parenthesized {
            self.advance(); // consume `(`
        }

        let mut items = Vec::new();
        // #1592: parenthesized tuple-targets in `as (a, b)` are desugared
        // by synthesizing a fresh alias and prepending an unpack assign
        // to the body. Collect those prepends here.
        let mut tuple_unpack_prepends: Vec<Spanned<Stmt>> = Vec::new();
        loop {
            // Skip newlines inside parenthesized form
            if parenthesized {
                self.skip_newlines();
            }
            // Stop at `)` in parenthesized form
            if parenthesized && self.peek_kind() == Some(TokenKind::RParen) {
                break;
            }

            let context = self.parse_expr()?;
            let alias = if self.peek_kind() == Some(TokenKind::As) {
                self.advance();
                if self.peek_kind() == Some(TokenKind::LParen) {
                    // `as (t1, t2, ...)` — each element may be any assignable
                    // target (bare name, `obj.attr`, `obj[idx]`, …) per the
                    // CPython grammar. Parse each as a full expression and
                    // desugar to a fresh temp + tuple-unpack assign that we
                    // prepend to the with-body below. Tuple-of-attr targets
                    // (e.g. `as (self.a, self.b)`) lower the same way the
                    // standalone `h.a, h.b = ...` assign does.
                    let (lp_start, _) = self.advance(); // consume `(`
                    let mut tuple_elems: Vec<Spanned<Expr>> = Vec::new();
                    tuple_elems.push(self.parse_expr()?);
                    while self.peek_kind() == Some(TokenKind::Comma) {
                        self.advance();
                        if self.peek_kind() == Some(TokenKind::RParen) {
                            break;
                        }
                        tuple_elems.push(self.parse_expr()?);
                    }
                    self.expect(TokenKind::RParen)?;
                    let tmp = format!("__with_target_{}__", lp_start);
                    let span = Span::dummy();
                    let tuple_target = Spanned::new(Expr::TupleLit(tuple_elems), span);
                    let tmp_ident = Spanned::new(Expr::Ident(tmp.clone()), span);
                    tuple_unpack_prepends.push(Spanned::new(
                        Stmt::Assign {
                            target: tuple_target,
                            value: tmp_ident,
                        },
                        span,
                    ));
                    Some(tmp)
                } else {
                    // Bare-name target (`as foo`) keeps the simple alias path;
                    // attribute / subscript targets (`as foo.bar`, `as foo[i]`)
                    // are valid in CPython and desugar to a fresh temp + a
                    // prepended `target = __tmp__` assignment, matching the
                    // tuple-unpack desugar above.
                    let is_simple_name = match self.peek_kind() {
                        Some(k) if Self::is_name_token(&k) => matches!(
                            self.tokens.get(self.pos + 1).map(|t| &t.kind),
                            Some(TokenKind::Colon)
                                | Some(TokenKind::Comma)
                                | Some(TokenKind::RParen)
                                | Some(TokenKind::Newline)
                        ),
                        _ => false,
                    };
                    if is_simple_name {
                        let (ns, ne) = self.expect_name()?;
                        Some(self.text_at(ns, ne).to_string())
                    } else {
                        let target_expr = self.parse_expr()?;
                        let span = Span::dummy();
                        let tmp = format!("__with_target_{}__", target_expr.span.start);
                        let tmp_ident = Spanned::new(Expr::Ident(tmp.clone()), span);
                        tuple_unpack_prepends.push(Spanned::new(
                            Stmt::Assign {
                                target: target_expr,
                                value: tmp_ident,
                            },
                            span,
                        ));
                        Some(tmp)
                    }
                }
            } else {
                None
            };
            items.push(WithItem { context, alias });
            if self.peek_kind() != Some(TokenKind::Comma) {
                break;
            }
            self.advance();
            // Allow trailing comma before `)`
            if parenthesized {
                self.skip_newlines();
                if self.peek_kind() == Some(TokenKind::RParen) {
                    break;
                }
            }
        }
        if parenthesized {
            self.skip_newlines();
            self.expect(TokenKind::RParen)?;
        }
        self.expect(TokenKind::Colon)?;
        let mut body = self.parse_block()?;
        // Prepend any tuple-unpack desugars (#1592) so `(a, b) = __with_target_N__`
        // runs at the start of the body and binds the user-visible names.
        if !tuple_unpack_prepends.is_empty() {
            let mut new_body = tuple_unpack_prepends;
            new_body.append(&mut body);
            body = new_body;
        }
        let stmt = if is_async {
            Stmt::AsyncWith { items, body }
        } else {
            Stmt::With { items, body }
        };
        Ok(Spanned::new(stmt, self.span_from(start)))
    }

    /// Lookahead to determine if a `(` after `with` introduces a parenthesized
    /// context-manager list (PEP 617) vs a parenthesized expression.
    ///
    /// We scan forward past the opening `(` to find either:
    ///   - A bare expression followed by `as`, `,`, or `)` → parenthesized with
    ///   - A `)` followed by `:` → empty parenthesized (unlikely but valid)
    ///   - A complex expression that looks like a single context manager → fall through
    fn is_parenthesized_with(&self) -> bool {
        // Called after `with` has already been consumed, so `self.pos` is at `(`.
        // Scan forward past the `(` and look for `as` or `,` at depth 1 before `:` or `)`.
        let mut i = self.pos; // pos is at `(`
                              // skip `(`
        if i >= self.tokens.len() || self.tokens[i].kind != TokenKind::LParen {
            return false;
        }
        i += 1;
        let mut depth = 1i32;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                    depth += 1;
                }
                TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        // Closed the outer paren. Check if `:` follows.
                        let next = i + 1;
                        if next < self.tokens.len() {
                            match &self.tokens[next].kind {
                                TokenKind::Colon | TokenKind::Newline => return true,
                                _ => {}
                            }
                        }
                        return false;
                    }
                }
                // `as` at depth 1 → definitely a parenthesized `with`
                TokenKind::As if depth == 1 => return true,
                // `,` at depth 1 → could be multiple context managers
                TokenKind::Comma if depth == 1 => return true,
                _ => {}
            }
            i += 1;
        }
        false
    }

    pub(crate) fn parse_assert(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance();
        let test = self.parse_expr()?;
        let msg = if self.peek_kind() == Some(TokenKind::Comma) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.skip_newlines();
        Ok(Spanned::new(
            Stmt::Assert { test, msg },
            self.span_from(start),
        ))
    }

    pub(crate) fn parse_del(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance();
        // Support `del a, b, c` — comma-separated delete targets.
        let target = self.parse_tuple_or_expr()?;
        self.skip_newlines();
        Ok(Spanned::new(Stmt::Del(target), self.span_from(start)))
    }

    pub(crate) fn parse_global(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance();
        let mut names = Vec::new();
        loop {
            let (ns, ne) = self.expect_name()?;
            names.push(self.text_at(ns, ne).to_string());
            if self.peek_kind() != Some(TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        self.skip_newlines();
        Ok(Spanned::new(Stmt::Global(names), self.span_from(start)))
    }

    pub(crate) fn parse_nonlocal(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance();
        let mut names = Vec::new();
        loop {
            let (ns, ne) = self.expect_name()?;
            names.push(self.text_at(ns, ne).to_string());
            if self.peek_kind() != Some(TokenKind::Comma) {
                break;
            }
            self.advance();
        }
        self.skip_newlines();
        Ok(Spanned::new(Stmt::Nonlocal(names), self.span_from(start)))
    }

    pub(crate) fn parse_type_alias(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance(); // consume `type`
        let (ns, ne) = self.expect_name()?;
        let name = self.text_at(ns, ne).to_string();
        let type_params = self.parse_optional_type_params()?;
        self.expect(TokenKind::Eq)?;
        // PEP 695: the alias value is an arbitrary expression, evaluated
        // lazily at runtime (e.g. `type Lazy[T] = lambda: T`).
        let value = self.parse_expr()?;
        self.skip_newlines();
        Ok(Spanned::new(
            Stmt::TypeAlias {
                name,
                type_params,
                value,
            },
            self.span_from(start),
        ))
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
    fn parse_stmt(src: &str) -> Stmt {
        let module = parser::parse(src, fid()).expect("parse failed");
        module.stmts.into_iter().next().unwrap().node
    }

    // --- Function def ---

    #[test]
    fn test_fn_def_basic() {
        match parse_stmt("def foo():\n    pass\n") {
            Stmt::FnDef {
                name,
                params,
                return_ty,
                body,
                decorators,
                ..
            } => {
                assert_eq!(name, "foo");
                assert!(params.is_empty());
                assert!(return_ty.is_none());
                assert!(decorators.is_empty());
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn test_fn_def_with_return_type() {
        match parse_stmt("def foo() -> int:\n    return 0\n") {
            Stmt::FnDef { return_ty, .. } => {
                let rt = return_ty.unwrap();
                assert!(matches!(rt.node, TypeExpr::Named(ref n) if n == "int"));
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn test_fn_def_with_type_params() {
        match parse_stmt("def foo[T, U]():\n    pass\n") {
            Stmt::FnDef { type_params, .. } => {
                let names: Vec<&str> = type_params.iter().map(|p| p.name.as_str()).collect();
                assert_eq!(names, vec!["T", "U"]);
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn test_fn_def_with_params() {
        match parse_stmt("def foo(a: int, b: str):\n    pass\n") {
            Stmt::FnDef { params, .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "a");
                assert_eq!(params[1].name, "b");
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    // --- Async fn def ---

    #[test]
    fn test_async_fn_def() {
        match parse_stmt("async def foo():\n    pass\n") {
            Stmt::AsyncFnDef { name, .. } => assert_eq!(name, "foo"),
            other => panic!("expected AsyncFnDef, got {other:?}"),
        }
    }

    // --- Class def ---

    #[test]
    fn test_class_def_basic() {
        match parse_stmt("class Foo:\n    pass\n") {
            Stmt::ClassDef {
                name,
                bases,
                body,
                decorators,
                ..
            } => {
                assert_eq!(name, "Foo");
                assert!(bases.is_empty());
                assert!(decorators.is_empty());
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected ClassDef, got {other:?}"),
        }
    }

    #[test]
    fn test_class_def_with_base() {
        match parse_stmt("class Foo(Bar):\n    pass\n") {
            Stmt::ClassDef { bases, .. } => {
                assert_eq!(bases.len(), 1);
            }
            other => panic!("expected ClassDef, got {other:?}"),
        }
    }

    #[test]
    fn test_class_def_multiple_bases() {
        match parse_stmt("class Foo(Bar, Baz):\n    pass\n") {
            Stmt::ClassDef { bases, .. } => {
                assert_eq!(bases.len(), 2);
            }
            other => panic!("expected ClassDef, got {other:?}"),
        }
    }

    #[test]
    fn test_class_def_with_type_params() {
        match parse_stmt("class Foo[T]:\n    pass\n") {
            Stmt::ClassDef { type_params, .. } => {
                let names: Vec<&str> = type_params.iter().map(|p| p.name.as_str()).collect();
                assert_eq!(names, vec!["T"]);
            }
            other => panic!("expected ClassDef, got {other:?}"),
        }
    }

    // --- Enum def ---

    #[test]
    fn test_enum_def_basic() {
        match parse_stmt("enum Color:\n    Red\n    Green\n    Blue\n") {
            Stmt::EnumDef { name, variants, .. } => {
                assert_eq!(name, "Color");
                assert_eq!(variants.len(), 3);
                assert_eq!(variants[0].name, "Red");
                assert_eq!(variants[1].name, "Green");
                assert_eq!(variants[2].name, "Blue");
            }
            other => panic!("expected EnumDef, got {other:?}"),
        }
    }

    #[test]
    fn test_enum_def_with_fields() {
        match parse_stmt("enum Shape:\n    Circle(r: float)\n") {
            Stmt::EnumDef { variants, .. } => {
                assert_eq!(variants.len(), 1);
                assert_eq!(variants[0].name, "Circle");
                assert_eq!(variants[0].fields.len(), 1);
                assert_eq!(variants[0].fields[0].name, "r");
            }
            other => panic!("expected EnumDef, got {other:?}"),
        }
    }

    // --- Decorators ---

    #[test]
    fn test_decorator_on_fn() {
        match parse_stmt("@my_dec\ndef foo():\n    pass\n") {
            Stmt::FnDef {
                decorators, name, ..
            } => {
                assert_eq!(decorators.len(), 1);
                assert_eq!(name, "foo");
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn test_decorator_on_class() {
        match parse_stmt("@my_dec\nclass Foo:\n    pass\n") {
            Stmt::ClassDef {
                decorators, name, ..
            } => {
                assert_eq!(decorators.len(), 1);
                assert_eq!(name, "Foo");
            }
            other => panic!("expected ClassDef, got {other:?}"),
        }
    }

    #[test]
    fn test_multiple_decorators() {
        match parse_stmt("@dec1\n@dec2\ndef foo():\n    pass\n") {
            Stmt::FnDef { decorators, .. } => {
                assert_eq!(decorators.len(), 2);
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn test_decorator_on_async_fn() {
        match parse_stmt("@my_dec\nasync def foo():\n    pass\n") {
            Stmt::AsyncFnDef {
                decorators, name, ..
            } => {
                assert_eq!(decorators.len(), 1);
                assert_eq!(name, "foo");
            }
            other => panic!("expected AsyncFnDef, got {other:?}"),
        }
    }

    // --- If ---

    #[test]
    fn test_if_basic() {
        match parse_stmt("if True:\n    pass\n") {
            Stmt::If {
                condition,
                body,
                elif_clauses,
                else_body,
            } => {
                assert!(matches!(condition.node, Expr::BoolLit(true)));
                assert_eq!(body.len(), 1);
                assert!(elif_clauses.is_empty());
                assert!(else_body.is_none());
            }
            other => panic!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn test_if_else() {
        match parse_stmt("if True:\n    pass\nelse:\n    pass\n") {
            Stmt::If { else_body, .. } => {
                assert!(else_body.is_some());
            }
            other => panic!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn test_if_elif_else() {
        let src = "if x:\n    pass\nelif y:\n    pass\nelif z:\n    pass\nelse:\n    pass\n";
        match parse_stmt(src) {
            Stmt::If {
                elif_clauses,
                else_body,
                ..
            } => {
                assert_eq!(elif_clauses.len(), 2);
                assert!(else_body.is_some());
            }
            other => panic!("expected If, got {other:?}"),
        }
    }

    // --- While ---

    #[test]
    fn test_while_basic() {
        match parse_stmt("while True:\n    pass\n") {
            Stmt::While {
                condition,
                body,
                else_body,
            } => {
                assert!(matches!(condition.node, Expr::BoolLit(true)));
                assert_eq!(body.len(), 1);
                assert!(else_body.is_none());
            }
            other => panic!("expected While, got {other:?}"),
        }
    }

    #[test]
    fn test_while_else() {
        match parse_stmt("while True:\n    pass\nelse:\n    pass\n") {
            Stmt::While { else_body, .. } => {
                assert!(else_body.is_some());
            }
            other => panic!("expected While, got {other:?}"),
        }
    }

    // --- For ---

    #[test]
    fn test_for_basic() {
        match parse_stmt("for x in items:\n    pass\n") {
            Stmt::For {
                targets,
                iter,
                body,
                else_body,
                ..
            } => {
                assert_eq!(targets, vec!["x"]);
                assert_eq!(body.len(), 1);
                assert!(else_body.is_none());
                assert!(matches!(iter.node, Expr::Ident(ref n) if n == "items"));
            }
            other => panic!("expected For, got {other:?}"),
        }
    }

    #[test]
    fn test_for_multi_target() {
        match parse_stmt("for k, v in items:\n    pass\n") {
            Stmt::For { targets, .. } => {
                assert_eq!(targets, vec!["k", "v"]);
            }
            other => panic!("expected For, got {other:?}"),
        }
    }

    #[test]
    fn test_for_else() {
        match parse_stmt("for x in items:\n    pass\nelse:\n    pass\n") {
            Stmt::For { else_body, .. } => {
                assert!(else_body.is_some());
            }
            other => panic!("expected For, got {other:?}"),
        }
    }

    #[test]
    fn test_async_for() {
        match parse_stmt("async for x in items:\n    pass\n") {
            Stmt::AsyncFor { targets, .. } => {
                assert_eq!(targets, vec!["x"]);
            }
            other => panic!("expected AsyncFor, got {other:?}"),
        }
    }

    // --- Try / Except ---

    #[test]
    fn test_try_except_basic() {
        match parse_stmt("try:\n    pass\nexcept:\n    pass\n") {
            Stmt::Try {
                body,
                handlers,
                else_body,
                finally_body,
            } => {
                assert_eq!(body.len(), 1);
                assert_eq!(handlers.len(), 1);
                assert!(handlers[0].exc_type.is_none());
                assert!(handlers[0].name.is_none());
                assert!(else_body.is_none());
                assert!(finally_body.is_none());
            }
            other => panic!("expected Try, got {other:?}"),
        }
    }

    #[test]
    fn test_try_except_typed() {
        match parse_stmt("try:\n    pass\nexcept ValueError:\n    pass\n") {
            Stmt::Try { handlers, .. } => {
                assert!(handlers[0].exc_type.is_some());
                assert!(handlers[0].name.is_none());
            }
            other => panic!("expected Try, got {other:?}"),
        }
    }

    #[test]
    fn test_try_except_as() {
        match parse_stmt("try:\n    pass\nexcept ValueError as e:\n    pass\n") {
            Stmt::Try { handlers, .. } => {
                assert!(handlers[0].exc_type.is_some());
                assert_eq!(handlers[0].name.as_deref(), Some("e"));
            }
            other => panic!("expected Try, got {other:?}"),
        }
    }

    #[test]
    fn test_try_finally() {
        match parse_stmt("try:\n    pass\nfinally:\n    pass\n") {
            Stmt::Try { finally_body, .. } => {
                assert!(finally_body.is_some());
            }
            other => panic!("expected Try, got {other:?}"),
        }
    }

    // --- Try with dict/set literals in body (#1112) ---

    #[test]
    fn test_try_empty_dict_subscript_in_body() {
        // S-TRY-DICT-1: empty dict subscript in try body
        match parse_stmt("try:\n    d = {}['x']\nexcept KeyError:\n    pass\n") {
            Stmt::Try { body, handlers, .. } => {
                assert_eq!(body.len(), 1);
                match &body[0].node {
                    Stmt::Assign { target, value } => {
                        assert!(matches!(&target.node, Expr::Ident(n) if n == "d"));
                        match &value.node {
                            Expr::Index { object, index } => {
                                assert!(
                                    matches!(&object.node, Expr::DictLit(entries) if entries.is_empty())
                                );
                                assert!(matches!(&index.node, Expr::StrLit(s) if s == "x"));
                            }
                            other => panic!("expected Index, got {other:?}"),
                        }
                    }
                    other => panic!("expected Assign, got {other:?}"),
                }
                assert_eq!(handlers.len(), 1);
                assert!(matches!(
                    &handlers[0].exc_type.as_ref().unwrap().node,
                    Expr::Ident(n) if n == "KeyError"
                ));
            }
            other => panic!("expected Try, got {other:?}"),
        }
    }

    #[test]
    fn test_try_set_method_call_in_body() {
        // S-TRY-DICT-2: set literal method call in try body
        match parse_stmt("try:\n    s = {1, 2}.remove(99)\nexcept KeyError:\n    pass\n") {
            Stmt::Try { body, handlers, .. } => {
                assert_eq!(body.len(), 1);
                match &body[0].node {
                    Stmt::Assign { target, value } => {
                        assert!(matches!(&target.node, Expr::Ident(n) if n == "s"));
                        // value should be Call(Attr(SetLit([1,2]), "remove"), [99])
                        match &value.node {
                            Expr::Call { func, args } => {
                                match &func.node {
                                    Expr::Attr { object, attr } => {
                                        match &object.node {
                                            Expr::SetLit(elems) => assert_eq!(elems.len(), 2),
                                            other => panic!("expected SetLit, got {other:?}"),
                                        }
                                        assert_eq!(attr, "remove");
                                    }
                                    other => panic!("expected Attr, got {other:?}"),
                                }
                                assert_eq!(args.len(), 1);
                            }
                            other => panic!("expected Call, got {other:?}"),
                        }
                    }
                    other => panic!("expected Assign, got {other:?}"),
                }
                assert_eq!(handlers.len(), 1);
            }
            other => panic!("expected Try, got {other:?}"),
        }
    }

    #[test]
    fn test_try_dict_literal_subscript_in_body() {
        // S-TRY-DICT-3: dict literal subscript in try body
        match parse_stmt("try:\n    x = {'a': 1}['a']\nexcept:\n    pass\n") {
            Stmt::Try { body, handlers, .. } => {
                assert_eq!(body.len(), 1);
                match &body[0].node {
                    Stmt::Assign { target, value } => {
                        assert!(matches!(&target.node, Expr::Ident(n) if n == "x"));
                        match &value.node {
                            Expr::Index { object, index } => {
                                match &object.node {
                                    Expr::DictLit(entries) => assert_eq!(entries.len(), 1),
                                    other => panic!("expected DictLit, got {other:?}"),
                                }
                                assert!(matches!(&index.node, Expr::StrLit(s) if s == "a"));
                            }
                            other => panic!("expected Index, got {other:?}"),
                        }
                    }
                    other => panic!("expected Assign, got {other:?}"),
                }
                assert_eq!(handlers.len(), 1);
                // Bare except (no type)
                assert!(handlers[0].exc_type.is_none());
            }
            other => panic!("expected Try, got {other:?}"),
        }
    }

    #[test]
    fn test_try_bare_empty_dict_in_body() {
        // Bare empty dict expression statement in try body
        match parse_stmt("try:\n    {}\nexcept:\n    pass\n") {
            Stmt::Try { body, handlers, .. } => {
                assert_eq!(body.len(), 1);
                match &body[0].node {
                    Stmt::ExprStmt(expr) => {
                        assert!(matches!(&expr.node, Expr::DictLit(entries) if entries.is_empty()));
                    }
                    other => panic!("expected ExprStmt(DictLit), got {other:?}"),
                }
                assert_eq!(handlers.len(), 1);
            }
            other => panic!("expected Try, got {other:?}"),
        }
    }

    #[test]
    fn test_if_dict_assignment_with_followup() {
        // S-TRY-DICT-4: dict in if body followed by another statement
        match parse_stmt("if True:\n    d = {}\n    print(d)\n") {
            Stmt::If { body, .. } => {
                assert_eq!(body.len(), 2);
                match &body[0].node {
                    Stmt::Assign { target, value } => {
                        assert!(matches!(&target.node, Expr::Ident(n) if n == "d"));
                        assert!(
                            matches!(&value.node, Expr::DictLit(entries) if entries.is_empty())
                        );
                    }
                    other => panic!("expected Assign, got {other:?}"),
                }
                match &body[1].node {
                    Stmt::ExprStmt(expr) => {
                        assert!(matches!(&expr.node, Expr::Call { .. }));
                    }
                    other => panic!("expected ExprStmt(Call), got {other:?}"),
                }
            }
            other => panic!("expected If, got {other:?}"),
        }
    }

    // --- Raise ---

    #[test]
    fn test_raise_bare() {
        match parse_stmt("raise\n") {
            Stmt::Raise {
                value: None,
                from: None,
            } => {}
            other => panic!("expected Raise(None), got {other:?}"),
        }
    }

    #[test]
    fn test_raise_value() {
        match parse_stmt("raise ValueError()\n") {
            Stmt::Raise {
                value: Some(_),
                from: None,
            } => {}
            other => panic!("expected Raise(value), got {other:?}"),
        }
    }

    #[test]
    fn test_raise_from() {
        match parse_stmt("raise ValueError() from e\n") {
            Stmt::Raise {
                value: Some(_),
                from: Some(f),
            } => {
                assert!(matches!(f.node, Expr::Ident(ref n) if n == "e"));
            }
            other => panic!("expected Raise from, got {other:?}"),
        }
    }

    // --- With ---

    #[test]
    fn test_with_basic() {
        match parse_stmt("with open(\"f\") as f:\n    pass\n") {
            Stmt::With { items, body } => {
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].alias.as_deref(), Some("f"));
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected With, got {other:?}"),
        }
    }

    #[test]
    fn test_with_no_alias() {
        match parse_stmt("with lock:\n    pass\n") {
            Stmt::With { items, .. } => {
                assert!(items[0].alias.is_none());
            }
            other => panic!("expected With, got {other:?}"),
        }
    }

    #[test]
    fn test_with_multiple() {
        match parse_stmt("with a as x, b as y:\n    pass\n") {
            Stmt::With { items, .. } => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].alias.as_deref(), Some("x"));
                assert_eq!(items[1].alias.as_deref(), Some("y"));
            }
            other => panic!("expected With, got {other:?}"),
        }
    }

    #[test]
    fn test_async_with() {
        match parse_stmt("async with lock:\n    pass\n") {
            Stmt::AsyncWith { items, .. } => {
                assert_eq!(items.len(), 1);
            }
            other => panic!("expected AsyncWith, got {other:?}"),
        }
    }

    // --- Assert ---

    #[test]
    fn test_assert_basic() {
        match parse_stmt("assert True\n") {
            Stmt::Assert { test, msg } => {
                assert!(matches!(test.node, Expr::BoolLit(true)));
                assert!(msg.is_none());
            }
            other => panic!("expected Assert, got {other:?}"),
        }
    }

    #[test]
    fn test_assert_with_message() {
        match parse_stmt("assert x, \"failed\"\n") {
            Stmt::Assert { msg, .. } => {
                assert!(msg.is_some());
            }
            other => panic!("expected Assert, got {other:?}"),
        }
    }

    // --- Del ---

    #[test]
    fn test_del() {
        match parse_stmt("del x\n") {
            Stmt::Del(e) => {
                assert!(matches!(e.node, Expr::Ident(ref n) if n == "x"));
            }
            other => panic!("expected Del, got {other:?}"),
        }
    }

    // --- Global / Nonlocal ---

    #[test]
    fn test_global() {
        match parse_stmt("global x, y\n") {
            Stmt::Global(names) => {
                assert_eq!(names, vec!["x", "y"]);
            }
            other => panic!("expected Global, got {other:?}"),
        }
    }

    #[test]
    fn test_nonlocal() {
        match parse_stmt("nonlocal x\n") {
            Stmt::Nonlocal(names) => {
                assert_eq!(names, vec!["x"]);
            }
            other => panic!("expected Nonlocal, got {other:?}"),
        }
    }

    // --- Type alias ---

    #[test]
    fn test_type_alias_simple() {
        match parse_stmt("type Number = int\n") {
            Stmt::TypeAlias {
                name,
                type_params,
                value,
            } => {
                assert_eq!(name, "Number");
                assert!(type_params.is_empty());
                assert!(matches!(value.node, Expr::Ident(ref n) if n == "int"));
            }
            other => panic!("expected TypeAlias, got {other:?}"),
        }
    }

    #[test]
    fn test_type_alias_union() {
        match parse_stmt("type Number = int | float\n") {
            Stmt::TypeAlias { value, .. } => {
                assert!(matches!(
                    value.node,
                    Expr::BinOp {
                        op: BinOp::BitOr,
                        ..
                    }
                ));
            }
            other => panic!("expected TypeAlias(Union), got {other:?}"),
        }
    }

    #[test]
    fn test_type_alias_with_type_params() {
        match parse_stmt("type Container[T] = list[int]\n") {
            Stmt::TypeAlias {
                name, type_params, ..
            } => {
                assert_eq!(name, "Container");
                let names: Vec<&str> = type_params.iter().map(|p| p.name.as_str()).collect();
                assert_eq!(names, vec!["T"]);
            }
            other => panic!("expected TypeAlias, got {other:?}"),
        }
    }

    // --- type used as expression (not alias) ---

    #[test]
    fn test_type_as_expr() {
        match parse_stmt("type(x)\n") {
            Stmt::ExprStmt(e) => {
                assert!(matches!(e.node, Expr::Call { .. }));
            }
            other => panic!("expected ExprStmt(Call), got {other:?}"),
        }
    }

    // --- Match (basic, tested more in pattern.rs) ---

    #[test]
    fn test_match_basic() {
        match parse_stmt("match x:\n    case 1:\n        pass\n") {
            Stmt::Match { expr, arms } => {
                assert!(matches!(expr.node, Expr::Ident(ref n) if n == "x"));
                assert_eq!(arms.len(), 1);
            }
            other => panic!("expected Match, got {other:?}"),
        }
    }

    // --- Decorator error ---

    #[test]
    fn test_decorator_on_invalid_target() {
        let result = parser::parse("@dec\npass\n", fid());
        assert!(result.is_err());
    }

    // --- Async invalid ---

    #[test]
    fn test_async_invalid_target() {
        let result = parser::parse("async pass\n", fid());
        assert!(result.is_err());
    }
}

// ── match-pattern static validation (CPython compile-time SyntaxErrors) ──

/// Literal mapping-pattern key, normalized so CPython's value-equality
/// duplicate rule holds: 0 == False == 0.0 == -0.
enum MatchKey {
    Num(f64),
    Str(String),
    NoneKey,
}

fn match_key_of(expr: &Expr) -> Option<MatchKey> {
    match expr {
        Expr::IntLit(i) => Some(MatchKey::Num(*i as f64)),
        Expr::FloatLit(f) => Some(MatchKey::Num(*f)),
        Expr::BoolLit(b) => Some(MatchKey::Num(if *b { 1.0 } else { 0.0 })),
        Expr::StrLit(s) => Some(MatchKey::Str(s.clone())),
        Expr::NoneLit => Some(MatchKey::NoneKey),
        Expr::UnaryOp {
            op: crate::parser::ast::UnaryOp::Neg,
            operand,
        } => match match_key_of(&operand.node) {
            Some(MatchKey::Num(n)) => Some(MatchKey::Num(-n)),
            _ => None,
        },
        _ => None,
    }
}

fn match_keys_equal(a: &MatchKey, b: &MatchKey) -> bool {
    match (a, b) {
        // -0.0 == 0.0 under f64 PartialEq, matching CPython 0/-0/False/0.0.
        (MatchKey::Num(x), MatchKey::Num(y)) => x == y,
        (MatchKey::Str(x), MatchKey::Str(y)) => x == y,
        (MatchKey::NoneKey, MatchKey::NoneKey) => true,
        _ => false,
    }
}

/// True when the pattern matches any subject (a bare capture/wildcard or an
/// OR whose last alternative is irrefutable, or an AS over one).
fn pattern_is_irrefutable(p: &Pattern) -> bool {
    match p {
        Pattern::Wildcard | Pattern::Binding(_) => true,
        Pattern::Or(alts) => alts
            .last()
            .map(|a| pattern_is_irrefutable(&a.node))
            .unwrap_or(false),
        Pattern::As { pattern, .. } => pattern_is_irrefutable(&pattern.node),
        _ => false,
    }
}

/// Per-pattern walk: records capture names (duplicates are SyntaxErrors),
/// checks mapping-key duplicates, class-pattern keyword repeats, and OR
/// alternation rules (same bound names per branch; only the final
/// alternative may be irrefutable).
fn validate_pattern(p: &Pattern, names: &mut Vec<String>) -> Result<(), String> {
    match p {
        Pattern::Wildcard | Pattern::Literal(_) => Ok(()),
        Pattern::Binding(n) => {
            if names.iter().any(|x| x == n) {
                return Err(format!("multiple assignments to name '{n}' in pattern"));
            }
            names.push(n.clone());
            Ok(())
        }
        Pattern::Star(name) => {
            if let Some(n) = name {
                if n != "_" {
                    if names.iter().any(|x| x == n) {
                        return Err(format!("multiple assignments to name '{n}' in pattern"));
                    }
                    names.push(n.clone());
                }
            }
            Ok(())
        }
        Pattern::As { pattern, name } => {
            if name == "_" {
                return Err("cannot use '_' as a target".to_string());
            }
            validate_pattern(&pattern.node, names)?;
            if names.iter().any(|x| x == name) {
                return Err(format!("multiple assignments to name '{name}' in pattern"));
            }
            names.push(name.clone());
            Ok(())
        }
        Pattern::Sequence(items) => {
            for it in items {
                validate_pattern(&it.node, names)?;
            }
            Ok(())
        }
        Pattern::Mapping { pairs, rest } => {
            let mut keys: Vec<MatchKey> = Vec::new();
            for (k, sub) in pairs {
                if let Some(key) = match_key_of(&k.node) {
                    if keys.iter().any(|seen| match_keys_equal(seen, &key)) {
                        return Err("mapping pattern checks duplicate key".to_string());
                    }
                    keys.push(key);
                }
                validate_pattern(&sub.node, names)?;
            }
            if let Some(r) = rest {
                if r != "_" {
                    if names.iter().any(|x| x == r) {
                        return Err(format!("multiple assignments to name '{r}' in pattern"));
                    }
                    names.push(r.clone());
                }
            }
            Ok(())
        }
        Pattern::ClassPattern { patterns, .. } => {
            let mut kw_seen: Vec<&String> = Vec::new();
            for (kw, sub) in patterns {
                if let Some(name) = kw {
                    if kw_seen.iter().any(|x| *x == name) {
                        return Err(format!("attribute name repeated in class pattern: {name}"));
                    }
                    kw_seen.push(name);
                }
                validate_pattern(&sub.node, names)?;
            }
            Ok(())
        }
        Pattern::Constructor { fields, .. } => {
            for n in fields {
                if n != "_" {
                    if names.iter().any(|x| x == n) {
                        return Err(format!("multiple assignments to name '{n}' in pattern"));
                    }
                    names.push(n.clone());
                }
            }
            Ok(())
        }
        Pattern::Or(alts) => {
            // Only the final alternative may be irrefutable.
            for a in alts.iter().take(alts.len().saturating_sub(1)) {
                if pattern_is_irrefutable(&a.node) {
                    return Err("wildcard makes remaining patterns unreachable".to_string());
                }
            }
            // Every alternative must bind the same set of names; the whole
            // OR contributes that set once to the enclosing pattern.
            let mut first_set: Option<Vec<String>> = None;
            for a in alts {
                let mut branch = Vec::new();
                validate_pattern(&a.node, &mut branch)?;
                let mut sorted = branch.clone();
                sorted.sort();
                match &first_set {
                    None => first_set = Some(sorted),
                    Some(expected) => {
                        if *expected != sorted {
                            return Err("alternative patterns bind different names".to_string());
                        }
                    }
                }
            }
            if let Some(set) = first_set {
                for n in set {
                    if names.iter().any(|x| *x == n) {
                        return Err(format!("multiple assignments to name '{n}' in pattern"));
                    }
                    names.push(n);
                }
            }
            Ok(())
        }
    }
}

/// Whole-statement validation: per-arm pattern rules plus the cross-arm
/// rule that an unguarded irrefutable case must be the last one.
fn validate_match_arms(arms: &[MatchArm]) -> Result<(), String> {
    for (idx, arm) in arms.iter().enumerate() {
        let mut names = Vec::new();
        validate_pattern(&arm.pattern.node, &mut names)?;
        if idx + 1 < arms.len() && arm.guard.is_none() && pattern_is_irrefutable(&arm.pattern.node)
        {
            return Err("wildcard makes remaining patterns unreachable".to_string());
        }
    }
    Ok(())
}
