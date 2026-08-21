//! QR code generation.
//!
//! Pure-Rust QR code encoder supporting versions 1-40, all four error
//! correction levels, and automatic version/mode detection.
//!
//! # Example
//! ```
//! use cryptokit::qr::{QrCode, QrError, EcLevel};
//!
//! let qr = QrCode::new(b"Hello, World!", EcLevel::M).unwrap();
//! let svg = qr.to_svg(10, "#000000", "#ffffff", 4);
//! let png = qr.to_png(10, [0, 0, 0], [255, 255, 255], 4);
//! ```

mod ec;
mod encode;
mod matrix;
mod penalty;
pub mod tables;

pub use tables::EcLevel;

use crate::CryptoError;

/// A generated QR code.
#[derive(Debug, Clone)]
pub struct QrCode {
    /// The module matrix: `true` = dark, `false` = light.
    pub modules: Vec<Vec<bool>>,
    /// QR version (1-40).
    pub version: u8,
    /// Side length in modules.
    pub size: usize,
}

/// QR code generation error.
#[derive(Debug, thiserror::Error)]
pub enum QrError {
    #[error("data too long for any QR version at EC level {0:?}")]
    DataTooLong(EcLevel),
    #[error("invalid error correction level: {0}")]
    InvalidEcLevel(String),
}

impl From<QrError> for CryptoError {
    fn from(e: QrError) -> Self {
        CryptoError::Encoding(e.to_string())
    }
}

impl QrCode {
    /// Create a new QR code from the given data and error correction level.
    ///
    /// Automatically detects the best encoding mode (numeric, alphanumeric,
    /// or byte) and selects the smallest version that fits the data.
    pub fn new(data: &[u8], ec: EcLevel) -> Result<Self, QrError> {
        let mode = tables::detect_mode(data);
        let version = encode::select_version(data, mode, ec).ok_or(QrError::DataTooLong(ec))?;

        let data_codewords = encode::encode_data(data, version, mode, ec);
        let full_codewords = interleave_with_ec(version, ec, &data_codewords);

        let size = tables::modules_per_side(version);
        let modules = matrix::build_matrix(version, ec, &full_codewords);

        Ok(Self {
            modules,
            version,
            size,
        })
    }

    /// Parse an EC level from a string ("L", "M", "Q", "H").
    pub fn parse_ec(s: &str) -> Result<EcLevel, QrError> {
        match s.to_uppercase().as_str() {
            "L" => Ok(EcLevel::L),
            "M" => Ok(EcLevel::M),
            "Q" => Ok(EcLevel::Q),
            "H" => Ok(EcLevel::H),
            _ => Err(QrError::InvalidEcLevel(s.to_string())),
        }
    }

    /// Render the QR code as an SVG string.
    pub fn to_svg(
        &self,
        module_size: u32,
        dark_color: &str,
        light_color: &str,
        quiet_zone: u32,
    ) -> String {
        super::qr_render::render_svg(
            &self.modules,
            self.size,
            module_size,
            dark_color,
            light_color,
            quiet_zone,
        )
    }

    /// Render the QR code as PNG bytes.
    pub fn to_png(
        &self,
        module_size: u32,
        dark_color: [u8; 3],
        light_color: [u8; 3],
        quiet_zone: u32,
    ) -> Vec<u8> {
        super::qr_render::render_png(
            &self.modules,
            self.size,
            module_size,
            dark_color,
            light_color,
            quiet_zone,
        )
    }
}

// ── Error correction interleaving ────────────────────────

/// Split data into blocks, compute EC for each, and interleave.
fn interleave_with_ec(version: u8, ec: EcLevel, data: &[u8]) -> Vec<u8> {
    let params = tables::ec_params(version, ec);
    let (_total_cw, ec_per_block, g1_blocks, g1_data, g2_blocks, g2_data) = params;

    let mut blocks: Vec<Vec<u8>> = Vec::new();
    let mut ec_blocks: Vec<Vec<u8>> = Vec::new();
    let mut offset = 0;

    // Group 1 blocks
    for _ in 0..g1_blocks {
        let block = data[offset..offset + g1_data].to_vec();
        let ec_cw = ec::rs_encode(&block, ec_per_block);
        blocks.push(block);
        ec_blocks.push(ec_cw);
        offset += g1_data;
    }

    // Group 2 blocks
    for _ in 0..g2_blocks {
        let block = data[offset..offset + g2_data].to_vec();
        let ec_cw = ec::rs_encode(&block, ec_per_block);
        blocks.push(block);
        ec_blocks.push(ec_cw);
        offset += g2_data;
    }

    // Interleave data codewords
    let max_data_len = blocks.iter().map(|b| b.len()).max().unwrap_or(0);
    let mut result = Vec::new();
    for i in 0..max_data_len {
        for block in &blocks {
            if i < block.len() {
                result.push(block[i]);
            }
        }
    }

    // Interleave EC codewords
    for i in 0..ec_per_block {
        for ec_block in &ec_blocks {
            if i < ec_block.len() {
                result.push(ec_block[i]);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_hello_world() {
        let qr = QrCode::new(b"HELLO WORLD", EcLevel::M).unwrap();
        assert_eq!(qr.version, 1);
        assert_eq!(qr.size, 21);
        assert_eq!(qr.modules.len(), 21);
    }

    #[test]
    fn test_qr_url() {
        let qr = QrCode::new(b"https://example.com", EcLevel::Q).unwrap();
        assert!(qr.version >= 1);
        assert!(qr.version <= 40);
    }

    #[test]
    fn test_qr_long_data() {
        let data = vec![b'A'; 200];
        let qr = QrCode::new(&data, EcLevel::H).unwrap();
        assert!(qr.version > 1);
    }

    #[test]
    fn test_qr_numeric() {
        let qr = QrCode::new(b"0123456789", EcLevel::L).unwrap();
        assert_eq!(qr.version, 1);
    }

    #[test]
    fn test_qr_too_long() {
        let data = vec![b'X'; 10000];
        let result = QrCode::new(&data, EcLevel::H);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ec() {
        assert_eq!(QrCode::parse_ec("L").unwrap(), EcLevel::L);
        assert_eq!(QrCode::parse_ec("m").unwrap(), EcLevel::M);
        assert_eq!(QrCode::parse_ec("Q").unwrap(), EcLevel::Q);
        assert_eq!(QrCode::parse_ec("h").unwrap(), EcLevel::H);
        assert!(QrCode::parse_ec("X").is_err());
    }

    #[test]
    fn test_interleave_v1() {
        let data = vec![0u8; 16]; // V1-M: 16 data codewords
        let result = interleave_with_ec(1, EcLevel::M, &data);
        assert_eq!(result.len(), 26); // 16 data + 10 EC
    }

    #[test]
    fn test_interleave_multi_block() {
        // V5-Q: 2 groups with different block sizes
        let data_cap = tables::data_capacity(5, EcLevel::Q);
        let data = vec![0u8; data_cap];
        let result = interleave_with_ec(5, EcLevel::Q, &data);
        assert_eq!(result.len(), tables::total_codewords(5));
    }
}
