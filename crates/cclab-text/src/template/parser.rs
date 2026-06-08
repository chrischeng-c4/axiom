//! Template parser for Jinja2-compatible template syntax.
//!
//! Parses template strings into a list of AST nodes.

use std::fmt;

/// A parsed template node.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// Raw text to output directly.
    Text(String),
    /// Variable expression: {{ expr }}
    Expression(Expr),
    /// If/elif/else conditional block.
    If {
        condition: Expr,
        body: Vec<Node>,
        elif_branches: Vec<(Expr, Vec<Node>)>,
        else_body: Vec<Node>,
    },
    /// For loop: {% for var in iterable %}
    For {
        variable: String,
        iterable: Expr,
        body: Vec<Node>,
        else_body: Vec<Node>,
    },
    /// Block definition for template inheritance: {% block name %}
    Block { name: String, body: Vec<Node> },
    /// Extends directive for template inheritance: {% extends "base.html" %}
    Extends(String),
    /// Include directive: {% include "partial.html" %}
    Include(String),
    /// Set variable: {% set name = expr %}
    Set { name: String, value: Expr },
    /// Raw block (no processing): {% raw %}...{% endraw %}
    Raw(String),
    /// Comment: {# comment #}
    Comment(String),
}

/// An expression in the template.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// String literal.
    StringLit(String),
    /// Integer literal.
    IntLit(i64),
    /// Float literal.
    FloatLit(f64),
    /// Boolean literal.
    BoolLit(bool),
    /// None literal.
    NoneLit,
    /// Variable reference.
    Variable(String),
    /// Attribute access: obj.attr
    Attribute(Box<Expr>, String),
    /// Index access: obj[idx]
    Index(Box<Expr>, Box<Expr>),
    /// Filter application: expr | filter(args...)
    Filter {
        expr: Box<Expr>,
        name: String,
        args: Vec<Expr>,
    },
    /// Binary operation.
    BinOp {
        left: Box<Expr>,
        op: BinOperator,
        right: Box<Expr>,
    },
    /// Unary not.
    Not(Box<Expr>),
    /// List literal: [a, b, c]
    List(Vec<Expr>),
}

/// Binary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum BinOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    In,
}

impl fmt::Display for BinOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOperator::Add => write!(f, "+"),
            BinOperator::Sub => write!(f, "-"),
            BinOperator::Mul => write!(f, "*"),
            BinOperator::Div => write!(f, "/"),
            BinOperator::Mod => write!(f, "%"),
            BinOperator::Eq => write!(f, "=="),
            BinOperator::Ne => write!(f, "!="),
            BinOperator::Lt => write!(f, "<"),
            BinOperator::Gt => write!(f, ">"),
            BinOperator::Le => write!(f, "<="),
            BinOperator::Ge => write!(f, ">="),
            BinOperator::And => write!(f, "and"),
            BinOperator::Or => write!(f, "or"),
            BinOperator::In => write!(f, "in"),
        }
    }
}

