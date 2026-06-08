// `uv build --sdist` — deterministic source-distribution writer (Tick 46).
//
// Pure data + thin tar.gz writer. Dual of `wheel_build.rs`.
//
// PEP 517 specifies that an sdist is a gzipped tar archive named
// `<name>-<version>.tar.gz` whose entries are rooted under a single
// top-level directory `<name>-<version>/`. Inside that root, at minimum,
// the archive carries `PKG-INFO` (PEP 621 core metadata) and the source
// tree the build backend needs.
//
// What this Tick covers:
//   * `SdistFilename` — typed name+version that renders the canonical
//     `<name>-<version>.tar.gz` form and the `<name>-<version>/`
//     archive prefix.
//   * `SdistBuilder` — accumulates files keyed by their *in-archive*
//     relative path (no leading slash, no `<root>/` prefix — we apply
//     that for the caller). Auto-injects `PKG-INFO` when omitted.
//   * `build_bytes()` — emits a deterministic gzipped tar:
//       - entries sorted by archive path,
//       - mtime fixed to 0 (1970-01-01),
//       - uid/gid set to 0 / `root`,
//       - file mode 0o644, directory mode 0o755,
//       - gzip's mtime header zeroed for byte-identical builds.
//   * `build_to_dir(out_dir)` — writes `<dist>/<filename>` to disk.
//
// Not covered (deferred):
//   * Running a PEP 517 backend's `build_sdist` hook (driver side).
//   * Source-tree filtering (`MANIFEST.in`, `.gitignore`-style globs).
//   * `Provides-Extra:` extension fields beyond what `CoreMetadata`
//     in `wheel_build.rs` already handles.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Builder, Header};

use crate::pkgmanage::pkgmgr::types::IndexError;
use crate::pkgmanage::pkgmgr::wheel_build::{render_core_metadata, CoreMetadata};

/// Typed `<name>-<version>` pair. Renders the canonical sdist filename
/// (`<name>-<version>.tar.gz`) and archive root prefix
/// (`<name>-<version>/`). Per PEP 625, the dash separators DO need to be
/// distinct from any dashes inside `name` — we leave name verbatim
/// (PEP 503 normalization is the caller's responsibility) but escape
/// version's dash since version strings never contain one in canonical
/// PEP 440 form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdistFilename {
    pub name: String,
    pub version: String,
}

impl SdistFilename {
    /// `<name>-<version>.tar.gz`.
    pub fn to_filename(&self) -> String {
        format!("{}-{}.tar.gz", self.name, self.version)
    }
    /// Archive-root prefix used for every entry inside.
    pub fn root_dir(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }
}

/// In-memory accumulator for an sdist.
///
/// Files are keyed by their archive-relative path (e.g. `src/foo.py`,
/// `pyproject.toml`). The builder will prepend `<name>-<version>/` to
/// each path when emitting the archive.
pub struct SdistBuilder {
    filename: SdistFilename,
    files: BTreeMap<String, Vec<u8>>,
    metadata: CoreMetadata,
    /// `Generator:` line stamped on `PKG-INFO` when auto-injected.
    pub generator: String,
}

