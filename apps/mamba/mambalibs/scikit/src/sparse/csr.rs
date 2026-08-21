//! Compressed Sparse Row (CSR) matrix format.

use super::csc::CscMatrix;

/// A sparse matrix in Compressed Sparse Row (CSR) format.
///
/// Stores nonzero values row by row. Efficient for row slicing
/// and matrix-vector multiplication.
#[derive(Debug, Clone)]
pub struct CsrMatrix {
    /// Number of rows.
    pub nrows: usize,
    /// Number of columns.
    pub ncols: usize,
    /// Row pointers: `indptr[i]..indptr[i+1]` gives the range of
    /// nonzero entries in row `i`. Length = nrows + 1.
    pub indptr: Vec<usize>,
    /// Column indices of nonzero entries. Length = nnz.
    pub indices: Vec<usize>,
    /// Values of nonzero entries. Length = nnz.
    pub data: Vec<f64>,
}

impl CsrMatrix {
    /// Create a CSR matrix from raw components.
    pub fn new(
        nrows: usize,
        ncols: usize,
        indptr: Vec<usize>,
        indices: Vec<usize>,
        data: Vec<f64>,
    ) -> Self {
        assert_eq!(indptr.len(), nrows + 1, "indptr length mismatch");
        assert_eq!(indices.len(), data.len(), "indices/data length mismatch");
        Self {
            nrows,
            ncols,
            indptr,
            indices,
            data,
        }
    }

