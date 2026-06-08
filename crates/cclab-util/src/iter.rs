//! Iteration utilities inspired by Python's `itertools` and `boltons.iterutils`.
//!
//! Provides ergonomic helpers for common iterator patterns.
//!
//! # Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | `chunked` | Split into fixed-size chunks |
//! | `windowed` | Sliding window over elements |
//! | `first` | Get first element or None |
//! | `one` | Expect exactly one element |
//! | `unique` | Deduplicate preserving order |
//! | `flatten` | Flatten nested iterables |
//! | `partition` | Split by predicate |
//! | `pairwise` | Consecutive pairs |

use std::collections::HashSet;
use std::hash::Hash;

/// Split a slice into chunks of the given size.
///
/// The last chunk may be shorter if the slice length is not evenly divisible.
///
/// # Examples
///
/// ```
/// use cclab_util::iter::chunked;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let chunks = chunked(&data, 2);
/// assert_eq!(chunks, vec![vec![1, 2], vec![3, 4], vec![5]]);
///
/// let chunks = chunked(&data, 3);
/// assert_eq!(chunks, vec![vec![1, 2, 3], vec![4, 5]]);
/// ```
pub fn chunked<T: Clone>(slice: &[T], chunk_size: usize) -> Vec<Vec<T>> {
    if chunk_size == 0 {
        return Vec::new();
    }
    slice.chunks(chunk_size).map(|c| c.to_vec()).collect()
}

/// Return a sliding window view over a slice.
///
/// Each window is a `Vec<T>` of exactly `window_size` elements.
/// Returns an empty vec if the slice is shorter than the window size.
///
/// # Examples
///
/// ```
/// use cclab_util::iter::windowed;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let windows = windowed(&data, 3);
/// assert_eq!(windows, vec![
///     vec![1, 2, 3],
///     vec![2, 3, 4],
///     vec![3, 4, 5],
/// ]);
/// ```
pub fn windowed<T: Clone>(slice: &[T], window_size: usize) -> Vec<Vec<T>> {
    if window_size == 0 || slice.len() < window_size {
        return Vec::new();
    }
    slice.windows(window_size).map(|w| w.to_vec()).collect()
}

/// Return the first element of a slice, or `None` if empty.
///
/// # Examples
///
/// ```
/// use cclab_util::iter::first;
///
/// assert_eq!(first(&[10, 20, 30]), Some(&10));
/// assert_eq!(first::<i32>(&[]), None);
/// ```
pub fn first<T>(slice: &[T]) -> Option<&T> {
    slice.first()
}

/// Return the element if the slice contains exactly one element.
///
/// Returns `Err` with a descriptive message if the slice is empty or has
/// more than one element.
///
/// # Examples
///
/// ```
/// use cclab_util::iter::one;
///
/// assert_eq!(one(&[42]), Ok(&42));
/// assert!(one::<i32>(&[]).is_err());
/// assert!(one(&[1, 2]).is_err());
/// ```
pub fn one<T>(slice: &[T]) -> Result<&T, &'static str> {
    match slice.len() {
        0 => Err("expected one element, got zero"),
        1 => Ok(&slice[0]),
        _ => Err("expected one element, got multiple"),
    }
}

/// Deduplicate elements preserving first-occurrence order.
///
/// # Examples
///
/// ```
/// use cclab_util::iter::unique;
///
/// let data = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
/// assert_eq!(unique(&data), vec![3, 1, 4, 5, 9, 2, 6]);
/// ```
pub fn unique<T: Clone + Eq + Hash>(slice: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in slice {
        if seen.insert(item.clone()) {
            result.push(item.clone());
        }
    }
    result
}

/// Flatten a nested slice of slices into a single `Vec`.
///
/// # Examples
///
/// ```
/// use cclab_util::iter::flatten;
///
/// let nested = vec![vec![1, 2], vec![3], vec![4, 5, 6]];
/// assert_eq!(flatten(&nested), vec![1, 2, 3, 4, 5, 6]);
/// ```
pub fn flatten<T: Clone>(nested: &[Vec<T>]) -> Vec<T> {
    nested.iter().flat_map(|v| v.iter().cloned()).collect()
}

/// Split elements into two groups based on a predicate.
///
/// Returns `(truthy, falsy)` where `truthy` contains elements for which
/// the predicate returns `true`, and `falsy` contains the rest.
///
/// # Examples
///
/// ```
/// use cclab_util::iter::partition;
///
/// let data = vec![1, 2, 3, 4, 5, 6];
/// let (evens, odds) = partition(&data, |x| x % 2 == 0);
/// assert_eq!(evens, vec![2, 4, 6]);
/// assert_eq!(odds, vec![1, 3, 5]);
/// ```
pub fn partition<T: Clone, F: Fn(&T) -> bool>(slice: &[T], predicate: F) -> (Vec<T>, Vec<T>) {
    let mut truthy = Vec::new();
    let mut falsy = Vec::new();
    for item in slice {
        if predicate(item) {
            truthy.push(item.clone());
        } else {
            falsy.push(item.clone());
        }
    }
    (truthy, falsy)
}

/// Return consecutive pairs from a slice.
///
/// Each pair `(a, b)` consists of adjacent elements. Returns an empty vec
/// if the slice has fewer than 2 elements.
///
/// # Examples
///
/// ```
/// use cclab_util::iter::pairwise;
///
/// let data = vec![1, 2, 3, 4];
/// let pairs = pairwise(&data);
/// assert_eq!(pairs, vec![(1, 2), (2, 3), (3, 4)]);
///
/// assert_eq!(pairwise::<i32>(&[]), Vec::new());
/// assert_eq!(pairwise(&[1]), Vec::new());
/// ```
pub fn pairwise<T: Clone>(slice: &[T]) -> Vec<(T, T)> {
    if slice.len() < 2 {
        return Vec::new();
    }
    slice
        .windows(2)
        .map(|w| (w[0].clone(), w[1].clone()))
        .collect()
}

