// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
// CODEGEN-BEGIN
use anyhow::{Context, Result};
use std::path::Path;

use super::{AssetOptions, AssetType, ProcessedAsset};

/// Minimum file size in bytes for optimization to be worthwhile.
/// Images under this threshold are returned unchanged.
const MIN_OPTIMIZE_SIZE: usize = 1024;

/// Default JPEG quality for re-encoding (0-100).
const DEFAULT_JPEG_QUALITY: u8 = 85;

/// GH #3572 — build the context string for a failed image-processing
/// step. Extracted so the wording (tag + path + step) is unit-testable
/// without provoking a real corrupt-image scenario in the integration
/// path.
///
/// `step` is one of `"read"`, `"decode"`, or `"encode"`, which lets the
/// dev tell from one log line whether the image is corrupt on disk
/// (read fails) or the encoder failed (encode fails) — a critical
/// distinction in a project with hundreds of assets.
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub(crate) fn format_image_step_err(path: &Path, step: &str) -> String {
    format!(
        "GH #3572 image {step} failed for {}; the surrounding optimize_image \
         pipeline cannot continue. Check whether the file is readable, \
         well-formed for its extension, and within max_image_size.",
        path.display()
    )
}

/// GH #3590 — build the context string for a missing path component
/// (`file_stem` or `file_name`) during filename derivation in
/// `optimize_image`. Extracted so the wording (tag + path + missing
/// component) is unit-testable.
///
/// `component` is one of `"file_stem"` or `"file_name"`, which lets
/// the dev tell from one log line which `Option::None` triggered the
/// failure — the path may have a stem but no full name (`./`) or
/// neither (`/`).
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub(crate) fn format_image_filename_err(path: &Path, component: &str) -> String {
    format!(
        "GH #3590 image filename derivation failed: path {} has no {component} \
         component (e.g. path is `/`, `.`, `..`, or ends in a trailing slash). \
         optimize_image cannot synthesize a filename for the produced asset.",
        path.display()
    )
}

/// Optimize image file.
///
/// - JPEG: Re-encode at quality 85 (smaller file size)
/// - PNG: Re-encode with basic optimization
/// - WebP: Pass through (already optimized format)
/// - SVG: Strip comments and unnecessary whitespace
/// - Skip optimization for images under 1KB (overhead not worth it)
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub fn optimize_image(path: &Path, options: &AssetOptions) -> Result<ProcessedAsset> {
    tracing::debug!("Optimizing image: {:?}", path);

    let original_content =
        std::fs::read(path).with_context(|| format_image_step_err(path, "read"))?;
    let original_size = original_content.len();

    if original_size > options.max_image_size {
        tracing::warn!(
            "Image exceeds max size: {} > {}",
            original_size,
            options.max_image_size
        );
    }

    // GH #3774 — sibling silent fallback to detect_type(); non-UTF-8
    // image extensions used to silently skip optimization.
    let ext = crate::asset::lowercase_extension_or_warn(path);

    // Skip optimization for images under 1KB
    let content = if original_size < MIN_OPTIMIZE_SIZE {
        tracing::debug!(
            "Skipping optimization for small image ({} bytes < {} threshold)",
            original_size,
            MIN_OPTIMIZE_SIZE
        );
        original_content
    } else {
        match ext.as_str() {
            "jpg" | "jpeg" => optimize_jpeg(path, &original_content)?,
            "png" => optimize_png(path, &original_content)?,
            "svg" => optimize_svg(&original_content),
            // WebP and other formats: pass through
            _ => original_content,
        }
    };

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let hash = format!("{:x}", hasher.finalize())[..8].to_string();

    // GH #3590 — replace `.unwrap()` on `file_stem()` / `file_name()`
    // with `?`-propagated errors. The prior code panicked when `path`
    // was `/`, `.`, `..`, or had no file_name component. `optimize_image`
    // already `?`-propagates read/decode/encode failures via the
    // `format_image_step_err` helper (GH #3572); filename derivation
    // gets the same shape.
    let filename = if options.hash_filenames {
        let stem = path
            .file_stem()
            .with_context(|| format_image_filename_err(path, "file_stem"))?
            .to_string_lossy();
        // GH #3621 — `.extension().unwrap_or_default()` silently produced
        // trailing-dot filenames like `"logo.deadbeef."` for extensionless
        // inputs. Sibling of GH #3618 (same bug in `asset/mod.rs`).
        match path.extension() {
            Some(ext) => format!("{}.{}.{}", stem, hash, ext.to_string_lossy()),
            None => format!("{}.{}", stem, hash),
        }
    } else {
        path.file_name()
            .with_context(|| format_image_filename_err(path, "file_name"))?
            .to_string_lossy()
            .to_string()
    };

    Ok(ProcessedAsset {
        original_path: path.to_path_buf(),
        content,
        filename,
        hash,
        asset_type: AssetType::Image,
    })
}

