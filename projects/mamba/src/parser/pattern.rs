use super::ast::*;
use super::Parser;
use crate::error::MambaError;
use crate::lexer::token::TokenKind;
use crate::source::span::{Span, Spanned};

impl<'a> Parser<'a> {
    /// Parse a match pattern (with OR: `p1 | p2 | p3`, and AS: `p as name`).
    pub fn parse_pattern(&mut self) -> crate::error::Result<Spanned<Pattern>> {
        let first = self.parse_single_pattern()?;
        let start = first.span.start;

        // OR pattern: `p1 | p2`
        let pat = if self.peek_kind() == Some(TokenKind::Pipe) {
            let mut alternatives = vec![first];
            while self.peek_kind() == Some(TokenKind::Pipe) {
                self.advance();
                alternatives.push(self.parse_single_pattern()?);
            }
            Spanned::new(Pattern::Or(alternatives), self.span_from(start))
        } else {
            first
        };

        // AS pattern: `p as name` (PEP 634, #827)
        if self.peek_kind() == Some(TokenKind::As) {
            self.advance(); // consume `as`
            let (ns, ne) = self.expect(TokenKind::Ident)?;
            let name = self.text_at(ns, ne).to_string();
            return Ok(Spanned::new(
                Pattern::As {
                    pattern: Box::new(pat),
                    name,
                },
                self.span_from(start),
            ));
        }

        Ok(pat)
    }

    /// Parse a single pattern (no OR at this level).
    fn parse_single_pattern(&mut self) -> crate::error::Result<Spanned<Pattern>> {
        let token = self
            .peek()
            .ok_or_else(|| MambaError::syntax(Span::dummy(), "expected pattern"))?;
        let start = token.start;

        match &token.kind {
            // Star patterns (`*rest`) are only valid inside sequence patterns.
            // Reject them at the top-level single-pattern parse so that invalid
            // syntax like `case *rest:` is caught with a clear error (#827).
            TokenKind::Star => {
                return Err(MambaError::syntax(
                    Span::new(self.file_id, start, token.end),
                    "star pattern '*name' is only valid inside a sequence pattern like [*rest, x]"
                        .to_string(),
                ));
            }
            // Sequence pattern: `[a, b, *rest]`
            TokenKind::LBracket => self.parse_sequence_pattern(),
            // Tuple sequence pattern: `(a, b)` — PEP 634 treats these as sequence patterns
            TokenKind::LParen => self.parse_paren_sequence_pattern(),
            // Mapping pattern: `{"key": value}`
            TokenKind::LBrace => self.parse_mapping_pattern(),
            // Wildcard: `_`
            TokenKind::Ident if self.current_text() == "_" => {
                self.advance();
                Ok(Spanned::new(Pattern::Wildcard, self.span_from(start)))
            }
            // Ident: binding, constructor, or class pattern
            TokenKind::Ident => self.parse_ident_pattern(),
            // Built-in type keywords as class pattern heads: `int(n)`, `str(s)`, etc.
            TokenKind::IntType
            | TokenKind::FloatType
            | TokenKind::BoolType
            | TokenKind::StrType
            | TokenKind::ListType
            | TokenKind::DictType
            | TokenKind::TupleType => self.parse_builtin_type_pattern(),
            // Literal patterns
            TokenKind::Int(_)
            | TokenKind::Float(_)
            | TokenKind::Str(_)
            | TokenKind::True
            | TokenKind::False
            | TokenKind::None_ => {
                let expr = self.parse_pattern_literal()?;
                Ok(Spanned::new(Pattern::Literal(expr.node), expr.span))
            }
            // Negative number literals: `-42`
            TokenKind::Minus => {
                let expr = self.parse_pattern_literal()?;
                Ok(Spanned::new(Pattern::Literal(expr.node), expr.span))
            }
            _ => Err(MambaError::syntax(
                Span::new(self.file_id, start, token.end),
                format!("expected pattern, got {}", token.kind),
            )),
        }
    }

