//! Compressed Sparse Column (CSC) matrix format.

use super::csr::CsrMatrix;

/// A sparse matrix in Compressed Sparse Column (CSC) format.
///
/// Stores nonzero values column by column. Efficient for column slicing
/// and certain linear algebra operations.
#[derive(Debug, Clone)]
pub struct CscMatrix {
    /// Number of rows.
    pub nrows: usize,
    /// Number of columns.
    pub ncols: usize,
    /// Column pointers: `indptr[j]..indptr[j+1]` gives the range of
    /// nonzero entries in column `j`. Length = ncols + 1.
    pub indptr: Vec<usize>,
    /// Row indices of nonzero entries. Length = nnz.
    pub indices: Vec<usize>,
    /// Values of nonzero entries. Length = nnz.
    pub data: Vec<f64>,
}

impl CscMatrix {
    /// Create a CSC matrix from raw components.
    pub fn new(
        nrows: usize,
        ncols: usize,
        indptr: Vec<usize>,
        indices: Vec<usize>,
        data: Vec<f64>,
    ) -> Self {
        assert_eq!(indptr.len(), ncols + 1, "indptr length mismatch");
        assert_eq!(indices.len(), data.len(), "indices/data length mismatch");
        Self {
            nrows,
            ncols,
            indptr,
            indices,
            data,
        }
    }

    /// Create a CSC matrix from dense column-major data.
    pub fn from_dense(dense: &[f64], nrows: usize, ncols: usize) -> Self {
        assert_eq!(dense.len(), nrows * ncols, "dense data size mismatch");
        let mut indptr = vec![0usize];
        let mut indices = Vec::new();
        let mut data = Vec::new();

        for j in 0..ncols {
            for i in 0..nrows {
                let val = dense[i * ncols + j]; // row-major input
                if val.abs() > 1e-15 {
                    indices.push(i);
                    data.push(val);
                }
            }
            indptr.push(data.len());
        }

        Self {
            nrows,
            ncols,
            indptr,
            indices,
            data,
        }
    }

    /// Number of nonzero entries.
    pub fn nnz(&self) -> usize {
        self.data.len()
    }

    /// Get value at (row, col). Returns 0.0 if not stored.
    pub fn get(&self, row: usize, col: usize) -> f64 {
        let start = self.indptr[col];
        let end = self.indptr[col + 1];
        for k in start..end {
            if self.indices[k] == row {
                return self.data[k];
            }
        }
        0.0
    }

    /// Convert to dense row-major format.
    pub fn to_dense(&self) -> Vec<f64> {
        let mut dense = vec![0.0; self.nrows * self.ncols];
        for j in 0..self.ncols {
            for k in self.indptr[j]..self.indptr[j + 1] {
                dense[self.indices[k] * self.ncols + j] = self.data[k];
            }
        }
        dense
    }

    /// Matrix-vector multiplication: y = A * x.
    pub fn dot(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(x.len(), self.ncols, "vector length mismatch");
        let mut y = vec![0.0; self.nrows];
        for j in 0..self.ncols {
            for k in self.indptr[j]..self.indptr[j + 1] {
                y[self.indices[k]] += self.data[k] * x[j];
            }
        }
        y
    }

    /// Convert to CSR format.
    pub fn to_csr(&self) -> CsrMatrix {
        let mut row_counts = vec![0usize; self.nrows];
        for &r in &self.indices {
            row_counts[r] += 1;
        }

        let mut indptr = vec![0usize; self.nrows + 1];
        for i in 0..self.nrows {
            indptr[i + 1] = indptr[i] + row_counts[i];
        }

        let nnz = self.nnz();
        let mut indices = vec![0usize; nnz];
        let mut data = vec![0.0; nnz];
        let mut current = indptr.clone();

        for j in 0..self.ncols {
            for k in self.indptr[j]..self.indptr[j + 1] {
                let row = self.indices[k];
                let dest = current[row];
                indices[dest] = j;
                data[dest] = self.data[k];
                current[row] += 1;
            }
        }

        CsrMatrix::new(self.nrows, self.ncols, indptr, indices, data)
    }

