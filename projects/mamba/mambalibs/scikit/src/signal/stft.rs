//! Short-Time Fourier Transform (STFT).

use crate::fft::{fft, Complex};

/// Result of the Short-Time Fourier Transform.
#[derive(Debug, Clone)]
pub struct StftResult {
    /// Frequency bins (Hz). Length = window_size / 2 + 1.
    pub frequencies: Vec<f64>,
    /// Time centers of each frame (seconds).
    pub times: Vec<f64>,
    /// Spectrogram: `spectrogram[t][f]` is the complex value for time frame `t`
    /// and frequency bin `f`. Outer length = `times.len()`, inner = `frequencies.len()`.
    pub spectrogram: Vec<Vec<Complex>>,
}

/// Compute the Short-Time Fourier Transform.
///
/// Segments the signal into overlapping windows, applies a Hann window, and
/// computes the FFT of each segment.
///
/// # Arguments
///
/// * `x` - Input signal
/// * `window_size` - Number of samples per window (must be >= 2)
/// * `hop_size` - Number of samples between successive windows (must be >= 1)
/// * `fs` - Sampling frequency in Hz
///
/// # Panics
///
/// Panics if `window_size < 2` or `hop_size < 1` or `x` is shorter than `window_size`.
///
/// # Example
///
/// ```
/// use scikit::signal::stft;
///
/// let fs = 100.0;
/// let x: Vec<f64> = (0..256).map(|i| (2.0 * std::f64::consts::PI * 10.0 * i as f64 / fs).sin()).collect();
/// let result = stft(&x, 64, 32, fs);
/// assert!(!result.times.is_empty());
/// assert_eq!(result.frequencies.len(), 33); // 64/2 + 1
/// ```
pub fn stft(x: &[f64], window_size: usize, hop_size: usize, fs: f64) -> StftResult {
    assert!(window_size >= 2, "window_size must be >= 2");
    assert!(hop_size >= 1, "hop_size must be >= 1");
    assert!(
        x.len() >= window_size,
        "signal length ({}) must be >= window_size ({window_size})",
        x.len()
    );

    let n_freqs = window_size / 2 + 1;
    let window = hann_window(window_size);

    let frequencies: Vec<f64> = (0..n_freqs)
        .map(|i| i as f64 * fs / window_size as f64)
        .collect();

    let mut times = Vec::new();
    let mut spectrogram = Vec::new();

    let mut start = 0;
    while start + window_size <= x.len() {
        // Center time of the window
        let t = (start as f64 + (window_size - 1) as f64 / 2.0) / fs;
        times.push(t);

        // Apply window and convert to Complex
        let windowed: Vec<Complex> = (0..window_size)
            .map(|i| Complex::new(x[start + i] * window[i], 0.0))
            .collect();

        let spectrum = fft(&windowed);

        // Keep only positive frequencies (first n_freqs bins)
        spectrogram.push(spectrum[..n_freqs].to_vec());

        start += hop_size;
    }

    StftResult {
        frequencies,
        times,
        spectrogram,
    }
}

/// Generate a Hann window of size `n`.
fn hann_window(n: usize) -> Vec<f64> {
    use std::f64::consts::PI;
    (0..n)
        .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f64 / (n - 1) as f64).cos()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_stft_dimensions() {
        let fs = 100.0;
        let n = 256;
        let window_size = 64;
        let hop_size = 32;

        let x: Vec<f64> = (0..n)
            .map(|i| (2.0 * PI * 10.0 * i as f64 / fs).sin())
            .collect();

        let result = stft(&x, window_size, hop_size, fs);

        // Number of frequency bins
        assert_eq!(result.frequencies.len(), window_size / 2 + 1);

        // Number of time frames: floor((n - window_size) / hop_size) + 1
        let expected_frames = (n - window_size) / hop_size + 1;
        assert_eq!(result.times.len(), expected_frames);
        assert_eq!(result.spectrogram.len(), expected_frames);

        // Each spectrum has the right length
        for spectrum in &result.spectrogram {
            assert_eq!(spectrum.len(), window_size / 2 + 1);
        }
    }

    #[test]
    fn test_stft_detects_frequency() {
        let fs = 256.0;
        let n = 512;
        let window_size = 64;
        let hop_size = 32;
        let freq = 32.0; // Hz

        let x: Vec<f64> = (0..n)
            .map(|i| (2.0 * PI * freq * i as f64 / fs).sin())
            .collect();

        let result = stft(&x, window_size, hop_size, fs);

        // The frequency resolution is fs / window_size = 4 Hz
        // 32 Hz should appear at bin index 32 / 4 = 8
        let target_bin = (freq / (fs / window_size as f64)).round() as usize;

        // Check a middle frame (avoid edges) to see dominant frequency
        let mid_frame = result.spectrogram.len() / 2;
        let magnitudes: Vec<f64> = result.spectrogram[mid_frame]
            .iter()
            .map(|c| c.norm())
            .collect();

        // The target bin should have the largest magnitude (skip DC)
        let peak_bin = (1..magnitudes.len())
            .max_by(|&a, &b| magnitudes[a].partial_cmp(&magnitudes[b]).unwrap())
            .unwrap();

        assert_eq!(
            peak_bin, target_bin,
            "expected peak at bin {target_bin} ({freq} Hz), got bin {peak_bin}"
        );
    }

    #[test]
    fn test_stft_frequency_values() {
        let fs = 100.0;
        let window_size = 16;
        let hop_size = 8;
        let x: Vec<f64> = (0..32).map(|i| i as f64).collect();

        let result = stft(&x, window_size, hop_size, fs);

        // First frequency should be 0 Hz (DC)
        assert!((result.frequencies[0] - 0.0).abs() < 1e-10);

        // Last frequency should be fs/2 (Nyquist)
        let nyquist = fs / 2.0;
        assert!(
            (result.frequencies[result.frequencies.len() - 1] - nyquist).abs() < 1e-10,
            "expected Nyquist = {nyquist}, got {}",
            result.frequencies[result.frequencies.len() - 1]
        );

        // Frequency spacing should be fs / window_size
        let df = fs / window_size as f64;
        for i in 0..result.frequencies.len() {
            let expected = i as f64 * df;
            assert!(
                (result.frequencies[i] - expected).abs() < 1e-10,
                "freq[{i}]: expected {expected}, got {}",
                result.frequencies[i]
            );
        }
    }
}
