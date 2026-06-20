use super::ast::*;
use super::Parser;
use crate::error::MambaError;
use crate::lexer::token::TokenKind;
use crate::source::span::{Span, Spanned};

/// Binding power for Pratt parsing.
fn prefix_bp(op: &UnaryOp) -> u8 {
    match op {
        UnaryOp::Pos | UnaryOp::Neg | UnaryOp::BitNot => 21,
        UnaryOp::Not => 5,
    }
}

/// Whether a binary operator is a comparison (eligible for chaining).
fn is_comparison_op(op: &BinOp) -> bool {
    matches!(
        op,
        BinOp::Lt
            | BinOp::Gt
            | BinOp::LtEq
            | BinOp::GtEq
            | BinOp::Eq
            | BinOp::NotEq
            | BinOp::Is
            | BinOp::IsNot
            | BinOp::In
            | BinOp::NotIn
    )
}

fn infix_bp(op: &BinOp) -> (u8, u8) {
    match op {
        BinOp::Or => (1, 2),
        BinOp::And => (3, 4),
        BinOp::Eq
        | BinOp::NotEq
        | BinOp::Lt
        | BinOp::Gt
        | BinOp::LtEq
        | BinOp::GtEq
        | BinOp::Is
        | BinOp::IsNot
        | BinOp::In
        | BinOp::NotIn => (7, 8),
        BinOp::BitOr => (9, 10),
        BinOp::BitXor => (11, 12),
        BinOp::BitAnd => (13, 14),
        BinOp::LShift | BinOp::RShift => (15, 16),
        BinOp::Add | BinOp::Sub => (17, 18),
        BinOp::Mul | BinOp::Div | BinOp::FloorDiv | BinOp::Mod | BinOp::MatMul => (19, 20),
        BinOp::Pow => (24, 23), // Right-associative
    }
}

impl<'a> Parser<'a> {
    /// Parse one or more comma-separated expressions into a tuple or single expr.
    /// Used for bare tuple on the RHS of assignment: `x = a, b, c`.
    pub fn parse_tuple_or_expr(&mut self) -> crate::error::Result<Spanned<Expr>> {
        let first = self.parse_expr()?;
        if self.peek_kind() != Some(TokenKind::Comma) {
            return Ok(first);
        }
        let mut elems = vec![first];
        while self.peek_kind() == Some(TokenKind::Comma) {
            self.advance();
            // Trailing comma: `x = a,` or end of statement
            if self.peek_kind() == Some(TokenKind::Newline)
                || self.peek_kind() == Some(TokenKind::Eof)
                || self.peek_kind() == Some(TokenKind::Dedent)
            {
                break;
            }
            elems.push(self.parse_expr()?);
        }
        let span = elems
            .first()
            .unwrap()
            .span
            .merge(elems.last().unwrap().span);
        Ok(Spanned::new(Expr::TupleLit(elems), span))
    }

    /// Parse an expression (top-level with yield/lambda/await/ternary/walrus).
    pub fn parse_expr(&mut self) -> crate::error::Result<Spanned<Expr>> {
        if self.peek_kind() == Some(TokenKind::Yield) {
            return self.parse_yield_expr();
        }
        if self.peek_kind() == Some(TokenKind::Lambda) {
            return self.parse_lambda();
        }
        if self.peek_kind() == Some(TokenKind::Await) {
            return self.parse_await_expr();
        }
        if self.peek_kind() == Some(TokenKind::Star) {
            let start = self.peek().unwrap().start;
            self.advance();
            let operand = self.parse_expr()?;
            let span = Span::new(self.file_id, start, operand.span.end);
            return Ok(Spanned::new(Expr::Starred(Box::new(operand)), span));
        }

        let expr = self.parse_expr_bp(0)?;

        // Ternary: `expr if cond else alt`
        if self.peek_kind() == Some(TokenKind::If) {
            return self.parse_ternary(expr);
        }
        // Walrus: `name := expr`
        if self.peek_kind() == Some(TokenKind::ColonEq) {
            if let Expr::Ident(ref name) = expr.node {
                let name = name.clone();
                let start = expr.span.start;
                self.advance();
                let value = self.parse_expr()?;
                let span = Span::new(self.file_id, start, value.span.end);
                return Ok(Spanned::new(
                    Expr::Walrus {
                        target: name,
                        value: Box::new(value),
                    },
                    span,
                ));
            }
        }
        Ok(expr)
    }

    pub(crate) fn parse_expr_bp(&mut self, min_bp: u8) -> crate::error::Result<Spanned<Expr>> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // Postfix operations (call, attr, index/slice) — loop to handle chains
            loop {
                lhs = match self.peek_kind() {
                    Some(TokenKind::LParen) => self.parse_call(lhs)?,
                    Some(TokenKind::Dot) => self.parse_attr(lhs)?,
                    Some(TokenKind::LBracket) => self.parse_index_or_slice(lhs)?,
                    _ => break,
                };
            }

            // Infix operations
            let op = match self.peek_kind() {
                Some(ref k) => match self.token_to_binop(k) {
                    Some(op) => op,
                    None => break,
                },
                None => break,
            };

            let (l_bp, r_bp) = infix_bp(&op);
            if l_bp < min_bp {
                break;
            }

            self.advance_binop(&op);
            let rhs = self.parse_expr_bp(r_bp)?;

            // Chained comparison: if this operator is a comparison and the next
            // token is also a comparison operator, collect into ChainedCompare.
            if is_comparison_op(&op) {
                if let Some(ref next_kind) = self.peek_kind() {
                    if let Some(next_op) = self.token_to_binop(next_kind) {
                        if is_comparison_op(&next_op) {
                            let (next_l_bp, _) = infix_bp(&next_op);
                            if next_l_bp >= min_bp {
                                // Start collecting a chain: operands=[lhs, rhs, ...], ops=[op, ...]
                                let mut operands = vec![lhs, rhs];
                                let mut ops = vec![op];
                                loop {
                                    let chain_op = match self.peek_kind() {
                                        Some(ref k) => match self.token_to_binop(k) {
                                            Some(o) if is_comparison_op(&o) => o,
                                            _ => break,
                                        },
                                        None => break,
                                    };
                                    let (chain_l_bp, chain_r_bp) = infix_bp(&chain_op);
                                    if chain_l_bp < min_bp {
                                        break;
                                    }
                                    self.advance_binop(&chain_op);
                                    let next_rhs = self.parse_expr_bp(chain_r_bp)?;
                                    ops.push(chain_op);
                                    operands.push(next_rhs);
                                }
                                let start = operands.first().unwrap().span;
                                let end = operands.last().unwrap().span;
                                let span = start.merge(end);
                                lhs = Spanned::new(Expr::ChainedCompare { operands, ops }, span);
                                continue;
                            }
                        }
                    }
                }
            }

