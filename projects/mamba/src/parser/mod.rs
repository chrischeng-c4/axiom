pub mod ast;
pub mod expr;
mod mangle;
pub mod expr_compound;
pub mod pattern;
pub mod stmt;
pub mod stmt_compound;
pub mod type_expr;

use crate::error::MambaError;
use crate::lexer::token::{Token, TokenKind};
use crate::source::span::{FileId, Span, Spanned};
use ast::{Module, Stmt};

/// Recursive descent parser for Mamba source code.
pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    source: &'a str,
    file_id: FileId,
    /// Statements that parse_stmt synthesized but hasn't yet returned.
    /// Populated only for forms like `a = b = c = expr` that desugar into
    /// multiple per-target `Stmt::Assign`s. Drained in LIFO order before
    /// the parser consumes any new tokens, so insertion order is preserved.
    pub(crate) pending_stmts: Vec<Spanned<Stmt>>,
    /// True when the next `parse_expr` is the top of an expression statement,
    /// where a bare walrus (`a := 5`) is a SyntaxError in CPython (it must be
    /// parenthesized). Captured-and-cleared at `parse_expr` entry so only the
    /// statement-top level — not parenthesized or nested sub-expressions — is
    /// affected.
    pub(crate) stmt_expr_toplevel: bool,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, source: &'a str, file_id: FileId) -> Self {
        Self {
            tokens,
            pos: 0,
            source,
            file_id,
            pending_stmts: Vec::new(),
            stmt_expr_toplevel: false,
        }
    }

    /// Parse the entire module.
    pub fn parse_module(&mut self) -> crate::error::Result<Module> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while self.peek_kind() != Some(TokenKind::Eof) && self.peek_kind().is_some() {
            stmts.push(self.parse_stmt()?);
            // Semicolons as statement separators between simple statements.
            while self.peek_kind() == Some(TokenKind::Semicolon) {
                self.advance(); // consume `;`
                                // Skip consecutive semicolons (empty statements)
                while self.peek_kind() == Some(TokenKind::Semicolon) {
                    self.advance();
                }
                // Trailing semicolon before newline/eof — stop
                if self.peek_kind() == Some(TokenKind::Newline)
                    || self.peek_kind() == Some(TokenKind::Eof)
                    || self.peek_kind().is_none()
                {
                    break;
                }
                // Compound statements are not allowed after `;`
                if let Some(kind) = self.peek_kind() {
                    if Self::is_compound_start(&kind) {
                        return Err(MambaError::syntax(
                            self.peek()
                                .map(|t| Span::new(self.file_id, t.start, t.end))
                                .unwrap_or(Span::dummy()),
                            "compound statement not allowed after semicolon".to_string(),
                        ));
                    }
                }
                stmts.push(self.parse_stmt()?);
            }
            self.skip_newlines();
        }
        Ok(Module { stmts })
    }

    // --- Token navigation ---

    pub(crate) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub(crate) fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|t| t.kind.clone())
    }

    /// Advance past the current token, returning its (start, end) byte offsets.
    pub(crate) fn advance(&mut self) -> (u32, u32) {
        let token = &self.tokens[self.pos];
        let result = (token.start, token.end);
        self.pos += 1;
        result
    }

    /// Expect a specific token kind, returning its (start, end) byte offsets.
    pub(crate) fn expect(&mut self, expected: TokenKind) -> crate::error::Result<(u32, u32)> {
        let pos = self.pos;
        let token = self.tokens.get(pos).ok_or_else(|| {
            MambaError::syntax(Span::dummy(), format!("expected {expected}, got EOF"))
        })?;
        if std::mem::discriminant(&token.kind) != std::mem::discriminant(&expected) {
            return Err(MambaError::syntax(
                Span::new(self.file_id, token.start, token.end),
                format!("expected {expected}, got {}", token.kind),
            ));
        }
        self.pos += 1;
        Ok((self.tokens[pos].start, self.tokens[pos].end))
    }

    /// Get the text content at a given byte range from the source.
    pub(crate) fn text_at(&self, start: u32, end: u32) -> &str {
        &self.source[start as usize..end as usize]
    }

    pub(crate) fn current_text(&self) -> &str {
        let token = &self.tokens[self.pos];
        token.text(self.source)
    }

    pub(crate) fn skip_newlines(&mut self) {
        while self.peek_kind() == Some(TokenKind::Newline) {
            self.advance();
        }
    }

    pub(crate) fn span_from(&self, start: u32) -> Span {
        let end = if self.pos > 0 {
            self.tokens[self.pos - 1].end
        } else {
            start
        };
        Span::new(self.file_id, start, end)
    }

    /// Check if a token starts a compound statement (if/while/for/def/class/etc.).
    /// These are not allowed after a semicolon separator.
    pub(crate) fn is_compound_start(kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::If
                | TokenKind::While
                | TokenKind::For
                | TokenKind::Def
                | TokenKind::Class
                | TokenKind::Async
                | TokenKind::Try
                | TokenKind::With
                | TokenKind::At
        )
    }

    /// Check if the current token can be used as a name (identifier or soft keyword).
    /// Python allows `int`, `float`, `bool`, `str`, `list`, `dict`, `tuple`, `type`
    /// as variable/parameter/attribute names.
    pub(crate) fn is_name_token(kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Ident
                | TokenKind::Self_
                | TokenKind::IntType
                | TokenKind::FloatType
                | TokenKind::BoolType
                | TokenKind::StrType
                | TokenKind::ListType
                | TokenKind::DictType
                | TokenKind::TupleType
                | TokenKind::Type
                | TokenKind::Match
                | TokenKind::Case
                | TokenKind::Enum
        )
    }

    /// Lookahead helper used by lambda type-annotation disambiguation
    /// (#1582). Scan forward from `self.pos` and return true iff a `:`
    /// appears at depth 0 (relative to current paren/bracket/brace nesting)
    /// before we close out a paren / hit a newline / hit EOF.
    ///
    /// Used to distinguish a typed-lambda continuation
    /// (`lambda x: T, y: U: body` — body-`:` exists ahead) from a lambda
    /// whose body is part of a tuple / call-arg list
    /// (`lambda x: x, rest_of_args` — no body-`:` ahead).
    pub(crate) fn has_lambda_body_colon_ahead(&self) -> bool {
        let mut depth: i32 = 0;
        for i in self.pos..self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
                TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                    depth -= 1;
                    if depth < 0 {
                        return false;
                    }
                }
                TokenKind::Colon if depth == 0 => return true,
                TokenKind::Newline | TokenKind::Semicolon | TokenKind::Eof if depth == 0 => {
                    return false;
                }
                _ => {}
            }
        }
        false
    }

    /// Expect a name token (identifier or soft keyword), returning (start, end).
    pub(crate) fn expect_name(&mut self) -> crate::error::Result<(u32, u32)> {
        let token = self.tokens.get(self.pos).ok_or_else(|| {
            MambaError::syntax(Span::dummy(), "expected identifier, got EOF".to_string())
        })?;
        if Self::is_name_token(&token.kind) {
            let result = (token.start, token.end);
            self.pos += 1;
            Ok(result)
        } else {
            Err(MambaError::syntax(
                Span::new(self.file_id, token.start, token.end),
                format!("expected identifier, got {}", token.kind),
            ))
        }
    }
}

