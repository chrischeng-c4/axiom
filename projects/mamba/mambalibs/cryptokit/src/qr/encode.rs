//! QR data encoding: numeric, alphanumeric, and byte modes.
//!
//! Converts input data into a bit stream with mode indicators, character count,
//! and padding as required by ISO 18004.

use super::tables::{self, EcLevel, Mode};

/// A bit stream builder for QR data encoding.
pub struct BitStream {
    bits: Vec<u8>, // each element is 0 or 1
}

impl BitStream {
    pub fn new() -> Self {
        Self { bits: Vec::with_capacity(256) }
    }

    /// Append `count` bits from `value` (MSB first).
    pub fn append(&mut self, value: u32, count: usize) {
        for i in (0..count).rev() {
            self.bits.push(((value >> i) & 1) as u8);
        }
    }

    /// Current length in bits.
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// Pack bits into bytes, padding with zeros on the right.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity((self.bits.len() + 7) / 8);
        for chunk in self.bits.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                byte |= bit << (7 - i);
            }
            bytes.push(byte);
        }
        bytes
    }
}

// ── Mode-specific encoding ───────────────────────────────

/// Encode numeric data into the bit stream.
fn encode_numeric(bs: &mut BitStream, data: &[u8]) {
    // Groups of 3 digits → 10 bits, 2 digits → 7 bits, 1 digit → 4 bits
    let mut i = 0;
    while i + 2 < data.len() {
        let val = (data[i] - b'0') as u32 * 100
            + (data[i + 1] - b'0') as u32 * 10
            + (data[i + 2] - b'0') as u32;
        bs.append(val, 10);
        i += 3;
    }
    let remaining = data.len() - i;
    if remaining == 2 {
        let val = (data[i] - b'0') as u32 * 10 + (data[i + 1] - b'0') as u32;
        bs.append(val, 7);
    } else if remaining == 1 {
        let val = (data[i] - b'0') as u32;
        bs.append(val, 4);
    }
}

/// Encode alphanumeric data into the bit stream.
fn encode_alphanumeric(bs: &mut BitStream, data: &[u8]) {
    // Pairs of characters → 11 bits, single remaining → 6 bits
    let mut i = 0;
    while i + 1 < data.len() {
        let v1 = tables::alphanumeric_value(data[i]).unwrap() as u32;
        let v2 = tables::alphanumeric_value(data[i + 1]).unwrap() as u32;
        bs.append(v1 * 45 + v2, 11);
        i += 2;
    }
    if i < data.len() {
        let v = tables::alphanumeric_value(data[i]).unwrap() as u32;
        bs.append(v, 6);
    }
}

/// Encode byte data into the bit stream.
fn encode_byte(bs: &mut BitStream, data: &[u8]) {
    for &b in data {
        bs.append(b as u32, 8);
    }
}

// ── Version selection ────────────────────────────────────

/// Select the minimum QR version that can hold the data at the given EC level.
/// Returns None if data is too large for any version.
pub fn select_version(data: &[u8], mode: Mode, ec: EcLevel) -> Option<u8> {
    for v in 1..=40u8 {
        let capacity_bits = tables::data_capacity(v, ec) * 8;
        let header_bits = 4 + mode.cci_bits(v); // mode indicator + character count
        let data_bits = data_bits_for_mode(mode, data.len());
        let total = header_bits + data_bits;
        if total <= capacity_bits {
            return Some(v);
        }
    }
    None
}

/// Calculate the number of data bits needed for a given mode and data length.
fn data_bits_for_mode(mode: Mode, len: usize) -> usize {
    match mode {
        Mode::Numeric => {
            let full_groups = len / 3;
            let remainder = len % 3;
            let rem_bits = match remainder {
                2 => 7,
                1 => 4,
                _ => 0,
            };
            full_groups * 10 + rem_bits
        }
        Mode::Alphanumeric => {
            let pairs = len / 2;
            let odd = len % 2;
            pairs * 11 + odd * 6
        }
        Mode::Byte => len * 8,
    }
}

// ── Full data encoding ──────────────────────────────────

