// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-css.md#schema
// CODEGEN-BEGIN
//! CSS build pipeline.
//!
//! Orchestrates the full CSS processing chain:
//! `@import` resolution → Tailwind JIT → directive expansion →
//! `lightningcss` transform → optional minification → `CssOutput`.

pub mod directives;
pub mod import_resolver;
pub mod output;
pub mod plugins;
pub mod scss;
pub mod tailwind;

pub use output::CssOutput;
pub use tailwind::config::TailwindConfig;

use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

use directives::process_directives;
use import_resolver::resolve_imports;
use plugins::emit_plugins;
use tailwind::TailwindEmitter;

// ─── CssPipeline ─────────────────────────────────────────────────────────────

/// Full CSS build pipeline.
///
/// # Usage
/// ```no_run
/// use std::path::Path;
/// use jet::css::{CssPipeline, TailwindConfig};
///
/// let config = TailwindConfig::load(Path::new(".")).unwrap();
/// let pipeline = CssPipeline::new(Path::new(".").to_path_buf(), config, false);
/// let output = pipeline.process(Path::new("src/index.css")).unwrap();
/// println!("Hash: {}", output.hash);
/// ```
/// @spec .aw/tech-design/projects/jet/semantic/jet-css.md#schema
pub struct CssPipeline {
    root: std::path::PathBuf,
    config: TailwindConfig,
    production: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-css.md#schema
impl CssPipeline {
    /// Create a new pipeline for the given project root.
    ///
    /// - `root`: project root directory (used for content scanning)
    /// - `config`: parsed Tailwind configuration
    /// - `production`: when `true`, enables CSS minification
    pub fn new(root: std::path::PathBuf, config: TailwindConfig, production: bool) -> Self {
        Self {
            root,
            config,
            production,
        }
    }

    /// Process a CSS entry file through the full pipeline.
    ///
    /// Steps:
    /// 1. Resolve and inline `@import` statements
    /// 2. Scan content files for used Tailwind classes
    /// 3. Generate Tailwind layers (base / components / utilities)
    /// 4. Inject `@tailwind` directives
    /// 5. Expand `@apply` directives
    /// 6. Inline `@layer` blocks
    /// 7. Apply `lightningcss` transforms (nesting, vendor prefixes)
    /// 8. Minify in production mode
    /// 9. Return `CssOutput` with content hash
    pub fn process(&self, entry: &Path) -> Result<CssOutput> {
        // 0. SCSS/Sass compile step (runs BEFORE @import resolution and the
        //    lightningcss transform). grass resolves `@use`/`@import` of
        //    partials itself, so a `.scss`/`.sass` entry is flattened to CSS
        //    here; the resulting CSS then flows through the identical
        //    @import → directives → lightningcss chain as a plain `.css`
        //    entry. Plain `.css` entries skip this and read from disk as
        //    before.
        let source = if scss::is_sass_family_path(entry) {
            let compiled = scss::compile_sass_file(entry)?;
            let base_dir = entry.parent().unwrap_or_else(|| Path::new("."));
            import_resolver::resolve_source(&compiled, base_dir)?
        } else {
            // 1. @import resolution
            resolve_imports(entry)?
        };

        // 2. Scan content files for used classes
        let emitter = TailwindEmitter::new(self.config.clone());
        let used_classes = emitter.scan(&self.root)?;

        // 3. Emit plugins with the actual used classes
        let plugin_css = emit_plugins(&self.config.plugins, &used_classes);

        // 4-6. Directive pipeline (tailwind inject + @apply + @layer)
        let processed = process_directives(
            &source,
            &emitter,
            &plugin_css,
            &self.root,
            Some(&used_classes),
        )?;

        // 7 + 8. lightningcss transform + optional minify
        let (css, source_map) = apply_lightningcss(&processed, self.production)?;

        Ok(CssOutput::new(css, source_map))
    }

    /// Process CSS from an in-memory source string.
    ///
    /// Used for testing and when CSS is synthesised rather than read from disk.
    pub fn process_source(
        &self,
        source: &str,
        base_dir: &Path,
        used_classes_override: Option<HashSet<String>>,
    ) -> Result<CssOutput> {
        // Resolve @imports from the given base_dir
        let source = import_resolver::resolve_source(source, base_dir)?;

        let emitter = TailwindEmitter::new(self.config.clone());

        // Determine used classes: use override if provided, otherwise scan
        let used_classes = match used_classes_override {
            Some(c) => c,
            None => emitter.scan(&self.root)?,
        };

        // Emit plugins with the actual used classes
        let plugin_css = emit_plugins(&self.config.plugins, &used_classes);

        let processed = process_directives(
            &source,
            &emitter,
            &plugin_css,
            &self.root,
            Some(&used_classes),
        )?;

        let (css, source_map) = apply_lightningcss(&processed, self.production)?;
        Ok(CssOutput::new(css, source_map))
    }
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::tailwind::config::{DarkMode, TailwindConfig, ThemeConfig};
    use std::collections::{HashMap, HashSet};
    use std::path::Path;

