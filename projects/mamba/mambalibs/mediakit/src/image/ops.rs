//! Basic image operations: resize, crop, flip, rotate, color conversion.

use super::types::{Image, PixelFormat};

/// Resize an image using nearest-neighbor interpolation.
pub fn resize(img: &Image, new_width: u32, new_height: u32) -> Image {
    let ch = img.format.channels();
    let mut out = Image::new(new_width, new_height, img.format);

    for y in 0..new_height {
        for x in 0..new_width {
            let src_x = (x as f64 * img.width as f64 / new_width as f64) as u32;
            let src_y = (y as f64 * img.height as f64 / new_height as f64) as u32;
            let src_x = src_x.min(img.width - 1);
            let src_y = src_y.min(img.height - 1);

            let src_idx = (src_y as usize * img.width as usize + src_x as usize) * ch;
            let dst_idx = (y as usize * new_width as usize + x as usize) * ch;
            out.data[dst_idx..dst_idx + ch].copy_from_slice(&img.data[src_idx..src_idx + ch]);
        }
    }
    out
}

/// Crop a rectangular region from an image.
pub fn crop(img: &Image, x: u32, y: u32, w: u32, h: u32) -> Image {
    let ch = img.format.channels();
    let mut out = Image::new(w, h, img.format);

    for dy in 0..h {
        for dx in 0..w {
            let src_x = (x + dx).min(img.width - 1);
            let src_y = (y + dy).min(img.height - 1);
            let src_idx = (src_y as usize * img.width as usize + src_x as usize) * ch;
            let dst_idx = (dy as usize * w as usize + dx as usize) * ch;
            out.data[dst_idx..dst_idx + ch].copy_from_slice(&img.data[src_idx..src_idx + ch]);
        }
    }
    out
}

/// Flip image horizontally (mirror).
pub fn flip_horizontal(img: &Image) -> Image {
    let ch = img.format.channels();
    let mut out = Image::new(img.width, img.height, img.format);

    for y in 0..img.height {
        for x in 0..img.width {
            let src = img.get_pixel(img.width - 1 - x, y);
            let dst_idx = (y as usize * img.width as usize + x as usize) * ch;
            out.data[dst_idx..dst_idx + ch].copy_from_slice(src);
        }
    }
    out
}

/// Flip image vertically.
pub fn flip_vertical(img: &Image) -> Image {
    let ch = img.format.channels();
    let mut out = Image::new(img.width, img.height, img.format);

    for y in 0..img.height {
        for x in 0..img.width {
            let src = img.get_pixel(x, img.height - 1 - y);
            let dst_idx = (y as usize * img.width as usize + x as usize) * ch;
            out.data[dst_idx..dst_idx + ch].copy_from_slice(src);
        }
    }
    out
}

/// Rotate image 90 degrees clockwise.
pub fn rotate90(img: &Image) -> Image {
    let ch = img.format.channels();
    let new_w = img.height;
    let new_h = img.width;
    let mut out = Image::new(new_w, new_h, img.format);

    for y in 0..img.height {
        for x in 0..img.width {
            let src = img.get_pixel(x, y);
            let new_x = img.height - 1 - y;
            let new_y = x;
            let dst_idx = (new_y as usize * new_w as usize + new_x as usize) * ch;
            out.data[dst_idx..dst_idx + ch].copy_from_slice(src);
        }
    }
    out
}

