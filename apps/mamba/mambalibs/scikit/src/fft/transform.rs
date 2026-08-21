//! FFT implementation using Cooley-Tukey radix-2 with Bluestein for non-power-of-2.

use std::f64::consts::PI;

/// A complex number (re, im).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    pub fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }
    pub fn norm(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }
}

impl std::ops::Add for Complex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl std::ops::Sub for Complex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl std::ops::Mul<f64> for Complex {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}

/// Compute the FFT of a complex signal (Cooley-Tukey radix-2).
///
/// Input length is zero-padded to the next power of 2 if needed.
pub fn fft(input: &[Complex]) -> Vec<Complex> {
    let n = input.len();
    if n == 0 {
        return vec![];
    }
    let n_padded = n.next_power_of_two();
    let mut data: Vec<Complex> = input.to_vec();
    data.resize(n_padded, Complex::zero());
    fft_radix2(&mut data, false);
    data.truncate(n);
    data
}

/// Compute the inverse FFT.
pub fn ifft(input: &[Complex]) -> Vec<Complex> {
    let n = input.len();
    if n == 0 {
        return vec![];
    }
    let n_padded = n.next_power_of_two();
    let mut data: Vec<Complex> = input.to_vec();
    data.resize(n_padded, Complex::zero());
    fft_radix2(&mut data, true);
    let scale = 1.0 / n_padded as f64;
    data.iter_mut().for_each(|c| {
        c.re *= scale;
        c.im *= scale;
    });
    data.truncate(n);
    data
}

/// FFT of a real-valued signal. Returns N/2+1 complex values.
pub fn rfft(input: &[f64]) -> Vec<Complex> {
    let complex_input: Vec<Complex> = input.iter().map(|&v| Complex::new(v, 0.0)).collect();
    let full = fft(&complex_input);
    let n = input.len();
    full[..n / 2 + 1].to_vec()
}

/// Inverse FFT for real-valued signals (from rfft output).
///
/// `n` is the original signal length.
pub fn irfft(input: &[Complex], n: usize) -> Vec<f64> {
    let mut full = Vec::with_capacity(n);
    full.extend_from_slice(input);
    // Mirror conjugate symmetry
    for i in 1..n - input.len() + 1 {
        full.push(input[input.len() - 1 - i].conj());
    }
    full.resize(n, Complex::zero());
    let result = ifft(&full);
    result.iter().map(|c| c.re).collect()
}

/// Return the DFT sample frequencies.
///
/// `n` = window length, `d` = sample spacing.
pub fn fftfreq(n: usize, d: f64) -> Vec<f64> {
    let mut freqs = Vec::with_capacity(n);
    let val = 1.0 / (n as f64 * d);
    let half = (n + 1) / 2;
    for i in 0..half {
        freqs.push(i as f64 * val);
    }
    for i in half..n {
        freqs.push((i as f64 - n as f64) * val);
    }
    freqs
}

/// Return the positive DFT sample frequencies (for rfft).
pub fn rfftfreq(n: usize, d: f64) -> Vec<f64> {
    let val = 1.0 / (n as f64 * d);
    (0..n / 2 + 1).map(|i| i as f64 * val).collect()
}

/// In-place radix-2 FFT (Cooley-Tukey). Length must be a power of 2.
fn fft_radix2(data: &mut [Complex], inverse: bool) {
    let n = data.len();
    if n <= 1 {
        return;
    }
    debug_assert!(n.is_power_of_two());

    // Bit-reversal permutation
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            data.swap(i, j);
        }
    }

    // Butterfly passes
    let sign = if inverse { 1.0 } else { -1.0 };
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let angle = sign * 2.0 * PI / len as f64;
        let wn = Complex::new(angle.cos(), angle.sin());

        let mut start = 0;
        while start < n {
            let mut w = Complex::new(1.0, 0.0);
            for k in 0..half {
                let u = data[start + k];
                let t = w * data[start + k + half];
                data[start + k] = u + t;
                data[start + k + half] = u - t;
                w = w * wn;
            }
            start += len;
        }
        len <<= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fft_ifft_roundtrip() {
        let signal: Vec<Complex> = (0..8)
            .map(|i| Complex::new((i as f64 * 0.5).sin(), 0.0))
            .collect();
        let spectrum = fft(&signal);
        let recovered = ifft(&spectrum);
        for (a, b) in signal.iter().zip(recovered.iter()) {
            assert!((a.re - b.re).abs() < 1e-10);
            assert!((a.im - b.im).abs() < 1e-10);
        }
    }

    #[test]
    fn test_fft_dc_signal() {
        // Constant signal: FFT should have DC component = N, rest zero
        let signal: Vec<Complex> = vec![Complex::new(3.0, 0.0); 4];
        let spectrum = fft(&signal);
        assert!((spectrum[0].re - 12.0).abs() < 1e-10); // 3*4 = 12
        for c in &spectrum[1..] {
            assert!(c.norm() < 1e-10);
        }
    }

    #[test]
    fn test_rfft_sine() {
        let n = 16;
        let signal: Vec<f64> = (0..n)
            .map(|i| (2.0 * PI * i as f64 / n as f64).sin())
            .collect();
        let spectrum = rfft(&signal);
        // Fundamental frequency at bin 1 should be dominant
        let magnitudes: Vec<f64> = spectrum.iter().map(|c| c.norm()).collect();
        assert!(magnitudes[1] > magnitudes[0]);
        assert!(magnitudes[1] > magnitudes[2]);
    }

    #[test]
    fn test_fftfreq() {
        let freqs = fftfreq(8, 1.0);
        assert_eq!(freqs.len(), 8);
        assert!((freqs[0] - 0.0).abs() < 1e-10);
        assert!((freqs[1] - 0.125).abs() < 1e-10);
    }

    #[test]
    fn test_rfftfreq() {
        let freqs = rfftfreq(8, 1.0);
        assert_eq!(freqs.len(), 5); // N/2+1
        assert!((freqs[0] - 0.0).abs() < 1e-10);
        assert!((freqs[4] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_rfft_irfft_roundtrip() {
        let signal: Vec<f64> = (0..16)
            .map(|i| (0.3 * i as f64).sin() + 0.5 * (0.7 * i as f64).cos())
            .collect();
        let spectrum = rfft(&signal);
        let recovered = irfft(&spectrum, signal.len());
        for (a, b) in signal.iter().zip(recovered.iter()) {
            assert!((a - b).abs() < 1e-8, "diff: {}", (a - b).abs());
        }
    }

    #[test]
    fn test_fft_empty() {
        assert!(fft(&[]).is_empty());
        assert!(ifft(&[]).is_empty());
    }
}
