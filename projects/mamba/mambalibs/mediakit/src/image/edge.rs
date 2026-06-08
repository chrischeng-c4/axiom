//! Edge detection algorithms: Sobel and Canny.
//!
//! Both operate on grayscale images and produce grayscale output.

#![allow(dead_code)]

use super::types::{Image, PixelFormat};

/// Sobel edge detection — returns gradient magnitude image.
///
/// Uses 3x3 Sobel kernels for horizontal and vertical gradients.
pub fn sobel(img: &Image) -> Image {
    assert_eq!(img.format, PixelFormat::Gray, "input must be grayscale");
    let w = img.width as usize;
    let h = img.height as usize;
    let mut out = Image::new(img.width, img.height, PixelFormat::Gray);

    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let (gx, gy) = sobel_at(img, w, x, y);
            let mag = ((gx * gx + gy * gy) as f64).sqrt().min(255.0) as u8;
            out.data[y * w + x] = mag;
        }
    }
    out
}

/// Compute Sobel gradients at a single pixel, returning (gx, gy) as i32.
fn sobel_at(img: &Image, w: usize, x: usize, y: usize) -> (i32, i32) {
    let p = |dx: usize, dy: usize| img.data[dy * w + dx] as i32;

    let gx = -p(x - 1, y - 1) + p(x + 1, y - 1)
        - 2 * p(x - 1, y) + 2 * p(x + 1, y)
        - p(x - 1, y + 1) + p(x + 1, y + 1);

    let gy = -p(x - 1, y - 1) - 2 * p(x, y - 1) - p(x + 1, y - 1)
        + p(x - 1, y + 1) + 2 * p(x, y + 1) + p(x + 1, y + 1);

    (gx, gy)
}

/// Canny edge detection.
///
/// Full pipeline: Gaussian blur -> Sobel gradients -> non-maximum suppression
/// -> double threshold -> hysteresis edge tracking.
///
/// * `low_thresh`  — weak-edge threshold (0-255)
/// * `high_thresh` — strong-edge threshold (0-255)
pub fn canny(img: &Image, low_thresh: u8, high_thresh: u8) -> Image {
    assert_eq!(img.format, PixelFormat::Gray, "input must be grayscale");
    let w = img.width as usize;
    let h = img.height as usize;

    // Step 1: Gaussian blur (5x5)
    let blurred = gaussian_blur_5x5(img);

    // Step 2: Sobel gradients — magnitude + direction
    let mut magnitude = vec![0.0f64; w * h];
    let mut direction = vec![0.0f64; w * h];

    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let (gx, gy) = sobel_at(&blurred, w, x, y);
            magnitude[y * w + x] = ((gx * gx + gy * gy) as f64).sqrt();
            direction[y * w + x] = (gy as f64).atan2(gx as f64);
        }
    }

    // Step 3: Non-maximum suppression
    let mut nms = vec![0.0f64; w * h];
    for y in 2..h.saturating_sub(2) {
        for x in 2..w.saturating_sub(2) {
            let angle = direction[y * w + x];
            let mag = magnitude[y * w + x];
            let (n1, n2) = neighbors_along_gradient(
                &magnitude, w, x, y, angle,
            );
            nms[y * w + x] = if mag >= n1 && mag >= n2 { mag } else { 0.0 };
        }
    }

    // Step 4: Double threshold
    let low = low_thresh as f64;
    let high = high_thresh as f64;
    let mut edges = vec![0u8; w * h]; // 0=none, 128=weak, 255=strong
    for y in 0..h {
        for x in 0..w {
            let v = nms[y * w + x];
            edges[y * w + x] = if v >= high {
                255
            } else if v >= low {
                128
            } else {
                0
            };
        }
    }

    // Step 5: Hysteresis — promote weak edges connected to strong edges
    let mut changed = true;
    while changed {
        changed = false;
        for y in 1..h.saturating_sub(1) {
            for x in 1..w.saturating_sub(1) {
                if edges[y * w + x] == 128 {
                    let has_strong = neighbors_8(w, x, y)
                        .iter()
                        .any(|&(nx, ny)| edges[ny * w + nx] == 255);
                    if has_strong {
                        edges[y * w + x] = 255;
                        changed = true;
                    }
                }
            }
        }
    }

    // Suppress remaining weak edges
    for v in &mut edges {
        if *v != 255 {
            *v = 0;
        }
    }

    Image::from_raw(edges, img.width, img.height, PixelFormat::Gray)
}

