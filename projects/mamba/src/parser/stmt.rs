use crate::error::MambaError;
use crate::lexer::token::TokenKind;
use crate::source::span::{Span, Spanned};
use super::ast::*;
use super::Parser;

impl<'a> Parser<'a> {
    /// Parse a single statement.
    pub fn parse_stmt(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        // If a previous call desugared something like `a = b = c = val` into
        // multiple per-target Assigns, hand those out first before consuming
        // any new tokens. Stored in reverse so pop() gives insertion order.
        if let Some(pending) = self.pending_stmts.pop() {
            return Ok(pending);
        }
        self.skip_newlines();
        let token = self.peek().ok_or_else(|| {
            MambaError::syntax(Span::dummy(), "unexpected end of input")
        })?;
        let start = token.start;

        match &token.kind {
            TokenKind::At => self.parse_decorated(),
            TokenKind::Def => self.parse_fn_def(vec![]),
            TokenKind::Async => self.parse_async_stmt(),
            TokenKind::Class => self.parse_class_def(vec![]),
            // `enum Foo:` (enum def) vs `enum = x` (expression/assignment)
            TokenKind::Enum => {
                if self.peek_at(1).map_or(false, |k| Self::is_name_token(k)) {
                    self.parse_enum_def()
                } else {
                    self.parse_ident_stmt()
                }
            }
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(false),
            // `match expr:` (match stmt) — `match` is a soft keyword in Python
            TokenKind::Match => {
                // match is a statement if there's a `:` at statement level after the subject.
                // Scan forward skipping balanced parens/brackets/braces to find a `:`.
                // If the token immediately after `match` is `=` or `.`, treat as ident.
                let next = self.peek_at(1);
                if next == Some(&TokenKind::Eq) || next == Some(&TokenKind::Dot) {
                    self.parse_ident_stmt()
                } else if self.match_subject_has_colon() {
                    self.parse_match()
                } else {
                    self.parse_ident_stmt()
                }
            }
            TokenKind::Return => self.parse_return(),
            TokenKind::Try => self.parse_try(),
            TokenKind::Raise => self.parse_raise(),
            TokenKind::With => self.parse_with(false),
            TokenKind::Assert => self.parse_assert(),
            TokenKind::Del => self.parse_del(),
            TokenKind::Global => self.parse_global(),
            TokenKind::Nonlocal => self.parse_nonlocal(),
            // `type X = int` (type alias) vs `type(x)` (expression)
            TokenKind::Type => {
                // Lookahead: type alias requires `type <Ident>`
                if self.peek_at(1).map_or(false, |k| Self::is_name_token(k)) {
                    self.parse_type_alias()
                } else {
                    self.parse_ident_stmt()
                }
            }
            TokenKind::Pass => {
                self.advance();
                self.skip_newlines();
                Ok(Spanned::new(Stmt::Pass, self.span_from(start)))
            }
            TokenKind::Break => {
                self.advance();
                self.skip_newlines();
                Ok(Spanned::new(Stmt::Break, self.span_from(start)))
            }
            TokenKind::Continue => {
                self.advance();
                self.skip_newlines();
                Ok(Spanned::new(Stmt::Continue, self.span_from(start)))
            }
            TokenKind::Import => self.parse_import(),
            TokenKind::From => self.parse_from_import(),
            // Brace-delimited expressions (dict/set literals) appearing as
            // the first token of a statement must route through the expression
            // parser via parse_ident_stmt(). Explicit arms prevent accidental
            // capture by future match arms and clarify that LBrace/LParen/
            // LBracket are valid statement-leading tokens (#1112).
            TokenKind::LBrace | TokenKind::LParen | TokenKind::LBracket => {
                self.parse_ident_stmt()
            }
            // Any other expression-starting token: handles assignment, augassign,
            // tuple unpacking, var decl, and bare expression statements.
            _ => self.parse_ident_stmt(),
        }
    }

    fn parse_return(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance();
        let value = if self.peek_kind() != Some(TokenKind::Newline)
            && self.peek_kind() != Some(TokenKind::Dedent)
            && self.peek_kind() != Some(TokenKind::Eof)
            && self.peek_kind() != Some(TokenKind::Semicolon)
        {
            Some(self.parse_tuple_or_expr()?)
        } else {
            None
        };
        self.skip_newlines();
        Ok(Spanned::new(Stmt::Return(value), self.span_from(start)))
    }

    fn parse_import(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance();
        let (s, e) = self.expect_name()?;
        let mut module = vec![self.text_at(s, e).to_string()];
        while self.peek_kind() == Some(TokenKind::Dot) {
            self.advance();
            let (s, e) = self.expect_name()?;
            module.push(self.text_at(s, e).to_string());
        }
        // `import X as Y` — module-level alias (#1014)
        let module_alias = if self.peek_kind() == Some(TokenKind::As) {
            self.advance();
            let (a_s, a_e) = self.expect_name()?;
            Some(self.text_at(a_s, a_e).to_string())
        } else {
            None
        };
        // `import a, b as y, c.d` — each comma-separated module is its own import
        // (Python semantics: `import a, b` == `import a; import b`). Emit the first
        // below; queue the rest via pending_stmts so every alias binds (#1014 fix).
        let mut extra: Vec<Spanned<Stmt>> = Vec::new();
        while self.peek_kind() == Some(TokenKind::Comma) {
            self.advance(); // consume `,`
            let (s, e) = self.expect_name()?;
            let mut m = vec![self.text_at(s, e).to_string()];
            while self.peek_kind() == Some(TokenKind::Dot) {
                self.advance();
                let (s, e) = self.expect_name()?;
                m.push(self.text_at(s, e).to_string());
            }
            let m_alias = if self.peek_kind() == Some(TokenKind::As) {
                self.advance();
                let (a_s, a_e) = self.expect_name()?;
                Some(self.text_at(a_s, a_e).to_string())
            } else {
                None
            };
            extra.push(Spanned::new(
                Stmt::Import { module: m, names: None, module_alias: m_alias },
                self.span_from(start),
            ));
        }
        self.skip_newlines();
        let first = Spanned::new(
            Stmt::Import { module, names: None, module_alias },
            self.span_from(start),
        );
        if !extra.is_empty() {
            // pending_stmts is drained LIFO (pop), so reverse to keep source order.
            extra.reverse();
            self.pending_stmts.extend(extra);
        }
        Ok(first)
    }

