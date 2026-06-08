//! KD-tree for k-nearest neighbor and range queries.

/// A neighbor result from a KD-tree query.
#[derive(Debug, Clone)]
pub struct Neighbor {
    /// Index of the point in the original dataset.
    pub index: usize,
    /// Distance from the query point.
    pub distance: f64,
}

/// A KD-tree for efficient nearest-neighbor queries.
#[derive(Debug)]
pub struct KdTree {
    nodes: Vec<KdNode>,
    points: Vec<Vec<f64>>,
    dim: usize,
}

#[derive(Debug)]
struct KdNode {
    /// Index into the original point array.
    point_idx: usize,
    /// Splitting dimension.
    split_dim: usize,
    /// Left child index (None if leaf).
    left: Option<usize>,
    /// Right child index (None if leaf).
    right: Option<usize>,
}

impl KdTree {
    /// Build a KD-tree from a set of points.
    ///
    /// Each point must have the same dimensionality.
    pub fn new(points: &[Vec<f64>]) -> Self {
        assert!(!points.is_empty(), "points must not be empty");
        let dim = points[0].len();
        assert!(dim > 0, "point dimension must be > 0");
        for p in points {
            assert_eq!(p.len(), dim, "all points must have same dimension");
        }

        let stored = points.to_vec();
        let mut indices: Vec<usize> = (0..points.len()).collect();
        let mut nodes = Vec::with_capacity(points.len());

        Self::build_recursive(&stored, &mut indices, 0, &mut nodes, dim);

        KdTree {
            nodes,
            points: stored,
            dim,
        }
    }

    fn build_recursive(
        points: &[Vec<f64>],
        indices: &mut [usize],
        depth: usize,
        nodes: &mut Vec<KdNode>,
        dim: usize,
    ) -> Option<usize> {
        if indices.is_empty() {
            return None;
        }

        let split_dim = depth % dim;

        // Sort by split dimension
        indices.sort_by(|&a, &b| {
            points[a][split_dim]
                .partial_cmp(&points[b][split_dim])
                .unwrap()
        });

        let mid = indices.len() / 2;
        let point_idx = indices[mid];

        let node_idx = nodes.len();
        nodes.push(KdNode {
            point_idx,
            split_dim,
            left: None,
            right: None,
        });

        let (left_slice, right_slice) = {
            let (left, rest) = indices.split_at_mut(mid);
            let right = if rest.len() > 1 { &mut rest[1..] } else { &mut [] };
            (left.to_vec(), right.to_vec())
        };

        let left =
            Self::build_recursive(points, &mut left_slice.clone(), depth + 1, nodes, dim);
        let right =
            Self::build_recursive(points, &mut right_slice.clone(), depth + 1, nodes, dim);

        nodes[node_idx].left = left;
        nodes[node_idx].right = right;

        Some(node_idx)
    }

    /// Query k nearest neighbors to the given point.
    ///
    /// Returns neighbors sorted by distance (closest first).
    pub fn query(&self, point: &[f64], k: usize) -> Vec<Neighbor> {
        assert_eq!(point.len(), self.dim, "query point dimension mismatch");
        assert!(k > 0, "k must be > 0");

        let mut best: Vec<Neighbor> = Vec::with_capacity(k + 1);
        if !self.nodes.is_empty() {
            self.query_recursive(0, point, k, &mut best);
        }

        best.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        best.truncate(k);
        best
    }

    fn query_recursive(
        &self,
        node_idx: usize,
        point: &[f64],
        k: usize,
        best: &mut Vec<Neighbor>,
    ) {
        let node = &self.nodes[node_idx];
        let node_point = &self.points[node.point_idx];

        let dist = squared_dist(point, node_point).sqrt();

        // Insert this node if it could be among k-best
        let worst_dist = if best.len() < k {
            f64::INFINITY
        } else {
            best.iter()
                .map(|n| n.distance)
                .fold(f64::NEG_INFINITY, f64::max)
        };

        if dist < worst_dist || best.len() < k {
            best.push(Neighbor {
                index: node.point_idx,
                distance: dist,
            });
            // Keep sorted and trim to k
            if best.len() > k {
                best.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
                best.truncate(k);
            }
        }

        let split_dim = node.split_dim;
        let diff = point[split_dim] - node_point[split_dim];

        let (first, second) = if diff < 0.0 {
            (node.left, node.right)
        } else {
            (node.right, node.left)
        };

        if let Some(child) = first {
            self.query_recursive(child, point, k, best);
        }

        // Check if we need to explore the other side
        let worst_dist = if best.len() < k {
            f64::INFINITY
        } else {
            best.iter()
                .map(|n| n.distance)
                .fold(f64::NEG_INFINITY, f64::max)
        };

        if diff.abs() < worst_dist {
            if let Some(child) = second {
                self.query_recursive(child, point, k, best);
            }
        }
    }

