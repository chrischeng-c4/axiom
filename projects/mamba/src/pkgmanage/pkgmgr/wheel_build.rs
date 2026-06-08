// Wheel build layer (Tick 39).
//
// Replicates the `uv build` wheel side: PEP 427 filename composition,
// PEP 491-shaped WHEEL + METADATA renderers, and a deterministic zip
// archive writer that produces a wheel byte-identical (modulo zip
// metadata fields we explicitly normalize) to what setuptools /
// flit_core would emit.
//
// Layered exactly like the other ticks in this branch:
//
//   1. *Filename* (pure data): `WheelFilename` + `compose_filename`
//      build `{name}-{version}-{python}-{abi}-{platform}.whl` from
//      typed parts, with PEP 427 escape rules for `-` in name/version.
//
//   2. *Renderers* (pure string): `render_wheel_metadata` and
//      `render_core_metadata` emit the body of `dist-info/WHEEL` and
//      `dist-info/METADATA`. Both match the byte format consumed by
//      our own installer + the wider PEP 427 ecosystem.
//
//   3. *Archive* (real I/O): `WheelBuilder` collects in-memory blobs,
//      RECORDs them via `record_writer`, and writes a single
//      deterministic zip with stored modification time = zero, sorted
//      entry names, and `Deflated` compression. The archive driver
//      delegates to the existing `zip` crate (already a dep) so this
//      Tick adds no new dependencies.
//
// What's *not* in scope (deferred to a later Tick):
//   * PEP 517 backend invocation (`prepare_metadata_for_build_wheel`,
//     `build_wheel`) — that's already drafted in `pep517.rs`. This
//     module is the *output* side: given the inputs, produce a wheel.
//   * compatibility tag negotiation — accept any tag the caller asks
//     for. `tags.rs` owns the negotiation table.
//   * source-distribution (`.tar.gz`) creation — `sdist.rs` covers
//     that today.

use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::record_writer::{self, RecordEntryDraft};
use crate::pkgmanage::pkgmgr::types::IndexError;

/// Typed wheel filename. PEP 427 requires every field; "any" still
/// has to be encoded explicitly (e.g. `py3-none-any`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WheelFilename {
    /// Distribution name, *unescaped* (we apply the PEP 427 escape on
    /// render so the caller passes the user-facing project name).
    pub name: String,
    /// PEP 440 version string.
    pub version: String,
    /// Optional build tag — e.g. `1`. None ≡ omit.
    pub build_tag: Option<String>,
    /// Python implementation/version tag, e.g. `py3` or `cp312`.
    pub python_tag: String,
    /// ABI tag, e.g. `none` or `cp312`.
    pub abi_tag: String,
    /// Platform tag, e.g. `any` or `manylinux_2_28_x86_64`.
    pub platform_tag: String,
}

impl WheelFilename {
    /// Render to the canonical filename. PEP 427 escapes `-` in
    /// name + version (and only those fields) into `_`.
    pub fn to_filename(&self) -> String {
        let n = escape_filename_segment(&self.name);
        let v = escape_filename_segment(&self.version);
        let core = format!("{n}-{v}");
        let with_build = match &self.build_tag {
            Some(b) if !b.is_empty() => format!("{core}-{b}"),
            _ => core,
        };
        format!(
            "{with_build}-{py}-{abi}-{plat}.whl",
            py = self.python_tag,
            abi = self.abi_tag,
            plat = self.platform_tag,
        )
    }

    /// dist-info directory base = `{escaped_name}-{escaped_version}`.
    pub fn dist_info_dir(&self) -> String {
        format!(
            "{}-{}.dist-info",
            escape_filename_segment(&self.name),
            escape_filename_segment(&self.version),
        )
    }
}

/// PEP 427 escape: replace `-` with `_` in name and version. We also
/// strip any character not in the allowed set (alnum + `_` + `.` + `+`)
/// — same conservative pass as setuptools.
fn escape_filename_segment(seg: &str) -> String {
    seg.chars()
        .map(|c| if c == '-' { '_' } else { c })
        .collect()
}

/// Compose a `WheelFilename` from the user-facing project name +
/// PEP 440 version + a `(python, abi, platform)` tag triple.
pub fn compose_filename(
    name: &str,
    version: &str,
    python_tag: &str,
    abi_tag: &str,
    platform_tag: &str,
) -> WheelFilename {
    WheelFilename {
        name: name.to_string(),
        version: version.to_string(),
        build_tag: None,
        python_tag: python_tag.to_string(),
        abi_tag: abi_tag.to_string(),
        platform_tag: platform_tag.to_string(),
    }
}

