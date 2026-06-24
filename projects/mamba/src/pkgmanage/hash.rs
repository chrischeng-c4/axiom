// `mamba hash` — content-addressed digest primitive.
//
// Acceptance (tests/governance/gates/pkgmgr/hash/manifest.toml, schema gate
// pkgmgr_hash_verification_fixture_2686.rs):
//
//   - Algorithm defaults to sha256; --algorithm sha384|sha512 supported.
//   - Output shape: "<algo>:<hex>  <path>" — `pip hash`-compatible so
//     downstream verifiers can grep deterministically.
//   - Exit 1 with an actionable error when the file is missing /
//     unreadable.
//   - Offline; never touches global cache or network.
//
// Note: full hash-verified install (the runner-side of #2686) lives
// in `sync` once the frozen index ships real wheels with sha256
// envelopes; the primitive here is the byte-deterministic digest
// shape it will consume.

use anyhow::{bail, Context, Result};
use clap::ArgMatches;
use sha2::{Digest, Sha256, Sha384, Sha512};
use std::fs::File;
use std::io::{BufReader, Read, Write as _};
use std::path::PathBuf;

const BUF_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgo {
    Sha256,
    Sha384,
    Sha512,
}

impl HashAlgo {
    pub fn parse(raw: &str) -> Result<Self> {
        match raw.to_ascii_lowercase().as_str() {
            "sha256" => Ok(HashAlgo::Sha256),
            "sha384" => Ok(HashAlgo::Sha384),
            "sha512" => Ok(HashAlgo::Sha512),
            other => bail!("unknown hash algorithm `{other}` (sha256|sha384|sha512)"),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            HashAlgo::Sha256 => "sha256",
            HashAlgo::Sha384 => "sha384",
            HashAlgo::Sha512 => "sha512",
        }
    }

    pub fn hex_length(self) -> usize {
        match self {
            HashAlgo::Sha256 => 64,
            HashAlgo::Sha384 => 96,
            HashAlgo::Sha512 => 128,
        }
    }
}

pub fn cmd_hash(sub: &ArgMatches) -> Result<()> {
    let raw_paths = sub
        .get_many::<String>("path")
        .context("missing required <path>")?;
    let algo = match sub.get_one::<String>("algorithm") {
        Some(a) => HashAlgo::parse(a)?,
        None => HashAlgo::Sha256,
    };
    let mut stdout = std::io::stdout().lock();
    for raw in raw_paths {
        let path = PathBuf::from(raw);
        let digest = hash_file(&path, algo).with_context(|| format!("hash {}", path.display()))?;
        writeln!(
            stdout,
            "{algo}:{digest}  {path}",
            algo = algo.label(),
            path = path.display()
        )
        .context("write hash line")?;
    }
    Ok(())
}

pub fn hash_file(path: &std::path::Path, algo: HashAlgo) -> Result<String> {
    let f = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut reader = BufReader::with_capacity(BUF_BYTES, f);
    match algo {
        HashAlgo::Sha256 => stream_hash(&mut reader, Sha256::new()),
        HashAlgo::Sha384 => stream_hash(&mut reader, Sha384::new()),
        HashAlgo::Sha512 => stream_hash(&mut reader, Sha512::new()),
    }
}

fn stream_hash<R: Read, D: Digest>(reader: &mut R, mut hasher: D) -> Result<String> {
    let mut buf = [0u8; BUF_BYTES];
    loop {
        let n = reader.read(&mut buf).context("read")?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let bytes = hasher.finalize();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        hex.push_str(&format!("{b:02x}"));
    }
    Ok(hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algo_parse_known() {
        assert_eq!(HashAlgo::parse("sha256").unwrap(), HashAlgo::Sha256);
        assert_eq!(HashAlgo::parse("SHA384").unwrap(), HashAlgo::Sha384);
        assert_eq!(HashAlgo::parse("sha512").unwrap(), HashAlgo::Sha512);
    }

    #[test]
    fn algo_parse_unknown_errors() {
        assert!(HashAlgo::parse("md5").is_err());
        assert!(HashAlgo::parse("").is_err());
    }

    #[test]
    fn algo_hex_length_is_pinned() {
        assert_eq!(HashAlgo::Sha256.hex_length(), 64);
        assert_eq!(HashAlgo::Sha384.hex_length(), 96);
        assert_eq!(HashAlgo::Sha512.hex_length(), 128);
    }

    #[test]
    fn empty_file_sha256_is_well_known() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let h = hash_file(tmp.path(), HashAlgo::Sha256).unwrap();
        assert_eq!(
            h,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn small_blob_sha256_matches_pinned() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"hello\n").unwrap();
        let h = hash_file(tmp.path(), HashAlgo::Sha256).unwrap();
        // Known: sha256("hello\n")
        assert_eq!(
            h,
            "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"
        );
    }
}
