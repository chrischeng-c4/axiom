---
id: libs-compass-src-lint-mdx-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lint/mdx.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/lint/mdx.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lint/mdx.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MdxChecker` | libs/compass/src/lint/mdx.rs | struct | pub | 10 | pub struct MdxChecker { |
| `new` | libs/compass/src/lint/mdx.rs | function | pub | 15 | pub fn new() -> Self { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! MDX lint checker (Markdown + JSX)

use super::markdown::line_range;
use super::{Checker, MarkdownChecker};
use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity};
use crate::syntax::{Language, ParsedFile};

/// MDX checker — runs all Markdown rules then adds JSX-specific checks.
pub struct MdxChecker {
    md: MarkdownChecker,
}

impl MdxChecker {
    pub fn new() -> Self {
        Self {
            md: MarkdownChecker::new(),
        }
    }

    /// MDX001: Unclosed JSX component.
    ///
    /// Detects `<ComponentName` that is not self-closed (`/>`) and has no
    /// matching `</ComponentName>` anywhere in the file.
    fn check_unclosed_jsx(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (line_idx, line) in file.source.lines().enumerate() {
            let line_num = line_idx as u32;
            let mut remaining = line;

            while let Some(open_pos) = remaining.find('<') {
                let after_open = &remaining[open_pos + 1..];

                // Only match component names (uppercase first char = JSX component)
                let name_end = after_open
                    .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                    .unwrap_or(after_open.len());
                let name = &after_open[..name_end];

                if name.starts_with(|c: char| c.is_ascii_uppercase()) {
                    // Check if this tag is self-closed on the same line
                    let tag_region = &remaining[open_pos..];
                    let tag_close = tag_region.find('>').unwrap_or(tag_region.len());
                    let tag_text = &tag_region[..tag_close];

                    if !tag_text.ends_with('/') {
                        // Look for a closing </Name> anywhere in the file
                        let closing = format!("</{}", name);
                        let self_close = format!("/>");
                        let has_close = file.source.contains(&closing)
                            || tag_region.contains(&self_close as &str);

                        if !has_close {
                            diagnostics.push(Diagnostic::new(
                                line_range(line_num),
                                DiagnosticSeverity::Warning,
                                "MDX001",
                                DiagnosticCategory::Syntax,
                                format!(
                                    "JSX component <{}> appears unclosed — add </{}> or use self-closing />",
                                    name, name
                                ),
                            ));
                        }
                    }
                }

                // Advance past this tag to avoid re-matching
                let advance = open_pos + 1 + name_end;
                if advance >= remaining.len() {
                    break;
                }
                remaining = &remaining[advance..];
            }
        }

        diagnostics
    }

    /// MDX002: Import statement where the imported binding is never used.
    fn check_unused_imports(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (line_idx, line) in file.source.lines().enumerate() {
            let line_num = line_idx as u32;
            let trimmed = line.trim();

            // Match: import Foo from '...'  or  import { Foo } from '...'
            if !trimmed.starts_with("import ") {
                continue;
            }

            // Extract the default binding name (first identifier after "import")
            let after_import = &trimmed["import ".len()..];

            // Only handle default imports for simplicity: `import Name from`
            let binding_end = after_import
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(after_import.len());
            let binding = &after_import[..binding_end];

            if binding.is_empty() || binding == "{" || binding == "*" {
                continue;
            }

            // Check whether the binding appears on any line other than the import line
            let used_outside_import = file
                .source
                .lines()
                .enumerate()
                .filter(|(idx, _)| *idx != line_idx)
                .any(|(_, l)| l.contains(binding));
            if !used_outside_import {
                diagnostics.push(Diagnostic::new(
                    line_range(line_num),
                    DiagnosticSeverity::Warning,
                    "MDX002",
                    DiagnosticCategory::Logic,
                    format!("Imported binding '{}' is never used in this file", binding),
                ));
            }
        }

        diagnostics
    }

    /// MDX003: `export` that is neither `export default` nor `export const`.
    fn check_invalid_export(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (line_idx, line) in file.source.lines().enumerate() {
            let line_num = line_idx as u32;
            let trimmed = line.trim();

            if !trimmed.starts_with("export ") {
                continue;
            }

            let after = &trimmed["export ".len()..];
            let is_valid = after.starts_with("default ")
                || after.starts_with("const ")
                || after.starts_with("let ")
                || after.starts_with("function ")
                || after.starts_with("type ")
                || after.starts_with("interface ");

            if !is_valid {
                diagnostics.push(Diagnostic::new(
                    line_range(line_num),
                    DiagnosticSeverity::Warning,
                    "MDX003",
                    DiagnosticCategory::Syntax,
                    "Invalid export — MDX supports 'export default' and 'export const/let/function/type'",
                ));
            }
        }

        diagnostics
    }
}

impl Default for MdxChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker for MdxChecker {
    fn language(&self) -> Language {
        Language::Mdx
    }

    fn check(&self, file: &ParsedFile, config: &LintConfig) -> Vec<Diagnostic> {
        // Re-use a Markdown-language file view for the base checks
        let md_file = ParsedFile::line_based(file.source.clone(), Language::Markdown);
        let mut diagnostics = self.md.check(&md_file, config);

        diagnostics.extend(self.check_unclosed_jsx(file));
        diagnostics.extend(self.check_unused_imports(file));
        diagnostics.extend(self.check_invalid_export(file));

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        let mut rules = self.md.available_rules();
        rules.extend_from_slice(&[
            "MDX001", // Unclosed JSX component
            "MDX002", // Import without usage
            "MDX003", // Invalid export
        ]);
        rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::LintConfig;

    fn make_file(source: &str) -> ParsedFile {
        ParsedFile::line_based(source.to_string(), Language::Mdx)
    }

    fn codes(diagnostics: &[Diagnostic]) -> Vec<&str> {
        diagnostics.iter().map(|d| d.code.as_str()).collect()
    }

    #[test]
    fn test_unclosed_jsx_component() {
        let source = "# Title\n\n<MyComponent prop=\"val\">\nsome content\n";
        let file = make_file(source);
        let checker = MdxChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"MDX001"),
            "expected MDX001 for unclosed JSX, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_closed_jsx_no_false_positive() {
        let source = "# Title\n\n<MyComponent />\n";
        let file = make_file(source);
        let checker = MdxChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            !codes(&diags).contains(&"MDX001"),
            "should not emit MDX001 for self-closed tag, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_unused_import() {
        let source = "import Foo from './Foo'\n\n# Title\n\nSome text without the component.\n";
        let file = make_file(source);
        let checker = MdxChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"MDX002"),
            "expected MDX002 for unused import, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_used_import_no_false_positive() {
        let source = "import Foo from './Foo'\n\n# Title\n\n<Foo />\n";
        let file = make_file(source);
        let checker = MdxChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            !codes(&diags).contains(&"MDX002"),
            "should not emit MDX002 when import is used, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_invalid_export() {
        let source = "# Title\n\nexport var x = 1\n";
        let file = make_file(source);
        let checker = MdxChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"MDX003"),
            "expected MDX003 for invalid export, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_valid_export_const() {
        let source = "# Title\n\nexport const meta = { title: 'Test' }\n";
        let file = make_file(source);
        let checker = MdxChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            !codes(&diags).contains(&"MDX003"),
            "should not emit MDX003 for 'export const', got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_inherits_markdown_rules() {
        // Missing code lang should still fire in MDX
        let source = "# Title\n\n```\nsome code\n```\n";
        let file = make_file(source);
        let checker = MdxChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"MD003"),
            "MDX checker should inherit MD003 from MarkdownChecker, got {:?}",
            codes(&diags)
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lint/mdx.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lint/mdx.rs` captured during libs codegen standardization.
```
