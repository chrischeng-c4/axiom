//! Distance metrics and distance matrix computation.

/// Euclidean distance between two points.
pub fn euclidean(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len(), "dimension mismatch");
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f64>()
        .sqrt()
}

/// Manhattan (L1) distance between two points.
pub fn manhattan(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len(), "dimension mismatch");
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
}

/// Chebyshev (L-infinity) distance between two points.
pub fn chebyshev(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len(), "dimension mismatch");
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .fold(0.0_f64, f64::max)
}

/// Minkowski distance of order `p` between two points.
pub fn minkowski(a: &[f64], b: &[f64], p: f64) -> f64 {
    assert_eq!(a.len(), b.len(), "dimension mismatch");
    assert!(p >= 1.0, "p must be >= 1");
    if p.is_infinite() {
        return chebyshev(a, b);
    }
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs().powf(p))
        .sum::<f64>()
        .powf(1.0 / p)
}

/// Cosine distance: 1 - cosine_similarity.
pub fn cosine_distance(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len(), "dimension mismatch");
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a < 1e-15 || norm_b < 1e-15 {
        return 1.0;
    }
    1.0 - dot / (norm_a * norm_b)
}

/// Compute pairwise distance matrix (condensed form) for a set of points.
///
/// Returns a condensed distance vector of length n*(n-1)/2,
/// where n is the number of points.
pub fn pdist(points: &[Vec<f64>], metric: &str) -> Vec<f64> {
    let n = points.len();
    let mut result = Vec::with_capacity(n * (n - 1) / 2);
    for i in 0..n {
        for j in i + 1..n {
            let d = match metric {
                "euclidean" => euclidean(&points[i], &points[j]),
                "manhattan" | "cityblock" => manhattan(&points[i], &points[j]),
                "chebyshev" => chebyshev(&points[i], &points[j]),
                "cosine" => cosine_distance(&points[i], &points[j]),
                _ => euclidean(&points[i], &points[j]),
            };
            result.push(d);
        }
    }
    result
}

/// Compute distance matrix between two sets of points.
///
/// Returns a flattened row-major matrix of shape (m, n).
pub fn cdist(xa: &[Vec<f64>], xb: &[Vec<f64>], metric: &str) -> Vec<f64> {
    let m = xa.len();
    let n = xb.len();
    let mut result = Vec::with_capacity(m * n);
    for a in xa {
        for b in xb {
            let d = match metric {
                "euclidean" => euclidean(a, b),
                "manhattan" | "cityblock" => manhattan(a, b),
                "chebyshev" => chebyshev(a, b),
                "cosine" => cosine_distance(a, b),
                _ => euclidean(a, b),
            };
            result.push(d);
        }
    }
    result
}

/// Convert condensed distance vector to full square distance matrix (flattened).
pub fn squareform(condensed: &[f64]) -> Vec<f64> {
    // n*(n-1)/2 = len => n = (1 + sqrt(1 + 8*len)) / 2
    let len = condensed.len();
    let n = ((1.0 + (1.0 + 8.0 * len as f64).sqrt()) / 2.0).round() as usize;
    assert_eq!(n * (n - 1) / 2, len, "invalid condensed vector length");

    let mut matrix = vec![0.0; n * n];
    let mut idx = 0;
    for i in 0..n {
        for j in i + 1..n {
            matrix[i * n + j] = condensed[idx];
            matrix[j * n + i] = condensed[idx];
            idx += 1;
        }
    }
    matrix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((euclidean(&a, &b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_manhattan() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((manhattan(&a, &b) - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_chebyshev() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((chebyshev(&a, &b) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_minkowski_p2_eq_euclidean() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        assert!((minkowski(&a, &b, 2.0) - euclidean(&a, &b)).abs() < 1e-10);
    }

    #[test]
    fn test_minkowski_p1_eq_manhattan() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        assert!((minkowski(&a, &b, 1.0) - manhattan(&a, &b)).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_distance() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_distance(&a, &b) - 1.0).abs() < 1e-10);

        let c = vec![1.0, 0.0];
        assert!((cosine_distance(&a, &c) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_pdist() {
        let pts = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![0.0, 1.0]];
        let d = pdist(&pts, "euclidean");
        assert_eq!(d.len(), 3); // 3 choose 2
        assert!((d[0] - 1.0).abs() < 1e-10); // (0,0)-(1,0)
        assert!((d[1] - 1.0).abs() < 1e-10); // (0,0)-(0,1)
        assert!((d[2] - 2.0_f64.sqrt()).abs() < 1e-10); // (1,0)-(0,1)
    }

    #[test]
    fn test_cdist() {
        let xa = vec![vec![0.0, 0.0], vec![1.0, 1.0]];
        let xb = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let d = cdist(&xa, &xb, "euclidean");
        assert_eq!(d.len(), 4); // 2x2
        assert!((d[0] - 1.0).abs() < 1e-10); // (0,0)-(1,0)
        assert!((d[1] - 1.0).abs() < 1e-10); // (0,0)-(0,1)
    }

    #[test]
    fn test_squareform() {
        let condensed = vec![1.0, 2.0, 3.0]; // 3 points
        let sq = squareform(&condensed);
        assert_eq!(sq.len(), 9); // 3x3
        assert!((sq[0] - 0.0).abs() < 1e-10); // diagonal
        assert!((sq[1] - 1.0).abs() < 1e-10); // (0,1)
        assert!((sq[3] - 1.0).abs() < 1e-10); // (1,0) symmetric
    }
}
