// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-css.md#schema
// CODEGEN-BEGIN
//! CSS directive processing.
//!
//! Handles `@tailwind`, `@apply`, and `@layer` directives in CSS source.

use std::collections::HashSet;

use crate::css::tailwind::utilities::apply_to_declarations;
use crate::css::tailwind::{TailwindEmitter, TailwindLayers};

// ─── directive detection ───────────────────────────────────────────────────────

/// Returns `true` if the CSS source contains any `@tailwind` directives.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css.md#schema
pub fn has_tailwind_directives(source: &str) -> bool {
    source.contains("@tailwind ")
}

/// Returns `true` if the CSS source contains any `@apply` directives.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css.md#schema
pub fn has_apply_directives(source: &str) -> bool {
    source.contains("@apply ")
}

// ─── @tailwind injection ──────────────────────────────────────────────────────

/// Replace `@tailwind base`, `@tailwind components`, and `@tailwind utilities`
/// directives with the corresponding layer CSS.
///
/// Any additional CSS from plugins (passed as `plugin_css`) is appended to
/// the utilities injection.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css.md#schema
pub fn inject_tailwind_layers(source: &str, layers: &TailwindLayers) -> String {
    let source = source.replace("@tailwind base;", &layers.base);
    let source = source.replace("@tailwind base", &layers.base);
    let source = source.replace("@tailwind components;", &layers.components);
    let source = source.replace("@tailwind components", &layers.components);
    let source = source.replace("@tailwind utilities;", &layers.utilities);
    source.replace("@tailwind utilities", &layers.utilities)
}

// ─── @apply expansion ─────────────────────────────────────────────────────────

/// Expand all `@apply <classes>;` directives in `source` into CSS declarations.
///
/// Example:
/// ```css
/// .btn { @apply flex rounded px-4; }
/// /* becomes */
/// .btn { display: flex; border-radius: 0.25rem; padding-left: 1rem; padding-right: 1rem; }
/// ```
/// @spec .aw/tech-design/projects/jet/semantic/jet-css.md#schema
pub fn expand_apply(source: &str) -> String {
    let mut out = String::with_capacity(source.len());

    let mut remaining = source;
    while let Some(pos) = remaining.find("@apply ") {
        // Push everything before @apply
        out.push_str(&remaining[..pos]);
        remaining = &remaining[pos + "@apply ".len()..];

        // Collect until semicolon or newline
        let end = remaining
            .find(|c| c == ';' || c == '\n' || c == '}')
            .unwrap_or(remaining.len());

        let classes_str = remaining[..end].trim();
        let classes: Vec<&str> = classes_str.split_whitespace().collect();

        // Resolve @apply declarations
        let declarations = apply_to_declarations(&classes);
        out.push_str(&declarations);

        // Skip past the semicolon
        remaining = remaining.get(end..).unwrap_or("");
        if remaining.starts_with(';') {
            remaining = &remaining[1..];
        }
    }

    out.push_str(remaining);
    out
}

// ─── @layer routing ───────────────────────────────────────────────────────────

/// Process `@layer base { ... }`, `@layer components { ... }`, and
/// `@layer utilities { ... }` blocks.
///
/// Custom rules inside each layer are extracted and returned via
/// `LayerOutput`.  The original `@layer` blocks are removed from the
/// returned CSS string (replaced with the layer body inline since
/// lightningcss will handle the cascade order).
/// @spec .aw/tech-design/projects/jet/semantic/jet-css.md#schema
pub struct LayerOutput {
    /// The CSS with `@layer` blocks expanded inline.
    pub css: String,
    /// Collected custom base rules.
    pub base_additions: String,
    /// Collected custom components rules.
    pub components_additions: String,
    /// Collected custom utilities additions.
    pub utilities_additions: String,
}

/// Extract and inline `@layer` blocks.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css.md#schema
pub fn process_layer_directives(source: &str) -> LayerOutput {
    let mut css = String::with_capacity(source.len());
    let mut base_additions = String::new();
    let mut components_additions = String::new();
    let mut utilities_additions = String::new();

    let mut remaining = source;

    while let Some(pos) = remaining.find("@layer ") {
        css.push_str(&remaining[..pos]);
        remaining = &remaining[pos + "@layer ".len()..];

        // Determine layer name
        let name_end = remaining
            .find(|c: char| c == '{' || c.is_whitespace())
            .unwrap_or(remaining.len());
        let layer_name = remaining[..name_end].trim();
        remaining = remaining.get(name_end..).unwrap_or("").trim_start();

        // Expect opening brace
        if !remaining.starts_with('{') {
            // Malformed — keep as-is
            css.push_str("@layer ");
            continue;
        }
        remaining = &remaining[1..]; // consume '{'

        // Find matching closing brace
        let body_len = find_matching_close_brace(remaining).unwrap_or(remaining.len());
        let body = &remaining[..body_len];
        remaining = remaining
            .get(body_len + 1..)
            .unwrap_or("")
            .trim_start_matches(';')
            .trim_start();

        match layer_name {
            "base" => {
                base_additions.push_str(body);
                css.push_str(body); // inline base rules
            }
            "components" => {
                components_additions.push_str(body);
                css.push_str(body);
            }
            "utilities" => {
                utilities_additions.push_str(body);
                css.push_str(body);
            }
            _ => {
                // Unknown layer — inline as-is
                css.push_str(body);
            }
        }
    }

    css.push_str(remaining);

    LayerOutput {
        css,
        base_additions,
        components_additions,
        utilities_additions,
    }
}

// ─── full directive pipeline ──────────────────────────────────────────────────

