//! QR code mask penalty evaluation.
//!
//! Implements the four penalty rules from ISO 18004 Section 8.8.2
//! used to select the optimal mask pattern.

/// Evaluate total penalty score for a QR matrix.
pub fn evaluate_penalty(grid: &[Vec<bool>], size: usize) -> u32 {
    penalty_rule1(grid, size)
        + penalty_rule2(grid, size)
        + penalty_rule3(grid, size)
        + penalty_rule4(grid, size)
}

/// Rule 1: Runs of same-color modules (>=5 consecutive) in rows/columns.
/// Penalty: N1 + (run_length - 5) for each run, where N1 = 3.
fn penalty_rule1(grid: &[Vec<bool>], size: usize) -> u32 {
    let mut penalty = 0u32;
    // Horizontal
    for r in 0..size {
        let mut count = 1u32;
        for c in 1..size {
            if grid[r][c] == grid[r][c - 1] {
                count += 1;
            } else {
                if count >= 5 {
                    penalty += count - 2;
                }
                count = 1;
            }
        }
        if count >= 5 {
            penalty += count - 2;
        }
    }
    // Vertical
    for c in 0..size {
        let mut count = 1u32;
        for r in 1..size {
            if grid[r][c] == grid[r - 1][c] {
                count += 1;
            } else {
                if count >= 5 {
                    penalty += count - 2;
                }
                count = 1;
            }
        }
        if count >= 5 {
            penalty += count - 2;
        }
    }
    penalty
}

/// Rule 2: 2x2 blocks of same color. Penalty: 3 per block.
fn penalty_rule2(grid: &[Vec<bool>], size: usize) -> u32 {
    let mut penalty = 0u32;
    for r in 0..size - 1 {
        for c in 0..size - 1 {
            let v = grid[r][c];
            if v == grid[r][c + 1] && v == grid[r + 1][c] && v == grid[r + 1][c + 1] {
                penalty += 3;
            }
        }
    }
    penalty
}

/// Rule 3: Finder-like patterns (1:1:3:1:1 + 4 white). Penalty: 40 per match.
fn penalty_rule3(grid: &[Vec<bool>], size: usize) -> u32 {
    let mut penalty = 0u32;
    let pattern_a: [bool; 11] = [
        true, false, true, true, true, false, true, false, false, false, false,
    ];
    let pattern_b: [bool; 11] = [
        false, false, false, false, true, false, true, true, true, false, true,
    ];

    for r in 0..size {
        for c in 0..=size.saturating_sub(11) {
            let slice: Vec<bool> = (0..11).map(|i| grid[r][c + i]).collect();
            if slice == pattern_a || slice == pattern_b {
                penalty += 40;
            }
        }
    }
    for c in 0..size {
        for r in 0..=size.saturating_sub(11) {
            let slice: Vec<bool> = (0..11).map(|i| grid[r + i][c]).collect();
            if slice == pattern_a || slice == pattern_b {
                penalty += 40;
            }
        }
    }
    penalty
}

/// Rule 4: Dark/light module ratio deviation from 50%. Penalty: 10 per 5% deviation.
fn penalty_rule4(grid: &[Vec<bool>], size: usize) -> u32 {
    let total = (size * size) as u32;
    let dark: u32 = grid.iter().flatten().filter(|&&v| v).count() as u32;
    let percent = (dark * 100) / total;
    let prev5 = (percent / 5) * 5;
    let next5 = prev5 + 5;
    let dev_prev = if prev5 >= 50 { prev5 - 50 } else { 50 - prev5 };
    let dev_next = if next5 >= 50 { next5 - 50 } else { 50 - next5 };
    let min_dev = dev_prev.min(dev_next);
    (min_dev / 5) * 10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_penalty_all_dark() {
        // Use 21x21 (V1 QR size) to avoid undersized grid issues in rule 3
        let size = 21;
        let grid = vec![vec![true; size]; size];
        let p = evaluate_penalty(&grid, size);
        assert!(p > 0); // Should have high penalty for uniform color
    }

    #[test]
    fn test_penalty_checkerboard() {
        // Checkerboard should have relatively low penalty
        let size = 21;
        let mut grid = vec![vec![false; size]; size];
        for r in 0..size {
            for c in 0..size {
                grid[r][c] = (r + c) % 2 == 0;
            }
        }
        let p = evaluate_penalty(&grid, size);
        // Rule 4 should be minimal (50% dark), rules 1-3 also minimal
        assert!(p < 100);
    }
}
