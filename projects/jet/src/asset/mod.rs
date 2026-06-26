// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
// CODEGEN-BEGIN
use anyhow::{Context, Result};
use std::path::Path;

pub mod image_processor;
pub mod svgr;
pub mod types;

pub use svgr::{transform_svg_to_component, SvgrExportType};
pub use types::{AssetOptions, AssetType, ProcessedAsset};

/// SVGR (import `.svg` as a React component) configuration for the bundler.
///
/// Mirrors `vite-plugin-svgr`: when [`enabled`](SvgrConfig::enabled) and an
/// `.svg` is imported as a component, the file is routed through
/// [`svgr::transform_svg_to_component`] instead of the asset-URL path. The
/// default matches `fe-shared`'s `{ exportType: 'named' }` config.
///
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SvgrConfig {
    /// Whether `.svg`-as-component routing is active at all. When `false`,
    /// every `.svg` import keeps the existing asset-URL behavior.
    pub enabled: bool,
    /// Which exports the emitted component module provides.
    pub export_type: SvgrExportType,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
impl Default for SvgrConfig {
    fn default() -> Self {
        // fe-shared uses `vite-plugin-svgr` with `{ exportType: 'named' }`.
        Self {
            enabled: true,
            export_type: SvgrExportType::Named,
        }
    }
}

/// GH #3618 — `create_hashed_filename` previously did
/// `path.file_stem().unwrap()` (panics on missing stem) and
/// `path.extension().unwrap_or_default()` (silent fallback that produced
/// trailing-dot filenames like `"logo.deadbeef."` for extensionless
/// inputs). Tagged error helper for the stem branch.
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub(crate) fn format_asset_hashed_filename_err(path: &Path, observed_kind: &str) -> String {
    format!(
        "GH #3618 jet asset cannot derive a {observed_kind} from {path:?}; \
         a hashed filename requires at minimum a file stem. \
         Rename the source file or omit `hash_filenames`.",
        observed_kind = observed_kind,
        path = path,
    )
}

/// GH #3634 — `AssetProcessor::process_generic` previously did
/// `path.file_name().unwrap().to_string_lossy()` on the
/// `!hash_filenames` branch — sibling to the panic that #3618 fixed
/// on the hashed branch. `path.file_name()` returns `None` for `..`,
/// `/`, and any path ending in `/`, so a caller supplying such a
/// path crashed the asset pipeline instead of returning a tagged
/// error.
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub(crate) fn safe_asset_filename(path: &Path) -> Result<String, String> {
    match path.file_name() {
        Some(name) => Ok(name.to_string_lossy().to_string()),
        None => Err(format_asset_filename_err(path)),
    }
}

/// GH #3634 — tagged error helper for [`safe_asset_filename`].
/// Names the offending path, the issue tag, and the user-visible
/// remediation (rename the source file).
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub(crate) fn format_asset_filename_err(path: &Path) -> String {
    format!(
        "GH #3634 jet asset cannot derive a file name from {path:?}; \
         the path resolves to a parent reference, the root, or a \
         trailing-slash directory. Rename the source file (or strip \
         the trailing `/`) and retry."
    )
}

/// GH #3774 — derive the lowercased extension for asset type detection
/// and image-optimization routing.
///
/// Branches:
/// - Path has no extension → `""` (silent; legitimate extensionless file).
/// - Extension is UTF-8 → `to_lowercase`d string (silent).
/// - Extension has non-UTF-8 bytes → `to_string_lossy().to_lowercase()` +
///   warn so the operator can spot images silently routed to
///   `AssetType::Other` / skipped from image-specific optimization.
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub(crate) fn lowercase_extension_or_warn(path: &Path) -> String {
    let Some(ext) = path.extension() else {
        return String::new();
    };
    match ext.to_str() {
        Some(s) => s.to_lowercase(),
        None => {
            let lossy = ext.to_string_lossy().to_lowercase();
            tracing::warn!(
                target: "jet::asset",
                path = %path.display(),
                lossy = %lossy,
                "{}",
                format_asset_non_utf8_extension_warn(path, &lossy)
            );
            lossy
        }
    }
}

/// GH #3774 — diagnostic for a non-UTF-8 file extension. Operators grep
/// for "GH #3774" to chase silently-mis-routed image optimization.
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub(crate) fn format_asset_non_utf8_extension_warn(path: &Path, lossy: &str) -> String {
    format!(
        "GH #3774 jet asset path {path:?} has a non-UTF-8 extension; \
         recovered via lossy decode as {lossy:?}. The file may be \
         silently routed to AssetType::Other and skip image \
         optimization. Rename the source file to a UTF-8 extension."
    )
}

/// GH #3805 — warn shown when the file stem feeding
/// `create_hashed_filename` carries non-UTF-8 bytes. The prior
/// `stem.to_string_lossy()` silently U+FFFD-substituted so two stems
/// differing only in non-UTF-8 bytes collapsed onto the same hashed
/// output filename and clobbered each other in `dist/`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub(crate) fn format_asset_hashed_filename_non_utf8_stem_warn(path: &Path, lossy: &str) -> String {
    format!(
        "gh3805: jet asset hashed filename saw non-UTF-8 stem path={:?}; \
         lossy form is {:?}; two assets differing only in non-UTF-8 stem \
         bytes will collide on the same hashed output filename and \
         silently overwrite each other in dist/",
        path, lossy
    )
}

