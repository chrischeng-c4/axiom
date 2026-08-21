//! Image convolution filters: blur, sharpen, edge detection.

use super::types::{Image, PixelFormat};

/// Apply a Gaussian blur (3x3 kernel) to a grayscale image.
pub fn gaussian_blur(img: &Image) -> Image {
    assert_eq!(img.format, PixelFormat::Gray, "input must be grayscale");
    let kernel = [
        1.0 / 16.0,
        2.0 / 16.0,
        1.0 / 16.0,
        2.0 / 16.0,
        4.0 / 16.0,
        2.0 / 16.0,
        1.0 / 16.0,
        2.0 / 16.0,
        1.0 / 16.0,
    ];
    convolve_gray(img, &kernel, 3)
}

/// Apply a sharpening filter (3x3) to a grayscale image.
pub fn sharpen(img: &Image) -> Image {
    assert_eq!(img.format, PixelFormat::Gray, "input must be grayscale");
    let kernel = [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];
    convolve_gray(img, &kernel, 3)
}

/// Sobel edge detection on a grayscale image.
///
/// Returns the magnitude of the gradient.
pub fn sobel_edges(img: &Image) -> Image {
    assert_eq!(img.format, PixelFormat::Gray, "input must be grayscale");
    let gx_kernel = [-1.0, 0.0, 1.0, -2.0, 0.0, 2.0, -1.0, 0.0, 1.0];
    let gy_kernel = [-1.0, -2.0, -1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0];

    let w = img.width as usize;
    let h = img.height as usize;
    let mut out = Image::new(img.width, img.height, PixelFormat::Gray);

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let mut gx = 0.0f64;
            let mut gy = 0.0f64;
            for ky in 0..3 {
                for kx in 0..3 {
                    let px = img.data[(y + ky - 1) * w + (x + kx - 1)] as f64;
                    gx += px * gx_kernel[ky * 3 + kx];
                    gy += px * gy_kernel[ky * 3 + kx];
                }
            }
            let mag = (gx * gx + gy * gy).sqrt().min(255.0) as u8;
            out.data[y * w + x] = mag;
        }
    }
    out
}

/// Canny edge detection on a grayscale image.
///
/// Applies the full Canny pipeline:
/// 1. Gaussian blur (noise reduction)
/// 2. Sobel gradients (magnitude + direction)
/// 3. Non-maximum suppression (thin edges)
/// 4. Double thresholding + edge tracking by hysteresis
///
/// `low_threshold` and `high_threshold` are in the range 0.0..255.0.
pub fn canny(img: &Image, low_threshold: f64, high_threshold: f64) -> Image {
    assert_eq!(img.format, PixelFormat::Gray, "input must be grayscale");
    assert!(
        low_threshold <= high_threshold,
        "low_threshold must be <= high_threshold"
    );

    let w = img.width as usize;
    let h = img.height as usize;

    // Step 1: Gaussian blur
    let blurred = gaussian_blur(img);

    // Step 2: Sobel gradients (magnitude and direction)
    let gx_kernel: [f64; 9] = [-1.0, 0.0, 1.0, -2.0, 0.0, 2.0, -1.0, 0.0, 1.0];
    let gy_kernel: [f64; 9] = [-1.0, -2.0, -1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0];

    let mut magnitude = vec![0.0f64; w * h];
    let mut direction = vec![0.0f64; w * h];

    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let mut gx = 0.0f64;
            let mut gy = 0.0f64;
            for ky in 0..3 {
                for kx in 0..3 {
                    let px = blurred.data[(y + ky - 1) * w + (x + kx - 1)] as f64;
                    gx += px * gx_kernel[ky * 3 + kx];
                    gy += px * gy_kernel[ky * 3 + kx];
                }
            }
            magnitude[y * w + x] = (gx * gx + gy * gy).sqrt();
            direction[y * w + x] = gy.atan2(gx);
        }
    }

    // Step 3: Non-maximum suppression
    let mut nms = vec![0.0f64; w * h];
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let angle = direction[y * w + x].to_degrees();
            // Normalize angle to 0..180
            let angle = if angle < 0.0 { angle + 180.0 } else { angle };

            let (n1, n2) = if angle < 22.5 || angle >= 157.5 {
                // Horizontal edge: compare with left/right
                (magnitude[y * w + x - 1], magnitude[y * w + x + 1])
            } else if angle < 67.5 {
                // 45-degree: compare with top-right/bottom-left
                (
                    magnitude[(y - 1) * w + x + 1],
                    magnitude[(y + 1) * w + x - 1],
                )
            } else if angle < 112.5 {
                // Vertical edge: compare with above/below
                (magnitude[(y - 1) * w + x], magnitude[(y + 1) * w + x])
            } else {
                // 135-degree: compare with top-left/bottom-right
                (
                    magnitude[(y - 1) * w + x - 1],
                    magnitude[(y + 1) * w + x + 1],
                )
            };

            let mag = magnitude[y * w + x];
            nms[y * w + x] = if mag >= n1 && mag >= n2 { mag } else { 0.0 };
        }
    }

    // Step 4: Double thresholding
    const STRONG: u8 = 255;
    const WEAK: u8 = 75;
    let mut result = vec![0u8; w * h];
    for i in 0..w * h {
        if nms[i] >= high_threshold {
            result[i] = STRONG;
        } else if nms[i] >= low_threshold {
            result[i] = WEAK;
        }
    }

    // Step 5: Edge tracking by hysteresis
    // Weak pixels connected to strong pixels become strong.
    let mut changed = true;
    while changed {
        changed = false;
        for y in 1..h.saturating_sub(1) {
            for x in 1..w.saturating_sub(1) {
                if result[y * w + x] != WEAK {
                    continue;
                }
                // Check 8-connected neighbors for a strong pixel
                let has_strong = [
                    result[(y - 1) * w + x - 1],
                    result[(y - 1) * w + x],
                    result[(y - 1) * w + x + 1],
                    result[y * w + x - 1],
                    result[y * w + x + 1],
                    result[(y + 1) * w + x - 1],
                    result[(y + 1) * w + x],
                    result[(y + 1) * w + x + 1],
                ]
                .iter()
                .any(|&v| v == STRONG);

                if has_strong {
                    result[y * w + x] = STRONG;
                    changed = true;
                }
            }
        }
    }

    // Suppress remaining weak pixels
    for v in &mut result {
        if *v != STRONG {
            *v = 0;
        }
    }

    Image::from_raw(result, img.width, img.height, PixelFormat::Gray)
}

