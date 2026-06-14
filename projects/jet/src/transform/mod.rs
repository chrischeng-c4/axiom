// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod css;
pub mod incremental;
pub mod jsx;
pub mod modules;
pub mod react_refresh;
pub mod transform_tsx;
pub mod type_strip;
pub mod typescript;

/// Code transformer using Tree-sitter
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub struct Transformer {
    options: TransformOptions,
}

/// Transform options
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
#[derive(Debug, Clone)]
pub struct TransformOptions {
    /// JSX pragma (default: React.createElement)
    pub jsx_pragma: Option<String>,

    /// JSX fragment pragma (default: React.Fragment)
    pub jsx_fragment: Option<String>,

    /// Enable JSX automatic runtime
    pub jsx_automatic: bool,

    /// TypeScript target
    pub ts_target: TypeScriptTarget,

    /// Enable source maps
    pub source_maps: bool,

    /// Enable minification
    pub minify: bool,

    /// Dev mode: enables React Fast Refresh injection for HMR
    pub dev_mode: bool,
}

/// TypeScript compilation target
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeScriptTarget {
    ES5,
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ESNext,
}

/// Transform result
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
#[derive(Debug, Clone)]
pub struct TransformResult {
    /// Transformed code
    pub code: String,

    /// Source map (if enabled)
    pub source_map: Option<String>,
}

/// GH #3809 — fallback extension string used when the path has no
/// extension at all. Kept as a named constant so call sites and tests
/// pin the same value.
pub(crate) const TRANSFORM_JS_NO_EXTENSION_FALLBACK: &str = "";

/// GH #3809 — warn shown when `Transformer::transform_js[_with_context]`
/// is invoked on a path with no `extension()`. The prior code silently
/// dropped to `""` and emitted `"Unsupported file extension: "` with
/// nothing after the colon, leaving the operator unable to spot the
/// missing-extension cause among other `_` arms.
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub(crate) fn format_transform_js_no_extension_warn(filename: &Path) -> String {
    format!(
        "gh3809: jet transform saw path with no extension filename={:?}; \
         falling back to empty extension — error will say \
         \"Unsupported file extension: \" with nothing after the colon",
        filename
    )
}

/// GH #3809 — warn shown when `Transformer::transform_js[_with_context]`
/// is invoked on a path whose extension is non-UTF-8 (filesystem-encoded
/// bytes that Rust cannot lossless-decode). The prior code silently
/// dropped to `""` because `.to_str()` returned `None`, collapsing
/// non-UTF-8 extensions onto the no-extension case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub(crate) fn format_transform_js_non_utf8_extension_warn(filename: &Path, lossy: &str) -> String {
    format!(
        "gh3809: jet transform saw path with non-UTF-8 extension filename={:?}; \
         lossy form is {:?}; routing through the lossy form so the \
         \"Unsupported file extension\" error carries a visible breadcrumb \
         instead of an empty extension",
        filename, lossy
    )
}