/// Body of `dist-info/WHEEL`. PEP 491 specifies keys; we pin
/// `Wheel-Version: 1.0` (matches every public tool today) and let
/// the caller pin Generator + Root-Is-Purelib + Tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WheelMetadata {
    pub generator: String,
    pub root_is_purelib: bool,
    /// One or more `Tag:` lines (e.g. `["py3-none-any"]`). PEP 425
    /// allows multiple compatibility tags per wheel; we render each on
    /// its own line.
    pub tags: Vec<String>,
    pub build_tag: Option<String>,
}

impl WheelMetadata {
    pub fn new(generator: impl Into<String>) -> Self {
        WheelMetadata {
            generator: generator.into(),
            root_is_purelib: true,
            tags: Vec::new(),
            build_tag: None,
        }
    }
}

/// Render `dist-info/WHEEL` body. Keys appear in PEP 491 order.
pub fn render_wheel_metadata(meta: &WheelMetadata) -> String {
    let mut out = String::new();
    out.push_str("Wheel-Version: 1.0\n");
    out.push_str(&format!("Generator: {}\n", meta.generator));
    out.push_str(&format!(
        "Root-Is-Purelib: {}\n",
        if meta.root_is_purelib { "true" } else { "false" }
    ));
    for tag in &meta.tags {
        out.push_str(&format!("Tag: {tag}\n"));
    }
    if let Some(build) = &meta.build_tag {
        out.push_str(&format!("Build: {build}\n"));
    }
    out
}

/// PEP 621-derived core metadata for `dist-info/METADATA`. We model
/// only the fields a wheel must / commonly carries; the renderer
/// emits the canonical RFC 822-style format installers expect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreMetadata {
    /// Metadata-Version pin. `2.1` is the floor accepted by the
    /// installer half of this branch.
    pub metadata_version: String,
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub description_content_type: Option<String>,
    pub author: Option<String>,
    pub author_email: Option<String>,
    pub license: Option<String>,
    pub requires_python: Option<String>,
    /// `Requires-Dist:` lines verbatim — caller has already formatted
    /// `requests>=2.31.0; python_version >= "3.11"` etc.
    pub requires_dist: Vec<String>,
    /// `Provides-Extra: NAME` lines for declared extras.
    pub provides_extras: Vec<String>,
    /// Classifiers (PEP 301).
    pub classifiers: Vec<String>,
    /// Project URLs as (label, url) pairs.
    pub project_urls: Vec<(String, String)>,
    /// Free-form keywords.
    pub keywords: Vec<String>,
}

impl CoreMetadata {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        CoreMetadata {
            metadata_version: "2.1".into(),
            name: name.into(),
            version: version.into(),
            summary: None,
            description: None,
            description_content_type: None,
            author: None,
            author_email: None,
            license: None,
            requires_python: None,
            requires_dist: Vec::new(),
            provides_extras: Vec::new(),
            classifiers: Vec::new(),
            project_urls: Vec::new(),
            keywords: Vec::new(),
        }
    }
}