/// 5x5 Gaussian blur for Canny preprocessing.
fn gaussian_blur_5x5(img: &Image) -> Image {
    let w = img.width as usize;
    let h = img.height as usize;
    #[rustfmt::skip]
    let kernel: [f64; 25] = [
        2.0, 4.0,  5.0,  4.0,  2.0,
        4.0, 9.0,  12.0, 9.0,  4.0,
        5.0, 12.0, 15.0, 12.0, 5.0,
        4.0, 9.0,  12.0, 9.0,  4.0,
        2.0, 4.0,  5.0,  4.0,  2.0,
    ];
    let sum: f64 = kernel.iter().sum(); // 159

    let mut out = Image::new(img.width, img.height, PixelFormat::Gray);
    for y in 2..h.saturating_sub(2) {
        for x in 2..w.saturating_sub(2) {
            let mut acc = 0.0;
            for ky in 0..5usize {
                for kx in 0..5usize {
                    acc += img.data[(y + ky - 2) * w + (x + kx - 2)] as f64
                        * kernel[ky * 5 + kx];
                }
            }
            out.data[y * w + x] = (acc / sum).clamp(0.0, 255.0) as u8;
        }
    }
    out
}

/// Get the two neighbor magnitudes along the gradient direction.
fn neighbors_along_gradient(
    mag: &[f64],
    w: usize,
    x: usize,
    y: usize,
    angle: f64,
) -> (f64, f64) {
    // Quantize angle to 0, 45, 90, 135 degrees
    let deg = angle.to_degrees();
    let deg = if deg < 0.0 { deg + 180.0 } else { deg };

    let (dx1, dy1, dx2, dy2) = if deg < 22.5 || deg >= 157.5 {
        // Horizontal edge
        (1i32, 0i32, -1i32, 0i32)
    } else if deg < 67.5 {
        // 45-degree edge
        (1, 1, -1, -1)
    } else if deg < 112.5 {
        // Vertical edge
        (0, 1, 0, -1)
    } else {
        // 135-degree edge
        (-1, 1, 1, -1)
    };

    let n1x = (x as i32 + dx1) as usize;
    let n1y = (y as i32 + dy1) as usize;
    let n2x = (x as i32 + dx2) as usize;
    let n2y = (y as i32 + dy2) as usize;

    (mag[n1y * w + n1x], mag[n2y * w + n2x])
}

/// 8-connected neighbor coordinates.
fn neighbors_8(_w: usize, x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::with_capacity(8);
    for dy in [-1i32, 0, 1] {
        for dx in [-1i32, 0, 1] {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && ny >= 0 {
                result.push((nx as usize, ny as usize));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_edge_image() -> Image {
        // 16x16 image with a vertical edge at column 8
        let mut img = Image::new(16, 16, PixelFormat::Gray);
        for y in 0..16u32 {
            for x in 0..16u32 {
                let v = if x < 8 { 0u8 } else { 255u8 };
                img.set_pixel(x, y, &[v]);
            }
        }
        img
    }

    #[test]
    fn test_sobel_detects_edge() {
        let img = make_edge_image();
        let edges = sobel(&img);
        assert_eq!(edges.width, 16);
        assert_eq!(edges.height, 16);
        // Edge should be detected near column 7-8
        let mid = edges.data[8 * 16 + 8];
        let flat = edges.data[8 * 16 + 2];
        assert!(mid > flat, "edge pixel ({mid}) should be brighter than flat ({flat})");
    }

    #[test]
    fn test_canny_detects_edge() {
        let img = make_edge_image();
        let edges = canny(&img, 50, 150);
        // Should have some 255-valued pixels along the edge
        let strong_count = edges.data.iter().filter(|&&v| v == 255).count();
        assert!(strong_count > 0, "canny should detect at least one strong edge pixel");
    }

    #[test]
    fn test_canny_uniform_image() {
        // Uniform image should have no edges in the interior
        // (border pixels may have artifacts from the Gaussian blur boundary)
        let img = Image::from_raw(vec![128; 32 * 32], 32, 32, PixelFormat::Gray);
        let edges = canny(&img, 50, 150);
        // Check interior only (skip 4-pixel border for 5x5 Gaussian)
        let mut interior_strong = 0;
        for y in 5..27 {
            for x in 5..27 {
                if edges.data[y * 32 + x] == 255 {
                    interior_strong += 1;
                }
            }
        }
        assert_eq!(interior_strong, 0, "uniform image interior should have zero edges");
    }

    #[test]
    fn test_sobel_uniform_image() {
        let img = Image::from_raw(vec![100; 8 * 8], 8, 8, PixelFormat::Gray);
        let edges = sobel(&img);
        // All interior pixels should be 0
        for y in 1..7 {
            for x in 1..7 {
                assert_eq!(edges.data[y * 8 + x], 0);
            }
        }
    }
}