    /// Find all points within `radius` of the query point.
    pub fn query_ball(&self, point: &[f64], radius: f64) -> Vec<Neighbor> {
        assert_eq!(point.len(), self.dim, "query point dimension mismatch");
        assert!(radius >= 0.0, "radius must be >= 0");

        let mut results = Vec::new();
        if !self.nodes.is_empty() {
            self.ball_recursive(0, point, radius, &mut results);
        }
        results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        results
    }

    fn ball_recursive(
        &self,
        node_idx: usize,
        point: &[f64],
        radius: f64,
        results: &mut Vec<Neighbor>,
    ) {
        let node = &self.nodes[node_idx];
        let node_point = &self.points[node.point_idx];

        let dist = squared_dist(point, node_point).sqrt();
        if dist <= radius {
            results.push(Neighbor {
                index: node.point_idx,
                distance: dist,
            });
        }

        let split_dim = node.split_dim;
        let diff = point[split_dim] - node_point[split_dim];

        let (first, second) = if diff < 0.0 {
            (node.left, node.right)
        } else {
            (node.right, node.left)
        };

        if let Some(child) = first {
            self.ball_recursive(child, point, radius, results);
        }

        if diff.abs() <= radius {
            if let Some(child) = second {
                self.ball_recursive(child, point, radius, results);
            }
        }
    }

    /// Number of points in the tree.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Whether the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Dimensionality of the points.
    pub fn dim(&self) -> usize {
        self.dim
    }
}

fn squared_dist(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdtree_query_1nn() {
        let points = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
            vec![1.0, 1.0],
        ];
        let tree = KdTree::new(&points);

        let neighbors = tree.query(&[0.1, 0.1], 1);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].index, 0);
    }

    #[test]
    fn test_kdtree_query_knn() {
        let points = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
            vec![1.0, 1.0],
            vec![5.0, 5.0],
        ];
        let tree = KdTree::new(&points);

        let neighbors = tree.query(&[0.5, 0.5], 3);
        assert_eq!(neighbors.len(), 3);
        // The 3 closest to (0.5, 0.5) should be indices 0,1,2,3 (all equidistant)
        // but only 3 returned
        for n in &neighbors {
            assert!(n.index < 4, "far point should not be among 3-NN");
        }
    }

    #[test]
    fn test_kdtree_query_ball() {
        let points = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
            vec![5.0, 5.0],
        ];
        let tree = KdTree::new(&points);

        let neighbors = tree.query_ball(&[0.0, 0.0], 1.5);
        assert_eq!(neighbors.len(), 3); // (0,0), (1,0), (0,1)
        for n in &neighbors {
            assert_ne!(n.index, 3, "far point should not be in ball");
        }
    }

    #[test]
    fn test_kdtree_3d() {
        let points = vec![
            vec![0.0, 0.0, 0.0],
            vec![1.0, 1.0, 1.0],
            vec![2.0, 2.0, 2.0],
        ];
        let tree = KdTree::new(&points);
        assert_eq!(tree.dim(), 3);

        let neighbors = tree.query(&[0.9, 0.9, 0.9], 1);
        assert_eq!(neighbors[0].index, 1);
    }

    #[test]
    fn test_kdtree_single_point() {
        let points = vec![vec![42.0, 7.0]];
        let tree = KdTree::new(&points);
        let neighbors = tree.query(&[0.0, 0.0], 1);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].index, 0);
    }
}
