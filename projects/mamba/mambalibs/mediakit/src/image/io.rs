//! Image file I/O using the `image` crate.

use super::types::{Image, PixelFormat};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during image I/O.
#[derive(Debug, Error)]
pub enum ImageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("image decode/encode error: {0}")]
    ImageLib(#[from] ::image::ImageError),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// Read an image from a file path (supports PNG, JPEG, BMP, WebP).
///
/// The returned image will be in `Gray`, `Rgb`, or `Rgba` format depending
/// on the source file's color type.
pub fn imread(path: &str) -> Result<Image, ImageError> {
    use ::image::DynamicImage;

    let dyn_img = ::image::open(Path::new(path))?;

    let (data, width, height, format) = match dyn_img {
        DynamicImage::ImageLuma8(gray) => {
            let (w, h) = gray.dimensions();
            (gray.into_raw(), w, h, PixelFormat::Gray)
        }
        DynamicImage::ImageRgba8(rgba) => {
            let (w, h) = rgba.dimensions();
            (rgba.into_raw(), w, h, PixelFormat::Rgba)
        }
        other => {
            // Convert everything else to RGB8
            let rgb = other.into_rgb8();
            let (w, h) = rgb.dimensions();
            (rgb.into_raw(), w, h, PixelFormat::Rgb)
        }
    };

    Ok(Image::from_raw(data, width, height, format))
}

/// Decode an image from in-memory bytes (JPEG, PNG, BMP, WebP, GIF, etc.).
///
/// The format is auto-detected from the byte header.
pub fn imdecode(bytes: &[u8]) -> Result<Image, ImageError> {
    use ::image::DynamicImage;
    use std::io::Cursor;

    let reader = ::image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| ImageError::Io(e))?;

    let dyn_img = reader.decode()?;

    let (data, width, height, format) = match dyn_img {
        DynamicImage::ImageLuma8(gray) => {
            let (w, h) = gray.dimensions();
            (gray.into_raw(), w, h, PixelFormat::Gray)
        }
        DynamicImage::ImageRgba8(rgba) => {
            let (w, h) = rgba.dimensions();
            (rgba.into_raw(), w, h, PixelFormat::Rgba)
        }
        other => {
            let rgb = other.into_rgb8();
            let (w, h) = rgb.dimensions();
            (rgb.into_raw(), w, h, PixelFormat::Rgb)
        }
    };

    Ok(Image::from_raw(data, width, height, format))
}

/// Encode an image to in-memory bytes in the specified format.
///
/// Supported formats: `"png"`, `"jpeg"`/`"jpg"`, `"bmp"`, `"webp"`.
pub fn imencode(img: &Image, format: &str) -> Result<Vec<u8>, ImageError> {
    use ::image::DynamicImage;
    use std::io::Cursor;

    let dyn_img = match img.format {
        PixelFormat::Gray => {
            let buf = ::image::GrayImage::from_raw(img.width, img.height, img.data.clone())
                .ok_or_else(|| {
                    ImageError::UnsupportedFormat("failed to create gray image buffer".into())
                })?;
            DynamicImage::ImageLuma8(buf)
        }
        PixelFormat::Rgb => {
            let buf = ::image::RgbImage::from_raw(img.width, img.height, img.data.clone())
                .ok_or_else(|| {
                    ImageError::UnsupportedFormat("failed to create RGB image buffer".into())
                })?;
            DynamicImage::ImageRgb8(buf)
        }
        PixelFormat::Rgba => {
            let buf = ::image::RgbaImage::from_raw(img.width, img.height, img.data.clone())
                .ok_or_else(|| {
                    ImageError::UnsupportedFormat("failed to create RGBA image buffer".into())
                })?;
            DynamicImage::ImageRgba8(buf)
        }
        PixelFormat::Hsv | PixelFormat::Lab => {
            return Err(ImageError::UnsupportedFormat(
                "HSV/Lab images must be converted to RGB before encoding".into(),
            ));
        }
    };

    let img_fmt = match format.to_lowercase().as_str() {
        "png" => ::image::ImageFormat::Png,
        "jpeg" | "jpg" => ::image::ImageFormat::Jpeg,
        "bmp" => ::image::ImageFormat::Bmp,
        "webp" => ::image::ImageFormat::WebP,
        other => {
            return Err(ImageError::UnsupportedFormat(format!(
                "unsupported encode format: {other}"
            )));
        }
    };

    let mut buf = Cursor::new(Vec::new());
    dyn_img.write_to(&mut buf, img_fmt)?;
    Ok(buf.into_inner())
}

