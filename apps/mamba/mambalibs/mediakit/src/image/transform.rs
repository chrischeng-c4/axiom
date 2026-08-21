//! Image transformations: brightness, contrast, histogram equalization, threshold.

use super::types::{Image, PixelFormat};

/// Adjust brightness of a grayscale image.
///
/// `delta` is added to each pixel value (clamped to 0-255).
pub fn adjust_brightness(img: &Image, delta: i16) -> Image {
    let mut out = img.clone();
    for v in &mut out.data {
        let new_val = *v as i16 + delta;
        *v = new_val.clamp(0, 255) as u8;
    }
    out
}

/// Adjust contrast of a grayscale image.
///
/// `factor` > 1.0 increases contrast, < 1.0 decreases.
pub fn adjust_contrast(img: &Image, factor: f64) -> Image {
    let mut out = img.clone();
    for v in &mut out.data {
        let new_val = ((*v as f64 - 128.0) * factor + 128.0).clamp(0.0, 255.0);
        *v = new_val as u8;
    }
    out
}

/// Apply binary threshold to a grayscale image.
pub fn threshold(img: &Image, thresh: u8) -> Image {
    assert_eq!(img.format, PixelFormat::Gray);
    let mut out = img.clone();
    for v in &mut out.data {
        *v = if *v >= thresh { 255 } else { 0 };
    }
    out
}

/// Histogram equalization for a grayscale image.
pub fn histogram_equalize(img: &Image) -> Image {
    assert_eq!(img.format, PixelFormat::Gray);
    let n = img.data.len();

    // Compute histogram
    let mut hist = [0u32; 256];
    for &v in &img.data {
        hist[v as usize] += 1;
    }

    // Compute CDF
    let mut cdf = [0u32; 256];
    cdf[0] = hist[0];
    for i in 1..256 {
        cdf[i] = cdf[i - 1] + hist[i];
    }

    // Find min non-zero CDF
    let cdf_min = cdf.iter().find(|&&v| v > 0).copied().unwrap_or(0);

    // Map pixel values
    let mut out = img.clone();
    for v in &mut out.data {
        let mapped = if n > 1 {
            ((cdf[*v as usize] - cdf_min) as f64 / (n as u32 - cdf_min) as f64 * 255.0) as u8
        } else {
            *v
        };
        *v = mapped;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjust_brightness() {
        let img = Image::from_raw(vec![100, 200, 50], 3, 1, PixelFormat::Gray);
        let bright = adjust_brightness(&img, 50);
        assert_eq!(bright.data, vec![150, 250, 100]);

        let dark = adjust_brightness(&img, -60);
        assert_eq!(dark.data, vec![40, 140, 0]);
    }

    #[test]
    fn test_adjust_contrast() {
        let img = Image::from_raw(vec![128], 1, 1, PixelFormat::Gray);
        let out = adjust_contrast(&img, 2.0);
        assert_eq!(out.data[0], 128); // center pixel unchanged
    }

    #[test]
    fn test_threshold() {
        let img = Image::from_raw(vec![50, 100, 150, 200], 4, 1, PixelFormat::Gray);
        let out = threshold(&img, 120);
        assert_eq!(out.data, vec![0, 0, 255, 255]);
    }

    #[test]
    fn test_histogram_equalize() {
        let img = Image::from_raw(vec![10, 10, 10, 200, 200, 200], 6, 1, PixelFormat::Gray);
        let out = histogram_equalize(&img);
        // After equalization, should use full range
        assert!(out.data.iter().any(|&v| v > 200));
    }
}
