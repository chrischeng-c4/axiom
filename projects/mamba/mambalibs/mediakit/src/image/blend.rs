//! Image composition and blending operations.
//!
//! All operations produce a new image (non-mutating).

use super::types::{Image, PixelFormat};

/// Blend mode for compositing two images.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// Standard alpha blending: out = src * alpha + dst * (1 - alpha).
    Normal,
    /// Multiply: out = src * dst / 255.
    Multiply,
    /// Screen: out = 255 - (255 - src) * (255 - dst) / 255.
    Screen,
    /// Overlay: combines Multiply and Screen.
    Overlay,
    /// Additive: out = min(src + dst, 255).
    Add,
    /// Difference: out = |src - dst|.
    Difference,
}

/// Alpha-blend two same-sized images.
///
/// `alpha` ∈ [0.0, 1.0] — 0.0 = all `base`, 1.0 = all `overlay`.
/// Both images must have the same dimensions and format.
pub fn alpha_blend(base: &Image, overlay: &Image, alpha: f64) -> Image {
    assert_eq!(base.width, overlay.width, "width mismatch");
    assert_eq!(base.height, overlay.height, "height mismatch");
    assert_eq!(base.format, overlay.format, "format mismatch");

    let alpha = alpha.clamp(0.0, 1.0);
    let inv = 1.0 - alpha;
    let mut out = base.clone();

    for i in 0..out.data.len() {
        out.data[i] = (base.data[i] as f64 * inv + overlay.data[i] as f64 * alpha)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
    out
}

/// Blend two same-sized images using the specified blend mode.
///
/// `opacity` ∈ [0.0, 1.0] controls how much of the blended result is mixed
/// with the base.
pub fn blend(base: &Image, overlay: &Image, mode: BlendMode, opacity: f64) -> Image {
    assert_eq!(base.width, overlay.width, "width mismatch");
    assert_eq!(base.height, overlay.height, "height mismatch");
    assert_eq!(base.format, overlay.format, "format mismatch");

    let opacity = opacity.clamp(0.0, 1.0);
    let inv = 1.0 - opacity;
    let mut out = base.clone();

    for i in 0..out.data.len() {
        let b = base.data[i] as f64;
        let o = overlay.data[i] as f64;

        let blended = match mode {
            BlendMode::Normal => o,
            BlendMode::Multiply => b * o / 255.0,
            BlendMode::Screen => 255.0 - (255.0 - b) * (255.0 - o) / 255.0,
            BlendMode::Overlay => {
                if b < 128.0 {
                    2.0 * b * o / 255.0
                } else {
                    255.0 - 2.0 * (255.0 - b) * (255.0 - o) / 255.0
                }
            }
            BlendMode::Add => (b + o).min(255.0),
            BlendMode::Difference => (b - o).abs(),
        };

        out.data[i] = (b * inv + blended * opacity).round().clamp(0.0, 255.0) as u8;
    }
    out
}

/// Composite `overlay` onto `base` at position `(x, y)`.
///
/// Pixels outside the base image bounds are clipped. If the overlay is RGBA,
/// the alpha channel is used for per-pixel blending. Otherwise, the global
/// `opacity` is used.
pub fn composite(base: &Image, overlay: &Image, x: i32, y: i32, opacity: f64) -> Image {
    let mut out = base.clone();
    let opacity = opacity.clamp(0.0, 1.0);
    let b_ch = base.format.channels();
    let o_ch = overlay.format.channels();

    for oy in 0..overlay.height as i32 {
        for ox in 0..overlay.width as i32 {
            let bx = x + ox;
            let by = y + oy;
            if bx < 0 || by < 0 || bx >= base.width as i32 || by >= base.height as i32 {
                continue;
            }

            let bx = bx as usize;
            let by = by as usize;
            let base_idx = (by * base.width as usize + bx) * b_ch;
            let over_idx =
                (oy as usize * overlay.width as usize + ox as usize) * o_ch;

            // Determine per-pixel alpha
            let px_alpha = if overlay.format == PixelFormat::Rgba {
                overlay.data[over_idx + 3] as f64 / 255.0 * opacity
            } else {
                opacity
            };
            let inv = 1.0 - px_alpha;

            // Blend channels (min of base channels and overlay channels)
            let ch = b_ch.min(if o_ch == 4 { 3 } else { o_ch });
            for c in 0..ch {
                let bv = out.data[base_idx + c] as f64;
                let ov = overlay.data[over_idx + c] as f64;
                out.data[base_idx + c] =
                    (bv * inv + ov * px_alpha).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

/// Create a horizontal gradient image (left color to right color).
///
/// Useful for testing blending or creating gradient overlays.
pub fn gradient_h(width: u32, height: u32, left: &[u8; 3], right: &[u8; 3]) -> Image {
    let mut img = Image::new(width, height, PixelFormat::Rgb);
    for y in 0..height {
        for x in 0..width {
            let t = x as f64 / (width.max(1) - 1).max(1) as f64;
            let r = (left[0] as f64 * (1.0 - t) + right[0] as f64 * t).round() as u8;
            let g = (left[1] as f64 * (1.0 - t) + right[1] as f64 * t).round() as u8;
            let b = (left[2] as f64 * (1.0 - t) + right[2] as f64 * t).round() as u8;
            img.set_pixel(x, y, &[r, g, b]);
        }
    }
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_solid(w: u32, h: u32, val: u8) -> Image {
        Image::from_raw(vec![val; (w * h) as usize], w, h, PixelFormat::Gray)
    }

    #[test]
    fn test_alpha_blend_50() {
        let base = make_solid(4, 4, 0);
        let over = make_solid(4, 4, 200);
        let result = alpha_blend(&base, &over, 0.5);
        assert_eq!(result.data[0], 100);
    }

    #[test]
    fn test_alpha_blend_zero() {
        let base = make_solid(4, 4, 50);
        let over = make_solid(4, 4, 200);
        let result = alpha_blend(&base, &over, 0.0);
        assert_eq!(result.data[0], 50);
    }

    #[test]
    fn test_alpha_blend_one() {
        let base = make_solid(4, 4, 50);
        let over = make_solid(4, 4, 200);
        let result = alpha_blend(&base, &over, 1.0);
        assert_eq!(result.data[0], 200);
    }

    #[test]
    fn test_blend_multiply() {
        let base = make_solid(2, 2, 128);
        let over = make_solid(2, 2, 128);
        let result = blend(&base, &over, BlendMode::Multiply, 1.0);
        // 128 * 128 / 255 ≈ 64
        assert!((result.data[0] as i16 - 64).abs() <= 1);
    }

    #[test]
    fn test_blend_screen() {
        let base = make_solid(2, 2, 128);
        let over = make_solid(2, 2, 128);
        let result = blend(&base, &over, BlendMode::Screen, 1.0);
        // 255 - (255-128)*(255-128)/255 ≈ 192
        assert!((result.data[0] as i16 - 192).abs() <= 1);
    }

    #[test]
    fn test_blend_add() {
        let base = make_solid(2, 2, 200);
        let over = make_solid(2, 2, 100);
        let result = blend(&base, &over, BlendMode::Add, 1.0);
        assert_eq!(result.data[0], 255); // clamped
    }

    #[test]
    fn test_blend_difference() {
        let base = make_solid(2, 2, 200);
        let over = make_solid(2, 2, 80);
        let result = blend(&base, &over, BlendMode::Difference, 1.0);
        assert_eq!(result.data[0], 120);
    }

    #[test]
    fn test_composite_with_offset() {
        let base = make_solid(8, 8, 0);
        let over = make_solid(4, 4, 255);
        let result = composite(&base, &over, 2, 2, 1.0);
        // (2,2) should be 255
        assert_eq!(result.data[2 * 8 + 2], 255);
        // (0,0) should still be 0
        assert_eq!(result.data[0], 0);
    }

    #[test]
    fn test_composite_clipped() {
        let base = make_solid(4, 4, 100);
        let over = make_solid(4, 4, 200);
        // Overlay partially outside
        let result = composite(&base, &over, -2, -2, 1.0);
        // Only bottom-right 2x2 of overlay should be visible
        assert_eq!(result.data[0], 200);
        assert_eq!(result.data[1], 200);
        assert_eq!(result.data[2], 100); // not covered
    }

    #[test]
    fn test_gradient_h() {
        let grad = gradient_h(256, 1, &[0, 0, 0], &[255, 255, 255]);
        assert_eq!(grad.data[0], 0); // left
        assert_eq!(grad.data[255 * 3], 255); // right
    }

    #[test]
    fn test_composite_rgba() {
        let base = Image::from_raw(vec![100; 4 * 3], 2, 2, PixelFormat::Rgb);
        // Semi-transparent overlay
        let mut over = Image::new(2, 2, PixelFormat::Rgba);
        for y in 0..2u32 {
            for x in 0..2u32 {
                over.set_pixel(x, y, &[200, 200, 200, 128]); // ~50% alpha
            }
        }
        let result = composite(&base, &over, 0, 0, 1.0);
        // Should be approximately 150 (midpoint of 100 and 200 at ~50% alpha)
        let v = result.data[0];
        assert!((v as i16 - 150).abs() <= 2, "expected ~150, got {v}");
    }
}