/// Return every nth element from a slice.
///
/// # Examples
///
/// ```
/// use cclab_util::iter::every_nth;
///
/// let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
/// assert_eq!(every_nth(&data, 3), vec![0, 3, 6, 9]);
/// assert_eq!(every_nth(&data, 5), vec![0, 5]);
/// ```
pub fn every_nth<T: Clone>(slice: &[T], n: usize) -> Vec<T> {
    if n == 0 {
        return Vec::new();
    }
    slice.iter().step_by(n).cloned().collect()
}

/// Interleave two slices element by element.
///
/// If slices have different lengths, remaining elements from the longer
/// slice are appended at the end.
///
/// # Examples
///
/// ```
/// use cclab_util::iter::interleave;
///
/// assert_eq!(interleave(&[1, 3, 5], &[2, 4, 6]), vec![1, 2, 3, 4, 5, 6]);
/// assert_eq!(interleave(&[1, 3], &[2, 4, 6, 8]), vec![1, 2, 3, 4, 6, 8]);
/// ```
pub fn interleave<T: Clone>(a: &[T], b: &[T]) -> Vec<T> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    let mut ai = a.iter();
    let mut bi = b.iter();

    loop {
        match (ai.next(), bi.next()) {
            (Some(x), Some(y)) => {
                result.push(x.clone());
                result.push(y.clone());
            }
            (Some(x), None) => {
                result.push(x.clone());
                result.extend(ai.cloned());
                break;
            }
            (None, Some(y)) => {
                result.push(y.clone());
                result.extend(bi.cloned());
                break;
            }
            (None, None) => break,
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunked() {
        assert_eq!(
            chunked(&[1, 2, 3, 4, 5], 2),
            vec![vec![1, 2], vec![3, 4], vec![5]]
        );
        assert_eq!(chunked(&[1, 2, 3], 3), vec![vec![1, 2, 3]]);
        assert_eq!(chunked(&[1, 2, 3], 5), vec![vec![1, 2, 3]]);
        assert_eq!(chunked::<i32>(&[], 2), Vec::<Vec<i32>>::new());
        assert_eq!(chunked(&[1, 2], 0), Vec::<Vec<i32>>::new());
    }

    #[test]
    fn test_windowed() {
        assert_eq!(
            windowed(&[1, 2, 3, 4], 2),
            vec![vec![1, 2], vec![2, 3], vec![3, 4]]
        );
        assert_eq!(windowed(&[1, 2, 3], 4), Vec::<Vec<i32>>::new());
        assert_eq!(windowed(&[1, 2, 3], 0), Vec::<Vec<i32>>::new());
        assert_eq!(windowed(&[1, 2, 3], 3), vec![vec![1, 2, 3]]);
    }

    #[test]
    fn test_first() {
        assert_eq!(first(&[10, 20]), Some(&10));
        assert_eq!(first::<i32>(&[]), None);
    }

    #[test]
    fn test_one() {
        assert_eq!(one(&[42]), Ok(&42));
        assert!(one::<i32>(&[]).is_err());
        assert!(one(&[1, 2]).is_err());
    }

    #[test]
    fn test_unique() {
        assert_eq!(unique(&[1, 2, 3, 2, 1]), vec![1, 2, 3]);
        assert_eq!(unique(&["a", "b", "a", "c"]), vec!["a", "b", "c"]);
        assert_eq!(unique::<i32>(&[]), Vec::<i32>::new());
    }

    #[test]
    fn test_flatten() {
        assert_eq!(
            flatten(&[vec![1, 2], vec![3], vec![4, 5]]),
            vec![1, 2, 3, 4, 5]
        );
        assert_eq!(flatten::<i32>(&[]), Vec::<i32>::new());
        assert_eq!(flatten(&[vec![], vec![1]]), vec![1]);
    }

    #[test]
    fn test_partition() {
        let (evens, odds) = partition(&[1, 2, 3, 4, 5, 6], |x| x % 2 == 0);
        assert_eq!(evens, vec![2, 4, 6]);
        assert_eq!(odds, vec![1, 3, 5]);
    }

    #[test]
    fn test_pairwise() {
        assert_eq!(pairwise(&[1, 2, 3, 4]), vec![(1, 2), (2, 3), (3, 4)]);
        assert_eq!(pairwise(&[1, 2]), vec![(1, 2)]);
        assert_eq!(pairwise(&[1]), Vec::<(i32, i32)>::new());
        assert_eq!(pairwise::<i32>(&[]), Vec::<(i32, i32)>::new());
    }

    #[test]
    fn test_every_nth() {
        assert_eq!(every_nth(&[0, 1, 2, 3, 4, 5], 2), vec![0, 2, 4]);
        assert_eq!(every_nth(&[0, 1, 2, 3, 4, 5], 3), vec![0, 3]);
        assert_eq!(every_nth(&[0, 1, 2], 0), Vec::<i32>::new());
    }

    #[test]
    fn test_interleave() {
        assert_eq!(interleave(&[1, 3, 5], &[2, 4, 6]), vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(interleave(&[1], &[2, 3, 4]), vec![1, 2, 3, 4]);
        assert_eq!(interleave::<i32>(&[], &[1, 2]), vec![1, 2]);
    }
}
