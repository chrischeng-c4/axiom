//! Audio extraction and representation.
//!
//! Provides types and utilities for extracting audio tracks from video data.
//! Currently supports raw PCM audio. Codec-based audio extraction requires
//! the `ffmpeg` feature.

use super::error::{Result, VideoError};

/// Audio sample format.
#[derive(Debug, Clone, PartialEq)]
pub enum SampleFormat {
    /// 16-bit signed integer PCM.
    I16,
    /// 32-bit floating point.
    F32,
}

/// Raw audio data extracted from a video.
#[derive(Debug, Clone)]
pub struct AudioTrack {
    /// PCM samples (interleaved if multi-channel).
    pub samples: Vec<f32>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: u16,
    /// Duration in seconds.
    pub duration: f64,
}

impl AudioTrack {
    /// Create a new audio track from f32 samples.
    pub fn from_samples(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        let total_samples = samples.len();
        let duration = total_samples as f64 / (sample_rate as f64 * channels as f64);
        Self {
            samples,
            sample_rate,
            channels,
            duration,
        }
    }

    /// Create a silent audio track with the given duration.
    pub fn silence(sample_rate: u32, channels: u16, duration: f64) -> Self {
        let total_samples = (sample_rate as f64 * channels as f64 * duration) as usize;
        Self {
            samples: vec![0.0; total_samples],
            sample_rate,
            channels,
            duration,
        }
    }

    /// Get the number of samples per channel.
    pub fn samples_per_channel(&self) -> usize {
        if self.channels == 0 {
            return 0;
        }
        self.samples.len() / self.channels as usize
    }

    /// Extract a single channel from interleaved audio.
    pub fn get_channel(&self, channel: u16) -> Result<Vec<f32>> {
        if channel >= self.channels {
            return Err(VideoError::InvalidParameter(format!(
                "channel {} out of range (0..{})",
                channel, self.channels
            )));
        }

        let ch = channel as usize;
        let nc = self.channels as usize;
        let channel_data: Vec<f32> = self.samples.iter().skip(ch).step_by(nc).copied().collect();
        Ok(channel_data)
    }

    /// Convert stereo to mono by averaging channels.
    pub fn to_mono(&self) -> Self {
        if self.channels <= 1 {
            return self.clone();
        }

        let nc = self.channels as usize;
        let samples_per_ch = self.samples.len() / nc;
        let mut mono = Vec::with_capacity(samples_per_ch);

        for i in 0..samples_per_ch {
            let mut sum = 0.0f32;
            for ch in 0..nc {
                sum += self.samples[i * nc + ch];
            }
            mono.push(sum / nc as f32);
        }

        Self {
            samples: mono,
            sample_rate: self.sample_rate,
            channels: 1,
            duration: self.duration,
        }
    }

    /// Resample to a different sample rate using linear interpolation.
    pub fn resample(&self, target_rate: u32) -> Self {
        if target_rate == self.sample_rate {
            return self.clone();
        }

        let ratio = target_rate as f64 / self.sample_rate as f64;
        let nc = self.channels as usize;
        let old_len = self.samples.len() / nc;
        let new_len = (old_len as f64 * ratio) as usize;
        let mut new_samples = Vec::with_capacity(new_len * nc);

        for i in 0..new_len {
            let src_pos = i as f64 / ratio;
            let idx = src_pos as usize;
            let frac = src_pos - idx as f64;

            for ch in 0..nc {
                let s0 = if idx < old_len {
                    self.samples[idx * nc + ch]
                } else {
                    0.0
                };
                let s1 = if idx + 1 < old_len {
                    self.samples[(idx + 1) * nc + ch]
                } else {
                    s0
                };
                new_samples.push(s0 + (s1 - s0) * frac as f32);
            }
        }

        Self {
            samples: new_samples,
            sample_rate: target_rate,
            channels: self.channels,
            duration: self.duration,
        }
    }
}