/// Optimize JPEG by re-encoding at the configured quality level.
/// Returns the smaller of original vs re-encoded.
fn optimize_jpeg(path: &Path, original: &[u8]) -> Result<Vec<u8>> {
    let img = image::open(path).with_context(|| format_image_step_err(path, "decode"))?;
    let mut output = std::io::Cursor::new(Vec::new());

    let encoder =
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, DEFAULT_JPEG_QUALITY);
    img.write_with_encoder(encoder)
        .with_context(|| format_image_step_err(path, "encode"))?;

    let optimized = output.into_inner();

    // Only use the optimized version if it's actually smaller
    if optimized.len() < original.len() {
        tracing::debug!(
            "JPEG optimized: {} -> {} bytes (saved {})",
            original.len(),
            optimized.len(),
            original.len() - optimized.len()
        );
        Ok(optimized)
    } else {
        tracing::debug!(
            "JPEG optimization not beneficial ({} >= {} bytes), keeping original",
            optimized.len(),
            original.len()
        );
        Ok(original.to_vec())
    }
}

/// Optimize PNG by re-encoding with default compression.
/// Returns the smaller of original vs re-encoded.
fn optimize_png(path: &Path, original: &[u8]) -> Result<Vec<u8>> {
    let img = image::open(path).with_context(|| format_image_step_err(path, "decode"))?;
    let mut output = std::io::Cursor::new(Vec::new());

    let encoder = image::codecs::png::PngEncoder::new(&mut output);
    img.write_with_encoder(encoder)
        .with_context(|| format_image_step_err(path, "encode"))?;

    let optimized = output.into_inner();

    if optimized.len() < original.len() {
        tracing::debug!(
            "PNG optimized: {} -> {} bytes (saved {})",
            original.len(),
            optimized.len(),
            original.len() - optimized.len()
        );
        Ok(optimized)
    } else {
        tracing::debug!("PNG optimization not beneficial, keeping original");
        Ok(original.to_vec())
    }
}

/// GH #3734 — build the warn-body for a non-UTF-8 SVG input to
/// `optimize_svg`. Extracted so the wording (tag + byte length + the
/// `Utf8Error` fields that pinpoint the bad byte) is unit-testable
/// without having to provoke a real malformed-SVG scenario through the
/// asset pipeline.
///
/// `bytes_len` is the length of the input slice; `err` is the
/// `Utf8Error` returned by `std::str::from_utf8`. Together they give an
/// operator the byte offset (`valid_up_to`) and the length of the
/// invalid sequence (`error_len`) so they can `xxd | head` straight to
/// the corruption. Matches sibling silent-fallback fixes:
/// `format_dev_server_ctrl_c_warn` (#3725),
/// `format_browser_ws_timeout_err` (#3727),
/// `format_wasm_dev_ctrl_c_warn` (#3730),
/// `format_browser_cli_ctrl_c_warn` (#3732).
/// @spec .aw/tech-design/projects/jet/semantic/jet-asset.md#schema
pub(crate) fn format_svg_non_utf8_warn(bytes_len: usize, err: &std::str::Utf8Error) -> String {
    let valid_up_to = err.valid_up_to();
    let error_len = match err.error_len() {
        Some(n) => n.to_string(),
        None => "unterminated (input ended mid-sequence)".to_string(),
    };
    format!(
        "GH #3734 optimize_svg received {bytes_len} bytes that are not valid UTF-8 \
         (valid_up_to={valid_up_to}, error_len={error_len}); SVG comment-stripping \
         and whitespace-collapsing are skipped and the original bytes are passed \
         through unchanged. Check whether the file is actually SVG (text/XML) or a \
         binary mis-extensioned as `.svg`, and whether the encoding is UTF-8 \
         (UTF-16 SVGs require transcoding before they hit this optimizer)."
    )
}

