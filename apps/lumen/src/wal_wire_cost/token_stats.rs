//! Borrowing generic WAL token preflight.
//!
//! This scans bytes only. It allocates no payload String, Vec, map, or value
//! tree. Its call stack is bounded by the matching decoder recursion limits.
//! `scan` chooses JSON for a leading non-whitespace `{` or `[`, then otherwise
//! follows the generic decoder's CBOR-first and JSON-fallback order. CBOR scans
//! one item and ignores trailing bytes, matching `ciborium::de::from_reader`.

use anyhow::{anyhow, bail, ensure, Result};

const CBOR_DEPTH: usize = 256;
const JSON_DEPTH: usize = 128;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TokenStats {
    pub text_bytes: usize,
    pub byte_string_bytes: usize,
    pub token_count: usize,
    pub largest_token_bytes: usize,
}

impl TokenStats {
    fn token(&mut self, bytes: usize, text: bool) -> Result<()> {
        self.token_count = self
            .token_count
            .checked_add(1)
            .ok_or_else(|| anyhow!("token count overflow"))?;
        if text {
            self.text_bytes = self
                .text_bytes
                .checked_add(bytes)
                .ok_or_else(|| anyhow!("text byte count overflow"))?;
        } else {
            self.byte_string_bytes = self
                .byte_string_bytes
                .checked_add(bytes)
                .ok_or_else(|| anyhow!("byte-string byte count overflow"))?;
        }
        self.largest_token_bytes = self.largest_token_bytes.max(bytes);
        Ok(())
    }
}

pub fn scan(bytes: &[u8]) -> Result<TokenStats> {
    if matches!(
        bytes
            .iter()
            .copied()
            .find(|byte| !byte.is_ascii_whitespace()),
        Some(b'{' | b'[')
    ) {
        return scan_json(bytes);
    }
    match scan_cbor(bytes) {
        Ok(stats) => Ok(stats),
        Err(cbor) => scan_json(bytes).map_err(|json| {
            anyhow!("generic token preflight as cbor ({cbor}) or legacy json ({json})")
        }),
    }
}

pub fn scan_cbor(bytes: &[u8]) -> Result<TokenStats> {
    let mut p = Cbor {
        bytes,
        pos: 0,
        stats: TokenStats::default(),
    };
    p.item(0)?;
    Ok(p.stats)
}

/// Scan exactly one CBOR item. Unlike [`scan_cbor`], this rejects a suffix.
/// Private durable receipts use this form because their payload length is part
/// of the integrity proof. Public generic WAL decoding keeps `scan_cbor`'s
/// historical one-item, suffix-tolerant behavior.
pub fn scan_exact_cbor(bytes: &[u8]) -> Result<TokenStats> {
    let mut p = Cbor {
        bytes,
        pos: 0,
        stats: TokenStats::default(),
    };
    p.item(0)?;
    ensure!(p.pos == bytes.len(), "trailing CBOR bytes");
    Ok(p.stats)
}

