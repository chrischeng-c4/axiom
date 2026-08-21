//! Linear programming via the Simplex method.

use super::error::{OptimizeError, Result};

/// Result of a linear programming solution.
#[derive(Debug, Clone)]
pub struct LinprogResult {
    /// Optimal solution vector.
    pub x: Vec<f64>,
    /// Optimal objective value.
    pub fun: f64,
    /// Number of iterations.
    pub nit: usize,
    /// Whether the optimization succeeded.
    pub success: bool,
    /// Status message.
    pub message: String,
}

/// Solve a linear programming problem using the revised Simplex method.
///
/// Minimize: `c^T x`
/// Subject to: `A_ub * x <= b_ub` (inequality constraints)
///
/// All variables are assumed non-negative: `x >= 0`.
///
/// # Arguments
/// * `c` - Objective coefficients (length n)
/// * `a_ub` - Inequality constraint matrix (m × n, row-major)
/// * `b_ub` - Inequality constraint bounds (length m, must be non-negative)
///
/// # Example
/// ```
/// use scikit::optimize::linprog;
/// // Minimize -x - 2y subject to x + y <= 4, x <= 3, y <= 3, x,y >= 0
/// let c = vec![-1.0, -2.0];
/// let a_ub = vec![1.0, 1.0, 1.0, 0.0, 0.0, 1.0];
/// let b_ub = vec![4.0, 3.0, 3.0];
/// let result = linprog(&c, &a_ub, &b_ub).unwrap();
/// assert!(result.success);
/// ```
pub fn linprog(c: &[f64], a_ub: &[f64], b_ub: &[f64]) -> Result<LinprogResult> {
    let n = c.len(); // number of decision variables
    let m = b_ub.len(); // number of constraints

    if n == 0 {
        return Err(OptimizeError::InvalidInput("empty objective".into()));
    }
    if a_ub.len() != m * n {
        return Err(OptimizeError::InvalidInput(format!(
            "a_ub has {} elements, expected {} ({}x{})",
            a_ub.len(),
            m * n,
            m,
            n
        )));
    }
    for (i, &b) in b_ub.iter().enumerate() {
        if b < 0.0 {
            return Err(OptimizeError::InvalidInput(format!(
                "b_ub[{}] = {} is negative; transform constraints first",
                i, b
            )));
        }
    }

    // Build initial simplex tableau with slack variables.
    // Variables: x_0..x_{n-1} (decision), s_0..s_{m-1} (slack)
    // Tableau rows: m constraint rows + 1 objective row
    // Columns: n + m variables + 1 rhs
    let total_vars = n + m;
    let cols = total_vars + 1;
    let rows = m + 1;
    let mut tableau = vec![0.0; rows * cols];

    let idx = |r: usize, c_idx: usize| r * cols + c_idx;

    // Fill constraint rows
    for i in 0..m {
        for j in 0..n {
            tableau[idx(i, j)] = a_ub[i * n + j];
        }
        // Slack variable
        tableau[idx(i, n + i)] = 1.0;
        // RHS
        tableau[idx(i, total_vars)] = b_ub[i];
    }

    // Objective row: minimize c^T x => last row is c coefficients
    for j in 0..n {
        tableau[idx(m, j)] = c[j];
    }

    // Basis tracking
    let mut basis: Vec<usize> = (n..n + m).collect();

    let max_iter = 1000;
    for iter in 0..max_iter {
        // Find entering variable: most negative coefficient in objective row
        let mut pivot_col = None;
        let mut min_val = -1e-10;
        for j in 0..total_vars {
            let val = tableau[idx(m, j)];
            if val < min_val {
                min_val = val;
                pivot_col = Some(j);
            }
        }

        let pivot_col = match pivot_col {
            Some(col) => col,
            None => {
                // Optimal: no negative coefficients
                let mut x = vec![0.0; n];
                for (i, &b_var) in basis.iter().enumerate() {
                    if b_var < n {
                        x[b_var] = tableau[idx(i, total_vars)];
                    }
                }
                let fun = c.iter().zip(x.iter()).map(|(&ci, &xi)| ci * xi).sum();
                return Ok(LinprogResult {
                    x,
                    fun,
                    nit: iter,
                    success: true,
                    message: "Optimization terminated successfully.".into(),
                });
            }
        };

        // Minimum ratio test for leaving variable
        let mut pivot_row = None;
        let mut min_ratio = f64::INFINITY;
        for i in 0..m {
            let elem = tableau[idx(i, pivot_col)];
            if elem > 1e-10 {
                let ratio = tableau[idx(i, total_vars)] / elem;
                if ratio < min_ratio {
                    min_ratio = ratio;
                    pivot_row = Some(i);
                }
            }
        }

        let pivot_row = match pivot_row {
            Some(row) => row,
            None => {
                return Ok(LinprogResult {
                    x: vec![0.0; n],
                    fun: f64::NEG_INFINITY,
                    nit: iter,
                    success: false,
                    message: "Problem is unbounded.".into(),
                });
            }
        };

        // Pivot operation
        let pivot_elem = tableau[idx(pivot_row, pivot_col)];
        for j in 0..cols {
            tableau[idx(pivot_row, j)] /= pivot_elem;
        }

        for i in 0..rows {
            if i == pivot_row {
                continue;
            }
            let factor = tableau[idx(i, pivot_col)];
            if factor.abs() > 1e-14 {
                for j in 0..cols {
                    tableau[idx(i, j)] -= factor * tableau[idx(pivot_row, j)];
                }
            }
        }

        basis[pivot_row] = pivot_col;
    }

    // Did not converge
    let mut x = vec![0.0; n];
    for (i, &b_var) in basis.iter().enumerate() {
        if b_var < n {
            x[b_var] = tableau[idx(i, total_vars)];
        }
    }
    let fun = c.iter().zip(x.iter()).map(|(&ci, &xi)| ci * xi).sum();
    Ok(LinprogResult {
        x,
        fun,
        nit: max_iter,
        success: false,
        message: "Iteration limit reached.".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linprog_basic() {
        // Minimize -x - 2y
        // s.t. x + y <= 4, x <= 3, y <= 3
        let c = vec![-1.0, -2.0];
        let a_ub = vec![1.0, 1.0, 1.0, 0.0, 0.0, 1.0];
        let b_ub = vec![4.0, 3.0, 3.0];
        let result = linprog(&c, &a_ub, &b_ub).unwrap();
        assert!(result.success);
        assert!((result.x[0] - 1.0).abs() < 1e-6);
        assert!((result.x[1] - 3.0).abs() < 1e-6);
        assert!((result.fun - (-7.0)).abs() < 1e-6);
    }

    #[test]
    fn test_linprog_single_variable() {
        // Minimize -3x, s.t. x <= 5
        let c = vec![-3.0];
        let a_ub = vec![1.0];
        let b_ub = vec![5.0];
        let result = linprog(&c, &a_ub, &b_ub).unwrap();
        assert!(result.success);
        assert!((result.x[0] - 5.0).abs() < 1e-6);
        assert!((result.fun - (-15.0)).abs() < 1e-6);
    }

    #[test]
    fn test_linprog_already_optimal() {
        // Minimize x + y, s.t. x + y <= 10 — optimum at origin
        let c = vec![1.0, 1.0];
        let a_ub = vec![1.0, 1.0];
        let b_ub = vec![10.0];
        let result = linprog(&c, &a_ub, &b_ub).unwrap();
        assert!(result.success);
        assert!(result.x[0].abs() < 1e-6);
        assert!(result.x[1].abs() < 1e-6);
        assert!(result.fun.abs() < 1e-6);
    }

    #[test]
    fn test_linprog_invalid_dimensions() {
        let c = vec![1.0, 2.0];
        let a_ub = vec![1.0]; // wrong size
        let b_ub = vec![5.0];
        assert!(linprog(&c, &a_ub, &b_ub).is_err());
    }
}
