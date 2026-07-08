---
id: libs-compass-src-refactoring-signature-helpers-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/refactoring/signature_helpers.rs`.
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

# Standardized libs/compass/src/refactoring/signature_helpers.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/refactoring/signature_helpers.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ParamEntry` | libs/compass/src/refactoring/signature_helpers.rs | struct | pub | 12 | pub(super) struct ParamEntry { |
| `parse_params` | libs/compass/src/refactoring/signature_helpers.rs | function | pub | 23 | pub(super) fn parse_params(source: &str, def_start: usize, _lang: Language) -> Vec<ParamEntry> { |
| `build_new_params` | libs/compass/src/refactoring/signature_helpers.rs | function | pub | 70 | pub(super) fn build_new_params( |
| `format_params` | libs/compass/src/refactoring/signature_helpers.rs | function | pub | 114 | pub(super) fn format_params(params: &[ParamEntry], _language: Language) -> String { |
| `find_param_list_span` | libs/compass/src/refactoring/signature_helpers.rs | function | pub | 135 | pub(super) fn find_param_list_span( |
| `find_return_type_span` | libs/compass/src/refactoring/signature_helpers.rs | function | pub | 146 | pub(super) fn find_return_type_span( |
| `update_call_sites` | libs/compass/src/refactoring/signature_helpers.rs | function | pub | 181 | pub(super) fn update_call_sites( |
| `line_start_byte` | libs/compass/src/refactoring/signature_helpers.rs | function | pub | 274 | pub(super) fn line_start_byte(source: &str, line: usize) -> usize { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Helpers for change-signature refactoring: parameter parsing, call-site
//! rewriting, and span utilities.

use crate::syntax::Language;
use crate::type_inference::{RefactorResult, SignatureChanges, Span, TextEdit};

// ============================================================================
// Parameter entry
// ============================================================================

#[derive(Debug, Clone)]
pub(super) struct ParamEntry {
    pub name: String,
    pub type_ann: Option<String>,
    pub default: Option<String>,
}

// ============================================================================
// Parameter parsing
// ============================================================================

/// Parse parameter names from the function definition line starting at `def_start`.
pub(super) fn parse_params(source: &str, def_start: usize, _lang: Language) -> Vec<ParamEntry> {
    let line = source[def_start..].lines().next().unwrap_or("");
    let open = match line.find('(') {
        Some(i) => i,
        None => return Vec::new(),
    };
    let close = match line[open..].find(')') {
        Some(i) => open + i,
        None => return Vec::new(),
    };
    let inner = &line[open + 1..close];
    if inner.trim().is_empty() {
        return Vec::new();
    }

    inner
        .split(',')
        .map(|part| {
            let part = part.trim();
            let (name_type, default) = if let Some(eq) = part.find('=') {
                let after_eq = part.as_bytes().get(eq + 1).copied().unwrap_or(0);
                if after_eq == b'=' {
                    (part, None)
                } else {
                    (&part[..eq], Some(part[eq + 1..].trim().to_string()))
                }
            } else {
                (part, None)
            };
            let (name, type_ann) = if let Some(colon) = name_type.find(':') {
                (
                    name_type[..colon].trim().to_string(),
                    Some(name_type[colon + 1..].trim().to_string()),
                )
            } else {
                (name_type.trim().to_string(), None)
            };
            ParamEntry {
                name,
                type_ann,
                default,
            }
        })
        .collect()
}

/// Build the new ordered parameter list after applying `changes`.
pub(super) fn build_new_params(
    existing: &[ParamEntry],
    changes: &SignatureChanges,
) -> Vec<ParamEntry> {
    let kept: Vec<(usize, &ParamEntry)> = existing
        .iter()
        .enumerate()
        .filter(|(i, _)| !changes.removed_params.contains(i))
        .collect();

    if !changes.param_order.is_empty() {
        let mut reordered = Vec::with_capacity(changes.param_order.len());
        for &idx in &changes.param_order {
            if let Some(entry) = kept.iter().find(|(i, _)| *i == idx) {
                reordered.push(entry.1.clone());
            }
        }
        for (i, p) in &kept {
            if !changes.param_order.contains(i) {
                reordered.push((*p).clone());
            }
        }
        for (name, type_ann, default) in &changes.new_params {
            reordered.push(ParamEntry {
                name: name.clone(),
                type_ann: type_ann.clone(),
                default: default.clone(),
            });
        }
        return reordered;
    }

    let mut result: Vec<ParamEntry> = kept.into_iter().map(|(_, p)| p.clone()).collect();
    for (name, type_ann, default) in &changes.new_params {
        result.push(ParamEntry {
            name: name.clone(),
            type_ann: type_ann.clone(),
            default: default.clone(),
        });
    }
    result
}

/// Format a parameter list as source text.
pub(super) fn format_params(params: &[ParamEntry], _language: Language) -> String {
    params
        .iter()
        .map(|p| {
            let mut s = p.name.clone();
            if let Some(ref ty) = p.type_ann {
                s = format!("{}: {}", s, ty);
            }
            if let Some(ref def) = p.default {
                s = format!("{} = {}", s, def);
            }
            s
        })
        .collect::<Vec<_>>()
        .join(", ")
}

// ============================================================================
// Span finders
// ============================================================================

pub(super) fn find_param_list_span(
    source: &str,
    def_start: usize,
    _lang: Language,
) -> Option<Span> {
    let line = source[def_start..].lines().next()?;
    let open = line.find('(')?;
    let close = line[open..].find(')')? + open;
    Some(Span::new(def_start + open + 1, def_start + close))
}

pub(super) fn find_return_type_span(
    source: &str,
    def_start: usize,
    language: Language,
) -> Option<Span> {
    let line = source[def_start..].lines().next()?;
    match language {
        Language::Python => {
            let arrow = line.find("->")?;
            let colon = line[arrow..].find(':')?;
            Some(Span::new(def_start + arrow + 2, def_start + arrow + colon))
        }
        Language::Rust => {
            let arrow = line.find("->")?;
            let brace = line[arrow..].find('{')?;
            Some(Span::new(def_start + arrow + 2, def_start + arrow + brace))
        }
        Language::TypeScript | Language::JavaScript => {
            let cp = line.find(')')?;
            let colon = line[cp..].find(':')?;
            let brace = line[cp..].find('{')?;
            Some(Span::new(
                def_start + cp + colon + 1,
                def_start + cp + brace,
            ))
        }
        _ => None,
    }
}

// ============================================================================
// Call-site updater
// ============================================================================

/// Find call sites of `func_name` in `source` and rewrite their arguments.
pub(super) fn update_call_sites(
    func_name: &str,
    old_params: &[ParamEntry],
    changes: &SignatureChanges,
    source: &str,
    file_path: &std::path::PathBuf,
    _language: Language,
    result: &mut RefactorResult,
) {
    let pattern = format!("{}(", func_name);
    let mut search_from = 0;

    while let Some(pos) = source[search_from..].find(&pattern) {
        let abs_pos = search_from + pos;
        let args_start = abs_pos + pattern.len();
        let close = match find_matching_paren(source, args_start - 1) {
            Some(c) => c,
            None => {
                search_from = args_start;
                continue;
            }
        };
        let old_args_text = &source[args_start..close];
        let old_args: Vec<&str> = if old_args_text.trim().is_empty() {
            Vec::new()
        } else {
            old_args_text.split(',').map(|a| a.trim()).collect()
        };
        let new_args = rewrite_args(&old_args, old_params, changes);
        result.add_edit(
            file_path.clone(),
            TextEdit {
                span: Span::new(args_start, close),
                new_text: new_args.join(", "),
            },
        );
        search_from = close + 1;
    }
}

fn rewrite_args(
    old_args: &[&str],
    _old_params: &[ParamEntry],
    changes: &SignatureChanges,
) -> Vec<String> {
    let kept: Vec<(usize, String)> = old_args
        .iter()
        .enumerate()
        .filter(|(i, _)| !changes.removed_params.contains(i))
        .map(|(i, a)| (i, a.to_string()))
        .collect();

    let mut result = if !changes.param_order.is_empty() {
        let mut reordered = Vec::new();
        for &idx in &changes.param_order {
            if let Some(entry) = kept.iter().find(|(i, _)| *i == idx) {
                reordered.push(entry.1.clone());
            }
        }
        for (i, a) in &kept {
            if !changes.param_order.contains(i) {
                reordered.push(a.clone());
            }
        }
        reordered
    } else {
        kept.into_iter().map(|(_, a)| a).collect()
    };

    for (_name, _type_ann, default) in &changes.new_params {
        result.push(default.clone().unwrap_or_else(|| "None".to_string()));
    }
    result
}

fn find_matching_paren(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (i, ch) in source[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Byte offset of the start of line `line` (0-indexed).
pub(super) fn line_start_byte(source: &str, line: usize) -> usize {
    let mut offset = 0;
    for (i, text_line) in source.split('\n').enumerate() {
        if i == line {
            return offset;
        }
        offset += text_line.len() + 1;
    }
    source.len()
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/refactoring/signature_helpers.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/refactoring/signature_helpers.rs` captured during libs codegen standardization.
```
