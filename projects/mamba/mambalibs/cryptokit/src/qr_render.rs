//! QR code rendering: SVG and PNG output.
//!
//! Pure-Rust implementations with no external image dependencies.
//! PNG is built from raw bytes using minimal IHDR/IDAT/IEND chunks.

use std::io::Write;

// ── SVG rendering ────────────────────────────────────────

/// Render a QR module matrix as an SVG string.
///
/// - `module_size`: pixel size of each module
/// - `dark_color` / `light_color`: CSS color strings (e.g. "#000000")
/// - `quiet_zone`: number of modules of quiet zone around the QR code
pub fn render_svg(
    modules: &[Vec<bool>],
    size: usize,
    module_size: u32,
    dark_color: &str,
    light_color: &str,
    quiet_zone: u32,
) -> String {
    let total = (size as u32 + quiet_zone * 2) * module_size;
    let offset = quiet_zone * module_size;

    let mut svg = String::with_capacity(size * size * 40);
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {total} {total}" width="{total}" height="{total}">"#,
    ));

    // Background
    svg.push_str(&format!(
        r#"<rect width="{total}" height="{total}" fill="{light_color}"/>"#,
    ));

    // Dark modules — merge horizontally for smaller SVG
    for (r, row) in modules.iter().enumerate() {
        let mut c = 0;
        while c < size {
            if row[c] {
                let start = c;
                while c < size && row[c] {
                    c += 1;
                }
                let x = offset + start as u32 * module_size;
                let y = offset + r as u32 * module_size;
                let w = (c - start) as u32 * module_size;
                svg.push_str(&format!(
                    r#"<rect x="{x}" y="{y}" width="{w}" height="{module_size}" fill="{dark_color}"/>"#,
                ));
            } else {
                c += 1;
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

// ── PNG rendering ────────────────────────────────────────

/// Render a QR module matrix as PNG bytes.
///
/// Builds a minimal valid PNG with:
/// - IHDR (image header)
/// - IDAT (image data with zlib/deflate)
/// - IEND (image end)
pub fn render_png(
    modules: &[Vec<bool>],
    size: usize,
    module_size: u32,
    dark_color: [u8; 3],
    light_color: [u8; 3],
    quiet_zone: u32,
) -> Vec<u8> {
    let total = (size as u32 + quiet_zone * 2) * module_size;
    let width = total as usize;
    let height = total as usize;
    let qz = (quiet_zone * module_size) as usize;
    let ms = module_size as usize;

    // Build raw image rows (filter byte + RGB pixels)
    let row_bytes = 1 + width * 3; // filter_type + RGB per pixel
    let mut raw_data = Vec::with_capacity(row_bytes * height);

    for py in 0..height {
        raw_data.push(0); // filter type: None
        for px in 0..width {
            let color = if px >= qz && py >= qz {
                let mx = (px - qz) / ms;
                let my = (py - qz) / ms;
                if mx < size && my < size && modules[my][mx] {
                    dark_color
                } else {
                    light_color
                }
            } else {
                light_color
            };
            raw_data.extend_from_slice(&color);
        }
    }

    // Compress with deflate (stored blocks, no compression for simplicity)
    let compressed = deflate_store(&raw_data);

    // Build PNG
    let mut png = Vec::with_capacity(compressed.len() + 100);

    // PNG signature
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    // IHDR chunk
    write_chunk(&mut png, b"IHDR", &build_ihdr(width as u32, height as u32));

    // IDAT chunk
    write_chunk(&mut png, b"IDAT", &compressed);

    // IEND chunk
    write_chunk(&mut png, b"IEND", &[]);

    png
}

/// Build IHDR data: width, height, bit depth, color type, etc.
fn build_ihdr(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity(13);
    data.extend_from_slice(&width.to_be_bytes());
    data.extend_from_slice(&height.to_be_bytes());
    data.push(8); // bit depth
    data.push(2); // color type: RGB
    data.push(0); // compression method
    data.push(0); // filter method
    data.push(0); // interlace method
    data
}

/// Write a PNG chunk: length + type + data + CRC32.
fn write_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);

    // CRC32 over type + data
    let crc = crc32(chunk_type, data);
    out.extend_from_slice(&crc.to_be_bytes());
}

// ── CRC32 (PNG uses CRC-32/ISO-HDLC) ───────────────────

/// Compute CRC32 for PNG chunks (over chunk type + data).
fn crc32(chunk_type: &[u8], data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in chunk_type.iter().chain(data.iter()) {
        let idx = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = CRC32_TABLE[idx] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

/// Pre-computed CRC32 lookup table.
const fn build_crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = 0xEDB88320 ^ (crc >> 1);
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

const CRC32_TABLE: [u32; 256] = build_crc32_table();

// ── Deflate (stored blocks) ─────────────────────────────

/// Wrap raw data in zlib format with stored (uncompressed) deflate blocks.
/// This produces valid but uncompressed output. For QR codes the data is
/// small enough that compression is not critical.
fn deflate_store(data: &[u8]) -> Vec<u8> {
    // zlib header: CM=8 (deflate), CINFO=7 (32K window), FCHECK makes it valid
    let cmf: u8 = 0x78; // CM=8, CINFO=7
    let flg: u8 = 0x01; // FCHECK=1, no dict, FLEVEL=0
    // Adjust FCHECK so (CMF*256+FLG) % 31 == 0
    let check = (cmf as u16 * 256 + flg as u16) % 31;
    let flg = if check == 0 { flg } else { flg + (31 - check) as u8 };

    let mut out = Vec::with_capacity(data.len() + data.len() / 0xFFFF * 5 + 20);
    out.push(cmf);
    out.push(flg);

    // Deflate stored blocks (max 65535 bytes each)
    let chunks: Vec<&[u8]> = data.chunks(0xFFFF).collect();
    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        out.push(if is_last { 0x01 } else { 0x00 }); // BFINAL + BTYPE=00 (stored)
        let len = chunk.len() as u16;
        let nlen = !len;
        out.write_all(&len.to_le_bytes()).unwrap();
        out.write_all(&nlen.to_le_bytes()).unwrap();
        out.write_all(chunk).unwrap();
    }

    // Adler-32 checksum
    let adler = adler32(data);
    out.extend_from_slice(&adler.to_be_bytes());

    out
}

/// Compute Adler-32 checksum.
fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_svg() {
        let modules = vec![
            vec![true, false, true],
            vec![false, true, false],
            vec![true, false, true],
        ];
        let svg = render_svg(&modules, 3, 10, "#000", "#fff", 1);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("rect"));
    }

    #[test]
    fn test_render_png_signature() {
        let modules = vec![
            vec![true, false],
            vec![false, true],
        ];
        let png = render_png(&modules, 2, 4, [0, 0, 0], [255, 255, 255], 1);
        // PNG signature
        assert_eq!(&png[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }

    #[test]
    fn test_render_png_ihdr() {
        let modules = vec![vec![true; 3]; 3];
        let png = render_png(&modules, 3, 2, [0, 0, 0], [255, 255, 255], 1);
        // After signature (8 bytes), first chunk should be IHDR
        assert_eq!(&png[12..16], b"IHDR");
    }

    #[test]
    fn test_crc32_known() {
        // CRC32 of "IEND" with no data
        let crc = crc32(b"IEND", &[]);
        assert_eq!(crc, 0xAE426082);
    }

    #[test]
    fn test_adler32_known() {
        // Adler32 of "Wikipedia"
        let a = adler32(b"Wikipedia");
        assert_eq!(a, 0x11E60398);
    }

    #[test]
    fn test_svg_quiet_zone() {
        let modules = vec![vec![true]];
        let svg = render_svg(&modules, 1, 10, "#000", "#fff", 4);
        // Total size should be (1 + 4*2) * 10 = 90
        assert!(svg.contains("90"));
    }
}