    fn parse_from_import(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let (start, _) = self.advance();
        // Handle relative imports: `from . import x`, `from .foo import x`
        let mut module = Vec::new();
        let is_relative_start = matches!(
            self.peek_kind(),
            Some(TokenKind::Dot) | Some(TokenKind::Ellipsis)
        );
        if is_relative_start {
            // relative import — dots then optional module name.
            // `...` is lexed as Ellipsis (not 3 × Dot), so handle both.
            while matches!(
                self.peek_kind(),
                Some(TokenKind::Dot) | Some(TokenKind::Ellipsis)
            ) {
                match self.peek_kind() {
                    Some(TokenKind::Ellipsis) => {
                        self.advance();
                        module.push(".".to_string());
                        module.push(".".to_string());
                        module.push(".".to_string());
                    }
                    _ => {
                        self.advance();
                        module.push(".".to_string());
                    }
                }
            }
            if self.peek_kind().as_ref().map_or(false, Self::is_name_token) {
                let (s, e) = self.expect_name()?;
                module.push(self.text_at(s, e).to_string());
                while self.peek_kind() == Some(TokenKind::Dot) {
                    self.advance();
                    let (s, e) = self.expect_name()?;
                    module.push(self.text_at(s, e).to_string());
                }
            }
        } else {
            let (s, e) = self.expect_name()?;
            module.push(self.text_at(s, e).to_string());
            while self.peek_kind() == Some(TokenKind::Dot) {
                self.advance();
                let (s, e) = self.expect_name()?;
                module.push(self.text_at(s, e).to_string());
            }
        }
        self.expect(TokenKind::Import)?;
        // Handle `from x import *`
        if self.peek_kind() == Some(TokenKind::Star) {
            self.advance();
            self.skip_newlines();
            return Ok(Spanned::new(
                Stmt::Import { module, names: Some(vec![("*".to_string(), None)]), module_alias: None },
                self.span_from(start),
            ));
        }
        // Handle `from x import (a, b, c)`
        let paren = self.peek_kind() == Some(TokenKind::LParen);
        if paren { self.advance(); }
        let mut names = Vec::new();
        loop {
            if paren && self.peek_kind() == Some(TokenKind::RParen) { break; }
            self.skip_newlines();
            let (s, e) = self.expect_name()?;
            let name = self.text_at(s, e).to_string();
            let alias = if self.peek_kind() == Some(TokenKind::As) {
                self.advance();
                let (s, e) = self.expect_name()?;
                Some(self.text_at(s, e).to_string())
            } else {
                None
            };
            names.push((name, alias));
            if self.peek_kind() != Some(TokenKind::Comma) { break; }
            self.advance();
            self.skip_newlines();
        }
        if paren { self.expect(TokenKind::RParen)?; }
        self.skip_newlines();
        Ok(Spanned::new(
            Stmt::Import { module, names: Some(names), module_alias: None },
            self.span_from(start),
        ))
    }

    /// Normalize an expression that appears in assignment-target position.
    ///
    /// Python allows a list display on the left of `=` as a sequence-unpacking
    /// target: `[*a, b] = xs` and `[a, b] = xs` mean the same as `(*a, b) = xs`
    /// / `(a, b) = xs`.  The parser produces a `ListLit` for `[...]`; rewrite it
    /// to a `TupleLit` so the existing tuple/unpack lowering (which already
    /// handles `Starred` elements) drives the unpack.  Recurses into nested
    /// targets (`[a, [b, c]] = ...`) and through `Starred` so a starred list
    /// element (`[*[x, y], z]`) is also normalized.  Non-sequence targets
    /// (`Ident`, `Index`, `Attr`, ...) are returned unchanged.
    fn normalize_assign_target(target: Spanned<Expr>) -> Spanned<Expr> {
        let span = target.span;
        match target.node {
            Expr::ListLit(elems) => {
                let elems = elems.into_iter().map(Self::normalize_assign_target).collect();
                Spanned::new(Expr::TupleLit(elems), span)
            }
            Expr::TupleLit(elems) => {
                let elems = elems.into_iter().map(Self::normalize_assign_target).collect();
                Spanned::new(Expr::TupleLit(elems), span)
            }
            Expr::Starred(inner) => {
                let inner = Self::normalize_assign_target(*inner);
                Spanned::new(Expr::Starred(Box::new(inner)), span)
            }
            other => Spanned::new(other, span),
        }
    }