struct Cbor<'a> {
    bytes: &'a [u8],
    pos: usize,
    stats: TokenStats,
}
impl<'a> Cbor<'a> {
    fn byte(&mut self) -> Result<u8> {
        let b = *self
            .bytes
            .get(self.pos)
            .ok_or_else(|| anyhow!("truncated CBOR"))?;
        self.pos += 1;
        Ok(b)
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| anyhow!("CBOR offset overflow"))?;
        let out = self
            .bytes
            .get(self.pos..end)
            .ok_or_else(|| anyhow!("truncated CBOR"))?;
        self.pos = end;
        Ok(out)
    }
    fn arg(&mut self, ai: u8) -> Result<Option<u64>> {
        let n = match ai {
            0..=23 => return Ok(Some(ai as u64)),
            24 => self.byte()? as u64,
            25 => u16::from_be_bytes(self.take(2)?.try_into().unwrap()) as u64,
            26 => u32::from_be_bytes(self.take(4)?.try_into().unwrap()) as u64,
            27 => u64::from_be_bytes(self.take(8)?.try_into().unwrap()),
            31 => return Ok(None),
            _ => bail!("reserved CBOR additional information"),
        };
        Ok(Some(n))
    }
    fn len(n: u64) -> Result<usize> {
        usize::try_from(n).map_err(|_| anyhow!("CBOR length does not fit usize"))
    }
    fn item(&mut self, depth: usize) -> Result<()> {
        // `ciborium::de::from_reader` starts with `recurse: 256`. Permit
        // 256 container entries and their final scalar; reject the 257th.
        ensure!(depth <= CBOR_DEPTH, "CBOR recursion limit exceeded");
        let first = self.byte()?;
        let major = first >> 5;
        let arg = self.arg(first & 31)?;
        match major {
            0 | 1 => ensure!(arg.is_some(), "indefinite CBOR integer"),
            2 | 3 => self.string(major == 3, arg)?,
            4 => self.array(arg, depth)?,
            5 => self.map(arg, depth)?,
            6 => {
                ensure!(arg.is_some(), "indefinite CBOR tag");
                self.item(depth + 1)?;
            }
            7 => match arg {
                Some(_) => {}
                None => bail!("unexpected CBOR break"),
            },
            _ => unreachable!(),
        };
        Ok(())
    }
    fn string(&mut self, text: bool, arg: Option<u64>) -> Result<()> {
        let mut total = 0usize;
        match arg {
            Some(n) => {
                let n = Self::len(n)?;
                let b = self.take(n)?;
                if text {
                    std::str::from_utf8(b).map_err(|_| anyhow!("invalid CBOR UTF-8"))?;
                }
                total = n;
            }
            None => loop {
                let head = self.byte()?;
                if head == 0xff {
                    break;
                }
                ensure!(
                    head >> 5 == if text { 3 } else { 2 } && head & 31 != 31,
                    "invalid indefinite CBOR string chunk"
                );
                let n = Self::len(self.arg(head & 31)?.unwrap())?;
                let b = self.take(n)?;
                if text {
                    std::str::from_utf8(b).map_err(|_| anyhow!("invalid CBOR UTF-8"))?;
                }
                total = total
                    .checked_add(n)
                    .ok_or_else(|| anyhow!("CBOR coalesced string overflow"))?;
            },
        }
        self.stats.token(total, text)
    }
    fn array(&mut self, arg: Option<u64>, depth: usize) -> Result<()> {
        match arg {
            Some(n) => {
                let mut n = n;
                while n > 0 {
                    self.item(depth + 1)?;
                    n -= 1;
                }
            }
            None => loop {
                if self.bytes.get(self.pos) == Some(&0xff) {
                    self.pos += 1;
                    break;
                }
                self.item(depth + 1)?;
            },
        };
        Ok(())
    }
    fn map(&mut self, arg: Option<u64>, depth: usize) -> Result<()> {
        match arg {
            Some(n) => {
                let mut n = n;
                while n > 0 {
                    self.item(depth + 1)?;
                    self.item(depth + 1)?;
                    n -= 1;
                }
            }
            None => loop {
                if self.bytes.get(self.pos) == Some(&0xff) {
                    self.pos += 1;
                    break;
                }
                self.item(depth + 1)?;
                self.item(depth + 1)?;
            },
        };
        Ok(())
    }
}