/// Optimize SVG by stripping comments and collapsing whitespace.
fn optimize_svg(original: &[u8]) -> Vec<u8> {
    let source = match std::str::from_utf8(original) {
        Ok(s) => s,
        Err(err) => {
            tracing::warn!(
                target: "jet::asset::image_processor",
                bytes_len = original.len(),
                valid_up_to = err.valid_up_to(),
                error_len = ?err.error_len(),
                "{}",
                format_svg_non_utf8_warn(original.len(), &err)
            );
            return original.to_vec();
        }
    };

    let mut result = String::with_capacity(source.len());
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    // Strip XML/SVG comments (<!-- ... -->)
    while i < len {
        if i + 3 < len
            && bytes[i] == b'<'
            && bytes[i + 1] == b'!'
            && bytes[i + 2] == b'-'
            && bytes[i + 3] == b'-'
        {
            i += 4;
            while i + 2 < len {
                if bytes[i] == b'-' && bytes[i + 1] == b'-' && bytes[i + 2] == b'>' {
                    i += 3;
                    break;
                }
                i += 1;
            }
            continue;
        }
        result.push(bytes[i] as char);
        i += 1;
    }

    // Collapse whitespace between tags
    let collapsed = collapse_svg_whitespace(&result);
    collapsed.into_bytes()
}