    /// Parse `ident: type = expr` / `ident = expr` / `ident += expr` / expr stmt.
    fn parse_ident_stmt(&mut self) -> crate::error::Result<Spanned<Stmt>> {
        let start = self.peek().unwrap().start;
        let expr = self.parse_expr()?;

        // Variable declaration `name: type = expr`
        // or bare annotation   `name: type`           (no value, #1014)
        // Also handles attribute annotations: `self.x: int = 0` (PEP 526)
        if self.peek_kind() == Some(TokenKind::Colon) {
            if let Expr::Ident(name) = &expr.node {
                let name = name.clone();
                self.advance();
                let ty = self.parse_type_expr()?;
                if self.peek_kind() == Some(TokenKind::Eq) {
                    self.advance();
                    let value = self.parse_expr()?;
                    self.skip_newlines();
                    return Ok(Spanned::new(
                        Stmt::VarDecl { name, ty, value },
                        self.span_from(start),
                    ));
                } else {
                    // Bare annotation: `id: int` (no default value)
                    self.skip_newlines();
                    return Ok(Spanned::new(
                        Stmt::BareAnnotation { name, ty },
                        self.span_from(start),
                    ));
                }
            } else {
                // Non-simple annotation target: `self.x: int = 0` or `obj[key]: T = v`
                // Consume and discard the annotation; emit as assignment if `= value` follows.
                self.advance(); // consume `:`
                let _ty = self.parse_type_expr()?;
                if self.peek_kind() == Some(TokenKind::Eq) {
                    self.advance();
                    let value = self.parse_expr()?;
                    self.skip_newlines();
                    return Ok(Spanned::new(
                        Stmt::Assign { target: expr, value },
                        self.span_from(start),
                    ));
                } else {
                    // Bare attr annotation: `self.x: int`
                    self.skip_newlines();
                    return Ok(Spanned::new(
                        Stmt::ExprStmt(expr),
                        self.span_from(start),
                    ));
                }
            }
        }

        // Tuple unpacking: `a, b, ... = expr` (also chained: `a, b = c = expr`)
        if self.peek_kind() == Some(TokenKind::Comma) {
            let mut tuple_elems = vec![expr];
            while self.peek_kind() == Some(TokenKind::Comma) {
                self.advance();
                if self.peek_kind() == Some(TokenKind::Eq) { break; }
                tuple_elems.push(self.parse_expr()?);
            }
            if self.peek_kind() == Some(TokenKind::Eq) {
                self.advance();
                let span = self.span_from(start);
                // List displays among the comma-separated targets are
                // sequence-unpack targets (`[a, b], c = ...`); normalize them.
                let tuple_elems: Vec<_> =
                    tuple_elems.into_iter().map(Self::normalize_assign_target).collect();
                let tuple_target = Spanned::new(Expr::TupleLit(tuple_elems), span);
                let mut value = self.parse_tuple_or_expr()?;
                // Chained: `a, b = c = val` — reuse the simple-chain desugar
                // (one __chained_N__ temp, share across targets).
                let mut targets: Vec<Spanned<Expr>> = vec![tuple_target];
                while self.peek_kind() == Some(TokenKind::Eq) {
                    self.advance();
                    // Intermediate chained target may itself be a list display.
                    targets.push(Self::normalize_assign_target(value));
                    value = self.parse_tuple_or_expr()?;
                }
                self.skip_newlines();
                if targets.len() == 1 {
                    return Ok(Spanned::new(
                        Stmt::Assign { target: targets.into_iter().next().unwrap(), value },
                        span,
                    ));
                }
                let tmp_name = format!("__chained_{}__", start);
                let tmp_ident = Spanned::new(Expr::Ident(tmp_name), span);
                let mut all_stmts: Vec<Spanned<Stmt>> = Vec::with_capacity(targets.len() + 1);
                all_stmts.push(Spanned::new(
                    Stmt::Assign { target: tmp_ident.clone(), value },
                    span,
                ));
                for target in targets.into_iter() {
                    all_stmts.push(Spanned::new(
                        Stmt::Assign { target, value: tmp_ident.clone() },
                        span,
                    ));
                }
                let first = all_stmts.remove(0);
                all_stmts.reverse();
                self.pending_stmts.extend(all_stmts);
                return Ok(first);
            }
            // Not assignment — tuple expression statement
            self.skip_newlines();
            let span = self.span_from(start);
            let tuple = Spanned::new(Expr::TupleLit(tuple_elems), span);
            return Ok(Spanned::new(Stmt::ExprStmt(tuple), span));
        }

        // Augmented assignment: `target op= expr`
        if let Some(aug_op) = self.peek_aug_op() {
            self.advance();
            let value = self.parse_tuple_or_expr()?;
            self.skip_newlines();
            return Ok(Spanned::new(
                Stmt::AugAssign { target: expr, op: aug_op, value },
                self.span_from(start),
            ));
        }

        // Assignment: `target = expr` or `a = b = val` (chained)
        if self.peek_kind() == Some(TokenKind::Eq) {
            self.advance();
            let mut value = self.parse_tuple_or_expr()?;
            // Chained assignment: `a = b = val` → emit as `a = val`, ignoring middle targets
            // We keep consuming `= expr` until no more `=`.
            let mut targets = vec![expr];
            while self.peek_kind() == Some(TokenKind::Eq) {
                self.advance();
                targets.push(value);
                value = self.parse_tuple_or_expr()?;
            }
            // A list display in target position (`[*a, b] = xs`) is a
            // sequence-unpack target; normalize every target to a tuple form.
            let targets: Vec<_> =
                targets.into_iter().map(Self::normalize_assign_target).collect();
            self.skip_newlines();
            let span = self.span_from(start);
            if targets.len() == 1 {
                return Ok(Spanned::new(
                    Stmt::Assign { target: targets.into_iter().next().unwrap(), value },
                    span,
                ));
            }
            // `a = b = c = val` — CPython semantics evaluate RHS once and
            // share it across every target (so `a.append(x)` mutates what
            // `b` / `c` see). Desugar into:
            //
            //   __chained_N = val
            //   a = __chained_N
            //   b = __chained_N
            //   c = __chained_N
            //
            // so side effects in `val` happen once and mutable references
            // stay shared. The temp uses the statement's byte offset as a
            // suffix to stay unique across nested scopes.
            let tmp_name = format!("__chained_{}__", start);
            let tmp_ident = Spanned::new(Expr::Ident(tmp_name.clone()), span);
            let mut all_stmts: Vec<Spanned<Stmt>> = Vec::with_capacity(targets.len() + 1);
            all_stmts.push(Spanned::new(
                Stmt::Assign { target: tmp_ident.clone(), value },
                span,
            ));
            for target in targets.into_iter() {
                all_stmts.push(Spanned::new(
                    Stmt::Assign { target, value: tmp_ident.clone() },
                    span,
                ));
            }
            // Return the first (temp assignment); the rest go in the pending
            // buffer, reversed so pop() yields source order.
            let first = all_stmts.remove(0);
            all_stmts.reverse();
            self.pending_stmts.extend(all_stmts);
            return Ok(first);
        }

        // Expression statement
        self.skip_newlines();
        let span = expr.span;
        Ok(Spanned::new(Stmt::ExprStmt(expr), span))
    }

