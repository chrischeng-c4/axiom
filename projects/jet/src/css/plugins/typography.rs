// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-css-plugins.md#schema
// CODEGEN-BEGIN
//! Native Rust emitter for `@tailwindcss/typography`.
//!
//! Generates `prose` class CSS for readable long-form content, equivalent to
//! the `@tailwindcss/typography` npm package.

use std::collections::HashSet;

/// Emit CSS for all `@tailwindcss/typography` classes referenced in
/// `used_classes`.
///
/// Returns the CSS string to append to the utilities layer.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-plugins.md#schema
pub fn emit(used_classes: &HashSet<String>) -> String {
    let uses_prose = used_classes.iter().any(|c| {
        c == "prose" || c.starts_with("prose-") || c.ends_with(":prose") || c.contains(":prose-")
    });

    if !uses_prose {
        return String::new();
    }

    let mut out = String::from("/* @tailwindcss/typography */\n");

    // Base prose class
    out.push_str(PROSE_BASE);
    out.push('\n');

    // Size variants
    if used_classes
        .iter()
        .any(|c| c == "prose-sm" || c.ends_with(":prose-sm"))
    {
        out.push_str(PROSE_SM);
        out.push('\n');
    }
    if used_classes
        .iter()
        .any(|c| c == "prose-base" || c.ends_with(":prose-base"))
    {
        out.push_str(PROSE_BASE_SIZE);
        out.push('\n');
    }
    if used_classes
        .iter()
        .any(|c| c == "prose-lg" || c.ends_with(":prose-lg"))
    {
        out.push_str(PROSE_LG);
        out.push('\n');
    }
    if used_classes
        .iter()
        .any(|c| c == "prose-xl" || c.ends_with(":prose-xl"))
    {
        out.push_str(PROSE_XL);
        out.push('\n');
    }
    if used_classes
        .iter()
        .any(|c| c == "prose-2xl" || c.ends_with(":prose-2xl"))
    {
        out.push_str(PROSE_2XL);
        out.push('\n');
    }

    // Dark mode invert
    if used_classes
        .iter()
        .any(|c| c == "prose-invert" || c.ends_with(":prose-invert") || c.contains("prose-invert"))
    {
        out.push_str(PROSE_INVERT);
        out.push('\n');
        // Dark scoped variant
        out.push_str(PROSE_DARK_INVERT);
        out.push('\n');
    }

    out
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn cls(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    /// T14: @tailwindcss/typography Prose (R8)
    ///
    /// Verifies that prose and prose-lg classes emit the typography CSS.
    #[test]
    fn t14_tailwindcss_typography_prose() {
        let used = cls(&["prose", "prose-lg"]);
        let css = emit(&used);

        assert!(!css.is_empty(), "Should emit CSS for prose classes");
        assert!(
            css.contains(".prose"),
            "Should emit .prose base class: {}",
            css
        );
        assert!(
            css.contains(".prose-lg"),
            "Should emit .prose-lg size variant: {}",
            css
        );
        // prose-lg defines font-size
        assert!(
            css.contains("1.125rem"),
            "prose-lg should emit font-size: 1.125rem: {}",
            css
        );
    }

    /// Unit test: no prose usage → no CSS emitted.
    #[test]
    fn no_prose_usage_emits_nothing() {
        let used = cls(&["flex", "text-blue-500"]);
        let css = emit(&used);
        assert!(
            css.is_empty(),
            "Should emit nothing when no prose classes are used: {}",
            css
        );
    }

    /// Unit test: prose-invert emits dark mode override CSS.
    #[test]
    fn prose_invert_dark_mode() {
        let used = cls(&["prose", "prose-invert"]);
        let css = emit(&used);
        assert!(
            css.contains(".prose-invert"),
            "Should emit .prose-invert: {}",
            css
        );
        assert!(
            css.contains("dark\\:prose-invert") || css.contains(".dark"),
            "Should emit dark mode scoped prose-invert: {}",
            css
        );
    }

    /// Unit test: prose-sm emits sm variant.
    #[test]
    fn prose_sm_size_variant() {
        let used = cls(&["prose", "prose-sm"]);
        let css = emit(&used);
        assert!(css.contains(".prose-sm"), "Should emit .prose-sm: {}", css);
        assert!(
            css.contains("0.875rem"),
            "prose-sm should use 0.875rem font-size: {}",
            css
        );
    }

    /// Unit test: typography prefix in /* @tailwindcss/typography */ comment.
    #[test]
    fn typography_css_comment_marker() {
        let used = cls(&["prose"]);
        let css = emit(&used);
        assert!(
            css.contains("@tailwindcss/typography"),
            "Output should have @tailwindcss/typography marker comment: {}",
            css
        );
    }
}

// ─── prose CSS ────────────────────────────────────────────────────────────────

const PROSE_BASE: &str = r#".prose {
  color: var(--tw-prose-body, #374151);
  max-width: 65ch;
  font-size: 1rem;
  line-height: 1.75;
}
.prose p {
  margin-top: 1.25em;
  margin-bottom: 1.25em;
}
.prose [class~="lead"] {
  color: var(--tw-prose-lead, #4b5563);
  font-size: 1.25em;
  line-height: 1.6;
  margin-top: 1.2em;
  margin-bottom: 1.2em;
}
.prose a {
  color: var(--tw-prose-links, #111827);
  text-decoration: underline;
  font-weight: 500;
}
.prose strong {
  color: var(--tw-prose-bold, #111827);
  font-weight: 600;
}
.prose h1 {
  color: var(--tw-prose-headings, #111827);
  font-weight: 800;
  font-size: 2.25em;
  margin-top: 0;
  margin-bottom: 0.8888889em;
  line-height: 1.1111111;
}
.prose h2 {
  color: var(--tw-prose-headings, #111827);
  font-weight: 700;
  font-size: 1.5em;
  margin-top: 2em;
  margin-bottom: 1em;
  line-height: 1.3333333;
}
.prose h3 {
  color: var(--tw-prose-headings, #111827);
  font-weight: 600;
  font-size: 1.25em;
  margin-top: 1.6em;
  margin-bottom: 0.6em;
  line-height: 1.6;
}
.prose h4 {
  color: var(--tw-prose-headings, #111827);
  font-weight: 600;
  margin-top: 1.5em;
  margin-bottom: 0.5em;
  line-height: 1.5;
}
.prose ul {
  list-style-type: disc;
  margin-top: 1.25em;
  margin-bottom: 1.25em;
  padding-left: 1.625em;
}
.prose ol {
  list-style-type: decimal;
  margin-top: 1.25em;
  margin-bottom: 1.25em;
  padding-left: 1.625em;
}
.prose li {
  margin-top: 0.5em;
  margin-bottom: 0.5em;
}
.prose blockquote {
  font-weight: 500;
  font-style: italic;
  color: var(--tw-prose-quotes, #111827);
  border-left-width: 0.25rem;
  border-left-color: var(--tw-prose-quote-borders, #e5e7eb);
  quotes: "\201C""\201D""\2018""\2019";
  margin-top: 1.6em;
  margin-bottom: 1.6em;
  padding-left: 1em;
}
.prose code {
  color: var(--tw-prose-code, #111827);
  font-weight: 600;
  font-size: 0.875em;
}
.prose pre {
  color: var(--tw-prose-pre-code, #e5e7eb);
  background-color: var(--tw-prose-pre-bg, #1f2937);
  overflow-x: auto;
  font-weight: 400;
  font-size: 0.875em;
  line-height: 1.7142857;
  margin-top: 1.7142857em;
  margin-bottom: 1.7142857em;
  border-radius: 0.375rem;
  padding: 0.8571429em 1.1428571em;
}
.prose table {
  width: 100%;
  table-layout: auto;
  text-align: left;
  margin-top: 2em;
  margin-bottom: 2em;
  font-size: 0.875em;
  line-height: 1.7142857;
}
.prose thead {
  border-bottom-width: 1px;
  border-bottom-color: var(--tw-prose-th-borders, #d1d5db);
}
.prose thead th {
  color: var(--tw-prose-headings, #111827);
  font-weight: 600;
  vertical-align: bottom;
  padding-right: 0.5714286em;
  padding-bottom: 0.5714286em;
  padding-left: 0.5714286em;
}
.prose tbody tr {
  border-bottom-width: 1px;
  border-bottom-color: var(--tw-prose-td-borders, #e5e7eb);
}
.prose tbody td {
  vertical-align: baseline;
  padding: 0.5714286em;
}
.prose hr {
  color: var(--tw-prose-hr, #e5e7eb);
  border-color: inherit;
  border-top-width: 1px;
  margin-top: 3em;
  margin-bottom: 3em;
}"#;

const PROSE_SM: &str = r#".prose-sm {
  font-size: 0.875rem;
  line-height: 1.7142857;
}
.prose-sm h1 { font-size: 2.1428571em; }
.prose-sm h2 { font-size: 1.4285714em; }
.prose-sm h3 { font-size: 1.2857143em; }
.prose-sm p { margin-top: 1.1428571em; margin-bottom: 1.1428571em; }"#;

const PROSE_BASE_SIZE: &str = r#".prose-base {
  font-size: 1rem;
  line-height: 1.75;
}"#;

const PROSE_LG: &str = r#".prose-lg {
  font-size: 1.125rem;
  line-height: 1.7777778;
}
.prose-lg h1 { font-size: 2.6666667em; }
.prose-lg h2 { font-size: 1.6666667em; }
.prose-lg h3 { font-size: 1.3333333em; }
.prose-lg p { margin-top: 1.3333333em; margin-bottom: 1.3333333em; }"#;

const PROSE_XL: &str = r#".prose-xl {
  font-size: 1.25rem;
  line-height: 1.8;
}
.prose-xl h1 { font-size: 2.8em; }
.prose-xl h2 { font-size: 1.8em; }
.prose-xl h3 { font-size: 1.4em; }
.prose-xl p { margin-top: 1.2em; margin-bottom: 1.2em; }"#;

const PROSE_2XL: &str = r#".prose-2xl {
  font-size: 1.5rem;
  line-height: 1.6666667;
}
.prose-2xl h1 { font-size: 2.6666667em; }
.prose-2xl h2 { font-size: 2em; }
.prose-2xl h3 { font-size: 1.5em; }
.prose-2xl p { margin-top: 1.3333333em; margin-bottom: 1.3333333em; }"#;

const PROSE_INVERT: &str = r#".prose-invert {
  --tw-prose-body: #d1d5db;
  --tw-prose-headings: #fff;
  --tw-prose-lead: #9ca3af;
  --tw-prose-links: #fff;
  --tw-prose-bold: #fff;
  --tw-prose-counters: #9ca3af;
  --tw-prose-bullets: #4b5563;
  --tw-prose-hr: #374151;
  --tw-prose-quotes: #f3f4f6;
  --tw-prose-quote-borders: #374151;
  --tw-prose-captions: #9ca3af;
  --tw-prose-kbd: #fff;
  --tw-prose-kbd-shadows: 255 255 255;
  --tw-prose-code: #fff;
  --tw-prose-pre-code: #d1d5db;
  --tw-prose-pre-bg: rgb(0 0 0 / 50%);
  --tw-prose-th-borders: #4b5563;
  --tw-prose-td-borders: #374151;
}"#;

/// Dark mode scoped `prose-invert` (for `dark:prose-invert` usage).
const PROSE_DARK_INVERT: &str = r#".dark .dark\:prose-invert {
  --tw-prose-body: #d1d5db;
  --tw-prose-headings: #fff;
  --tw-prose-links: #fff;
  --tw-prose-bold: #fff;
  --tw-prose-code: #fff;
  --tw-prose-pre-code: #d1d5db;
  --tw-prose-pre-bg: rgb(0 0 0 / 50%);
}"#;
// CODEGEN-END
