//! Video reader — frame extraction from raw video data.
//!
//! This module provides a frame-by-frame reader. In the current implementation,
//! it works with raw uncompressed video data. When the `ffmpeg` feature is enabled,
//! it will use ffmpeg-next for codec support.

use super::error::{Result, VideoError};
use super::types::{Frame, VideoMeta};

/// Configuration for video reading.
#[derive(Debug, Clone)]
pub struct ReaderConfig {
    /// Extract every N-th frame (1 = all frames).
    pub frame_step: usize,
    /// Start reading from this timestamp (seconds).
    pub start_time: f64,
    /// Stop reading at this timestamp (None = until end).
    pub end_time: Option<f64>,
    /// Resize frames to this width (None = original).
    pub resize_width: Option<usize>,
    /// Resize frames to this height (None = original).
    pub resize_height: Option<usize>,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self {
            frame_step: 1,
            start_time: 0.0,
            end_time: None,
            resize_width: None,
            resize_height: None,
        }
    }
}

/// Video reader that iterates over frames.
///
/// Currently supports raw uncompressed video data (RGB, packed).
/// Codec support requires the `ffmpeg` feature (wrapping ffmpeg-next).
pub struct VideoReader {
    data: Vec<u8>,
    meta: VideoMeta,
    config: ReaderConfig,
    current_frame: usize,
}

impl VideoReader {
    /// Create a video reader from raw RGB frame data.
    ///
    /// `data` contains all frames packed sequentially (each frame = width * height * 3 bytes).
    pub fn from_raw(
        data: Vec<u8>,
        width: usize,
        height: usize,
        fps: f64,
    ) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(VideoError::InvalidParameter(
                "width and height must be > 0".into(),
            ));
        }
        if fps <= 0.0 {
            return Err(VideoError::InvalidParameter("fps must be > 0".into()));
        }

        let frame_size = width * height * 3;
        let frame_count = if frame_size > 0 {
            data.len() / frame_size
        } else {
            0
        };
        let duration = frame_count as f64 / fps;

        Ok(Self {
            data,
            meta: VideoMeta {
                width,
                height,
                fps,
                frame_count,
                duration,
                codec: "raw".to_string(),
            },
            config: ReaderConfig::default(),
            current_frame: 0,
        })
    }

    /// Set reader configuration.
    pub fn with_config(mut self, config: ReaderConfig) -> Self {
        self.config = config;
        self
    }

    /// Get video metadata.
    pub fn meta(&self) -> &VideoMeta {
        &self.meta
    }

    /// Get total frame count.
    pub fn frame_count(&self) -> usize {
        self.meta.frame_count
    }

    /// Get duration in seconds.
    pub fn duration(&self) -> f64 {
        self.meta.duration
    }

    /// Get frames per second.
    pub fn fps(&self) -> f64 {
        self.meta.fps
    }

    /// Seek to a specific frame index.
    pub fn seek(&mut self, frame_index: usize) -> Result<()> {
        if frame_index >= self.meta.frame_count {
            return Err(VideoError::SeekOutOfRange(frame_index));
        }
        self.current_frame = frame_index;
        Ok(())
    }

    /// Seek to a specific timestamp (seconds).
    pub fn seek_time(&mut self, time: f64) -> Result<()> {
        let frame_index = (time * self.meta.fps) as usize;
        self.seek(frame_index)
    }

    /// Read the next frame.
    pub fn next_frame(&mut self) -> Result<Frame> {
        if self.current_frame >= self.meta.frame_count {
            return Err(VideoError::EndOfStream);
        }

        let frame_size = self.meta.width * self.meta.height * 3;
        let start = self.current_frame * frame_size;
        let end = start + frame_size;

        if end > self.data.len() {
            return Err(VideoError::EndOfStream);
        }

        let frame_data = self.data[start..end].to_vec();
        let timestamp = self.current_frame as f64 / self.meta.fps;
        let index = self.current_frame;

        let mut frame = Frame::new(frame_data, self.meta.width, self.meta.height, timestamp, index);

        // Apply resize if configured
        if let (Some(w), Some(h)) = (self.config.resize_width, self.config.resize_height) {
            frame = resize_frame(&frame, w, h);
        }

        self.current_frame += self.config.frame_step;
        Ok(frame)
    }

    /// Extract frames at a specific fps rate.
    pub fn extract_frames(&mut self, target_fps: f64) -> Result<Vec<Frame>> {
        let step = (self.meta.fps / target_fps).max(1.0) as usize;
        let mut frames = Vec::new();

        self.current_frame = 0;
        while self.current_frame < self.meta.frame_count {
            match self.next_frame() {
                Ok(frame) => frames.push(frame),
                Err(VideoError::EndOfStream) => break,
                Err(e) => return Err(e),
            }
            // Adjust for step (next_frame already advances by frame_step)
            self.current_frame =
                self.current_frame.saturating_sub(self.config.frame_step) + step;
        }

        Ok(frames)
    }

    /// Get all frames as an iterator.
    pub fn frames(self) -> FrameIterator {
        FrameIterator { reader: self }
    }
}