    /// Parse a built-in type keyword (`int`, `str`, `float`, etc.) as a class
    /// pattern head. If followed by `(`, parse as ClassPattern; otherwise
    /// treat as a bare constructor with no fields.
    fn parse_builtin_type_pattern(&mut self) -> crate::error::Result<Spanned<Pattern>> {
        let start = self.peek().unwrap().start;
        // Map the token kind to its string name
        let type_name = match self.peek_kind().unwrap() {
            TokenKind::IntType => "int",
            TokenKind::FloatType => "float",
            TokenKind::BoolType => "bool",
            TokenKind::StrType => "str",
            TokenKind::ListType => "list",
            TokenKind::DictType => "dict",
            TokenKind::TupleType => "tuple",
            _ => unreachable!(),
        };
        self.advance(); // consume the type keyword token

        let cls = vec![type_name.to_string()];
        if self.peek_kind() == Some(TokenKind::LParen) {
            self.parse_class_or_constructor_args(cls, start)
        } else {
            // bare type name without parens — Constructor with no fields
            Ok(Spanned::new(
                Pattern::Constructor {
                    path: cls,
                    fields: Vec::new(),
                },
                self.span_from(start),
            ))
        }
    }

    /// Parse `[a, b, *rest]` sequence pattern.
    fn parse_sequence_pattern(&mut self) -> crate::error::Result<Spanned<Pattern>> {
        let start = self.peek().unwrap().start;
        self.advance(); // consume [

        let mut patterns = Vec::new();
        while self.peek_kind() != Some(TokenKind::RBracket)
            && self.peek_kind() != Some(TokenKind::Eof)
        {
            // Star elements are valid only inside sequences.
            patterns.push(self.parse_sequence_element_pattern()?);
            if self.peek_kind() != Some(TokenKind::RBracket) {
                self.expect(TokenKind::Comma)?;
            }
        }
        self.expect(TokenKind::RBracket)?;

        // PEP 634: at most one starred element is allowed in a sequence pattern.
        let star_count = patterns
            .iter()
            .filter(|p| matches!(&p.node, Pattern::Star(_)))
            .count();
        if star_count > 1 {
            return Err(MambaError::syntax(
                self.span_from(start),
                "sequence pattern may only have one starred element",
            ));
        }

        Ok(Spanned::new(
            Pattern::Sequence(patterns),
            self.span_from(start),
        ))
    }

    /// Parse a single element inside a sequence pattern, allowing `*name` stars.
    /// Star patterns are only valid in this context, not at the top-level (#827).
    fn parse_sequence_element_pattern(&mut self) -> crate::error::Result<Spanned<Pattern>> {
        let token = self
            .peek()
            .ok_or_else(|| MambaError::syntax(Span::dummy(), "expected pattern"))?;
        let start = token.start;
        if token.kind == TokenKind::Star {
            self.advance();
            let name = if self.peek_kind() == Some(TokenKind::Ident) {
                let text = self.current_text().to_string();
                self.advance();
                if text == "_" {
                    None
                } else {
                    Some(text)
                }
            } else {
                None
            };
            return Ok(Spanned::new(Pattern::Star(name), self.span_from(start)));
        }
        self.parse_pattern()
    }

