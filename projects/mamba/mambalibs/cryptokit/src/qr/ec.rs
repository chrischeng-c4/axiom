//! Reed-Solomon error correction for QR codes.
//!
//! Operates in GF(2^8) with primitive polynomial x^8 + x^4 + x^3 + x^2 + 1
//! (0x11D, same as QR code spec).

// ── GF(2^8) tables ───────────────────────────────────────

/// GF(2^8) exponent table: EXP[i] = alpha^i
const fn build_exp_table() -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut val: u16 = 1;
    let mut i = 0;
    while i < 256 {
        table[i] = val as u8;
        val <<= 1;
        if val >= 256 {
            val ^= 0x11D; // primitive polynomial
        }
        i += 1;
    }
    table
}

/// GF(2^8) logarithm table: LOG[alpha^i] = i
const fn build_log_table(exp: &[u8; 256]) -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 255 {
        table[exp[i] as usize] = i as u8;
        i += 1;
    }
    table
}

const GF_EXP: [u8; 256] = build_exp_table();
const GF_LOG: [u8; 256] = build_log_table(&GF_EXP);

/// Multiply two values in GF(2^8).
fn gf_mul(a: u8, b: u8) -> u8 {
    if a == 0 || b == 0 {
        return 0;
    }
    let log_a = GF_LOG[a as usize] as u16;
    let log_b = GF_LOG[b as usize] as u16;
    GF_EXP[((log_a + log_b) % 255) as usize]
}

// ── Generator polynomial ─────────────────────────────────

/// Build the generator polynomial for `n` error correction codewords.
/// Returns coefficients in descending power order: g[0]*x^n + g[1]*x^(n-1) + ... + g[n]
fn generator_poly(n: usize) -> Vec<u8> {
    let mut poly = vec![0u8; n + 1];
    poly[0] = 1; // Start with x^0 = 1 (coeff stored at index 0 as leading term)

    // We actually build it as: gen = [1], then multiply by (x - alpha^i) for i in 0..n
    // Using a flat coefficient array where gen[i] is the coefficient of x^(n-i)
    let mut gen = Vec::with_capacity(n + 1);
    gen.push(1u8);

    for i in 0..n {
        let alpha_i = GF_EXP[i % 255];
        let mut next = vec![0u8; gen.len() + 1];
        // Multiply gen by (x - alpha^i) = (x + alpha^i) in GF(2^8)
        for (j, &coeff) in gen.iter().enumerate() {
            next[j] ^= coeff; // coeff * x
            next[j + 1] ^= gf_mul(coeff, alpha_i); // coeff * alpha^i
        }
        gen = next;
    }

    gen
}

// ── Reed-Solomon encoding ────────────────────────────────

/// Compute Reed-Solomon error correction codewords for the given data.
///
/// `data` is the message polynomial coefficients (data codewords).
/// `ec_count` is the number of error correction codewords to generate.
///
/// Returns `ec_count` error correction codewords.
pub fn rs_encode(data: &[u8], ec_count: usize) -> Vec<u8> {
    let gen = generator_poly(ec_count);

    // Polynomial division: data * x^ec_count / gen
    let mut remainder = vec![0u8; ec_count];

    for &byte in data {
        let lead = byte ^ remainder[0];
        // Shift remainder left by 1
        for j in 0..ec_count - 1 {
            remainder[j] = remainder[j + 1];
        }
        remainder[ec_count - 1] = 0;

        // Subtract (XOR) lead * gen from remainder
        if lead != 0 {
            for j in 0..ec_count {
                remainder[j] ^= gf_mul(lead, gen[j + 1]);
            }
        }
    }

    remainder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gf_mul() {
        // alpha^0 * alpha^0 = alpha^0 = 1
        assert_eq!(gf_mul(1, 1), 1);
        // 0 * anything = 0
        assert_eq!(gf_mul(0, 42), 0);
        // alpha^1 * alpha^1 = alpha^2 = 4
        assert_eq!(gf_mul(2, 2), 4);
    }

    #[test]
    fn test_generator_poly_length() {
        let gen = generator_poly(7);
        assert_eq!(gen.len(), 8); // degree 7 polynomial has 8 coefficients
        assert_eq!(gen[0], 1); // leading coefficient is always 1
    }

    #[test]
    fn test_rs_encode_v1_m() {
        // Version 1-M: 16 data codewords, 10 EC codewords
        // Known test vector: "HELLO WORLD" in alphanumeric mode
        // Data codewords for "01100001011" ... (simplified test)
        let data = vec![32, 91, 11, 120, 209, 114, 220, 77, 67, 64, 236, 17, 236, 17, 236, 17];
        let ec = rs_encode(&data, 10);
        assert_eq!(ec.len(), 10);
        // Verify it's deterministic
        let ec2 = rs_encode(&data, 10);
        assert_eq!(ec, ec2);
    }

    #[test]
    fn test_rs_encode_simple() {
        // Simple test: single byte data, 2 EC codewords
        let data = vec![0x40]; // Just the byte 0x40
        let ec = rs_encode(&data, 2);
        assert_eq!(ec.len(), 2);
    }

    #[test]
    fn test_gf_tables_consistent() {
        // Verify EXP and LOG are inverse operations
        for i in 0..255u8 {
            let exp_val = GF_EXP[i as usize];
            if exp_val != 0 {
                assert_eq!(GF_LOG[exp_val as usize], i);
            }
        }
    }
}
