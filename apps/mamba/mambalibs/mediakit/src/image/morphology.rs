//! Morphological operations and median filter for grayscale images.

use super::types::{Image, PixelFormat};

/// Dilate a grayscale image using a square structuring element.
///
/// Each output pixel is the **maximum** value within the `kernel_size x kernel_size`
/// neighborhood around the corresponding input pixel. `kernel_size` must be odd.
pub fn dilate(img: &Image, kernel_size: usize) -> Image {
    assert_eq!(img.format, PixelFormat::Gray, "input must be grayscale");
    assert!(kernel_size % 2 == 1, "kernel_size must be odd");

    let w = img.width as usize;
    let h = img.height as usize;
    let half = kernel_size / 2;
    let mut out = Image::new(img.width, img.height, PixelFormat::Gray);

    for y in 0..h {
        for x in 0..w {
            let mut max_val: u8 = 0;
            let y_start = y.saturating_sub(half);
            let y_end = (y + half + 1).min(h);
            let x_start = x.saturating_sub(half);
            let x_end = (x + half + 1).min(w);

            for ny in y_start..y_end {
                for nx in x_start..x_end {
                    let v = img.data[ny * w + nx];
                    if v > max_val {
                        max_val = v;
                    }
                }
            }
            out.data[y * w + x] = max_val;
        }
    }
    out
}

/// Erode a grayscale image using a square structuring element.
///
/// Each output pixel is the **minimum** value within the `kernel_size x kernel_size`
/// neighborhood around the corresponding input pixel. `kernel_size` must be odd.
pub fn erode(img: &Image, kernel_size: usize) -> Image {
    assert_eq!(img.format, PixelFormat::Gray, "input must be grayscale");
    assert!(kernel_size % 2 == 1, "kernel_size must be odd");

    let w = img.width as usize;
    let h = img.height as usize;
    let half = kernel_size / 2;
    let mut out = Image::new(img.width, img.height, PixelFormat::Gray);

    for y in 0..h {
        for x in 0..w {
            let mut min_val: u8 = 255;
            let y_start = y.saturating_sub(half);
            let y_end = (y + half + 1).min(h);
            let x_start = x.saturating_sub(half);
            let x_end = (x + half + 1).min(w);

            for ny in y_start..y_end {
                for nx in x_start..x_end {
                    let v = img.data[ny * w + nx];
                    if v < min_val {
                        min_val = v;
                    }
                }
            }
            out.data[y * w + x] = min_val;
        }
    }
    out
}

/// Apply a median blur to a grayscale image.
///
/// Each output pixel is the **median** of the values within the
/// `kernel_size x kernel_size` neighborhood. `kernel_size` must be odd.
pub fn median_blur(img: &Image, kernel_size: usize) -> Image {
    assert_eq!(img.format, PixelFormat::Gray, "input must be grayscale");
    assert!(kernel_size % 2 == 1, "kernel_size must be odd");

    let w = img.width as usize;
    let h = img.height as usize;
    let half = kernel_size / 2;
    let mut out = Image::new(img.width, img.height, PixelFormat::Gray);

    for y in 0..h {
        for x in 0..w {
            let mut neighbors = Vec::new();
            let y_start = y.saturating_sub(half);
            let y_end = (y + half + 1).min(h);
            let x_start = x.saturating_sub(half);
            let x_end = (x + half + 1).min(w);

            for ny in y_start..y_end {
                for nx in x_start..x_end {
                    neighbors.push(img.data[ny * w + nx]);
                }
            }
            neighbors.sort_unstable();
            out.data[y * w + x] = neighbors[neighbors.len() / 2];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- dilate tests ---

    #[test]
    fn test_dilate_basic() {
        // A single bright pixel should expand into the neighborhood.
        let mut img = Image::new(5, 5, PixelFormat::Gray);
        img.data[12] = 200; // center pixel (2,2)
        let out = dilate(&img, 3);
        // Center and its 3x3 neighbors should all be 200
        for dy in 1..=3 {
            for dx in 1..=3 {
                assert_eq!(out.data[dy * 5 + dx], 200);
            }
        }
        // Corner (0,0) should remain 0
        assert_eq!(out.data[0], 0);
    }

    #[test]
    fn test_dilate_uniform() {
        // Uniform image should be unchanged.
        let img = Image::from_raw(vec![42; 16], 4, 4, PixelFormat::Gray);
        let out = dilate(&img, 3);
        assert!(out.data.iter().all(|&v| v == 42));
    }

    #[test]
    fn test_dilate_kernel_1() {
        // kernel_size=1 should be identity.
        let img = Image::from_raw(vec![10, 20, 30, 40], 2, 2, PixelFormat::Gray);
        let out = dilate(&img, 1);
        assert_eq!(out.data, img.data);
    }

    // --- erode tests ---

    #[test]
    fn test_erode_basic() {
        // A single dark pixel should expand into the neighborhood.
        let mut img = Image::from_raw(vec![255; 25], 5, 5, PixelFormat::Gray);
        img.data[12] = 50; // center pixel (2,2)
        let out = erode(&img, 3);
        // Center and its 3x3 neighbors should all be 50
        for dy in 1..=3 {
            for dx in 1..=3 {
                assert_eq!(out.data[dy * 5 + dx], 50);
            }
        }
    }

    #[test]
    fn test_erode_uniform() {
        let img = Image::from_raw(vec![100; 16], 4, 4, PixelFormat::Gray);
        let out = erode(&img, 3);
        assert!(out.data.iter().all(|&v| v == 100));
    }

    #[test]
    fn test_dilate_erode_inverse() {
        // On a uniform image, dilate then erode should preserve the image.
        let img = Image::from_raw(vec![128; 25], 5, 5, PixelFormat::Gray);
        let dilated = dilate(&img, 3);
        let restored = erode(&dilated, 3);
        assert_eq!(restored.data, img.data);
    }

    // --- median_blur tests ---

    #[test]
    fn test_median_blur_removes_impulse() {
        // Salt noise on a constant background should be removed by median filter.
        let mut img = Image::from_raw(vec![100; 25], 5, 5, PixelFormat::Gray);
        img.data[12] = 255; // single impulse at center
        let out = median_blur(&img, 3);
        assert_eq!(out.data[12], 100);
    }

    #[test]
    fn test_median_blur_uniform() {
        let img = Image::from_raw(vec![77; 9], 3, 3, PixelFormat::Gray);
        let out = median_blur(&img, 3);
        assert!(out.data.iter().all(|&v| v == 77));
    }

    #[test]
    fn test_median_blur_sorted_row() {
        // 1D-like: 1x5 image
        let img = Image::from_raw(vec![10, 30, 20, 50, 40], 5, 1, PixelFormat::Gray);
        let out = median_blur(&img, 3);
        // Middle pixel: median of [30, 20, 50] = 30
        assert_eq!(out.data[2], 30);
    }
}
