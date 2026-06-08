//! QR code specification tables.
//!
//! Capacity, error correction, alignment pattern, and format lookup tables
//! per ISO/IEC 18004.

/// Error correction levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcLevel {
    L = 0, // ~7% recovery
    M = 1, // ~15% recovery
    Q = 2, // ~25% recovery
    H = 3, // ~30% recovery
}

impl EcLevel {
    /// Format bits for this EC level (used in format information).
    pub fn format_bits(self) -> u8 {
        match self {
            EcLevel::L => 0b01,
            EcLevel::M => 0b00,
            EcLevel::Q => 0b11,
            EcLevel::H => 0b10,
        }
    }
}

/// Encoding mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Numeric,      // 0-9
    Alphanumeric, // 0-9, A-Z, space, $%*+-./:
    Byte,         // ISO 8859-1 / UTF-8
}

impl Mode {
    /// Mode indicator bits (4-bit).
    pub fn indicator(self) -> u8 {
        match self {
            Mode::Numeric => 0b0001,
            Mode::Alphanumeric => 0b0010,
            Mode::Byte => 0b0100,
        }
    }

    /// Character count indicator length in bits for a given version.
    pub fn cci_bits(self, version: u8) -> usize {
        match self {
            Mode::Numeric => match version {
                1..=9 => 10,
                10..=26 => 12,
                _ => 14,
            },
            Mode::Alphanumeric => match version {
                1..=9 => 9,
                10..=26 => 11,
                _ => 13,
            },
            Mode::Byte => match version {
                1..=9 => 8,
                10..=26 => 16,
                _ => 16,
            },
        }
    }
}

/// Alphanumeric encoding character set.
pub const ALPHANUMERIC_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

/// Look up alphanumeric character value. Returns None if not in charset.
pub fn alphanumeric_value(ch: u8) -> Option<u8> {
    ALPHANUMERIC_CHARS
        .iter()
        .position(|&c| c == ch)
        .map(|p| p as u8)
}

/// Detect the best encoding mode for the given data.
pub fn detect_mode(data: &[u8]) -> Mode {
    if data.iter().all(|&b| b.is_ascii_digit()) {
        Mode::Numeric
    } else if data.iter().all(|&b| alphanumeric_value(b).is_some()) {
        Mode::Alphanumeric
    } else {
        Mode::Byte
    }
}

// ── Capacity table ────────────────────────────────────────
// Data capacity in bytes (data codewords) for each version and EC level.
// Index: [version-1][ec_level as usize]
// Source: ISO 18004, Table 7

/// (total_codewords, ec_codewords_per_block, num_blocks_group1, data_cw_group1, num_blocks_group2, data_cw_group2)
pub type VersionEcParams = (usize, usize, usize, usize, usize, usize);

/// Get error correction parameters for a version and EC level.
/// Returns (total_codewords, ec_per_block, g1_blocks, g1_data_cw, g2_blocks, g2_data_cw)
pub fn ec_params(version: u8, ec: EcLevel) -> VersionEcParams {
    let idx = ((version as usize - 1) * 4) + ec as usize;
    EC_PARAMS_TABLE[idx]
}

/// Data capacity in bytes for a (version, ec_level).
pub fn data_capacity(version: u8, ec: EcLevel) -> usize {
    let p = ec_params(version, ec);
    p.2 * p.3 + p.4 * p.5
}

/// Total codewords for a version.
pub fn total_codewords(version: u8) -> usize {
    ec_params(version, EcLevel::L).0
}

/// Module count (side length) for a version.
pub fn modules_per_side(version: u8) -> usize {
    (version as usize) * 4 + 17
}

/// Alignment pattern center positions for a version. Empty for version 1.
pub fn alignment_positions(version: u8) -> &'static [u8] {
    if version < 2 || version > 40 {
        return &[];
    }
    ALIGNMENT_POSITIONS[version as usize - 2]
}