/// Render `dist-info/METADATA` body. Headers come in PEP 621 canonical
/// order; the long description is appended after a single blank line.
pub fn render_core_metadata(meta: &CoreMetadata) -> String {
    let mut out = String::new();
    out.push_str(&format!("Metadata-Version: {}\n", meta.metadata_version));
    out.push_str(&format!("Name: {}\n", meta.name));
    out.push_str(&format!("Version: {}\n", meta.version));
    if let Some(s) = &meta.summary {
        out.push_str(&format!("Summary: {s}\n"));
    }
    if !meta.keywords.is_empty() {
        out.push_str(&format!("Keywords: {}\n", meta.keywords.join(",")));
    }
    if let Some(a) = &meta.author {
        out.push_str(&format!("Author: {a}\n"));
    }
    if let Some(a) = &meta.author_email {
        out.push_str(&format!("Author-email: {a}\n"));
    }
    if let Some(l) = &meta.license {
        out.push_str(&format!("License: {l}\n"));
    }
    for c in &meta.classifiers {
        out.push_str(&format!("Classifier: {c}\n"));
    }
    if let Some(rp) = &meta.requires_python {
        out.push_str(&format!("Requires-Python: {rp}\n"));
    }
    for r in &meta.requires_dist {
        out.push_str(&format!("Requires-Dist: {r}\n"));
    }
    for e in &meta.provides_extras {
        out.push_str(&format!("Provides-Extra: {e}\n"));
    }
    for (label, url) in &meta.project_urls {
        out.push_str(&format!("Project-URL: {label}, {url}\n"));
    }
    if let Some(ct) = &meta.description_content_type {
        out.push_str(&format!("Description-Content-Type: {ct}\n"));
    }
    if let Some(desc) = &meta.description {
        out.push('\n');
        out.push_str(desc);
        if !desc.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

/// Parse `dist-info/METADATA` (or sdist `PKG-INFO`) back into a
/// `CoreMetadata`. Round-trips with `render_core_metadata` for any value
/// it can emit.
///
/// RFC 822 rules applied (Tick 74):
///   * Headers are unfolded — a line that starts with a space or tab is
///     a continuation of the previous header, with the leading whitespace
///     collapsed to a single space.
///   * The first blank line separates headers from the description body
///     (Description field).
///   * Header names match case-insensitively, but the canonical names
///     emitted by the writer are recognised first.
///   * Multi-value fields (`Requires-Dist`, `Provides-Extra`,
///     `Classifier`, `Project-URL`) collect in source order.
///   * `Project-URL` is split on the first `,` into `(label, url)` with
///     surrounding whitespace trimmed.
///   * `Keywords` is `,`-separated; empty pieces are dropped to mirror
///     the writer (which joins a `Vec<String>` with `,`).
///   * Unknown header names are tolerated and silently ignored — METADATA
///     readers must round-trip newer-version fields they don't know.
pub fn parse_core_metadata(src: &str) -> Result<CoreMetadata, IndexError> {
    // Normalize line endings up front so CRLF inputs from Windows wheels
    // round-trip identically.
    let normalized = src.replace("\r\n", "\n");
    let src = normalized.as_str();
    // 1. Split on first blank line: header section vs description body.
    let (header_block, description) = split_header_body(src);

    // 2. Unfold continuation lines in the header section.
    let unfolded = unfold_headers(header_block);

    // 3. Walk fields and slot them into CoreMetadata.
    let mut meta = CoreMetadata::new(String::new(), String::new());
    let mut saw_name = false;
    let mut saw_version = false;
    let mut saw_metadata_version = false;

    for (name, value) in &unfolded {
        let key = name.to_ascii_lowercase();
        match key.as_str() {
            "metadata-version" => {
                meta.metadata_version = value.clone();
                saw_metadata_version = true;
            }
            "name" => {
                meta.name = value.clone();
                saw_name = true;
            }
            "version" => {
                meta.version = value.clone();
                saw_version = true;
            }
            "summary" => meta.summary = Some(value.clone()),
            "description-content-type" => meta.description_content_type = Some(value.clone()),
            "author" => meta.author = Some(value.clone()),
            "author-email" => meta.author_email = Some(value.clone()),
            "license" => meta.license = Some(value.clone()),
            "requires-python" => meta.requires_python = Some(value.clone()),
            "requires-dist" => meta.requires_dist.push(value.clone()),
            "provides-extra" => meta.provides_extras.push(value.clone()),
            "classifier" => meta.classifiers.push(value.clone()),
            "project-url" => match value.split_once(',') {
                Some((label, url)) => meta
                    .project_urls
                    .push((label.trim().to_string(), url.trim().to_string())),
                None => {
                    return Err(IndexError::ParseError {
                        url: "METADATA".into(),
                        detail: format!(
                            "malformed Project-URL header (expected 'label, url'): {value:?}"
                        ),
                    });
                }
            },
            "keywords" => {
                meta.keywords = value
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect();
            }
            "description" => {
                // Tools sometimes inline Description as a header instead of
                // using the body. Accept both.
                meta.description = Some(value.clone());
            }
            _ => { /* unknown header — tolerated for forward compat */ }
        }
    }

    if !saw_metadata_version {
        return Err(IndexError::ParseError {
            url: "METADATA".into(),
            detail: "missing required header 'Metadata-Version'".into(),
        });
    }
    if !saw_name {
        return Err(IndexError::ParseError {
            url: "METADATA".into(),
            detail: "missing required header 'Name'".into(),
        });
    }
    if !saw_version {
        return Err(IndexError::ParseError {
            url: "METADATA".into(),
            detail: "missing required header 'Version'".into(),
        });
    }

    // 4. Body description takes precedence over any inline Description
    //    header (it's the more conventional shape).
    if !description.is_empty() {
        meta.description = Some(description);
    }

    Ok(meta)
}

fn split_header_body(src: &str) -> (&str, String) {
    // Find first physical blank line. We treat a line of only `\r` as
    // blank too, to be CRLF-tolerant.
    let mut byte_idx = 0usize;
    let mut found: Option<usize> = None;
    for line in src.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if trimmed.is_empty() {
            // Blank line — header section ends here. Body starts after.
            found = Some(byte_idx + line.len());
            break;
        }
        byte_idx += line.len();
    }
    match found {
        Some(end) => {
            let body = src[end..].to_string();
            let body = body.trim_end_matches('\n').to_string();
            (&src[..byte_idx], body)
        }
        None => (src, String::new()),
    }
}

fn unfold_headers(block: &str) -> Vec<(String, String)> {
    let mut fields: Vec<(String, String)> = Vec::new();
    for raw in block.lines() {
        if raw.is_empty() {
            continue;
        }
        // Continuation: leading whitespace.
        if raw.starts_with(' ') || raw.starts_with('\t') {
            if let Some(last) = fields.last_mut() {
                let cont = raw.trim_start();
                if !last.1.is_empty() {
                    last.1.push(' ');
                }
                last.1.push_str(cont);
            }
            continue;
        }
        match raw.split_once(':') {
            Some((name, value)) => {
                fields.push((name.trim().to_string(), value.trim().to_string()));
            }
            None => {
                // Malformed line without a colon — skip rather than reject,
                // matching the forward-compat stance.
            }
        }
    }
    fields
}

/// In-memory wheel builder. Collects (arcname, bytes) pairs, plus the
/// caller-supplied filename + WHEEL/METADATA. At `finish()` time we:
///   * insert the three dist-info files
///     (METADATA, WHEEL, optional entry_points.txt),
///   * compute SHA256+size for *every* archive entry,
///   * append the dist-info/RECORD CSV (with the canonical blank-hash
///     self-row from `record_writer`),
///   * write a single zip archive at the chosen output path.
///
/// Archive determinism: every entry uses mtime = 1980-01-01 (zip
/// epoch) and Deflate compression. Entry order is sorted by arcname so
/// hash of the bytes is reproducible across runs.
pub struct WheelBuilder {
    filename: WheelFilename,
    wheel_meta: WheelMetadata,
    core_meta: CoreMetadata,
    /// Optional entry_points.txt body. None ≡ omit.
    entry_points_txt: Option<String>,
    /// arcname (forward-slash) -> bytes. Caller-owned data files.
    files: BTreeMap<String, Vec<u8>>,
}

impl WheelBuilder {
    pub fn new(
        filename: WheelFilename,
        wheel_meta: WheelMetadata,
        core_meta: CoreMetadata,
    ) -> Self {
        WheelBuilder {
            filename,
            wheel_meta,
            core_meta,
            entry_points_txt: None,
            files: BTreeMap::new(),
        }
    }
    pub fn add_file(&mut self, arcname: impl Into<String>, body: impl Into<Vec<u8>>) {
        self.files.insert(arcname.into(), body.into());
    }
    pub fn set_entry_points(&mut self, body: impl Into<String>) {
        self.entry_points_txt = Some(body.into());
    }

    /// Render the final byte buffer of the wheel — bytes the caller
    /// may write to disk or hash. The split-out form is convenient for
    /// tests that want to re-read the archive without writing it.
    pub fn build_bytes(&self) -> Result<Vec<u8>, IndexError> {
        // Snapshot full file list incl. dist-info entries.
        let dist_info = self.filename.dist_info_dir();
        let mut all: BTreeMap<String, Vec<u8>> = self.files.clone();
        let metadata_arc = format!("{dist_info}/METADATA");
        let wheel_arc = format!("{dist_info}/WHEEL");
        let record_arc = format!("{dist_info}/RECORD");
        let entry_points_arc = format!("{dist_info}/entry_points.txt");

        all.insert(metadata_arc.clone(), render_core_metadata(&self.core_meta).into_bytes());
        all.insert(wheel_arc.clone(), render_wheel_metadata(&self.wheel_meta).into_bytes());
        if let Some(ep) = &self.entry_points_txt {
            all.insert(entry_points_arc.clone(), ep.as_bytes().to_vec());
        }

        // RECORD entries for everything except RECORD itself.
        let mut record_drafts: Vec<RecordEntryDraft> = Vec::with_capacity(all.len());
        for (name, body) in &all {
            let (digest, size) = record_writer::hash_bytes(body);
            record_drafts.push(RecordEntryDraft {
                path: name.clone(),
                sha256_b64url: Some(digest),
                size: Some(size),
            });
        }
        let record_body = record_writer::render_record(&record_drafts, &record_arc)?;
        all.insert(record_arc.clone(), record_body.into_bytes());

        // Now write the zip in sorted order.
        let mut buf = Vec::new();
        {
            let cursor = Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(cursor);
            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .last_modified_time(
                    // PEP 491 / "reproducible builds" — pin mtime to
                    // the zip epoch so the archive is byte-stable.
                    zip::DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0)
                        .unwrap_or_default(),
                )
                .unix_permissions(0o644);
            for (name, body) in &all {
                zip.start_file(name, options).map_err(zip_err)?;
                zip.write_all(body).map_err(|e| IndexError::ParseError {
                    url: name.clone(),
                    detail: format!("zip write: {e}"),
                })?;
            }
            zip.finish().map_err(zip_err)?;
        }
        Ok(buf)
    }

    /// Write the wheel to `out_dir` and return the full path. The
    /// filename is `compose_filename`-derived; the caller need not
    /// pre-create the directory.
    pub fn build_to_dir(&self, out_dir: &Path) -> Result<PathBuf, IndexError> {
        std::fs::create_dir_all(out_dir).map_err(|e| IndexError::ParseError {
            url: out_dir.display().to_string(),
            detail: format!("creating wheel output dir: {e}"),
        })?;
        let bytes = self.build_bytes()?;
        let path = out_dir.join(self.filename.to_filename());
        std::fs::write(&path, &bytes).map_err(|e| IndexError::ParseError {
            url: path.display().to_string(),
            detail: format!("writing wheel: {e}"),
        })?;
        Ok(path)
    }
}

