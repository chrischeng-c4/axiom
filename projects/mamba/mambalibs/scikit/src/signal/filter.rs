//! Digital filtering: FIR/IIR filters and Butterworth design.

use std::f64::consts::PI;

/// Filter type for design functions.
#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    Lowpass,
    Highpass,
}

/// A second-order section (biquad) filter: (b0, b1, b2, a0, a1, a2).
#[derive(Debug, Clone, Copy)]
pub struct SosSection {
    pub b: [f64; 3],
    pub a: [f64; 3],
}

/// Apply a transfer-function filter (IIR/FIR) to data.
///
/// `b` = numerator coefficients, `a` = denominator coefficients.
/// Uses direct form II transposed.
pub fn lfilter(b: &[f64], a: &[f64], x: &[f64]) -> Vec<f64> {
    assert!(!a.is_empty() && a[0].abs() > 1e-14, "a[0] must be nonzero");
    let a0 = a[0];

    let n = b.len().max(a.len());
    let mut d = vec![0.0; n]; // delay line
    let mut y = Vec::with_capacity(x.len());

    for &xi in x {
        let yi = b.first().copied().unwrap_or(0.0) / a0 * xi + d[0];
        y.push(yi);

        for j in 0..n - 1 {
            let bj1 = if j + 1 < b.len() { b[j + 1] } else { 0.0 };
            let aj1 = if j + 1 < a.len() { a[j + 1] } else { 0.0 };
            d[j] = bj1 / a0 * xi - aj1 / a0 * yi + d[j + 1];
        }
        d[n - 1] = 0.0;
    }

    y
}

/// Filter data using cascaded second-order sections.
pub fn sosfilt(sos: &[SosSection], x: &[f64]) -> Vec<f64> {
    let mut data = x.to_vec();
    for section in sos {
        data = lfilter(&section.b, &section.a, &data);
    }
    data
}

/// Design a Butterworth filter (analog prototype → digital via bilinear transform).
///
/// Returns second-order sections.
///
/// * `order` - Filter order (1-8 typical)
/// * `wn` - Normalized cutoff frequency (0 to 1, where 1 = Nyquist)
/// * `ftype` - Lowpass or Highpass
pub fn butter(order: usize, wn: f64, ftype: FilterType) -> Vec<SosSection> {
    assert!(order > 0, "order must be > 0");
    assert!(wn > 0.0 && wn < 1.0, "wn must be in (0, 1)");

    // Pre-warp
    let fs = 2.0;
    let warped = 2.0 * fs * (PI * wn / fs).tan();

    // Analog Butterworth poles
    let mut poles = Vec::new();
    for k in 0..order {
        let theta = PI * (2 * k + order + 1) as f64 / (2 * order) as f64;
        poles.push((warped * theta.cos(), warped * theta.sin()));
    }

    // Convert to second-order sections via bilinear transform
    let mut sections = Vec::new();
    let mut i = 0;

    while i < poles.len() {
        if poles[i].1.abs() < 1e-14 {
            // Real pole → first-order section (embed in SOS)
            let p_re = poles[i].0;
            let pd_re = (1.0 + p_re / (2.0 * fs)) / (1.0 - p_re / (2.0 * fs));

            let (b, a) = match ftype {
                FilterType::Lowpass => {
                    let g = (1.0 - pd_re) / 2.0;
                    ([g, g, 0.0], [1.0, -pd_re, 0.0])
                }
                FilterType::Highpass => {
                    let g = (1.0 + pd_re) / 2.0;
                    ([g, -g, 0.0], [1.0, -pd_re, 0.0])
                }
            };
            sections.push(SosSection { b, a });
            i += 1;
        } else {
            // Conjugate pair → second-order section
            let (p_re, p_im) = poles[i];
            // Bilinear transform: z = (1 + s/(2fs)) / (1 - s/(2fs))
            let c = 2.0 * fs;
            let denom_re = 1.0 - p_re / c;
            let denom_im = -p_im / c;
            let num_re = 1.0 + p_re / c;
            let num_im = p_im / c;
            let d2 = denom_re * denom_re + denom_im * denom_im;
            let pd_re = (num_re * denom_re + num_im * denom_im) / d2;
            let pd_im = (num_im * denom_re - num_re * denom_im) / d2;

            let a1 = -2.0 * pd_re;
            let a2 = pd_re * pd_re + pd_im * pd_im;

            let (b0, b1, b2) = match ftype {
                FilterType::Lowpass => {
                    let g = (1.0 + a1 + a2) / 4.0;
                    (g, 2.0 * g, g)
                }
                FilterType::Highpass => {
                    let g = (1.0 - a1 + a2) / 4.0;
                    (g, -2.0 * g, g)
                }
            };

            sections.push(SosSection {
                b: [b0, b1, b2],
                a: [1.0, a1, a2],
            });
            i += 2;
        }
    }

    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfilter_fir() {
        // Simple moving average (FIR)
        let b = vec![1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
        let a = vec![1.0];
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = lfilter(&b, &a, &x);
        assert_eq!(y.len(), 5);
        assert!((y[2] - 2.0).abs() < 1e-10); // (1+2+3)/3
        assert!((y[3] - 3.0).abs() < 1e-10); // (2+3+4)/3
    }

    #[test]
    fn test_lfilter_iir() {
        // First-order IIR: y[n] = x[n] + 0.5*y[n-1]
        let b = vec![1.0];
        let a = vec![1.0, -0.5];
        let x = vec![1.0, 0.0, 0.0, 0.0, 0.0];
        let y = lfilter(&b, &a, &x);
        // Impulse response: 1, 0.5, 0.25, 0.125, 0.0625
        assert!((y[0] - 1.0).abs() < 1e-10);
        assert!((y[1] - 0.5).abs() < 1e-10);
        assert!((y[2] - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_butter_lowpass() {
        let sos = butter(2, 0.3, FilterType::Lowpass);
        assert!(!sos.is_empty());
        // Apply to signal: should attenuate high frequencies
        let n = 100;
        let x: Vec<f64> = (0..n)
            .map(|i| {
                let t = i as f64 / n as f64;
                (2.0 * std::f64::consts::PI * 2.0 * t).sin()
                    + (2.0 * std::f64::consts::PI * 40.0 * t).sin()
            })
            .collect();
        let y = sosfilt(&sos, &x);
        assert_eq!(y.len(), n);
    }

    #[test]
    fn test_sosfilt_passthrough() {
        // Identity filter: b=[1,0,0], a=[1,0,0]
        let sos = vec![SosSection {
            b: [1.0, 0.0, 0.0],
            a: [1.0, 0.0, 0.0],
        }];
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = sosfilt(&sos, &x);
        for (a, b) in x.iter().zip(y.iter()) {
            assert!((a - b).abs() < 1e-10);
        }
    }
}
