//! 2D bilinear interpolation on a regular grid.

/// Bilinear interpolation on a 2D grid.
///
/// # Arguments
///
/// * `x_grid` - Sorted x coordinates of the grid
/// * `y_grid` - Sorted y coordinates of the grid
/// * `values` - Z values at grid points in row-major order (len = x_grid.len() * y_grid.len()).
///   `values[i * y_grid.len() + j]` corresponds to `(x_grid[i], y_grid[j])`.
/// * `xi` - Query x coordinate
/// * `yi` - Query y coordinate
///
/// # Errors
///
/// Returns an error if grid dimensions are invalid or the query point is outside the grid.
///
/// # Example
///
/// ```
/// use scikit::interpolate::interp2d;
///
/// let x = vec![0.0, 1.0];
/// let y = vec![0.0, 1.0];
/// let values = vec![0.0, 1.0, 2.0, 3.0]; // z(0,0)=0, z(0,1)=1, z(1,0)=2, z(1,1)=3
/// let z = interp2d(&x, &y, &values, 0.5, 0.5).unwrap();
/// assert!((z - 1.5).abs() < 1e-10);
/// ```
pub fn interp2d(
    x_grid: &[f64],
    y_grid: &[f64],
    values: &[f64],
    xi: f64,
    yi: f64,
) -> Result<f64, Interp2dError> {
    let nx = x_grid.len();
    let ny = y_grid.len();

    if nx < 2 || ny < 2 {
        return Err(Interp2dError::InsufficientGrid { nx, ny });
    }
    if values.len() != nx * ny {
        return Err(Interp2dError::DimensionMismatch {
            expected: nx * ny,
            got: values.len(),
        });
    }

    // Clamp to grid boundaries
    let xi_clamped = xi.clamp(x_grid[0], x_grid[nx - 1]);
    let yi_clamped = yi.clamp(y_grid[0], y_grid[ny - 1]);

    let ix = find_interval(x_grid, xi_clamped);
    let iy = find_interval(y_grid, yi_clamped);

    let x0 = x_grid[ix];
    let x1 = x_grid[ix + 1];
    let y0 = y_grid[iy];
    let y1 = y_grid[iy + 1];

    // Normalized distances
    let tx = if (x1 - x0).abs() < 1e-14 {
        0.0
    } else {
        (xi_clamped - x0) / (x1 - x0)
    };
    let ty = if (y1 - y0).abs() < 1e-14 {
        0.0
    } else {
        (yi_clamped - y0) / (y1 - y0)
    };

    // Four corner values
    let q00 = values[ix * ny + iy];
    let q01 = values[ix * ny + (iy + 1)];
    let q10 = values[(ix + 1) * ny + iy];
    let q11 = values[(ix + 1) * ny + (iy + 1)];

    // Bilinear interpolation
    let result = q00 * (1.0 - tx) * (1.0 - ty)
        + q10 * tx * (1.0 - ty)
        + q01 * (1.0 - tx) * ty
        + q11 * tx * ty;

    Ok(result)
}

/// Errors from 2D interpolation.
#[derive(Debug, Clone)]
pub enum Interp2dError {
    /// Grid too small (need at least 2 points in each dimension).
    InsufficientGrid { nx: usize, ny: usize },
    /// Values length doesn't match grid dimensions.
    DimensionMismatch { expected: usize, got: usize },
}

impl std::fmt::Display for Interp2dError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Interp2dError::InsufficientGrid { nx, ny } => {
                write!(f, "grid too small: nx={nx}, ny={ny} (need >= 2 each)")
            }
            Interp2dError::DimensionMismatch { expected, got } => {
                write!(f, "values length mismatch: expected {expected}, got {got}")
            }
        }
    }
}

impl std::error::Error for Interp2dError {}

