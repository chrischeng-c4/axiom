// HANDWRITE-BEGIN gap="missing-generator:unit-test:725a3278" tracker="pending-tracker" reason="Tests: a .scss with nesting + a variable compiles, resolved rules appear in build CSS; lib emits compiled SCSS into style.css; plain .css still works."
//! SCSS / Sass compilation in jet's build + lib CSS pipeline.
//!
//! jet's CSS pipeline is lightningcss (plain CSS) and cannot parse Sass.
//! `jet::css::scss` adds a grass (pure-Rust Sass) compile step that runs
//! BEFORE lightningcss, so `.scss`/`.sass` sources flow through the existing
//! pipeline (and into the lib single `style.css`).
//!
//! Coverage here:
//! - (a) a `.scss` with nesting + a variable compiles to flattened CSS rules;
//! - (b) `@use` / `@import` of a partial composes two scss files;
//! - (c) the compiled CSS is included in a build's CSS output (the exact
//!   `CssPipeline::process` path the app build's `try_process_css_entry`
//!   uses for a sibling stylesheet entry). The lib single `style.css` SCSS
//!   path is unit-covered in `src/wasm_build/mod.rs`
//!   (`copy_style_imports_compiles_scss_into_style_css`), which calls the
//!   private `copy_style_import_groups` in-crate;
//! - (d) plain `.css` still passes through the pipeline unchanged.
//!
//! TODO(#204 follow-up): legacy node-sass-only quirks and very advanced Sass
//! modules are out of scope — see `src/css/scss.rs`.

use jet::bundler::imports::{classify_style_import, is_scss_specifier, StyleImportRoute};
use jet::css::scss::{compile_sass_file, compile_sass_source, is_sass_family_path};
use jet::css::{CssPipeline, TailwindConfig};
use std::path::Path;

/// A CSS pipeline with no Tailwind content scanning, so output is fully
/// determined by the input stylesheet (no repo-dependent utility CSS).
fn pipeline(root: &Path, production: bool) -> CssPipeline {
    let config = TailwindConfig {
        content: vec![],
        ..TailwindConfig::default()
    };
    CssPipeline::new(root.to_path_buf(), config, production)
}

// ─── (a) nesting + variable compile to flattened rules ───────────────────────

#[test]
fn scss_nesting_and_variable_compile_to_flattened_rules() {
    let scss = "\
$primary: #336699;
.menu {
  color: $primary;
  .item {
    padding: 4px 8px;
    &:hover { color: $primary; }
  }
}";
    let css = compile_sass_source(scss, Path::new("."), false).unwrap();

    // Nesting flattened into descendant selectors.
    assert!(
        css.contains(".menu .item"),
        "nesting must flatten, got:\n{css}"
    );
    // `&:hover` resolves to the parent selector.
    assert!(
        css.contains(".menu .item:hover"),
        "parent selector must resolve, got:\n{css}"
    );
    // The variable is resolved to its value and does not survive raw.
    assert!(
        css.contains("#336699"),
        "variable must resolve, got:\n{css}"
    );
    assert!(!css.contains("$primary"), "no raw variables, got:\n{css}");
    assert!(
        css.contains("4px 8px"),
        "declarations preserved, got:\n{css}"
    );
}

// ─── (b) @use / @import of a partial composes two scss files ─────────────────

#[test]
fn scss_use_partial_composes_two_files() {
    let dir = tempfile::tempdir().unwrap();
    // Partial: leading-underscore convention, referenced as `tokens`.
    std::fs::write(
        dir.path().join("_tokens.scss"),
        "$brand: #ff5722;\n$space: 16px;",
    )
    .unwrap();
    // Entry: `@use` the partial and consume its namespaced variables.
    std::fs::write(
        dir.path().join("main.scss"),
        "@use 'tokens';\n.btn { background: tokens.$brand; margin: tokens.$space; }",
    )
    .unwrap();

    let css = compile_sass_file(&dir.path().join("main.scss")).unwrap();
    assert!(css.contains(".btn"), "got:\n{css}");
    assert!(
        css.contains("#ff5722"),
        "partial @use var must resolve, got:\n{css}"
    );
    assert!(
        css.contains("16px"),
        "partial @use var must resolve, got:\n{css}"
    );
    assert!(!css.contains("@use"), "no raw @use, got:\n{css}");
}

#[test]
fn scss_import_partial_composes_two_files() {
    let dir = tempfile::tempdir().unwrap();
    // Legacy `@import` of a partial (global scope, no namespace).
    std::fs::write(dir.path().join("_vars.scss"), "$accent: #00bcd4;").unwrap();
    std::fs::write(
        dir.path().join("app.scss"),
        "@import 'vars';\n.link { color: $accent; }",
    )
    .unwrap();

    let css = compile_sass_file(&dir.path().join("app.scss")).unwrap();
    assert!(css.contains(".link"), "got:\n{css}");
    assert!(
        css.contains("#00bcd4"),
        "imported var must resolve, got:\n{css}"
    );
    assert!(!css.contains("@import"), "no raw @import, got:\n{css}");
}