/// Resize an image using bilinear interpolation.
pub fn resize_bilinear(img: &Image, new_width: u32, new_height: u32) -> Image {
    let ch = img.format.channels();
    let mut out = Image::new(new_width, new_height, img.format);
    let w = img.width as f64;
    let h = img.height as f64;

    for y in 0..new_height {
        for x in 0..new_width {
            let src_x = x as f64 * (w - 1.0) / (new_width.max(1) - 1).max(1) as f64;
            let src_y = y as f64 * (h - 1.0) / (new_height.max(1) - 1).max(1) as f64;

            let x0 = src_x.floor() as u32;
            let y0 = src_y.floor() as u32;
            let x1 = (x0 + 1).min(img.width - 1);
            let y1 = (y0 + 1).min(img.height - 1);

            let fx = src_x - x0 as f64;
            let fy = src_y - y0 as f64;

            let dst_idx = (y as usize * new_width as usize + x as usize) * ch;
            for c in 0..ch {
                let p00 =
                    img.data[(y0 as usize * img.width as usize + x0 as usize) * ch + c] as f64;
                let p10 =
                    img.data[(y0 as usize * img.width as usize + x1 as usize) * ch + c] as f64;
                let p01 =
                    img.data[(y1 as usize * img.width as usize + x0 as usize) * ch + c] as f64;
                let p11 =
                    img.data[(y1 as usize * img.width as usize + x1 as usize) * ch + c] as f64;

                let top = p00 * (1.0 - fx) + p10 * fx;
                let bot = p01 * (1.0 - fx) + p11 * fx;
                let val = top * (1.0 - fy) + bot * fy;
                out.data[dst_idx + c] = val.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

/// Resize an image using bicubic interpolation.
pub fn resize_bicubic(img: &Image, new_width: u32, new_height: u32) -> Image {
    let ch = img.format.channels();
    let mut out = Image::new(new_width, new_height, img.format);
    let w = img.width as f64;
    let h = img.height as f64;
    let iw = img.width as usize;

    for y in 0..new_height {
        for x in 0..new_width {
            let src_x = x as f64 * (w - 1.0) / (new_width.max(1) - 1).max(1) as f64;
            let src_y = y as f64 * (h - 1.0) / (new_height.max(1) - 1).max(1) as f64;

            let ix = src_x.floor() as i32;
            let iy = src_y.floor() as i32;
            let fx = src_x - ix as f64;
            let fy = src_y - iy as f64;

            let dst_idx = (y as usize * new_width as usize + x as usize) * ch;
            for c in 0..ch {
                let mut val = 0.0;
                for m in -1..=2i32 {
                    let wy = cubic_weight(fy - m as f64);
                    let py = (iy + m).max(0).min(img.height as i32 - 1) as usize;
                    for n in -1..=2i32 {
                        let wx = cubic_weight(fx - n as f64);
                        let px = (ix + n).max(0).min(img.width as i32 - 1) as usize;
                        val += img.data[(py * iw + px) * ch + c] as f64 * wx * wy;
                    }
                }
                out.data[dst_idx + c] = val.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    out
}

/// Bicubic interpolation weight (Catmull-Rom, a = -0.5).
fn cubic_weight(t: f64) -> f64 {
    let t = t.abs();
    if t <= 1.0 {
        (1.5 * t - 2.5) * t * t + 1.0
    } else if t <= 2.0 {
        ((-0.5 * t + 2.5) * t - 4.0) * t + 2.0
    } else {
        0.0
    }
}

/// Convert RGB image to grayscale (luminance).
pub fn rgb_to_grayscale(img: &Image) -> Image {
    assert_eq!(img.format, PixelFormat::Rgb, "input must be RGB");
    let mut out = Image::new(img.width, img.height, PixelFormat::Gray);

    for y in 0..img.height {
        for x in 0..img.width {
            let px = img.get_pixel(x, y);
            let gray = (0.299 * px[0] as f64 + 0.587 * px[1] as f64 + 0.114 * px[2] as f64) as u8;
            out.set_pixel(x, y, &[gray]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_image() -> Image {
        let mut img = Image::new(4, 4, PixelFormat::Rgb);
        for y in 0..4 {
            for x in 0..4 {
                let v = (y * 4 + x) as u8 * 16;
                img.set_pixel(x, y, &[v, v, v]);
            }
        }
        img
    }

    #[test]
    fn test_resize() {
        let img = make_test_image();
        let resized = resize(&img, 8, 8);
        assert_eq!(resized.width, 8);
        assert_eq!(resized.height, 8);
        assert_eq!(resized.data.len(), 8 * 8 * 3);
    }

    #[test]
    fn test_crop() {
        let img = make_test_image();
        let cropped = crop(&img, 1, 1, 2, 2);
        assert_eq!(cropped.width, 2);
        assert_eq!(cropped.height, 2);
    }

    #[test]
    fn test_flip_horizontal() {
        let mut img = Image::new(3, 1, PixelFormat::Gray);
        img.data = vec![1, 2, 3];
        let flipped = flip_horizontal(&img);
        assert_eq!(flipped.data, vec![3, 2, 1]);
    }

    #[test]
    fn test_flip_vertical() {
        let mut img = Image::new(1, 3, PixelFormat::Gray);
        img.data = vec![1, 2, 3];
        let flipped = flip_vertical(&img);
        assert_eq!(flipped.data, vec![3, 2, 1]);
    }

    #[test]
    fn test_rotate90() {
        let mut img = Image::new(2, 3, PixelFormat::Gray);
        img.data = vec![1, 2, 3, 4, 5, 6];
        let rotated = rotate90(&img);
        assert_eq!(rotated.width, 3);
        assert_eq!(rotated.height, 2);
        assert_eq!(rotated.data, vec![5, 3, 1, 6, 4, 2]);
    }

    #[test]
    fn test_rgb_to_grayscale() {
        let mut img = Image::new(2, 1, PixelFormat::Rgb);
        img.set_pixel(0, 0, &[255, 255, 255]);
        img.set_pixel(1, 0, &[0, 0, 0]);
        let gray = rgb_to_grayscale(&img);
        assert_eq!(gray.format, PixelFormat::Gray);
        assert!(gray.get_pixel(0, 0)[0] >= 254); // ~255 * (0.299+0.587+0.114)
        assert_eq!(gray.get_pixel(1, 0), &[0]);
    }
}
