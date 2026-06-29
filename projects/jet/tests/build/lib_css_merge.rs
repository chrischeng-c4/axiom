// HANDWRITE-BEGIN gap="missing-generator:unit-test:992dc696" tracker="pending-tracker" reason="Tests: css_merge produces style.css with dep CSS in declared order; raw_copy lands icons/images/audio verbatim at deep-import paths; neither configured = unchanged."
//! Integration tests for `jet build --lib` CSS cascade-merge + raw-asset copy.
//!
//! These replace the two bespoke fe-shared vite plugins jet could not express:
//!   * `mergeDepStyles` — concatenate dependent packages' `style.css` into the
//!     meta-package's output `style.css` in a DECLARED (cascade) order, and
//!   * `copyRawAssets`   — copy configured directories (`icons/`, …) verbatim
//!     into the lib output so consumers can deep-import the assets.
//!
//! Coverage:
//!   (a) css_merge concatenates two dependent `style.css` files into
//!       `out_dir/style.css` in the DECLARED order (first file's rule appears
//!       before the second's),
//!   (b) raw_copy copies an `icons/` fixture dir (with a nested file) verbatim
//!       into out_dir, preserving the subpath,
//!   (c) a lib build with neither config set is unchanged — no `style.css` and
//!       no extra asset files appear in the output.

use jet::bundler::types::OutputFormat;
use jet::bundler::types::SourceMapOption;
use jet::bundler::{build_library, LibBuildOptions, LibBuildResult, RawCopyDir};
use std::collections::HashSet;
use tempfile::tempdir;

/// Write a file, creating parent dirs as needed.
fn write_file(base: &std::path::Path, rel: &str, content: &str) {
    let path = base.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, content).unwrap();
}

/// A minimal publishable library fixture: one JS entry, no deps. The CSS-merge
/// and raw-copy steps run *after* this normal emit, so the fixture only needs
/// enough to make `build_library` succeed.
fn write_minimal_lib(root: &std::path::Path) {
    write_file(
        root,
        "package.json",
        r#"{
            "name": "meta-pkg",
            "version": "1.0.0",
            "module": "./src/index.js"
        }"#,
    );
    write_file(root, "src/index.js", "export function go() { return 1; }\n");
}

/// Build the library with the given css_merge + raw_copy config. Untyped JS
/// entry → declarations off (exercised separately in `library_dts.rs`).
fn run_lib_build(
    root: &std::path::Path,
    css_merge: Vec<String>,
    raw_copy: Vec<RawCopyDir>,
) -> LibBuildResult {
    let options = LibBuildOptions {
        project_root: root.to_path_buf(),
        out_dir: root.join("dist"),
        formats: vec![OutputFormat::Esm],
        conditions: vec!["import".to_string(), "default".to_string()],
        extra_externals: HashSet::new(),
        preserve_modules: false,
        declaration: false,
        library_global_name: None,
        entry: Vec::new(),
        css_merge,
        raw_copy,
        sourcemap: SourceMapOption::None,
    };
    build_library(options).expect("library build must succeed")
}

