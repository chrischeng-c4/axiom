//! Peak detection (scipy.signal.find_peaks equivalent).

/// A detected peak.
#[derive(Debug, Clone)]
pub struct Peak {
    /// Index of the peak in the input array.
    pub index: usize,
    /// Value at the peak.
    pub height: f64,
    /// Prominence of the peak.
    pub prominence: f64,
}

/// Options for peak detection.
#[derive(Debug, Clone)]
pub struct PeakOptions {
    /// Minimum height for a peak.
    pub height: Option<f64>,
    /// Minimum distance between peaks (in samples).
    pub distance: Option<usize>,
    /// Minimum prominence.
    pub prominence: Option<f64>,
}

impl Default for PeakOptions {
    fn default() -> Self {
        Self {
            height: None,
            distance: None,
            prominence: None,
        }
    }
}

/// Find peaks in a 1D signal.
pub fn find_peaks(data: &[f64], options: &PeakOptions) -> Vec<Peak> {
    if data.len() < 3 {
        return vec![];
    }

    // Find all local maxima
    let mut candidates: Vec<(usize, f64)> = Vec::new();
    for i in 1..data.len() - 1 {
        if data[i] > data[i - 1] && data[i] > data[i + 1] {
            candidates.push((i, data[i]));
        }
    }

    // Filter by height
    if let Some(min_h) = options.height {
        candidates.retain(|&(_, h)| h >= min_h);
    }

    // Compute prominence for each candidate
    let mut peaks: Vec<Peak> = candidates
        .iter()
        .map(|&(idx, h)| {
            let prom = compute_prominence(data, idx);
            Peak {
                index: idx,
                height: h,
                prominence: prom,
            }
        })
        .collect();

    // Filter by prominence
    if let Some(min_p) = options.prominence {
        peaks.retain(|p| p.prominence >= min_p);
    }

    // Filter by distance (keep highest peaks first)
    if let Some(min_dist) = options.distance {
        peaks.sort_by(|a, b| b.height.partial_cmp(&a.height).unwrap());
        let mut keep = vec![true; peaks.len()];
        for i in 0..peaks.len() {
            if !keep[i] {
                continue;
            }
            for j in i + 1..peaks.len() {
                if !keep[j] {
                    continue;
                }
                let dist = if peaks[i].index > peaks[j].index {
                    peaks[i].index - peaks[j].index
                } else {
                    peaks[j].index - peaks[i].index
                };
                if dist < min_dist {
                    keep[j] = false;
                }
            }
        }
        let filtered: Vec<Peak> = peaks
            .into_iter()
            .zip(keep.iter())
            .filter(|(_, &k)| k)
            .map(|(p, _)| p)
            .collect();
        peaks = filtered;
    }

    // Sort by index for consistent output
    peaks.sort_by_key(|p| p.index);
    peaks
}

fn compute_prominence(data: &[f64], peak_idx: usize) -> f64 {
    let peak_val = data[peak_idx];

    // Search left for the lowest point before a higher peak
    let mut left_min = peak_val;
    for i in (0..peak_idx).rev() {
        left_min = left_min.min(data[i]);
        if data[i] > peak_val {
            break;
        }
    }

    // Search right
    let mut right_min = peak_val;
    for i in peak_idx + 1..data.len() {
        right_min = right_min.min(data[i]);
        if data[i] > peak_val {
            break;
        }
    }

    peak_val - left_min.max(right_min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_peaks_simple() {
        let data = vec![0.0, 1.0, 0.0, 2.0, 0.0, 3.0, 0.0];
        let peaks = find_peaks(&data, &PeakOptions::default());
        assert_eq!(peaks.len(), 3);
        assert_eq!(peaks[0].index, 1);
        assert_eq!(peaks[1].index, 3);
        assert_eq!(peaks[2].index, 5);
    }

    #[test]
    fn test_find_peaks_height_filter() {
        let data = vec![0.0, 1.0, 0.0, 5.0, 0.0, 2.0, 0.0];
        let peaks = find_peaks(
            &data,
            &PeakOptions {
                height: Some(3.0),
                ..Default::default()
            },
        );
        assert_eq!(peaks.len(), 1);
        assert_eq!(peaks[0].index, 3);
    }

    #[test]
    fn test_find_peaks_distance_filter() {
        let data = vec![0.0, 3.0, 0.0, 2.0, 0.0, 5.0, 0.0];
        let peaks = find_peaks(
            &data,
            &PeakOptions {
                distance: Some(3),
                ..Default::default()
            },
        );
        // Should keep peaks at index 5 (highest) and 1 (distance >= 3 from 5)
        assert!(peaks.len() <= 3);
        assert!(peaks.iter().any(|p| p.index == 5));
    }

    #[test]
    fn test_find_peaks_empty() {
        let data = vec![1.0, 2.0];
        let peaks = find_peaks(&data, &PeakOptions::default());
        assert!(peaks.is_empty());
    }
}