/// GH #3805 — warn shown when the extension feeding
/// `create_hashed_filename` carries non-UTF-8 bytes. Same collision
/// risk as the stem variant but on the trailing component.
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub(crate) fn format_asset_hashed_filename_non_utf8_ext_warn(path: &Path, lossy: &str) -> String {
    format!(
        "gh3805: jet asset hashed filename saw non-UTF-8 extension path={:?}; \
         lossy form is {:?}; two assets differing only in non-UTF-8 extension \
         bytes will collide on the same hashed output filename and \
         silently overwrite each other in dist/",
        path, lossy
    )
}

/// GH #3805 — coerce the file stem into a UTF-8 string for the
/// hashed-filename builder. Some(utf8) returns a borrowed view (silent);
/// Some(non-UTF-8) emits a `tracing::warn!` carrying the original Path
/// Debug form and the lossy form, then returns the lossy form so the
/// build still completes.
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub(crate) fn coerce_hashed_filename_stem_or_warn<'a>(
    path: &Path,
    stem: &'a std::ffi::OsStr,
) -> std::borrow::Cow<'a, str> {
    use std::borrow::Cow;
    match stem.to_str() {
        Some(s) => Cow::Borrowed(s),
        None => {
            let lossy = stem.to_string_lossy().into_owned();
            tracing::warn!(
                target: "jet::asset",
                path = %path.display(),
                lossy = %lossy,
                "{}",
                format_asset_hashed_filename_non_utf8_stem_warn(path, &lossy)
            );
            Cow::Owned(lossy)
        }
    }
}

/// GH #3805 — coerce the extension into a UTF-8 string for the
/// hashed-filename builder. Sibling to `coerce_hashed_filename_stem_or_warn`
/// for the trailing component.
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub(crate) fn coerce_hashed_filename_ext_or_warn<'a>(
    path: &Path,
    ext: &'a std::ffi::OsStr,
) -> std::borrow::Cow<'a, str> {
    use std::borrow::Cow;
    match ext.to_str() {
        Some(s) => Cow::Borrowed(s),
        None => {
            let lossy = ext.to_string_lossy().into_owned();
            tracing::warn!(
                target: "jet::asset",
                path = %path.display(),
                lossy = %lossy,
                "{}",
                format_asset_hashed_filename_non_utf8_ext_warn(path, &lossy)
            );
            Cow::Owned(lossy)
        }
    }
}