// ──────────────────────────────────────────────────────────────────────────
// (a) css_merge concatenates dependent style.css in DECLARED (cascade) order
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn css_merge_concatenates_deps_in_declared_order() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write_minimal_lib(root);

    // Two dependent packages' built style.css. Declared order = cascade order:
    // button is listed FIRST, so its rule must appear BEFORE input's.
    write_file(
        root,
        "deps/button/dist/style.css",
        ".button { color: red; }\n",
    );
    write_file(
        root,
        "deps/input/dist/style.css",
        ".input { color: blue; }\n",
    );

    let result = run_lib_build(
        root,
        vec![
            "deps/button/dist/style.css".to_string(),
            "deps/input/dist/style.css".to_string(),
        ],
        Vec::new(),
    );

    let out_css = root.join("dist/style.css");
    assert!(
        out_css.is_file(),
        "css_merge must produce out_dir/style.css"
    );
    let merged = std::fs::read_to_string(&out_css).unwrap();

    let button_at = merged
        .find(".button { color: red; }")
        .expect("button rule must be present in merged style.css");
    let input_at = merged
        .find(".input { color: blue; }")
        .expect("input rule must be present in merged style.css");
    assert!(
        button_at < input_at,
        "declared order is cascade order: the first-listed file's rule must \
         appear before the second's. merged:\n{merged}"
    );

    // The merged style.css is recorded as an asset side-effect.
    assert!(
        result.assets.iter().any(|a| a.path == out_css),
        "merged style.css must be recorded in LibBuildResult.assets"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (b) raw_copy copies a directory tree verbatim, preserving nested subpaths
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn raw_copy_copies_dir_tree_verbatim_preserving_subpath() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write_minimal_lib(root);

    // An icons dir with a NESTED file — the subpath must be preserved so a
    // consumer can deep-import `@meta-pkg/icons/social/x.svg`.
    let svg = "<svg><path d=\"M0 0h1v1H0z\"/></svg>\n";
    write_file(root, "assets/icons/social/x.svg", svg);

    run_lib_build(
        root,
        Vec::new(),
        vec![RawCopyDir {
            from: "assets/icons".to_string(),
            to: Some("icons".to_string()),
        }],
    );

    let copied = root.join("dist/icons/social/x.svg");
    assert!(
        copied.is_file(),
        "raw_copy must land the nested file at dist/icons/social/x.svg \
         (subpath preserved)"
    );
    assert_eq!(
        std::fs::read_to_string(&copied).unwrap(),
        svg,
        "copied asset must be byte-identical to the source"
    );
}

#[test]
fn raw_copy_default_dest_mirrors_from_path() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write_minimal_lib(root);

    write_file(root, "assets/images/logo.png", "PNGDATA");

    // No `to` → destination mirrors the `from` relative path under out_dir.
    run_lib_build(
        root,
        Vec::new(),
        vec![RawCopyDir {
            from: "assets/images".to_string(),
            to: None,
        }],
    );

    let copied = root.join("dist/assets/images/logo.png");
    assert!(
        copied.is_file(),
        "absent `to` must mirror the `from` path: dist/assets/images/logo.png"
    );
    assert_eq!(std::fs::read_to_string(&copied).unwrap(), "PNGDATA");
}

#[test]
fn wildcard_asset_exports_copy_matching_src_assets_to_dist_prefix() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write_file(
        root,
        "package.json",
        r#"{
            "name": "wildcard-assets",
            "version": "1.0.0",
            "module": "./src/index.js",
            "exports": {
                ".": "./dist/index.js",
                "./icons/*": "./dist/icons/*"
            }
        }"#,
    );
    write_file(root, "src/index.js", "export const ok = true;\n");
    let svg = "<svg><path d=\"M0 0h1v1H0z\"/></svg>\n";
    write_file(root, "src/icons/star.svg", svg);

    let result = run_lib_build(root, Vec::new(), Vec::new());

    let copied = root.join("dist/icons/star.svg");
    assert!(
        copied.is_file(),
        "wildcard export ./icons/* -> ./dist/icons/* must copy src/icons/star.svg"
    );
    assert_eq!(
        std::fs::read_to_string(&copied).unwrap(),
        svg,
        "wildcard-copied asset must be byte-identical"
    );
    assert!(
        result.assets.iter().any(|a| a.path == copied),
        "wildcard-copied asset should be recorded in LibBuildResult.assets"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (c) neither config set → unchanged: no style.css, no extra asset files
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn no_css_merge_or_raw_copy_leaves_output_unchanged() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write_minimal_lib(root);

    let result = run_lib_build(root, Vec::new(), Vec::new());

    // The normal lib emit still happens (the JS entry), but no post-emit asset
    // side-effects fire: no merged style.css and no copied files.
    assert!(
        result.assets.is_empty(),
        "a build with neither css_merge nor raw_copy records no assets"
    );
    assert!(
        !root.join("dist/style.css").exists(),
        "no style.css must appear when css_merge is not configured"
    );

    // Only the JS entry (index.js) lives under dist/ — no extra asset files.
    let dist_files: Vec<_> = walkdir::WalkDir::new(root.join("dist"))
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(
        dist_files,
        vec!["index.js".to_string()],
        "only the JS entry should be emitted; got {dist_files:?}"
    );
}
// HANDWRITE-END
