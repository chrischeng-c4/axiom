//! Language-specific checkers

mod asyncapi;
pub mod autofix;
mod css;
pub mod custom;
mod dockerfile;
pub mod embedded_markdown;
mod gitlab_ci;
mod gitlab_ci_rules;
mod go;
mod graphql;
mod html;
mod javascript;
mod kubernetes;
mod kubernetes_rules;
pub mod markdown;
mod mdx;
mod mermaid;
mod openapi;
mod openrpc;
mod proto;
mod python;
mod python_security;
mod rust_checker;
mod sql;
mod terraform;
mod terraform_rules;
mod toml_checker;
mod typescript;
mod yaml_dispatch;

use crate::checker::LintConfig;
use crate::diagnostic::Diagnostic;
use crate::syntax::{Language, ParsedFile};
use std::collections::HashMap;

pub use asyncapi::AsyncApiChecker;
pub use css::CssChecker;
pub use custom::CustomLintEngine;
pub use dockerfile::DockerfileChecker;
pub use gitlab_ci::GitlabCiChecker;
pub use go::GoChecker;
pub use graphql::GraphqlChecker;
pub use html::HtmlChecker;
pub use javascript::JavaScriptChecker;
pub use kubernetes::KubernetesChecker;
pub use markdown::MarkdownChecker;
pub use mdx::MdxChecker;
pub use mermaid::MermaidChecker;
pub use openapi::OpenApiChecker;
pub use openrpc::OpenRpcChecker;
pub use proto::ProtoChecker;
pub use python::PythonChecker;
pub use rust_checker::RustChecker;
pub use sql::{detect_sql_injection, SqlChecker};
pub use terraform::TerraformChecker;
pub use toml_checker::TomlChecker;
pub use typescript::TypeScriptChecker;
pub use yaml_dispatch::YamlDispatcher;

/// Trait for language-specific checkers
pub trait Checker: Send + Sync {
    fn language(&self) -> Language;
    fn check(&self, file: &ParsedFile, config: &LintConfig) -> Vec<Diagnostic>;
    fn available_rules(&self) -> Vec<&'static str>;
}

/// Registry of all checkers
pub struct CheckerRegistry {
    checkers: HashMap<Language, Box<dyn Checker>>,
}

impl CheckerRegistry {
    pub fn new() -> Self {
        let mut checkers: HashMap<Language, Box<dyn Checker>> = HashMap::new();

        checkers.insert(Language::Python, Box::new(PythonChecker::new()));
        checkers.insert(Language::TypeScript, Box::new(TypeScriptChecker::new()));
        checkers.insert(Language::Rust, Box::new(RustChecker::new()));
        checkers.insert(Language::JavaScript, Box::new(JavaScriptChecker::new()));
        checkers.insert(Language::Go, Box::new(GoChecker::new()));
        checkers.insert(Language::Html, Box::new(HtmlChecker::new()));
        checkers.insert(Language::Css, Box::new(CssChecker::new()));
        checkers.insert(Language::Dockerfile, Box::new(DockerfileChecker));
        checkers.insert(Language::Hcl, Box::new(TerraformChecker));
        checkers.insert(Language::Yaml, Box::new(YamlDispatcher::new()));
        checkers.insert(Language::Markdown, Box::new(MarkdownChecker::new()));
        checkers.insert(Language::Mdx, Box::new(MdxChecker::new()));
        checkers.insert(Language::Mermaid, Box::new(MermaidChecker::new()));
        checkers.insert(Language::Toml, Box::new(TomlChecker::new()));
        checkers.insert(Language::Sql, Box::new(SqlChecker::new()));
        checkers.insert(Language::Proto, Box::new(ProtoChecker::new()));
        checkers.insert(Language::GraphQL, Box::new(GraphqlChecker::new()));

        Self { checkers }
    }

    pub fn get(&self, language: Language) -> Option<&dyn Checker> {
        self.checkers.get(&language).map(|c| c.as_ref())
    }
}

impl Default for CheckerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