/// Encode the data into codewords for the given version and EC level.
///
/// Returns the data codewords (without EC codewords).
pub fn encode_data(data: &[u8], version: u8, mode: Mode, ec: EcLevel) -> Vec<u8> {
    let capacity_bytes = tables::data_capacity(version, ec);
    let capacity_bits = capacity_bytes * 8;

    let mut bs = BitStream::new();

    // Mode indicator (4 bits)
    bs.append(mode.indicator() as u32, 4);

    // Character count indicator
    let cci_len = mode.cci_bits(version);
    bs.append(data.len() as u32, cci_len);

    // Data encoding
    match mode {
        Mode::Numeric => encode_numeric(&mut bs, data),
        Mode::Alphanumeric => encode_alphanumeric(&mut bs, data),
        Mode::Byte => encode_byte(&mut bs, data),
    }

    // Terminator: up to 4 zero bits
    let remaining = capacity_bits.saturating_sub(bs.len());
    let terminator_len = remaining.min(4);
    if terminator_len > 0 {
        bs.append(0, terminator_len);
    }

    // Pad to byte boundary
    let pad_to_byte = (8 - (bs.len() % 8)) % 8;
    if pad_to_byte > 0 {
        bs.append(0, pad_to_byte);
    }

    let mut codewords = bs.to_bytes();

    // Pad with alternating 0xEC 0x11 to fill capacity
    let pad_bytes = [0xEC, 0x11];
    let mut pad_idx = 0;
    while codewords.len() < capacity_bytes {
        codewords.push(pad_bytes[pad_idx % 2]);
        pad_idx += 1;
    }

    codewords.truncate(capacity_bytes);
    codewords
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitstream_append() {
        let mut bs = BitStream::new();
        bs.append(0b1010, 4);
        assert_eq!(bs.len(), 4);
        let bytes = bs.to_bytes();
        assert_eq!(bytes[0], 0b10100000);
    }

    #[test]
    fn test_bitstream_to_bytes() {
        let mut bs = BitStream::new();
        bs.append(0xFF, 8);
        bs.append(0x0F, 4);
        let bytes = bs.to_bytes();
        assert_eq!(bytes, vec![0xFF, 0xF0]);
    }

    #[test]
    fn test_select_version_hello() {
        let v = select_version(b"HELLO WORLD", Mode::Alphanumeric, EcLevel::M);
        assert_eq!(v, Some(1)); // Should fit in version 1-M
    }

    #[test]
    fn test_select_version_long() {
        // V40-L byte capacity is 2953 characters
        let data = vec![b'A'; 2900];
        let v = select_version(&data, Mode::Byte, EcLevel::L);
        assert!(v.is_some()); // Should fit within version 40
    }

    #[test]
    fn test_select_version_too_long() {
        let data = vec![b'X'; 5000];
        let v = select_version(&data, Mode::Byte, EcLevel::L);
        assert!(v.is_none()); // Exceeds max QR capacity
    }

    #[test]
    fn test_encode_data_length() {
        let data = b"HELLO WORLD";
        let cw = encode_data(data, 1, Mode::Alphanumeric, EcLevel::M);
        assert_eq!(cw.len(), 16); // V1-M has 16 data codewords
    }

    #[test]
    fn test_data_bits_numeric() {
        assert_eq!(data_bits_for_mode(Mode::Numeric, 3), 10);
        assert_eq!(data_bits_for_mode(Mode::Numeric, 5), 17); // 10 + 7
        assert_eq!(data_bits_for_mode(Mode::Numeric, 1), 4);
    }

    #[test]
    fn test_data_bits_alphanumeric() {
        assert_eq!(data_bits_for_mode(Mode::Alphanumeric, 2), 11);
        assert_eq!(data_bits_for_mode(Mode::Alphanumeric, 3), 17); // 11 + 6
    }

    #[test]
    fn test_encode_data_padding() {
        let data = b"1";
        let cw = encode_data(data, 1, Mode::Numeric, EcLevel::L);
        assert_eq!(cw.len(), 19); // V1-L has 19 data codewords
        // Check padding pattern at the end
        // After data + terminator + byte padding, rest should alternate 0xEC 0x11
    }
}