    /// Create a CSR matrix from dense row-major data.
    pub fn from_dense(dense: &[f64], nrows: usize, ncols: usize) -> Self {
        assert_eq!(dense.len(), nrows * ncols, "dense data size mismatch");
        let mut indptr = vec![0usize];
        let mut indices = Vec::new();
        let mut data = Vec::new();

        for i in 0..nrows {
            for j in 0..ncols {
                let val = dense[i * ncols + j];
                if val.abs() > 1e-15 {
                    indices.push(j);
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

    /// Create a CSR matrix from coordinate (COO) triplets.
    pub fn from_coo(
        nrows: usize,
        ncols: usize,
        rows: &[usize],
        cols: &[usize],
        values: &[f64],
    ) -> Self {
        assert_eq!(rows.len(), cols.len(), "rows/cols length mismatch");
        assert_eq!(rows.len(), values.len(), "rows/values length mismatch");

        // Sort by row then column
        let mut entries: Vec<(usize, usize, f64)> = rows
            .iter()
            .zip(cols.iter())
            .zip(values.iter())
            .map(|((&r, &c), &v)| (r, c, v))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        let mut indptr = vec![0usize; nrows + 1];
        let mut indices = Vec::with_capacity(entries.len());
        let mut data = Vec::with_capacity(entries.len());

        for (r, c, v) in entries {
            if v.abs() > 1e-15 {
                indices.push(c);
                data.push(v);
                indptr[r + 1] += 1;
            }
        }

        // Cumulative sum
        for i in 1..=nrows {
            indptr[i] += indptr[i - 1];
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
        let start = self.indptr[row];
        let end = self.indptr[row + 1];
        for k in start..end {
            if self.indices[k] == col {
                return self.data[k];
            }
        }
        0.0
    }

    /// Convert to dense row-major format.
    pub fn to_dense(&self) -> Vec<f64> {
        let mut dense = vec![0.0; self.nrows * self.ncols];
        for i in 0..self.nrows {
            for k in self.indptr[i]..self.indptr[i + 1] {
                dense[i * self.ncols + self.indices[k]] = self.data[k];
            }
        }
        dense
    }

    /// Matrix-vector multiplication: y = A * x.
    pub fn dot(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(x.len(), self.ncols, "vector length mismatch");
        let mut y = vec![0.0; self.nrows];
        for i in 0..self.nrows {
            for k in self.indptr[i]..self.indptr[i + 1] {
                y[i] += self.data[k] * x[self.indices[k]];
            }
        }
        y
    }

    /// Sparse matrix addition: C = A + B.
    pub fn add(&self, other: &CsrMatrix) -> CsrMatrix {
        assert_eq!(self.nrows, other.nrows, "row count mismatch");
        assert_eq!(self.ncols, other.ncols, "column count mismatch");

        let mut indptr = vec![0usize];
        let mut indices = Vec::new();
        let mut data = Vec::new();

        for i in 0..self.nrows {
            let mut row_entries = std::collections::BTreeMap::new();

            for k in self.indptr[i]..self.indptr[i + 1] {
                *row_entries.entry(self.indices[k]).or_insert(0.0) += self.data[k];
            }
            for k in other.indptr[i]..other.indptr[i + 1] {
                *row_entries.entry(other.indices[k]).or_insert(0.0) += other.data[k];
            }

            for (col, val) in row_entries {
                if val.abs() > 1e-15 {
                    indices.push(col);
                    data.push(val);
                }
            }
            indptr.push(data.len());
        }

        CsrMatrix::new(self.nrows, self.ncols, indptr, indices, data)
    }

    /// Scalar multiplication: B = alpha * A.
    pub fn scale(&self, alpha: f64) -> CsrMatrix {
        let data: Vec<f64> = self.data.iter().map(|&v| v * alpha).collect();
        CsrMatrix::new(
            self.nrows,
            self.ncols,
            self.indptr.clone(),
            self.indices.clone(),
            data,
        )
    }

    /// Transpose the matrix (returns a new CSR matrix).
    pub fn transpose(&self) -> CsrMatrix {
        let csc = self.to_csc();
        // CSC of A is CSR of A^T
        CsrMatrix::new(csc.ncols, csc.nrows, csc.indptr, csc.indices, csc.data)
    }

    /// Convert to CSC format.
    pub fn to_csc(&self) -> CscMatrix {
        let mut col_counts = vec![0usize; self.ncols];
        for &c in &self.indices {
            col_counts[c] += 1;
        }

        let mut indptr = vec![0usize; self.ncols + 1];
        for j in 0..self.ncols {
            indptr[j + 1] = indptr[j] + col_counts[j];
        }

        let nnz = self.nnz();
        let mut indices = vec![0usize; nnz];
        let mut data = vec![0.0; nnz];
        let mut current = indptr.clone();

        for i in 0..self.nrows {
            for k in self.indptr[i]..self.indptr[i + 1] {
                let col = self.indices[k];
                let dest = current[col];
                indices[dest] = i;
                data[dest] = self.data[k];
                current[col] += 1;
            }
        }

        CscMatrix::new(self.nrows, self.ncols, indptr, indices, data)
    }

    /// Create an identity matrix of size n.
    pub fn eye(n: usize) -> Self {
        let indptr: Vec<usize> = (0..=n).collect();
        let indices: Vec<usize> = (0..n).collect();
        let data = vec![1.0; n];
        Self::new(n, n, indptr, indices, data)
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
    fn test_csr_from_dense() {
        let dense = vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 4.0, 0.0, 5.0];
        let m = CsrMatrix::from_dense(&dense, 3, 3);
        assert_eq!(m.nnz(), 5);
        assert!((m.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((m.get(0, 1) - 0.0).abs() < 1e-10);
        assert!((m.get(0, 2) - 2.0).abs() < 1e-10);
        assert!((m.get(2, 2) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_csr_dot() {
        let dense = vec![1.0, 2.0, 0.0, 3.0, 0.0, 4.0];
        let m = CsrMatrix::from_dense(&dense, 2, 3);
        let x = vec![1.0, 2.0, 3.0];
        let y = m.dot(&x);
        assert!((y[0] - 5.0).abs() < 1e-10); // 1*1 + 2*2
        assert!((y[1] - 15.0).abs() < 1e-10); // 3*1 + 4*3
    }

    #[test]
    fn test_csr_add() {
        let a = CsrMatrix::from_dense(&[1.0, 0.0, 0.0, 2.0], 2, 2);
        let b = CsrMatrix::from_dense(&[0.0, 3.0, 4.0, 0.0], 2, 2);
        let c = a.add(&b);
        assert!((c.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((c.get(0, 1) - 3.0).abs() < 1e-10);
        assert!((c.get(1, 0) - 4.0).abs() < 1e-10);
        assert!((c.get(1, 1) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_csr_scale() {
        let a = CsrMatrix::from_dense(&[1.0, 2.0, 3.0, 4.0], 2, 2);
        let b = a.scale(2.0);
        assert!((b.get(0, 0) - 2.0).abs() < 1e-10);
        assert!((b.get(1, 1) - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_csr_to_dense_roundtrip() {
        let dense = vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 4.0, 0.0, 5.0];
        let m = CsrMatrix::from_dense(&dense, 3, 3);
        let recovered = m.to_dense();
        for (a, b) in dense.iter().zip(recovered.iter()) {
            assert!((a - b).abs() < 1e-10);
        }
    }

    #[test]
    fn test_csr_from_coo() {
        let rows = vec![0, 1, 2, 0];
        let cols = vec![0, 1, 2, 2];
        let values = vec![1.0, 2.0, 3.0, 4.0];
        let m = CsrMatrix::from_coo(3, 3, &rows, &cols, &values);
        assert_eq!(m.nnz(), 4);
        assert!((m.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((m.get(0, 2) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_csr_eye() {
        let m = CsrMatrix::eye(3);
        assert_eq!(m.nnz(), 3);
        assert!((m.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((m.get(1, 1) - 1.0).abs() < 1e-10);
        assert!((m.get(2, 2) - 1.0).abs() < 1e-10);
        assert!((m.get(0, 1) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_csr_transpose() {
        let dense = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let m = CsrMatrix::from_dense(&dense, 2, 3);
        let mt = m.transpose();
        assert_eq!(mt.nrows, 3);
        assert_eq!(mt.ncols, 2);
        assert!((mt.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((mt.get(0, 1) - 4.0).abs() < 1e-10);
        assert!((mt.get(2, 0) - 3.0).abs() < 1e-10);
        assert!((mt.get(2, 1) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_csr_density() {
        let m = CsrMatrix::from_dense(&[1.0, 0.0, 0.0, 2.0], 2, 2);
        assert!((m.density() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_csr_to_csc() {
        let dense = vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0];
        let csr = CsrMatrix::from_dense(&dense, 2, 3);
        let csc = csr.to_csc();
        assert_eq!(csc.nrows, 2);
        assert_eq!(csc.ncols, 3);
        assert!((csc.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((csc.get(0, 2) - 2.0).abs() < 1e-10);
        assert!((csc.get(1, 1) - 3.0).abs() < 1e-10);
    }
}
