//! Jinja2-compatible template engine.
//!
//! Provides template rendering with:
//! - Variable interpolation: `{{ variable }}`
//! - Filters: `{{ value | upper }}`, `{{ value | default("fallback") }}`
//! - Conditionals: `{% if %} ... {% elif %} ... {% else %} ... {% endif %}`
//! - Loops: `{% for item in items %} ... {% endfor %}`
//! - Template inheritance: `{% extends "base.html" %}` / `{% block name %}`
//! - Includes: `{% include "partial.html" %}`
//! - Set variables: `{% set x = value %}`
//! - Comments: `{# comment #}`
//! - Raw blocks: `{% raw %} ... {% endraw %}`

pub mod engine;
pub mod filters;
pub mod parser;

pub use engine::{render, Context, Engine, FileLoader, MapLoader, TemplateLoader, Value};
pub use parser::{parse, BinOperator, Expr, Node, ParseError};
