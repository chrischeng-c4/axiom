//! Codec abstraction layer.
//!
//! Provides a trait-based codec interface. The `Raw` codec is always available.
//! When the `ffmpeg` feature is enabled, additional codecs (H.264, H.265, etc.)
//! become available via ffmpeg-next.

use super::error::{Result, VideoError};
use super::types::{Codec, Frame, VideoMeta};

/// Trait for video decoders.
pub trait Decoder {
    /// Decode raw bytes into frames.
    fn decode(&self, data: &[u8], meta: &VideoMeta) -> Result<Vec<Frame>>;
    /// Supported codec.
    fn codec(&self) -> Codec;
}

/// Trait for video encoders.
pub trait Encoder {
    /// Encode frames into raw bytes.
    fn encode(&self, frames: &[Frame], meta: &VideoMeta) -> Result<Vec<u8>>;
    /// Supported codec.
    fn codec(&self) -> Codec;
}

/// Raw (uncompressed RGB) decoder.
pub struct RawDecoder;

impl Decoder for RawDecoder {
    fn decode(&self, data: &[u8], meta: &VideoMeta) -> Result<Vec<Frame>> {
        let frame_size = meta.width * meta.height * 3;
        if frame_size == 0 {
            return Err(VideoError::InvalidParameter("zero-size frame".into()));
        }
        if data.len() % frame_size != 0 {
            return Err(VideoError::DecodeError(
                "data length not aligned to frame size".into(),
            ));
        }

        let frame_count = data.len() / frame_size;
        let mut frames = Vec::with_capacity(frame_count);

        for i in 0..frame_count {
            let start = i * frame_size;
            let frame_data = data[start..start + frame_size].to_vec();
            let timestamp = i as f64 / meta.fps;
            frames.push(Frame::new(
                frame_data,
                meta.width,
                meta.height,
                timestamp,
                i,
            ));
        }

        Ok(frames)
    }

    fn codec(&self) -> Codec {
        Codec::Raw
    }
}

/// Raw (uncompressed RGB) encoder.
pub struct RawEncoder;

impl Encoder for RawEncoder {
    fn encode(&self, frames: &[Frame], _meta: &VideoMeta) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        for frame in frames {
            data.extend_from_slice(&frame.data);
        }
        Ok(data)
    }

    fn codec(&self) -> Codec {
        Codec::Raw
    }
}

/// Get a decoder for the given codec.
pub fn get_decoder(codec: &Codec) -> Result<Box<dyn Decoder>> {
    match codec {
        Codec::Raw => Ok(Box::new(RawDecoder)),
        other => Err(VideoError::UnsupportedCodec(format!(
            "{} (enable 'ffmpeg' feature for codec support)",
            other
        ))),
    }
}

/// Get an encoder for the given codec.
pub fn get_encoder(codec: &Codec) -> Result<Box<dyn Encoder>> {
    match codec {
        Codec::Raw => Ok(Box::new(RawEncoder)),
        other => Err(VideoError::UnsupportedCodec(format!(
            "{} (enable 'ffmpeg' feature for codec support)",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_decoder() {
        let meta = VideoMeta {
            width: 2,
            height: 2,
            fps: 10.0,
            frame_count: 2,
            duration: 0.2,
            codec: "raw".into(),
        };
        let data = vec![128u8; 2 * 2 * 3 * 2];
        let decoder = RawDecoder;
        let frames = decoder.decode(&data, &meta).unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].width, 2);
        assert_eq!(frames[1].index, 1);
    }

    #[test]
    fn test_raw_encoder() {
        let frames = vec![Frame::blank(2, 2), Frame::blank(2, 2)];
        let meta = VideoMeta {
            width: 2,
            height: 2,
            fps: 10.0,
            frame_count: 2,
            duration: 0.2,
            codec: "raw".into(),
        };
        let encoder = RawEncoder;
        let data = encoder.encode(&frames, &meta).unwrap();
        assert_eq!(data.len(), 2 * 2 * 3 * 2);
    }

    #[test]
    fn test_get_decoder_raw() {
        assert!(get_decoder(&Codec::Raw).is_ok());
    }

    #[test]
    fn test_get_decoder_unsupported() {
        assert!(get_decoder(&Codec::H264).is_err());
    }

    #[test]
    fn test_roundtrip() {
        let meta = VideoMeta {
            width: 4,
            height: 4,
            fps: 30.0,
            frame_count: 3,
            duration: 0.1,
            codec: "raw".into(),
        };
        let original: Vec<u8> = (0..4 * 4 * 3 * 3).map(|i| (i % 256) as u8).collect();
        let decoder = RawDecoder;
        let frames = decoder.decode(&original, &meta).unwrap();
        let encoder = RawEncoder;
        let encoded = encoder.encode(&frames, &meta).unwrap();
        assert_eq!(original, encoded);
    }
}