/// Iterator over video frames.
pub struct FrameIterator {
    reader: VideoReader,
}

impl Iterator for FrameIterator {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.next_frame().ok()
    }
}

/// Simple nearest-neighbor frame resize.
fn resize_frame(frame: &Frame, new_w: usize, new_h: usize) -> Frame {
    let mut data = vec![0u8; new_w * new_h * 3];
    for y in 0..new_h {
        for x in 0..new_w {
            let src_x = (x * frame.width) / new_w;
            let src_y = (y * frame.height) / new_h;
            let src_idx = (src_y * frame.width + src_x) * 3;
            let dst_idx = (y * new_w + x) * 3;
            data[dst_idx] = frame.data[src_idx];
            data[dst_idx + 1] = frame.data[src_idx + 1];
            data[dst_idx + 2] = frame.data[src_idx + 2];
        }
    }
    Frame::new(data, new_w, new_h, frame.timestamp, frame.index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_video(width: usize, height: usize, frames: usize) -> Vec<u8> {
        let frame_size = width * height * 3;
        let mut data = vec![0u8; frame_size * frames];
        for f in 0..frames {
            // Each frame has a unique gray value
            let val = ((f + 1) * 25).min(255) as u8;
            for i in 0..frame_size {
                data[f * frame_size + i] = val;
            }
        }
        data
    }

    #[test]
    fn test_reader_basic() {
        let data = make_test_video(4, 4, 10);
        let reader = VideoReader::from_raw(data, 4, 4, 30.0).unwrap();
        assert_eq!(reader.frame_count(), 10);
        assert!((reader.fps() - 30.0).abs() < 1e-10);
        assert!((reader.duration() - 10.0 / 30.0).abs() < 0.01);
    }

    #[test]
    fn test_reader_next_frame() {
        let data = make_test_video(2, 2, 3);
        let mut reader = VideoReader::from_raw(data, 2, 2, 10.0).unwrap();
        let f1 = reader.next_frame().unwrap();
        assert_eq!(f1.index, 0);
        assert_eq!(f1.data[0], 25);
        let f2 = reader.next_frame().unwrap();
        assert_eq!(f2.index, 1);
        assert_eq!(f2.data[0], 50);
    }

    #[test]
    fn test_reader_seek() {
        let data = make_test_video(2, 2, 5);
        let mut reader = VideoReader::from_raw(data, 2, 2, 10.0).unwrap();
        reader.seek(3).unwrap();
        let frame = reader.next_frame().unwrap();
        assert_eq!(frame.index, 3);
        assert_eq!(frame.data[0], 100);
    }

    #[test]
    fn test_reader_end_of_stream() {
        let data = make_test_video(2, 2, 1);
        let mut reader = VideoReader::from_raw(data, 2, 2, 10.0).unwrap();
        assert!(reader.next_frame().is_ok());
        assert!(reader.next_frame().is_err());
    }

    #[test]
    fn test_frame_iterator() {
        let data = make_test_video(2, 2, 4);
        let reader = VideoReader::from_raw(data, 2, 2, 10.0).unwrap();
        let frames: Vec<_> = reader.frames().collect();
        assert_eq!(frames.len(), 4);
    }

    #[test]
    fn test_extract_frames() {
        let data = make_test_video(2, 2, 30);
        let mut reader = VideoReader::from_raw(data, 2, 2, 30.0).unwrap();
        // Extract at 10fps from 30fps video -> every 3rd frame
        let frames = reader.extract_frames(10.0).unwrap();
        assert_eq!(frames.len(), 10);
    }
}
