//! QR matrix construction: pattern placement, data filling, and masking.
//!
//! Builds the module grid from encoded data, applying finder/alignment/timing
//! patterns, format/version info, and selecting the optimal mask.

use super::penalty::evaluate_penalty;
use super::tables::{self, EcLevel};

/// Cell state in the QR matrix.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Cell {
    /// Not yet assigned.
    Empty,
    /// Function pattern (finder, alignment, timing, separator, format, version).
    Function(bool),
    /// Data/EC bit.
    Data(bool),
}

impl Cell {
    fn is_dark(self) -> bool {
        match self {
            Cell::Function(v) | Cell::Data(v) => v,
            Cell::Empty => false,
        }
    }

    fn is_empty(self) -> bool {
        matches!(self, Cell::Empty)
    }
}

/// Build the complete QR matrix for the given version, EC level, and codewords.
/// Returns a 2D boolean grid where `true` = dark module.
pub fn build_matrix(version: u8, ec: EcLevel, data_with_ec: &[u8]) -> Vec<Vec<bool>> {
    let size = tables::modules_per_side(version);
    let mut grid = vec![vec![Cell::Empty; size]; size];

    // 1. Place function patterns
    place_finder_patterns(&mut grid, size);
    place_separators(&mut grid, size);
    place_alignment_patterns(&mut grid, version);
    place_timing_patterns(&mut grid, size);
    place_dark_module(&mut grid, version);

    // Reserve format info areas (will fill later)
    reserve_format_info(&mut grid, size);
    // Reserve version info areas if needed
    if version >= 7 {
        reserve_version_info(&mut grid, size);
    }

    // 2. Place data bits
    place_data_bits(&mut grid, size, data_with_ec);

    // 3. Evaluate masks and pick the best one
    let best_mask = select_best_mask(&grid, size, version, ec);

    // 4. Apply the chosen mask to data cells
    apply_mask(&mut grid, size, best_mask);

    // 5. Write format info
    write_format_info(&mut grid, size, ec, best_mask);

    // 6. Write version info
    if version >= 7 {
        write_version_info(&mut grid, size, version);
    }

    // Convert to bool grid
    grid.iter()
        .map(|row| row.iter().map(|c| c.is_dark()).collect())
        .collect()
}

// ── Finder patterns ──────────────────────────────────────

fn place_finder_patterns(grid: &mut [Vec<Cell>], size: usize) {
    let positions = [(0, 0), (0, size - 7), (size - 7, 0)];
    for &(row, col) in &positions {
        place_finder_pattern(grid, row, col);
    }
}

fn place_finder_pattern(grid: &mut [Vec<Cell>], row: usize, col: usize) {
    #[rustfmt::skip]
    const PATTERN: [[bool; 7]; 7] = [
        [true,  true,  true,  true,  true,  true,  true],
        [true,  false, false, false, false, false, true],
        [true,  false, true,  true,  true,  false, true],
        [true,  false, true,  true,  true,  false, true],
        [true,  false, true,  true,  true,  false, true],
        [true,  false, false, false, false, false, true],
        [true,  true,  true,  true,  true,  true,  true],
    ];
    for (r, pattern_row) in PATTERN.iter().enumerate() {
        for (c, &val) in pattern_row.iter().enumerate() {
            grid[row + r][col + c] = Cell::Function(val);
        }
    }
}

// ── Separators ───────────────────────────────────────────

fn place_separators(grid: &mut [Vec<Cell>], size: usize) {
    // Horizontal and vertical separators around each finder pattern
    for i in 0..8 {
        // Top-left
        if i < size {
            set_fn(grid, 7, i, false); // bottom
            set_fn(grid, i, 7, false); // right
        }
        // Top-right
        set_fn(grid, 7, size - 8 + i, false);
        set_fn(grid, i, size - 8, false);
        // Bottom-left
        set_fn(grid, size - 8 + i, 7, false);
        set_fn(grid, size - 8, i, false);
    }
}

// ── Alignment patterns ───────────────────────────────────

fn place_alignment_patterns(grid: &mut [Vec<Cell>], version: u8) {
    let positions = tables::alignment_positions(version);
    if positions.is_empty() {
        return;
    }
    // All combinations of positions, except those overlapping finder patterns
    for &row in positions {
        for &col in positions {
            let r = row as usize;
            let c = col as usize;
            // Skip if overlapping a finder pattern area
            if overlaps_finder(r, c, tables::modules_per_side(version)) {
                continue;
            }
            place_alignment_pattern(grid, r, c);
        }
    }
}