// ── EC Parameters table ───────────────────────────────────
// (total_cw, ec_per_block, g1_blocks, g1_data, g2_blocks, g2_data)
// Versions 1-40, L/M/Q/H in order
#[rustfmt::skip]
const EC_PARAMS_TABLE: &[VersionEcParams] = &[
    // Version 1
    (26, 7, 1, 19, 0, 0),   // L
    (26, 10, 1, 16, 0, 0),  // M
    (26, 13, 1, 13, 0, 0),  // Q
    (26, 17, 1, 9, 0, 0),   // H
    // Version 2
    (44, 10, 1, 34, 0, 0),  // L
    (44, 16, 1, 28, 0, 0),  // M
    (44, 22, 1, 22, 0, 0),  // Q
    (44, 28, 1, 16, 0, 0),  // H
    // Version 3
    (70, 15, 1, 55, 0, 0),  // L
    (70, 26, 1, 44, 0, 0),  // M
    (70, 18, 2, 17, 0, 0),  // Q
    (70, 22, 2, 13, 0, 0),  // H
    // Version 4
    (100, 20, 1, 80, 0, 0), // L
    (100, 18, 2, 32, 0, 0), // M
    (100, 26, 2, 24, 0, 0), // Q
    (100, 16, 4, 9, 0, 0),  // H
    // Version 5
    (134, 26, 1, 108, 0, 0), // L
    (134, 24, 2, 43, 0, 0),  // M
    (134, 18, 2, 15, 2, 16), // Q
    (134, 22, 2, 11, 2, 12), // H
    // Version 6
    (172, 18, 2, 68, 0, 0),  // L
    (172, 16, 4, 27, 0, 0),  // M
    (172, 24, 4, 19, 0, 0),  // Q
    (172, 28, 4, 15, 0, 0),  // H
    // Version 7
    (196, 20, 2, 78, 0, 0),  // L
    (196, 18, 4, 31, 0, 0),  // M
    (196, 18, 2, 14, 4, 15), // Q
    (196, 26, 4, 13, 1, 14), // H
    // Version 8
    (242, 24, 2, 97, 0, 0),  // L
    (242, 22, 2, 38, 2, 39), // M
    (242, 22, 4, 18, 2, 19), // Q
    (242, 26, 4, 14, 2, 15), // H
    // Version 9
    (292, 30, 2, 116, 0, 0), // L
    (292, 22, 3, 36, 2, 37), // M
    (292, 20, 4, 16, 4, 17), // Q
    (292, 24, 4, 12, 4, 13), // H
    // Version 10
    (346, 18, 2, 68, 2, 69), // L
    (346, 26, 4, 43, 1, 44), // M
    (346, 24, 6, 19, 2, 20), // Q
    (346, 28, 6, 15, 2, 16), // H
    // Version 11
    (404, 20, 4, 81, 0, 0),  // L
    (404, 30, 1, 50, 4, 51), // M
    (404, 28, 4, 22, 4, 23), // Q
    (404, 24, 3, 12, 8, 13), // H
    // Version 12
    (466, 24, 2, 92, 2, 93), // L
    (466, 22, 6, 36, 2, 37), // M
    (466, 26, 4, 20, 6, 21), // Q
    (466, 28, 7, 14, 4, 15), // H
    // Version 13
    (532, 26, 4, 107, 0, 0),  // L
    (532, 22, 8, 37, 1, 38),  // M
    (532, 24, 8, 20, 4, 21),  // Q
    (532, 22, 12, 11, 4, 12), // H
    // Version 14
    (581, 30, 3, 115, 1, 116), // L
    (581, 24, 4, 40, 5, 41),   // M
    (581, 20, 11, 16, 5, 17),  // Q
    (581, 24, 11, 12, 5, 13),  // H
    // Version 15
    (655, 22, 5, 87, 1, 88),   // L
    (655, 24, 5, 41, 5, 42),   // M
    (655, 30, 5, 24, 7, 25),   // Q
    (655, 24, 11, 12, 7, 13),  // H
    // Version 16
    (733, 24, 5, 98, 1, 99),   // L
    (733, 28, 7, 45, 3, 46),   // M
    (733, 24, 15, 19, 2, 20),  // Q
    (733, 30, 3, 15, 13, 16),  // H
    // Version 17
    (815, 28, 1, 107, 5, 108), // L
    (815, 28, 10, 46, 1, 47),  // M
    (815, 28, 1, 22, 15, 23),  // Q
    (815, 28, 2, 14, 17, 15),  // H
    // Version 18
    (901, 30, 5, 120, 1, 121), // L
    (901, 26, 9, 43, 4, 44),   // M
    (901, 28, 17, 22, 1, 23),  // Q
    (901, 28, 2, 14, 19, 15),  // H
    // Version 19
    (991, 28, 3, 113, 4, 114), // L
    (991, 26, 3, 44, 11, 45),  // M
    (991, 26, 17, 21, 4, 22),  // Q
    (991, 26, 9, 13, 16, 14),  // H
    // Version 20
    (1085, 28, 3, 107, 5, 108), // L
    (1085, 26, 3, 41, 13, 42),  // M
    (1085, 30, 15, 24, 5, 25),  // Q
    (1085, 28, 15, 15, 10, 16), // H
    // Version 21
    (1156, 28, 4, 116, 4, 117), // L
    (1156, 26, 17, 42, 0, 0),   // M
    (1156, 28, 17, 22, 6, 23),  // Q
    (1156, 30, 19, 16, 6, 17),  // H
    // Version 22
    (1258, 28, 2, 111, 7, 112), // L
    (1258, 28, 17, 46, 0, 0),   // M
    (1258, 30, 7, 24, 16, 25),  // Q
    (1258, 24, 34, 13, 0, 0),   // H
    // Version 23
    (1364, 30, 4, 121, 5, 122), // L
    (1364, 28, 4, 47, 14, 48),  // M
    (1364, 30, 11, 24, 14, 25), // Q
    (1364, 30, 16, 15, 14, 16), // H
    // Version 24
    (1474, 30, 6, 117, 4, 118), // L
    (1474, 28, 6, 45, 14, 46),  // M
    (1474, 30, 11, 24, 16, 25), // Q
    (1474, 30, 30, 16, 2, 17),  // H
    // Version 25
    (1588, 26, 8, 106, 4, 107), // L
    (1588, 28, 8, 47, 13, 48),  // M
    (1588, 30, 7, 24, 22, 25),  // Q
    (1588, 30, 22, 15, 13, 16), // H
    // Version 26
    (1706, 28, 10, 114, 2, 115), // L
    (1706, 28, 19, 46, 4, 47),   // M
    (1706, 28, 28, 22, 6, 23),   // Q
    (1706, 30, 33, 16, 4, 17),   // H
    // Version 27
    (1828, 30, 8, 122, 4, 123), // L
    (1828, 28, 22, 45, 3, 46),  // M
    (1828, 30, 8, 23, 26, 24),  // Q
    (1828, 30, 12, 15, 28, 16), // H
    // Version 28
    (1921, 30, 3, 117, 10, 118), // L
    (1921, 28, 3, 45, 23, 46),   // M
    (1921, 30, 4, 24, 31, 25),   // Q
    (1921, 30, 11, 15, 31, 16),  // H
    // Version 29
    (2051, 30, 7, 116, 7, 117), // L
    (2051, 28, 21, 45, 7, 46),  // M
    (2051, 30, 1, 23, 37, 24),  // Q
    (2051, 30, 19, 15, 26, 16), // H
    // Version 30
    (2185, 30, 5, 115, 10, 116), // L
    (2185, 28, 19, 47, 10, 48),  // M
    (2185, 30, 15, 24, 25, 25),  // Q
    (2185, 30, 23, 15, 25, 16),  // H
    // Version 31
    (2323, 30, 13, 115, 3, 116), // L
    (2323, 28, 2, 46, 29, 47),   // M
    (2323, 30, 42, 24, 1, 25),   // Q
    (2323, 30, 23, 15, 28, 16),  // H
    // Version 32
    (2465, 30, 17, 115, 0, 0),   // L
    (2465, 28, 10, 46, 23, 47),  // M
    (2465, 30, 10, 24, 35, 25),  // Q
    (2465, 30, 19, 15, 35, 16),  // H
    // Version 33
    (2611, 30, 17, 115, 1, 116), // L
    (2611, 28, 14, 46, 21, 47),  // M
    (2611, 30, 29, 24, 19, 25),  // Q
    (2611, 30, 11, 15, 46, 16),  // H
    // Version 34
    (2761, 30, 13, 115, 6, 116), // L
    (2761, 28, 14, 46, 23, 47),  // M
    (2761, 30, 44, 24, 7, 25),   // Q
    (2761, 30, 59, 16, 1, 17),   // H
    // Version 35
    (2876, 30, 12, 121, 7, 122), // L
    (2876, 28, 12, 47, 26, 48),  // M
    (2876, 30, 39, 24, 14, 25),  // Q
    (2876, 30, 22, 15, 41, 16),  // H
    // Version 36
    (3034, 30, 6, 121, 14, 122), // L
    (3034, 28, 6, 47, 34, 48),   // M
    (3034, 30, 46, 24, 10, 25),  // Q
    (3034, 30, 2, 15, 64, 16),   // H
    // Version 37
    (3196, 30, 17, 122, 4, 123), // L
    (3196, 28, 29, 46, 14, 47),  // M
    (3196, 30, 49, 24, 10, 25),  // Q
    (3196, 30, 24, 15, 46, 16),  // H
    // Version 38
    (3362, 30, 4, 122, 18, 123), // L
    (3362, 28, 13, 46, 32, 47),  // M
    (3362, 30, 48, 24, 14, 25),  // Q
    (3362, 30, 42, 15, 32, 16),  // H
    // Version 39
    (3532, 30, 20, 117, 4, 118), // L
    (3532, 28, 40, 47, 7, 48),   // M
    (3532, 30, 43, 24, 22, 25),  // Q
    (3532, 30, 10, 15, 67, 16),  // H
    // Version 40
    (3706, 30, 19, 118, 6, 119), // L
    (3706, 28, 18, 47, 31, 48),  // M
    (3706, 30, 34, 24, 34, 25),  // Q
    (3706, 30, 20, 15, 61, 16),  // H
];