    /// Lookahead: starting after the `match` token (pos+1), scan forward over
    /// the subject expression (skipping balanced parens/brackets/braces) to
    /// determine if there is a `:` at the outermost nesting level.
    /// Returns `true` if a `:` is found before a newline/EOF at depth 0.
    fn match_subject_has_colon(&self) -> bool {
        let mut depth = 0usize;
        let mut i = self.pos + 1; // start after the `match` token
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                    depth += 1;
                }
                TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                }
                TokenKind::Colon if depth == 0 => return true,
                TokenKind::Newline | TokenKind::Eof if depth == 0 => return false,
                _ => {}
            }
            i += 1;
        }
        false
    }

    fn peek_aug_op(&self) -> Option<AugOp> {
        match self.peek_kind()? {
            TokenKind::PlusEq => Some(AugOp::Add),
            TokenKind::MinusEq => Some(AugOp::Sub),
            TokenKind::StarEq => Some(AugOp::Mul),
            TokenKind::SlashEq => Some(AugOp::Div),
            TokenKind::DoubleSlashEq => Some(AugOp::FloorDiv),
            TokenKind::PercentEq => Some(AugOp::Mod),
            TokenKind::DoubleStarEq => Some(AugOp::Pow),
            TokenKind::AmpEq => Some(AugOp::BitAnd),
            TokenKind::PipeEq => Some(AugOp::BitOr),
            TokenKind::CaretEq => Some(AugOp::BitXor),
            TokenKind::LShiftEq => Some(AugOp::LShift),
            TokenKind::RShiftEq => Some(AugOp::RShift),
            TokenKind::AtEq => Some(AugOp::MatMul),
            _ => None,
        }
    }

    // --- Helpers ---

    pub(crate) fn parse_params(&mut self) -> crate::error::Result<Vec<Param>> {
        let mut params = Vec::new();
        while self.peek_kind() != Some(TokenKind::RParen)
            && self.peek_kind() != Some(TokenKind::Eof)
        {
            let p_start = self.peek().map(|t| t.start).unwrap_or(0);
            // Handle `self` parameter
            if self.peek_kind() == Some(TokenKind::Self_) {
                self.advance();
                params.push(Param {
                    name: "self".to_string(),
                    ty: Spanned::new(
                        TypeExpr::Named("Self".to_string()),
                        self.span_from(p_start),
                    ),
                    default: None,
                    kind: ParamKind::Regular,
                    span: self.span_from(p_start),
                });
            } else if self.peek_kind() == Some(TokenKind::DoubleStar) {
                // **kwargs
                self.advance();
                let (ns, ne) = self.expect_name()?;
                let name = self.text_at(ns, ne).to_string();
                let ty = if self.peek_kind() == Some(TokenKind::Colon) {
                    self.advance();
                    self.parse_type_expr()?
                } else {
                    Spanned::new(TypeExpr::Named("Any".to_string()), self.span_from(p_start))
                };
                params.push(Param {
                    name, ty, default: None,
                    kind: ParamKind::DoubleStar, span: self.span_from(p_start),
                });
            } else if self.peek_kind() == Some(TokenKind::Slash) {
                // `/` positional-only separator — skip
                self.advance();
            } else if self.peek_kind() == Some(TokenKind::Star) {
                // bare `*` (keyword-only separator) vs `*args`
                self.advance();
                if !self.peek_kind().as_ref().map_or(false, Self::is_name_token) {
                    // bare `*` — keyword-only separator, no param.
                    // Consume the trailing comma so the loop can continue to
                    // the keyword-only parameters that follow.
                    if self.peek_kind() == Some(TokenKind::Comma) {
                        self.advance();
                    }
                    continue;
                }
                let (ns, ne) = self.expect_name()?;
                let name = self.text_at(ns, ne).to_string();
                let ty = if self.peek_kind() == Some(TokenKind::Colon) {
                    self.advance();
                    self.parse_type_expr()?
                } else {
                    Spanned::new(TypeExpr::Named("Any".to_string()), self.span_from(p_start))
                };
                params.push(Param {
                    name, ty, default: None,
                    kind: ParamKind::Star, span: self.span_from(p_start),
                });
            } else if self.peek_kind().as_ref().map_or(false, Self::is_name_token) {
                let (ns, ne) = self.expect_name()?;
                let name = self.text_at(ns, ne).to_string();
                let ty = if self.peek_kind() == Some(TokenKind::Colon) {
                    self.advance();
                    self.parse_type_expr()?
                } else {
                    Spanned::new(TypeExpr::Named("Any".to_string()), self.span_from(p_start))
                };
                let default = if self.peek_kind() == Some(TokenKind::Eq) {
                    self.advance();
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                params.push(Param {
                    name, ty, default,
                    kind: ParamKind::Regular, span: self.span_from(p_start),
                });
            } else {
                break;
            }
            if self.peek_kind() == Some(TokenKind::Comma) {
                self.advance();
            }
        }
        Ok(params)
    }

    pub(crate) fn parse_optional_type_params(
        &mut self,
    ) -> crate::error::Result<Vec<crate::parser::ast::TypeParam>> {
        use crate::parser::ast::{TypeParam, TypeParamKind};
        if self.peek_kind() != Some(TokenKind::LBracket) {
            return Ok(Vec::new());
        }
        self.advance();
        let mut params = Vec::new();
        while self.peek_kind() != Some(TokenKind::RBracket)
            && self.peek_kind() != Some(TokenKind::Eof)
        {
            // ParamSpec: **P
            if self.peek_kind() == Some(TokenKind::DoubleStar) {
                self.advance(); // consume **
                let (s, e) = self.expect_name()?;
                params.push(TypeParam {
                    name: self.text_at(s, e).to_string(),
                    kind: TypeParamKind::ParamSpec,
                    bound: None,
                    constraints: None,
                });
            }
            // TypeVarTuple: *Ts
            else if self.peek_kind() == Some(TokenKind::Star) {
                self.advance(); // consume *
                let (s, e) = self.expect_name()?;
                params.push(TypeParam {
                    name: self.text_at(s, e).to_string(),
                    kind: TypeParamKind::TypeVarTuple,
                    bound: None,
                    constraints: None,
                });
            }
            // Regular type param: T  or  T: bound  or  T: (c1, c2, ...)
            else {
                let (s, e) = self.expect_name()?;
                let name = self.text_at(s, e).to_string();
                let mut bound = None;
                let mut constraints = None;
                // Optional bound: T: expr  or  T: (expr1, expr2, ...)
                if self.peek_kind() == Some(TokenKind::Colon) {
                    self.advance(); // consume :
                    if self.peek_kind() == Some(TokenKind::LParen) {
                        // Constrained: T: (int, float, str)
                        self.advance(); // consume (
                        let mut items = Vec::new();
                        while self.peek_kind() != Some(TokenKind::RParen)
                            && self.peek_kind() != Some(TokenKind::Eof)
                        {
                            items.push(self.parse_expr()?);
                            if self.peek_kind() == Some(TokenKind::Comma) {
                                self.advance();
                            }
                        }
                        self.expect(TokenKind::RParen)?;
                        constraints = Some(items);
                    } else {
                        // Bounded: T: int  (any non-tuple expression)
                        bound = Some(self.parse_expr()?);
                    }
                }
                params.push(TypeParam {
                    name,
                    kind: TypeParamKind::TypeVar,
                    bound,
                    constraints,
                });
            }
            if self.peek_kind() == Some(TokenKind::Comma) {
                self.advance();
            }
        }
        self.expect(TokenKind::RBracket)?;
        Ok(params)
    }

    /// Parse an indented block or single-line suite.
    /// Python allows: `if cond: stmt` (single-line) or `if cond:\n    stmt` (block).
    pub fn parse_block(&mut self) -> crate::error::Result<Vec<Spanned<Stmt>>> {
        // Single-line suite: no newline/indent after colon
        if self.peek_kind() != Some(TokenKind::Newline)
            && self.peek_kind() != Some(TokenKind::Indent)
        {
            let stmt = self.parse_stmt()?;
            let mut stmts = vec![stmt];
            // Handle semicolons in single-line suite
            while self.peek_kind() == Some(TokenKind::Semicolon) {
                self.advance();
                while self.peek_kind() == Some(TokenKind::Semicolon) {
                    self.advance();
                }
                if self.peek_kind() == Some(TokenKind::Newline)
                    || self.peek_kind() == Some(TokenKind::Eof)
                    || self.peek_kind().is_none()
                {
                    break;
                }
                stmts.push(self.parse_stmt()?);
            }
            return Ok(stmts);
        }

        self.skip_newlines();
        self.expect(TokenKind::Indent)?;
        let mut stmts = Vec::new();
        while !self.pending_stmts.is_empty()
            || (self.peek_kind() != Some(TokenKind::Dedent)
                && self.peek_kind() != Some(TokenKind::Eof))
        {
            // Drain any desugared continuations (e.g. chained assign tail) that
            // belong to the *current* block before deciding whether to stop.
            if self.pending_stmts.is_empty() {
                self.skip_newlines();
                if self.peek_kind() == Some(TokenKind::Dedent)
                    || self.peek_kind() == Some(TokenKind::Eof)
                {
                    break;
                }
            }
            stmts.push(self.parse_stmt()?);
            // Handle semicolons inside indented blocks
            while self.peek_kind() == Some(TokenKind::Semicolon) {
                self.advance();
                while self.peek_kind() == Some(TokenKind::Semicolon) {
                    self.advance();
                }
                if self.peek_kind() == Some(TokenKind::Newline)
                    || self.peek_kind() == Some(TokenKind::Dedent)
                    || self.peek_kind() == Some(TokenKind::Eof)
                    || self.peek_kind().is_none()
                {
                    break;
                }
                stmts.push(self.parse_stmt()?);
            }
        }
        if self.peek_kind() == Some(TokenKind::Dedent) {
            self.advance();
        }
        Ok(stmts)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser;
    use crate::parser::ast::*;
    use crate::source::span::FileId;

    fn fid() -> FileId { FileId(0) }
    fn parse_stmt(src: &str) -> Stmt {
        let module = parser::parse(src, fid()).expect("parse failed");
        module.stmts.into_iter().next().unwrap().node
    }

    // --- Simple statements ---

    #[test]
    fn test_pass() {
        assert!(matches!(parse_stmt("pass\n"), Stmt::Pass));
    }

    #[test]
    fn test_break() {
        assert!(matches!(parse_stmt("break\n"), Stmt::Break));
    }

    #[test]
    fn test_continue() {
        assert!(matches!(parse_stmt("continue\n"), Stmt::Continue));
    }

    // --- Return ---

    #[test]
    fn test_return_none() {
        match parse_stmt("return\n") {
            Stmt::Return(None) => {}
            other => panic!("expected Return(None), got {other:?}"),
        }
    }

    #[test]
    fn test_return_value() {
        match parse_stmt("return 42\n") {
            Stmt::Return(Some(e)) => {
                assert!(matches!(e.node, Expr::IntLit(42)));
            }
            other => panic!("expected Return(Some(42)), got {other:?}"),
        }
    }

    #[test]
    fn test_return_tuple() {
        match parse_stmt("return 1, 2\n") {
            Stmt::Return(Some(e)) => {
                assert!(matches!(e.node, Expr::TupleLit(_)));
            }
            other => panic!("expected Return(TupleLit), got {other:?}"),
        }
    }

    // --- Import ---

    #[test]
    fn test_import_simple() {
        match parse_stmt("import os\n") {
            Stmt::Import { module, names, module_alias } => {
                assert_eq!(module, vec!["os"]);
                assert!(names.is_none());
                assert!(module_alias.is_none());
            }
            other => panic!("expected Import, got {other:?}"),
        }
    }

    #[test]
    fn test_import_dotted() {
        match parse_stmt("import os.path\n") {
            Stmt::Import { module, names, .. } => {
                assert_eq!(module, vec!["os", "path"]);
                assert!(names.is_none());
            }
            other => panic!("expected Import, got {other:?}"),
        }
    }

    #[test]
    fn test_import_alias() {
        match parse_stmt("import sys as system\n") {
            Stmt::Import { module, names, module_alias } => {
                assert_eq!(module, vec!["sys"]);
                assert!(names.is_none());
                assert_eq!(module_alias.as_deref(), Some("system"));
            }
            other => panic!("expected Import, got {other:?}"),
        }
    }

    #[test]
    fn test_from_import() {
        match parse_stmt("from os import getcwd\n") {
            Stmt::Import { module, names, .. } => {
                assert_eq!(module, vec!["os"]);
                let names = names.unwrap();
                assert_eq!(names.len(), 1);
                assert_eq!(names[0].0, "getcwd");
                assert!(names[0].1.is_none());
            }
            other => panic!("expected Import, got {other:?}"),
        }
    }

    #[test]
    fn test_from_import_alias() {
        match parse_stmt("from os import getcwd as gw\n") {
            Stmt::Import { names, .. } => {
                let names = names.unwrap();
                assert_eq!(names[0].0, "getcwd");
                assert_eq!(names[0].1.as_deref(), Some("gw"));
            }
            other => panic!("expected Import, got {other:?}"),
        }
    }

    #[test]
    fn test_from_import_star() {
        match parse_stmt("from os import *\n") {
            Stmt::Import { names, .. } => {
                let names = names.unwrap();
                assert_eq!(names[0].0, "*");
            }
            other => panic!("expected Import(*), got {other:?}"),
        }
    }

    #[test]
    fn test_from_import_relative() {
        match parse_stmt("from . import foo\n") {
            Stmt::Import { module, names, .. } => {
                assert_eq!(module[0], ".");
                assert!(names.is_some());
            }
            other => panic!("expected Import, got {other:?}"),
        }
    }

    #[test]
    fn test_from_import_parens() {
        match parse_stmt("from os import (getcwd, listdir)\n") {
            Stmt::Import { names, .. } => {
                let names = names.unwrap();
                assert_eq!(names.len(), 2);
                assert_eq!(names[0].0, "getcwd");
                assert_eq!(names[1].0, "listdir");
            }
            other => panic!("expected Import, got {other:?}"),
        }
    }

    // --- Var decl & bare annotations ---

    #[test]
    fn test_bare_annotation() {
        match parse_stmt("id: int\n") {
            Stmt::BareAnnotation { name, ty } => {
                assert_eq!(name, "id");
                assert!(matches!(ty.node, TypeExpr::Named(ref n) if n == "int"));
            }
            other => panic!("expected BareAnnotation, got {other:?}"),
        }
    }

    #[test]
    fn test_var_decl() {
        match parse_stmt("x: int = 0\n") {
            Stmt::VarDecl { name, ty, value } => {
                assert_eq!(name, "x");
                assert!(matches!(ty.node, TypeExpr::Named(ref n) if n == "int"));
                assert!(matches!(value.node, Expr::IntLit(0)));
            }
            other => panic!("expected VarDecl, got {other:?}"),
        }
    }

    // --- Assignment ---

    #[test]
    fn test_simple_assign() {
        match parse_stmt("x = 1\n") {
            Stmt::Assign { target, value } => {
                assert!(matches!(target.node, Expr::Ident(ref n) if n == "x"));
                assert!(matches!(value.node, Expr::IntLit(1)));
            }
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    #[test]
    fn test_tuple_unpack_assign() {
        match parse_stmt("a, b = 1, 2\n") {
            Stmt::Assign { target, value } => {
                assert!(matches!(target.node, Expr::TupleLit(_)));
                assert!(matches!(value.node, Expr::TupleLit(_)));
            }
            other => panic!("expected Assign(tuple), got {other:?}"),
        }
    }

    // --- Augmented assignment ---

    #[test]
    fn test_aug_assign_add() {
        match parse_stmt("x += 1\n") {
            Stmt::AugAssign { target, op, value } => {
                assert!(matches!(target.node, Expr::Ident(ref n) if n == "x"));
                assert_eq!(op, AugOp::Add);
                assert!(matches!(value.node, Expr::IntLit(1)));
            }
            other => panic!("expected AugAssign, got {other:?}"),
        }
    }

    #[test]
    fn test_aug_assign_sub() {
        match parse_stmt("x -= 1\n") {
            Stmt::AugAssign { op, .. } => assert_eq!(op, AugOp::Sub),
            other => panic!("expected AugAssign, got {other:?}"),
        }
    }

    #[test]
    fn test_aug_assign_mul() {
        match parse_stmt("x *= 2\n") {
            Stmt::AugAssign { op, .. } => assert_eq!(op, AugOp::Mul),
            other => panic!("expected AugAssign, got {other:?}"),
        }
    }

    #[test]
    fn test_aug_assign_div() {
        match parse_stmt("x /= 2\n") {
            Stmt::AugAssign { op, .. } => assert_eq!(op, AugOp::Div),
            other => panic!("expected AugAssign, got {other:?}"),
        }
    }

    #[test]
    fn test_aug_assign_floordiv() {
        match parse_stmt("x //= 2\n") {
            Stmt::AugAssign { op, .. } => assert_eq!(op, AugOp::FloorDiv),
            other => panic!("expected AugAssign, got {other:?}"),
        }
    }

    #[test]
    fn test_aug_assign_mod() {
        match parse_stmt("x %= 2\n") {
            Stmt::AugAssign { op, .. } => assert_eq!(op, AugOp::Mod),
            other => panic!("expected AugAssign, got {other:?}"),
        }
    }

    #[test]
    fn test_aug_assign_pow() {
        match parse_stmt("x **= 2\n") {
            Stmt::AugAssign { op, .. } => assert_eq!(op, AugOp::Pow),
            other => panic!("expected AugAssign, got {other:?}"),
        }
    }

    #[test]
    fn test_aug_assign_bitand() {
        match parse_stmt("x &= 2\n") {
            Stmt::AugAssign { op, .. } => assert_eq!(op, AugOp::BitAnd),
            other => panic!("expected AugAssign, got {other:?}"),
        }
    }

    #[test]
    fn test_aug_assign_bitor() {
        match parse_stmt("x |= 2\n") {
            Stmt::AugAssign { op, .. } => assert_eq!(op, AugOp::BitOr),
            other => panic!("expected AugAssign, got {other:?}"),
        }
    }

    #[test]
    fn test_aug_assign_bitxor() {
        match parse_stmt("x ^= 2\n") {
            Stmt::AugAssign { op, .. } => assert_eq!(op, AugOp::BitXor),
            other => panic!("expected AugAssign, got {other:?}"),
        }
    }

    #[test]
    fn test_aug_assign_lshift() {
        match parse_stmt("x <<= 2\n") {
            Stmt::AugAssign { op, .. } => assert_eq!(op, AugOp::LShift),
            other => panic!("expected AugAssign, got {other:?}"),
        }
    }

    #[test]
    fn test_aug_assign_rshift() {
        match parse_stmt("x >>= 2\n") {
            Stmt::AugAssign { op, .. } => assert_eq!(op, AugOp::RShift),
            other => panic!("expected AugAssign, got {other:?}"),
        }
    }

    // --- Expression statement ---

    #[test]
    fn test_expr_stmt() {
        match parse_stmt("42\n") {
            Stmt::ExprStmt(e) => {
                assert!(matches!(e.node, Expr::IntLit(42)));
            }
            other => panic!("expected ExprStmt, got {other:?}"),
        }
    }

    #[test]
    fn test_expr_stmt_call() {
        match parse_stmt("print(42)\n") {
            Stmt::ExprStmt(e) => {
                assert!(matches!(e.node, Expr::Call { .. }));
            }
            other => panic!("expected ExprStmt(Call), got {other:?}"),
        }
    }

    // --- Soft keyword disambiguation ---

    #[test]
    fn test_match_as_assignment_target() {
        match parse_stmt("match = 1\n") {
            Stmt::Assign { target, .. } => {
                assert!(matches!(target.node, Expr::Ident(ref n) if n == "match"));
            }
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    #[test]
    fn test_list_display_assignment_target_normalized_to_tuple() {
        // `[*a, b] = xs` — the list display on the LHS is a sequence-unpack
        // target and must be rewritten to a TupleLit so the existing
        // tuple/unpack lowering (which understands Starred) drives the unpack.
        match parse_stmt("[*a, b] = xs\n") {
            Stmt::Assign { target, .. } => match target.node {
                Expr::TupleLit(elems) => {
                    assert_eq!(elems.len(), 2);
                    assert!(matches!(&elems[0].node, Expr::Starred(_)));
                    assert!(matches!(&elems[1].node, Expr::Ident(n) if n == "b"));
                }
                other => panic!("expected TupleLit target, got {other:?}"),
            },
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    #[test]
    fn test_nested_list_display_target_normalized() {
        // `[a, [b, c]] = xs` — nested list display normalizes recursively.
        match parse_stmt("[a, [b, c]] = xs\n") {
            Stmt::Assign { target, .. } => match target.node {
                Expr::TupleLit(elems) => {
                    assert_eq!(elems.len(), 2);
                    assert!(matches!(&elems[1].node, Expr::TupleLit(inner) if inner.len() == 2));
                }
                other => panic!("expected TupleLit target, got {other:?}"),
            },
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    #[test]
    fn test_rhs_list_literal_not_treated_as_target() {
        // The list literal on the RHS must stay a ListLit (not normalized).
        match parse_stmt("v = [1, 2]\n") {
            Stmt::Assign { value, .. } => {
                assert!(matches!(value.node, Expr::ListLit(_)));
            }
            other => panic!("expected Assign with ListLit value, got {other:?}"),
        }
    }

    #[test]
    fn test_match_tuple_subject() {
        let src = "match (1, 2):\n    case (x, y):\n        z = x\n";
        let module = parser::parse(src, fid()).expect("parse failed");
        match module.stmts.into_iter().next().unwrap().node {
            Stmt::Match { arms, .. } => {
                assert_eq!(arms.len(), 1);
                // The case pattern should be a Sequence pattern (x, y)
                assert!(matches!(&arms[0].pattern.node, Pattern::Sequence(_)));
            }
            other => panic!("expected Match, got {other:?}"),
        }
    }

    #[test]
    fn test_enum_as_expression() {
        match parse_stmt("enum = 1\n") {
            Stmt::Assign { target, .. } => {
                assert!(matches!(target.node, Expr::Ident(ref n) if n == "enum"));
            }
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    // --- Params ---

    #[test]
    fn test_function_self_param() {
        match parse_stmt("def foo(self):\n    pass\n") {
            Stmt::FnDef { params, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "self");
                assert_eq!(params[0].kind, ParamKind::Regular);
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn test_function_star_args() {
        match parse_stmt("def foo(*args: int):\n    pass\n") {
            Stmt::FnDef { params, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "args");
                assert_eq!(params[0].kind, ParamKind::Star);
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn test_function_double_star_kwargs() {
        match parse_stmt("def foo(**kwargs: int):\n    pass\n") {
            Stmt::FnDef { params, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "kwargs");
                assert_eq!(params[0].kind, ParamKind::DoubleStar);
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn test_function_default_param() {
        match parse_stmt("def foo(x: int = 0):\n    pass\n") {
            Stmt::FnDef { params, .. } => {
                assert_eq!(params.len(), 1);
                assert!(params[0].default.is_some());
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    // --- Chained assignment ---

    #[test]
    fn test_chained_assign() {
        // `a = b = 1` desugars into `__chained_N = 1; b = __chained_N; a = __chained_N`
        // (CPython-shared-RHS semantics). parse_stmt returns the first
        // statement of the desugared chain — the temp assignment.
        match parse_stmt("a = b = 1\n") {
            Stmt::Assign { target, value } => {
                assert!(
                    matches!(target.node, Expr::Ident(ref n) if n.starts_with("__chained_")),
                    "target should be the desugared __chained_N temp",
                );
                assert!(matches!(value.node, Expr::IntLit(1)));
            }
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    // --- Assign to attribute ---

    #[test]
    fn test_assign_attr() {
        match parse_stmt("a.b = 1\n") {
            Stmt::Assign { target, .. } => {
                assert!(matches!(target.node, Expr::Attr { .. }));
            }
            other => panic!("expected Assign(Attr), got {other:?}"),
        }
    }

    // --- Assign to index ---

    #[test]
    fn test_assign_index() {
        match parse_stmt("a[0] = 1\n") {
            Stmt::Assign { target, .. } => {
                assert!(matches!(target.node, Expr::Index { .. }));
            }
            other => panic!("expected Assign(Index), got {other:?}"),
        }
    }

    // --- LBrace-leading statements (#1112) ---

    #[test]
    fn test_bare_dict_expr_stmt() {
        // Bare dict literal as expression statement
        match parse_stmt("{}\n") {
            Stmt::ExprStmt(e) => {
                assert!(matches!(e.node, Expr::DictLit(ref entries) if entries.is_empty()));
            }
            other => panic!("expected ExprStmt(DictLit), got {other:?}"),
        }
    }

    #[test]
    fn test_bare_set_expr_stmt() {
        // Bare set literal as expression statement
        match parse_stmt("{1, 2, 3}\n") {
            Stmt::ExprStmt(e) => {
                assert!(matches!(e.node, Expr::SetLit(ref elems) if elems.len() == 3));
            }
            other => panic!("expected ExprStmt(SetLit), got {other:?}"),
        }
    }

    #[test]
    fn test_dict_subscript_assign() {
        // Dict literal subscript assignment
        match parse_stmt("d = {}['x']\n") {
            Stmt::Assign { value, .. } => {
                assert!(matches!(value.node, Expr::Index { .. }));
            }
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    #[test]
    fn test_dict_literal_assign() {
        // Dict literal assignment with entries
        match parse_stmt("d = {'a': 1, 'b': 2}\n") {
            Stmt::Assign { value, .. } => {
                match value.node {
                    Expr::DictLit(entries) => assert_eq!(entries.len(), 2),
                    other => panic!("expected DictLit, got {other:?}"),
                }
            }
            other => panic!("expected Assign, got {other:?}"),
        }
    }
}