/// Template parse error.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at position {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// Parse a template string into a list of AST nodes.
pub fn parse(template: &str) -> Result<Vec<Node>, ParseError> {
    let mut parser = Parser::new(template);
    parser.parse_nodes(&[])
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn parse_nodes(&mut self, end_tags: &[&str]) -> Result<Vec<Node>, ParseError> {
        let mut nodes = Vec::new();

        while self.pos < self.input.len() {
            // Check for end tags
            if !end_tags.is_empty() {
                let saved_pos = self.pos;
                if self.remaining().starts_with("{%") {
                    let tag_content = self.peek_tag_content();
                    if let Some(content) = tag_content {
                        let trimmed = content.trim();
                        for end_tag in end_tags {
                            if trimmed == *end_tag || trimmed.starts_with(end_tag) {
                                return Ok(nodes);
                            }
                        }
                    }
                    self.pos = saved_pos;
                }
            }

            if self.remaining().starts_with("{#") {
                let node = self.parse_comment()?;
                nodes.push(node);
            } else if self.remaining().starts_with("{{") {
                let node = self.parse_expression_tag()?;
                nodes.push(node);
            } else if self.remaining().starts_with("{%") {
                let node = self.parse_statement()?;
                nodes.push(node);
            } else {
                let node = self.parse_text();
                if let Node::Text(ref t) = node {
                    if !t.is_empty() {
                        nodes.push(node);
                    }
                }
            }
        }

        if !end_tags.is_empty() {
            return Err(ParseError {
                message: format!("Expected one of: {:?}", end_tags),
                position: self.pos,
            });
        }

        Ok(nodes)
    }

    fn peek_tag_content(&self) -> Option<String> {
        let remaining = self.remaining();
        if !remaining.starts_with("{%") {
            return None;
        }
        let start = 2;
        // Handle whitespace-stripping tag {%-
        let content_start = if remaining[start..].starts_with('-') {
            start + 1
        } else {
            start
        };
        if let Some(end) = remaining.find("%}") {
            let content = &remaining[content_start..end];
            // Handle whitespace-stripping -%}
            let content = content.strip_suffix('-').unwrap_or(content);
            Some(content.to_string())
        } else {
            None
        }
    }

    fn parse_text(&mut self) -> Node {
        let start = self.pos;
        while self.pos < self.input.len() {
            if self.remaining().starts_with("{{")
                || self.remaining().starts_with("{%")
                || self.remaining().starts_with("{#")
            {
                break;
            }
            self.pos += self.input[self.pos..]
                .chars()
                .next()
                .map_or(1, |c| c.len_utf8());
        }
        Node::Text(self.input[start..self.pos].to_string())
    }

    fn parse_comment(&mut self) -> Result<Node, ParseError> {
        self.pos += 2; // skip {#
        let start = self.pos;
        if let Some(end_offset) = self.remaining().find("#}") {
            let comment = self.input[start..self.pos + end_offset].to_string();
            self.pos += end_offset + 2;
            Ok(Node::Comment(comment.trim().to_string()))
        } else {
            Err(ParseError {
                message: "Unclosed comment".to_string(),
                position: start - 2,
            })
        }
    }

    fn parse_expression_tag(&mut self) -> Result<Node, ParseError> {
        self.pos += 2; // skip {{
                       // Handle whitespace-stripping {{-
        if self.remaining().starts_with('-') {
            self.pos += 1;
        }
        self.skip_whitespace();
        let expr = self.parse_expr()?;
        self.skip_whitespace();
        // Handle whitespace-stripping -}}
        if self.remaining().starts_with('-') {
            self.pos += 1;
        }
        if !self.remaining().starts_with("}}") {
            return Err(ParseError {
                message: "Expected }}".to_string(),
                position: self.pos,
            });
        }
        self.pos += 2;
        Ok(Node::Expression(expr))
    }

    fn parse_statement(&mut self) -> Result<Node, ParseError> {
        self.pos += 2; // skip {%
                       // Handle whitespace-stripping {%-
        if self.remaining().starts_with('-') {
            self.pos += 1;
        }
        self.skip_whitespace();

        let keyword = self.read_identifier()?;
        self.skip_whitespace();

        match keyword.as_str() {
            "if" => self.parse_if_statement(),
            "for" => self.parse_for_statement(),
            "block" => self.parse_block_statement(),
            "extends" => self.parse_extends_statement(),
            "include" => self.parse_include_statement(),
            "set" => self.parse_set_statement(),
            "raw" => self.parse_raw_block(),
            _ => Err(ParseError {
                message: format!("Unknown statement: {}", keyword),
                position: self.pos,
            }),
        }
    }

    fn parse_if_statement(&mut self) -> Result<Node, ParseError> {
        let condition = self.parse_expr()?;
        self.close_tag()?;

        let body = self.parse_nodes(&["endif", "elif", "else"])?;

        let mut elif_branches = Vec::new();
        let mut else_body = Vec::new();

        loop {
            let tag = self.consume_tag_keyword()?;
            match tag.as_str() {
                "endif" => {
                    self.close_tag()?;
                    break;
                }
                "elif" => {
                    self.skip_whitespace();
                    let elif_cond = self.parse_expr()?;
                    self.close_tag()?;
                    let elif_body = self.parse_nodes(&["endif", "elif", "else"])?;
                    elif_branches.push((elif_cond, elif_body));
                }
                "else" => {
                    self.close_tag()?;
                    else_body = self.parse_nodes(&["endif"])?;
                    self.consume_tag_keyword()?; // consume "endif"
                    self.close_tag()?;
                    break;
                }
                _ => {
                    return Err(ParseError {
                        message: format!("Unexpected tag: {}", tag),
                        position: self.pos,
                    });
                }
            }
        }

        Ok(Node::If {
            condition,
            body,
            elif_branches,
            else_body,
        })
    }

    fn parse_for_statement(&mut self) -> Result<Node, ParseError> {
        let variable = self.read_identifier()?;
        self.skip_whitespace();

        // Expect "in"
        let in_kw = self.read_identifier()?;
        if in_kw != "in" {
            return Err(ParseError {
                message: format!("Expected 'in', got '{}'", in_kw),
                position: self.pos,
            });
        }
        self.skip_whitespace();

        let iterable = self.parse_expr()?;
        self.close_tag()?;

        let body = self.parse_nodes(&["endfor", "else"])?;
        let mut else_body = Vec::new();

        let tag = self.consume_tag_keyword()?;
        match tag.as_str() {
            "endfor" => {
                self.close_tag()?;
            }
            "else" => {
                self.close_tag()?;
                else_body = self.parse_nodes(&["endfor"])?;
                self.consume_tag_keyword()?;
                self.close_tag()?;
            }
            _ => {
                return Err(ParseError {
                    message: format!("Expected endfor or else, got '{}'", tag),
                    position: self.pos,
                });
            }
        }

        Ok(Node::For {
            variable,
            iterable,
            body,
            else_body,
        })
    }

    fn parse_block_statement(&mut self) -> Result<Node, ParseError> {
        let name = self.read_identifier()?;
        self.close_tag()?;
        let body = self.parse_nodes(&["endblock"])?;
        self.consume_tag_keyword()?;
        self.skip_whitespace();
        // Optional block name after endblock
        if !self.remaining().starts_with("%}") && !self.remaining().starts_with("-%}") {
            let _ = self.read_identifier(); // optional name
        }
        self.close_tag()?;
        Ok(Node::Block { name, body })
    }

    fn parse_extends_statement(&mut self) -> Result<Node, ParseError> {
        let name = self.parse_string_literal()?;
        self.close_tag()?;
        Ok(Node::Extends(name))
    }

    fn parse_include_statement(&mut self) -> Result<Node, ParseError> {
        let name = self.parse_string_literal()?;
        self.close_tag()?;
        Ok(Node::Include(name))
    }

    fn parse_set_statement(&mut self) -> Result<Node, ParseError> {
        let name = self.read_identifier()?;
        self.skip_whitespace();
        if !self.remaining().starts_with('=') {
            return Err(ParseError {
                message: "Expected '=' in set statement".to_string(),
                position: self.pos,
            });
        }
        self.pos += 1;
        self.skip_whitespace();
        let value = self.parse_expr()?;
        self.close_tag()?;
        Ok(Node::Set { name, value })
    }

    fn parse_raw_block(&mut self) -> Result<Node, ParseError> {
        self.close_tag()?;
        let start = self.pos;
        let marker = "{% endraw %}";
        if let Some(end_offset) = self.remaining().find(marker) {
            let raw_text = self.input[start..self.pos + end_offset].to_string();
            self.pos += end_offset + marker.len();
            Ok(Node::Raw(raw_text))
        } else {
            Err(ParseError {
                message: "Unclosed raw block".to_string(),
                position: start,
            })
        }
    }

    fn consume_tag_keyword(&mut self) -> Result<String, ParseError> {
        // We should be at {%
        if !self.remaining().starts_with("{%") {
            return Err(ParseError {
                message: "Expected {%".to_string(),
                position: self.pos,
            });
        }
        self.pos += 2;
        if self.remaining().starts_with('-') {
            self.pos += 1;
        }
        self.skip_whitespace();
        self.read_identifier()
    }

    fn close_tag(&mut self) -> Result<(), ParseError> {
        self.skip_whitespace();
        if self.remaining().starts_with("-%}") {
            self.pos += 3;
            Ok(())
        } else if self.remaining().starts_with("%}") {
            self.pos += 2;
            Ok(())
        } else {
            Err(ParseError {
                message: "Expected %}".to_string(),
                position: self.pos,
            })
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expr()?;
        while self.try_consume_keyword("or") {
            let right = self.parse_and_expr()?;
            left = Expr::BinOp {
                left: Box::new(left),
                op: BinOperator::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not_expr()?;
        while self.try_consume_keyword("and") {
            let right = self.parse_not_expr()?;
            left = Expr::BinOp {
                left: Box::new(left),
                op: BinOperator::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_not_expr(&mut self) -> Result<Expr, ParseError> {
        if self.try_consume_keyword("not") {
            let expr = self.parse_comparison()?;
            Ok(Expr::Not(Box::new(expr)))
        } else {
            self.parse_comparison()
        }
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_additive()?;
        self.skip_whitespace();

        let op = if self.remaining().starts_with("==") {
            self.pos += 2;
            Some(BinOperator::Eq)
        } else if self.remaining().starts_with("!=") {
            self.pos += 2;
            Some(BinOperator::Ne)
        } else if self.remaining().starts_with("<=") {
            self.pos += 2;
            Some(BinOperator::Le)
        } else if self.remaining().starts_with(">=") {
            self.pos += 2;
            Some(BinOperator::Ge)
        } else if self.remaining().starts_with('<') {
            self.pos += 1;
            Some(BinOperator::Lt)
        } else if self.remaining().starts_with('>') {
            self.pos += 1;
            Some(BinOperator::Gt)
        } else if self.try_consume_keyword("in") {
            Some(BinOperator::In)
        } else {
            None
        };

        if let Some(op) = op {
            self.skip_whitespace();
            let right = self.parse_additive()?;
            Ok(Expr::BinOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            self.skip_whitespace();
            if self.remaining().starts_with('+') {
                self.pos += 1;
                self.skip_whitespace();
                let right = self.parse_multiplicative()?;
                left = Expr::BinOp {
                    left: Box::new(left),
                    op: BinOperator::Add,
                    right: Box::new(right),
                };
            } else if self.remaining().starts_with('-')
                && !self.remaining().starts_with("-%}")
                && !self.remaining().starts_with("-}}")
            {
                self.pos += 1;
                self.skip_whitespace();
                let right = self.parse_multiplicative()?;
                left = Expr::BinOp {
                    left: Box::new(left),
                    op: BinOperator::Sub,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_filter_expr()?;
        loop {
            self.skip_whitespace();
            if self.remaining().starts_with('*') {
                self.pos += 1;
                self.skip_whitespace();
                let right = self.parse_filter_expr()?;
                left = Expr::BinOp {
                    left: Box::new(left),
                    op: BinOperator::Mul,
                    right: Box::new(right),
                };
            } else if self.remaining().starts_with('/') {
                self.pos += 1;
                self.skip_whitespace();
                let right = self.parse_filter_expr()?;
                left = Expr::BinOp {
                    left: Box::new(left),
                    op: BinOperator::Div,
                    right: Box::new(right),
                };
            } else if self.remaining().starts_with('%') && !self.remaining().starts_with("%}") {
                self.pos += 1;
                self.skip_whitespace();
                let right = self.parse_filter_expr()?;
                left = Expr::BinOp {
                    left: Box::new(left),
                    op: BinOperator::Mod,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_filter_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_postfix()?;

        loop {
            self.skip_whitespace();
            if !self.remaining().starts_with('|') {
                break;
            }
            self.pos += 1;
            self.skip_whitespace();
            let name = self.read_identifier()?;
            let mut args = Vec::new();

            self.skip_whitespace();
            if self.remaining().starts_with('(') {
                self.pos += 1;
                self.skip_whitespace();
                if !self.remaining().starts_with(')') {
                    args.push(self.parse_expr()?);
                    while self.remaining().starts_with(',') {
                        self.pos += 1;
                        self.skip_whitespace();
                        args.push(self.parse_expr()?);
                    }
                }
                self.skip_whitespace();
                if !self.remaining().starts_with(')') {
                    return Err(ParseError {
                        message: "Expected )".to_string(),
                        position: self.pos,
                    });
                }
                self.pos += 1;
            }

            expr = Expr::Filter {
                expr: Box::new(expr),
                name,
                args,
            };
        }

        Ok(expr)
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.remaining().starts_with('.') {
                self.pos += 1;
                let attr = self.read_identifier()?;
                expr = Expr::Attribute(Box::new(expr), attr);
            } else if self.remaining().starts_with('[') {
                self.pos += 1;
                self.skip_whitespace();
                let idx = self.parse_expr()?;
                self.skip_whitespace();
                if !self.remaining().starts_with(']') {
                    return Err(ParseError {
                        message: "Expected ]".to_string(),
                        position: self.pos,
                    });
                }
                self.pos += 1;
                expr = Expr::Index(Box::new(expr), Box::new(idx));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        self.skip_whitespace();

        // String literal
        if self.remaining().starts_with('"') || self.remaining().starts_with('\'') {
            let s = self.parse_string_literal()?;
            return Ok(Expr::StringLit(s));
        }

        // List literal
        if self.remaining().starts_with('[') {
            return self.parse_list_literal();
        }

        // Parenthesized expression
        if self.remaining().starts_with('(') {
            self.pos += 1;
            self.skip_whitespace();
            let expr = self.parse_expr()?;
            self.skip_whitespace();
            if !self.remaining().starts_with(')') {
                return Err(ParseError {
                    message: "Expected )".to_string(),
                    position: self.pos,
                });
            }
            self.pos += 1;
            return Ok(expr);
        }

        // Number literal (including negative)
        if self.remaining().starts_with(|c: char| c.is_ascii_digit())
            || (self.remaining().starts_with('-')
                && self.remaining().len() > 1
                && self.remaining().as_bytes()[1].is_ascii_digit())
        {
            return self.parse_number();
        }

        // Identifier or keyword
        let ident = self.read_identifier()?;
        match ident.as_str() {
            "true" | "True" => Ok(Expr::BoolLit(true)),
            "false" | "False" => Ok(Expr::BoolLit(false)),
            "none" | "None" => Ok(Expr::NoneLit),
            _ => Ok(Expr::Variable(ident)),
        }
    }

    fn parse_list_literal(&mut self) -> Result<Expr, ParseError> {
        self.pos += 1; // skip [
        self.skip_whitespace();
        let mut items = Vec::new();

        if !self.remaining().starts_with(']') {
            items.push(self.parse_expr()?);
            while self.remaining().starts_with(',') {
                self.pos += 1;
                self.skip_whitespace();
                if self.remaining().starts_with(']') {
                    break; // trailing comma
                }
                items.push(self.parse_expr()?);
            }
        }

        self.skip_whitespace();
        if !self.remaining().starts_with(']') {
            return Err(ParseError {
                message: "Expected ]".to_string(),
                position: self.pos,
            });
        }
        self.pos += 1;
        Ok(Expr::List(items))
    }

    fn parse_number(&mut self) -> Result<Expr, ParseError> {
        let start = self.pos;
        if self.remaining().starts_with('-') {
            self.pos += 1;
        }
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'.' {
            self.pos += 1;
            while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            let s = &self.input[start..self.pos];
            let f: f64 = s.parse().map_err(|_| ParseError {
                message: format!("Invalid float: {}", s),
                position: start,
            })?;
            Ok(Expr::FloatLit(f))
        } else {
            let s = &self.input[start..self.pos];
            let i: i64 = s.parse().map_err(|_| ParseError {
                message: format!("Invalid integer: {}", s),
                position: start,
            })?;
            Ok(Expr::IntLit(i))
        }
    }

    fn parse_string_literal(&mut self) -> Result<String, ParseError> {
        let quote = self.input.as_bytes()[self.pos] as char;
        self.pos += 1;
        let start = self.pos;
        let mut result = String::new();

        while self.pos < self.input.len() {
            let ch = self.input.as_bytes()[self.pos] as char;
            if ch == '\\' && self.pos + 1 < self.input.len() {
                let next = self.input.as_bytes()[self.pos + 1] as char;
                match next {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    '\\' => result.push('\\'),
                    '\'' => result.push('\''),
                    '"' => result.push('"'),
                    _ => {
                        result.push('\\');
                        result.push(next);
                    }
                }
                self.pos += 2;
            } else if ch == quote {
                self.pos += 1;
                return Ok(result);
            } else {
                result.push(ch);
                self.pos += 1;
            }
        }

        Err(ParseError {
            message: "Unclosed string literal".to_string(),
            position: start - 1,
        })
    }

    fn read_identifier(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        while self.pos < self.input.len() {
            let ch = self.input.as_bytes()[self.pos];
            if ch.is_ascii_alphanumeric() || ch == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(ParseError {
                message: "Expected identifier".to_string(),
                position: self.pos,
            });
        }
        Ok(self.input[start..self.pos].to_string())
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn try_consume_keyword(&mut self, keyword: &str) -> bool {
        self.skip_whitespace();
        let remaining = self.remaining();
        if remaining.starts_with(keyword) {
            let after = remaining.as_bytes().get(keyword.len());
            // Ensure it's a full word match (not prefix of another identifier)
            if after.is_none()
                || (!after.unwrap().is_ascii_alphanumeric() && *after.unwrap() != b'_')
            {
                self.pos += keyword.len();
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text() {
        let nodes = parse("Hello, World!").unwrap();
        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0], Node::Text(t) if t == "Hello, World!"));
    }

    #[test]
    fn test_parse_variable() {
        let nodes = parse("{{ name }}").unwrap();
        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0], Node::Expression(Expr::Variable(v)) if v == "name"));
    }

    #[test]
    fn test_parse_if() {
        let nodes = parse("{% if x %}yes{% endif %}").unwrap();
        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0], Node::If { .. }));
    }

    #[test]
    fn test_parse_for() {
        let nodes = parse("{% for item in items %}{{ item }}{% endfor %}").unwrap();
        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0], Node::For { .. }));
    }

    #[test]
    fn test_parse_filter() {
        let nodes = parse("{{ name | upper }}").unwrap();
        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0], Node::Expression(Expr::Filter { .. })));
    }

    #[test]
    fn test_parse_comment() {
        let nodes = parse("{# this is a comment #}text").unwrap();
        assert_eq!(nodes.len(), 2);
        assert!(matches!(&nodes[0], Node::Comment(c) if c == "this is a comment"));
    }
}