    /// Sparse matrix addition: C = A + B.
    pub fn add(&self, other: &CscMatrix) -> CscMatrix {
        assert_eq!(self.nrows, other.nrows, "row count mismatch");
        assert_eq!(self.ncols, other.ncols, "column count mismatch");

        let mut indptr = vec![0usize];
        let mut indices = Vec::new();
        let mut data = Vec::new();

        for j in 0..self.ncols {
            let mut col_entries = std::collections::BTreeMap::new();

            for k in self.indptr[j]..self.indptr[j + 1] {
                *col_entries.entry(self.indices[k]).or_insert(0.0) += self.data[k];
            }
            for k in other.indptr[j]..other.indptr[j + 1] {
                *col_entries.entry(other.indices[k]).or_insert(0.0) += other.data[k];
            }

            for (row, val) in col_entries {
                if val.abs() > 1e-15 {
                    indices.push(row);
                    data.push(val);
                }
            }
            indptr.push(data.len());
        }

        CscMatrix::new(self.nrows, self.ncols, indptr, indices, data)
    }

    /// Scalar multiplication: B = alpha * A.
    pub fn scale(&self, alpha: f64) -> CscMatrix {
        let data: Vec<f64> = self.data.iter().map(|&v| v * alpha).collect();
        CscMatrix::new(
            self.nrows,
            self.ncols,
            self.indptr.clone(),
            self.indices.clone(),
            data,
        )
    }

    /// Density: nnz / (nrows * ncols).
    pub fn density(&self) -> f64 {
        if self.nrows == 0 || self.ncols == 0 {
            return 0.0;
        }
        self.nnz() as f64 / (self.nrows * self.ncols) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csc_from_dense() {
        let dense = vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 4.0, 0.0, 5.0];
        let m = CscMatrix::from_dense(&dense, 3, 3);
        assert_eq!(m.nnz(), 5);
        assert!((m.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((m.get(0, 2) - 2.0).abs() < 1e-10);
        assert!((m.get(2, 2) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_csc_dot() {
        let dense = vec![1.0, 2.0, 0.0, 3.0, 0.0, 4.0];
        let m = CscMatrix::from_dense(&dense, 2, 3);
        let x = vec![1.0, 2.0, 3.0];
        let y = m.dot(&x);
        assert!((y[0] - 5.0).abs() < 1e-10); // 1*1 + 2*2
        assert!((y[1] - 15.0).abs() < 1e-10); // 3*1 + 4*3
    }

    #[test]
    fn test_csc_to_dense_roundtrip() {
        let dense = vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 4.0, 0.0, 5.0];
        let m = CscMatrix::from_dense(&dense, 3, 3);
        let recovered = m.to_dense();
        for (a, b) in dense.iter().zip(recovered.iter()) {
            assert!((a - b).abs() < 1e-10);
        }
    }

    #[test]
    fn test_csc_add() {
        let a = CscMatrix::from_dense(&[1.0, 0.0, 0.0, 2.0], 2, 2);
        let b = CscMatrix::from_dense(&[0.0, 3.0, 4.0, 0.0], 2, 2);
        let c = a.add(&b);
        assert!((c.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((c.get(0, 1) - 3.0).abs() < 1e-10);
        assert!((c.get(1, 0) - 4.0).abs() < 1e-10);
        assert!((c.get(1, 1) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_csc_to_csr_roundtrip() {
        let dense = vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0];
        let csc = CscMatrix::from_dense(&dense, 2, 3);
        let csr = csc.to_csr();
        assert_eq!(csr.nrows, 2);
        assert_eq!(csr.ncols, 3);
        assert!((csr.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((csr.get(0, 2) - 2.0).abs() < 1e-10);
        assert!((csr.get(1, 1) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_csc_scale() {
        let a = CscMatrix::from_dense(&[1.0, 2.0, 3.0, 4.0], 2, 2);
        let b = a.scale(3.0);
        assert!((b.get(0, 0) - 3.0).abs() < 1e-10);
        assert!((b.get(1, 1) - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_csc_density() {
        let m = CscMatrix::from_dense(&[1.0, 0.0, 0.0, 2.0], 2, 2);
        assert!((m.density() - 0.5).abs() < 1e-10);
    }
}