fn zip_err(e: zip::result::ZipError) -> IndexError {
    IndexError::ParseError {
        url: "<wheel zip>".into(),
        detail: format!("zip: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn filename_renders_canonical_form() {
        let fname = compose_filename("my-pkg", "1.2.3", "py3", "none", "any");
        assert_eq!(fname.to_filename(), "my_pkg-1.2.3-py3-none-any.whl");
        assert_eq!(fname.dist_info_dir(), "my_pkg-1.2.3.dist-info");
    }

    #[test]
    fn filename_with_build_tag() {
        let fname = WheelFilename {
            name: "x".into(),
            version: "1.0".into(),
            build_tag: Some("1".into()),
            python_tag: "py3".into(),
            abi_tag: "none".into(),
            platform_tag: "any".into(),
        };
        assert_eq!(fname.to_filename(), "x-1.0-1-py3-none-any.whl");
    }

    #[test]
    fn filename_empty_build_tag_treated_as_absent() {
        let fname = WheelFilename {
            name: "x".into(),
            version: "1".into(),
            build_tag: Some(String::new()),
            python_tag: "py3".into(),
            abi_tag: "none".into(),
            platform_tag: "any".into(),
        };
        assert_eq!(fname.to_filename(), "x-1-py3-none-any.whl");
    }

    #[test]
    fn filename_escapes_dashes_in_name_and_version() {
        // PEP 440 doesn't allow `-` in version, but uv build-tag tests
        // exercise the escape anyway because pre-release fragments can
        // round-trip. Confirm both fields get rewritten.
        let fname = compose_filename("a-b-c", "1-2", "py3", "none", "any");
        assert_eq!(fname.to_filename(), "a_b_c-1_2-py3-none-any.whl");
    }

    #[test]
    fn wheel_metadata_renders_pep_491_keys_in_order() {
        let mut wm = WheelMetadata::new("mamba 0.1");
        wm.tags = vec!["py3-none-any".into()];
        let body = render_wheel_metadata(&wm);
        let expected = "Wheel-Version: 1.0\n\
                        Generator: mamba 0.1\n\
                        Root-Is-Purelib: true\n\
                        Tag: py3-none-any\n";
        assert_eq!(body, expected);
    }

    #[test]
    fn wheel_metadata_multiple_tags_each_on_own_line() {
        let mut wm = WheelMetadata::new("mamba 0.1");
        wm.tags = vec!["py3-none-any".into(), "py2.py3-none-any".into()];
        let body = render_wheel_metadata(&wm);
        let tag_lines: Vec<&str> = body
            .lines()
            .filter(|l| l.starts_with("Tag: "))
            .collect();
        assert_eq!(tag_lines, vec!["Tag: py3-none-any", "Tag: py2.py3-none-any"]);
    }

    #[test]
    fn wheel_metadata_root_is_purelib_false() {
        let mut wm = WheelMetadata::new("mamba");
        wm.root_is_purelib = false;
        wm.tags = vec!["cp312-cp312-manylinux_2_28_x86_64".into()];
        assert!(render_wheel_metadata(&wm).contains("Root-Is-Purelib: false\n"));
    }

    #[test]
    fn wheel_metadata_with_build_tag() {
        let mut wm = WheelMetadata::new("mamba");
        wm.tags = vec!["py3-none-any".into()];
        wm.build_tag = Some("3".into());
        assert!(render_wheel_metadata(&wm).contains("Build: 3\n"));
    }

    #[test]
    fn core_metadata_minimum_required_keys() {
        let m = CoreMetadata::new("requests", "2.31.0");
        let body = render_core_metadata(&m);
        assert!(body.starts_with("Metadata-Version: 2.1\n"));
        assert!(body.contains("Name: requests\n"));
        assert!(body.contains("Version: 2.31.0\n"));
        // No description means no trailing body separator.
        assert_eq!(body.matches('\n').count(), 3);
    }

    #[test]
    fn core_metadata_renders_full_field_set() {
        let mut m = CoreMetadata::new("flask", "3.0.0");
        m.summary = Some("The microframework".into());
        m.requires_python = Some(">=3.11".into());
        m.requires_dist = vec!["jinja2>=3.0".into(), "werkzeug>=3.0".into()];
        m.classifiers = vec!["License :: OSI Approved :: BSD License".into()];
        m.project_urls = vec![("Homepage".into(), "https://flask.palletsprojects.com".into())];
        m.provides_extras = vec!["async".into()];
        m.description = Some("Long form description.".into());
        m.description_content_type = Some("text/markdown".into());
        m.keywords = vec!["wsgi".into(), "web".into()];
        m.author = Some("Pallets".into());
        m.license = Some("BSD-3-Clause".into());
        let body = render_core_metadata(&m);
        assert!(body.contains("Summary: The microframework\n"));
        assert!(body.contains("Keywords: wsgi,web\n"));
        assert!(body.contains("Author: Pallets\n"));
        assert!(body.contains("License: BSD-3-Clause\n"));
        assert!(body.contains("Classifier: License :: OSI Approved :: BSD License\n"));
        assert!(body.contains("Requires-Python: >=3.11\n"));
        assert!(body.contains("Requires-Dist: jinja2>=3.0\n"));
        assert!(body.contains("Requires-Dist: werkzeug>=3.0\n"));
        assert!(body.contains("Provides-Extra: async\n"));
        assert!(body.contains("Project-URL: Homepage, https://flask.palletsprojects.com\n"));
        assert!(body.contains("Description-Content-Type: text/markdown\n"));
        assert!(body.ends_with("Long form description.\n"));
    }

    #[test]
    fn builder_writes_wheel_with_expected_dist_info_entries() {
        let fname = compose_filename("my-pkg", "0.1.0", "py3", "none", "any");
        let mut wm = WheelMetadata::new("mamba 0.0.1");
        wm.tags = vec!["py3-none-any".into()];
        let cm = CoreMetadata::new("my-pkg", "0.1.0");
        let mut b = WheelBuilder::new(fname.clone(), wm, cm);
        b.add_file("my_pkg/__init__.py", "from .core import main\n");
        b.add_file("my_pkg/core.py", "def main():\n    print('hi')\n");

        let bytes = b.build_bytes().unwrap();
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();

        let names: Vec<String> = (0..zip.len())
            .map(|i| zip.by_index(i).unwrap().name().to_string())
            .collect();
        assert!(names.contains(&"my_pkg/__init__.py".to_string()));
        assert!(names.contains(&"my_pkg/core.py".to_string()));
        let di = fname.dist_info_dir();
        assert!(names.contains(&format!("{di}/WHEEL")));
        assert!(names.contains(&format!("{di}/METADATA")));
        assert!(names.contains(&format!("{di}/RECORD")));
    }

    #[test]
    fn builder_emits_record_consumable_by_installer_parser() {
        // The wheel build half should produce a RECORD the existing
        // installer half (from Tick 35) can read back without complaint.
        let fname = compose_filename("x", "0.0.1", "py3", "none", "any");
        let wm = {
            let mut w = WheelMetadata::new("mamba 0.0.1");
            w.tags = vec!["py3-none-any".into()];
            w
        };
        let cm = CoreMetadata::new("x", "0.0.1");
        let mut b = WheelBuilder::new(fname.clone(), wm, cm);
        b.add_file("x/__init__.py", "X = 1\n");
        let bytes = b.build_bytes().unwrap();
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let record_arc = format!("{}/RECORD", fname.dist_info_dir());
        let mut buf = Vec::new();
        zip.by_name(&record_arc).unwrap().read_to_end(&mut buf).unwrap();
        let body = String::from_utf8(buf).unwrap();

        // Round-trip through the installer's parser — guards against
        // future format drift on either side.
        let parsed = crate::pkgmanage::pkgmgr::installer::record::parse(&body)
            .expect("installer parses our RECORD");
        // Must contain at least the package payload, METADATA, WHEEL,
        // entry_points (optional, here absent), and the RECORD self row.
        let paths: Vec<&str> = parsed.iter().map(|e| e.path.as_str()).collect();
        assert!(paths.contains(&"x/__init__.py"));
        assert!(paths.contains(&record_arc.as_str()));
        // Self-row must have blank hash + blank size per PEP 376.
        let self_row = parsed.iter().find(|e| e.path == record_arc).unwrap();
        assert!(self_row.sha256_b64url.is_none());
        assert!(self_row.size.is_none());
    }

    #[test]
    fn builder_includes_entry_points_when_set() {
        let fname = compose_filename("x", "0.1.0", "py3", "none", "any");
        let mut wm = WheelMetadata::new("mamba");
        wm.tags = vec!["py3-none-any".into()];
        let cm = CoreMetadata::new("x", "0.1.0");
        let mut b = WheelBuilder::new(fname.clone(), wm, cm);
        b.add_file("x/__init__.py", "");
        b.set_entry_points("[console_scripts]\nx = x:main\n");
        let bytes = b.build_bytes().unwrap();
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let arc = format!("{}/entry_points.txt", fname.dist_info_dir());
        let mut buf = Vec::new();
        zip.by_name(&arc).unwrap().read_to_end(&mut buf).unwrap();
        let body = String::from_utf8(buf).unwrap();
        assert!(body.contains("[console_scripts]"));
        assert!(body.contains("x = x:main"));
    }

    #[test]
    fn builder_writes_to_disk_and_returns_path() {
        let tmp = tempfile::tempdir().unwrap();
        let fname = compose_filename("write-me", "0.0.1", "py3", "none", "any");
        let mut wm = WheelMetadata::new("mamba");
        wm.tags = vec!["py3-none-any".into()];
        let cm = CoreMetadata::new("write-me", "0.0.1");
        let mut b = WheelBuilder::new(fname.clone(), wm, cm);
        b.add_file("write_me/__init__.py", "");
        let written = b.build_to_dir(tmp.path()).unwrap();
        assert_eq!(written.file_name().unwrap(), fname.to_filename().as_str());
        assert!(written.is_file());
        // Should be a valid zip.
        let zip = zip::ZipArchive::new(std::fs::File::open(&written).unwrap()).unwrap();
        assert!(zip.len() >= 4); // payload + METADATA + WHEEL + RECORD
    }

    #[test]
    fn builder_archive_is_deterministic_across_runs() {
        let fname = compose_filename("d", "0.0.1", "py3", "none", "any");
        let make = || {
            let mut wm = WheelMetadata::new("mamba 0.0.1");
            wm.tags = vec!["py3-none-any".into()];
            let cm = CoreMetadata::new("d", "0.0.1");
            let mut b = WheelBuilder::new(fname.clone(), wm, cm);
            b.add_file("d/__init__.py", "ZERO = 0\n");
            b.add_file("d/core.py", "def f(): return 1\n");
            b.build_bytes().unwrap()
        };
        let a = make();
        let b = make();
        assert_eq!(a, b, "two builds must produce byte-identical wheels");
    }

    // ----- Tick 74: parse_core_metadata + round-trip ----------------------

    #[test]
    fn parse_minimal_metadata() {
        let src = "Metadata-Version: 2.1\nName: spam\nVersion: 1.0\n";
        let m = parse_core_metadata(src).unwrap();
        assert_eq!(m.metadata_version, "2.1");
        assert_eq!(m.name, "spam");
        assert_eq!(m.version, "1.0");
        assert!(m.summary.is_none());
        assert!(m.description.is_none());
    }

    #[test]
    fn parse_round_trips_full_metadata() {
        let mut original = CoreMetadata::new("eggs", "2.3.4");
        original.summary = Some("Eggs are nutritious".into());
        original.author = Some("Alice".into());
        original.author_email = Some("alice@example.com".into());
        original.license = Some("MIT".into());
        original.requires_python = Some(">=3.10".into());
        original.requires_dist = vec![
            "requests>=2.31.0".into(),
            "click<9; python_version >= \"3.10\"".into(),
        ];
        original.provides_extras = vec!["dev".into(), "docs".into()];
        original.classifiers = vec![
            "Programming Language :: Python :: 3".into(),
            "License :: OSI Approved :: MIT License".into(),
        ];
        original.project_urls = vec![
            ("Homepage".into(), "https://example.com".into()),
            ("Source".into(), "https://github.com/example/eggs".into()),
        ];
        original.keywords = vec!["eggs".into(), "breakfast".into(), "protein".into()];
        original.description_content_type = Some("text/markdown".into());
        // Convention: description text has no trailing newline — the
        // line terminator is a serialization detail the writer re-adds.
        original.description = Some("# Eggs\n\nA fine breakfast item.".into());

        let rendered = render_core_metadata(&original);
        let parsed = parse_core_metadata(&rendered).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn parse_header_case_insensitive() {
        let src = "metadata-version: 2.1\nNAME: spam\nVeRsIoN: 1.0\n";
        let m = parse_core_metadata(src).unwrap();
        assert_eq!(m.name, "spam");
        assert_eq!(m.version, "1.0");
    }

    #[test]
    fn parse_unfolds_continuation_lines() {
        // RFC 822 folding: continuation lines start with whitespace.
        let src = "Metadata-Version: 2.1\nName: spam\nVersion: 1.0\nSummary: one\n two\n\tthree\n";
        let m = parse_core_metadata(src).unwrap();
        assert_eq!(m.summary.as_deref(), Some("one two three"));
    }

    #[test]
    fn parse_handles_multi_value_fields_in_order() {
        let src = "\
Metadata-Version: 2.1
Name: spam
Version: 1.0
Requires-Dist: a
Requires-Dist: b
Requires-Dist: c
Classifier: Programming Language :: Python :: 3
Classifier: License :: OSI Approved :: MIT License
Provides-Extra: dev
Provides-Extra: test
";
        let m = parse_core_metadata(src).unwrap();
        assert_eq!(m.requires_dist, vec!["a", "b", "c"]);
        assert_eq!(
            m.classifiers,
            vec![
                "Programming Language :: Python :: 3",
                "License :: OSI Approved :: MIT License",
            ]
        );
        assert_eq!(m.provides_extras, vec!["dev", "test"]);
    }

    #[test]
    fn parse_project_url_splits_on_first_comma() {
        let src = "\
Metadata-Version: 2.1
Name: spam
Version: 1.0
Project-URL: Homepage, https://example.com
Project-URL: Source Code, https://github.com/x/y
";
        let m = parse_core_metadata(src).unwrap();
        assert_eq!(
            m.project_urls,
            vec![
                ("Homepage".into(), "https://example.com".into()),
                ("Source Code".into(), "https://github.com/x/y".into()),
            ]
        );
    }

    #[test]
    fn parse_project_url_without_comma_rejected() {
        let src = "Metadata-Version: 2.1\nName: spam\nVersion: 1.0\nProject-URL: just-a-url\n";
        let err = parse_core_metadata(src).unwrap_err();
        let s = err.to_string();
        assert!(s.contains("Project-URL"), "got {s}");
    }

    #[test]
    fn parse_keywords_split_on_comma_and_trim() {
        let src = "Metadata-Version: 2.1\nName: x\nVersion: 1\nKeywords: a, b ,c\n";
        let m = parse_core_metadata(src).unwrap();
        assert_eq!(m.keywords, vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_keywords_drops_empty_pieces() {
        let src = "Metadata-Version: 2.1\nName: x\nVersion: 1\nKeywords: a,,b,\n";
        let m = parse_core_metadata(src).unwrap();
        assert_eq!(m.keywords, vec!["a", "b"]);
    }

    #[test]
    fn parse_description_from_body() {
        let src = "Metadata-Version: 2.1\nName: x\nVersion: 1\n\nlong\ndescription\n";
        let m = parse_core_metadata(src).unwrap();
        assert_eq!(m.description.as_deref(), Some("long\ndescription"));
    }

    #[test]
    fn parse_body_takes_precedence_over_inline_description() {
        let src = "\
Metadata-Version: 2.1
Name: x
Version: 1
Description: ignored

body wins
";
        let m = parse_core_metadata(src).unwrap();
        assert_eq!(m.description.as_deref(), Some("body wins"));
    }

    #[test]
    fn parse_unknown_headers_tolerated() {
        let src = "\
Metadata-Version: 2.4
Name: x
Version: 1
Dynamic: Author
License-Expression: MIT
License-File: LICENSE.txt
";
        let m = parse_core_metadata(src).unwrap();
        assert_eq!(m.metadata_version, "2.4");
        assert_eq!(m.name, "x");
    }

    #[test]
    fn parse_missing_metadata_version_rejected() {
        let src = "Name: x\nVersion: 1\n";
        let err = parse_core_metadata(src).unwrap_err();
        assert!(err.to_string().contains("Metadata-Version"));
    }

    #[test]
    fn parse_missing_name_rejected() {
        let src = "Metadata-Version: 2.1\nVersion: 1\n";
        let err = parse_core_metadata(src).unwrap_err();
        assert!(err.to_string().contains("Name"));
    }

    #[test]
    fn parse_missing_version_rejected() {
        let src = "Metadata-Version: 2.1\nName: x\n";
        let err = parse_core_metadata(src).unwrap_err();
        assert!(err.to_string().contains("Version"));
    }

    #[test]
    fn parse_crlf_line_endings_tolerated() {
        let src = "Metadata-Version: 2.1\r\nName: x\r\nVersion: 1\r\n\r\nbody\r\n";
        let m = parse_core_metadata(src).unwrap();
        assert_eq!(m.name, "x");
        assert_eq!(m.description.as_deref(), Some("body"));
    }

    #[test]
    fn parse_round_trips_empty_optionals() {
        let original = CoreMetadata::new("min", "0.0.1");
        let rendered = render_core_metadata(&original);
        let parsed = parse_core_metadata(&rendered).unwrap();
        assert_eq!(parsed, original);
    }
}