    /// Parse `(a, b, *rest)` tuple-style sequence pattern.
    /// - `()` → empty Sequence
    /// - `(pat,)` → Sequence with one element
    /// - `(pat, pat, ...)` → Sequence
    /// - `(pat)` with no trailing comma → grouping (returns inner pattern, not Sequence)
    fn parse_paren_sequence_pattern(&mut self) -> crate::error::Result<Spanned<Pattern>> {
        let start = self.peek().unwrap().start;
        self.advance(); // consume (

        // Empty parens: ()
        if self.peek_kind() == Some(TokenKind::RParen) {
            self.advance();
            return Ok(Spanned::new(
                Pattern::Sequence(vec![]),
                self.span_from(start),
            ));
        }

        let first = self.parse_sequence_element_pattern()?;

        // If next is `)` without a trailing comma: grouping, return inner pattern
        if self.peek_kind() == Some(TokenKind::RParen) {
            self.advance();
            return Ok(first);
        }

        // Trailing comma or more elements → this is a Sequence
        self.expect(TokenKind::Comma)?;
        let mut patterns = vec![first];

        while self.peek_kind() != Some(TokenKind::RParen)
            && self.peek_kind() != Some(TokenKind::Eof)
        {
            patterns.push(self.parse_sequence_element_pattern()?);
            if self.peek_kind() != Some(TokenKind::RParen) {
                self.expect(TokenKind::Comma)?;
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(Spanned::new(
            Pattern::Sequence(patterns),
            self.span_from(start),
        ))
    }

    /// Parse `{"key": value, ..., **rest}` mapping pattern (#827).
    fn parse_mapping_pattern(&mut self) -> crate::error::Result<Spanned<Pattern>> {
        let start = self.peek().unwrap().start;
        self.advance(); // consume {

        let mut pairs = Vec::new();
        let mut rest: Option<Name> = None;

        while self.peek_kind() != Some(TokenKind::RBrace)
            && self.peek_kind() != Some(TokenKind::Eof)
        {
            // Rest capture: `**rest`
            // PEP 634: `**_` is not valid syntax; a real identifier is required.
            if self.peek_kind() == Some(TokenKind::DoubleStar) {
                self.advance(); // consume **
                let name = if self.peek_kind() == Some(TokenKind::Ident) {
                    let text = self.current_text().to_string();
                    if text == "_" {
                        let span = self
                            .peek()
                            .map(|t| Span::new(self.file_id, t.start, t.end))
                            .unwrap_or_else(Span::dummy);
                        return Err(MambaError::syntax(
                            span,
                            "mapping rest pattern requires an identifier, \
                             `**_` is not valid; use `case {**rest}:` to capture remaining entries",
                        ));
                    }
                    self.advance();
                    Some(text)
                } else {
                    let span = self
                        .peek()
                        .map(|t| Span::new(self.file_id, t.start, t.end))
                        .unwrap_or_else(Span::dummy);
                    return Err(MambaError::syntax(
                        span,
                        "expected identifier after `**` in mapping pattern",
                    ));
                };
                rest = name;
                // `**rest` must be last; skip optional trailing comma
                if self.peek_kind() == Some(TokenKind::Comma) {
                    self.advance();
                }
                break;
            }
            // PEP 634: mapping keys must be literals, not arbitrary expressions (#827)
            let key = self.parse_pattern_literal()?;
            self.expect(TokenKind::Colon)?;
            let value = self.parse_pattern()?;
            pairs.push((key, value));
            if self.peek_kind() != Some(TokenKind::RBrace) {
                self.expect(TokenKind::Comma)?;
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(Spanned::new(
            Pattern::Mapping { pairs, rest },
            self.span_from(start),
        ))
    }

    /// Parse ident-starting patterns: bindings, constructors, or class patterns.
    fn parse_ident_pattern(&mut self) -> crate::error::Result<Spanned<Pattern>> {
        let start = self.peek().unwrap().start;
        let first = self.current_text().to_string();
        self.advance();

        // Dotted path: `Shape.Circle(r)` or `mod.Class(x=1)`
        if self.peek_kind() == Some(TokenKind::Dot) {
            let mut path = vec![first];
            while self.peek_kind() == Some(TokenKind::Dot) {
                self.advance();
                let (ps, pe) = self.expect(TokenKind::Ident)?;
                path.push(self.text_at(ps, pe).to_string());
            }
            if self.peek_kind() == Some(TokenKind::LParen) {
                return self.parse_class_or_constructor_args(path, start);
            }
            // Dotted name without parens — treat as constructor with no fields
            return Ok(Spanned::new(
                Pattern::Constructor {
                    path,
                    fields: Vec::new(),
                },
                self.span_from(start),
            ));
        }

        // Parens after bare ident: `Circle(r)` or `ClassName(x=1)`
        if self.peek_kind() == Some(TokenKind::LParen) {
            return self.parse_class_or_constructor_args(vec![first], start);
        }

        // Simple binding
        Ok(Spanned::new(Pattern::Binding(first), self.span_from(start)))
    }

    /// Parse a literal for use in a pattern: atom or negated atom only.
    /// Does NOT call parse_expr to avoid consuming `|` as bitwise OR.
    fn parse_pattern_literal(&mut self) -> crate::error::Result<Spanned<Expr>> {
        let start = self.peek().unwrap().start;
        let negative = if self.peek_kind() == Some(TokenKind::Minus) {
            self.advance();
            true
        } else {
            false
        };
        // Parse just the atom (number, string, bool, None)
        let inner = self.parse_literal_atom()?;
        if negative {
            let span = self.span_from(start);
            Ok(Spanned::new(
                Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    operand: Box::new(inner),
                },
                span,
            ))
        } else {
            Ok(inner)
        }
    }

    /// Parse a single literal atom: Int, Float, Str, True, False, None_.
    fn parse_literal_atom(&mut self) -> crate::error::Result<Spanned<Expr>> {
        let token = self
            .peek()
            .ok_or_else(|| MambaError::syntax(Span::dummy(), "expected literal"))?;
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
            TokenKind::Str(v) | TokenKind::TripleStr(v) | TokenKind::RawStr(v) => {
                let v = v.clone();
                self.advance();
                Ok(Spanned::new(Expr::StrLit(v), self.span_from(start)))
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
            other => Err(MambaError::syntax(
                Span::new(self.file_id, start, token.end),
                format!("expected literal, got {other}"),
            )),
        }
    }

    /// Parse `(...)` args for constructor or class pattern.
    /// Distinguishes between:
    /// - Constructor: `Circle(r, g)` — all positional idents
    /// - ClassPattern: `Point(x=1, y=2)` — has `name=pattern` form
    fn parse_class_or_constructor_args(
        &mut self,
        cls: Vec<Name>,
        start: u32,
    ) -> crate::error::Result<Spanned<Pattern>> {
        self.advance(); // consume (

        // Empty parens
        if self.peek_kind() == Some(TokenKind::RParen) {
            self.advance();
            return Ok(Spanned::new(
                Pattern::Constructor {
                    path: cls,
                    fields: Vec::new(),
                },
                self.span_from(start),
            ));
        }

        // Always parse as ClassPattern to support mixed positional/keyword args (PEP 634 #827)
        let mut patterns = Vec::new();
        let mut seen_keyword = false;
        while self.peek_kind() != Some(TokenKind::RParen)
            && self.peek_kind() != Some(TokenKind::Eof)
        {
            // PEP 634: star patterns are not allowed inside class patterns.
            if self.peek_kind() == Some(TokenKind::Star) {
                let span = self
                    .peek()
                    .map(|t| Span::new(self.file_id, t.start, t.end))
                    .unwrap_or_else(Span::dummy);
                return Err(MambaError::syntax(
                    span,
                    "starred patterns are not allowed in class patterns",
                ));
            }
            if self.peek_kind() == Some(TokenKind::Ident) {
                let saved = self.pos;
                let (ns, ne) = self.expect(TokenKind::Ident)?;
                let name_text = self.text_at(ns, ne).to_string();
                if self.peek_kind() == Some(TokenKind::Eq) {
                    self.advance(); // consume =
                    let pat = self.parse_pattern()?;
                    patterns.push((Some(name_text), pat));
                    seen_keyword = true;
                } else {
                    // Backtrack — positional pattern starting with an ident
                    // PEP 634: positional may not follow keyword.
                    if seen_keyword {
                        let span = self
                            .peek()
                            .map(|t| Span::new(self.file_id, t.start, t.end))
                            .unwrap_or_else(Span::dummy);
                        return Err(MambaError::syntax(
                            span,
                            "positional pattern argument may not follow keyword argument",
                        ));
                    }
                    self.pos = saved;
                    let pat = self.parse_pattern()?;
                    patterns.push((None, pat));
                }
            } else {
                // Positional pattern (literal, sequence, mapping, etc.)
                // PEP 634: positional may not follow keyword.
                if seen_keyword {
                    let span = self
                        .peek()
                        .map(|t| Span::new(self.file_id, t.start, t.end))
                        .unwrap_or_else(Span::dummy);
                    return Err(MambaError::syntax(
                        span,
                        "positional pattern argument may not follow keyword argument",
                    ));
                }
                let pat = self.parse_pattern()?;
                patterns.push((None, pat));
            }
            if self.peek_kind() != Some(TokenKind::RParen) {
                self.expect(TokenKind::Comma)?;
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(Spanned::new(
            Pattern::ClassPattern { cls, patterns },
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

    /// Parse a match statement with a single arm and return the Pattern.
    fn parse_pattern(pat_src: &str) -> Pattern {
        let src = format!("match x:\n    case {pat_src}:\n        pass\n");
        let module = parser::parse(&src, fid()).expect("parse failed");
        match module.stmts.into_iter().next().unwrap().node {
            Stmt::Match { arms, .. } => arms.into_iter().next().unwrap().pattern.node,
            other => panic!("expected Match, got {other:?}"),
        }
    }

    // --- Wildcard ---

    #[test]
    fn test_wildcard_pattern() {
        assert!(matches!(parse_pattern("_"), Pattern::Wildcard));
    }

    // --- Binding ---

    #[test]
    fn test_binding_pattern() {
        match parse_pattern("x") {
            Pattern::Binding(name) => assert_eq!(name, "x"),
            other => panic!("expected Binding, got {other:?}"),
        }
    }

    // --- Literal patterns ---

    #[test]
    fn test_int_literal_pattern() {
        match parse_pattern("42") {
            Pattern::Literal(Expr::IntLit(42)) => {}
            other => panic!("expected Literal(42), got {other:?}"),
        }
    }

    #[test]
    fn test_string_literal_pattern() {
        match parse_pattern("\"hello\"") {
            Pattern::Literal(Expr::StrLit(s)) => assert_eq!(s, "hello"),
            other => panic!("expected Literal(str), got {other:?}"),
        }
    }

    #[test]
    fn test_true_literal_pattern() {
        match parse_pattern("True") {
            Pattern::Literal(Expr::BoolLit(true)) => {}
            other => panic!("expected Literal(True), got {other:?}"),
        }
    }

    #[test]
    fn test_false_literal_pattern() {
        match parse_pattern("False") {
            Pattern::Literal(Expr::BoolLit(false)) => {}
            other => panic!("expected Literal(False), got {other:?}"),
        }
    }

    #[test]
    fn test_none_literal_pattern() {
        match parse_pattern("None") {
            Pattern::Literal(Expr::NoneLit) => {}
            other => panic!("expected Literal(None), got {other:?}"),
        }
    }

    #[test]
    fn test_negative_literal_pattern() {
        match parse_pattern("-1") {
            Pattern::Literal(Expr::UnaryOp {
                op: UnaryOp::Neg,
                operand,
            }) => {
                assert!(matches!(operand.node, Expr::IntLit(1)));
            }
            other => panic!("expected Literal(-1), got {other:?}"),
        }
    }

    // --- Constructor ---

    #[test]
    fn test_constructor_no_fields() {
        match parse_pattern("Point()") {
            Pattern::Constructor { path, fields } => {
                assert_eq!(path, vec!["Point"]);
                assert!(fields.is_empty());
            }
            other => panic!("expected Constructor, got {other:?}"),
        }
    }

    #[test]
    fn test_constructor_with_fields() {
        // Since Fix #827, positional args are parsed as ClassPattern with (None, pattern)
        match parse_pattern("Circle(r)") {
            Pattern::ClassPattern { cls, patterns } => {
                assert_eq!(cls, vec!["Circle"]);
                assert_eq!(patterns.len(), 1);
                assert_eq!(patterns[0].0, None);
                assert!(matches!(&patterns[0].1.node, Pattern::Binding(n) if n == "r"));
            }
            other => panic!("expected ClassPattern, got {other:?}"),
        }
    }

    #[test]
    fn test_constructor_dotted_path() {
        match parse_pattern("Shape.Circle") {
            Pattern::Constructor { path, fields } => {
                assert_eq!(path, vec!["Shape", "Circle"]);
                assert!(fields.is_empty());
            }
            other => panic!("expected Constructor, got {other:?}"),
        }
    }

    #[test]
    fn test_constructor_dotted_with_fields() {
        // Since Fix #827, positional args are parsed as ClassPattern with (None, pattern)
        match parse_pattern("Shape.Circle(r)") {
            Pattern::ClassPattern { cls, patterns } => {
                assert_eq!(cls, vec!["Shape", "Circle"]);
                assert_eq!(patterns.len(), 1);
                assert_eq!(patterns[0].0, None);
                assert!(matches!(&patterns[0].1.node, Pattern::Binding(n) if n == "r"));
            }
            other => panic!("expected ClassPattern, got {other:?}"),
        }
    }

    // --- Class pattern ---

    #[test]
    fn test_class_pattern() {
        match parse_pattern("Point(x=1, y=2)") {
            Pattern::ClassPattern { cls, patterns } => {
                assert_eq!(cls, vec!["Point"]);
                assert_eq!(patterns.len(), 2);
                assert_eq!(patterns[0].0.as_deref(), Some("x"));
                assert_eq!(patterns[1].0.as_deref(), Some("y"));
            }
            other => panic!("expected ClassPattern, got {other:?}"),
        }
    }

    // --- Sequence pattern ---

    #[test]
    fn test_sequence_pattern_empty() {
        match parse_pattern("[]") {
            Pattern::Sequence(pats) => assert!(pats.is_empty()),
            other => panic!("expected Sequence, got {other:?}"),
        }
    }

    #[test]
    fn test_sequence_pattern_elements() {
        match parse_pattern("[a, b, c]") {
            Pattern::Sequence(pats) => {
                assert_eq!(pats.len(), 3);
                assert!(matches!(&pats[0].node, Pattern::Binding(n) if n == "a"));
                assert!(matches!(&pats[1].node, Pattern::Binding(n) if n == "b"));
                assert!(matches!(&pats[2].node, Pattern::Binding(n) if n == "c"));
            }
            other => panic!("expected Sequence, got {other:?}"),
        }
    }

    #[test]
    fn test_sequence_with_star() {
        match parse_pattern("[a, *rest]") {
            Pattern::Sequence(pats) => {
                assert_eq!(pats.len(), 2);
                assert!(matches!(&pats[0].node, Pattern::Binding(n) if n == "a"));
                match &pats[1].node {
                    Pattern::Star(Some(name)) => assert_eq!(name, "rest"),
                    other => panic!("expected Star(rest), got {other:?}"),
                }
            }
            other => panic!("expected Sequence, got {other:?}"),
        }
    }

    // --- Mapping pattern ---

    #[test]
    fn test_mapping_pattern() {
        match parse_pattern("{\"key\": value}") {
            Pattern::Mapping { pairs, rest } => {
                assert_eq!(pairs.len(), 1);
                assert!(rest.is_none());
                match &pairs[0].0.node {
                    Expr::StrLit(s) => assert_eq!(s, "key"),
                    other => panic!("expected StrLit key, got {other:?}"),
                }
                assert!(matches!(&pairs[0].1.node, Pattern::Binding(n) if n == "value"));
            }
            other => panic!("expected Mapping, got {other:?}"),
        }
    }

    // --- Mapping pattern with rest capture (#827) ---

    #[test]
    fn test_mapping_pattern_with_rest() {
        match parse_pattern("{\"type\": kind, **rest}") {
            Pattern::Mapping { pairs, rest } => {
                assert_eq!(pairs.len(), 1);
                assert_eq!(rest.as_deref(), Some("rest"));
                match &pairs[0].0.node {
                    Expr::StrLit(s) => assert_eq!(s, "type"),
                    other => panic!("expected StrLit key, got {other:?}"),
                }
                assert!(matches!(&pairs[0].1.node, Pattern::Binding(n) if n == "kind"));
            }
            other => panic!("expected Mapping, got {other:?}"),
        }
    }

    #[test]
    fn test_mapping_pattern_rest_only() {
        match parse_pattern("{**rest}") {
            Pattern::Mapping { pairs, rest } => {
                assert!(pairs.is_empty());
                assert_eq!(rest.as_deref(), Some("rest"));
            }
            other => panic!("expected Mapping, got {other:?}"),
        }
    }

    // --- AS pattern (#827) ---

    #[test]
    fn test_as_pattern_with_binding() {
        match parse_pattern("x as y") {
            Pattern::As { pattern, name } => {
                assert_eq!(name, "y");
                assert!(matches!(&pattern.node, Pattern::Binding(n) if n == "x"));
            }
            other => panic!("expected As, got {other:?}"),
        }
    }

    #[test]
    fn test_as_pattern_with_literal() {
        match parse_pattern("42 as n") {
            Pattern::As { pattern, name } => {
                assert_eq!(name, "n");
                assert!(matches!(&pattern.node, Pattern::Literal(Expr::IntLit(42))));
            }
            other => panic!("expected As, got {other:?}"),
        }
    }

    // --- OR pattern ---

    #[test]
    fn test_literal_or_pattern() {
        match parse_pattern("1 | 2 | 3") {
            Pattern::Or(alts) => {
                assert_eq!(alts.len(), 3);
                assert!(matches!(&alts[0].node, Pattern::Literal(Expr::IntLit(1))));
                assert!(matches!(&alts[1].node, Pattern::Literal(Expr::IntLit(2))));
                assert!(matches!(&alts[2].node, Pattern::Literal(Expr::IntLit(3))));
            }
            other => panic!("expected Or, got {other:?}"),
        }
    }

    #[test]
    fn test_or_pattern() {
        // Use binding patterns to avoid expr-level BitOr consumption
        match parse_pattern("a | b | c") {
            Pattern::Or(alts) => {
                assert_eq!(alts.len(), 3);
                assert!(matches!(&alts[0].node, Pattern::Binding(n) if n == "a"));
                assert!(matches!(&alts[1].node, Pattern::Binding(n) if n == "b"));
                assert!(matches!(&alts[2].node, Pattern::Binding(n) if n == "c"));
            }
            other => panic!("expected Or, got {other:?}"),
        }
    }

    // --- Tuple sequence pattern (#827) ---

    #[test]
    fn test_tuple_sequence_pattern() {
        // (a, b) should parse as Pattern::Sequence, not a literal tuple
        match parse_pattern("(a, b)") {
            Pattern::Sequence(v) => assert_eq!(v.len(), 2),
            other => panic!("expected Sequence, got {other:?}"),
        }

        // () → empty sequence
        match parse_pattern("()") {
            Pattern::Sequence(v) => assert!(v.is_empty()),
            other => panic!("expected empty Sequence, got {other:?}"),
        }

        // (a,) → single-element sequence
        match parse_pattern("(a,)") {
            Pattern::Sequence(v) => assert_eq!(v.len(), 1),
            other => panic!("expected single-element Sequence, got {other:?}"),
        }
    }

    // --- Parser rejection tests for invalid pattern forms ---

    #[test]
    fn test_sequence_multiple_stars_rejected() {
        // PEP 634: at most one starred element allowed in a sequence pattern
        let src = "match x:\n    case [*a, *b]:\n        pass\n";
        let result = parser::parse(src, fid());
        assert!(
            result.is_err(),
            "multiple starred elements should produce a parse error"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("starred") || err_msg.contains("star"),
            "error should mention starred element, got: {err_msg}"
        );
    }

    #[test]
    fn test_class_pattern_positional_after_keyword_rejected() {
        // PEP 634: positional argument may not follow keyword argument
        let src = "match x:\n    case Point(x=1, 2):\n        pass\n";
        let result = parser::parse(src, fid());
        assert!(
            result.is_err(),
            "positional after keyword should produce a parse error"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("positional") || err_msg.contains("keyword"),
            "error should mention positional/keyword ordering, got: {err_msg}"
        );
    }

    #[test]
    fn test_class_pattern_star_inside_rejected() {
        // PEP 634: starred patterns are not allowed in class patterns
        let src = "match x:\n    case Point(*rest):\n        pass\n";
        let result = parser::parse(src, fid());
        assert!(
            result.is_err(),
            "star inside class pattern should produce a parse error"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("starred") || err_msg.contains("star"),
            "error should mention starred pattern, got: {err_msg}"
        );
    }

    // --- Mapping pattern: `**_` is rejected (PEP 634) ---

    #[test]
    fn test_mapping_pattern_double_star_underscore_rejected() {
        // PEP 634 disallows `**_` in mapping patterns; only a real identifier is valid
        let src = "match x:\n    case {**_}:\n        pass\n";
        let result = parser::parse(src, fid());
        assert!(
            result.is_err(),
            "`**_` in mapping pattern should produce a parse error"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("**_") || err_msg.contains("identifier"),
            "error should mention `**_` or identifier, got: {err_msg}"
        );
    }

    // --- Star pattern: only valid inside sequence patterns (#827) ---

    #[test]
    fn test_star_pattern_named_rejected_standalone() {
        // `case *rest:` is invalid PEP 634 — star patterns only inside sequences.
        let src = "match x:\n    case *rest:\n        pass\n";
        let result = parser::parse(src, fid());
        assert!(
            result.is_err(),
            "standalone star pattern should be rejected"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("star") || msg.contains("sequence"),
            "error should mention star or sequence, got: {msg}"
        );
    }

    #[test]
    fn test_star_pattern_anonymous_rejected_standalone() {
        // `case *_:` is invalid PEP 634 — star patterns only inside sequences.
        let src = "match x:\n    case *_:\n        pass\n";
        let result = parser::parse(src, fid());
        assert!(
            result.is_err(),
            "standalone star wildcard should be rejected"
        );
    }

    #[test]
    fn test_star_pattern_valid_in_sequence() {
        // `case [*rest]:` is valid — star pattern inside a sequence.
        match parse_pattern("[*rest]") {
            Pattern::Sequence(pats) => {
                assert_eq!(pats.len(), 1);
                assert!(matches!(&pats[0].node, Pattern::Star(Some(n)) if n == "rest"));
            }
            other => panic!("expected Sequence([Star(rest)]), got {other:?}"),
        }
    }

    // --- Guard ---

    #[test]
    fn test_pattern_with_guard() {
        let src = "match x:\n    case y if y > 0:\n        pass\n";
        let module = parser::parse(src, fid()).expect("parse failed");
        match module.stmts.into_iter().next().unwrap().node {
            Stmt::Match { arms, .. } => {
                assert!(arms[0].guard.is_some());
                assert!(matches!(&arms[0].pattern.node, Pattern::Binding(n) if n == "y"));
            }
            other => panic!("expected Match, got {other:?}"),
        }
    }

    // --- Multiple arms ---

    #[test]
    fn test_match_multiple_arms() {
        let src = "match x:\n    case 1:\n        pass\n    case _:\n        pass\n";
        let module = parser::parse(src, fid()).expect("parse failed");
        match module.stmts.into_iter().next().unwrap().node {
            Stmt::Match { arms, .. } => {
                assert_eq!(arms.len(), 2);
                assert!(matches!(&arms[1].pattern.node, Pattern::Wildcard));
            }
            other => panic!("expected Match, got {other:?}"),
        }
    }

    // --- Built-in type class patterns (#827) ---

    #[test]
    fn test_builtin_int_class_pattern() {
        let src = "match x:\n    case int(n):\n        pass\n";
        let module = parser::parse(src, fid()).expect("parse failed");
        match module.stmts.into_iter().next().unwrap().node {
            Stmt::Match { arms, .. } => match &arms[0].pattern.node {
                Pattern::ClassPattern { cls, patterns } => {
                    assert_eq!(cls, &vec!["int".to_string()]);
                    assert_eq!(patterns.len(), 1);
                    assert_eq!(patterns[0].0, None);
                    assert!(matches!(&patterns[0].1.node, Pattern::Binding(n) if n == "n"));
                }
                other => panic!("expected ClassPattern, got {other:?}"),
            },
            other => panic!("expected Match, got {other:?}"),
        }
    }

    #[test]
    fn test_builtin_str_class_pattern() {
        let src = "match x:\n    case str(s):\n        pass\n";
        let module = parser::parse(src, fid()).expect("parse failed");
        match module.stmts.into_iter().next().unwrap().node {
            Stmt::Match { arms, .. } => match &arms[0].pattern.node {
                Pattern::ClassPattern { cls, patterns } => {
                    assert_eq!(cls, &vec!["str".to_string()]);
                    assert_eq!(patterns.len(), 1);
                    assert_eq!(patterns[0].0, None);
                    assert!(matches!(&patterns[0].1.node, Pattern::Binding(n) if n == "s"));
                }
                other => panic!("expected ClassPattern, got {other:?}"),
            },
            other => panic!("expected Match, got {other:?}"),
        }
    }
}
