// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
// CODEGEN-BEGIN
//! Tailwind variant prefix handling.
//!
//! Wraps a CSS rule in the appropriate selector or media query for a given
//! variant prefix such as `hover:`, `dark:`, `sm:`, etc.

// ─── breakpoints ─────────────────────────────────────────────────────────────

/// Responsive breakpoint prefix → `min-width` value.
const BREAKPOINTS: &[(&str, &str)] = &[
    ("sm", "640px"),
    ("md", "768px"),
    ("lg", "1024px"),
    ("xl", "1280px"),
    ("2xl", "1536px"),
];

// ─── public API ──────────────────────────────────────────────────────────────

/// A parsed Tailwind class with any variant prefixes stripped.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
#[derive(Debug, Clone)]
pub struct ParsedClass {
    /// The base utility class name (no variant prefixes).
    pub base: String,
    /// Ordered list of variant prefixes (innermost last).
    ///
    /// E.g. `"md:hover:text-blue-500"` → `["md", "hover"]`.
    pub variants: Vec<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
impl ParsedClass {
    /// Split a raw Tailwind class string into its variants and base class.
    pub fn parse(class: &str) -> Self {
        let mut parts: Vec<&str> = class.split(':').collect();
        let base = parts.pop().unwrap_or(class).to_string();
        let variants = parts.iter().map(|s| s.to_string()).collect();
        Self { base, variants }
    }
}

/// Wrap a CSS declaration block in the selectors and media queries implied by
/// `variants`.
///
/// `selector` is the base selector for the rule (e.g. `.flex`).
/// `declarations` is the raw CSS declarations (e.g. `display: flex;`).
/// `dark_class` controls whether `dark:` generates `.dark selector` (true, default)
/// or `@media (prefers-color-scheme: dark)` (false).
///
/// Returns the complete CSS rule string.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
pub fn wrap_with_variants(
    selector: &str,
    declarations: &str,
    variants: &[String],
    dark_class: bool,
) -> String {
    // Work from innermost (last) to outermost (first) variant.
    let mut current_selector = selector.to_string();
    let mut media_wrappers: Vec<String> = Vec::new();

    for variant in variants.iter().rev() {
        match variant.as_str() {
            // Pseudo-class variants
            "hover" => current_selector = format!("{}:hover", current_selector),
            "focus" => current_selector = format!("{}:focus", current_selector),
            "focus-within" => current_selector = format!("{}:focus-within", current_selector),
            "focus-visible" => current_selector = format!("{}:focus-visible", current_selector),
            "active" => current_selector = format!("{}:active", current_selector),
            "visited" => current_selector = format!("{}:visited", current_selector),
            "disabled" => current_selector = format!("{}:disabled", current_selector),
            "checked" => current_selector = format!("{}:checked", current_selector),
            "placeholder" => current_selector = format!("{}::placeholder", current_selector),
            "first" => current_selector = format!("{}:first-child", current_selector),
            "last" => current_selector = format!("{}:last-child", current_selector),
            "odd" => current_selector = format!("{}:nth-child(odd)", current_selector),
            "even" => current_selector = format!("{}:nth-child(even)", current_selector),
            "not-first" => current_selector = format!("{}:not(:first-child)", current_selector),
            "not-last" => current_selector = format!("{}:not(:last-child)", current_selector),

            // Group/peer variants
            v if v.starts_with("group-") => {
                let pseudo = v.strip_prefix("group-").unwrap_or("");
                current_selector = format!(".group:{} {}", pseudo, current_selector);
            }
            v if v.starts_with("peer-") => {
                let pseudo = v.strip_prefix("peer-").unwrap_or("");
                current_selector = format!(".peer:{} ~ {}", pseudo, current_selector);
            }
            "group-hover" => current_selector = format!(".group:hover {}", current_selector),
            "group-focus" => current_selector = format!(".group:focus {}", current_selector),
            "peer-hover" => current_selector = format!(".peer:hover ~ {}", current_selector),
            "peer-focus" => current_selector = format!(".peer:focus ~ {}", current_selector),

            // Dark mode
            "dark" => {
                if dark_class {
                    current_selector = format!(".dark {}", current_selector);
                } else {
                    media_wrappers.push("@media (prefers-color-scheme: dark)".to_string());
                }
            }

            // Motion variants
            "motion-safe" => {
                media_wrappers.push("@media (prefers-reduced-motion: no-preference)".to_string());
            }
            "motion-reduce" => {
                media_wrappers.push("@media (prefers-reduced-motion: reduce)".to_string());
            }

            // Print
            "print" => media_wrappers.push("@media print".to_string()),

            // Responsive breakpoints
            v => {
                if let Some(width) = breakpoint_min_width(v) {
                    media_wrappers.push(format!("@media (min-width: {})", width));
                }
                // Unknown variants are silently ignored
            }
        }
    }

    // Build the rule
    let rule = format!("{} {{ {} }}", current_selector, declarations);

    // Wrap in media queries (innermost media wrapper last in the list = outermost scope)
    media_wrappers
        .iter()
        .fold(rule, |inner, media| format!("{} {{ {} }}", media, inner))
}

/// Look up a breakpoint name and return its `min-width` value.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
pub fn breakpoint_min_width(name: &str) -> Option<&'static str> {
    BREAKPOINTS
        .iter()
        .find(|(bp, _)| *bp == name)
        .map(|(_, width)| *width)
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── variant prefix selector wrapping (TR5) ───────────────────────────────

