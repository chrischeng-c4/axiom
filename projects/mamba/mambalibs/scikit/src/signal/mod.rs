//! Signal processing module — scipy.signal equivalent.
//!
//! - **convolve**: 1D convolution and correlation
//! - **filter**: FIR/IIR digital filters, Butterworth design
//! - **filtfilt**: Zero-phase forward-backward filtering
//! - **windows**: Hann, Hamming, Blackman, Kaiser window functions
//! - **peaks**: Peak detection (find_peaks)
//! - **stft**: Short-Time Fourier Transform

mod convolve;
mod filter;
mod filtfilt;
mod peaks;
mod stft;
mod windows;

pub use convolve::{convolve, correlate, ConvolveMode};
pub use filter::{butter, lfilter, sosfilt, FilterType, SosSection};
pub use filtfilt::filtfilt;
pub use peaks::{find_peaks, Peak, PeakOptions};
pub use stft::{stft, StftResult};
pub use windows::{blackman, hamming, hann, kaiser};
