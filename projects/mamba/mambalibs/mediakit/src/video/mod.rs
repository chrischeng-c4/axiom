//! Video processing module.
//!
//! Provides video I/O, frame extraction, codec abstraction, audio extraction,
//! and basic operations. Currently supports raw uncompressed video by default;
//! additional codecs (H.264, etc.) require the `ffmpeg` feature wrapping ffmpeg-next.

pub mod audio;
mod codec;
mod error;
mod ops;
mod reader;
mod types;
mod writer;

pub use audio::{extract_audio_raw, AudioTrack, SampleFormat};
pub use codec::{get_decoder, get_encoder, Decoder, Encoder, RawDecoder, RawEncoder};
pub use error::{Result, VideoError};
pub use ops::{frame_diff, resize_video, sample_frames, to_grayscale};
pub use reader::{FrameIterator, ReaderConfig, VideoReader};
pub use types::{Codec, Frame, VideoMeta};
pub use writer::{clip_video, concat_videos, VideoWriter, WriterConfig};