/// Run the complete directive processing pipeline on a CSS string.
///
/// Pipeline:
/// 1. Detect if Tailwind directives are present
/// 2. If yes: scan content, generate layers via `emitter`, inject them
/// 3. Expand `@apply` directives
/// 4. Inline `@layer` blocks
///
/// Returns the processed CSS string.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css.md#schema
pub fn process_directives(
    source: &str,
    emitter: &TailwindEmitter,
    plugin_css: &str,
    root: &std::path::Path,
    used_classes_override: Option<&HashSet<String>>,
) -> anyhow::Result<String> {
    let css = if has_tailwind_directives(source) {
        // Scan content for used classes (or use override for testing)
        let used_classes = match used_classes_override {
            Some(c) => c.clone(),
            None => emitter.scan(root)?,
        };

        let layers = emitter.generate(&used_classes, plugin_css);
        inject_tailwind_layers(source, &layers)
    } else {
        source.to_string()
    };

    // @apply expansion
    let css = expand_apply(&css);

    // @layer routing
    let layer_out = process_layer_directives(&css);

    Ok(layer_out.css)
}

// ─── helpers ──────────────────────────────────────────────────────────────────

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::tailwind::TailwindLayers;

    fn empty_layers() -> TailwindLayers {
        TailwindLayers {
            base: "/* base */".to_string(),
            components: "/* components */".to_string(),
            utilities: "/* utilities */".to_string(),
        }
    }

    // ── detection ─────────────────────────────────────────────────────────

    #[test]
    fn has_tailwind_directives_detects_directive() {
        assert!(has_tailwind_directives("@tailwind base;"));
        assert!(has_tailwind_directives("/* css */ @tailwind utilities;"));
        assert!(!has_tailwind_directives("a { color: red; }"));
    }

    #[test]
    fn has_apply_directives_detects_directive() {
        assert!(has_apply_directives(".btn { @apply flex; }"));
        assert!(!has_apply_directives("a { color: red; }"));
    }

    // ── @tailwind injection ───────────────────────────────────────────────

    #[test]
    fn inject_tailwind_layers_replaces_base() {
        let layers = empty_layers();
        let result = inject_tailwind_layers("@tailwind base;", &layers);
        assert!(
            result.contains("/* base */"),
            "Should replace @tailwind base: {}",
            result
        );
        assert!(
            !result.contains("@tailwind base"),
            "Should not leave @tailwind base: {}",
            result
        );
    }

    #[test]
    fn inject_tailwind_layers_replaces_utilities() {
        let layers = empty_layers();
        let result = inject_tailwind_layers("@tailwind utilities;", &layers);
        assert!(
            result.contains("/* utilities */"),
            "Should replace @tailwind utilities: {}",
            result
        );
        assert!(
            !result.contains("@tailwind utilities"),
            "Should not leave @tailwind utilities: {}",
            result
        );
    }

    // ── @apply expansion ──────────────────────────────────────────────────

    /// T9 unit-level: @apply directives are expanded to CSS declarations.
    #[test]
    fn t9_expand_apply_flex_rounded() {
        let input = ".btn { @apply flex rounded; }";
        let result = expand_apply(input);
        assert!(
            !result.contains("@apply"),
            "Should not contain @apply: {}",
            result
        );
        assert!(
            result.contains("display: flex"),
            "Should expand flex to display: flex: {}",
            result
        );
        assert!(
            result.contains("border-radius"),
            "Should expand rounded to border-radius: {}",
            result
        );
    }

    #[test]
    fn expand_apply_multiple_classes() {
        let input = ".card { @apply flex items-center justify-between; }";
        let result = expand_apply(input);
        assert!(
            !result.contains("@apply"),
            "No @apply in output: {}",
            result
        );
        assert!(
            result.contains("display: flex"),
            "flex expanded: {}",
            result
        );
        assert!(
            result.contains("align-items"),
            "items-center expanded: {}",
            result
        );
        assert!(
            result.contains("justify-content"),
            "justify-between expanded: {}",
            result
        );
    }

    #[test]
    fn expand_apply_noop_when_no_apply() {
        let input = ".btn { display: flex; }";
        let result = expand_apply(input);
        assert_eq!(result.trim(), input.trim(), "Should pass through unchanged");
    }

    // ── @layer routing ────────────────────────────────────────────────────

    #[test]
    fn process_layer_directives_extracts_base_rules() {
        let input = "@layer base { h1 { margin: 0; } }";
        let result = process_layer_directives(input);
        // Base additions should contain the h1 rule
        assert!(
            result.base_additions.contains("h1"),
            "base_additions should contain h1: {}",
            result.base_additions
        );
        // The CSS output should have the rule inlined
        assert!(
            result.css.contains("h1"),
            "css should contain h1 rule: {}",
            result.css
        );
        // @layer base should be removed
        assert!(
            !result.css.contains("@layer base"),
            "css should not contain @layer base: {}",
            result.css
        );
    }

    #[test]
    fn process_layer_directives_extracts_utilities_rules() {
        let input = "@layer utilities { .custom { opacity: 0.5; } }";
        let result = process_layer_directives(input);
        assert!(
            result.utilities_additions.contains("custom"),
            "utilities_additions should contain custom: {}",
            result.utilities_additions
        );
    }
}

/// Find the position of the matching `}` for the content already past the
/// opening `{`.  Returns the byte index within `s` where `}` was found,
/// accounting for nested braces and string literals.
fn find_matching_close_brace(s: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut string_char = '"';
    let mut prev = '\0';

    for (i, c) in s.char_indices() {
        if in_string {
            if c == string_char && prev != '\\' {
                in_string = false;
            }
        } else {
            match c {
                '"' | '\'' => {
                    in_string = true;
                    string_char = c;
                }
                '{' => depth += 1,
                '}' => {
                    if depth == 0 {
                        return Some(i);
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
        prev = c;
    }
    None
}
// CODEGEN-END