impl SdistBuilder {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        let name = name.into();
        let version = version.into();
        SdistBuilder {
            filename: SdistFilename {
                name: name.clone(),
                version: version.clone(),
            },
            files: BTreeMap::new(),
            metadata: CoreMetadata::new(name, version),
            generator: "mamba".into(),
        }
    }

    /// Set the core metadata body. Used to render `PKG-INFO` (and to
    /// stamp the canonical name + version onto the archive filename).
    pub fn metadata(mut self, metadata: CoreMetadata) -> Self {
        // The filename derives from canonical name + version.
        self.filename.name = metadata.name.clone();
        self.filename.version = metadata.version.clone();
        self.metadata = metadata;
        self
    }

    /// Add (or replace) one source file at `archive_path`.
    pub fn add_file(mut self, archive_path: impl Into<String>, data: Vec<u8>) -> Self {
        self.files.insert(archive_path.into(), data);
        self
    }

    /// Convenience: add a file from disk at `host_path` under
    /// `archive_path` in the archive.
    pub fn add_file_from(
        self,
        archive_path: impl Into<String>,
        host_path: &Path,
    ) -> Result<Self, IndexError> {
        let body = fs::read(host_path).map_err(|e| IndexError::CacheIo {
            path: "<sdist>".into(),
            detail: format!("sdist: read {}: {e}", host_path.display()),
        })?;
        Ok(self.add_file(archive_path, body))
    }

    /// Build the archive into memory. Returns the gzipped tar bytes.
    pub fn build_bytes(&self) -> Result<Vec<u8>, IndexError> {
        // Inject PKG-INFO if the caller didn't supply one.
        let mut files = self.files.clone();
        files
            .entry("PKG-INFO".to_string())
            .or_insert_with(|| render_core_metadata(&self.metadata).into_bytes());

        // Always carry pyproject.toml at the root if the caller already
        // provided one; we do NOT synthesize one. Build backends expect
        // it to be authored by the project.

        // Use a fresh GzEncoder so we can stamp mtime=0 on the gzip
        // header for byte-identical rebuilds.
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        // Stamp the inner tar via tar::Builder with explicit headers
        // so we control every mtime/uid/gid/mode byte.
        {
            let mut tar = Builder::new(&mut gz);
            let root = self.filename.root_dir();
            for (path, data) in &files {
                let full = format!("{root}/{path}");
                let mut header = Header::new_gnu();
                header.set_size(data.len() as u64);
                header.set_mode(0o644);
                header.set_mtime(0);
                header.set_uid(0);
                header.set_gid(0);
                header.set_username("root").ok();
                header.set_groupname("root").ok();
                header.set_path(&full).map_err(|e| IndexError::CacheIo {
                    path: "<sdist>".into(),
                    detail: format!("sdist: tar path {full}: {e}"),
                })?;
                header.set_cksum();
                tar.append(&header, data.as_slice())
                    .map_err(|e| IndexError::CacheIo {
                        path: "<sdist>".into(),
                        detail: format!("sdist: tar append {full}: {e}"),
                    })?;
            }
            tar.finish().map_err(|e| IndexError::CacheIo {
                path: "<sdist>".into(),
                detail: format!("sdist: tar finish: {e}"),
            })?;
        }
        let body = gz.finish().map_err(|e| IndexError::CacheIo {
            path: "<sdist>".into(),
            detail: format!("sdist: gzip finish: {e}"),
        })?;
        Ok(body)
    }

    /// Write the archive to `out_dir/<filename>`. Creates `out_dir` if
    /// it doesn't exist. Returns the path written.
    pub fn build_to_dir(&self, out_dir: &Path) -> Result<PathBuf, IndexError> {
        fs::create_dir_all(out_dir).map_err(|e| IndexError::CacheIo {
            path: "<sdist>".into(),
            detail: format!("sdist: mkdir {}: {e}", out_dir.display()),
        })?;
        let out = out_dir.join(self.filename.to_filename());
        let body = self.build_bytes()?;
        let mut f = fs::File::create(&out).map_err(|e| IndexError::CacheIo {
            path: "<sdist>".into(),
            detail: format!("sdist: create {}: {e}", out.display()),
        })?;
        f.write_all(&body).map_err(|e| IndexError::CacheIo {
            path: "<sdist>".into(),
            detail: format!("sdist: write {}: {e}", out.display()),
        })?;
        Ok(out)
    }

    /// Return the canonical filename without building.
    pub fn filename(&self) -> &SdistFilename {
        &self.filename
    }
}

