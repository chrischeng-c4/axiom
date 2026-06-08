#![cfg(test)]

use crate::lexer;
use crate::lexer::token::TokenKind;
use crate::source::span::FileId;

fn lex(src: &str) -> Vec<TokenKind> {
    let tokens = lexer::lex(src, FileId(0));
    tokens.into_iter().map(|t| t.kind).collect()
}

fn lex_raw(src: &str) -> Vec<TokenKind> {
    let tokens = lexer::lex_raw(src, FileId(0));
    tokens.into_iter().map(|t| t.kind).collect()
}

#[test]
fn test_keywords() {
    let kinds = lex_raw("def return if elif else while for in class enum match case");
    assert!(kinds.contains(&TokenKind::Def));
    assert!(kinds.contains(&TokenKind::Return));
    assert!(kinds.contains(&TokenKind::If));
    assert!(kinds.contains(&TokenKind::Elif));
    assert!(kinds.contains(&TokenKind::Else));
    assert!(kinds.contains(&TokenKind::While));
    assert!(kinds.contains(&TokenKind::For));
    assert!(kinds.contains(&TokenKind::In));
    assert!(kinds.contains(&TokenKind::Class));
    assert!(kinds.contains(&TokenKind::Enum));
    assert!(kinds.contains(&TokenKind::Match));
    assert!(kinds.contains(&TokenKind::Case));
}

#[test]
fn test_integer_literal() {
    let kinds = lex_raw("42");
    assert!(kinds.contains(&TokenKind::Int(42)));
}

#[test]
fn test_float_literal() {
    let kinds = lex_raw("3.14");
    assert!(kinds.contains(&TokenKind::Float(3.14)));
}

#[test]
fn test_string_literal() {
    let kinds = lex_raw("\"hello\"");
    assert!(kinds.contains(&TokenKind::Str("hello".to_string())));
}

#[test]
fn test_single_quote_string() {
    let kinds = lex_raw("'world'");
    assert!(kinds.contains(&TokenKind::Str("world".to_string())));
}

#[test]
fn test_operators() {
    let kinds = lex_raw("+ - * / // % ** == != < > <= >= -> | ?");
    assert!(kinds.contains(&TokenKind::Plus));
    assert!(kinds.contains(&TokenKind::Minus));
    assert!(kinds.contains(&TokenKind::Star));
    assert!(kinds.contains(&TokenKind::Slash));
    assert!(kinds.contains(&TokenKind::DoubleSlash));
    assert!(kinds.contains(&TokenKind::Percent));
    assert!(kinds.contains(&TokenKind::DoubleStar));
    assert!(kinds.contains(&TokenKind::EqEq));
    assert!(kinds.contains(&TokenKind::NotEq));
    assert!(kinds.contains(&TokenKind::Arrow));
    assert!(kinds.contains(&TokenKind::Pipe));
    assert!(kinds.contains(&TokenKind::Question));
}

#[test]
fn test_delimiters() {
    let kinds = lex_raw("( ) [ ] { } : , .");
    assert!(kinds.contains(&TokenKind::LParen));
    assert!(kinds.contains(&TokenKind::RParen));
    assert!(kinds.contains(&TokenKind::LBracket));
    assert!(kinds.contains(&TokenKind::RBracket));
    assert!(kinds.contains(&TokenKind::LBrace));
    assert!(kinds.contains(&TokenKind::RBrace));
    assert!(kinds.contains(&TokenKind::Colon));
    assert!(kinds.contains(&TokenKind::Comma));
    assert!(kinds.contains(&TokenKind::Dot));
}

#[test]
fn test_indent_dedent() {
    let src = "if True:\n    x = 1\ny = 2\n";
    let kinds = lex(src);
    assert!(kinds.contains(&TokenKind::Indent));
    assert!(kinds.contains(&TokenKind::Dedent));
    assert!(kinds.contains(&TokenKind::Eof));
}

#[test]
fn test_nested_indent() {
    let src = "if True:\n    if True:\n        x = 1\n";
    let kinds = lex(src);
    let indent_count = kinds.iter().filter(|k| **k == TokenKind::Indent).count();
    let dedent_count = kinds.iter().filter(|k| **k == TokenKind::Dedent).count();
    assert_eq!(indent_count, 2);
    assert_eq!(dedent_count, 2);
}

#[test]
fn test_boolean_none() {
    let kinds = lex_raw("True False None");
    assert!(kinds.contains(&TokenKind::True));
    assert!(kinds.contains(&TokenKind::False));
    assert!(kinds.contains(&TokenKind::None_));
}

#[test]
fn test_comment_ignored() {
    let kinds = lex_raw("x # this is a comment\ny");
    // Comments should be present in raw lex
    assert!(kinds.contains(&TokenKind::Comment));
}

#[test]
fn test_type_keywords() {
    let kinds = lex_raw("int float bool str list dict tuple");
    assert!(kinds.contains(&TokenKind::IntType));
    assert!(kinds.contains(&TokenKind::FloatType));
    assert!(kinds.contains(&TokenKind::BoolType));
    assert!(kinds.contains(&TokenKind::StrType));
    assert!(kinds.contains(&TokenKind::ListType));
    assert!(kinds.contains(&TokenKind::DictType));
    assert!(kinds.contains(&TokenKind::TupleType));
}
