// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
use std::collections::HashMap;

/// Build a JSON importmap from pre-bundled dependencies, ESM dependencies, and polyfills.
///
/// `prebundled` — map from bare specifier (e.g. `"react"`) to `.jet/` filename
/// `esm_deps` — map from bare specifier (e.g. `"@tanstack/react-query"`) to entry path
/// `polyfill_builtins` — list of builtin names that have polyfill files
///
/// Returns a JSON string suitable for embedding in `<script type="importmap">`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn build_importmap(
    prebundled: &HashMap<String, String>,
    polyfill_builtins: &[String],
) -> String {
    build_importmap_full(prebundled, &HashMap::new(), polyfill_builtins)
}

/// Full importmap builder including ESM deps.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn build_importmap_full(
    prebundled: &HashMap<String, String>,
    esm_deps: &HashMap<String, String>,
    polyfill_builtins: &[String],
) -> String {
    let mut imports: HashMap<String, String> = HashMap::new();

    // Pre-bundled dependency mappings (CJS → ESM converted)
    for (specifier, filename) in prebundled {
        imports.insert(
            specifier.clone(),
            format!("/node_modules/.jet/{}", filename),
        );
    }

    // ESM dependency mappings (served directly from node_modules)
    for (specifier, entry_path) in esm_deps {
        if !imports.contains_key(specifier) {
            imports.insert(specifier.clone(), format!("/node_modules/{}", entry_path));
        }
    }

    // Polyfill mappings — both bare and node:-prefixed
    for builtin in polyfill_builtins {
        let path = format!("/node_modules/.jet/polyfill-{}.mjs", builtin);
        imports.insert(builtin.clone(), path.clone());
        imports.insert(format!("node:{}", builtin), path);
    }

    // MUI/Emotion patch table — overrides any auto-resolver entries that point
    // at broken CJS/ESM combinations for these well-known packages.
    // See jet#1908: react-is, dom-helpers, @mui/utils subpaths, @emotion/* roots
    // all need explicit aliases or browser/ESM entries to load under jet dev.
    for (specifier, entry) in mui_emotion_patches() {
        imports.insert(specifier.to_string(), entry.to_string());
    }

    // Sort keys for deterministic output
    let mut entries: Vec<_> = imports.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut json = String::from("{\n  \"imports\": {\n");
    for (i, (key, value)) in entries.iter().enumerate() {
        json.push_str(&format!("    \"{}\": \"{}\"", key, value));
        if i < entries.len() - 1 {
            json.push(',');
        }
        json.push('\n');
    }
    json.push_str("  }\n}");

    json
}

/// Inject an importmap `<script>` tag into HTML content.
///
/// If an existing `<script type="importmap">` is present, it is replaced.
/// Otherwise the importmap is inserted at the end of `<head>`.
///
/// This operation is idempotent — calling it multiple times with the same
/// importmap produces the same output.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn inject_importmap_html(html: &str, importmap_json: &str) -> String {
    let tag = format!("<script type=\"importmap\">\n{}\n</script>", importmap_json);

    // Check for existing importmap and replace it
    let lower = html.to_lowercase();
    if let Some(start) = lower.find("<script type=\"importmap\">") {
        if let Some(end) = lower[start..].find("</script>") {
            let end_pos = start + end + "</script>".len();
            let mut result = String::with_capacity(html.len());
            result.push_str(&html[..start]);
            result.push_str(&tag);
            result.push_str(&html[end_pos..]);
            return result;
        }
    }

    // Insert before </head>
    if let Some(pos) = lower.find("</head>") {
        let mut result = String::with_capacity(html.len() + tag.len() + 1);
        result.push_str(&html[..pos]);
        result.push_str(&tag);
        result.push('\n');
        result.push_str(&html[pos..]);
        return result;
    }

    // Insert after <head>
    if let Some(pos) = lower.find("<head>") {
        let insert_pos = pos + "<head>".len();
        let mut result = String::with_capacity(html.len() + tag.len() + 1);
        result.push_str(&html[..insert_pos]);
        result.push('\n');
        result.push_str(&tag);
        result.push_str(&html[insert_pos..]);
        return result;
    }

    // Fallback: prepend
    format!("{}\n{}", tag, html)
}