// ── Alignment pattern positions ───────────────────────────
// Versions 2-40. Each sub-slice lists center coordinates.

#[rustfmt::skip]
const ALIGNMENT_POSITIONS: &[&[u8]] = &[
    &[6, 18],                             // V2
    &[6, 22],                             // V3
    &[6, 26],                             // V4
    &[6, 30],                             // V5
    &[6, 34],                             // V6
    &[6, 22, 38],                         // V7
    &[6, 24, 42],                         // V8
    &[6, 26, 46],                         // V9
    &[6, 28, 50],                         // V10
    &[6, 30, 54],                         // V11
    &[6, 32, 58],                         // V12
    &[6, 34, 62],                         // V13
    &[6, 26, 46, 66],                     // V14
    &[6, 26, 48, 70],                     // V15
    &[6, 26, 50, 74],                     // V16
    &[6, 30, 54, 78],                     // V17
    &[6, 30, 56, 82],                     // V18
    &[6, 30, 58, 86],                     // V19
    &[6, 34, 62, 90],                     // V20
    &[6, 28, 50, 72, 94],                 // V21
    &[6, 26, 50, 74, 98],                 // V22
    &[6, 30, 54, 78, 102],               // V23
    &[6, 28, 54, 80, 106],               // V24
    &[6, 32, 58, 84, 110],               // V25
    &[6, 30, 58, 86, 114],               // V26
    &[6, 34, 62, 90, 118],               // V27
    &[6, 26, 50, 74, 98, 122],           // V28
    &[6, 30, 54, 78, 102, 126],          // V29
    &[6, 26, 52, 78, 104, 130],          // V30
    &[6, 30, 56, 82, 108, 134],          // V31
    &[6, 34, 60, 86, 112, 138],          // V32
    &[6, 30, 58, 86, 114, 142],          // V33
    &[6, 34, 62, 90, 118, 146],          // V34
    &[6, 30, 54, 78, 102, 126, 150],     // V35
    &[6, 24, 50, 76, 102, 128, 154],     // V36
    &[6, 28, 54, 80, 106, 132, 158],     // V37
    &[6, 32, 58, 84, 110, 136, 162],     // V38
    &[6, 26, 54, 82, 110, 138, 166],     // V39
    &[6, 30, 58, 86, 114, 142, 170],     // V40
];