fn convolve_gray(img: &Image, kernel: &[f64], ksize: usize) -> Image {
    let w = img.width as usize;
    let h = img.height as usize;
    let half = ksize / 2;
    let mut out = Image::new(img.width, img.height, PixelFormat::Gray);

    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f64;
            for ky in 0..ksize {
                for kx in 0..ksize {
                    let sy = (y as isize + ky as isize - half as isize)
                        .max(0)
                        .min(h as isize - 1) as usize;
                    let sx = (x as isize + kx as isize - half as isize)
                        .max(0)
                        .min(w as isize - 1) as usize;
                    let px = img.data[sy * w + sx] as f64;
                    sum += px * kernel[ky * ksize + kx];
                }
            }
            out.data[y * w + x] = sum.clamp(0.0, 255.0) as u8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_gray_image() -> Image {
        let mut img = Image::new(8, 8, PixelFormat::Gray);
        for i in 0..64 {
            img.data[i] = (i * 4) as u8;
        }
        img
    }

    #[test]
    fn test_gaussian_blur() {
        let img = make_gray_image();
        let blurred = gaussian_blur(&img);
        assert_eq!(blurred.width, 8);
        assert_eq!(blurred.height, 8);
    }

    #[test]
    fn test_sharpen() {
        let img = make_gray_image();
        let sharp = sharpen(&img);
        assert_eq!(sharp.data.len(), 64);
    }

    #[test]
    fn test_sobel_edges() {
        // Create image with a vertical edge
        let mut img = Image::new(8, 8, PixelFormat::Gray);
        for y in 0..8 {
            for x in 0..8 {
                img.data[y * 8 + x] = if x < 4 { 0 } else { 255 };
            }
        }
        let edges = sobel_edges(&img);
        // Edge should be detected at column 3-4 boundary
        assert!(edges.data[3 * 8 + 4] > 0 || edges.data[3 * 8 + 3] > 0);
    }

    #[test]
    fn test_canny_detects_edge() {
        // Strong vertical edge: left half black, right half white.
        let mut img = Image::new(16, 16, PixelFormat::Gray);
        for y in 0..16 {
            for x in 0..16 {
                img.data[y * 16 + x] = if x < 8 { 0 } else { 255 };
            }
        }
        let edges = canny(&img, 50.0, 100.0);
        assert_eq!(edges.width, 16);
        assert_eq!(edges.height, 16);
        // Should have at least some edge pixels around the boundary
        let edge_count = edges.data.iter().filter(|&&v| v == 255).count();
        assert!(edge_count > 0, "Canny should detect the vertical edge");
    }

    #[test]
    fn test_canny_uniform_no_edges() {
        // Uniform image should produce no edges.
        let img = Image::from_raw(vec![128; 16 * 16], 16, 16, PixelFormat::Gray);
        let edges = canny(&img, 50.0, 100.0);
        let edge_count = edges.data.iter().filter(|&&v| v == 255).count();
        assert_eq!(edge_count, 0, "uniform image should have no edges");
    }

    #[test]
    fn test_canny_threshold_ordering() {
        // Higher thresholds should produce fewer or equal edge pixels.
        let mut img = Image::new(16, 16, PixelFormat::Gray);
        for y in 0..16 {
            for x in 0..16 {
                img.data[y * 16 + x] = if x < 8 { 0 } else { 255 };
            }
        }
        let edges_low = canny(&img, 20.0, 60.0);
        let edges_high = canny(&img, 100.0, 200.0);
        let count_low = edges_low.data.iter().filter(|&&v| v == 255).count();
        let count_high = edges_high.data.iter().filter(|&&v| v == 255).count();
        assert!(count_low >= count_high);
    }
}
