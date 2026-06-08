use super::ast::TypeExpr;
use super::Parser;
use crate::error::MambaError;
use crate::lexer::token::TokenKind;
use crate::source::span::{Span, Spanned};

impl<'a> Parser<'a> {
    /// Parse a type annotation expression.
    pub fn parse_type_expr(&mut self) -> crate::error::Result<Spanned<TypeExpr>> {
        let mut ty = self.parse_type_atom()?;

        // Check for union: `T | U`
        if self.peek_kind() == Some(TokenKind::Pipe) {
            let mut types = vec![ty];
            while self.peek_kind() == Some(TokenKind::Pipe) {
                self.advance();
                types.push(self.parse_type_atom()?);
            }
            let span = types
                .first()
                .unwrap()
                .span
                .merge(types.last().unwrap().span);
            ty = Spanned::new(TypeExpr::Union(types), span);
        }

        // Check for optional shorthand: `T?`
        if self.peek_kind() == Some(TokenKind::Question) {
            self.advance();
            let span = ty.span.merge(self.span_from(ty.span.start));
            ty = Spanned::new(TypeExpr::Optional(Box::new(ty)), span);
        }

        Ok(ty)
    }

    fn parse_type_atom(&mut self) -> crate::error::Result<Spanned<TypeExpr>> {
        let token = self
            .peek()
            .ok_or_else(|| MambaError::syntax(Span::dummy(), "expected type expression"))?;
        let start = token.start;

        match &token.kind {
            // Built-in type keywords
            TokenKind::IntType
            | TokenKind::FloatType
            | TokenKind::BoolType
            | TokenKind::StrType => {
                let name = self.current_text().to_string();
                self.advance();
                Ok(Spanned::new(TypeExpr::Named(name), self.span_from(start)))
            }
            // Generic container types: list[T], dict[K, V], tuple[T, U]
            TokenKind::ListType | TokenKind::DictType | TokenKind::TupleType => {
                let name = self.current_text().to_string();
                self.advance();
                if self.peek_kind() == Some(TokenKind::LBracket) {
                    self.advance();
                    let mut args = vec![self.parse_type_expr()?];
                    while self.peek_kind() == Some(TokenKind::Comma) {
                        self.advance();
                        args.push(self.parse_type_expr()?);
                    }
                    self.expect(TokenKind::RBracket)?;
                    let span = self.span_from(start);
                    if name == "tuple" {
                        Ok(Spanned::new(TypeExpr::Tuple(args), span))
                    } else {
                        Ok(Spanned::new(TypeExpr::Generic { name, args }, span))
                    }
                } else {
                    Ok(Spanned::new(TypeExpr::Named(name), self.span_from(start)))
                }
            }
            // User-defined type or generic: `MyType`, `MyType[T]`,
            // or dotted reference: `collections.abc.Mapping` (#1576).
            TokenKind::Ident => {
                let mut name = self.current_text().to_string();
                self.advance();
                while self.peek_kind() == Some(TokenKind::Dot) {
                    self.advance();
                    let (s, e) = self.expect_name()?;
                    name.push('.');
                    name.push_str(self.text_at(s, e));
                }
                if self.peek_kind() == Some(TokenKind::LBracket) {
                    self.advance();
                    let mut args = vec![self.parse_type_expr()?];
                    while self.peek_kind() == Some(TokenKind::Comma) {
                        self.advance();
                        args.push(self.parse_type_expr()?);
                    }
                    self.expect(TokenKind::RBracket)?;
                    Ok(Spanned::new(
                        TypeExpr::Generic { name, args },
                        self.span_from(start),
                    ))
                } else {
                    Ok(Spanned::new(TypeExpr::Named(name), self.span_from(start)))
                }
            }
            // Parenthesized type or function type
            TokenKind::LParen => {
                self.advance();
                let mut params = Vec::new();
                while self.peek_kind() != Some(TokenKind::RParen)
                    && self.peek_kind() != Some(TokenKind::Eof)
                {
                    params.push(self.parse_type_expr()?);
                    if self.peek_kind() != Some(TokenKind::RParen) {
                        self.expect(TokenKind::Comma)?;
                    }
                }
                self.expect(TokenKind::RParen)?;

                // Check for arrow -> to make it a function type
                if self.peek_kind() == Some(TokenKind::Arrow) {
                    self.advance();
                    let ret = self.parse_type_expr()?;
                    let span = self.span_from(start);
                    Ok(Spanned::new(
                        TypeExpr::Fn {
                            params,
                            ret: Box::new(ret),
                        },
                        span,
                    ))
                } else if params.len() == 1 {
                    Ok(params.into_iter().next().unwrap())
                } else {
                    Ok(Spanned::new(TypeExpr::Tuple(params), self.span_from(start)))
                }
            }
            // String literal type annotation: `-> "TypeName"` (PEP 484 forward reference).
            // Treat the string content as a type name (resolves to Any if unknown).
            TokenKind::Str(v) | TokenKind::TripleStr(v) | TokenKind::RawStr(v) => {
                let name = v.clone();
                self.advance();
                Ok(Spanned::new(TypeExpr::Named(name), self.span_from(start)))
            }
            TokenKind::None_ => {
                self.advance();
                Ok(Spanned::new(
                    TypeExpr::Named("None".to_string()),
                    self.span_from(start),
                ))
            }
            // `type` used as a type expression (e.g. `type[BaseModel]`).
            // Python's `type[X]` is the builtin generic for the class-object of X.
            // `type` is a soft keyword — in type annotation position it acts as an identifier.
            TokenKind::Type => {
                self.advance(); // consume 'type'
                if self.peek_kind() == Some(TokenKind::LBracket) {
                    self.advance(); // consume '['
                    let mut args = vec![self.parse_type_expr()?];
                    while self.peek_kind() == Some(TokenKind::Comma) {
                        self.advance();
                        args.push(self.parse_type_expr()?);
                    }
                    self.expect(TokenKind::RBracket)?;
                    Ok(Spanned::new(
                        TypeExpr::Generic {
                            name: "type".to_string(),
                            args,
                        },
                        self.span_from(start),
                    ))
                } else {
                    Ok(Spanned::new(
                        TypeExpr::Named("type".to_string()),
                        self.span_from(start),
                    ))
                }
            }
            // TypeVarTuple spread: `*Ts` used in type position (e.g. `tuple[*Ts]`)
            TokenKind::Star => {
                self.advance(); // consume *
                let (s, e) = self.expect_name()?;
                let name = self.text_at(s, e).to_string();
                Ok(Spanned::new(TypeExpr::Named(name), self.span_from(start)))
            }
            _ => Err(MambaError::syntax(
                Span::new(self.file_id, start, token.end),
                format!("expected type, got {}", token.kind),
            )),
        }
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
    /// Parse `x: <type> = 0` and return the TypeExpr.
    fn parse_type(ty_src: &str) -> TypeExpr {
        let src = format!("x: {ty_src} = 0\n");
        let module = parser::parse(&src, fid()).expect("parse failed");
        match module.stmts.into_iter().next().unwrap().node {
            Stmt::VarDecl { ty, .. } => ty.node,
            other => panic!("expected VarDecl, got {other:?}"),
        }
    }

    // --- Named types ---

    #[test]
    fn test_named_int() {
        match parse_type("int") {
            TypeExpr::Named(n) => assert_eq!(n, "int"),
            other => panic!("expected Named('int'), got {other:?}"),
        }
    }

    #[test]
    fn test_named_float() {
        match parse_type("float") {
            TypeExpr::Named(n) => assert_eq!(n, "float"),
            other => panic!("expected Named('float'), got {other:?}"),
        }
    }

    #[test]
    fn test_named_bool() {
        match parse_type("bool") {
            TypeExpr::Named(n) => assert_eq!(n, "bool"),
            other => panic!("expected Named('bool'), got {other:?}"),
        }
    }

    #[test]
    fn test_named_str() {
        match parse_type("str") {
            TypeExpr::Named(n) => assert_eq!(n, "str"),
            other => panic!("expected Named('str'), got {other:?}"),
        }
    }

    #[test]
    fn test_named_user_type() {
        match parse_type("MyClass") {
            TypeExpr::Named(n) => assert_eq!(n, "MyClass"),
            other => panic!("expected Named('MyClass'), got {other:?}"),
        }
    }

    #[test]
    fn test_named_none() {
        match parse_type("None") {
            TypeExpr::Named(n) => assert_eq!(n, "None"),
            other => panic!("expected Named('None'), got {other:?}"),
        }
    }

    // --- Generic types ---

    #[test]
    fn test_generic_list_int() {
        match parse_type("list[int]") {
            TypeExpr::Generic { name, args } => {
                assert_eq!(name, "list");
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0].node, TypeExpr::Named(n) if n == "int"));
            }
            other => panic!("expected Generic, got {other:?}"),
        }
    }

    #[test]
    fn test_generic_dict() {
        match parse_type("dict[str, int]") {
            TypeExpr::Generic { name, args } => {
                assert_eq!(name, "dict");
                assert_eq!(args.len(), 2);
                assert!(matches!(&args[0].node, TypeExpr::Named(n) if n == "str"));
                assert!(matches!(&args[1].node, TypeExpr::Named(n) if n == "int"));
            }
            other => panic!("expected Generic, got {other:?}"),
        }
    }

    #[test]
    fn test_generic_user_type() {
        match parse_type("Optional[int]") {
            TypeExpr::Generic { name, args } => {
                assert_eq!(name, "Optional");
                assert_eq!(args.len(), 1);
            }
            other => panic!("expected Generic, got {other:?}"),
        }
    }

    // --- Tuple type ---

    #[test]
    fn test_tuple_type() {
        match parse_type("tuple[int, str]") {
            TypeExpr::Tuple(args) => {
                assert_eq!(args.len(), 2);
                assert!(matches!(&args[0].node, TypeExpr::Named(n) if n == "int"));
                assert!(matches!(&args[1].node, TypeExpr::Named(n) if n == "str"));
            }
            other => panic!("expected Tuple, got {other:?}"),
        }
    }

    // --- Optional ---

    #[test]
    fn test_optional_shorthand() {
        match parse_type("int?") {
            TypeExpr::Optional(inner) => {
                assert!(matches!(&inner.node, TypeExpr::Named(n) if n == "int"));
            }
            other => panic!("expected Optional, got {other:?}"),
        }
    }

    // --- Union ---

    #[test]
    fn test_union_two() {
        match parse_type("int | str") {
            TypeExpr::Union(types) => {
                assert_eq!(types.len(), 2);
                assert!(matches!(&types[0].node, TypeExpr::Named(n) if n == "int"));
                assert!(matches!(&types[1].node, TypeExpr::Named(n) if n == "str"));
            }
            other => panic!("expected Union, got {other:?}"),
        }
    }

    #[test]
    fn test_union_three() {
        match parse_type("int | str | float") {
            TypeExpr::Union(types) => {
                assert_eq!(types.len(), 3);
            }
            other => panic!("expected Union with 3, got {other:?}"),
        }
    }

    // --- Function type ---

    #[test]
    fn test_fn_type() {
        match parse_type("(int, str) -> bool") {
            TypeExpr::Fn { params, ret } => {
                assert_eq!(params.len(), 2);
                assert!(matches!(&ret.node, TypeExpr::Named(n) if n == "bool"));
            }
            other => panic!("expected Fn, got {other:?}"),
        }
    }

    #[test]
    fn test_fn_type_no_params() {
        match parse_type("() -> int") {
            TypeExpr::Fn { params, ret } => {
                assert!(params.is_empty());
                assert!(matches!(&ret.node, TypeExpr::Named(n) if n == "int"));
            }
            other => panic!("expected Fn, got {other:?}"),
        }
    }

    // --- Parenthesized type ---

    #[test]
    fn test_parenthesized_type() {
        // `(int)` should unwrap to just `int`
        match parse_type("(int)") {
            TypeExpr::Named(n) => assert_eq!(n, "int"),
            other => panic!("expected Named through paren, got {other:?}"),
        }
    }

    // --- Bare container without brackets ---

    #[test]
    fn test_list_without_brackets() {
        // `list` without brackets is just Named("list")
        match parse_type("list") {
            TypeExpr::Named(n) => assert_eq!(n, "list"),
            other => panic!("expected Named('list'), got {other:?}"),
        }
    }

    // --- Nested generics ---

    #[test]
    fn test_nested_generic() {
        match parse_type("list[dict[str, int]]") {
            TypeExpr::Generic { name, args } => {
                assert_eq!(name, "list");
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0].node, TypeExpr::Generic { .. }));
            }
            other => panic!("expected nested Generic, got {other:?}"),
        }
    }

    // --- Error cases ---

    #[test]
    fn test_type_error_on_invalid_token() {
        // Using a number as a type should fail
        let src = "x: 42 = 0\n";
        let result = parser::parse(src, fid());
        assert!(result.is_err());
    }

    // --- Optional after union ---

    #[test]
    fn test_optional_after_union() {
        // `int | str?` should make Optional around the union
        match parse_type("int | str?") {
            // The `?` applies after union is fully parsed
            TypeExpr::Optional(inner) => {
                assert!(matches!(&inner.node, TypeExpr::Union(_)));
            }
            other => panic!("expected Optional(Union), got {other:?}"),
        }
    }
}
