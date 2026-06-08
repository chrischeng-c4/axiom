//! Core image types.

/// Pixel format.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelFormat {
    Gray,
    Rgb,
    Rgba,
    Hsv,
    Lab,
}

impl PixelFormat {
    pub fn channels(&self) -> usize {
        match self {
            PixelFormat::Gray => 1,
            PixelFormat::Rgb | PixelFormat::Hsv | PixelFormat::Lab => 3,
            PixelFormat::Rgba => 4,
        }
    }
}

/// A simple image stored as raw bytes.
#[derive(Debug, Clone)]
pub struct Image {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
}

impl Image {
    /// Create a new image filled with zeros.
    pub fn new(width: u32, height: u32, format: PixelFormat) -> Self {
        let size = width as usize * height as usize * format.channels();
        Self {
            data: vec![0u8; size],
            width,
            height,
            format,
        }
    }

    /// Create an image from raw data.
    pub fn from_raw(data: Vec<u8>, width: u32, height: u32, format: PixelFormat) -> Self {
        Self {
            data,
            width,
            height,
            format,
        }
    }

    /// Get pixel value at (x, y).
    pub fn get_pixel(&self, x: u32, y: u32) -> &[u8] {
        let ch = self.format.channels();
        let idx = (y as usize * self.width as usize + x as usize) * ch;
        &self.data[idx..idx + ch]
    }

    /// Set pixel value at (x, y).
    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: &[u8]) {
        let ch = self.format.channels();
        let idx = (y as usize * self.width as usize + x as usize) * ch;
        self.data[idx..idx + ch].copy_from_slice(pixel);
    }

    /// Total number of pixels.
    pub fn n_pixels(&self) -> usize {
        self.width as usize * self.height as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_new() {
        let img = Image::new(10, 20, PixelFormat::Rgb);
        assert_eq!(img.data.len(), 10 * 20 * 3);
        assert_eq!(img.width, 10);
        assert_eq!(img.height, 20);
    }

    #[test]
    fn test_pixel_access() {
        let mut img = Image::new(5, 5, PixelFormat::Rgb);
        img.set_pixel(2, 3, &[255, 128, 64]);
        let px = img.get_pixel(2, 3);
        assert_eq!(px, &[255, 128, 64]);
    }
}