    fn make_pipeline(production: bool) -> CssPipeline {
        CssPipeline::new(
            std::path::PathBuf::from("."),
            TailwindConfig::default(),
            production,
        )
    }

    fn make_pipeline_cfg(config: TailwindConfig, production: bool) -> CssPipeline {
        CssPipeline::new(std::path::PathBuf::from("."), config, production)
    }

    fn cls(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    fn no_content_config() -> TailwindConfig {
        TailwindConfig {
            content: vec![],
            ..TailwindConfig::default()
        }
    }

    /// T1: lightningcss Parse + Transform (R1)
    ///
    /// Verifies the pipeline integrates lightningcss for CSS parsing and output.
    #[test]
    fn t1_lightningcss_css_transform() {
        let pipeline = make_pipeline(false);
        let css = "a { color: red; } a:hover { color: blue; }";
        let result = pipeline.process_source(css, Path::new("."), None);
        assert!(
            result.is_ok(),
            "Pipeline should handle valid CSS: {:?}",
            result
        );
        let output = result.unwrap();
        assert!(
            output.css.contains("red") || output.css.contains("#f00"),
            "Output should contain color red (or #f00), got: {}",
            output.css
        );
        // lightningcss may normalize the `blue` keyword to its hex equivalent #00f
        assert!(
            output.css.contains("blue")
                || output.css.contains("#00f")
                || output.css.contains("#0000ff"),
            "Output should contain color blue (or #00f), got: {}",
            output.css
        );
        assert!(
            output.css.contains("hover"),
            "Output should contain hover selector, got: {}",
            output.css
        );
        assert_eq!(output.hash.len(), 8, "Hash should be 8 hex chars");
        assert!(
            output.hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be lowercase hex: {}",
            output.hash
        );
    }

    /// T2: lightningcss Production Minification (R1, R10)
    ///
    /// Verifies minification strips comments and compacts whitespace.
    #[test]
    fn t2_production_minification() {
        let pipeline = make_pipeline(true);
        // Verbose CSS with a comment and extra whitespace
        let css = "/* comment */ a {   color:   red ;   }\n\n  b  {  font-weight:  bold ;  }  ";
        let result = pipeline.process_source(css, Path::new("."), None);
        assert!(
            result.is_ok(),
            "Pipeline should succeed in production mode: {:?}",
            result
        );
        let output = result.unwrap();
        // No block comments in minified output
        assert!(
            !output.css.contains("/* comment */"),
            "Minified output should not contain comments, got: {}",
            output.css
        );
        // Minified output is shorter than the input (whitespace collapsed)
        assert!(
            output.css.len() < css.len(),
            "Minified output should be shorter than input ({} >= {}), got: {}",
            output.css.len(),
            css.len(),
            output.css
        );
        // Hash must be exactly 8 hex characters
        assert_eq!(
            output.hash.len(),
            8,
            "Hash should be 8 hex chars, got: {}",
            output.hash
        );
        assert!(
            output.hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should be hexadecimal: {}",
            output.hash
        );
    }

    /// T8: @tailwind Directive Injection (R5)
    ///
    /// Verifies that @tailwind base and @tailwind utilities are replaced with
    /// the generated Preflight and utility CSS respectively.
    #[test]
    fn t8_tailwind_directive_injection() {
        let pipeline = make_pipeline_cfg(no_content_config(), false);
        let css = "@tailwind base;\n@tailwind utilities;";
        let result = pipeline.process_source(css, Path::new("."), Some(cls(&["flex"])));
        assert!(result.is_ok(), "Pipeline should succeed: {:?}", result);
        let output = result.unwrap();
        // Preflight is injected at @tailwind base — it contains box-sizing
        assert!(
            output.css.contains("box-sizing"),
            "Output should contain Preflight CSS (box-sizing) from @tailwind base: {}",
            output.css
        );
        // flex utility is injected at @tailwind utilities
        assert!(
            output.css.contains("display") && output.css.contains("flex"),
            "Output should contain .flex display rule from @tailwind utilities: {}",
            output.css
        );
    }

    /// T9: @apply Expansion (R5)
    ///
    /// Verifies that @apply directives are expanded to CSS declarations.
    #[test]
    fn t9_apply_expansion() {
        let pipeline = make_pipeline(false);
        let css = ".btn { @apply flex rounded px-4; }";
        let result = pipeline.process_source(css, Path::new("."), None);
        assert!(result.is_ok(), "Pipeline should succeed: {:?}", result);
        let output = result.unwrap();
        // @apply must not remain in the output
        assert!(
            !output.css.contains("@apply"),
            "Output should not contain @apply: {}",
            output.css
        );
        // flex → display: flex
        assert!(
            output.css.contains("display") && output.css.contains("flex"),
            "Output should contain 'display: flex' from @apply flex: {}",
            output.css
        );
        // rounded → border-radius: 0.25rem
        assert!(
            output.css.contains("border-radius"),
            "Output should contain border-radius from @apply rounded: {}",
            output.css
        );
        // px-4 → padding-left + padding-right
        assert!(
            output.css.contains("padding"),
            "Output should contain padding from @apply px-4: {}",
            output.css
        );
    }

    /// T10: @layer Custom Base (R5)
    ///
    /// Verifies that @layer base { ... } is inlined into the base layer output.
    #[test]
    fn t10_layer_custom_base() {
        let pipeline = make_pipeline_cfg(no_content_config(), false);
        let css =
            "@tailwind base;\n@tailwind utilities;\n@layer base { h1 { font-size: 2.25rem; } }";
        let result = pipeline.process_source(css, Path::new("."), Some(cls(&[])));
        assert!(result.is_ok(), "Pipeline should succeed: {:?}", result);
        let output = result.unwrap();
        // Custom h1 rule should appear in output
        assert!(
            output.css.contains("h1") && output.css.contains("font-size"),
            "Output should contain custom h1 font-size rule from @layer base: {}",
            output.css
        );
        // @layer block should not remain verbatim
        assert!(
            !output.css.contains("@layer base"),
            "Output should not contain raw @layer base directive: {}",
            output.css
        );
    }

    /// T11: CSS Variable Theme Color (R6)
    ///
    /// Verifies that extended theme colors generate CSS variable references.
    #[test]
    fn t11_css_variable_theme_color() {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), "var(--primary)".to_string());
        let config = TailwindConfig {
            content: vec![],
            dark_mode: DarkMode::Class,
            theme: ThemeConfig {
                extend_colors: colors,
            },
            plugins: vec![],
        };
        let pipeline = make_pipeline_cfg(config, false);
        let css = "@tailwind utilities;";
        let result = pipeline.process_source(css, Path::new("."), Some(cls(&["text-primary"])));
        assert!(result.is_ok(), "Pipeline should succeed: {:?}", result);
        let output = result.unwrap();
        // .text-primary should reference --primary
        assert!(
            output.css.contains("--primary"),
            "Output should reference --primary CSS variable: {}",
            output.css
        );
    }