// ─── (c) compiled CSS appears in a build's CSS output ────────────────────────

#[test]
fn scss_entry_compiles_into_build_css_output() {
    // Mirrors the app build's `try_process_css_entry`: a `.scss` sibling
    // entry is run through `CssPipeline::process` and emitted as a hashed
    // `.css` asset. We assert the resolved rules land in `output.css`.
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("index.scss"),
        "$radius: 6px;\n.card { border-radius: $radius; .body { padding: 12px; } }",
    )
    .unwrap();

    let out = pipeline(dir.path(), false)
        .process(&dir.path().join("index.scss"))
        .expect("scss entry should compile through the CSS pipeline");

    assert!(out.css.contains(".card"), "got:\n{}", out.css);
    assert!(
        out.css.contains(".card .body"),
        "nesting flattened, got:\n{}",
        out.css
    );
    assert!(
        out.css.contains("6px"),
        "variable resolved, got:\n{}",
        out.css
    );
    assert!(
        out.css.contains("12px"),
        "nested decl present, got:\n{}",
        out.css
    );
    assert!(
        !out.css.contains("$radius"),
        "no raw vars, got:\n{}",
        out.css
    );
    // Hash is the standard 8-hex content hash the build emits.
    assert_eq!(out.hash.len(), 8, "hash: {}", out.hash);
}

#[test]
fn scss_partial_imported_from_css_entry_is_compiled() {
    // A plain `.css` entry that `@import`s a `.scss` partial: the import
    // resolver routes the `.scss` file through grass before inlining, so the
    // Sass rules appear (flattened) in the final build CSS.
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("theme.scss"),
        "$fg: #222;\n.note { color: $fg; strong { font-weight: 700; } }",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("index.css"),
        "@import \"./theme.scss\";\n.page { margin: 0; }",
    )
    .unwrap();

    let out = pipeline(dir.path(), false)
        .process(&dir.path().join("index.css"))
        .expect("css entry importing a scss partial should compile");

    assert!(
        out.css.contains(".note strong"),
        "scss nesting flattened, got:\n{}",
        out.css
    );
    assert!(
        out.css.contains("#222"),
        "scss var resolved, got:\n{}",
        out.css
    );
    assert!(
        out.css.contains(".page"),
        "plain css preserved, got:\n{}",
        out.css
    );
    assert!(
        !out.css.contains("@import"),
        "@import inlined, got:\n{}",
        out.css
    );
}

// ─── (d) plain .css still passes through unchanged ───────────────────────────

#[test]
fn plain_css_entry_passes_through_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("plain.css"),
        ".btn { color: red; }\n.btn:hover { color: blue; }",
    )
    .unwrap();

    let out = pipeline(dir.path(), false)
        .process(&dir.path().join("plain.css"))
        .expect("plain css must still process");

    // Plain CSS is untouched by the SCSS step; lightningcss may normalize
    // color keywords to hex, so accept either form.
    assert!(out.css.contains(".btn"), "got:\n{}", out.css);
    assert!(
        out.css.contains("red") || out.css.contains("#f00") || out.css.contains("#ff0000"),
        "got:\n{}",
        out.css
    );
    assert!(out.css.contains(":hover"), "got:\n{}", out.css);
}

#[test]
fn plain_css_is_not_treated_as_sass() {
    // Extension routing: `.css` is never diverted through the SCSS compile.
    assert!(!is_sass_family_path(Path::new("a.css")));
    assert!(is_sass_family_path(Path::new("a.scss")));
    assert!(is_sass_family_path(Path::new("a.sass")));

    assert!(!is_scss_specifier("./styles.css"));
    assert!(is_scss_specifier("./styles.scss"));
    assert_eq!(
        classify_style_import("./styles.css"),
        StyleImportRoute::PlainCss
    );
    assert_eq!(
        classify_style_import("./styles.scss"),
        StyleImportRoute::Sass
    );
    assert_eq!(
        classify_style_import("./styles.sass"),
        StyleImportRoute::Sass
    );
}

// ─── indented `.sass` syntax ─────────────────────────────────────────────────

#[test]
fn indented_sass_syntax_compiles() {
    let dir = tempfile::tempdir().unwrap();
    // `.sass` uses indentation rather than braces/semicolons.
    std::fs::write(
        dir.path().join("ind.sass"),
        "$c: green\n.box\n  color: $c\n  .inner\n    padding: 2px\n",
    )
    .unwrap();

    let css = compile_sass_file(&dir.path().join("ind.sass")).unwrap();
    assert!(
        css.contains(".box .inner"),
        "indented nesting flattened, got:\n{css}"
    );
    assert!(css.contains("green"), "indented var resolved, got:\n{css}");
    assert!(css.contains("2px"), "got:\n{css}");
}
// HANDWRITE-END