fn overlaps_finder(row: usize, col: usize, size: usize) -> bool {
    // Top-left finder: (0..9, 0..9)
    if row <= 8 && col <= 8 {
        return true;
    }
    // Top-right finder: (0..9, size-9..)
    if row <= 8 && col + 8 >= size {
        return true;
    }
    // Bottom-left finder: (size-9.., 0..9)
    if row + 8 >= size && col <= 8 {
        return true;
    }
    false
}

fn place_alignment_pattern(grid: &mut [Vec<Cell>], center_r: usize, center_c: usize) {
    for dr in 0..5 {
        for dc in 0..5 {
            let r = center_r - 2 + dr;
            let c = center_c - 2 + dc;
            let dark = dr == 0 || dr == 4 || dc == 0 || dc == 4 || (dr == 2 && dc == 2);
            grid[r][c] = Cell::Function(dark);
        }
    }
}

// ── Timing patterns ──────────────────────────────────────

fn place_timing_patterns(grid: &mut [Vec<Cell>], size: usize) {
    for i in 8..size - 8 {
        let dark = i % 2 == 0;
        if grid[6][i].is_empty() {
            grid[6][i] = Cell::Function(dark);
        }
        if grid[i][6].is_empty() {
            grid[i][6] = Cell::Function(dark);
        }
    }
}

// ── Dark module ──────────────────────────────────────────

fn place_dark_module(grid: &mut [Vec<Cell>], version: u8) {
    let row = 4 * version as usize + 9;
    grid[row][8] = Cell::Function(true);
}

// ── Format info reservation ──────────────────────────────

fn reserve_format_info(grid: &mut [Vec<Cell>], size: usize) {
    // Around top-left finder pattern
    for i in 0..9 {
        if grid[8][i].is_empty() {
            grid[8][i] = Cell::Function(false);
        }
        if grid[i][8].is_empty() {
            grid[i][8] = Cell::Function(false);
        }
    }
    // Below top-right finder
    for i in 0..8 {
        if grid[8][size - 1 - i].is_empty() {
            grid[8][size - 1 - i] = Cell::Function(false);
        }
    }
    // Right of bottom-left finder
    for i in 0..7 {
        if grid[size - 1 - i][8].is_empty() {
            grid[size - 1 - i][8] = Cell::Function(false);
        }
    }
}

// ── Version info reservation ─────────────────────────────

fn reserve_version_info(grid: &mut [Vec<Cell>], size: usize) {
    // Bottom-left of top-right finder: 6x3 area
    for i in 0..6 {
        for j in 0..3 {
            if grid[i][size - 11 + j].is_empty() {
                grid[i][size - 11 + j] = Cell::Function(false);
            }
            if grid[size - 11 + j][i].is_empty() {
                grid[size - 11 + j][i] = Cell::Function(false);
            }
        }
    }
}

// ── Data bit placement ───────────────────────────────────

fn place_data_bits(grid: &mut [Vec<Cell>], size: usize, codewords: &[u8]) {
    let mut bit_idx = 0;
    let total_bits = codewords.len() * 8;

    // Data is placed in 2-column strips from right to left
    // Column 6 (timing pattern) is skipped
    let mut col = size as isize - 1;

    while col >= 0 {
        // Skip column 6 (timing pattern)
        if col == 6 {
            col -= 1;
            continue;
        }

        // Alternate upward/downward within each strip
        let upward = ((size as isize - 1 - col) / 2) % 2 == 0;

        for step in 0..size {
            let row = if upward { size - 1 - step } else { step };

            // Place in right column, then left column of the 2-col strip
            for dc in 0..2 {
                let c = (col - dc) as usize;
                if c >= size {
                    continue;
                }
                if !grid[row][c].is_empty() {
                    continue;
                }
                if bit_idx < total_bits {
                    let byte_idx = bit_idx / 8;
                    let bit_pos = 7 - (bit_idx % 8);
                    let dark = (codewords[byte_idx] >> bit_pos) & 1 == 1;
                    grid[row][c] = Cell::Data(dark);
                    bit_idx += 1;
                } else {
                    grid[row][c] = Cell::Data(false);
                }
            }
        }

        col -= 2;
    }
}

// ── Masking ──────────────────────────────────────────────

/// Evaluate all 8 mask patterns and return the best one (lowest penalty).
fn select_best_mask(grid: &[Vec<Cell>], size: usize, version: u8, ec: EcLevel) -> u8 {
    let mut best_mask = 0u8;
    let mut best_penalty = u32::MAX;

    for mask in 0..8u8 {
        let mut test = grid.to_vec();
        apply_mask(&mut test, size, mask);
        write_format_info(&mut test, size, ec, mask);
        if version >= 7 {
            write_version_info(&mut test, size, version);
        }

        let bool_grid: Vec<Vec<bool>> = test
            .iter()
            .map(|row| row.iter().map(|c| c.is_dark()).collect())
            .collect();
        let penalty = evaluate_penalty(&bool_grid, size);

        if penalty < best_penalty {
            best_penalty = penalty;
            best_mask = mask;
        }
    }

    best_mask
}

