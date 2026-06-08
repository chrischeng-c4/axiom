//! Resampling (downsample / upsample) for time series data.
//!
//! Operates on raw `&[f64]` slices with a specified frequency ratio.

use super::error::{Result, TsError};

/// Aggregation method for downsampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggMethod {
    Mean,
    Sum,
    Min,
    Max,
    First,
    Last,
    Median,
}

/// Interpolation method for upsampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpMethod {
    Linear,
    ForwardFill,
    BackwardFill,
    Nearest,
}

/// Downsample a time series by grouping every `factor` consecutive points.
///
/// If the data length is not evenly divisible by `factor`, the last
/// (partial) group is still included.
pub fn downsample(data: &[f64], factor: usize, method: AggMethod) -> Result<Vec<f64>> {
    if factor == 0 {
        return Err(TsError::InvalidParameter(
            "downsample factor must be > 0".into(),
        ));
    }
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let mut result = Vec::with_capacity((data.len() + factor - 1) / factor);
    for chunk in data.chunks(factor) {
        let val = aggregate(chunk, method);
        result.push(val);
    }
    Ok(result)
}

/// Upsample a time series by inserting `factor - 1` interpolated points
/// between each original point.
///
/// Output length = `(data.len() - 1) * factor + 1` for linear/nearest,
/// or `data.len() * factor` for ffill/bfill.
pub fn upsample(data: &[f64], factor: usize, method: InterpMethod) -> Result<Vec<f64>> {
    if factor == 0 {
        return Err(TsError::InvalidParameter(
            "upsample factor must be > 0".into(),
        ));
    }
    if data.is_empty() {
        return Ok(Vec::new());
    }
    if factor == 1 {
        return Ok(data.to_vec());
    }

    match method {
        InterpMethod::Linear => upsample_linear(data, factor),
        InterpMethod::ForwardFill => upsample_ffill(data, factor),
        InterpMethod::BackwardFill => upsample_bfill(data, factor),
        InterpMethod::Nearest => upsample_nearest(data, factor),
    }
}

fn aggregate(chunk: &[f64], method: AggMethod) -> f64 {
    match method {
        AggMethod::Mean => chunk.iter().sum::<f64>() / chunk.len() as f64,
        AggMethod::Sum => chunk.iter().sum(),
        AggMethod::Min => chunk.iter().copied().fold(f64::INFINITY, f64::min),
        AggMethod::Max => chunk.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        AggMethod::First => chunk[0],
        AggMethod::Last => chunk[chunk.len() - 1],
        AggMethod::Median => {
            let mut sorted = chunk.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mid = sorted.len() / 2;
            if sorted.len() % 2 == 0 {
                (sorted[mid - 1] + sorted[mid]) / 2.0
            } else {
                sorted[mid]
            }
        }
    }
}

fn upsample_linear(data: &[f64], factor: usize) -> Result<Vec<f64>> {
    if data.len() == 1 {
        return Ok(vec![data[0]]);
    }
    let out_len = (data.len() - 1) * factor + 1;
    let mut result = Vec::with_capacity(out_len);
    for i in 0..data.len() - 1 {
        let start = data[i];
        let end = data[i + 1];
        for j in 0..factor {
            let t = j as f64 / factor as f64;
            result.push(start + t * (end - start));
        }
    }
    result.push(*data.last().unwrap());
    Ok(result)
}

fn upsample_ffill(data: &[f64], factor: usize) -> Result<Vec<f64>> {
    let mut result = Vec::with_capacity(data.len() * factor);
    for &val in data {
        for _ in 0..factor {
            result.push(val);
        }
    }
    Ok(result)
}

fn upsample_bfill(data: &[f64], factor: usize) -> Result<Vec<f64>> {
    let mut result = Vec::with_capacity(data.len() * factor);
    for (i, &_val) in data.iter().enumerate() {
        let fill_val = if i + 1 < data.len() {
            data[i + 1]
        } else {
            data[i]
        };
        // First position keeps original, rest fill with next
        result.push(data[i]);
        for _ in 1..factor {
            result.push(fill_val);
        }
    }
    Ok(result)
}

