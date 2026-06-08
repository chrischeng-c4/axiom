//! Convex hull algorithms (Graham scan).

/// Compute the 2D convex hull using Graham scan.
///
/// Returns indices of points on the hull in counter-clockwise order.
/// Points are given as `(x, y)` pairs.
pub fn convex_hull(points: &[(f64, f64)]) -> Vec<usize> {
    let n = points.len();
    if n < 3 {
        return (0..n).collect();
    }

    // Find the bottom-most point (and leftmost if tied)
    let mut pivot = 0;
    for i in 1..n {
        if points[i].1 < points[pivot].1
            || (points[i].1 == points[pivot].1 && points[i].0 < points[pivot].0)
        {
            pivot = i;
        }
    }

    // Sort by polar angle with respect to pivot
    let mut indices: Vec<usize> = (0..n).collect();
    indices.swap(0, pivot);
    let p0 = points[pivot];

    indices[1..].sort_by(|&a, &b| {
        let angle_a = (points[a].1 - p0.1).atan2(points[a].0 - p0.0);
        let angle_b = (points[b].1 - p0.1).atan2(points[b].0 - p0.0);
        angle_a
            .partial_cmp(&angle_b)
            .unwrap()
            .then_with(|| {
                let da = sq_dist_2d(p0, points[a]);
                let db = sq_dist_2d(p0, points[b]);
                da.partial_cmp(&db).unwrap()
            })
    });

    // Graham scan
    let mut hull: Vec<usize> = Vec::with_capacity(n);
    for &idx in &indices {
        while hull.len() >= 2 {
            let a = hull[hull.len() - 2];
            let b = hull[hull.len() - 1];
            if cross(points[a], points[b], points[idx]) <= 0.0 {
                hull.pop();
            } else {
                break;
            }
        }
        hull.push(idx);
    }

    hull
}

/// Compute the area enclosed by the convex hull of 2D points.
///
/// Uses the shoelace formula on the hull vertices.
pub fn convex_hull_area(points: &[(f64, f64)]) -> f64 {
    let hull_indices = convex_hull(points);
    if hull_indices.len() < 3 {
        return 0.0;
    }

    let hull_pts: Vec<(f64, f64)> = hull_indices.iter().map(|&i| points[i]).collect();
    shoelace_area(&hull_pts)
}

/// Shoelace formula for polygon area.
fn shoelace_area(polygon: &[(f64, f64)]) -> f64 {
    let n = polygon.len();
    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += polygon[i].0 * polygon[j].1;
        area -= polygon[j].0 * polygon[i].1;
    }
    area.abs() / 2.0
}

/// Cross product of vectors OA and OB where O=(ox,oy), A=(ax,ay), B=(bx,by).
fn cross(o: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
}

fn sq_dist_2d(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convex_hull_square() {
        let points = vec![
            (0.0, 0.0),
            (1.0, 0.0),
            (1.0, 1.0),
            (0.0, 1.0),
            (0.5, 0.5), // interior point
        ];
        let hull = convex_hull(&points);
        assert_eq!(hull.len(), 4);
        // Interior point (0.5, 0.5) should not be on hull
        assert!(!hull.contains(&4));
    }

    #[test]
    fn test_convex_hull_triangle() {
        let points = vec![(0.0, 0.0), (4.0, 0.0), (2.0, 3.0)];
        let hull = convex_hull(&points);
        assert_eq!(hull.len(), 3);
    }

    #[test]
    fn test_convex_hull_area_square() {
        let points = vec![
            (0.0, 0.0),
            (2.0, 0.0),
            (2.0, 2.0),
            (0.0, 2.0),
        ];
        let area = convex_hull_area(&points);
        assert!((area - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_convex_hull_area_with_interior() {
        let points = vec![
            (0.0, 0.0),
            (4.0, 0.0),
            (4.0, 4.0),
            (0.0, 4.0),
            (1.0, 1.0), // interior
            (2.0, 2.0), // interior
        ];
        let area = convex_hull_area(&points);
        assert!((area - 16.0).abs() < 1e-10);
    }

    #[test]
    fn test_convex_hull_collinear() {
        let points = vec![(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)];
        let hull = convex_hull(&points);
        assert!(hull.len() <= 3);
    }

    #[test]
    fn test_convex_hull_two_points() {
        let points = vec![(0.0, 0.0), (1.0, 1.0)];
        let hull = convex_hull(&points);
        assert_eq!(hull.len(), 2);
    }
}
