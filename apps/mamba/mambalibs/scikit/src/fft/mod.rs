//! FFT module — Cooley-Tukey radix-2 and related transforms.
//!
//! - **fft / ifft**: Complex FFT and inverse
//! - **rfft / irfft**: Real-valued FFT
//! - **fftfreq / rfftfreq**: Frequency bins

mod transform;

pub use transform::{fft, fftfreq, ifft, irfft, rfft, rfftfreq, Complex};