/// Binary-search for the interval containing `val` in a sorted array.
/// Returns `i` such that `grid[i] <= val <= grid[i+1]`.
fn find_interval(grid: &[f64], val: f64) -> usize {
    let n = grid.len();
    if val <= grid[0] {
        return 0;
    }
    if val >= grid[n - 1] {
        return n - 2;
    }
    let mut lo = 0;
    let mut hi = n - 1;
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if grid[mid] <= val {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interp2d_center() {
        // 2x2 grid: z = x + y
        let x_grid = vec![0.0, 1.0];
        let y_grid = vec![0.0, 1.0];
        let values = vec![0.0, 1.0, 1.0, 2.0];
        let z = interp2d(&x_grid, &y_grid, &values, 0.5, 0.5).unwrap();
        assert!((z - 1.0).abs() < 1e-10, "expected 1.0, got {z}");
    }

    #[test]
    fn test_interp2d_corners_exact() {
        let x_grid = vec![0.0, 1.0, 2.0];
        let y_grid = vec![0.0, 1.0, 2.0];
        // z = x * y
        let values = vec![
            0.0, 0.0, 0.0, // x=0
            0.0, 1.0, 2.0, // x=1
            0.0, 2.0, 4.0, // x=2
        ];
        // Query at grid points should return exact values
        assert!((interp2d(&x_grid, &y_grid, &values, 0.0, 0.0).unwrap() - 0.0).abs() < 1e-10);
        assert!((interp2d(&x_grid, &y_grid, &values, 1.0, 1.0).unwrap() - 1.0).abs() < 1e-10);
        assert!((interp2d(&x_grid, &y_grid, &values, 2.0, 2.0).unwrap() - 4.0).abs() < 1e-10);
        assert!((interp2d(&x_grid, &y_grid, &values, 1.0, 2.0).unwrap() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_interp2d_midpoints() {
        let x_grid = vec![0.0, 1.0, 2.0];
        let y_grid = vec![0.0, 1.0, 2.0];
        // z = 2*x + 3*y (bilinear in x and y — should be exact)
        let mut values = vec![0.0; 9];
        for i in 0..3 {
            for j in 0..3 {
                values[i * 3 + j] = 2.0 * x_grid[i] + 3.0 * y_grid[j];
            }
        }
        let z = interp2d(&x_grid, &y_grid, &values, 0.5, 0.5).unwrap();
        let expected = 2.0 * 0.5 + 3.0 * 0.5;
        assert!((z - expected).abs() < 1e-10, "expected {expected}, got {z}");

        let z2 = interp2d(&x_grid, &y_grid, &values, 1.5, 1.5).unwrap();
        let expected2 = 2.0 * 1.5 + 3.0 * 1.5;
        assert!(
            (z2 - expected2).abs() < 1e-10,
            "expected {expected2}, got {z2}"
        );
    }

    #[test]
    fn test_interp2d_boundary_clamp() {
        let x_grid = vec![0.0, 1.0];
        let y_grid = vec![0.0, 1.0];
        let values = vec![1.0, 2.0, 3.0, 4.0];
        // Query outside boundary should be clamped
        let z = interp2d(&x_grid, &y_grid, &values, -1.0, 0.5).unwrap();
        // Clamped to x=0: lerp between z(0,0)=1 and z(0,1)=2 at ty=0.5 -> 1.5
        assert!((z - 1.5).abs() < 1e-10, "expected 1.5, got {z}");
    }

    #[test]
    fn test_interp2d_dimension_mismatch() {
        let x_grid = vec![0.0, 1.0];
        let y_grid = vec![0.0, 1.0];
        let values = vec![1.0, 2.0, 3.0]; // wrong length
        assert!(interp2d(&x_grid, &y_grid, &values, 0.5, 0.5).is_err());
    }

    #[test]
    fn test_interp2d_insufficient_grid() {
        let x_grid = vec![0.0];
        let y_grid = vec![0.0, 1.0];
        let values = vec![1.0, 2.0];
        assert!(interp2d(&x_grid, &y_grid, &values, 0.0, 0.5).is_err());
    }
}
