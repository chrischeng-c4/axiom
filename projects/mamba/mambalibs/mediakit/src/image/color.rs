//! Color space conversions: RGB <-> HSV, RGB <-> LAB, RGB <-> Gray.
//!
//! All conversions operate on whole images and produce a new `Image` in the
//! target color space. Pixel values are stored as `u8` (0-255); intermediate
//! calculations use `f64` for precision.

use super::types::{Image, PixelFormat};

// ── RGB → Grayscale ──────────────────────────────────────────────────

/// Convert an RGB image to single-channel grayscale using BT.601 luminance.
pub fn rgb_to_gray(img: &Image) -> Image {
    assert_eq!(img.format, PixelFormat::Rgb, "input must be RGB");
    let mut out = Image::new(img.width, img.height, PixelFormat::Gray);

    for i in 0..img.n_pixels() {
        let si = i * 3;
        let r = img.data[si] as f64;
        let g = img.data[si + 1] as f64;
        let b = img.data[si + 2] as f64;
        out.data[i] = (0.299 * r + 0.587 * g + 0.114 * b).round() as u8;
    }
    out
}

/// Convert a grayscale image to 3-channel RGB (all channels equal).
pub fn gray_to_rgb(img: &Image) -> Image {
    assert_eq!(img.format, PixelFormat::Gray, "input must be Gray");
    let mut out = Image::new(img.width, img.height, PixelFormat::Rgb);

    for i in 0..img.n_pixels() {
        let v = img.data[i];
        let di = i * 3;
        out.data[di] = v;
        out.data[di + 1] = v;
        out.data[di + 2] = v;
    }
    out
}

// ── RGB → HSV ────────────────────────────────────────────────────────

/// Convert an RGB image to HSV.
///
/// HSV encoding: H ∈ [0, 179] (OpenCV convention), S ∈ [0, 255], V ∈ [0, 255].
pub fn rgb_to_hsv(img: &Image) -> Image {
    assert_eq!(img.format, PixelFormat::Rgb, "input must be RGB");
    let mut out = Image::new(img.width, img.height, PixelFormat::Hsv);

    for i in 0..img.n_pixels() {
        let si = i * 3;
        let r = img.data[si] as f64 / 255.0;
        let g = img.data[si + 1] as f64 / 255.0;
        let b = img.data[si + 2] as f64 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        // Hue
        let h = if delta < 1e-10 {
            0.0
        } else if (max - r).abs() < 1e-10 {
            60.0 * (((g - b) / delta) % 6.0)
        } else if (max - g).abs() < 1e-10 {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };
        let h = if h < 0.0 { h + 360.0 } else { h };

        // Saturation
        let s = if max < 1e-10 { 0.0 } else { delta / max };

        let di = i * 3;
        out.data[di] = (h / 2.0).round().min(179.0) as u8; // H: 0-179
        out.data[di + 1] = (s * 255.0).round() as u8; // S: 0-255
        out.data[di + 2] = (max * 255.0).round() as u8; // V: 0-255
    }
    out
}

/// Convert an HSV image back to RGB.
pub fn hsv_to_rgb(img: &Image) -> Image {
    assert_eq!(img.format, PixelFormat::Hsv, "input must be HSV");
    let mut out = Image::new(img.width, img.height, PixelFormat::Rgb);

    for i in 0..img.n_pixels() {
        let si = i * 3;
        let h = img.data[si] as f64 * 2.0; // restore 0-360
        let s = img.data[si + 1] as f64 / 255.0;
        let v = img.data[si + 2] as f64 / 255.0;

        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r1, g1, b1) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let di = i * 3;
        out.data[di] = ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
        out.data[di + 1] = ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
        out.data[di + 2] = ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    out
}

// ── RGB → CIELAB ─────────────────────────────────────────────────────

/// D65 reference white point.
const D65_X: f64 = 95.047;
const D65_Y: f64 = 100.0;
const D65_Z: f64 = 108.883;

/// sRGB gamma linearization.
fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Inverse sRGB gamma.
fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// CIE LAB f(t) function.
fn lab_f(t: f64) -> f64 {
    let delta = 6.0 / 29.0;
    if t > delta * delta * delta {
        t.cbrt()
    } else {
        t / (3.0 * delta * delta) + 4.0 / 29.0
    }
}

/// Inverse CIE LAB f(t).
fn lab_f_inv(t: f64) -> f64 {
    let delta = 6.0 / 29.0;
    if t > delta {
        t * t * t
    } else {
        3.0 * delta * delta * (t - 4.0 / 29.0)
    }
}