    /// S10: hover variant appends :hover pseudo-class to selector.
    #[test]
    fn variant_hover_pseudo_class() {
        let result = wrap_with_variants(
            r".hover\:bg-blue-500",
            "background-color: rgb(59 130 246);",
            &["hover".into()],
            true,
        );
        assert!(
            result.contains(r".hover\:bg-blue-500:hover"),
            "hover variant must append :hover pseudo-class, got: {}",
            result
        );
        assert!(
            result.contains("background-color: rgb(59 130 246);"),
            "declarations must be preserved, got: {}",
            result
        );
    }

    /// S11: group-hover variant prepends .group:hover ancestor selector.
    #[test]
    fn variant_group_hover_ancestor() {
        let result = wrap_with_variants(
            r".group-hover\:text-white",
            "color: rgb(255 255 255);",
            &["group-hover".into()],
            true,
        );
        assert!(
            result.contains(r".group:hover .group-hover\:text-white"),
            "group-hover variant must prepend .group:hover ancestor, got: {}",
            result
        );
    }

    /// S12: Combined sm:hover wraps in media query + :hover pseudo.
    #[test]
    fn variant_sm_hover_combined() {
        let result = wrap_with_variants(
            r".sm\:hover\:underline",
            "text-decoration-line: underline;",
            &["sm".into(), "hover".into()],
            true,
        );
        assert!(
            result.contains("@media (min-width: 640px)"),
            "sm variant must wrap in @media (min-width: 640px), got: {}",
            result
        );
        assert!(
            result.contains(":hover"),
            "hover variant must add :hover pseudo-class inside media query, got: {}",
            result
        );
    }

    // ── ParsedClass variant splitting (TR6) ──────────────────────────────────

    /// S13: ParsedClass splits compound variants md:hover:text-blue-500.
    #[test]
    fn parsed_class_compound_variants() {
        let parsed = ParsedClass::parse("md:hover:text-blue-500");
        assert_eq!(
            parsed.variants,
            vec!["md", "hover"],
            "variants must be [\"md\", \"hover\"], got: {:?}",
            parsed.variants
        );
        assert_eq!(
            parsed.base, "text-blue-500",
            "base must be \"text-blue-500\", got: {}",
            parsed.base
        );
    }

    /// S14: ParsedClass with no variants returns base only.
    #[test]
    fn parsed_class_no_variants() {
        let parsed = ParsedClass::parse("flex");
        assert!(
            parsed.variants.is_empty(),
            "variants must be empty for \"flex\", got: {:?}",
            parsed.variants
        );
        assert_eq!(
            parsed.base, "flex",
            "base must be \"flex\", got: {}",
            parsed.base
        );
    }
}
// CODEGEN-END