/// Convenience function: parse source into a Module.
pub fn parse(source: &str, file_id: FileId) -> crate::error::Result<Module> {
    let tokens = crate::lexer::lex(source, file_id);
    let mut parser = Parser::new(tokens, source, file_id);
    let mut module = parser.parse_module()?;
    // PEP-classic private name mangling — rewrite `__name` inside class bodies
    // before the type checker and lowering observe the AST.
    mangle::mangle_module(&mut module);
    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::span::FileId;

    fn fid() -> FileId {
        FileId(0)
    }

    // --- Parser::new and basic token navigation ---

    #[test]
    fn test_parse_empty_source() {
        let module = parse("", fid()).unwrap();
        assert!(module.stmts.is_empty());
    }

    #[test]
    fn test_parse_only_newlines() {
        let module = parse("\n\n\n", fid()).unwrap();
        assert!(module.stmts.is_empty());
    }

    #[test]
    fn test_parse_single_pass() {
        let module = parse("pass\n", fid()).unwrap();
        assert_eq!(module.stmts.len(), 1);
        assert!(matches!(&module.stmts[0].node, ast::Stmt::Pass));
    }

    #[test]
    fn test_parse_multiple_statements() {
        let module = parse("pass\nbreak\ncontinue\n", fid()).unwrap();
        assert_eq!(module.stmts.len(), 3);
        assert!(matches!(&module.stmts[0].node, ast::Stmt::Pass));
        assert!(matches!(&module.stmts[1].node, ast::Stmt::Break));
        assert!(matches!(&module.stmts[2].node, ast::Stmt::Continue));
    }

    // --- peek / peek_kind / advance ---

    #[test]
    fn test_peek_returns_first_token() {
        let tokens = crate::lexer::lex("42\n", fid());
        let parser = Parser::new(tokens, "42\n", fid());
        assert!(parser.peek().is_some());
        assert!(matches!(parser.peek_kind(), Some(TokenKind::Int(42))));
    }

    #[test]
    fn test_advance_moves_position() {
        let tokens = crate::lexer::lex("42\n", fid());
        let mut parser = Parser::new(tokens, "42\n", fid());
        let (start, end) = parser.advance();
        assert_eq!(start, 0);
        assert_eq!(end, 2);
        // After advancing past Int(42), next should be Newline or Eof
        assert!(parser.peek().is_some());
    }

    // --- expect ---

    #[test]
    fn test_expect_success() {
        let src = "pass\n";
        let tokens = crate::lexer::lex(src, fid());
        let mut parser = Parser::new(tokens, src, fid());
        assert!(parser.expect(TokenKind::Pass).is_ok());
    }

    #[test]
    fn test_expect_wrong_token() {
        let src = "pass\n";
        let tokens = crate::lexer::lex(src, fid());
        let mut parser = Parser::new(tokens, src, fid());
        let err = parser.expect(TokenKind::Break).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("expected"), "error message: {msg}");
    }

    // --- expect_name ---

    #[test]
    fn test_expect_name_ident() {
        let src = "foo\n";
        let tokens = crate::lexer::lex(src, fid());
        let mut parser = Parser::new(tokens, src, fid());
        let (s, e) = parser.expect_name().unwrap();
        assert_eq!(parser.text_at(s, e), "foo");
    }

    #[test]
    fn test_expect_name_soft_keyword() {
        // `match` and `enum` are soft keywords usable as names
        let src = "match\n";
        let tokens = crate::lexer::lex(src, fid());
        let mut parser = Parser::new(tokens, src, fid());
        assert!(parser.expect_name().is_ok());
    }

    #[test]
    fn test_expect_name_rejects_hard_keyword() {
        let src = "if\n";
        let tokens = crate::lexer::lex(src, fid());
        let mut parser = Parser::new(tokens, src, fid());
        assert!(parser.expect_name().is_err());
    }

    // --- is_name_token ---

    #[test]
    fn test_is_name_token_variants() {
        assert!(Parser::is_name_token(&TokenKind::Ident));
        assert!(Parser::is_name_token(&TokenKind::Self_));
        assert!(Parser::is_name_token(&TokenKind::IntType));
        assert!(Parser::is_name_token(&TokenKind::Match));
        assert!(Parser::is_name_token(&TokenKind::Enum));
        assert!(!Parser::is_name_token(&TokenKind::If));
        assert!(!Parser::is_name_token(&TokenKind::Def));
        assert!(!Parser::is_name_token(&TokenKind::Class));
    }

    // --- skip_newlines ---

    #[test]
    fn test_skip_newlines() {
        let src = "\n\n\npass\n";
        let tokens = crate::lexer::lex(src, fid());
        let mut parser = Parser::new(tokens, src, fid());
        parser.skip_newlines();
        assert!(matches!(parser.peek_kind(), Some(TokenKind::Pass)));
    }

    // --- span_from ---

    #[test]
    fn test_span_from() {
        let src = "x = 1\n";
        let tokens = crate::lexer::lex(src, fid());
        let mut parser = Parser::new(tokens, src, fid());
        parser.advance(); // x
        parser.advance(); // =
        parser.advance(); // 1
        let span = parser.span_from(0);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5); // end of `1`
    }

    // --- parse convenience function error ---

    #[test]
    fn test_parse_syntax_error() {
        let result = parse("def\n", fid());
        assert!(result.is_err());
    }

    // --- text_at ---

    #[test]
    fn test_text_at() {
        let src = "hello world\n";
        let tokens = crate::lexer::lex(src, fid());
        let parser = Parser::new(tokens, src, fid());
        assert_eq!(parser.text_at(0, 5), "hello");
        assert_eq!(parser.text_at(6, 11), "world");
    }
}

#[cfg(test)]
#[path = "tests"]
mod inline_integration_tests {
    mod core;
}