/// Write an image to a file path (format inferred from extension).
///
/// Supported extensions: `.png`, `.jpg`/`.jpeg`, `.bmp`, `.webp`.
pub fn imwrite(path: &str, img: &Image) -> Result<(), ImageError> {
    use ::image::DynamicImage;

    let dyn_img = match img.format {
        PixelFormat::Gray => {
            let buf = ::image::GrayImage::from_raw(img.width, img.height, img.data.clone())
                .ok_or_else(|| {
                    ImageError::UnsupportedFormat("failed to create gray image buffer".into())
                })?;
            DynamicImage::ImageLuma8(buf)
        }
        PixelFormat::Rgb => {
            let buf = ::image::RgbImage::from_raw(img.width, img.height, img.data.clone())
                .ok_or_else(|| {
                    ImageError::UnsupportedFormat("failed to create RGB image buffer".into())
                })?;
            DynamicImage::ImageRgb8(buf)
        }
        PixelFormat::Rgba => {
            let buf = ::image::RgbaImage::from_raw(img.width, img.height, img.data.clone())
                .ok_or_else(|| {
                    ImageError::UnsupportedFormat("failed to create RGBA image buffer".into())
                })?;
            DynamicImage::ImageRgba8(buf)
        }
        PixelFormat::Hsv | PixelFormat::Lab => {
            return Err(ImageError::UnsupportedFormat(
                "HSV/Lab images must be converted to RGB before saving".into(),
            ));
        }
    };

    dyn_img.save(Path::new(path))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_gray_png() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_gray.png");
        let path_str = path.to_str().unwrap();

        let mut img = Image::new(4, 3, PixelFormat::Gray);
        for (i, v) in img.data.iter_mut().enumerate() {
            *v = (i * 20) as u8;
        }

        imwrite(path_str, &img).unwrap();
        let loaded = imread(path_str).unwrap();

        assert_eq!(loaded.width, 4);
        assert_eq!(loaded.height, 3);
        assert_eq!(loaded.format, PixelFormat::Gray);
        assert_eq!(loaded.data, img.data);
    }

    #[test]
    fn test_roundtrip_rgb_png() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_rgb.png");
        let path_str = path.to_str().unwrap();

        let mut img = Image::new(3, 2, PixelFormat::Rgb);
        for (i, v) in img.data.iter_mut().enumerate() {
            *v = (i * 14) as u8;
        }

        imwrite(path_str, &img).unwrap();
        let loaded = imread(path_str).unwrap();

        assert_eq!(loaded.width, 3);
        assert_eq!(loaded.height, 2);
        assert_eq!(loaded.format, PixelFormat::Rgb);
        assert_eq!(loaded.data, img.data);
    }

    #[test]
    fn test_roundtrip_rgba_png() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_rgba.png");
        let path_str = path.to_str().unwrap();

        let mut img = Image::new(2, 2, PixelFormat::Rgba);
        img.set_pixel(0, 0, &[255, 0, 0, 255]);
        img.set_pixel(1, 0, &[0, 255, 0, 128]);
        img.set_pixel(0, 1, &[0, 0, 255, 64]);
        img.set_pixel(1, 1, &[100, 100, 100, 0]);

        imwrite(path_str, &img).unwrap();
        let loaded = imread(path_str).unwrap();

        assert_eq!(loaded.width, 2);
        assert_eq!(loaded.height, 2);
        assert_eq!(loaded.format, PixelFormat::Rgba);
        assert_eq!(loaded.data, img.data);
    }

    #[test]
    fn test_imread_nonexistent_file() {
        let result = imread("/tmp/nonexistent_pulsar_test_image.png");
        assert!(result.is_err());
    }

    #[test]
    fn test_imdecode_png_roundtrip() {
        let mut img = Image::new(4, 3, PixelFormat::Rgb);
        for (i, v) in img.data.iter_mut().enumerate() {
            *v = (i * 14) as u8;
        }

        let encoded = imencode(&img, "png").unwrap();
        let decoded = imdecode(&encoded).unwrap();

        assert_eq!(decoded.width, 4);
        assert_eq!(decoded.height, 3);
        assert_eq!(decoded.format, PixelFormat::Rgb);
        assert_eq!(decoded.data, img.data);
    }

    #[test]
    fn test_imdecode_jpeg_roundtrip() {
        // JPEG is lossy so we just check dimensions and format
        let img = Image::new(8, 6, PixelFormat::Rgb);
        let encoded = imencode(&img, "jpeg").unwrap();
        let decoded = imdecode(&encoded).unwrap();

        assert_eq!(decoded.width, 8);
        assert_eq!(decoded.height, 6);
        assert_eq!(decoded.format, PixelFormat::Rgb);
    }

    #[test]
    fn test_imdecode_invalid_bytes() {
        let result = imdecode(b"not an image");
        assert!(result.is_err());
    }

    #[test]
    fn test_imencode_unsupported_format() {
        let img = Image::new(2, 2, PixelFormat::Rgb);
        let result = imencode(&img, "tiff");
        assert!(result.is_err());
    }
}