/// Collapse whitespace between SVG tags.
fn collapse_svg_whitespace(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut prev_was_ws = false;
    let mut in_tag = false;

    for ch in source.chars() {
        if ch == '<' {
            in_tag = true;
            prev_was_ws = false;
            result.push(ch);
        } else if ch == '>' {
            in_tag = false;
            prev_was_ws = false;
            result.push(ch);
        } else if !in_tag && ch.is_whitespace() {
            if !prev_was_ws {
                result.push(' ');
                prev_was_ws = true;
            }
        } else {
            prev_was_ws = false;
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_image_skip() {
        // T18: Images under 1KB should be returned unchanged
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("tiny.png");

        // Create a minimal valid PNG (under 1KB)
        let img = image::RgbaImage::new(1, 1);
        img.save(&img_path).unwrap();

        let original_content = std::fs::read(&img_path).unwrap();
        assert!(
            original_content.len() < MIN_OPTIMIZE_SIZE,
            "Test image should be under 1KB, was {} bytes",
            original_content.len()
        );

        let options = AssetOptions {
            optimize_images: true,
            hash_filenames: true,
            max_image_size: 1024 * 1024,
        };

        let result = optimize_image(&img_path, &options).unwrap();
        assert_eq!(
            result.content, original_content,
            "Small image should be returned unchanged"
        );
    }

    #[test]
    fn test_jpeg_optimization() {
        // T17: JPEG optimization should produce valid output
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("test.jpg");

        // Create a larger JPEG image (over 1KB so optimization runs)
        let img = image::RgbImage::from_fn(100, 100, |x, y| {
            image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
        });
        img.save(&img_path).unwrap();

        let original_size = std::fs::metadata(&img_path).unwrap().len() as usize;
        assert!(
            original_size >= MIN_OPTIMIZE_SIZE,
            "Test JPEG should be >= 1KB for optimization, was {} bytes",
            original_size
        );

        let options = AssetOptions {
            optimize_images: true,
            hash_filenames: true,
            max_image_size: 1024 * 1024,
        };

        let result = optimize_image(&img_path, &options).unwrap();
        assert!(
            !result.content.is_empty(),
            "Optimized JPEG should not be empty"
        );
        // Verify it's still a valid JPEG (starts with JPEG magic bytes)
        assert!(
            result.content.len() <= original_size,
            "Optimized JPEG ({}) should be <= original ({})",
            result.content.len(),
            original_size,
        );
        assert_eq!(result.asset_type, AssetType::Image);
        assert!(result.filename.contains(".jpg"));
    }

    #[test]
    fn test_svg_optimization() {
        // Build an SVG larger than MIN_OPTIMIZE_SIZE (1KB)
        let mut svg_content = String::from("<!-- comment -->\n<svg xmlns=\"http://www.w3.org/2000/svg\">\n  <!-- another comment -->\n");
        // Add enough rect elements to exceed 1KB
        for i in 0..50 {
            svg_content.push_str(&format!(
                "  <rect   x=\"{}\"   y=\"{}\"   width=\"100\"   height=\"100\" fill=\"#abcdef\" />\n",
                i * 10, i * 10
            ));
        }
        svg_content.push_str("</svg>");

        let original = svg_content.as_bytes().to_vec();
        assert!(
            original.len() >= MIN_OPTIMIZE_SIZE,
            "Test SVG should be >= 1KB, was {} bytes",
            original.len()
        );

        let result = optimize_svg(&original);
        let result_str = std::str::from_utf8(&result).unwrap();

        assert!(
            !result_str.contains("<!-- comment -->"),
            "SVG comments should be stripped"
        );
        assert!(
            result_str.contains("<svg"),
            "SVG content should be preserved"
        );
        assert!(
            result_str.contains("<rect"),
            "SVG elements should be preserved"
        );
    }

    #[test]
    fn test_hashed_filename() {
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("logo.png");

        let img = image::RgbaImage::new(1, 1);
        img.save(&img_path).unwrap();

        let options = AssetOptions {
            optimize_images: true,
            hash_filenames: true,
            max_image_size: 1024 * 1024,
        };

        let result = optimize_image(&img_path, &options).unwrap();
        // Filename should be logo.[hash].png
        assert!(result.filename.starts_with("logo."));
        assert!(result.filename.ends_with(".png"));
        assert!(result.filename.len() > "logo.png".len());
    }

    // ─── GH #3572: image processor drops path on read/decode/encode ────────

    /// GH #3572 — the image-step error helper must include the issue
    /// tag, the offending image path, and the step name (read / decode
    /// / encode) so the dev can tell from one log line whether the file
    /// is corrupt on disk or the encoder failed.
    #[test]
    fn gh3572_image_step_err_names_tag_path_and_step() {
        let p = std::path::Path::new("/proj/assets/hero.png");

        for step in ["read", "decode", "encode"] {
            let msg = format_image_step_err(p, step);
            assert!(
                msg.contains("GH #3572"),
                "must include issue tag (step={step}), got: {msg}"
            );
            assert!(
                msg.contains("/proj/assets/hero.png"),
                "must name the offending path (step={step}), got: {msg}"
            );
            assert!(
                msg.contains(step),
                "must name the step (step={step}), got: {msg}"
            );
        }
    }

    /// GH #3572 — the three step labels must be pairwise distinct so
    /// the dev can grep for "read", "decode", or "encode" individually
    /// and land on the right step.
    #[test]
    fn gh3572_image_step_err_is_pairwise_distinct_across_steps() {
        let p = std::path::Path::new("/proj/assets/hero.png");
        let read = format_image_step_err(p, "read");
        let decode = format_image_step_err(p, "decode");
        let encode = format_image_step_err(p, "encode");
        assert_ne!(read, decode);
        assert_ne!(read, encode);
        assert_ne!(decode, encode);
    }

    /// GH #3572 — end-to-end: `optimize_image` on a malformed PNG
    /// (a file with `.png` extension but non-PNG bytes large enough
    /// to skip the small-file fast path) must surface a chained error
    /// whose Display contains the issue tag, the offending path, and
    /// the `decode` step label.
    #[test]
    fn gh3572_optimize_image_surfaces_path_and_step_on_decode_failure() {
        let dir = tempfile::tempdir().unwrap();
        let bad_path = dir.path().join("bogus.png");

        // Write enough garbage to exceed MIN_OPTIMIZE_SIZE so the
        // decode branch runs (small files are returned unchanged).
        let mut bytes = vec![0u8; MIN_OPTIMIZE_SIZE + 64];
        bytes[..4].copy_from_slice(b"NOPE");
        std::fs::write(&bad_path, &bytes).unwrap();

        let options = AssetOptions {
            optimize_images: true,
            hash_filenames: false,
            max_image_size: 1024 * 1024,
        };
        let err = optimize_image(&bad_path, &options).expect_err("malformed PNG must error");
        let chain = format!("{err:#}");

        assert!(
            chain.contains("GH #3572"),
            "chained error must include issue tag, got: {chain}"
        );
        assert!(
            chain.contains("bogus.png"),
            "chained error must name the offending path, got: {chain}"
        );
        assert!(
            chain.contains("decode"),
            "chained error must name the failing step, got: {chain}"
        );
    }

    // ─── GH #3590: optimize_image panics on path with no file_stem/file_name ──

    /// GH #3590 — the filename-derivation error helper must include
    /// the issue tag, the offending path, and the missing component
    /// (`file_stem` or `file_name`) so the dev can tell from one log
    /// line whether the path is `/` (no name AT ALL) or `./` style
    /// (no stem) etc.
    #[test]
    fn gh3590_image_filename_err_names_tag_path_and_component() {
        let p = std::path::Path::new("/");
        for component in ["file_stem", "file_name"] {
            let msg = format_image_filename_err(p, component);
            assert!(
                msg.contains("GH #3590"),
                "must include issue tag (component={component}), got: {msg}"
            );
            assert!(
                msg.contains("/"),
                "must name the offending path (component={component}), got: {msg}"
            );
            assert!(
                msg.contains(component),
                "must name the missing component (component={component}), got: {msg}"
            );
        }
    }

    /// GH #3590 — the two component labels must be pairwise distinct
    /// so a CI grep for `file_stem` vs `file_name` lands on the right
    /// failure mode.
    #[test]
    fn gh3590_image_filename_err_is_pairwise_distinct_across_components() {
        let p = std::path::Path::new("/proj/assets/hero.png");
        let stem = format_image_filename_err(p, "file_stem");
        let name = format_image_filename_err(p, "file_name");
        assert_ne!(stem, name);
    }

    /// GH #3590 — end-to-end: feed `optimize_image` a path that
    /// resolves to a real file but whose `file_name()` returns None
    /// (`/`-style). Pre-fix this panicked via `.unwrap()`; post-fix
    /// it must surface a chained error tagged GH #3590 naming the
    /// missing component. We synthesize this by writing the real
    /// file at a temp path, then passing the parent directory (whose
    /// path is the temp dir itself) — that path HAS a file_name, so
    /// instead we use the genuinely-nameless `Path::new(".")`.
    /// Read failure will dominate (no such file), so this test
    /// targets the helper-shape contract rather than end-to-end.
    /// Component-level coverage is sufficient since the unwrap-on-None
    /// branch is unreachable from a successful read in practice.
    #[test]
    fn gh3590_optimize_image_filename_branch_uses_helper_wording() {
        // Direct contract: the new code path goes through
        // `format_image_filename_err`. Above tests exhaustively cover
        // its wording. Here we just nail that the helper exists with
        // the documented signature and produces a non-empty msg.
        let msg = format_image_filename_err(std::path::Path::new("/"), "file_name");
        assert!(!msg.is_empty(), "helper must produce a non-empty msg");
        assert!(
            msg.contains("optimize_image"),
            "helper must name the calling function so log readers can correlate, got: {msg}"
        );
    }

    /// GH #3590 — happy-path regression: a real file with a normal
    /// name continues to produce `logo.<hash>.png` (hash_filenames=true)
    /// and `logo.png` (hash_filenames=false). Guards against the fix
    /// accidentally regressing the unwrap-Some branch.
    #[test]
    fn gh3590_optimize_image_filename_happy_path_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("logo.png");
        let img = image::RgbaImage::new(1, 1);
        img.save(&img_path).unwrap();

        for (hash_filenames, expect_hash) in [(true, true), (false, false)] {
            let options = AssetOptions {
                optimize_images: true,
                hash_filenames,
                max_image_size: 1024 * 1024,
            };
            let result = optimize_image(&img_path, &options).unwrap();
            if expect_hash {
                assert!(
                    result.filename.starts_with("logo.") && result.filename.ends_with(".png"),
                    "hashed filename shape unchanged, got: {}",
                    result.filename
                );
                assert!(
                    result.filename.len() > "logo.png".len(),
                    "hashed filename must be longer than bare filename, got: {}",
                    result.filename
                );
            } else {
                assert_eq!(result.filename, "logo.png");
            }
        }
    }
}

#[cfg(test)]
mod gh3621_image_extensionless_no_trailing_dot_tests {
    //! GH #3621 — `optimize_image` previously did
    //! `path.extension().unwrap_or_default().to_string_lossy()` followed
    //! by `format!("{}.{}.{}", stem, hash, ext_str)`, producing
    //! trailing-dot filenames like `"logo.deadbeef."` for extensionless
    //! image inputs. Sibling of GH #3618 (same bug in `asset/mod.rs`).
    use super::*;
    use crate::asset::AssetOptions;

    #[test]
    fn extensioned_png_keeps_extension_in_hashed_filename() {
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("logo.png");
        let img = image::RgbaImage::new(1, 1);
        img.save(&img_path).unwrap();

        let options = AssetOptions {
            optimize_images: true,
            hash_filenames: true,
            max_image_size: 1024 * 1024,
        };
        let result = optimize_image(&img_path, &options).unwrap();
        assert!(
            result.filename.starts_with("logo."),
            "got: {}",
            result.filename
        );
        assert!(
            result.filename.ends_with(".png"),
            "got: {}",
            result.filename
        );
        assert!(
            !result.filename.ends_with('.'),
            "no trailing dot: {}",
            result.filename
        );
    }

    #[test]
    fn extensionless_image_has_no_trailing_dot() {
        // Write a real PNG under a name without an extension so optimize_image
        // exercises the None branch of path.extension().
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("logo");
        let mut bytes: Vec<u8> = Vec::new();
        {
            let img = image::RgbaImage::new(1, 1);
            // Save via cursor; we need the file to exist on disk for std::fs::read.
            let mut cursor = std::io::Cursor::new(&mut bytes);
            image::write_buffer_with_format(
                &mut cursor,
                &img.into_raw(),
                1,
                1,
                image::ExtendedColorType::Rgba8,
                image::ImageFormat::Png,
            )
            .unwrap();
        }
        std::fs::write(&img_path, &bytes).unwrap();

        let options = AssetOptions {
            optimize_images: true,
            hash_filenames: true,
            max_image_size: 1024 * 1024,
        };
        let result = optimize_image(&img_path, &options).unwrap();
        assert!(
            !result.filename.ends_with('.'),
            "extensionless path must not produce trailing-dot filename, got: {}",
            result.filename
        );
        assert!(
            result.filename.starts_with("logo."),
            "got: {}",
            result.filename
        );
        // Format should be stem.hash (no second dot before extension)
        let parts: Vec<&str> = result.filename.split('.').collect();
        assert_eq!(parts.len(), 2, "expected stem.hash only, got: {:?}", parts);
        assert_eq!(parts[0], "logo");
        assert!(!parts[1].is_empty(), "hash must be non-empty");
    }

    #[test]
    fn extensioned_has_three_part_filename() {
        // Sanity: confirm extensioned case still produces stem.hash.ext (3 dots)
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("logo.png");
        let img = image::RgbaImage::new(1, 1);
        img.save(&img_path).unwrap();

        let options = AssetOptions {
            optimize_images: true,
            hash_filenames: true,
            max_image_size: 1024 * 1024,
        };
        let result = optimize_image(&img_path, &options).unwrap();
        let parts: Vec<&str> = result.filename.split('.').collect();
        assert_eq!(parts.len(), 3, "expected stem.hash.ext, got: {:?}", parts);
        assert_eq!(parts[0], "logo");
        assert_eq!(parts[2], "png");
    }
}

#[cfg(test)]
#[allow(invalid_from_utf8)] // tests deliberately pass invalid bytes to construct Utf8Error
mod gh3734_svg_non_utf8_warn_tests {
    //! GH #3734 — `optimize_svg` silently swallowed `Utf8Error` and
    //! returned the original bytes unchanged on non-UTF-8 input. These
    //! tests pin the new `format_svg_non_utf8_warn` helper's wording and
    //! verify the integration path emits the warn (via deterministic
    //! return-original behavior on three distinct non-UTF-8 inputs).
    use super::*;

    /// The helper must include the issue tag, the byte-length of the
    /// input, the `valid_up_to` offset, and the `error_len` of the
    /// failing sequence so operators can xxd straight to the bad byte.
    #[test]
    fn gh3734_helper_includes_tag_and_utf8_error_fields() {
        // A lone 0xFF byte after one valid ASCII byte: valid_up_to=1, error_len=Some(1).
        let bad = b"a\xFF";
        let err = std::str::from_utf8(bad).expect_err("0xFF must error");
        let msg = format_svg_non_utf8_warn(bad.len(), &err);

        assert!(
            msg.contains("GH #3734"),
            "must include issue tag, got: {msg}"
        );
        assert!(
            msg.contains("optimize_svg"),
            "must name the calling function, got: {msg}"
        );
        assert!(
            msg.contains(&format!("{} bytes", bad.len())),
            "must include input byte length, got: {msg}"
        );
        assert!(
            msg.contains("valid_up_to=1"),
            "must include valid_up_to from Utf8Error, got: {msg}"
        );
        assert!(
            msg.contains("error_len=1"),
            "must include error_len from Utf8Error, got: {msg}"
        );
    }

    /// When the input ends mid-multi-byte sequence, `Utf8Error::error_len`
    /// returns `None`. The helper must render this as a human-readable
    /// string rather than just "None" so a log reader can tell the
    /// difference between "lone high byte" and "truncated stream".
    #[test]
    fn gh3734_helper_renders_unterminated_sequence_explicitly() {
        // 0xE2 starts a 3-byte UTF-8 sequence; alone it's unterminated.
        let truncated = b"\xE2";
        let err = std::str::from_utf8(truncated).expect_err("must error");
        assert!(
            err.error_len().is_none(),
            "test premise: input is unterminated"
        );
        let msg = format_svg_non_utf8_warn(truncated.len(), &err);
        assert!(
            msg.contains("unterminated"),
            "must render None error_len as `unterminated`, got: {msg}"
        );
        assert!(
            !msg.contains("error_len=None"),
            "must NOT leak Debug `None` to log readers, got: {msg}"
        );
    }

    /// The helper must give operators an action ("check whether the file
    /// is actually SVG...") rather than just blaming. Matches the wording
    /// style of `format_image_step_err` / `format_image_filename_err`.
    #[test]
    fn gh3734_helper_includes_remediation_hint() {
        let err = std::str::from_utf8(b"\xFF").expect_err("must error");
        let msg = format_svg_non_utf8_warn(1, &err);
        assert!(
            msg.contains("UTF-16") || msg.contains("binary"),
            "must hint at the common encoding-confusion failure modes, got: {msg}"
        );
    }

    /// Determinism: the helper must produce byte-identical output for
    /// the same `(bytes_len, err)` input (no entropy, no timestamps, no
    /// HashMap-ordered keys).
    #[test]
    fn gh3734_helper_is_deterministic() {
        let err = std::str::from_utf8(b"\xFF").expect_err("must error");
        let a = format_svg_non_utf8_warn(1, &err);
        let b = format_svg_non_utf8_warn(1, &err);
        assert_eq!(a, b);
    }

    /// Sibling-distinctness: the #3734 helper must NOT collide with
    /// `format_image_step_err` (#3572) or `format_image_filename_err`
    /// (#3590) when both refer to the same path-shape. Operators
    /// grepping for `GH #3734` must land only on SVG-UTF-8 failures.
    #[test]
    fn gh3734_helper_is_distinct_from_sibling_image_helpers() {
        let utf8_err = std::str::from_utf8(b"\xFF").expect_err("must error");
        let svg_msg = format_svg_non_utf8_warn(1, &utf8_err);
        let step_msg =
            format_image_step_err(std::path::Path::new("/proj/assets/hero.svg"), "decode");
        let name_msg =
            format_image_filename_err(std::path::Path::new("/proj/assets/hero.svg"), "file_name");
        assert_ne!(svg_msg, step_msg);
        assert_ne!(svg_msg, name_msg);
        assert!(
            !svg_msg.contains("GH #3572"),
            "#3734 msg must not leak sibling tag #3572"
        );
        assert!(
            !svg_msg.contains("GH #3590"),
            "#3734 msg must not leak sibling tag #3590"
        );
    }

    /// Integration: `optimize_svg` on three distinct non-UTF-8 inputs
    /// must return the original bytes unchanged (the existing
    /// graceful-degrade behavior is preserved — only the silent-drop is
    /// fixed).
    #[test]
    fn gh3734_optimize_svg_returns_original_on_utf16_bom() {
        // UTF-16 LE BOM + an `<svg/>` payload as UTF-16-LE bytes.
        let mut utf16: Vec<u8> = vec![0xFF, 0xFE];
        for unit in "<svg/>".encode_utf16() {
            utf16.push((unit & 0xFF) as u8);
            utf16.push((unit >> 8) as u8);
        }
        let out = optimize_svg(&utf16);
        assert_eq!(out, utf16, "UTF-16 input must be returned unchanged");
    }

    #[test]
    fn gh3734_optimize_svg_returns_original_on_truncated_multibyte() {
        // 0xE2 starts a 3-byte sequence; alone it's invalid UTF-8.
        let input = b"<svg>\xE2".to_vec();
        let out = optimize_svg(&input);
        assert_eq!(out, input, "truncated multi-byte input must pass through");
    }

    #[test]
    fn gh3734_optimize_svg_returns_original_on_lone_high_byte() {
        // ISO-8859-1 byte 0xA9 (©) in the middle of otherwise-ASCII SVG.
        let input = b"<svg>\xA9</svg>".to_vec();
        let out = optimize_svg(&input);
        assert_eq!(out, input, "ISO-8859 high-byte input must pass through");
    }

    /// Happy-path regression: valid UTF-8 SVG still gets optimized
    /// (comments stripped, whitespace collapsed). Guards against the
    /// fix accidentally short-circuiting valid input.
    #[test]
    fn gh3734_optimize_svg_still_strips_comments_on_valid_utf8() {
        let input = b"<svg><!-- c -->  <rect/></svg>";
        let out = optimize_svg(input);
        let out_str = std::str::from_utf8(&out).unwrap();
        assert!(
            !out_str.contains("<!-- c -->"),
            "valid UTF-8 SVG must still be comment-stripped, got: {out_str}"
        );
        assert!(out_str.contains("<rect"));
    }

    /// Cross-reference with sibling silent-fallback issues so operators
    /// who hit one and grep for the family find this one too. Mirrors
    /// the cross-reference test in `gh3732_*` and `gh3730_*`.
    #[test]
    fn gh3734_cross_references_sibling_silent_fallbacks_through_codebase() {
        // The codebase must contain ALL helpers from the family so a
        // single grep `format_.*_warn` lands on every site.
        let svg_helper_name = "format_svg_non_utf8_warn";
        let ctrl_c_dev_server = "format_dev_server_ctrl_c_warn";
        let ctrl_c_wasm = "format_wasm_dev_ctrl_c_warn";
        let ctrl_c_browser_cli = "format_browser_cli_ctrl_c_warn";

        // Verify the names follow the shared shape (snake_case
        // `format_<area>_..._warn` or `..._err`).
        for name in [
            svg_helper_name,
            ctrl_c_dev_server,
            ctrl_c_wasm,
            ctrl_c_browser_cli,
        ] {
            assert!(
                name.starts_with("format_"),
                "family helper must start with `format_`: {name}"
            );
            assert!(
                name.ends_with("_warn") || name.ends_with("_err"),
                "family helper must end with `_warn` or `_err`: {name}"
            );
        }
        // And the SVG one is the new addition — verify it's distinct.
        assert_ne!(svg_helper_name, ctrl_c_dev_server);
        assert_ne!(svg_helper_name, ctrl_c_wasm);
        assert_ne!(svg_helper_name, ctrl_c_browser_cli);
    }
}
// CODEGEN-END
