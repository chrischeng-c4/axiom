// pep639.rs — SPDX license-expression parser per PEP 639.
//
// PEP 639 standardizes how Python project metadata records license
// information: `License-Expression` is a single SPDX 2.x expression made of
// identifiers joined by the boolean operators `AND` and `OR`, with `WITH`
// attaching an exception to an individual identifier, and parentheses for
// explicit grouping.
//
// This module accepts identifiers opaquely — it does not validate against
// the SPDX license / exception databases. That validation can be added
// later by checking each `Id { id, exception }` leaf against a static set.
//
// Grammar (SPDX 2.x compound expression, simplified to the subset PEP 639
// requires):
//
//     or_expr  := and_expr ('OR'  and_expr)*
//     and_expr := unit     ('AND' unit)*
//     unit     := id_with | '(' or_expr ')'
//     id_with  := IDENT ('WITH' IDENT)?
//
// Precedence: WITH binds tightest (it forms part of a single leaf), then
// AND, then OR. Operators are left-associative. Operator keywords are
// recognized case-insensitively but rendered in uppercase to match the
// SPDX canonical form.

use crate::pkgmanage::pkgmgr::types::IndexError;

/// One node in a parsed SPDX license expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenseExpr {
    /// A single SPDX license identifier, optionally with an exception.
    Id {
        id: String,
        exception: Option<String>,
    },
    /// `lhs AND rhs`.
    And(Box<LicenseExpr>, Box<LicenseExpr>),
    /// `lhs OR rhs`.
    Or(Box<LicenseExpr>, Box<LicenseExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Ident(String),
    And,
    Or,
    With,
    LParen,
    RParen,
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '+'
}