/// Convert an RGB image to CIELAB.
///
/// Internal representation: L ∈ [0, 255] (mapped from 0-100),
/// a ∈ [0, 255] (mapped from -128..+127), b ∈ [0, 255] (mapped from -128..+127).
pub fn rgb_to_lab(img: &Image) -> Image {
    assert_eq!(img.format, PixelFormat::Rgb, "input must be RGB");
    let mut out = Image::new(img.width, img.height, PixelFormat::Lab);

    for i in 0..img.n_pixels() {
        let si = i * 3;
        // sRGB -> linear
        let r_lin = srgb_to_linear(img.data[si] as f64 / 255.0);
        let g_lin = srgb_to_linear(img.data[si + 1] as f64 / 255.0);
        let b_lin = srgb_to_linear(img.data[si + 2] as f64 / 255.0);

        // linear RGB -> XYZ (sRGB D65)
        let x = 0.4124564 * r_lin + 0.3575761 * g_lin + 0.1804375 * b_lin;
        let y = 0.2126729 * r_lin + 0.7151522 * g_lin + 0.0721750 * b_lin;
        let z = 0.0193339 * r_lin + 0.1191920 * g_lin + 0.9503041 * b_lin;

        // XYZ -> Lab
        let fx = lab_f(x * 100.0 / D65_X);
        let fy = lab_f(y * 100.0 / D65_Y);
        let fz = lab_f(z * 100.0 / D65_Z);

        let l_val = 116.0 * fy - 16.0; // 0-100
        let a_val = 500.0 * (fx - fy); // roughly -128..+127
        let b_val = 200.0 * (fy - fz); // roughly -128..+127

        let di = i * 3;
        out.data[di] = (l_val * 255.0 / 100.0).round().clamp(0.0, 255.0) as u8;
        out.data[di + 1] = (a_val + 128.0).round().clamp(0.0, 255.0) as u8;
        out.data[di + 2] = (b_val + 128.0).round().clamp(0.0, 255.0) as u8;
    }
    out
}

/// Convert a CIELAB image back to RGB.
pub fn lab_to_rgb(img: &Image) -> Image {
    assert_eq!(img.format, PixelFormat::Lab, "input must be Lab");
    let mut out = Image::new(img.width, img.height, PixelFormat::Rgb);

    for i in 0..img.n_pixels() {
        let si = i * 3;
        let l_val = img.data[si] as f64 * 100.0 / 255.0;
        let a_val = img.data[si + 1] as f64 - 128.0;
        let b_val = img.data[si + 2] as f64 - 128.0;

        // Lab -> XYZ
        let fy = (l_val + 16.0) / 116.0;
        let fx = a_val / 500.0 + fy;
        let fz = fy - b_val / 200.0;

        let x = D65_X / 100.0 * lab_f_inv(fx);
        let y = D65_Y / 100.0 * lab_f_inv(fy);
        let z = D65_Z / 100.0 * lab_f_inv(fz);

        // XYZ -> linear RGB
        let r_lin = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
        let g_lin = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z;
        let b_lin = 0.0556434 * x - 0.2040259 * y + 1.0572252 * z;

        // linear -> sRGB
        let di = i * 3;
        out.data[di] = (linear_to_srgb(r_lin) * 255.0).round().clamp(0.0, 255.0) as u8;
        out.data[di + 1] = (linear_to_srgb(g_lin) * 255.0).round().clamp(0.0, 255.0) as u8;
        out.data[di + 2] = (linear_to_srgb(b_lin) * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_gray() {
        let mut img = Image::new(2, 1, PixelFormat::Rgb);
        img.set_pixel(0, 0, &[255, 255, 255]);
        img.set_pixel(1, 0, &[0, 0, 0]);
        let gray = rgb_to_gray(&img);
        assert_eq!(gray.format, PixelFormat::Gray);
        assert!(gray.data[0] >= 254);
        assert_eq!(gray.data[1], 0);
    }

    #[test]
    fn test_gray_to_rgb() {
        let img = Image::from_raw(vec![100, 200], 2, 1, PixelFormat::Gray);
        let rgb = gray_to_rgb(&img);
        assert_eq!(rgb.get_pixel(0, 0), &[100, 100, 100]);
        assert_eq!(rgb.get_pixel(1, 0), &[200, 200, 200]);
    }

    #[test]
    fn test_rgb_hsv_roundtrip() {
        let mut img = Image::new(3, 1, PixelFormat::Rgb);
        img.set_pixel(0, 0, &[255, 0, 0]); // pure red
        img.set_pixel(1, 0, &[0, 255, 0]); // pure green
        img.set_pixel(2, 0, &[0, 0, 255]); // pure blue

        let hsv = rgb_to_hsv(&img);
        assert_eq!(hsv.format, PixelFormat::Hsv);

        // Red: H=0, S=255, V=255
        assert_eq!(hsv.data[0], 0);
        assert_eq!(hsv.data[1], 255);
        assert_eq!(hsv.data[2], 255);

        // Roundtrip
        let back = hsv_to_rgb(&hsv);
        assert_eq!(back.format, PixelFormat::Rgb);
        // Allow +-1 for rounding
        assert!((back.data[0] as i16 - 255).abs() <= 1);
        assert!(back.data[1] <= 1);
        assert!(back.data[2] <= 1);
    }

    #[test]
    fn test_rgb_lab_roundtrip() {
        let mut img = Image::new(2, 1, PixelFormat::Rgb);
        img.set_pixel(0, 0, &[128, 64, 200]);
        img.set_pixel(1, 0, &[50, 100, 150]);

        let lab = rgb_to_lab(&img);
        assert_eq!(lab.format, PixelFormat::Lab);

        let back = lab_to_rgb(&lab);
        // Allow tolerance due to u8 quantization in Lab space
        for i in 0..6 {
            let diff = (back.data[i] as i16 - img.data[i] as i16).abs();
            assert!(diff <= 3, "pixel[{i}]: expected ~{}, got {} (diff {diff})",
                    img.data[i], back.data[i]);
        }
    }

    #[test]
    fn test_black_white_lab() {
        let mut img = Image::new(2, 1, PixelFormat::Rgb);
        img.set_pixel(0, 0, &[0, 0, 0]);
        img.set_pixel(1, 0, &[255, 255, 255]);

        let lab = rgb_to_lab(&img);
        // Black: L should be ~0
        assert!(lab.data[0] < 5);
        // White: L should be ~255
        assert!(lab.data[3] > 250);
    }
}