            let span = lhs.span.merge(rhs.span);
            lhs = Spanned::new(
                Expr::BinOp {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            );
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> crate::error::Result<Spanned<Expr>> {
        let token = self
            .peek()
            .ok_or_else(|| MambaError::syntax(Span::dummy(), "unexpected end of input"))?;
        let start = token.start;

        match &token.kind {
            TokenKind::Int(v) => {
                let v = *v;
                self.advance();
                Ok(Spanned::new(Expr::IntLit(v), self.span_from(start)))
            }
            TokenKind::Float(v) => {
                let v = *v;
                self.advance();
                Ok(Spanned::new(Expr::FloatLit(v), self.span_from(start)))
            }
            TokenKind::Complex(v) => {
                let v = *v;
                self.advance();
                Ok(Spanned::new(Expr::ComplexLit(v), self.span_from(start)))
            }
            TokenKind::Str(v) | TokenKind::TripleStr(v) | TokenKind::RawStr(v) => {
                let mut buf = v.clone();
                self.advance();
                // PEP 3105 implicit string-literal concatenation.
                // Adjacent Str/TripleStr/RawStr tokens — Logos already
                // skipped whitespace and the IndentProcessor suppresses
                // newlines inside parens — concatenate at compile time.
                loop {
                    match self.peek_kind() {
                        Some(TokenKind::Str(s))
                        | Some(TokenKind::TripleStr(s))
                        | Some(TokenKind::RawStr(s)) => {
                            buf.push_str(&s);
                            self.advance();
                        }
                        Some(TokenKind::FStr(s)) | Some(TokenKind::RawFStr(s)) => {
                            // str + f-string promotes the whole expression
                            // to an FString whose first part is the buffered
                            // literal, followed by the f-string's parts.
                            let is_raw = matches!(self.peek_kind(), Some(TokenKind::RawFStr(_)));
                            self.advance();
                            let mut parts = vec![FStringPart::Literal(buf)];
                            parts.extend(parse_fstring_parts(&s, is_raw).map_err(|m| {
                                crate::error::MambaError::syntax(self.span_from(start), m)
                            })?);
                            // keep absorbing trailing literals/f-strings.
                            loop {
                                match self.peek_kind() {
                                    Some(TokenKind::Str(s2))
                                    | Some(TokenKind::TripleStr(s2))
                                    | Some(TokenKind::RawStr(s2)) => {
                                        parts.push(FStringPart::Literal(s2));
                                        self.advance();
                                    }
                                    Some(TokenKind::FStr(s2)) | Some(TokenKind::RawFStr(s2)) => {
                                        let raw2 =
                                            matches!(self.peek_kind(), Some(TokenKind::RawFStr(_)));
                                        self.advance();
                                        parts.extend(parse_fstring_parts(&s2, raw2).map_err(
                                            |m| {
                                                crate::error::MambaError::syntax(
                                                    self.span_from(start),
                                                    m,
                                                )
                                            },
                                        )?);
                                    }
                                    _ => break,
                                }
                            }
                            return Ok(Spanned::new(Expr::FString(parts), self.span_from(start)));
                        }
                        _ => break,
                    }
                }
                Ok(Spanned::new(Expr::StrLit(buf), self.span_from(start)))
            }
            TokenKind::ByteStr(v) => {
                let v = v.clone();
                self.advance();
                let mut acc = crate::lexer::token::apply_bytes_escapes(&v);
                // Bytes literals concatenate the same way; mixing bytes and
                // str is a Python TypeError, so we stop at non-ByteStr.
                while let Some(TokenKind::ByteStr(s)) = self.peek_kind() {
                    self.advance();
                    let next = crate::lexer::token::apply_bytes_escapes(&s);
                    acc.extend_from_slice(&next);
                }
                Ok(Spanned::new(Expr::BytesLit(acc), self.span_from(start)))
            }
            TokenKind::FStr(v) | TokenKind::RawFStr(v) => {
                let v = v.clone();
                let is_raw = matches!(self.peek_kind(), Some(TokenKind::RawFStr(_)));
                self.advance();
                let mut parts = parse_fstring_parts(&v, is_raw)
                    .map_err(|m| crate::error::MambaError::syntax(self.span_from(start), m))?;
                // f-string + (str | f-string) implicit join.
                loop {
                    match self.peek_kind() {
                        Some(TokenKind::Str(s))
                        | Some(TokenKind::TripleStr(s))
                        | Some(TokenKind::RawStr(s)) => {
                            parts.push(FStringPart::Literal(s));
                            self.advance();
                        }
                        Some(TokenKind::FStr(s)) | Some(TokenKind::RawFStr(s)) => {
                            let raw2 = matches!(self.peek_kind(), Some(TokenKind::RawFStr(_)));
                            self.advance();
                            parts.extend(parse_fstring_parts(&s, raw2).map_err(|m| {
                                crate::error::MambaError::syntax(self.span_from(start), m)
                            })?);
                        }
                        _ => break,
                    }
                }
                Ok(Spanned::new(Expr::FString(parts), self.span_from(start)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Spanned::new(Expr::BoolLit(true), self.span_from(start)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Spanned::new(Expr::BoolLit(false), self.span_from(start)))
            }
            TokenKind::None_ => {
                self.advance();
                Ok(Spanned::new(Expr::NoneLit, self.span_from(start)))
            }
            TokenKind::Ellipsis => {
                self.advance();
                Ok(Spanned::new(Expr::Ellipsis, self.span_from(start)))
            }
            TokenKind::Ident => {
                let name = self.current_text().to_string();
                self.advance();
                Ok(Spanned::new(Expr::Ident(name), self.span_from(start)))
            }
            TokenKind::Self_ => {
                self.advance();
                Ok(Spanned::new(
                    Expr::Ident("self".to_string()),
                    self.span_from(start),
                ))
            }
            TokenKind::Plus => {
                self.advance();
                let bp = prefix_bp(&UnaryOp::Pos);
                let operand = self.parse_expr_bp(bp)?;
                let span = Span::new(self.file_id, start, operand.span.end);
                Ok(Spanned::new(
                    Expr::UnaryOp {
                        op: UnaryOp::Pos,
                        operand: Box::new(operand),
                    },
                    span,
                ))
            }
            TokenKind::Minus => {
                self.advance();
                let bp = prefix_bp(&UnaryOp::Neg);
                let operand = self.parse_expr_bp(bp)?;
                let span = Span::new(self.file_id, start, operand.span.end);
                Ok(Spanned::new(
                    Expr::UnaryOp {
                        op: UnaryOp::Neg,
                        operand: Box::new(operand),
                    },
                    span,
                ))
            }
            TokenKind::Not => {
                self.advance();
                let bp = prefix_bp(&UnaryOp::Not);
                let operand = self.parse_expr_bp(bp)?;
                let span = Span::new(self.file_id, start, operand.span.end);
                Ok(Spanned::new(
                    Expr::UnaryOp {
                        op: UnaryOp::Not,
                        operand: Box::new(operand),
                    },
                    span,
                ))
            }
            TokenKind::Tilde => {
                self.advance();
                let bp = prefix_bp(&UnaryOp::BitNot);
                let operand = self.parse_expr_bp(bp)?;
                let span = Span::new(self.file_id, start, operand.span.end);
                Ok(Spanned::new(
                    Expr::UnaryOp {
                        op: UnaryOp::BitNot,
                        operand: Box::new(operand),
                    },
                    span,
                ))
            }
            // `await` in operand position (e.g. `-await f()`, `await f() * 10`).
            // It binds at the unary level — tighter than binary operators — so
            // those parse as `-(await f())` and `(await f()) * 10`. (Top-level
            // `await …` at the start of an expression is handled in parse_expr.)
            TokenKind::Await => {
                self.advance();
                let bp = prefix_bp(&UnaryOp::Neg);
                let operand = self.parse_expr_bp(bp)?;
                let span = Span::new(self.file_id, start, operand.span.end);
                Ok(Spanned::new(Expr::Await(Box::new(operand)), span))
            }
            // Type keywords usable as expressions: int(x), bool(x), etc.
            TokenKind::IntType
            | TokenKind::FloatType
            | TokenKind::BoolType
            | TokenKind::StrType
            | TokenKind::ListType
            | TokenKind::DictType
            | TokenKind::TupleType
            | TokenKind::Type
            | TokenKind::Enum
            | TokenKind::Match
            | TokenKind::Case => {
                let name = self.current_text().to_string();
                self.advance();
                Ok(Spanned::new(Expr::Ident(name), self.span_from(start)))
            }
            TokenKind::LParen => self.parse_paren_expr(),
            TokenKind::LBracket => self.parse_list_or_comp(),
            TokenKind::LBrace => self.parse_brace_expr(),
            _ => Err(MambaError::syntax(
                Span::new(self.file_id, start, token.end),
                format!("unexpected token: {}", token.kind),
            )),
        }
    }

    fn parse_call(&mut self, func: Spanned<Expr>) -> crate::error::Result<Spanned<Expr>> {
        self.expect(TokenKind::LParen)?;
        let mut args = Vec::new();
        while self.peek_kind() != Some(TokenKind::RParen)
            && self.peek_kind() != Some(TokenKind::Eof)
        {
            if self.peek_kind() == Some(TokenKind::DoubleStar) {
                self.advance();
                let expr = self.parse_expr()?;
                args.push(CallArg::DoubleStarArg(expr));
            } else if self.peek_kind() == Some(TokenKind::Star) {
                self.advance();
                let expr = self.parse_expr()?;
                args.push(CallArg::StarArg(expr));
            } else {
                let expr = self.parse_expr()?;
                // Generator expression as sole argument: `f(x for x in ...)`
                if self.peek_kind() == Some(TokenKind::For) {
                    let generators = self.parse_comprehension_clauses()?;
                    let span = expr.span;
                    let gen = Spanned::new(
                        Expr::GeneratorExpr {
                            element: Box::new(expr),
                            generators,
                        },
                        span,
                    );
                    args.push(CallArg::Positional(gen));
                    break; // generator expr is the only argument
                }
                // Check for keyword arg: `name=value`
                if self.peek_kind() == Some(TokenKind::Eq) {
                    if let Expr::Ident(name) = &expr.node {
                        let name = name.clone();
                        self.advance(); // consume =
                        let value = self.parse_expr()?;
                        args.push(CallArg::Keyword { name, value });
                    } else {
                        args.push(CallArg::Positional(expr));
                    }
                } else {
                    args.push(CallArg::Positional(expr));
                }
            }
            if self.peek_kind() != Some(TokenKind::RParen) {
                self.expect(TokenKind::Comma)?;
            }
        }
        let (end_s, end_e) = self.expect(TokenKind::RParen)?;
        let span = func.span.merge(Span::new(self.file_id, end_s, end_e));
        Ok(Spanned::new(
            Expr::Call {
                func: Box::new(func),
                args,
            },
            span,
        ))
    }

    fn parse_attr(&mut self, object: Spanned<Expr>) -> crate::error::Result<Spanned<Expr>> {
        self.expect(TokenKind::Dot)?;
        let (ns, ne) = self.expect_name()?;
        let attr = self.text_at(ns, ne).to_string();
        let span = object.span.merge(Span::new(self.file_id, ns, ne));
        Ok(Spanned::new(
            Expr::Attr {
                object: Box::new(object),
                attr,
            },
            span,
        ))
    }

    fn parse_index_or_slice(
        &mut self,
        object: Spanned<Expr>,
    ) -> crate::error::Result<Spanned<Expr>> {
        self.expect(TokenKind::LBracket)?;
        let first = self.parse_subscript_element()?;
        // Comma-separated subscript list: `m[a, b, c]` → `m[(a, b, c)]` (#1606),
        // including mixed slice + expr forms `m[a:b, c:d]` / `m[:42, ..., :24:]`
        // (#1670). Python builds an implicit tuple inside the subscript.
        if self.peek_kind() == Some(TokenKind::Comma) {
            let mut elems = vec![first];
            while self.peek_kind() == Some(TokenKind::Comma) {
                self.advance();
                if self.peek_kind() == Some(TokenKind::RBracket) {
                    break;
                }
                elems.push(self.parse_subscript_element()?);
            }
            let (end_s, end_e) = self.expect(TokenKind::RBracket)?;
            let outer = Span::new(self.file_id, end_s, end_e);
            let tuple_span = elems
                .first()
                .unwrap()
                .span
                .merge(elems.last().unwrap().span);
            let index = Spanned::new(Expr::TupleLit(elems), tuple_span);
            let span = object.span.merge(outer);
            return Ok(Spanned::new(
                Expr::Index {
                    object: Box::new(object),
                    index: Box::new(index),
                },
                span,
            ));
        }
        let (end_s, end_e) = self.expect(TokenKind::RBracket)?;
        let span = object.span.merge(Span::new(self.file_id, end_s, end_e));
        Ok(Spanned::new(
            Expr::Index {
                object: Box::new(object),
                index: Box::new(first),
            },
            span,
        ))
    }

    /// Parse one subscript element — either an `Expr::Slice` if a colon
    /// is present, or a regular expression. Stops at `,` and `]` so the
    /// caller can decide whether to wrap a sequence in a tuple.
    fn parse_subscript_element(&mut self) -> crate::error::Result<Spanned<Expr>> {
        if self.peek_kind() == Some(TokenKind::Colon) {
            return self.finish_slice_element(None);
        }
        let first = self.parse_expr()?;
        if self.peek_kind() == Some(TokenKind::Colon) {
            return self.finish_slice_element(Some(first));
        }
        Ok(first)
    }

    fn finish_slice_element(
        &mut self,
        start: Option<Spanned<Expr>>,
    ) -> crate::error::Result<Spanned<Expr>> {
        let span_start = start
            .as_ref()
            .map(|s| s.span.start)
            .unwrap_or_else(|| self.tokens[self.pos].start);
        let (_colon_s, colon_e) = self.expect(TokenKind::Colon)?;
        let is_terminator =
            |k: &TokenKind| matches!(k, TokenKind::Colon | TokenKind::Comma | TokenKind::RBracket);
        let stop = if !self.peek_kind().as_ref().map_or(true, is_terminator) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        let step = if self.peek_kind() == Some(TokenKind::Colon) {
            self.advance();
            let is_post = |k: &TokenKind| matches!(k, TokenKind::Comma | TokenKind::RBracket);
            if !self.peek_kind().as_ref().map_or(true, is_post) {
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            }
        } else {
            None
        };
        let span_end = self
            .tokens
            .get(self.pos.saturating_sub(1))
            .map(|t| t.end)
            .unwrap_or(colon_e);
        Ok(Spanned::new(
            Expr::Slice {
                start: start.map(Box::new),
                stop,
                step,
            },
            Span::new(self.file_id, span_start, span_end),
        ))
    }

    // --- Lookahead helpers for multi-token operators ---

    pub(crate) fn peek_at(&self, offset: usize) -> Option<&TokenKind> {
        self.tokens.get(self.pos + offset).map(|t| &t.kind)
    }

    fn token_to_binop(&self, kind: &TokenKind) -> Option<BinOp> {
        match kind {
            TokenKind::Plus => Some(BinOp::Add),
            TokenKind::Minus => Some(BinOp::Sub),
            TokenKind::Star => Some(BinOp::Mul),
            TokenKind::Slash => Some(BinOp::Div),
            TokenKind::DoubleSlash => Some(BinOp::FloorDiv),
            TokenKind::Percent => Some(BinOp::Mod),
            TokenKind::DoubleStar => Some(BinOp::Pow),
            TokenKind::At => Some(BinOp::MatMul),
            TokenKind::EqEq => Some(BinOp::Eq),
            TokenKind::NotEq => Some(BinOp::NotEq),
            TokenKind::Lt => Some(BinOp::Lt),
            TokenKind::Gt => Some(BinOp::Gt),
            TokenKind::LtEq => Some(BinOp::LtEq),
            TokenKind::GtEq => Some(BinOp::GtEq),
            TokenKind::And => Some(BinOp::And),
            TokenKind::Or => Some(BinOp::Or),
            TokenKind::Amp => Some(BinOp::BitAnd),
            TokenKind::Pipe => Some(BinOp::BitOr),
            TokenKind::Caret => Some(BinOp::BitXor),
            TokenKind::LShift => Some(BinOp::LShift),
            TokenKind::RShift => Some(BinOp::RShift),
            TokenKind::In => Some(BinOp::In),
            TokenKind::Is => {
                if self.peek_at(1) == Some(&TokenKind::Not) {
                    Some(BinOp::IsNot)
                } else {
                    Some(BinOp::Is)
                }
            }
            TokenKind::Not => {
                if self.peek_at(1) == Some(&TokenKind::In) {
                    Some(BinOp::NotIn)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn advance_binop(&mut self, op: &BinOp) {
        match op {
            BinOp::IsNot | BinOp::NotIn => {
                self.advance();
                self.advance();
            }
            _ => {
                self.advance();
            }
        }
    }
}

/// Process backslash escapes in an f-string literal run unless the f-string is
/// raw (`rf'...'` / `fr'...'`).  `{{` / `}}` have already been collapsed by the
/// caller; only true backslash escapes (`\n`, `\t`, `\\`, ...) remain.
fn finish_fstring_literal(lit: String, is_raw: bool) -> String {
    if is_raw {
        lit
    } else {
        crate::lexer::token::apply_escape_sequences(&lit)
    }
}

/// Parse f-string content into literal/expression parts.  `is_raw` carries the
/// `r` prefix (`rf'...'` / `fr'...'`): when set, backslash escapes in the
/// literal runs are kept verbatim.
fn parse_fstring_parts(content: &str, is_raw: bool) -> Result<Vec<FStringPart>, String> {
    let mut parts = Vec::new();
    // Accumulate the literal run as raw bytes so multi-byte UTF-8 characters
    // survive intact; decode to a `String` (lossily, though `content` is valid
    // UTF-8 so this never substitutes) only when flushing the run.
    let mut lit: Vec<u8> = Vec::new();
    let flush_lit = |lit: &mut Vec<u8>| String::from_utf8_lossy(&std::mem::take(lit)).into_owned();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if !is_raw && bytes[i] == b'\\' {
            // Non-raw f-string backslash escape. The braces of a `\N{NAME}`
            // named-unicode escape are part of the escape and must NOT be
            // treated as replacement-field delimiters (PEP-701 + CPython 3.12).
            // Copy the escape bytes verbatim into the literal run; the actual
            // escape resolution happens later in `apply_escape_sequences`.
            if i + 2 < bytes.len() && bytes[i + 1] == b'N' && bytes[i + 2] == b'{' {
                // Consume `\N{...}` through the closing `}` (or end of input).
                lit.push(b'\\');
                lit.push(b'N');
                lit.push(b'{');
                i += 3;
                while i < bytes.len() {
                    let b = bytes[i];
                    lit.push(b);
                    i += 1;
                    if b == b'}' {
                        break;
                    }
                }
            } else if i + 1 < bytes.len() && bytes[i + 1] != b'{' && bytes[i + 1] != b'}' {
                // Ordinary escape (`\n`, `\\`, `\xHH`, ...): consume the
                // backslash and the following byte as a pair so that an escaped
                // backslash (`\\`) does not leave a stray backslash in front of
                // a real replacement field. A backslash before `{`/`}` is not a
                // valid escape, so the brace is left for field handling below.
                lit.push(b'\\');
                lit.push(bytes[i + 1]);
                i += 2;
            } else {
                lit.push(b'\\');
                i += 1;
            }
        } else if bytes[i] == b'{' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            lit.push(b'{');
            i += 2;
        } else if bytes[i] == b'}' && i + 1 < bytes.len() && bytes[i + 1] == b'}' {
            lit.push(b'}');
            i += 2;
        } else if bytes[i] == b'{' {
            if !lit.is_empty() {
                parts.push(FStringPart::Literal(finish_fstring_literal(
                    flush_lit(&mut lit),
                    is_raw,
                )));
            }
            i += 1;
            let start = i;
            let mut depth = 1i32;
            // Use a stack to track nested strings so that `{` and `}` inside
            // string literals don't confuse the brace counter (PEP-701).
            let mut str_stack: Vec<u8> = Vec::new();
            while i < bytes.len() && depth > 0 {
                let b = bytes[i];
                if str_stack.is_empty() {
                    // Outside any nested string
                    if b == b'{' {
                        depth += 1;
                        i += 1;
                    } else if b == b'}' {
                        depth -= 1;
                        if depth > 0 {
                            i += 1;
                        }
                    } else if b == b'\'' || b == b'"' {
                        str_stack.push(b);
                        i += 1;
                    } else if b == b'\\' {
                        // Skip escaped character
                        i += if i + 1 < bytes.len() { 2 } else { 1 };
                    } else {
                        i += 1;
                    }
                } else {
                    let quote = *str_stack.last().unwrap();
                    if b == b'\\' {
                        // Skip escaped character inside string
                        i += if i + 1 < bytes.len() { 2 } else { 1 };
                    } else if b == quote {
                        str_stack.pop();
                        i += 1;
                    } else {
                        // Any other byte — including the OTHER quote kind —
                        // is plain string content (`f"""{"eric's"}"""`).
                        i += 1;
                    }
                }
            }
            if depth > 0 {
                return Err("f-string: expecting '}'".to_string());
            }
            let raw = &content[start..i];
            i += 1; // skip closing }
                    // Split on first top-level `:` to separate expression from format spec.
                    // We must not split inside brackets/parens/strings (e.g., `d['key']`).
            let (expr_with_conv, format_spec) = split_expr_and_spec(raw);
            // Peel off `!r`, `!s`, or `!a` conversion suffix at top level.
            let (expr_str0, conversion) = split_expr_and_conversion(expr_with_conv);
            // `=` self-documenting form (PEP 498 / 3.8 debug specifier):
            // `{expr=}`, `{expr = }`, `{expr=!r}`, `{expr=:spec}`. The text up
            // to and including the `=` (plus surrounding whitespace) is echoed
            // verbatim, then the value follows — repr by default unless an
            // explicit conversion or spec is present.
            let trimmed = expr_str0.trim_end();
            let is_debug_eq = trimmed.ends_with('=')
                && !trimmed.ends_with("==")
                && !trimmed.ends_with("!=")
                && !trimmed.ends_with("<=")
                && !trimmed.ends_with(">=")
                && !trimmed.ends_with(":=");
            let (expr_str, debug_echo) = if is_debug_eq {
                (
                    trimmed[..trimmed.len() - 1].trim_end(),
                    Some(expr_str0.to_string()),
                )
            } else {
                (expr_str0, None)
            };
            if expr_str.trim().is_empty() {
                return Err("f-string: valid expression required before '}'".to_string());
            }
            // Detect nested f-strings: if the entire expression is `f"..."` or
            // `f'...'`, recursively invoke parse_fstring_parts on the inner
            // content for more direct handling (PEP-701 nested f-strings).
            let mut expr_node = if let Some(inner) = strip_fstring_literal(expr_str) {
                // The nested literal is `f"..."`/`f'...'` (strip_fstring_literal
                // only matches the bare `f` prefix), so it is never raw.
                let inner_parts = parse_fstring_parts(inner, false)?;
                Expr::FString(inner_parts)
            } else {
                parse_fstring_expr(expr_str)
            };
            if matches!(expr_node, Expr::Starred(_)) || expr_str.trim_start().starts_with('*') {
                return Err("f-string: cannot use starred expression here".to_string());
            }
            let span = Span::dummy();
            if let Some(echo) = debug_echo {
                parts.push(FStringPart::Literal(echo));
            }
            // Apply conversion by wrapping the expression in a call to the
            // corresponding builtin. `!s` is a no-op because the default
            // conversion already calls str(). `!r` and `!a` both go through
            // repr() — mamba doesn't distinguish ASCII yet. The `=` debug
            // form defaults to repr when no conversion or spec is given.
            let effective_conv = match conversion {
                Some(c) => Some(c),
                None if is_debug_eq && format_spec.is_none() => Some('r'),
                None => None,
            };
            if let Some(conv) = effective_conv {
                if conv == 'r' || conv == 'a' {
                    // `!r` -> repr(); `!a` -> ascii() (backslash-escapes non-ASCII).
                    let fname = if conv == 'a' { "ascii" } else { "repr" };
                    let inner = Spanned::new(expr_node, span);
                    expr_node = Expr::Call {
                        func: Box::new(Spanned::new(Expr::Ident(fname.to_string()), span)),
                        args: vec![CallArg::Positional(inner)],
                    };
                }
            }
            // Structure the spec: a static spec is one Literal part; a spec
            // containing replacement fields parses recursively so the nested
            // expressions evaluate at runtime ({value:{width}}).
            let spec_parts = match format_spec {
                None => None,
                Some(sp) if !sp.contains('{') => Some(vec![FStringPart::Literal(sp)]),
                Some(sp) => Some(parse_fstring_parts(&sp, is_raw)?),
            };
            parts.push(FStringPart::Expr(Spanned::new(expr_node, span), spec_parts));
        } else {
            // Copy the raw byte into the literal run; multi-byte UTF-8 sequences
            // are preserved (decoded as a unit when the run is flushed) instead
            // of being mangled by a per-byte `as char` (Latin-1) cast.
            lit.push(bytes[i]);
            i += 1;
        }
    }
    if !lit.is_empty() {
        parts.push(FStringPart::Literal(finish_fstring_literal(
            flush_lit(&mut lit),
            is_raw,
        )));
    }
    Ok(parts)
}

/// If `s` is a standalone f-string literal (`f"..."` or `f'...'`), return
/// the inner content (between the quotes).  Returns `None` for anything
/// more complex (e.g. `f'{x}' + "y"`).  Handles matching quote pairs only.
fn strip_fstring_literal(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    if bytes.len() < 3 || bytes[0] != b'f' {
        return None;
    }
    let quote = bytes[1];
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    if *bytes.last()? != quote {
        return None;
    }
    // Verify no unmatched quotes between start and end: walk the inner
    // content and ensure we don't exit the string prematurely.
    let inner = &s[2..s.len() - 1];
    let ib = inner.as_bytes();
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;
    let mut j = 0;
    while j < ib.len() {
        let b = ib[j];
        if let Some(q) = in_str {
            if b == b'\\' {
                j += if j + 1 < ib.len() { 2 } else { 1 };
                continue;
            }
            if b == q {
                in_str = None;
            }
        } else {
            match b {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                b'\'' | b'"' => {
                    if b == quote {
                        // Unmatched outer quote in the middle → not a simple f-string literal
                        return None;
                    }
                    in_str = Some(b);
                }
                _ => {}
            }
        }
        j += 1;
    }
    if depth != 0 {
        return None;
    }
    Some(inner)
}

/// Parse an f-string expression through the full parser.
/// Falls back to `Ident(s)` if parsing fails (e.g., keyword args).
fn parse_fstring_expr(s: &str) -> Expr {
    let tokens = crate::lexer::lex(s, crate::source::span::FileId(0));
    let mut parser = super::Parser::new(tokens, s, crate::source::span::FileId(0));
    match parser.parse_expr_bp(0) {
        Ok(spanned) => spanned.node,
        Err(_) => Expr::Ident(s.to_string()),
    }
}

/// Peel an optional `!r` / `!s` / `!a` conversion suffix off an f-string
/// replacement-field expression. Only detects a trailing `!<letter>` at the
/// top level (ignores `!` inside brackets/strings and `!=` operators).
/// Returns (expr_without_conversion, conversion_char).
fn split_expr_and_conversion(raw: &str) -> (&str, Option<char>) {
    let bytes = raw.as_bytes();
    // Find the last top-level `!` followed by a single ASCII alpha at end-of-string.
    // We scan forward tracking bracket/string depth and record the latest candidate.
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;
    let mut candidate: Option<(usize, char)> = None;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = in_str {
            if b == b'\\' {
                i += if i + 1 < bytes.len() { 2 } else { 1 };
                continue;
            }
            if b == q {
                in_str = None;
            }
        } else {
            match b {
                b'\'' | b'"' => in_str = Some(b),
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => depth -= 1,
                b'!' if depth == 0 => {
                    // Must not be `!=` (inequality).
                    let next = bytes.get(i + 1).copied();
                    if next != Some(b'=') {
                        if let Some(c) = next {
                            if c.is_ascii_alphabetic() && bytes.get(i + 2).is_none() {
                                candidate = Some((i, c as char));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    if let Some((pos, c)) = candidate {
        (&raw[..pos], Some(c))
    } else {
        (raw, None)
    }
}

/// Split f-string `{...}` content into expression and optional format spec.
/// The spec starts at the first `:` that is NOT inside brackets/parens/strings.
fn split_expr_and_spec(raw: &str) -> (&str, Option<String>) {
    let bytes = raw.as_bytes();
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = in_str {
            if b == b'\\' {
                i += if i + 1 < bytes.len() { 2 } else { 1 };
                continue;
            }
            if b == q {
                in_str = None;
            }
        } else {
            match b {
                b'\'' | b'"' => in_str = Some(b),
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => depth -= 1,
                b':' if depth == 0 => {
                    return (&raw[..i], Some(raw[i + 1..].to_string()));
                }
                _ => {}
            }
        }
        i += 1;
    }
    (raw, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::parser::ast::{BinOp, CallArg, Expr, FStringPart, Stmt, UnaryOp};
    use crate::source::span::FileId;

    fn fid() -> FileId {
        FileId(0)
    }
    fn parse_expr_str(src: &str) -> Expr {
        let full = format!("{src}\n");
        let module = parser::parse(&full, fid()).expect("parse failed");
        match module.stmts.into_iter().next().unwrap().node {
            Stmt::ExprStmt(e) => e.node,
            other => panic!("expected ExprStmt, got {other:?}"),
        }
    }

    // --- Literals ---

    #[test]
    fn test_int_literal() {
        assert!(matches!(parse_expr_str("42"), Expr::IntLit(42)));
    }

    #[test]
    fn test_float_literal() {
        match parse_expr_str("3.14") {
            Expr::FloatLit(v) => assert!((v - 3.14).abs() < 1e-10),
            other => panic!("expected FloatLit, got {other:?}"),
        }
    }

    #[test]
    fn test_string_literal() {
        match parse_expr_str("\"hello\"") {
            Expr::StrLit(s) => assert_eq!(s, "hello"),
            other => panic!("expected StrLit, got {other:?}"),
        }
    }

    #[test]
    fn test_implicit_string_concat_adjacent() {
        // PEP 3105 — adjacent string literals concatenate at compile time.
        match parse_expr_str("\"foo\" \"bar\"") {
            Expr::StrLit(s) => assert_eq!(s, "foobar"),
            other => panic!("expected StrLit, got {other:?}"),
        }
    }

    #[test]
    fn test_implicit_string_concat_three_way() {
        match parse_expr_str("\"a\" \"b\" \"c\"") {
            Expr::StrLit(s) => assert_eq!(s, "abc"),
            other => panic!("expected StrLit, got {other:?}"),
        }
    }

    #[test]
    fn test_implicit_bytes_concat() {
        match parse_expr_str("b\"foo\" b\"bar\"") {
            Expr::BytesLit(b) => assert_eq!(b, b"foobar"),
            other => panic!("expected BytesLit, got {other:?}"),
        }
    }

    #[test]
    fn test_bool_true() {
        assert!(matches!(parse_expr_str("True"), Expr::BoolLit(true)));
    }

    #[test]
    fn test_bool_false() {
        assert!(matches!(parse_expr_str("False"), Expr::BoolLit(false)));
    }

    #[test]
    fn test_none_literal() {
        assert!(matches!(parse_expr_str("None"), Expr::NoneLit));
    }

    #[test]
    fn test_ellipsis() {
        assert!(matches!(parse_expr_str("..."), Expr::Ellipsis));
    }

    // --- Unary operators ---

    #[test]
    fn test_unary_pos() {
        match parse_expr_str("+5") {
            Expr::UnaryOp {
                op: UnaryOp::Pos,
                operand,
            } => {
                assert!(matches!(operand.node, Expr::IntLit(5)));
            }
            other => panic!("expected UnaryOp::Pos, got {other:?}"),
        }
    }

    #[test]
    fn test_unary_neg() {
        match parse_expr_str("-10") {
            Expr::UnaryOp {
                op: UnaryOp::Neg,
                operand,
            } => {
                assert!(matches!(operand.node, Expr::IntLit(10)));
            }
            other => panic!("expected UnaryOp::Neg, got {other:?}"),
        }
    }

    #[test]
    fn test_unary_not() {
        match parse_expr_str("not True") {
            Expr::UnaryOp {
                op: UnaryOp::Not,
                operand,
            } => {
                assert!(matches!(operand.node, Expr::BoolLit(true)));
            }
            other => panic!("expected UnaryOp::Not, got {other:?}"),
        }
    }

    #[test]
    fn test_unary_bitnot() {
        match parse_expr_str("~x") {
            Expr::UnaryOp {
                op: UnaryOp::BitNot,
                operand,
            } => {
                assert!(matches!(operand.node, Expr::Ident(ref n) if n == "x"));
            }
            other => panic!("expected UnaryOp::BitNot, got {other:?}"),
        }
    }

    // --- Binary operators and precedence ---

    #[test]
    fn test_add() {
        match parse_expr_str("1 + 2") {
            Expr::BinOp {
                op: BinOp::Add,
                lhs,
                rhs,
            } => {
                assert!(matches!(lhs.node, Expr::IntLit(1)));
                assert!(matches!(rhs.node, Expr::IntLit(2)));
            }
            other => panic!("expected BinOp::Add, got {other:?}"),
        }
    }

    #[test]
    fn test_mul_higher_than_add() {
        // 1 + 2 * 3 => Add(1, Mul(2, 3))
        match parse_expr_str("1 + 2 * 3") {
            Expr::BinOp {
                op: BinOp::Add,
                lhs,
                rhs,
            } => {
                assert!(matches!(lhs.node, Expr::IntLit(1)));
                assert!(matches!(rhs.node, Expr::BinOp { op: BinOp::Mul, .. }));
            }
            other => panic!("expected Add(1, Mul(2,3)), got {other:?}"),
        }
    }

    #[test]
    fn test_pow_right_associative() {
        // 2 ** 3 ** 4 => Pow(2, Pow(3, 4))
        match parse_expr_str("2 ** 3 ** 4") {
            Expr::BinOp {
                op: BinOp::Pow,
                lhs,
                rhs,
            } => {
                assert!(matches!(lhs.node, Expr::IntLit(2)));
                match rhs.node {
                    Expr::BinOp {
                        op: BinOp::Pow,
                        lhs: inner_l,
                        rhs: inner_r,
                    } => {
                        assert!(matches!(inner_l.node, Expr::IntLit(3)));
                        assert!(matches!(inner_r.node, Expr::IntLit(4)));
                    }
                    other => panic!("expected inner Pow, got {other:?}"),
                }
            }
            other => panic!("expected Pow, got {other:?}"),
        }
    }

    #[test]
    fn test_comparison_eq() {
        match parse_expr_str("a == b") {
            Expr::BinOp { op: BinOp::Eq, .. } => {}
            other => panic!("expected BinOp::Eq, got {other:?}"),
        }
    }

    #[test]
    fn test_logical_and_or() {
        // a or b and c => Or(a, And(b, c))
        match parse_expr_str("a or b and c") {
            Expr::BinOp {
                op: BinOp::Or,
                lhs,
                rhs,
            } => {
                assert!(matches!(lhs.node, Expr::Ident(ref n) if n == "a"));
                assert!(matches!(rhs.node, Expr::BinOp { op: BinOp::And, .. }));
            }
            other => panic!("expected Or(a, And(b,c)), got {other:?}"),
        }
    }

    #[test]
    fn test_bitwise_ops() {
        match parse_expr_str("a | b") {
            Expr::BinOp {
                op: BinOp::BitOr, ..
            } => {}
            other => panic!("expected BitOr, got {other:?}"),
        }
        match parse_expr_str("a & b") {
            Expr::BinOp {
                op: BinOp::BitAnd, ..
            } => {}
            other => panic!("expected BitAnd, got {other:?}"),
        }
        match parse_expr_str("a ^ b") {
            Expr::BinOp {
                op: BinOp::BitXor, ..
            } => {}
            other => panic!("expected BitXor, got {other:?}"),
        }
    }

    #[test]
    fn test_shift_ops() {
        match parse_expr_str("a << b") {
            Expr::BinOp {
                op: BinOp::LShift, ..
            } => {}
            other => panic!("expected LShift, got {other:?}"),
        }
        match parse_expr_str("a >> b") {
            Expr::BinOp {
                op: BinOp::RShift, ..
            } => {}
            other => panic!("expected RShift, got {other:?}"),
        }
    }

    #[test]
    fn test_floor_div() {
        match parse_expr_str("a // b") {
            Expr::BinOp {
                op: BinOp::FloorDiv,
                ..
            } => {}
            other => panic!("expected FloorDiv, got {other:?}"),
        }
    }

    #[test]
    fn test_modulo() {
        match parse_expr_str("a % b") {
            Expr::BinOp { op: BinOp::Mod, .. } => {}
            other => panic!("expected Mod, got {other:?}"),
        }
    }

    #[test]
    fn test_is_and_is_not() {
        match parse_expr_str("x is None") {
            Expr::BinOp { op: BinOp::Is, .. } => {}
            other => panic!("expected Is, got {other:?}"),
        }
        match parse_expr_str("x is not None") {
            Expr::BinOp {
                op: BinOp::IsNot, ..
            } => {}
            other => panic!("expected IsNot, got {other:?}"),
        }
    }

    #[test]
    fn test_in_and_not_in() {
        match parse_expr_str("x in y") {
            Expr::BinOp { op: BinOp::In, .. } => {}
            other => panic!("expected In, got {other:?}"),
        }
        match parse_expr_str("x not in y") {
            Expr::BinOp {
                op: BinOp::NotIn, ..
            } => {}
            other => panic!("expected NotIn, got {other:?}"),
        }
    }

    // --- Call ---

    #[test]
    fn test_call_no_args() {
        match parse_expr_str("f()") {
            Expr::Call { func, args } => {
                assert!(matches!(func.node, Expr::Ident(ref n) if n == "f"));
                assert!(args.is_empty());
            }
            other => panic!("expected Call, got {other:?}"),
        }
    }

    #[test]
    fn test_call_with_keyword_arg() {
        match parse_expr_str("f(x=1)") {
            Expr::Call { args, .. } => {
                assert_eq!(args.len(), 1);
                match &args[0] {
                    CallArg::Keyword { name, value } => {
                        assert_eq!(name, "x");
                        assert!(matches!(value.node, Expr::IntLit(1)));
                    }
                    other => panic!("expected Keyword arg, got {other:?}"),
                }
            }
            other => panic!("expected Call, got {other:?}"),
        }
    }

    #[test]
    fn test_call_with_star_and_doublestar() {
        match parse_expr_str("f(*a, **b)") {
            Expr::Call { args, .. } => {
                assert_eq!(args.len(), 2);
                assert!(matches!(&args[0], CallArg::StarArg(_)));
                assert!(matches!(&args[1], CallArg::DoubleStarArg(_)));
            }
            other => panic!("expected Call with star args, got {other:?}"),
        }
    }

    // --- Attribute access ---

    #[test]
    fn test_attr_access() {
        match parse_expr_str("a.b") {
            Expr::Attr { object, attr } => {
                assert!(matches!(object.node, Expr::Ident(ref n) if n == "a"));
                assert_eq!(attr, "b");
            }
            other => panic!("expected Attr, got {other:?}"),
        }
    }

    #[test]
    fn test_chained_attr() {
        match parse_expr_str("a.b.c") {
            Expr::Attr { object, attr } => {
                assert_eq!(attr, "c");
                assert!(matches!(object.node, Expr::Attr { .. }));
            }
            other => panic!("expected chained Attr, got {other:?}"),
        }
    }

    // --- Index / Slice ---

    #[test]
    fn test_index() {
        match parse_expr_str("a[0]") {
            Expr::Index { object, index } => {
                assert!(matches!(object.node, Expr::Ident(ref n) if n == "a"));
                assert!(matches!(index.node, Expr::IntLit(0)));
            }
            other => panic!("expected Index, got {other:?}"),
        }
    }

    #[test]
    fn test_slice_start_stop() {
        match parse_expr_str("a[1:3]") {
            Expr::Index { index, .. } => match index.node {
                Expr::Slice { start, stop, step } => {
                    assert!(start.is_some());
                    assert!(stop.is_some());
                    assert!(step.is_none());
                }
                other => panic!("expected Slice, got {other:?}"),
            },
            other => panic!("expected Index(Slice), got {other:?}"),
        }
    }

    #[test]
    fn test_slice_no_start() {
        match parse_expr_str("a[:3]") {
            Expr::Index { index, .. } => match index.node {
                Expr::Slice { start, stop, step } => {
                    assert!(start.is_none());
                    assert!(stop.is_some());
                    assert!(step.is_none());
                }
                other => panic!("expected Slice, got {other:?}"),
            },
            other => panic!("expected Index(Slice), got {other:?}"),
        }
    }

    #[test]
    fn test_slice_with_step() {
        match parse_expr_str("a[::2]") {
            Expr::Index { index, .. } => match index.node {
                Expr::Slice { start, stop, step } => {
                    assert!(start.is_none());
                    assert!(stop.is_none());
                    assert!(step.is_some());
                }
                other => panic!("expected Slice, got {other:?}"),
            },
            other => panic!("expected Index(Slice), got {other:?}"),
        }
    }

    // --- Starred ---

    #[test]
    fn test_starred_expr() {
        match parse_expr_str("*a") {
            Expr::Starred(inner) => {
                assert!(matches!(inner.node, Expr::Ident(ref n) if n == "a"));
            }
            other => panic!("expected Starred, got {other:?}"),
        }
    }

    // --- Type keywords as expressions ---

    #[test]
    fn test_type_keyword_as_expr() {
        // `int` used as expression (e.g., int(x))
        match parse_expr_str("int") {
            Expr::Ident(n) => assert_eq!(n, "int"),
            other => panic!("expected Ident('int'), got {other:?}"),
        }
    }

    // --- Self ---

    #[test]
    fn test_self_expr() {
        match parse_expr_str("self") {
            Expr::Ident(n) => assert_eq!(n, "self"),
            other => panic!("expected Ident('self'), got {other:?}"),
        }
    }

    // --- f-string helpers ---

    #[test]
    fn test_parse_fstring_parts_literal_only() {
        let parts = parse_fstring_parts("hello world", false).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(matches!(&parts[0], FStringPart::Literal(s) if s == "hello world"));
    }

    #[test]
    fn test_parse_fstring_parts_single_expr() {
        let parts = parse_fstring_parts("hello {name}", false).unwrap();
        assert_eq!(parts.len(), 2);
        assert!(matches!(&parts[0], FStringPart::Literal(s) if s == "hello "));
        assert!(matches!(&parts[1], FStringPart::Expr(_, None)));
    }

    #[test]
    fn test_parse_fstring_parts_with_format_spec() {
        let parts = parse_fstring_parts("{x:.2f}", false).unwrap();
        assert_eq!(parts.len(), 1);
        match &parts[0] {
            FStringPart::Expr(_, Some(spec)) => {
                assert!(matches!(&spec[..], [FStringPart::Literal(l)] if l == ".2f"));
            }
            other => panic!("expected Expr with spec, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_fstring_parts_escaped_braces() {
        let parts = parse_fstring_parts("{{literal}}", false).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(matches!(&parts[0], FStringPart::Literal(s) if s == "{literal}"));
    }

    #[test]
    fn test_parse_fstring_literal_escapes_cooked() {
        // Non-raw f-string: literal runs get Python escape processing, so
        // `\n` collapses to a newline (matching `str` literals).
        let parts = parse_fstring_parts("a\\nb", false).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(matches!(&parts[0], FStringPart::Literal(s) if s == "a\nb"));
    }

    #[test]
    fn test_parse_fstring_literal_escapes_raw_kept() {
        // Raw f-string (`rf'...'`/`fr'...'`): backslash escapes stay verbatim.
        let parts = parse_fstring_parts("a\\nb", true).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(matches!(&parts[0], FStringPart::Literal(s) if s == "a\\nb"));
    }

    #[test]
    fn test_split_expr_and_spec_no_spec() {
        let (expr, spec) = split_expr_and_spec("name");
        assert_eq!(expr, "name");
        assert!(spec.is_none());
    }

    #[test]
    fn test_split_expr_and_spec_with_spec() {
        let (expr, spec) = split_expr_and_spec("x:.2f");
        assert_eq!(expr, "x");
        assert_eq!(spec.unwrap(), ".2f");
    }

    #[test]
    fn test_split_expr_and_spec_nested_brackets() {
        // Colon inside brackets should NOT split
        let (expr, spec) = split_expr_and_spec("d['key']");
        assert_eq!(expr, "d['key']");
        assert!(spec.is_none());
    }

    // --- Error cases ---

    #[test]
    fn test_unexpected_token_error() {
        let result = parser::parse(")\n", fid());
        assert!(result.is_err());
    }

    // ── R7: Dict/set literal in expression statement position ─────────────

    /// Parsing `{}` as an expression statement should produce a DictLit.
    #[test]
    fn test_empty_dict_literal_as_stmt() {
        match parse_expr_str("{}") {
            Expr::DictLit(entries) => {
                assert!(
                    entries.is_empty(),
                    "empty dict literal should have 0 entries"
                );
            }
            other => panic!("expected DictLit, got {other:?}"),
        }
    }

    /// Parsing `{1, 2, 3}` as an expression statement should produce a SetLit.
    #[test]
    fn test_set_literal_as_stmt() {
        match parse_expr_str("{1, 2, 3}") {
            Expr::SetLit(items) => {
                assert_eq!(items.len(), 3, "set literal should have 3 items");
            }
            other => panic!("expected SetLit, got {other:?}"),
        }
    }

    /// Parsing `{'a': 1, 'b': 2}` as an expression statement should produce a DictLit.
    #[test]
    fn test_dict_literal_as_stmt() {
        match parse_expr_str("{'a': 1, 'b': 2}") {
            Expr::DictLit(entries) => {
                assert_eq!(entries.len(), 2, "dict literal should have 2 entries");
            }
            other => panic!("expected DictLit, got {other:?}"),
        }
    }

    /// Parsing `{}['x']` should parse as an index operation on empty dict.
    #[test]
    fn test_empty_dict_subscript() {
        match parse_expr_str("{}['x']") {
            Expr::Index { object, index } => {
                assert!(
                    matches!(object.node, Expr::DictLit(ref e) if e.is_empty()),
                    "index object should be empty DictLit"
                );
                assert!(
                    matches!(index.node, Expr::StrLit(ref s) if s == "x"),
                    "index should be 'x'"
                );
            }
            other => panic!("expected Index on DictLit, got {other:?}"),
        }
    }

    /// Parsing `{1, 2}` as expression statement should produce a SetLit.
    #[test]
    fn test_set_literal_two_elements() {
        match parse_expr_str("{1, 2}") {
            Expr::SetLit(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(items[0].node, Expr::IntLit(1)));
                assert!(matches!(items[1].node, Expr::IntLit(2)));
            }
            other => panic!("expected SetLit, got {other:?}"),
        }
    }
}
