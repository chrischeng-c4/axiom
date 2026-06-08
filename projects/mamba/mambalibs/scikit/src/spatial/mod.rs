//! Spatial algorithms module — scipy.spatial equivalent.
//!
//! - **distance**: Euclidean, Manhattan, Chebyshev, Minkowski, cosine, distance matrix
//! - **kdtree**: KD-tree for nearest-neighbor queries
//! - **hull**: Convex hull via Graham scan

mod distance;
mod hull;
mod kdtree;

pub use distance::{
    cdist, chebyshev, cosine_distance, euclidean, manhattan, minkowski, pdist, squareform,
};
pub use hull::{convex_hull, convex_hull_area};
pub use kdtree::{KdTree, Neighbor};
