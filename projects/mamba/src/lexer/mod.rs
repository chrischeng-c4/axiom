pub mod indent;
pub mod token;

use logos::Logos;

use crate::source::span::{FileId, Span};
use indent::IndentProcessor;
use token::{Token, TokenKind};

/// Lex a source string into a token stream with INDENT/DEDENT processing.
pub fn lex(source: &str, file_id: FileId) -> Vec<Token> {
    let raw_tokens = lex_raw(source, file_id);
    let mut processor = IndentProcessor::new();
    processor.process(raw_tokens)
}

/// Lex into raw tokens (no INDENT/DEDENT). Used for testing.
pub fn lex_raw(source: &str, _file_id: FileId) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut lexer = TokenKind::lexer(source);

    while let Some(result) = lexer.next() {
        let span = lexer.span();
        match result {
            Ok(kind) => {
                tokens.push(Token::new(kind, span.start as u32, span.end as u32));
            }
            Err(()) => {
                // Skip unrecognized characters
            }
        }
    }

    tokens
}

/// Create a Span from a Token and FileId.
pub fn token_span(token: &Token, file_id: FileId) -> Span {
    Span::new(file_id, token.start, token.end)
}

#[cfg(test)]
#[path = "tests"]
mod tests_files {
    mod core;
}