/// Apply a mask pattern to all data cells (XOR toggle).
fn apply_mask(grid: &mut [Vec<Cell>], size: usize, mask: u8) {
    for r in 0..size {
        for c in 0..size {
            if let Cell::Data(val) = grid[r][c] {
                if should_mask(mask, r, c) {
                    grid[r][c] = Cell::Data(!val);
                }
            }
        }
    }
}

/// Check if a cell should be toggled for the given mask pattern.
fn should_mask(mask: u8, row: usize, col: usize) -> bool {
    let r = row;
    let c = col;
    match mask {
        0 => (r + c) % 2 == 0,
        1 => r % 2 == 0,
        2 => c % 3 == 0,
        3 => (r + c) % 3 == 0,
        4 => (r / 2 + c / 3) % 2 == 0,
        5 => (r * c) % 2 + (r * c) % 3 == 0,
        6 => ((r * c) % 2 + (r * c) % 3) % 2 == 0,
        7 => ((r + c) % 2 + (r * c) % 3) % 2 == 0,
        _ => false,
    }
}

// ── Format info writing ─────────────────────────────────

fn write_format_info(grid: &mut [Vec<Cell>], size: usize, ec: EcLevel, mask: u8) {
    let bits = tables::format_info_bits(ec, mask);

    // Positions around top-left finder
    #[rustfmt::skip]
    let tl_positions: [(usize, usize); 15] = [
        (0, 8), (1, 8), (2, 8), (3, 8), (4, 8), (5, 8), (7, 8), (8, 8),
        (8, 7), (8, 5), (8, 4), (8, 3), (8, 2), (8, 1), (8, 0),
    ];

    for (i, &(r, c)) in tl_positions.iter().enumerate() {
        let dark = (bits >> (14 - i)) & 1 == 1;
        grid[r][c] = Cell::Function(dark);
    }

    // Positions near top-right and bottom-left finders
    // Top-right: row 8, columns size-1 down to size-8
    for i in 0..8 {
        let dark = (bits >> (14 - i)) & 1 == 1;
        grid[8][size - 1 - (7 - i)] = Cell::Function(dark);
    }
    // Bottom-left: column 8, rows size-7 to size-1
    for i in 0..7 {
        let dark = (bits >> (6 - i)) & 1 == 1;
        grid[size - 7 + i][8] = Cell::Function(dark);
    }
}

// ── Version info writing ─────────────────────────────────

fn write_version_info(grid: &mut [Vec<Cell>], size: usize, version: u8) {
    if let Some(bits) = tables::version_info_bits(version) {
        for i in 0..18 {
            let dark = (bits >> i) & 1 == 1;
            let row = i / 3;
            let col = i % 3;
            // Bottom-left area (left of bottom-left finder)
            grid[size - 11 + col][row] = Cell::Function(dark);
            // Top-right area (above top-right finder)
            grid[row][size - 11 + col] = Cell::Function(dark);
        }
    }
}

// ── Helper ───────────────────────────────────────────────

fn set_fn(grid: &mut [Vec<Cell>], row: usize, col: usize, dark: bool) {
    grid[row][col] = Cell::Function(dark);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_matrix_v1() {
        // Minimal test: build a version 1 QR code
        let data = vec![0u8; 26]; // V1 total codewords = 26
        let matrix = build_matrix(1, EcLevel::M, &data);
        assert_eq!(matrix.len(), 21);
        assert_eq!(matrix[0].len(), 21);
    }

    #[test]
    fn test_finder_pattern_corners() {
        let size = 21; // Version 1
        let mut grid = vec![vec![Cell::Empty; size]; size];
        place_finder_patterns(&mut grid, size);

        // Top-left corner should be dark
        assert!(matches!(grid[0][0], Cell::Function(true)));
        // Center of top-left finder
        assert!(matches!(grid[3][3], Cell::Function(true)));
        // Just inside border of top-left finder
        assert!(matches!(grid[1][1], Cell::Function(false)));
    }

    #[test]
    fn test_should_mask() {
        // Mask 0: (row + col) % 2 == 0
        assert!(should_mask(0, 0, 0));
        assert!(!should_mask(0, 0, 1));
        assert!(should_mask(0, 1, 1));
    }

    #[test]
    fn test_matrix_size() {
        for v in [1, 5, 10, 20, 40] {
            let size = tables::modules_per_side(v);
            let total = tables::total_codewords(v);
            let data = vec![0u8; total];
            let matrix = build_matrix(v, EcLevel::L, &data);
            assert_eq!(matrix.len(), size);
            assert_eq!(matrix[0].len(), size);
        }
    }
}