fn upsample_nearest(data: &[f64], factor: usize) -> Result<Vec<f64>> {
    if data.len() == 1 {
        return Ok(vec![data[0]]);
    }
    let out_len = (data.len() - 1) * factor + 1;
    let mut result = Vec::with_capacity(out_len);
    for i in 0..data.len() - 1 {
        for j in 0..factor {
            let t = j as f64 / factor as f64;
            if t < 0.5 {
                result.push(data[i]);
            } else {
                result.push(data[i + 1]);
            }
        }
    }
    result.push(*data.last().unwrap());
    Ok(result)
}

/// Resample with a ratio (general case).
///
/// `target_len` specifies the desired output length. Uses linear interpolation.
pub fn resample_to_len(data: &[f64], target_len: usize) -> Result<Vec<f64>> {
    if target_len == 0 {
        return Err(TsError::InvalidParameter(
            "target_len must be > 0".into(),
        ));
    }
    if data.is_empty() {
        return Ok(Vec::new());
    }
    if data.len() == 1 || target_len == 1 {
        return Ok(vec![data[0]]);
    }

    let mut result = Vec::with_capacity(target_len);
    for i in 0..target_len {
        let t = i as f64 * (data.len() - 1) as f64 / (target_len - 1) as f64;
        let lo = t.floor() as usize;
        let hi = (lo + 1).min(data.len() - 1);
        let frac = t - lo as f64;
        result.push(data[lo] * (1.0 - frac) + data[hi] * frac);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downsample_mean() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let result = downsample(&data, 2, AggMethod::Mean).unwrap();
        assert_eq!(result.len(), 3);
        assert!((result[0] - 1.5).abs() < 1e-10);
        assert!((result[1] - 3.5).abs() < 1e-10);
        assert!((result[2] - 5.5).abs() < 1e-10);
    }

    #[test]
    fn test_downsample_sum() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = downsample(&data, 3, AggMethod::Sum).unwrap();
        assert_eq!(result.len(), 2);
        assert!((result[0] - 6.0).abs() < 1e-10);
        assert!((result[1] - 9.0).abs() < 1e-10);
    }

    #[test]
    fn test_downsample_min_max() {
        let data = vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0];
        let mins = downsample(&data, 3, AggMethod::Min).unwrap();
        assert!((mins[0] - 1.0).abs() < 1e-10);
        let maxs = downsample(&data, 3, AggMethod::Max).unwrap();
        assert!((maxs[1] - 9.0).abs() < 1e-10);
    }

    #[test]
    fn test_upsample_linear() {
        let data = vec![0.0, 10.0, 20.0];
        let result = upsample(&data, 2, InterpMethod::Linear).unwrap();
        assert_eq!(result.len(), 5);
        assert!((result[0] - 0.0).abs() < 1e-10);
        assert!((result[1] - 5.0).abs() < 1e-10);
        assert!((result[2] - 10.0).abs() < 1e-10);
        assert!((result[3] - 15.0).abs() < 1e-10);
        assert!((result[4] - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_upsample_ffill() {
        let data = vec![1.0, 2.0, 3.0];
        let result = upsample(&data, 3, InterpMethod::ForwardFill).unwrap();
        assert_eq!(result.len(), 9);
        assert!((result[0] - 1.0).abs() < 1e-10);
        assert!((result[1] - 1.0).abs() < 1e-10);
        assert!((result[2] - 1.0).abs() < 1e-10);
        assert!((result[3] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_resample_to_len() {
        let data = vec![0.0, 10.0, 20.0];
        let result = resample_to_len(&data, 5).unwrap();
        assert_eq!(result.len(), 5);
        assert!((result[0] - 0.0).abs() < 1e-10);
        assert!((result[2] - 10.0).abs() < 1e-10);
        assert!((result[4] - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_empty() {
        assert!(downsample(&[], 2, AggMethod::Mean).unwrap().is_empty());
        assert!(upsample(&[], 2, InterpMethod::Linear).unwrap().is_empty());
    }

    #[test]
    fn test_invalid_factor() {
        assert!(downsample(&[1.0], 0, AggMethod::Mean).is_err());
        assert!(upsample(&[1.0], 0, InterpMethod::Linear).is_err());
    }

    #[test]
    fn test_downsample_median() {
        let data = vec![1.0, 3.0, 2.0, 8.0, 5.0, 7.0];
        let result = downsample(&data, 3, AggMethod::Median).unwrap();
        assert!((result[0] - 2.0).abs() < 1e-10); // median of [1,3,2]
        assert!((result[1] - 7.0).abs() < 1e-10); // median of [8,5,7]
    }
}
