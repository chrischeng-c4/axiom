// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
// CODEGEN-BEGIN
//! Tailwind JIT emitter — generates CSS only for classes present in content.
//!
//! Orchestrates scanning, utility resolution, and layer assembly.

pub mod config;
pub mod preflight;
pub mod scanner;
pub mod utilities;
pub mod variants;

use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

use config::{DarkMode, TailwindConfig};
use scanner::ContentScanner;
use utilities::class_to_css;
use variants::{wrap_with_variants, ParsedClass};

// ─── public types ─────────────────────────────────────────────────────────────

/// CSS output split into the three Tailwind layers.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
#[derive(Debug, Clone, Default)]
pub struct TailwindLayers {
    /// `@tailwind base` output (Preflight + custom base rules).
    pub base: String,
    /// `@tailwind components` output.
    pub components: String,
    /// `@tailwind utilities` output — generated JIT rules.
    pub utilities: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
impl TailwindLayers {
    /// Concatenate all layers in canonical order: base → components → utilities.
    pub fn concat(&self) -> String {
        let mut out = String::new();
        if !self.base.is_empty() {
            out.push_str(&self.base);
            out.push('\n');
        }
        if !self.components.is_empty() {
            out.push_str(&self.components);
            out.push('\n');
        }
        if !self.utilities.is_empty() {
            out.push_str(&self.utilities);
            out.push('\n');
        }
        out
    }
}

// ─── TailwindEmitter ──────────────────────────────────────────────────────────

/// Generates Tailwind CSS layers from a set of used class names.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
pub struct TailwindEmitter {
    config: TailwindConfig,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
impl TailwindEmitter {
    pub fn new(config: TailwindConfig) -> Self {
        Self { config }
    }

    /// Scan content files under `root` and return the used class set.
    pub fn scan(&self, root: &Path) -> Result<HashSet<String>> {
        let scanner = ContentScanner::new(self.config.content.clone());
        scanner.scan(root)
    }

    /// Generate `TailwindLayers` for `used_classes`.
    ///
    /// `plugin_css` is additional CSS produced by plugin emitters (e.g.
    /// `tailwindcss-animate`).  It is appended to the utilities layer.
    pub fn generate(&self, used_classes: &HashSet<String>, plugin_css: &str) -> TailwindLayers {
        let dark_class = self.config.dark_mode == DarkMode::Class;
        let base = self.build_base();
        let components = self.build_components();
        let utilities = self.build_utilities(used_classes, dark_class, plugin_css);

        TailwindLayers {
            base,
            components,
            utilities,
        }
    }

    // ── layer builders ───────────────────────────────────────────────────────

    fn build_base(&self) -> String {
        let mut out = String::from(preflight::PREFLIGHT);

        // Inject CSS custom properties for extended theme colors
        if !self.config.theme.extend_colors.is_empty() {
            out.push_str("\n/* Theme color custom properties */\n:root {\n");
            for (name, value) in &self.config.theme.extend_colors {
                out.push_str(&format!("  --color-{}: {};\n", name, value));
            }
            out.push_str("}\n");
        }

        out
    }

    fn build_components(&self) -> String {
        // No built-in components in this implementation.
        // Custom @layer components {} rules are injected by the directive processor.
        String::new()
    }