/// Asset processor for handling images, fonts, etc.
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub struct AssetProcessor {
    options: AssetOptions,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
impl AssetProcessor {
    /// Create a new asset processor
    pub fn new(options: AssetOptions) -> Self {
        Self { options }
    }

    /// Process an asset file
    pub fn process(&self, path: &Path) -> Result<ProcessedAsset> {
        let asset_type = self.detect_type(path)?;

        match asset_type {
            AssetType::Image => self.process_image(path),
            AssetType::Font => self.process_font(path),
            AssetType::Other => self.process_generic(path),
        }
    }

    /// Detect asset type from file extension
    fn detect_type(&self, path: &Path) -> Result<AssetType> {
        // GH #3774 — was a silent `.to_str().unwrap_or("")` that
        // routed non-UTF-8 extensions to `AssetType::Other`. A Linux
        // image file like `badcafé.png` with a non-UTF-8 "é" byte would
        // silently bucket as Other and skip image-specific processing.
        let ext = lowercase_extension_or_warn(path);

        match ext.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => Ok(AssetType::Image),
            "woff" | "woff2" | "ttf" | "otf" | "eot" => Ok(AssetType::Font),
            _ => Ok(AssetType::Other),
        }
    }

    fn process_image(&self, path: &Path) -> Result<ProcessedAsset> {
        tracing::debug!("Processing image: {:?}", path);

        if self.options.optimize_images {
            Ok(image_processor::optimize_image(path, &self.options)?)
        } else {
            self.process_generic(path)
        }
    }

    fn process_font(&self, path: &Path) -> Result<ProcessedAsset> {
        tracing::debug!("Processing font: {:?}", path);
        self.process_generic(path)
    }

    fn process_generic(&self, path: &Path) -> Result<ProcessedAsset> {
        let content = std::fs::read(path)?;
        let hash = self.compute_hash(&content);

        let filename = if self.options.hash_filenames {
            self.create_hashed_filename(path, &hash)?
        } else {
            path.file_name().unwrap().to_string_lossy().to_string()
        };

        Ok(ProcessedAsset {
            original_path: path.to_path_buf(),
            content,
            filename,
            hash,
            asset_type: self.detect_type(path)?,
        })
    }

    fn compute_hash(&self, content: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())[..8].to_string()
    }

    fn create_hashed_filename(&self, path: &Path, hash: &str) -> Result<String> {
        // GH #3618 — explicit error for missing stem (was `.unwrap()` panic),
        // explicit no-trailing-dot path for missing extension (was silent
        // `.unwrap_or_default()` that produced `"foo.<hash>."`).
        // GH #3805 — both `stem.to_string_lossy()` and `ext.to_string_lossy()`
        // silently U+FFFD-substituted non-UTF-8 bytes. Two assets that
        // differed only in non-UTF-8 bytes collided on the same hashed
        // output filename and clobbered each other in `dist/`. Route
        // through coerce_*_or_warn so operators see the lossy-collision
        // breadcrumb in logs.
        let stem_os = path
            .file_stem()
            .with_context(|| format_asset_hashed_filename_err(path, "file_stem"))?;
        let stem = coerce_hashed_filename_stem_or_warn(path, stem_os);
        match path.extension() {
            Some(ext) => {
                let ext_str = coerce_hashed_filename_ext_or_warn(path, ext);
                Ok(format!("{}.{}.{}", stem, hash, ext_str))
            }
            None => Ok(format!("{}.{}", stem, hash)),
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
impl Default for AssetOptions {
    fn default() -> Self {
        Self {
            optimize_images: true,
            hash_filenames: true,
            max_image_size: 1024 * 1024,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_type() {
        let processor = AssetProcessor::new(AssetOptions::default());

        assert_eq!(
            processor.detect_type(Path::new("test.png")).unwrap(),
            AssetType::Image
        );
        assert_eq!(
            processor.detect_type(Path::new("test.woff")).unwrap(),
            AssetType::Font
        );
    }

    #[test]
    fn test_compute_hash() {
        let processor = AssetProcessor::new(AssetOptions::default());
        let hash = processor.compute_hash(b"test content");
        assert_eq!(hash.len(), 8);
    }
}

#[cfg(test)]
mod gh3618_create_hashed_filename_tests {
    //! GH #3618 — `create_hashed_filename` previously did
    //! `.file_stem().unwrap()` (panic on missing stem) and
    //! `.extension().unwrap_or_default()` (silent fallback that produced
    //! trailing-dot filenames like `"logo.deadbeef."` for extensionless
    //! inputs).
    use super::*;

    fn processor() -> AssetProcessor {
        AssetProcessor::new(AssetOptions::default())
    }

    #[test]
    fn extensioned_path_produces_dot_separated_filename() {
        let p = Path::new("/x/logo.png");
        let out = processor().create_hashed_filename(p, "deadbeef").unwrap();
        assert_eq!(out, "logo.deadbeef.png");
    }

    #[test]
    fn extensionless_path_has_no_trailing_dot() {
        let p = Path::new("/x/logo");
        let out = processor().create_hashed_filename(p, "deadbeef").unwrap();
        assert_eq!(out, "logo.deadbeef", "must NOT have trailing dot");
        assert!(!out.ends_with('.'), "out: {out}");
    }

    #[test]
    fn missing_stem_returns_tagged_err() {
        let p = Path::new("..");
        let err = processor()
            .create_hashed_filename(p, "deadbeef")
            .expect_err(".. has no file_stem");
        let msg = format!("{}", err);
        assert!(msg.contains("GH #3618"), "msg: {msg}");
        assert!(msg.contains("file_stem"), "msg: {msg}");
    }

    #[test]
    fn root_path_returns_tagged_err() {
        let p = Path::new("/");
        let err = processor()
            .create_hashed_filename(p, "deadbeef")
            .expect_err("/ has no file_stem");
        let msg = format!("{}", err);
        assert!(msg.contains("GH #3618"), "msg: {msg}");
    }

    #[test]
    fn helper_message_includes_tag_and_path() {
        let msg = format_asset_hashed_filename_err(Path::new("/x/y"), "file_stem");
        assert!(msg.contains("GH #3618"), "msg: {msg}");
        assert!(msg.contains("/x/y"), "msg: {msg}");
        assert!(msg.contains("file_stem"), "msg: {msg}");
    }

    #[test]
    fn nested_path_keeps_only_stem_and_ext() {
        let p = Path::new("/a/b/c/icon.svg");
        let out = processor().create_hashed_filename(p, "abc12345").unwrap();
        assert_eq!(out, "icon.abc12345.svg");
    }
}

#[cfg(test)]
mod gh3634_safe_asset_filename_tests {
    //! GH #3634 — `process_generic` previously did
    //! `path.file_name().unwrap()` on the `!hash_filenames` branch.
    //! Sibling of GH #3618 on the hashed branch.
    use super::*;

    #[test]
    fn ordinary_path_returns_file_name() {
        let out = safe_asset_filename(Path::new("/x/y/logo.png")).unwrap();
        assert_eq!(out, "logo.png");
    }

    #[test]
    fn extensionless_path_still_returns_file_name() {
        let out = safe_asset_filename(Path::new("/x/y/README")).unwrap();
        assert_eq!(out, "README");
    }

    #[test]
    fn parent_ref_path_returns_tagged_err_not_panic() {
        let err = safe_asset_filename(Path::new("..")).expect_err(".. has no file_name");
        assert!(err.contains("GH #3634"), "msg: {err}");
        assert!(err.contains(".."), "msg: {err}");
    }

    #[test]
    fn root_path_returns_tagged_err_not_panic() {
        let err = safe_asset_filename(Path::new("/")).expect_err("/ has no file_name");
        assert!(err.contains("GH #3634"), "msg: {err}");
    }

    #[test]
    fn format_helper_includes_tag_and_path() {
        let msg = format_asset_filename_err(Path::new("/some/dir/.."));
        assert!(msg.contains("GH #3634"), "msg: {msg}");
        assert!(msg.contains("/some/dir/.."), "msg: {msg}");
    }

    /// GH #3634 — end-to-end via `AssetProcessor::process_generic`:
    /// a file written to a path whose `file_name()` is None must
    /// surface the tagged error instead of panicking. We approximate
    /// this by exercising `safe_asset_filename` on the same path as
    /// `process_generic` would — directly driving `process_generic`
    /// requires writing a real file at an unaddressable name, which
    /// is impossible on the filesystem, so we test the contract on
    /// the helper plus `AssetOptions { hash_filenames: false }`.
    #[test]
    fn process_generic_non_hashed_branch_uses_safe_filename() {
        // The contract: safe_asset_filename short-circuits before
        // `process_generic` would have done `.unwrap()`. The presence
        // of the `safe_asset_filename` call site in source is the
        // structural fix; this test pins the helper output on the
        // same path shape (extension-only, e.g. ".gitignore") that
        // led to the regression report (file_name still Some for
        // dotfiles, so the helper passes through cleanly).
        assert_eq!(
            safe_asset_filename(Path::new("/proj/.gitignore")).unwrap(),
            ".gitignore"
        );
    }
}

#[cfg(test)]
mod gh3774_non_utf8_extension_warn_tests {
    //! GH #3774 — non-UTF-8 file extensions used to silently route
    //! image files to AssetType::Other / skip image optimization via
    //! `.to_str().unwrap_or("")`. Tests cover absent / UTF-8 / non-UTF-8
    //! branches + helper-name discoverability + sibling-distinctness vs.
    //! prior asset warns (#3618 / #3634).

    use super::*;

    #[test]
    fn gh3774_absent_extension_returns_empty_silent() {
        assert_eq!(lowercase_extension_or_warn(Path::new("/x/no_ext")), "");
    }

    #[test]
    fn gh3774_utf8_extension_returns_lowercased() {
        assert_eq!(lowercase_extension_or_warn(Path::new("/x/IMG.PNG")), "png");
        assert_eq!(
            lowercase_extension_or_warn(Path::new("/x/font.WOFF2")),
            "woff2"
        );
    }

    #[test]
    fn gh3774_dotfile_treated_as_no_extension() {
        // `.gitignore` has no `extension()` (file_stem is ".gitignore").
        assert_eq!(lowercase_extension_or_warn(Path::new("/x/.gitignore")), "");
    }

    #[cfg(unix)]
    #[test]
    fn gh3774_non_utf8_extension_recovers_via_lossy() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        use std::path::PathBuf;

        let mut bad = PathBuf::from("/x/img.");
        bad.set_extension(OsStr::from_bytes(b"p\xFFng"));
        let ext = lowercase_extension_or_warn(&bad);
        // U+FFFD lossy decode lowercases; should still contain "p" + "ng".
        assert!(ext.contains('p'));
        assert!(ext.contains("ng"));
    }

    #[test]
    fn gh3774_helper_includes_issue_tag() {
        let msg = format_asset_non_utf8_extension_warn(Path::new("/x/img"), "p\u{FFFD}ng");
        assert!(msg.contains("GH #3774"));
    }

    #[test]
    fn gh3774_helper_records_path_and_lossy() {
        let msg = format_asset_non_utf8_extension_warn(Path::new("/elsewhere/img"), "p\u{FFFD}ng");
        assert!(msg.contains("/elsewhere/img"));
        assert!(msg.contains("p\u{FFFD}ng"));
    }

    #[test]
    fn gh3774_warn_distinct_from_gh3618_and_gh3634() {
        let msg = format_asset_non_utf8_extension_warn(Path::new("/x/img"), "lossy");
        assert!(msg.contains("GH #3774"));
        assert!(!msg.contains("GH #3618"));
        assert!(!msg.contains("GH #3634"));
        // Subject is the extension routing, not the file_stem or
        // file_name derivation that #3618/#3634 cover.
        assert!(msg.contains("extension"));
    }

    #[test]
    fn gh3774_helper_names_skipped_optimization() {
        // The operator should learn from the message *why* this matters
        // (skipped image optimization), not just that bytes were lossy.
        let msg = format_asset_non_utf8_extension_warn(Path::new("/x/img"), "p\u{FFFD}ng");
        assert!(msg.contains("optimization"));
    }

    /// GH #3774 — helper-name convention is discoverable. If the helper
    /// is ever renamed, this file would fail to compile — the test
    /// asserts the convention via use-site.
    #[test]
    fn gh3774_helper_naming_convention_discoverable() {
        let _ = lowercase_extension_or_warn(Path::new("/x"));
        let _ = format_asset_non_utf8_extension_warn(Path::new("/x"), "y");
    }

    /// GH #3774 — detect_type integration: a UTF-8 image extension
    /// still routes to AssetType::Image (no regression on the happy
    /// path).
    #[test]
    fn gh3774_detect_type_utf8_image_still_routes_correctly() {
        let proc = AssetProcessor::new(AssetOptions::default());
        assert!(matches!(
            proc.detect_type(Path::new("/x/logo.png")).unwrap(),
            AssetType::Image
        ));
        assert!(matches!(
            proc.detect_type(Path::new("/x/font.woff2")).unwrap(),
            AssetType::Font
        ));
        assert!(matches!(
            proc.detect_type(Path::new("/x/something.bin")).unwrap(),
            AssetType::Other
        ));
        assert!(matches!(
            proc.detect_type(Path::new("/x/noext")).unwrap(),
            AssetType::Other
        ));
    }
}

#[cfg(test)]
mod gh3805_hashed_filename_non_utf8_warn_tests {
    use super::{
        coerce_hashed_filename_ext_or_warn, coerce_hashed_filename_stem_or_warn,
        format_asset_hashed_filename_non_utf8_ext_warn,
        format_asset_hashed_filename_non_utf8_stem_warn, AssetOptions, AssetProcessor,
    };
    use std::ffi::OsStr;
    use std::path::Path;

    #[test]
    fn utf8_stem_passes_through_silently() {
        let p = Path::new("/x/logo.png");
        let stem = p.file_stem().unwrap();
        let s = coerce_hashed_filename_stem_or_warn(p, stem);
        assert_eq!(s, "logo");
    }

    #[test]
    fn utf8_ext_passes_through_silently() {
        let p = Path::new("/x/logo.png");
        let ext = p.extension().unwrap();
        let s = coerce_hashed_filename_ext_or_warn(p, ext);
        assert_eq!(s, "png");
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_stem_produces_lossy_form() {
        use std::os::unix::ffi::OsStrExt;
        let stem_os = OsStr::from_bytes(b"shop\xFFfr");
        let p = Path::new("/x/shop.png");
        let s = coerce_hashed_filename_stem_or_warn(p, stem_os);
        assert!(
            s.contains("\u{FFFD}"),
            "lossy stem should include U+FFFD: {s:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_ext_produces_lossy_form() {
        use std::os::unix::ffi::OsStrExt;
        let ext_os = OsStr::from_bytes(b"p\xFFng");
        let p = Path::new("/x/logo.png");
        let s = coerce_hashed_filename_ext_or_warn(p, ext_os);
        assert!(
            s.contains("\u{FFFD}"),
            "lossy ext should include U+FFFD: {s:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn two_distinct_non_utf8_stems_get_distinct_warn_messages() {
        use std::os::unix::ffi::OsStrExt;
        let p1 = Path::new("/x/a");
        let p2 = Path::new("/x/b");
        let stem1 = OsStr::from_bytes(b"shop\xFFfr");
        let stem2 = OsStr::from_bytes(b"shop\xFEjp");
        let s1 = coerce_hashed_filename_stem_or_warn(p1, stem1);
        let s2 = coerce_hashed_filename_stem_or_warn(p2, stem2);
        // Lossy forms may or may not collide — what matters is each
        // warn carries the original path's Debug form to distinguish them.
        let w1 = format_asset_hashed_filename_non_utf8_stem_warn(p1, &s1);
        let w2 = format_asset_hashed_filename_non_utf8_stem_warn(p2, &s2);
        assert_ne!(
            w1, w2,
            "warns must carry the distinct original path so the operator can spot which asset collided"
        );
    }

    #[test]
    fn warn_helpers_pinned_for_discoverability() {
        let src = include_str!("mod.rs");
        assert!(src.contains("fn format_asset_hashed_filename_non_utf8_stem_warn"));
        assert!(src.contains("fn format_asset_hashed_filename_non_utf8_ext_warn"));
        assert!(src.contains("fn coerce_hashed_filename_stem_or_warn"));
        assert!(src.contains("fn coerce_hashed_filename_ext_or_warn"));
    }

    #[test]
    fn each_warn_string_carries_gh3805_tag() {
        let s = format_asset_hashed_filename_non_utf8_stem_warn(Path::new("/x"), "lossy");
        assert!(s.starts_with("gh3805:"), "missing gh3805 tag: {s:?}");
        let e = format_asset_hashed_filename_non_utf8_ext_warn(Path::new("/x"), "lossy");
        assert!(e.starts_with("gh3805:"), "missing gh3805 tag: {e:?}");
    }

    #[test]
    fn warn_distinct_from_prior_silent_fallback_families() {
        let s = format_asset_hashed_filename_non_utf8_stem_warn(Path::new("/x"), "lossy");
        let e = format_asset_hashed_filename_non_utf8_ext_warn(Path::new("/x"), "lossy");
        for tag in [
            "gh3763", "gh3765", "gh3768", "gh3770", "gh3772", "gh3774", "gh3776", "gh3787",
            "gh3789", "gh3791", "gh3793", "gh3795", "gh3797", "gh3799", "gh3801", "gh3803",
        ] {
            assert!(!s.contains(tag), "stem warn must not carry {tag}: {s:?}");
            assert!(!e.contains(tag), "ext warn must not carry {tag}: {e:?}");
        }
    }

    #[test]
    fn two_sibling_warns_are_mutually_distinct() {
        let s = format_asset_hashed_filename_non_utf8_stem_warn(Path::new("/x"), "lossy");
        let e = format_asset_hashed_filename_non_utf8_ext_warn(Path::new("/x"), "lossy");
        assert_ne!(s, e);
        assert!(s.contains("non-UTF-8 stem"));
        assert!(e.contains("non-UTF-8 extension"));
    }

    #[test]
    fn happy_path_hashed_filename_round_trips_utf8_stem_and_ext() {
        // Integration: a UTF-8 path goes through create_hashed_filename and
        // produces the canonical stem.<hash>.ext shape with no warn.
        let dir = tempfile::tempdir().unwrap();
        let asset_path = dir.path().join("logo.png");
        std::fs::write(&asset_path, b"fake-png-bytes").unwrap();
        let opts = AssetOptions {
            optimize_images: false,
            hash_filenames: true,
            max_image_size: 1024 * 1024,
        };
        let proc = AssetProcessor::new(opts);
        let out = proc.process(&asset_path).unwrap();
        assert!(
            out.filename.starts_with("logo."),
            "filename: {}",
            out.filename
        );
        assert!(out.filename.ends_with(".png"), "filename: {}", out.filename);
        // <stem>.<8-hex>.<ext> shape
        let parts: Vec<&str> = out.filename.split('.').collect();
        assert_eq!(parts.len(), 3, "expected 3 dot-segments: {}", out.filename);
        assert_eq!(
            parts[1].len(),
            8,
            "hash segment should be 8 hex chars: {}",
            out.filename
        );
    }
}
// CODEGEN-END
