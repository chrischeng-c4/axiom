//! Histogram operations: computation, equalization, CLAHE.

use super::types::{Image, PixelFormat};

/// Compute the intensity histogram of a grayscale image.
///
/// Returns an array of 256 bin counts.
pub fn compute_histogram(img: &Image) -> [u32; 256] {
    assert_eq!(img.format, PixelFormat::Gray, "input must be grayscale");
    let mut hist = [0u32; 256];
    for &v in &img.data {
        hist[v as usize] += 1;
    }
    hist
}

/// Histogram equalization for a grayscale image.
///
/// Redistributes pixel intensities so the output histogram is approximately
/// uniform, enhancing overall contrast.
pub fn equalize(img: &Image) -> Image {
    assert_eq!(img.format, PixelFormat::Gray, "input must be grayscale");
    let n = img.data.len() as u32;
    if n == 0 {
        return img.clone();
    }

    let hist = compute_histogram(img);

    // Cumulative distribution function
    let mut cdf = [0u32; 256];
    cdf[0] = hist[0];
    for i in 1..256 {
        cdf[i] = cdf[i - 1] + hist[i];
    }

    // Find minimum non-zero CDF value
    let cdf_min = cdf.iter().copied().find(|&v| v > 0).unwrap_or(0);
    let denom = n - cdf_min;

    let mut out = img.clone();
    if denom == 0 {
        return out;
    }

    for v in &mut out.data {
        let mapped = ((cdf[*v as usize] - cdf_min) as f64 / denom as f64 * 255.0).round();
        *v = mapped.clamp(0.0, 255.0) as u8;
    }
    out
}

/// Contrast-Limited Adaptive Histogram Equalization (CLAHE).
///
/// Divides the image into `tile_w x tile_h` tiles and applies histogram
/// equalization locally with a clip limit, then bilinearly interpolates
/// between tiles to avoid blocking artifacts.
///
/// * `tile_w` — tile width in pixels (typical: 8)
/// * `tile_h` — tile height in pixels (typical: 8)
/// * `clip_limit` — histogram bin clip limit (typical: 2.0-4.0)
pub fn clahe(img: &Image, tile_w: usize, tile_h: usize, clip_limit: f64) -> Image {
    assert_eq!(img.format, PixelFormat::Gray, "input must be grayscale");
    let w = img.width as usize;
    let h = img.height as usize;

    let tiles_x = (w + tile_w - 1) / tile_w;
    let tiles_y = (h + tile_h - 1) / tile_h;

    // Compute clipped+redistributed CDF for each tile
    let mut tile_cdfs: Vec<Vec<[f64; 256]>> = Vec::with_capacity(tiles_y);

    for ty in 0..tiles_y {
        let mut row_cdfs = Vec::with_capacity(tiles_x);
        for tx in 0..tiles_x {
            let x0 = tx * tile_w;
            let y0 = ty * tile_h;
            let x1 = (x0 + tile_w).min(w);
            let y1 = (y0 + tile_h).min(h);
            let n_pix = (x1 - x0) * (y1 - y0);

            // Local histogram
            let mut hist = [0u32; 256];
            for py in y0..y1 {
                for px in x0..x1 {
                    hist[img.data[py * w + px] as usize] += 1;
                }
            }

            // Clip histogram and redistribute
            let clip = (clip_limit * n_pix as f64 / 256.0).max(1.0) as u32;
            let mut excess = 0u32;
            for bin in &mut hist {
                if *bin > clip {
                    excess += *bin - clip;
                    *bin = clip;
                }
            }
            let per_bin = excess / 256;
            let remainder = (excess % 256) as usize;
            for (i, bin) in hist.iter_mut().enumerate() {
                *bin += per_bin;
                if i < remainder {
                    *bin += 1;
                }
            }

            // CDF (normalized to 0.0-255.0)
            let mut cdf = [0f64; 256];
            cdf[0] = hist[0] as f64;
            for i in 1..256 {
                cdf[i] = cdf[i - 1] + hist[i] as f64;
            }
            let total = cdf[255];
            if total > 0.0 {
                for v in &mut cdf {
                    *v = *v / total * 255.0;
                }
            }
            row_cdfs.push(cdf);
        }
        tile_cdfs.push(row_cdfs);
    }

    // Bilinear interpolation between tile CDFs
    let mut out = Image::new(img.width, img.height, PixelFormat::Gray);

    for y in 0..h {
        for x in 0..w {
            let val = img.data[y * w + x] as usize;

            // Tile center coordinates
            let tcx = (x as f64 + 0.5) / tile_w as f64 - 0.5;
            let tcy = (y as f64 + 0.5) / tile_h as f64 - 0.5;

            let tx0 = (tcx.floor() as isize).max(0) as usize;
            let ty0 = (tcy.floor() as isize).max(0) as usize;
            let tx1 = (tx0 + 1).min(tiles_x - 1);
            let ty1 = (ty0 + 1).min(tiles_y - 1);

            let fx = (tcx - tx0 as f64).clamp(0.0, 1.0);
            let fy = (tcy - ty0 as f64).clamp(0.0, 1.0);

            let c00 = tile_cdfs[ty0][tx0][val];
            let c10 = tile_cdfs[ty0][tx1][val];
            let c01 = tile_cdfs[ty1][tx0][val];
            let c11 = tile_cdfs[ty1][tx1][val];

            let top = c00 * (1.0 - fx) + c10 * fx;
            let bot = c01 * (1.0 - fx) + c11 * fx;
            let result = top * (1.0 - fy) + bot * fy;

            out.data[y * w + x] = result.round().clamp(0.0, 255.0) as u8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_histogram() {
        let img = Image::from_raw(vec![0, 0, 0, 128, 128, 255], 6, 1, PixelFormat::Gray);
        let hist = compute_histogram(&img);
        assert_eq!(hist[0], 3);
        assert_eq!(hist[128], 2);
        assert_eq!(hist[255], 1);
        assert_eq!(hist[1], 0);
    }

    #[test]
    fn test_equalize_low_contrast() {
        // All pixels in narrow range 100-110
        let data: Vec<u8> = (0..64).map(|i| 100 + (i % 11) as u8).collect();
        let img = Image::from_raw(data, 8, 8, PixelFormat::Gray);
        let eq = equalize(&img);
        // After equalization, range should be wider
        let min_v = *eq.data.iter().min().unwrap();
        let max_v = *eq.data.iter().max().unwrap();
        assert!(max_v - min_v > 100, "equalization should spread range");
    }

    #[test]
    fn test_equalize_single_value() {
        let img = Image::from_raw(vec![42; 16], 4, 4, PixelFormat::Gray);
        let eq = equalize(&img);
        // Single-value image: all pixels map to same value
        let first = eq.data[0];
        assert!(eq.data.iter().all(|&v| v == first));
    }

    #[test]
    fn test_clahe_basic() {
        // Gradient image
        let data: Vec<u8> = (0..256).map(|i| i as u8).collect();
        let img = Image::from_raw(data, 16, 16, PixelFormat::Gray);
        let result = clahe(&img, 8, 8, 2.0);
        assert_eq!(result.width, 16);
        assert_eq!(result.height, 16);
        assert_eq!(result.data.len(), 256);
    }

    #[test]
    fn test_clahe_preserves_dimensions() {
        let img = Image::new(32, 32, PixelFormat::Gray);
        let result = clahe(&img, 8, 8, 3.0);
        assert_eq!(result.width, 32);
        assert_eq!(result.height, 32);
    }
}