    fn build_utilities(
        &self,
        used_classes: &HashSet<String>,
        dark_class: bool,
        plugin_css: &str,
    ) -> String {
        let mut rules: Vec<String> = Vec::new();

        // Sort for deterministic output
        let mut sorted: Vec<&str> = used_classes.iter().map(String::as_str).collect();
        sorted.sort_unstable();

        for class in sorted {
            if let Some(rule) = emit_class(class, dark_class, &self.config) {
                rules.push(rule);
            }
        }

        let mut out = rules.join("\n");

        if !plugin_css.is_empty() {
            out.push('\n');
            out.push_str(plugin_css);
        }

        out
    }
}

// ─── class emission ───────────────────────────────────────────────────────────

/// Emit a single CSS rule for a Tailwind class (including variants).
///
/// Returns `None` for unknown classes.
fn emit_class(class: &str, dark_class: bool, config: &TailwindConfig) -> Option<String> {
    let parsed = ParsedClass::parse(class);
    let base = &parsed.base;

    // Resolve base class declarations
    let declarations = resolve_declarations(base, config)?;

    // Build escaped CSS selector
    let selector = escape_selector(class);
    let full_selector = format!(".{}", selector);

    if parsed.variants.is_empty() {
        Some(format!("{} {{ {} }}", full_selector, declarations))
    } else {
        Some(wrap_with_variants(
            &full_selector,
            &declarations,
            &parsed.variants,
            dark_class,
        ))
    }
}

/// Resolve the CSS declarations for a base class name.
fn resolve_declarations(base: &str, config: &TailwindConfig) -> Option<String> {
    // 1. Check custom theme colors: text-{custom}, bg-{custom}
    for prefix in &["text-", "bg-", "border-"] {
        if let Some(color_name) = base.strip_prefix(prefix) {
            if let Some(value) = config.theme.extend_colors.get(color_name) {
                let prop = match *prefix {
                    "text-" => "color",
                    "bg-" => "background-color",
                    "border-" => "border-color",
                    _ => continue,
                };
                return Some(format!("{}: {};", prop, value));
            }
        }
    }

    // 2. Standard utility table
    class_to_css(base)
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use config::{DarkMode, TailwindConfig, ThemeConfig};
    use std::collections::HashSet;

    fn cls(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    fn default_emitter() -> TailwindEmitter {
        TailwindEmitter::new(TailwindConfig {
            content: vec![],
            ..TailwindConfig::default()
        })
    }

    /// T6: JIT Utility Emission — Used Only (R4)
    ///
    /// Verifies that only used utility classes are emitted.
    #[test]
    fn t6_jit_utility_emission_used_only() {
        let emitter = default_emitter();
        let used = cls(&["flex", "items-center"]);
        let layers = emitter.generate(&used, "");
        let utilities = &layers.utilities;

        // Used classes must be present
        assert!(
            utilities.contains("display") && utilities.contains("flex"),
            "Utilities should contain .flex display rule: {}",
            utilities
        );
        assert!(
            utilities.contains("align-items") && utilities.contains("center"),
            "Utilities should contain .items-center rule: {}",
            utilities
        );

        // Unused classes must NOT be present
        assert!(
            !utilities.contains(".hidden"),
            "Utilities should not emit .hidden (not used): {}",
            utilities
        );
    }

    /// T7: Responsive Prefix Emission (R4)
    ///
    /// Verifies that responsive variants are wrapped in the correct @media query.
    #[test]
    fn t7_responsive_prefix_emission() {
        let emitter = default_emitter();
        let used = cls(&["md:text-lg"]);
        let layers = emitter.generate(&used, "");
        let utilities = &layers.utilities;

        // Must wrap in @media (min-width: 768px)
        assert!(
            utilities.contains("@media") && utilities.contains("768px"),
            "Utilities should contain @media (min-width: 768px) for md: prefix: {}",
            utilities
        );
        // text-lg font-size must be emitted inside the media query
        assert!(
            utilities.contains("font-size") || utilities.contains("1.125rem"),
            "Utilities should contain text-lg font-size: {}",
            utilities
        );
    }

    /// Unit test: dark: variant wraps in .dark selector.
    #[test]
    fn dark_variant_wraps_in_dark_selector() {
        let config = TailwindConfig {
            content: vec![],
            dark_mode: DarkMode::Class,
            theme: ThemeConfig::default(),
            plugins: vec![],
        };
        let emitter = TailwindEmitter::new(config);
        let used = cls(&["dark:text-white"]);
        let layers = emitter.generate(&used, "");
        assert!(
            layers.utilities.contains(".dark"),
            "dark: variant should produce .dark-scoped selector: {}",
            layers.utilities
        );
    }

    /// Unit test: plugin CSS is appended to utilities layer.
    #[test]
    fn plugin_css_appended_to_utilities() {
        let emitter = default_emitter();
        let used = cls(&[]);
        let plugin_css = "/* plugin */ .custom { color: red; }";
        let layers = emitter.generate(&used, plugin_css);
        assert!(
            layers.utilities.contains("custom"),
            "Plugin CSS should be appended to utilities: {}",
            layers.utilities
        );
    }

    /// Unit test: TailwindLayers::concat returns layers in base→components→utilities order.
    #[test]
    fn tailwind_layers_concat_order() {
        let layers = TailwindLayers {
            base: "base-css".to_string(),
            components: "components-css".to_string(),
            utilities: "utilities-css".to_string(),
        };
        let concat = layers.concat();
        let base_pos = concat.find("base-css").unwrap();
        let comp_pos = concat.find("components-css").unwrap();
        let util_pos = concat.find("utilities-css").unwrap();
        assert!(base_pos < comp_pos, "base should appear before components");
        assert!(
            comp_pos < util_pos,
            "components should appear before utilities"
        );
    }
}

/// CSS-escape a selector string (backslash-prefix special characters).
fn escape_selector(class: &str) -> String {
    let mut out = String::with_capacity(class.len() + 4);
    for c in class.chars() {
        match c {
            ':' | '[' | ']' | '/' | '.' | '#' | '!' | '%' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}
// CODEGEN-END