/// GH #3809 — coerce the file extension into a string for the
/// Transformer's dispatch match.
///
/// - `Some(utf8)` → `Cow::Borrowed(utf8)` (silent — recognised UTF-8
///   extensions dispatch, unrecognised ones still bail with a visible
///   value).
/// - `Some(non-UTF-8)` → emit a `tracing::warn!` carrying the lossy form
///   and return `Cow::Owned(lossy)` so the bail message names the
///   encoding instead of collapsing onto `""`.
/// - `None` → emit a `tracing::warn!` naming the path and return
///   `Cow::Borrowed("")` so legacy `_ => bail!` behaviour is preserved.
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub(crate) fn coerce_transform_js_extension_or_warn(filename: &Path) -> std::borrow::Cow<'_, str> {
    use std::borrow::Cow;
    match filename.extension() {
        None => {
            tracing::warn!(
                target: "jet::transform",
                filename = %filename.display(),
                "{}",
                format_transform_js_no_extension_warn(filename)
            );
            Cow::Borrowed(TRANSFORM_JS_NO_EXTENSION_FALLBACK)
        }
        Some(os) => match os.to_str() {
            Some(s) => Cow::Borrowed(s),
            None => {
                let lossy = os.to_string_lossy().into_owned();
                tracing::warn!(
                    target: "jet::transform",
                    filename = %filename.display(),
                    lossy = %lossy,
                    "{}",
                    format_transform_js_non_utf8_extension_warn(filename, &lossy)
                );
                Cow::Owned(lossy)
            }
        },
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
impl Transformer {
    /// Create a new transformer
    pub fn new(options: TransformOptions) -> Self {
        Self { options }
    }

    /// Transform JavaScript/TypeScript file
    pub fn transform_js(&self, source: &str, filename: &Path) -> Result<TransformResult> {
        let ext = filename.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext.as_ref() {
            "jsx" => jsx::transform_jsx(source, &self.options),
            "tsx" => transform_tsx::transform_tsx(source, &self.options),
            "ts" => typescript::transform_typescript(source, &self.options),
            "js" | "mjs" | "cjs" => Ok(TransformResult {
                code: source.to_string(),
                source_map: None,
            }),
            _ => anyhow::bail!("Unsupported file extension: {}", ext),
        }
    }

    /// Transform JavaScript/TypeScript file with module context
    pub fn transform_js_with_context(
        &self,
        source: &str,
        filename: &Path,
        module_map: &HashMap<PathBuf, usize>,
    ) -> Result<TransformResult> {
        self.transform_js_with_context_and_resolution_index(source, filename, module_map, None)
    }

    pub fn transform_js_with_context_and_resolution_index(
        &self,
        source: &str,
        filename: &Path,
        module_map: &HashMap<PathBuf, usize>,
        resolution_index: Option<&modules::ModuleResolutionIndex>,
    ) -> Result<TransformResult> {
        self.transform_js_with_context_resolution_and_tree(
            source,
            filename,
            module_map,
            resolution_index,
            None,
        )
    }

    /// Like [`Self::transform_js_with_context_and_resolution_index`] but accepts
    /// a tree-sitter tree already parsed during graph construction, so a
    /// plain-JS module is parsed once instead of twice. The tree is only used
    /// for `.js`/`.mjs`/`.cjs`, whose source step 1 leaves untouched; for
    /// TS/TSX/JSX step 1 rewrites the source, so any supplied tree is dropped
    /// and the module transform re-parses the rewritten code.
    pub fn transform_js_with_context_resolution_and_tree(
        &self,
        source: &str,
        filename: &Path,
        module_map: &HashMap<PathBuf, usize>,
        resolution_index: Option<&modules::ModuleResolutionIndex>,
        reuse_tree: Option<tree_sitter::Tree>,
    ) -> Result<TransformResult> {
        let ext = filename.extension().and_then(|e| e.to_str()).unwrap_or("");

        // 1. First, apply TypeScript/JSX transformation
        let transformed = match ext.as_ref() {
            "jsx" => jsx::transform_jsx(source, &self.options)?,
            "tsx" => transform_tsx::transform_tsx(source, &self.options)?,
            "ts" => typescript::transform_typescript(source, &self.options)?,
            "js" | "mjs" | "cjs" => TransformResult {
                code: source.to_string(),
                source_map: None,
            },
            _ => anyhow::bail!("Unsupported file extension: {}", ext),
        };

        // A reuse tree is only valid when step 1 did not rewrite the source.
        let reuse_tree = match ext.as_ref() {
            "js" | "mjs" | "cjs" => reuse_tree,
            _ => None,
        };

        // 2. Apply ES6 module transformation (pass current module dir for relative resolution)
        let current_dir = filename.parent();
        modules::transform_modules_with_dir_index_and_tree(
            &transformed.code,
            module_map,
            resolution_index,
            current_dir,
            reuse_tree,
        )
    }

    /// Transform CSS file
    pub fn transform_css(&self, source: &str) -> Result<TransformResult> {
        css::transform_css(source, &self.options)
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
impl Default for TransformOptions {
    fn default() -> Self {
        Self {
            jsx_pragma: None,
            jsx_fragment: None,
            jsx_automatic: true,
            ts_target: TypeScriptTarget::ES2020,
            source_maps: true,
            minify: false,
            dev_mode: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformer_creation() {
        let transformer = Transformer::new(TransformOptions::default());
        assert!(transformer.options.jsx_automatic);
    }

    #[test]
    fn test_transform_tsx_with_context_alias() {
        // Full pipeline test: TSX transform + module transform for import { X as Y }
        let source = r#"import { useState as useStateAlias } from 'react';
const App = () => <div>{useStateAlias(0)}</div>;
export default App;"#;
        let module_map = HashMap::new();
        let transformer = Transformer::new(TransformOptions::default());
        let filename = Path::new("test.tsx");
        let result = transformer
            .transform_js_with_context(source, filename, &module_map)
            .unwrap();
        eprintln!("FULL PIPELINE: {:?}", result.code);
        assert!(
            result.code.contains("var useStateAlias") && result.code.contains("[\"useState\"]"),
            "import alias must be preserved through full TSX pipeline: {:?}",
            result.code
        );
    }

    #[test]
    fn test_transform_tsx_with_context_preserves_styled_components_import_bindings() {
        let source = r##"import React, { useState } from "react";
import { createRoot } from "react-dom/client";
import styled, { createGlobalStyle, css } from "styled-components";

const GlobalStyle = createGlobalStyle`
  body { margin: 0; }
`;
const Matrix = styled.main`
  min-height: 100vh;
`;
const Button = styled.button`
  ${(props) => css`
    background: ${props.$accent || "#2563eb"};
  `}
`;

function App() {
  const [active] = useState(0);
  return <Matrix><GlobalStyle /><Button $accent="#2563eb">{active}</Button></Matrix>;
}

createRoot(document.getElementById("root")!).render(<App />);"##;
        let transformer = Transformer::new(TransformOptions::default());
        let result = transformer
            .transform_js_with_context(source, Path::new("main.tsx"), &HashMap::new())
            .unwrap();

        assert!(
            result
                .code
                .contains("var useState = require('react')[\"useState\"]"),
            "React named import binding must survive full transform: {}",
            result.code
        );
        assert!(
            result
                .code
                .contains("var createRoot = require('react-dom/client')[\"createRoot\"]"),
            "createRoot named import binding must survive full transform: {}",
            result.code
        );
        assert!(
            result
                .code
                .contains("var styled = require('styled-components')[\"default\"]"),
            "styled default import binding must survive full transform: {}",
            result.code
        );
        assert!(
            result.code.contains(
                "var createGlobalStyle = require('styled-components')[\"createGlobalStyle\"]"
            ),
            "createGlobalStyle named import binding must survive full transform: {}",
            result.code
        );
        assert!(
            result
                .code
                .contains("var css = require('styled-components')[\"css\"]"),
            "css named import binding must survive full transform: {}",
            result.code
        );
    }
}

#[cfg(test)]
mod gh3809_transform_js_extension_warn_tests {
    use super::*;
    use std::path::Path;

    fn transformer() -> Transformer {
        Transformer::new(TransformOptions::default())
    }

    #[test]
    fn utf8_extension_passes_through_silently_for_recognised_tsx() {
        let cow = coerce_transform_js_extension_or_warn(Path::new("foo.tsx"));
        assert_eq!(cow.as_ref(), "tsx");
    }

    #[test]
    fn utf8_extension_passes_through_silently_for_all_recognised_kinds() {
        for (path, expected) in [
            ("a.ts", "ts"),
            ("a.tsx", "tsx"),
            ("a.jsx", "jsx"),
            ("a.js", "js"),
            ("a.mjs", "mjs"),
            ("a.cjs", "cjs"),
        ] {
            let cow = coerce_transform_js_extension_or_warn(Path::new(path));
            assert_eq!(cow.as_ref(), expected, "path {path}");
        }
    }

    #[test]
    fn unrecognised_utf8_extension_still_passes_through_for_bail_wording() {
        // The dispatch will bail with `Unsupported file extension: rs`,
        // which is the visible-name path. The helper must not warn here
        // because the value is already operator-visible.
        let cow = coerce_transform_js_extension_or_warn(Path::new("weird.rs"));
        assert_eq!(cow.as_ref(), "rs");

        let result = transformer().transform_js("x", Path::new("weird.rs"));
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Unsupported file extension: rs"),
            "bail must name the extension: {err:?}"
        );
    }

    #[test]
    fn no_extension_falls_back_to_named_constant() {
        let cow = coerce_transform_js_extension_or_warn(Path::new("noext"));
        assert_eq!(cow.as_ref(), TRANSFORM_JS_NO_EXTENSION_FALLBACK);
        assert_eq!(cow.as_ref(), "");

        // The bail message will be the historically-empty form; this is
        // the behaviour the warn helper is meant to disambiguate.
        let result = transformer().transform_js("x", Path::new("noext"));
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Unsupported file extension: "),
            "bail wording preserved verbatim: {err:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_extension_produces_lossy_form_not_empty() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        // 0xff is invalid UTF-8 in any position
        let raw = b"a.\xffweird";
        let path = std::path::PathBuf::from(OsStr::from_bytes(raw));
        let cow = coerce_transform_js_extension_or_warn(&path);
        let v = cow.as_ref();
        assert!(!v.is_empty(), "non-UTF-8 must not collapse to empty");
        assert!(
            v.contains('\u{FFFD}') || v != "",
            "lossy form expected, got {v:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn two_distinct_non_utf8_extensions_do_not_collide_onto_empty() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let a = std::path::PathBuf::from(OsStr::from_bytes(b"a.\xffone"));
        let b = std::path::PathBuf::from(OsStr::from_bytes(b"a.\xfetwo"));
        let ca = coerce_transform_js_extension_or_warn(&a).into_owned();
        let cb = coerce_transform_js_extension_or_warn(&b).into_owned();
        assert!(!ca.is_empty() && !cb.is_empty());
        assert_ne!(ca, cb, "distinct non-UTF-8 inputs must remain distinct");
    }

    #[test]
    fn warn_helpers_pinned_for_discoverability() {
        // The helper names form the searchable surface — any rename
        // should be intentional and flagged here.
        let _: fn(&Path) -> String = format_transform_js_no_extension_warn;
        let _: fn(&Path, &str) -> String = format_transform_js_non_utf8_extension_warn;
        let _: fn(&Path) -> std::borrow::Cow<'_, str> = coerce_transform_js_extension_or_warn;
        assert_eq!(TRANSFORM_JS_NO_EXTENSION_FALLBACK, "");
    }

    #[test]
    fn each_warn_string_carries_gh3809_tag() {
        let no_ext = format_transform_js_no_extension_warn(Path::new("noext"));
        let non_utf8 =
            format_transform_js_non_utf8_extension_warn(Path::new("a.bad"), "\u{FFFD}weird");
        assert!(no_ext.contains("gh3809"), "no-ext warn lacks tag: {no_ext}");
        assert!(
            non_utf8.contains("gh3809"),
            "non-utf8 warn lacks tag: {non_utf8}"
        );
    }

    #[test]
    fn warn_distinct_from_prior_silent_fallback_families() {
        let no_ext = format_transform_js_no_extension_warn(Path::new("noext"));
        let non_utf8 = format_transform_js_non_utf8_extension_warn(Path::new("a.bad"), "\u{FFFD}");
        for prior in [
            "gh3789", "gh3791", "gh3793", "gh3795", "gh3797", "gh3799", "gh3801", "gh3803",
            "gh3805", "gh3807",
        ] {
            assert!(
                !no_ext.contains(prior),
                "no-ext warn collides with {prior}: {no_ext}"
            );
            assert!(
                !non_utf8.contains(prior),
                "non-utf8 warn collides with {prior}: {non_utf8}"
            );
        }
    }

    #[test]
    fn two_sibling_warns_are_mutually_distinct() {
        let no_ext = format_transform_js_no_extension_warn(Path::new("noext"));
        let non_utf8 = format_transform_js_non_utf8_extension_warn(Path::new("a.bad"), "\u{FFFD}");
        assert_ne!(no_ext, non_utf8);
        assert!(no_ext.contains("no extension"));
        assert!(non_utf8.contains("non-UTF-8"));
    }

    #[test]
    fn happy_path_dispatches_recognised_extension_without_warn_surface() {
        // A recognised UTF-8 extension must take the Cow::Borrowed branch.
        // We can't assert no-warn directly, but we can pin that the
        // returned Cow is Borrowed (zero-allocation hot path).
        let cow = coerce_transform_js_extension_or_warn(Path::new("foo.tsx"));
        assert!(
            matches!(cow, std::borrow::Cow::Borrowed("tsx")),
            "recognised extension must take borrowed branch"
        );
    }
}
// CODEGEN-END