pub fn scan_json(bytes: &[u8]) -> Result<TokenStats> {
    let mut p = Json {
        bytes,
        pos: 0,
        stats: TokenStats::default(),
    };
    p.ws();
    p.value(0)?;
    p.ws();
    ensure!(p.pos == bytes.len(), "trailing JSON bytes");
    Ok(p.stats)
}
struct Json<'a> {
    bytes: &'a [u8],
    pos: usize,
    stats: TokenStats,
}
impl<'a> Json<'a> {
    fn ws(&mut self) {
        while matches!(self.bytes.get(self.pos), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1
        }
    }
    fn peek(&self) -> Result<u8> {
        self.bytes
            .get(self.pos)
            .copied()
            .ok_or_else(|| anyhow!("truncated JSON"))
    }
    fn eat(&mut self, b: u8) -> Result<()> {
        ensure!(self.peek()? == b, "invalid JSON syntax");
        self.pos += 1;
        Ok(())
    }
    fn value(&mut self, depth: usize) -> Result<()> {
        ensure!(depth < JSON_DEPTH, "JSON recursion limit exceeded");
        self.ws();
        match self.peek()? {
            b'"' => self.string(),
            b'[' => self.array(depth + 1),
            b'{' => self.map(depth + 1),
            b't' => self.word(b"true"),
            b'f' => self.word(b"false"),
            b'n' => self.word(b"null"),
            b'-' | b'0'..=b'9' => self.number(),
            _ => bail!("invalid JSON value"),
        }
    }
    fn word(&mut self, w: &[u8]) -> Result<()> {
        ensure!(
            self.bytes.get(self.pos..self.pos + w.len()) == Some(w),
            "invalid JSON literal"
        );
        self.pos += w.len();
        Ok(())
    }
    fn string(&mut self) -> Result<()> {
        self.eat(b'"')?;
        let mut decoded = 0usize;
        loop {
            let b = self.peek()?;
            self.pos += 1;
            match b {
                b'"' => break,
                b'\\' => {
                    let e = self.peek()?;
                    self.pos += 1;
                    match e {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {
                            decoded = decoded
                                .checked_add(1)
                                .ok_or_else(|| anyhow!("JSON string overflow"))?
                        }
                        b'u' => {
                            let a = self.hex4()?;
                            let cp = if (0xd800..=0xdbff).contains(&a) {
                                self.eat(b'\\')?;
                                self.eat(b'u')?;
                                let b = self.hex4()?;
                                ensure!(
                                    (0xdc00..=0xdfff).contains(&b),
                                    "invalid JSON surrogate pair"
                                );
                                0x10000 + (((a - 0xd800) as usize) << 10) + (b - 0xdc00) as usize
                            } else {
                                ensure!(
                                    !(0xdc00..=0xdfff).contains(&a),
                                    "unpaired JSON low surrogate"
                                );
                                a as usize
                            };
                            decoded = decoded
                                .checked_add(char::from_u32(cp as u32).unwrap().len_utf8())
                                .ok_or_else(|| anyhow!("JSON string overflow"))?
                        }
                        _ => bail!("invalid JSON escape"),
                    }
                }
                0x00..=0x1f => bail!("JSON control in string"),
                0x80..=0xff => {
                    let start = self.pos - 1;
                    let width = match b {
                        0xc2..=0xdf => 2,
                        0xe0..=0xef => 3,
                        0xf0..=0xf4 => 4,
                        _ => bail!("invalid JSON UTF-8"),
                    };
                    let end = start
                        .checked_add(width)
                        .ok_or_else(|| anyhow!("JSON offset overflow"))?;
                    let s = std::str::from_utf8(
                        self.bytes
                            .get(start..end)
                            .ok_or_else(|| anyhow!("truncated JSON UTF-8"))?,
                    )
                    .map_err(|_| anyhow!("invalid JSON UTF-8"))?;
                    ensure!(s.chars().count() == 1, "invalid JSON UTF-8");
                    self.pos = end;
                    decoded = decoded
                        .checked_add(width)
                        .ok_or_else(|| anyhow!("JSON string overflow"))?
                }
                _ => {
                    decoded = decoded
                        .checked_add(1)
                        .ok_or_else(|| anyhow!("JSON string overflow"))?
                }
            }
        }
        self.stats.token(decoded, true)
    }
    fn hex4(&mut self) -> Result<u16> {
        let b = self
            .bytes
            .get(
                self.pos
                    ..self
                        .pos
                        .checked_add(4)
                        .ok_or_else(|| anyhow!("JSON offset overflow"))?,
            )
            .ok_or_else(|| anyhow!("truncated JSON unicode escape"))?;
        self.pos += 4;
        let mut n = 0u16;
        for x in b {
            n = n
                .checked_mul(16)
                .ok_or_else(|| anyhow!("JSON hex overflow"))?;
            n = n
                .checked_add(match x {
                    b'0'..=b'9' => (x - b'0') as u16,
                    b'a'..=b'f' => (x - b'a' + 10) as u16,
                    b'A'..=b'F' => (x - b'A' + 10) as u16,
                    _ => bail!("invalid JSON unicode escape"),
                })
                .ok_or_else(|| anyhow!("JSON hex overflow"))?;
        }
        Ok(n)
    }
    fn array(&mut self, d: usize) -> Result<()> {
        self.eat(b'[')?;
        self.ws();
        if self.bytes.get(self.pos) == Some(&b']') {
            self.pos += 1;
            return Ok(());
        }
        loop {
            self.value(d)?;
            self.ws();
            match self.peek()? {
                b']' => {
                    self.pos += 1;
                    return Ok(());
                }
                b',' => self.pos += 1,
                _ => bail!("invalid JSON array"),
            }
        }
    }
    fn map(&mut self, d: usize) -> Result<()> {
        self.eat(b'{')?;
        self.ws();
        if self.bytes.get(self.pos) == Some(&b'}') {
            self.pos += 1;
            return Ok(());
        }
        loop {
            self.string()?;
            self.ws();
            self.eat(b':')?;
            self.value(d)?;
            self.ws();
            match self.peek()? {
                b'}' => {
                    self.pos += 1;
                    return Ok(());
                }
                b',' => {
                    self.pos += 1;
                    self.ws()
                }
                _ => bail!("invalid JSON object"),
            }
        }
    }
    fn number(&mut self) -> Result<()> {
        let start = self.pos;
        if self.bytes.get(self.pos) == Some(&b'-') {
            self.pos += 1
        }
        match self.peek()? {
            b'0' => self.pos += 1,
            b'1'..=b'9' => {
                self.pos += 1;
                while matches!(self.bytes.get(self.pos), Some(b'0'..=b'9')) {
                    self.pos += 1
                }
            }
            _ => bail!("invalid JSON number"),
        };
        if self.bytes.get(self.pos) == Some(&b'.') {
            self.pos += 1;
            let before = self.pos;
            while matches!(self.bytes.get(self.pos), Some(b'0'..=b'9')) {
                self.pos += 1
            }
            ensure!(self.pos > before, "invalid JSON fraction")
        };
        if matches!(self.bytes.get(self.pos), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.bytes.get(self.pos), Some(b'+' | b'-')) {
                self.pos += 1
            }
            let before = self.pos;
            while matches!(self.bytes.get(self.pos), Some(b'0'..=b'9')) {
                self.pos += 1
            }
            ensure!(self.pos > before, "invalid JSON exponent")
        };
        ensure!(self.pos > start, "invalid JSON number");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cbor_and_json_text_stats_match_escapes_multibyte_and_indefinite_chunks() {
        let json = b" \n {\"k\":[\"A\\n\\u96ea\\uD83D\\uDE00\",\"z\"]}";
        let cbor = [
            0xa1, 0x61, b'k', 0x9f, 0x7f, 0x62, b'A', b'\n', 0x63, 0xe9, 0x9b, 0xaa, 0x64, 0xf0,
            0x9f, 0x98, 0x80, 0xff, 0x61, b'z', 0xff,
        ];
        let j = scan(json).unwrap();
        let c = scan_cbor(&cbor).unwrap();
        assert_eq!(j.text_bytes, 11);
        assert_eq!(j.token_count, 3);
        assert_eq!(c.text_bytes, 11);
        assert_eq!(c.token_count, 3);
        assert_eq!(c.largest_token_bytes, 9);
    }
    #[test]
    fn rejects_huge_truncated_and_malformed_lengths_without_allocating() {
        assert!(scan_cbor(&[0x7b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]).is_err());
        assert!(scan_cbor(&[0x7a, 0xff, 0xff, 0xff, 0xff]).is_err());
        assert!(scan_json(br#""\uD800""#).is_err());
        assert!(scan_json(br#"["#).is_err());
        let mut stats = TokenStats {
            text_bytes: usize::MAX,
            ..TokenStats::default()
        };
        assert!(
            stats.token(1, true).is_err(),
            "checked aggregate must not wrap"
        );
    }
    #[test]
    fn enforces_current_depth_bound_and_accepts_mixed_structure() {
        let mut ok = vec![b'['];
        ok.extend(std::iter::repeat_n(b'[', 126));
        ok.push(b'0');
        ok.extend(std::iter::repeat_n(b']', 127));
        assert!(scan_json(&ok).is_ok());
        let mut bad = vec![b'['];
        bad.extend(std::iter::repeat_n(b'[', 128));
        bad.push(b'0');
        bad.extend(std::iter::repeat_n(b']', 129));
        assert!(scan_json(&bad).is_err());
        assert!(scan_json(br#"{"a":[1,true,{"b":"x"}]}"#).is_ok());
    }

    #[test]
    fn cbor_depth_and_trailing_match_decoder_boundaries() {
        let mut allowed = vec![0x81; 256];
        allowed.push(0);
        assert!(scan_cbor(&allowed).is_ok(), "256 containers plus scalar");
        let mut rejected = vec![0x81; 257];
        rejected.push(0);
        assert!(scan_cbor(&rejected).is_err(), "257th container");

        let trailing = [0x61, b'x', 0x00];
        assert_eq!(
            scan_cbor(&trailing).unwrap(),
            TokenStats {
                text_bytes: 1,
                byte_string_bytes: 0,
                token_count: 1,
                largest_token_bytes: 1
            }
        );
        assert!(scan_exact_cbor(&trailing).is_err());

        let mut exact_allowed = vec![0x81; 256];
        exact_allowed.push(0);
        assert!(scan_exact_cbor(&exact_allowed).is_ok());
        let mut exact_rejected = vec![0x81; 257];
        exact_rejected.push(0);
        assert!(scan_exact_cbor(&exact_rejected).is_err());
        assert!(scan_exact_cbor(&[0x61]).is_err());
    }

    #[test]
    fn leading_json_map_or_sequence_never_uses_ambiguous_cbor_syntax() {
        let map = scan(br#" {"a":123}"#).unwrap();
        assert_eq!(map.text_bytes, 1);
        assert_eq!(map.token_count, 1);
        let sequence = scan(br#" ["snow", "\u96ea"]"#).unwrap();
        assert_eq!(sequence.text_bytes, 7);
        assert_eq!(sequence.token_count, 2);
    }
    #[test]
    fn sixty_mib_token_has_scalar_stats_without_payload_token_allocation() {
        let n = 60 * 1024 * 1024;
        let mut json = Vec::with_capacity(n + 2);
        json.push(b'"');
        json.extend(std::iter::repeat_n(b'x', n));
        json.push(b'"');
        let s = scan_json(&json).unwrap();
        assert_eq!(s.text_bytes, n);
        assert_eq!(s.largest_token_bytes, n);
        assert_eq!(s.token_count, 1);
    }
}
