//! Video writer — frame-by-frame video creation.
//!
//! Currently outputs raw uncompressed video data.
//! Codec support requires the `ffmpeg` feature.

use super::error::{Result, VideoError};
use super::types::{Codec, Frame, VideoMeta};

/// Configuration for video writing.
#[derive(Debug, Clone)]
pub struct WriterConfig {
    /// Output width.
    pub width: usize,
    /// Output height.
    pub height: usize,
    /// Frames per second.
    pub fps: f64,
    /// Codec to use.
    pub codec: Codec,
}

/// Video writer that accumulates frames.
pub struct VideoWriter {
    config: WriterConfig,
    frames: Vec<Frame>,
}

impl VideoWriter {
    /// Create a new video writer.
    pub fn new(width: usize, height: usize, fps: f64) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(VideoError::InvalidParameter(
                "width and height must be > 0".into(),
            ));
        }
        if fps <= 0.0 {
            return Err(VideoError::InvalidParameter("fps must be > 0".into()));
        }

        Ok(Self {
            config: WriterConfig {
                width,
                height,
                fps,
                codec: Codec::Raw,
            },
            frames: Vec::new(),
        })
    }

    /// Set the codec.
    pub fn with_codec(mut self, codec: Codec) -> Self {
        self.config.codec = codec;
        self
    }

    /// Write a frame to the video.
    pub fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        let expected_size = self.config.width * self.config.height * 3;
        if frame.width != self.config.width || frame.height != self.config.height {
            return Err(VideoError::InvalidParameter(format!(
                "frame size {}x{} doesn't match writer {}x{}",
                frame.width, frame.height, self.config.width, self.config.height
            )));
        }
        if frame.data.len() != expected_size {
            return Err(VideoError::InvalidParameter(format!(
                "frame data length {} doesn't match expected {}",
                frame.data.len(),
                expected_size
            )));
        }

        self.frames.push(frame.clone());
        Ok(())
    }

    /// Get the number of frames written.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Get video metadata.
    pub fn meta(&self) -> VideoMeta {
        VideoMeta {
            width: self.config.width,
            height: self.config.height,
            fps: self.config.fps,
            frame_count: self.frames.len(),
            duration: self.frames.len() as f64 / self.config.fps,
            codec: self.config.codec.to_string(),
        }
    }

    /// Finalize and get the raw video data (all frames concatenated).
    pub fn finalize(self) -> Vec<u8> {
        let frame_size = self.config.width * self.config.height * 3;
        let mut data = Vec::with_capacity(frame_size * self.frames.len());
        for frame in &self.frames {
            data.extend_from_slice(&frame.data);
        }
        data
    }

    /// Finalize and get frames.
    pub fn into_frames(self) -> Vec<Frame> {
        self.frames
    }
}

/// Concatenate multiple video datas (same dimensions required).
pub fn concat_videos(videos: &[&[u8]], width: usize, height: usize) -> Result<Vec<u8>> {
    let frame_size = width * height * 3;
    let total: usize = videos.iter().map(|v| v.len()).sum();
    if videos.iter().any(|v| v.len() % frame_size != 0) {
        return Err(VideoError::InvalidParameter(
            "video data not aligned to frame size".into(),
        ));
    }
    let mut result = Vec::with_capacity(total);
    for video in videos {
        result.extend_from_slice(video);
    }
    Ok(result)
}

/// Clip a video to a time range.
pub fn clip_video(
    data: &[u8],
    width: usize,
    height: usize,
    fps: f64,
    start: f64,
    end: f64,
) -> Result<Vec<u8>> {
    let frame_size = width * height * 3;
    let total_frames = data.len() / frame_size;
    let start_frame = (start * fps) as usize;
    let end_frame = ((end * fps) as usize).min(total_frames);

    if start_frame >= end_frame {
        return Err(VideoError::InvalidParameter(
            "start must be before end".into(),
        ));
    }

    let start_byte = start_frame * frame_size;
    let end_byte = end_frame * frame_size;
    Ok(data[start_byte..end_byte].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writer_basic() {
        let mut writer = VideoWriter::new(4, 4, 30.0).unwrap();
        let frame = Frame::blank(4, 4);
        writer.write_frame(&frame).unwrap();
        writer.write_frame(&frame).unwrap();
        assert_eq!(writer.frame_count(), 2);
        let data = writer.finalize();
        assert_eq!(data.len(), 4 * 4 * 3 * 2);
    }

    #[test]
    fn test_writer_size_mismatch() {
        let mut writer = VideoWriter::new(4, 4, 30.0).unwrap();
        let frame = Frame::blank(8, 8); // wrong size
        assert!(writer.write_frame(&frame).is_err());
    }

    #[test]
    fn test_concat_videos() {
        let v1 = vec![0u8; 2 * 2 * 3 * 3]; // 3 frames
        let v2 = vec![1u8; 2 * 2 * 3 * 2]; // 2 frames
        let result = concat_videos(&[&v1, &v2], 2, 2).unwrap();
        assert_eq!(result.len(), 2 * 2 * 3 * 5);
    }

    #[test]
    fn test_clip_video() {
        // 10 frames at 10fps = 1 second
        let data = vec![0u8; 4 * 4 * 3 * 10];
        let clipped = clip_video(&data, 4, 4, 10.0, 0.3, 0.7).unwrap();
        // frames 3..7 = 4 frames
        assert_eq!(clipped.len(), 4 * 4 * 3 * 4);
    }

    #[test]
    fn test_meta() {
        let mut writer = VideoWriter::new(1920, 1080, 60.0).unwrap();
        for _ in 0..120 {
            writer.write_frame(&Frame::blank(1920, 1080)).unwrap();
        }
        let meta = writer.meta();
        assert_eq!(meta.frame_count, 120);
        assert!((meta.duration - 2.0).abs() < 0.01);
    }
}