// ── Version info (versions 7-40) ─────────────────────────
// 18-bit version information string, BCH(18,6) encoded.
#[rustfmt::skip]
pub const VERSION_INFO: &[u32] = &[
    0x07C94, 0x085BC, 0x09A99, 0x0A4D3, 0x0BBF6, 0x0C762, 0x0D847, 0x0E60D,
    0x0F928, 0x10B78, 0x1145D, 0x12A17, 0x13532, 0x149A6, 0x15683, 0x168C9,
    0x177EC, 0x18EC4, 0x191E1, 0x1AFAB, 0x1B08E, 0x1CC1A, 0x1D33F, 0x1ED75,
    0x1F250, 0x209D5, 0x216F0, 0x228BA, 0x2379F, 0x24B0B, 0x2542E, 0x26A64,
    0x27541, 0x28C69,
];

/// Get 18-bit version info for versions 7-40.
pub fn version_info_bits(version: u8) -> Option<u32> {
    if version >= 7 && version <= 40 {
        Some(VERSION_INFO[(version - 7) as usize])
    } else {
        None
    }
}

// ── Format info ──────────────────────────────────────────
// 15-bit format information, BCH(15,5) encoded.
// Index: ec_level_bits(2) << 3 | mask_pattern(3)

#[rustfmt::skip]
pub const FORMAT_INFO: &[u16] = &[
    0x5412, 0x5125, 0x5E7C, 0x5B4B, 0x45F9, 0x40CE, 0x4F97, 0x4AA0,
    0x77C4, 0x72F3, 0x7DAA, 0x789D, 0x662F, 0x6318, 0x6C41, 0x6976,
    0x1689, 0x13BE, 0x1CE7, 0x19D0, 0x0762, 0x0255, 0x0D0C, 0x083B,
    0x355F, 0x3068, 0x3F31, 0x3A06, 0x24B4, 0x2183, 0x2EDA, 0x2BED,
];