/// Extract audio from raw interleaved audio+video data.
///
/// This is a placeholder for the `ffmpeg` feature. Currently returns an error
/// unless raw PCM audio data is provided directly.
pub fn extract_audio_raw(
    pcm_data: &[u8],
    sample_rate: u32,
    channels: u16,
    format: SampleFormat,
) -> Result<AudioTrack> {
    match format {
        SampleFormat::I16 => {
            if pcm_data.len() % 2 != 0 {
                return Err(VideoError::DecodeError(
                    "I16 PCM data must have even byte count".into(),
                ));
            }
            let samples: Vec<f32> = pcm_data
                .chunks_exact(2)
                .map(|chunk| {
                    let val = i16::from_le_bytes([chunk[0], chunk[1]]);
                    val as f32 / 32768.0
                })
                .collect();
            Ok(AudioTrack::from_samples(samples, sample_rate, channels))
        }
        SampleFormat::F32 => {
            if pcm_data.len() % 4 != 0 {
                return Err(VideoError::DecodeError(
                    "F32 PCM data must have byte count divisible by 4".into(),
                ));
            }
            let samples: Vec<f32> = pcm_data
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();
            Ok(AudioTrack::from_samples(samples, sample_rate, channels))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence() {
        let track = AudioTrack::silence(44100, 2, 1.0);
        assert_eq!(track.sample_rate, 44100);
        assert_eq!(track.channels, 2);
        assert_eq!(track.samples.len(), 88200);
        assert!(track.samples.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_to_mono() {
        let samples = vec![1.0, 0.0, 0.5, 0.5, 0.0, 1.0]; // 3 stereo samples
        let track = AudioTrack::from_samples(samples, 44100, 2);
        let mono = track.to_mono();
        assert_eq!(mono.channels, 1);
        assert_eq!(mono.samples.len(), 3);
        assert!((mono.samples[0] - 0.5).abs() < 1e-6);
        assert!((mono.samples[1] - 0.5).abs() < 1e-6);
        assert!((mono.samples[2] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_get_channel() {
        let samples = vec![1.0, 2.0, 3.0, 4.0]; // 2 stereo samples
        let track = AudioTrack::from_samples(samples, 44100, 2);
        let left = track.get_channel(0).unwrap();
        let right = track.get_channel(1).unwrap();
        assert_eq!(left, vec![1.0, 3.0]);
        assert_eq!(right, vec![2.0, 4.0]);
    }

    #[test]
    fn test_extract_audio_i16() {
        // Two samples: 0 and max
        let data: Vec<u8> = vec![0, 0, 0xFF, 0x7F]; // 0 and 32767
        let track = extract_audio_raw(&data, 44100, 1, SampleFormat::I16).unwrap();
        assert_eq!(track.samples.len(), 2);
        assert!((track.samples[0]).abs() < 1e-6);
        assert!((track.samples[1] - 32767.0 / 32768.0).abs() < 1e-4);
    }

    #[test]
    fn test_extract_audio_f32() {
        let val: f32 = 0.5;
        let bytes = val.to_le_bytes();
        let track = extract_audio_raw(&bytes, 44100, 1, SampleFormat::F32).unwrap();
        assert_eq!(track.samples.len(), 1);
        assert!((track.samples[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_resample() {
        let samples: Vec<f32> = (0..100).map(|i| i as f32 / 100.0).collect();
        let track = AudioTrack::from_samples(samples, 44100, 1);
        let resampled = track.resample(22050);
        assert_eq!(resampled.sample_rate, 22050);
        // Should be roughly half the samples
        assert!((resampled.samples.len() as f64 - 50.0).abs() < 2.0);
    }

    #[test]
    fn test_samples_per_channel() {
        let track = AudioTrack::from_samples(vec![0.0; 100], 44100, 2);
        assert_eq!(track.samples_per_channel(), 50);
    }
}
