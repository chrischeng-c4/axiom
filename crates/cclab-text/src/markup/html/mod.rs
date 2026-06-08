//! HTML parsing module.

mod parser;
mod tokenizer;

pub use parser::parse_html;
pub use tokenizer::{Token, Tokenizer};
