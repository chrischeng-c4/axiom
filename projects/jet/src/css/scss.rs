// <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-src-css-scss-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! SCSS / Sass compilation.
//!
//! Compiles `.scss` (SCSS syntax) and `.sass` (indented syntax) sources to
//! plain CSS using [`grass`], a pure-Rust Sass compiler (no C deps). The
//! compiled CSS is then handed back to the rest of the CSS pipeline
//! ([`crate::css::import_resolver`] → directives → `lightningcss`), so Sass
//! features land in the same `style.css` / build CSS as plain `.css`.
//!
//! grass natively resolves `@use` / `@import` of partials (including the
//! leading-underscore `_partial.scss` convention and extensionless
//! specifiers) against the configured load path, so a `.scss` entry that
//! pulls in sibling partials is flattened in one compile pass. Supported
//! out of the box: nesting, variables, `@use` / `@import` partials, and
//! mixins (`@mixin` / `@include`).
//!
//! TODO(#204 follow-up): legacy node-sass-only quirks (e.g. `@import` of a
//! `.css` file treated as Sass, `/` division semantics) and very advanced
//! Sass modules (`sass:math`, `sass:color` builtins beyond grass coverage)
//! are not specially handled here — they degrade to grass's own behavior.

use anyhow::{Context, Result};
use std::path::Path;

/// True when `path` has a `.scss` extension (case-insensitive).
pub fn is_scss_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some(ext) if ext.eq_ignore_ascii_case("scss")
    )
}

/// True when `path` has a `.sass` extension (case-insensitive, indented
/// syntax).
pub fn is_sass_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some(ext) if ext.eq_ignore_ascii_case("sass")
    )
}

/// True when `path` is a Sass source (`.scss` or `.sass`) that must be
/// compiled to CSS before entering the lightningcss pipeline.
pub fn is_sass_family_path(path: &Path) -> bool {
    is_scss_path(path) || is_sass_path(path)
}

/// Compile a Sass source string to CSS.
///
/// - `source`: the `.scss` / `.sass` file contents.
/// - `load_path`: directory used to resolve `@use` / `@import` of partials;
///   normally the directory containing the source file.
/// - `indented`: `true` for `.sass` (indented syntax), `false` for `.scss`.
///
/// Returns the flattened CSS string. Errors carry the grass diagnostic so a
/// failing build surfaces the offending file/line.
pub fn compile_sass_source(source: &str, load_path: &Path, indented: bool) -> Result<String> {
    let syntax = if indented {
        grass::InputSyntax::Sass
    } else {
        grass::InputSyntax::Scss
    };
    let options = grass::Options::default()
        .input_syntax(syntax)
        .load_path(load_path);
    grass::from_string(source.to_string(), &options)
        .map_err(|e| anyhow::anyhow!("Sass compile error: {e}"))
}

/// Compile a Sass file (`.scss` / `.sass`) to CSS, picking the input syntax
/// from the file extension and using the file's parent directory as the
/// load path for partial resolution.
pub fn compile_sass_file(path: &Path) -> Result<String> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("reading Sass file {}", path.display()))?;
    let load_path = path.parent().unwrap_or_else(|| Path::new("."));
    compile_sass_source(&source, load_path, is_sass_path(path))
        .with_context(|| format!("compiling Sass file {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_scss_and_sass_extensions() {
        assert!(is_scss_path(Path::new("a.scss")));
        assert!(is_scss_path(Path::new("a.SCSS")));
        assert!(!is_scss_path(Path::new("a.css")));
        assert!(is_sass_path(Path::new("a.sass")));
        assert!(!is_sass_path(Path::new("a.scss")));
        assert!(is_sass_family_path(Path::new("a.scss")));
        assert!(is_sass_family_path(Path::new("a.sass")));
        assert!(!is_sass_family_path(Path::new("a.css")));
    }

    #[test]
    fn compiles_nesting_and_variables() {
        let scss = "$pad: 8px;\n.card { padding: $pad; .title { color: red; } }";
        let css = compile_sass_source(scss, Path::new("."), false).unwrap();
        // Nesting is flattened to a descendant selector.
        assert!(css.contains(".card .title"), "got: {css}");
        // Variable is resolved.
        assert!(css.contains("8px"), "got: {css}");
        assert!(
            !css.contains("$pad"),
            "variable must be resolved, got: {css}"
        );
    }

    #[test]
    fn compiles_mixins() {
        let scss = "@mixin flex { display: flex; }\n.row { @include flex; }";
        let css = compile_sass_source(scss, Path::new("."), false).unwrap();
        assert!(
            css.contains("display") && css.contains("flex"),
            "got: {css}"
        );
        assert!(!css.contains("@include"), "got: {css}");
    }

    #[test]
    fn compiles_indented_sass_syntax() {
        let sass = ".box\n  color: blue";
        let css = compile_sass_source(sass, Path::new("."), true).unwrap();
        assert!(css.contains(".box") && css.contains("blue"), "got: {css}");
    }

    #[test]
    fn compile_error_is_surfaced() {
        let scss = ".broken { color: ; "; // missing value + unclosed
        let err =
            compile_sass_source(scss, Path::new("."), false).expect_err("invalid scss must error");
        assert!(
            err.to_string().contains("Sass compile error"),
            "error should be tagged, got: {err}"
        );
    }
}

// </HANDWRITE>
