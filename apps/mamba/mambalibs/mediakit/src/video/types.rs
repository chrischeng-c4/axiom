//! Video types and metadata.

/// Video metadata.
#[derive(Debug, Clone)]
pub struct VideoMeta {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// Frames per second.
    pub fps: f64,
    /// Total frame count (0 if unknown).
    pub frame_count: usize,
    /// Duration in seconds.
    pub duration: f64,
    /// Codec name.
    pub codec: String,
}

/// A single video frame as raw pixel data (RGB, HWC layout).
#[derive(Debug, Clone)]
pub struct Frame {
    /// Pixel data, RGB format, row-major.
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// Timestamp in seconds.
    pub timestamp: f64,
    /// Frame index.
    pub index: usize,
}

impl Frame {
    /// Create a new frame from raw RGB data.
    pub fn new(data: Vec<u8>, width: usize, height: usize, timestamp: f64, index: usize) -> Self {
        Self {
            data,
            width,
            height,
            timestamp,
            index,
        }
    }

    /// Create a blank (black) frame.
    pub fn blank(width: usize, height: usize) -> Self {
        Self {
            data: vec![0u8; width * height * 3],
            width,
            height,
            timestamp: 0.0,
            index: 0,
        }
    }

    /// Get pixel at (x, y) as (R, G, B).
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<(u8, u8, u8)> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) * 3;
        Some((self.data[idx], self.data[idx + 1], self.data[idx + 2]))
    }

    /// Set pixel at (x, y) to (R, G, B).
    pub fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * 3;
            self.data[idx] = r;
            self.data[idx + 1] = g;
            self.data[idx + 2] = b;
        }
    }

    /// Number of bytes per row.
    pub fn stride(&self) -> usize {
        self.width * 3
    }
}

/// Video codec specification.
#[derive(Debug, Clone, PartialEq)]
pub enum Codec {
    H264,
    H265,
    Vp8,
    Vp9,
    Av1,
    Raw,
}

impl std::fmt::Display for Codec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Codec::H264 => write!(f, "h264"),
            Codec::H265 => write!(f, "h265"),
            Codec::Vp8 => write!(f, "vp8"),
            Codec::Vp9 => write!(f, "vp9"),
            Codec::Av1 => write!(f, "av1"),
            Codec::Raw => write!(f, "raw"),
        }
    }
}