fn tokenize(src: &str) -> Result<Vec<Token>, IndexError> {
    let mut out = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c == '(' {
            chars.next();
            out.push(Token::LParen);
            continue;
        }
        if c == ')' {
            chars.next();
            out.push(Token::RParen);
            continue;
        }
        if is_ident_char(c) {
            let mut s = String::new();
            while let Some(&c2) = chars.peek() {
                if is_ident_char(c2) {
                    s.push(c2);
                    chars.next();
                } else {
                    break;
                }
            }
            // Keywords are case-insensitive per SPDX.
            let upper = s.to_ascii_uppercase();
            let tok = match upper.as_str() {
                "AND" => Token::And,
                "OR" => Token::Or,
                "WITH" => Token::With,
                _ => Token::Ident(s),
            };
            out.push(tok);
            continue;
        }
        return Err(IndexError::ParseError {
            url: "<license-expression>".into(),
            detail: format!("unexpected character {c:?} in SPDX expression"),
        });
    }
    Ok(out)
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.toks.get(self.pos)
    }

    fn bump(&mut self) -> Option<Token> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn parse_unit(&mut self) -> Result<LicenseExpr, IndexError> {
        match self.bump() {
            Some(Token::LParen) => {
                let inner = self.parse_or()?;
                match self.bump() {
                    Some(Token::RParen) => Ok(inner),
                    Some(other) => Err(IndexError::ParseError {
                        url: "<license-expression>".into(),
                        detail: format!("expected ')' but found {other:?}"),
                    }),
                    None => Err(IndexError::ParseError {
                        url: "<license-expression>".into(),
                        detail: "unclosed '(' in SPDX expression".into(),
                    }),
                }
            }
            Some(Token::Ident(id)) => {
                let exception = if matches!(self.peek(), Some(Token::With)) {
                    self.bump();
                    match self.bump() {
                        Some(Token::Ident(e)) => Some(e),
                        Some(other) => {
                            return Err(IndexError::ParseError {
                                url: "<license-expression>".into(),
                                detail: format!(
                                    "expected exception identifier after WITH, found {other:?}"
                                ),
                            });
                        }
                        None => {
                            return Err(IndexError::ParseError {
                                url: "<license-expression>".into(),
                                detail: "dangling WITH at end of SPDX expression".into(),
                            });
                        }
                    }
                } else {
                    None
                };
                Ok(LicenseExpr::Id { id, exception })
            }
            Some(tok) => Err(IndexError::ParseError {
                url: "<license-expression>".into(),
                detail: format!("expected identifier or '(' but found {tok:?}"),
            }),
            None => Err(IndexError::ParseError {
                url: "<license-expression>".into(),
                detail: "unexpected end of SPDX expression".into(),
            }),
        }
    }

    fn parse_and(&mut self) -> Result<LicenseExpr, IndexError> {
        let mut lhs = self.parse_unit()?;
        while matches!(self.peek(), Some(Token::And)) {
            self.bump();
            let rhs = self.parse_unit()?;
            lhs = LicenseExpr::And(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_or(&mut self) -> Result<LicenseExpr, IndexError> {
        let mut lhs = self.parse_and()?;
        while matches!(self.peek(), Some(Token::Or)) {
            self.bump();
            let rhs = self.parse_and()?;
            lhs = LicenseExpr::Or(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }
}

/// Parse an SPDX license expression. Identifiers are accepted opaquely; no
/// SPDX-database lookup is performed.
pub fn parse_license_expression(src: &str) -> Result<LicenseExpr, IndexError> {
    let toks = tokenize(src)?;
    if toks.is_empty() {
        return Err(IndexError::ParseError {
            url: "<license-expression>".into(),
            detail: "empty SPDX expression".into(),
        });
    }
    let mut p = Parser { toks, pos: 0 };
    let expr = p.parse_or()?;
    if p.pos < p.toks.len() {
        return Err(IndexError::ParseError {
            url: "<license-expression>".into(),
            detail: format!(
                "trailing tokens after SPDX expression at position {}",
                p.pos
            ),
        });
    }
    Ok(expr)
}

/// Render an expression in SPDX canonical form: uppercase keywords, parens
/// only where precedence requires them.
pub fn render_license_expression(expr: &LicenseExpr) -> String {
    render_at(expr, Prec::Or)
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    Or,
    And,
}

fn render_at(expr: &LicenseExpr, ctx: Prec) -> String {
    match expr {
        LicenseExpr::Id { id, exception } => match exception {
            Some(e) => format!("{id} WITH {e}"),
            None => id.clone(),
        },
        LicenseExpr::And(a, b) => {
            let s = format!(
                "{} AND {}",
                render_at(a, Prec::And),
                render_at(b, Prec::And)
            );
            // AND inside OR context needs no parens (AND binds tighter).
            // AND inside AND context is left-associative, also no parens.
            if ctx > Prec::And {
                format!("({s})")
            } else {
                s
            }
        }
        LicenseExpr::Or(a, b) => {
            let s = format!("{} OR {}", render_at(a, Prec::Or), render_at(b, Prec::Or));
            // OR inside AND or higher needs parens.
            if ctx > Prec::Or {
                format!("({s})")
            } else {
                s
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> LicenseExpr {
        LicenseExpr::Id {
            id: s.into(),
            exception: None,
        }
    }

    fn id_with(s: &str, e: &str) -> LicenseExpr {
        LicenseExpr::Id {
            id: s.into(),
            exception: Some(e.into()),
        }
    }

    fn and(a: LicenseExpr, b: LicenseExpr) -> LicenseExpr {
        LicenseExpr::And(Box::new(a), Box::new(b))
    }

    fn or(a: LicenseExpr, b: LicenseExpr) -> LicenseExpr {
        LicenseExpr::Or(Box::new(a), Box::new(b))
    }

    #[test]
    fn simple_identifier() {
        let e = parse_license_expression("MIT").unwrap();
        assert_eq!(e, id("MIT"));
        assert_eq!(render_license_expression(&e), "MIT");
    }

    #[test]
    fn identifier_with_plus_and_dots() {
        // PEP 639 allows `.`, `-`, `+` in SPDX identifiers like `Apache-2.0+`.
        let e = parse_license_expression("Apache-2.0+").unwrap();
        assert_eq!(e, id("Apache-2.0+"));
    }

    #[test]
    fn with_exception() {
        let e = parse_license_expression("GPL-3.0 WITH Classpath-exception-2.0").unwrap();
        assert_eq!(e, id_with("GPL-3.0", "Classpath-exception-2.0"));
        assert_eq!(
            render_license_expression(&e),
            "GPL-3.0 WITH Classpath-exception-2.0"
        );
    }

    #[test]
    fn and_precedence_tighter_than_or() {
        // `A OR B AND C` parses as `A OR (B AND C)`.
        let e = parse_license_expression("MIT OR Apache-2.0 AND BSD-3-Clause").unwrap();
        assert_eq!(e, or(id("MIT"), and(id("Apache-2.0"), id("BSD-3-Clause"))));
        // Canonical render does not insert redundant parens.
        assert_eq!(
            render_license_expression(&e),
            "MIT OR Apache-2.0 AND BSD-3-Clause"
        );
    }

    #[test]
    fn parens_force_or_inside_and() {
        // `(A OR B) AND C` should keep parens on render.
        let e = parse_license_expression("(MIT OR Apache-2.0) AND BSD-3-Clause").unwrap();
        assert_eq!(e, and(or(id("MIT"), id("Apache-2.0")), id("BSD-3-Clause")));
        assert_eq!(
            render_license_expression(&e),
            "(MIT OR Apache-2.0) AND BSD-3-Clause"
        );
    }

    #[test]
    fn keywords_are_case_insensitive() {
        let e = parse_license_expression("mit or apache-2.0 and bsd-3-clause").unwrap();
        // Identifiers themselves preserve their original casing — they are
        // opaque.
        assert_eq!(e, or(id("mit"), and(id("apache-2.0"), id("bsd-3-clause"))));
        // But keywords always render uppercase.
        assert_eq!(
            render_license_expression(&e),
            "mit OR apache-2.0 AND bsd-3-clause"
        );
    }

    #[test]
    fn left_associativity_or() {
        let e = parse_license_expression("MIT OR Apache-2.0 OR BSD-3-Clause").unwrap();
        // Left-associative: (MIT OR Apache-2.0) OR BSD-3-Clause.
        assert_eq!(e, or(or(id("MIT"), id("Apache-2.0")), id("BSD-3-Clause")));
        assert_eq!(
            render_license_expression(&e),
            "MIT OR Apache-2.0 OR BSD-3-Clause"
        );
    }

    #[test]
    fn left_associativity_and() {
        let e = parse_license_expression("MIT AND Apache-2.0 AND BSD-3-Clause").unwrap();
        assert_eq!(e, and(and(id("MIT"), id("Apache-2.0")), id("BSD-3-Clause")));
    }

    #[test]
    fn with_binds_tighter_than_and() {
        // GPL-3.0 WITH Classpath-exception-2.0 AND MIT
        // parses as (GPL-3.0 WITH exc) AND MIT.
        let e = parse_license_expression("GPL-3.0 WITH Classpath-exception-2.0 AND MIT").unwrap();
        assert_eq!(
            e,
            and(id_with("GPL-3.0", "Classpath-exception-2.0"), id("MIT"))
        );
    }

    #[test]
    fn round_trip_canonicalizes_whitespace() {
        let e = parse_license_expression("  MIT   OR    Apache-2.0  ").unwrap();
        assert_eq!(render_license_expression(&e), "MIT OR Apache-2.0");
    }

    #[test]
    fn empty_input_rejected() {
        let err = parse_license_expression("").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("empty"), "got {s}");
    }

    #[test]
    fn whitespace_only_rejected() {
        let err = parse_license_expression("   ").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("empty"), "got {s}");
    }

    #[test]
    fn unclosed_paren_rejected() {
        let err = parse_license_expression("(MIT OR Apache-2.0").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("unclosed"), "got {s}");
    }

    #[test]
    fn dangling_operator_rejected() {
        let err = parse_license_expression("MIT OR").unwrap_err();
        let s = err.to_string();
        assert!(
            s.contains("unexpected end") || s.contains("expected"),
            "got {s}"
        );
    }

    #[test]
    fn dangling_with_rejected() {
        let err = parse_license_expression("GPL-3.0 WITH").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("dangling WITH") || s.contains("WITH"), "got {s}");
    }

    #[test]
    fn with_followed_by_paren_rejected() {
        // Exception must be an identifier, not a parenthesised sub-expression.
        let err = parse_license_expression("GPL-3.0 WITH (Classpath-exception-2.0)").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("exception"), "got {s}");
    }

    #[test]
    fn unexpected_character_rejected() {
        let err = parse_license_expression("MIT & Apache-2.0").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("unexpected character"), "got {s}");
    }

    #[test]
    fn trailing_token_rejected() {
        let err = parse_license_expression("MIT MIT").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("trailing"), "got {s}");
    }

    #[test]
    fn render_nested_or_inside_and_with_with() {
        let e = and(
            or(id("MIT"), id_with("GPL-3.0", "Classpath-exception-2.0")),
            id("BSD-3-Clause"),
        );
        assert_eq!(
            render_license_expression(&e),
            "(MIT OR GPL-3.0 WITH Classpath-exception-2.0) AND BSD-3-Clause"
        );
    }

    #[test]
    fn render_no_redundant_parens_or_inside_or() {
        let e = or(or(id("A"), id("B")), id("C"));
        assert_eq!(render_license_expression(&e), "A OR B OR C");
    }

    #[test]
    fn round_trip_three_way_complex() {
        let src = "(MIT OR Apache-2.0) AND (GPL-3.0 WITH Classpath-exception-2.0 OR BSD-3-Clause)";
        let e = parse_license_expression(src).unwrap();
        assert_eq!(render_license_expression(&e), src);
    }
}