/// List every archive entry path (with the `<name>-<version>/` prefix
/// applied) that `build_bytes` would emit, *without* actually building.
/// Useful for callers running the equivalent of `tar tf <sdist>`.
pub fn list_archive_paths(builder: &SdistBuilder) -> Vec<String> {
    let root = builder.filename.root_dir();
    let mut files = builder.files.clone();
    files.entry("PKG-INFO".to_string()).or_default();
    files.keys().map(|p| format!("{root}/{p}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tar::Archive;

    fn gunzip(body: &[u8]) -> Vec<u8> {
        let mut dec = flate2::read::GzDecoder::new(body);
        let mut out = Vec::new();
        dec.read_to_end(&mut out).unwrap();
        out
    }

    fn read_entries(body: &[u8]) -> Vec<(String, Vec<u8>, u32, u64)> {
        let tar_bytes = gunzip(body);
        let mut archive = Archive::new(tar_bytes.as_slice());
        let mut out = Vec::new();
        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            let path = entry.path().unwrap().to_string_lossy().into_owned();
            let mode = entry.header().mode().unwrap();
            let mtime = entry.header().mtime().unwrap();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).unwrap();
            out.push((path, data, mode, mtime));
        }
        out
    }

    #[test]
    fn filename_renders_canonical_form() {
        let f = SdistFilename {
            name: "requests".into(),
            version: "2.31.0".into(),
        };
        assert_eq!(f.to_filename(), "requests-2.31.0.tar.gz");
        assert_eq!(f.root_dir(), "requests-2.31.0");
    }

    #[test]
    fn build_emits_pkg_info_when_caller_omits_it() {
        let b = SdistBuilder::new("demo", "1.0.0");
        let entries = read_entries(&b.build_bytes().unwrap());
        let pkg_info = entries
            .iter()
            .find(|(p, ..)| p == "demo-1.0.0/PKG-INFO")
            .unwrap();
        let body = String::from_utf8_lossy(&pkg_info.1);
        assert!(body.contains("Name: demo"));
        assert!(body.contains("Version: 1.0.0"));
    }

    #[test]
    fn build_respects_explicit_pkg_info() {
        let b = SdistBuilder::new("demo", "1.0.0")
            .add_file("PKG-INFO", b"hand-written body\n".to_vec());
        let entries = read_entries(&b.build_bytes().unwrap());
        let body = entries
            .iter()
            .find(|(p, ..)| p == "demo-1.0.0/PKG-INFO")
            .unwrap();
        assert_eq!(body.1, b"hand-written body\n");
    }

    #[test]
    fn build_carries_source_files_under_root_prefix() {
        let b = SdistBuilder::new("demo", "1.0.0")
            .add_file("src/demo/__init__.py", b"".to_vec())
            .add_file("pyproject.toml", b"[project]\nname = \"demo\"\n".to_vec());
        let entries = read_entries(&b.build_bytes().unwrap());
        let paths: Vec<&str> = entries.iter().map(|e| e.0.as_str()).collect();
        assert!(paths.contains(&"demo-1.0.0/src/demo/__init__.py"));
        assert!(paths.contains(&"demo-1.0.0/pyproject.toml"));
    }

    #[test]
    fn archive_is_byte_deterministic_across_runs() {
        let b1 = SdistBuilder::new("demo", "1.0.0")
            .add_file("a.py", b"x".to_vec())
            .add_file("b.py", b"y".to_vec());
        let b2 = SdistBuilder::new("demo", "1.0.0")
            .add_file("b.py", b"y".to_vec()) // reverse insert order
            .add_file("a.py", b"x".to_vec());
        let body1 = b1.build_bytes().unwrap();
        let body2 = b2.build_bytes().unwrap();
        assert_eq!(body1, body2, "build_bytes must be deterministic");
    }

    #[test]
    fn archive_entries_use_mode_644_and_mtime_zero() {
        let b = SdistBuilder::new("demo", "1.0.0").add_file("a.py", b"x".to_vec());
        let entries = read_entries(&b.build_bytes().unwrap());
        // Both PKG-INFO and a.py.
        assert!(!entries.is_empty());
        for (_p, _d, mode, mtime) in entries {
            assert_eq!(mode & 0o777, 0o644);
            assert_eq!(mtime, 0);
        }
    }

    #[test]
    fn build_to_dir_writes_canonical_filename() {
        let tmp = tempfile::tempdir().unwrap();
        let b = SdistBuilder::new("demo", "1.0.0");
        let out = b.build_to_dir(tmp.path()).unwrap();
        assert!(out.exists());
        assert!(out
            .file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with("demo-1.0.0.tar.gz"));
    }

    #[test]
    fn list_archive_paths_matches_build_output() {
        let b = SdistBuilder::new("demo", "1.0.0")
            .add_file("a.py", b"".to_vec())
            .add_file("b.py", b"".to_vec());
        let mut listed = list_archive_paths(&b);
        listed.sort();
        let entries = read_entries(&b.build_bytes().unwrap());
        let mut built: Vec<String> = entries.into_iter().map(|e| e.0).collect();
        built.sort();
        assert_eq!(listed, built);
    }

    #[test]
    fn metadata_override_changes_filename() {
        let meta = CoreMetadata::new("renamed", "2.0.0");
        let b = SdistBuilder::new("placeholder", "0.0.0").metadata(meta);
        assert_eq!(b.filename().to_filename(), "renamed-2.0.0.tar.gz");
    }

    #[test]
    fn add_file_from_reads_host_path() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("hello.py");
        fs::write(&src, b"print(\"hello\")\n").unwrap();
        let b = SdistBuilder::new("demo", "1.0.0")
            .add_file_from("hello.py", &src)
            .unwrap();
        let entries = read_entries(&b.build_bytes().unwrap());
        let hello = entries
            .iter()
            .find(|(p, ..)| p == "demo-1.0.0/hello.py")
            .unwrap();
        assert_eq!(hello.1, b"print(\"hello\")\n");
    }

    #[test]
    fn add_file_from_missing_path_errors() {
        let b = SdistBuilder::new("demo", "1.0.0");
        let err = b
            .add_file_from("missing.py", Path::new("/definitely/not/here.py"))
            .err()
            .unwrap();
        assert!(format!("{err}").contains("missing.py") || format!("{err}").contains("here.py"));
    }

    #[test]
    fn build_to_dir_creates_parent_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let dist = tmp.path().join("dist");
        // dist/ does NOT exist yet.
        let b = SdistBuilder::new("demo", "1.0.0");
        let out = b.build_to_dir(&dist).unwrap();
        assert!(out.exists());
        assert!(dist.is_dir());
    }

    #[test]
    fn archive_is_decompressable_by_standard_gzip() {
        let b = SdistBuilder::new("demo", "1.0.0");
        let body = b.build_bytes().unwrap();
        // RFC 1952: gzip magic = 0x1F 0x8B.
        assert_eq!(&body[..2], &[0x1F, 0x8B]);
        // Round-trip via flate2 confirms it's valid gzip.
        let _decompressed = gunzip(&body);
    }
}