    /// T12: Dark Mode Class Strategy (R7)
    ///
    /// Verifies that dark: variant generates .dark-scoped selectors.
    #[test]
    fn t12_dark_mode_class_strategy() {
        let config = TailwindConfig {
            content: vec![],
            dark_mode: DarkMode::Class,
            theme: ThemeConfig::default(),
            plugins: vec![],
        };
        let pipeline = make_pipeline_cfg(config, false);
        let css = "@tailwind utilities;";
        let result =
            pipeline.process_source(css, Path::new("."), Some(cls(&["dark:bg-slate-800"])));
        assert!(result.is_ok(), "Pipeline should succeed: {:?}", result);
        let output = result.unwrap();
        // Selector must be scoped to .dark
        assert!(
            output.css.contains(".dark"),
            "Output should contain .dark selector for dark: variant: {}",
            output.css
        );
        // Background color for slate-800 must be emitted
        assert!(
            output.css.contains("background-color"),
            "Output should contain background-color for dark:bg-slate-800: {}",
            output.css
        );
    }
}

// ─── lightningcss integration ─────────────────────────────────────────────────

/// Apply `lightningcss` transforms to a CSS string.
///
/// When `minify` is `true`, the output is compacted (no comments, minimal
/// whitespace).  Returns `(css, source_map)`.
fn apply_lightningcss(css: &str, minify: bool) -> Result<(String, Option<String>)> {
    use lightningcss::printer::PrinterOptions;
    use lightningcss::stylesheet::{ParserOptions, StyleSheet};

    let stylesheet = StyleSheet::parse(css, ParserOptions::default())
        .map_err(|e| anyhow::anyhow!("CSS parse error: {}", e))?;

    let result = stylesheet
        .to_css(PrinterOptions {
            minify,
            ..Default::default()
        })
        .map_err(|e| anyhow::anyhow!("CSS print error: {:?}", e))?;

    // lightningcss 1.0.0-alpha.57 source maps require a SourceMap object;
    // inline source maps are not supported in this configuration.
    Ok((result.code, None))
}
// CODEGEN-END