/// Get 15-bit format info for a given EC level and mask pattern.
pub fn format_info_bits(ec: EcLevel, mask: u8) -> u16 {
    let idx = ((ec.format_bits() as usize) << 3) | (mask as usize);
    FORMAT_INFO[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_mode() {
        assert_eq!(detect_mode(b"12345"), Mode::Numeric);
        assert_eq!(detect_mode(b"HELLO"), Mode::Alphanumeric);
        assert_eq!(detect_mode(b"Hello"), Mode::Byte);
    }

    #[test]
    fn test_modules_per_side() {
        assert_eq!(modules_per_side(1), 21);
        assert_eq!(modules_per_side(2), 25);
        assert_eq!(modules_per_side(40), 177);
    }

    #[test]
    fn test_data_capacity_v1() {
        assert_eq!(data_capacity(1, EcLevel::L), 19);
        assert_eq!(data_capacity(1, EcLevel::M), 16);
        assert_eq!(data_capacity(1, EcLevel::Q), 13);
        assert_eq!(data_capacity(1, EcLevel::H), 9);
    }

    #[test]
    fn test_alphanumeric_value() {
        assert_eq!(alphanumeric_value(b'0'), Some(0));
        assert_eq!(alphanumeric_value(b'A'), Some(10));
        assert_eq!(alphanumeric_value(b' '), Some(36));
        assert_eq!(alphanumeric_value(b'a'), None);
    }
}