/// Return the static MUI/Emotion patch table.
///
/// Each entry maps a bare specifier to a known-good ESM URL. These overrides
/// fix runtime errors that ordinarily appear when an MUI/Emotion React app
/// runs under `jet dev` without manual `_importmap.json` patches:
///
/// - `react-is`, `prop-types`, `dom-helpers/*` — CJS packages with named
///   exports that ESM consumers (MUI, Emotion) destructure directly. They
///   must be resolved through the `.jet/` pre-bundle so the CJS→ESM wrapper
///   surfaces named exports rather than `default`-only.
/// - `@mui/utils/<helper>` — common helper subpaths (e.g. `deepmerge`,
///   `formatMuiErrorMessage`) that must use the package's ESM `index.mjs`
///   files so their relative `./helper.mjs` re-exports resolve correctly.
/// - `@emotion/*` roots — must point at the browser ESM entry (`dist/emotion-*.browser.esm.js`)
///   so consumers can import `default` and named exports without the CJS
///   "does not provide an export named default" error.
/// - `ReactPropTypes` — bare alias used by some legacy MUI CJS bundles;
///   aliased to the `prop-types` pre-bundle entry.
///
/// The list is intentionally narrow: we only patch packages that the issue
/// reproducer (`@mui/material` + `@mui/icons-material` + `@emotion/react`
/// + `@emotion/styled`) is known to trip over. Generic patch-DSL work is
/// out of scope here.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn mui_emotion_patches() -> &'static [(&'static str, &'static str)] {
    &[
        // CJS packages with named-export interop issues. Routing through the
        // pre-bundle wrapper exposes `default` plus all `exports.foo = ...`
        // names as ESM bindings.
        ("react-is", "/node_modules/.jet/react-is.mjs"),
        ("prop-types", "/node_modules/.jet/prop-types.mjs"),
        ("ReactPropTypes", "/node_modules/.jet/prop-types.mjs"),
        // dom-helpers subpaths used by transition-group / MUI internals.
        // The package publishes individual ESM files in `esm/`; map common
        // subpaths so bare specifiers like `dom-helpers/addClass` resolve.
        ("dom-helpers", "/node_modules/dom-helpers/esm/index.js"),
        (
            "dom-helpers/addClass",
            "/node_modules/dom-helpers/cjs/addClass.js",
        ),
        (
            "dom-helpers/removeClass",
            "/node_modules/dom-helpers/cjs/removeClass.js",
        ),
        (
            "dom-helpers/hasClass",
            "/node_modules/dom-helpers/cjs/hasClass.js",
        ),
        // @mui/utils helper subpaths. Use the published ESM entry; routing
        // these re-export files through `.jet/` breaks their relative imports.
        (
            "@mui/utils/deepmerge",
            "/node_modules/@mui/utils/deepmerge/index.mjs",
        ),
        (
            "@mui/utils/formatMuiErrorMessage",
            "/node_modules/@mui/utils/formatMuiErrorMessage/index.mjs",
        ),
        (
            "@mui/utils/capitalize",
            "/node_modules/@mui/utils/capitalize/index.mjs",
        ),
        (
            "@mui/utils/chainPropTypes",
            "/node_modules/@mui/utils/chainPropTypes/index.mjs",
        ),
        // Emotion root packages — the legacy `main`/`module` fields can point
        // at CJS-flavored bundles. Use the package's published ESM dist files.
        (
            "@emotion/react",
            "/node_modules/@emotion/react/dist/emotion-react.browser.esm.js",
        ),
        (
            "@emotion/styled",
            "/node_modules/@emotion/styled/dist/emotion-styled.browser.esm.js",
        ),
        (
            "@emotion/sheet",
            "/node_modules/@emotion/sheet/dist/emotion-sheet.esm.js",
        ),
        (
            "@emotion/cache",
            "/node_modules/@emotion/cache/dist/emotion-cache.browser.esm.js",
        ),
        (
            "@emotion/serialize",
            "/node_modules/@emotion/serialize/dist/emotion-serialize.esm.js",
        ),
        (
            "@emotion/utils",
            "/node_modules/@emotion/utils/dist/emotion-utils.browser.esm.js",
        ),
        ("@emotion/hash", "/node_modules/@emotion/hash/src/index.ts"),
        (
            "@emotion/use-insertion-effect-with-fallbacks",
            "/node_modules/@emotion/use-insertion-effect-with-fallbacks/dist/emotion-use-insertion-effect-with-fallbacks.esm.js",
        ),
        (
            "@emotion/unitless",
            "/node_modules/@emotion/unitless/dist/emotion-unitless.esm.js",
        ),
        (
            "@emotion/memoize",
            "/node_modules/@emotion/memoize/dist/emotion-memoize.esm.js",
        ),
        (
            "@emotion/is-prop-valid",
            "/node_modules/@emotion/is-prop-valid/dist/emotion-is-prop-valid.esm.js",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T9: Importmap Generated From Pre-Bundled Deps
    #[test]
    fn t09_importmap_generated_from_prebundled_deps() {
        let mut prebundled = HashMap::new();
        prebundled.insert("react".to_string(), "react.mjs".to_string());
        prebundled.insert("react-dom".to_string(), "react-dom.mjs".to_string());
        prebundled.insert("axios".to_string(), "axios.mjs".to_string());

        let json = build_importmap(&prebundled, &[]);
        assert!(json.contains("\"imports\""), "must have imports key");
        assert!(
            json.contains("\"react\": \"/node_modules/.jet/react.mjs\""),
            "must map react: {}",
            json
        );
        assert!(
            json.contains("\"react-dom\": \"/node_modules/.jet/react-dom.mjs\""),
            "must map react-dom: {}",
            json
        );
        assert!(
            json.contains("\"axios\": \"/node_modules/.jet/axios.mjs\""),
            "must map axios: {}",
            json
        );
    }

    /// T10: Importmap Injected Into HTML
    #[test]
    fn t10_importmap_injected_into_html() {
        let html = r#"<html><head><title>Test</title></head><body></body></html>"#;
        let importmap = r#"{ "imports": { "react": "/node_modules/.jet/react.mjs" } }"#;
        let result = inject_importmap_html(html, importmap);
        assert!(
            result.contains("<script type=\"importmap\">"),
            "must contain importmap tag: {}",
            result
        );
        assert!(
            result.contains(importmap),
            "must contain importmap JSON: {}",
            result
        );
        // Should be inside <head>
        let head_pos = result.find("<head>").unwrap();
        let importmap_pos = result.find("<script type=\"importmap\">").unwrap();
        let head_end = result.find("</head>").unwrap();
        assert!(
            importmap_pos > head_pos && importmap_pos < head_end,
            "importmap must be inside <head>"
        );
    }

    /// T11: Importmap Injection Idempotent
    #[test]
    fn t11_importmap_injection_idempotent() {
        let html = r#"<html><head><script type="importmap">
{ "imports": { "old": "/old.mjs" } }
</script></head><body></body></html>"#;
        let importmap = r#"{ "imports": { "react": "/node_modules/.jet/react.mjs" } }"#;
        let result = inject_importmap_html(html, importmap);

        // Count occurrences of importmap tag
        let count = result.matches("<script type=\"importmap\">").count();
        assert_eq!(
            count, 1,
            "must have exactly one importmap tag, got {}",
            count
        );
        // Must contain new importmap, not old
        assert!(
            result.contains("react"),
            "must contain new importmap: {}",
            result
        );
        assert!(
            !result.contains("\"old\""),
            "must not contain old importmap: {}",
            result
        );
    }

    /// T45: Polyfill Importmap Entries Generated
    #[test]
    fn t45_polyfill_importmap_entries() {
        let prebundled = HashMap::new();
        let polyfills = vec!["crypto".to_string(), "url".to_string()];
        let json = build_importmap(&prebundled, &polyfills);
        assert!(
            json.contains("\"crypto\": \"/node_modules/.jet/polyfill-crypto.mjs\""),
            "must map crypto: {}",
            json
        );
        assert!(
            json.contains("\"url\": \"/node_modules/.jet/polyfill-url.mjs\""),
            "must map url: {}",
            json
        );
    }

    /// T46: node: Prefix Mapped in Importmap
    #[test]
    fn t46_node_prefix_mapped_in_importmap() {
        let prebundled = HashMap::new();
        let polyfills = vec!["crypto".to_string()];
        let json = build_importmap(&prebundled, &polyfills);
        assert!(
            json.contains("\"crypto\": \"/node_modules/.jet/polyfill-crypto.mjs\""),
            "must map bare crypto: {}",
            json
        );
        assert!(
            json.contains("\"node:crypto\": \"/node_modules/.jet/polyfill-crypto.mjs\""),
            "must map node:crypto: {}",
            json
        );
    }

    /// jet#1908 — MUI/Emotion patch table covers every specifier from the
    /// observed-failures list in the issue. Without these aliases the dev
    /// server returns importmap entries that resolve to broken CJS bundles.
    #[test]
    fn mui_emotion_patches_cover_issue_1908_specifiers() {
        let json = build_importmap(&HashMap::new(), &[]);
        // CJS interop shims for ESM-named-export consumers
        assert!(
            json.contains("\"react-is\": \"/node_modules/.jet/react-is.mjs\""),
            "react-is must be aliased to .jet/ pre-bundle wrapper: {}",
            json
        );
        assert!(
            json.contains("\"prop-types\": \"/node_modules/.jet/prop-types.mjs\""),
            "prop-types must be aliased to .jet/ pre-bundle wrapper: {}",
            json
        );
        assert!(
            json.contains("\"ReactPropTypes\": \"/node_modules/.jet/prop-types.mjs\""),
            "legacy ReactPropTypes alias must point at prop-types: {}",
            json
        );
        // dom-helpers subpaths
        assert!(
            json.contains(
                "\"dom-helpers/addClass\": \"/node_modules/dom-helpers/cjs/addClass.js\""
            ),
            "dom-helpers/addClass subpath must resolve to installed CJS file for Jet wrapping: {}",
            json
        );
        // @mui/utils helper subpaths
        assert!(
            json.contains(
                "\"@mui/utils/deepmerge\": \"/node_modules/@mui/utils/deepmerge/index.mjs\""
            ),
            "@mui/utils/deepmerge must resolve to package ESM so relative helper imports work: {}",
            json
        );
        // Emotion roots and key subpackages
        assert!(
            json.contains("\"@emotion/react\": \"/node_modules/@emotion/react/dist/emotion-react.browser.esm.js\""),
            "@emotion/react must resolve to browser ESM entry: {}",
            json
        );
        assert!(
            json.contains("\"@emotion/styled\": \"/node_modules/@emotion/styled/dist/emotion-styled.browser.esm.js\""),
            "@emotion/styled must resolve to browser ESM entry: {}",
            json
        );
        assert!(
            json.contains("\"@emotion/sheet\": \"/node_modules/@emotion/sheet/dist/emotion-sheet.esm.js\""),
            "@emotion/sheet must resolve to the published ESM entry (fixes 'no default export' error): {}",
            json
        );
        assert!(
            json.contains("\"@emotion/hash\": \"/node_modules/@emotion/hash/src/index.ts\""),
            "@emotion/hash must resolve to the installed source entry when the package has no dist files: {}",
            json
        );
        assert!(
            json.contains("\"@emotion/use-insertion-effect-with-fallbacks\": \"/node_modules/@emotion/use-insertion-effect-with-fallbacks/dist/emotion-use-insertion-effect-with-fallbacks.esm.js\""),
            "@emotion/use-insertion-effect-with-fallbacks must resolve to Jet's virtual hook fallback when the store package is empty: {}",
            json
        );
    }

    /// jet#1908 — MUI/Emotion patch overrides any conflicting auto-resolver
    /// entry. The patch table runs last so it wins over `prebundled`/`esm_deps`.
    #[test]
    fn mui_emotion_patch_overrides_auto_resolved_entry() {
        let mut esm_deps = HashMap::new();
        // Simulate the scan_esm_deps default: @emotion/sheet pointed at the
        // CJS main entry, which is the bug we're fixing.
        esm_deps.insert(
            "@emotion/sheet".to_string(),
            "@emotion/sheet/dist/emotion-sheet.cjs.dev.js".to_string(),
        );

        let json = build_importmap_full(&HashMap::new(), &esm_deps, &[]);
        assert!(
            json.contains(
                "\"@emotion/sheet\": \"/node_modules/@emotion/sheet/dist/emotion-sheet.esm.js\""
            ),
            "patch table must override the auto-resolved CJS entry: {}",
            json
        );
        assert!(
            !json.contains("emotion-sheet.cjs.dev.js"),
            "stale CJS entry must not appear in the rendered importmap: {}",
            json
        );
    }

    /// jet#1908 — Patches must not produce duplicate `prop-types` keys when
    /// the consumer already pre-bundled it under the same name.
    #[test]
    fn mui_emotion_patch_dedupes_with_prebundled_entry() {
        let mut prebundled = HashMap::new();
        prebundled.insert("prop-types".to_string(), "prop-types.mjs".to_string());

        let json = build_importmap(&prebundled, &[]);
        // The key appears exactly once in the rendered JSON
        let occurrences = json.matches("\"prop-types\":").count();
        assert_eq!(
            occurrences, 1,
            "prop-types must appear exactly once after patch merge: {}",
            json
        );
    }
}
// CODEGEN-END
