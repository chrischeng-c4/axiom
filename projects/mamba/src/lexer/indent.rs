use super::token::{Token, TokenKind};

/// Processes a raw token stream and injects synthetic INDENT/DEDENT tokens
/// based on indentation level changes at the start of each line.
pub struct IndentProcessor {
    indent_stack: Vec<u32>,
    at_line_start: bool,
    paren_depth: u32,
}

impl IndentProcessor {
    pub fn new() -> Self {
        Self {
            indent_stack: vec![0],
            at_line_start: true,
            paren_depth: 0,
        }
    }

    /// Process a stream of raw tokens and return tokens with INDENT/DEDENT injected.
    pub fn process(&mut self, raw_tokens: Vec<Token>) -> Vec<Token> {
        let mut output = Vec::new();

        for token in raw_tokens {
            match &token.kind {
                TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                    // Emit INDENT/DEDENT before incrementing paren depth
                    // so that `{`, `[`, `(` at line start inside a block
                    // still trigger proper indentation handling.
                    if self.at_line_start && self.paren_depth == 0 {
                        self.at_line_start = false;
                        let indent = self.compute_indent(&token, &output);
                        self.emit_indent_dedent(indent, token.start, &mut output);
                    }
                    self.paren_depth += 1;
                    self.at_line_start = false;
                    output.push(token);
                }
                TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                    self.paren_depth = self.paren_depth.saturating_sub(1);
                    self.at_line_start = false;
                    output.push(token);
                }
                TokenKind::Newline => {
                    if self.paren_depth == 0 {
                        output.push(token);
                        self.at_line_start = true;
                    }
                    // Inside parens/brackets, newlines are ignored
                }
                TokenKind::Comment => {
                    // Skip comments entirely
                }
                _ => {
                    if self.at_line_start && self.paren_depth == 0 {
                        self.at_line_start = false;
                        // This is a non-whitespace token at line start.
                        // The indent level is determined by token.start relative
                        // to the start of the current line. We use the column
                        // offset computed during lexing.
                        let indent = self.compute_indent(&token, &output);
                        self.emit_indent_dedent(indent, token.start, &mut output);
                    }
                    output.push(token);
                }
            }
        }

        // Emit remaining DEDENTs at EOF
        let eof_pos = output.last().map(|t| t.end).unwrap_or(0);
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            output.push(Token::new(TokenKind::Dedent, eof_pos, eof_pos));
        }
        output.push(Token::new(TokenKind::Eof, eof_pos, eof_pos));

        output
    }

    fn compute_indent(&self, token: &Token, output: &[Token]) -> u32 {
        // Walk backward from token.start to find the preceding newline
        // The indent is the number of spaces between the newline and token start.
        // We find the last Newline in output and compute offset from there.
        let last_newline_end = output
            .iter()
            .rev()
            .find(|t| t.kind == TokenKind::Newline)
            .map(|t| t.end)
            .unwrap_or(0);
        token.start - last_newline_end
    }

    fn emit_indent_dedent(&mut self, indent: u32, pos: u32, output: &mut Vec<Token>) {
        let current = *self.indent_stack.last().unwrap();

        if indent > current {
            self.indent_stack.push(indent);
            output.push(Token::new(TokenKind::Indent, pos, pos));
        } else if indent < current {
            while let Some(&top) = self.indent_stack.last() {
                if top <= indent {
                    break;
                }
                self.indent_stack.pop();
                output.push(Token::new(TokenKind::Dedent, pos, pos));
            }
        }
    }
}

impl Default for IndentProcessor {
    fn default() -> Self {
        Self::new()
    }
}
