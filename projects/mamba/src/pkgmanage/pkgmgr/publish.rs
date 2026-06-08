// `uv publish` — credential resolution + upload-form builder (Tick 45).
//
// Pure data layer. Mirrors what `twine` / `uv publish` do on the way to
// PyPI's legacy upload endpoint:
//
//   1. Parse `~/.pypirc` (INI-style) into a name -> `PypiRepository` map.
//   2. Resolve a target repository: explicit `--repository`, environment
//      overrides (`UV_PUBLISH_*`, `TWINE_REPOSITORY_URL`, etc.), or the
//      `pypi` / `testpypi` defaults.
//   3. Build the multipart/form-data body that the upload endpoint
//      expects — the same field set twine ships with: `:action=file_upload`,
//      `protocol_version=1`, package metadata, and the artifact bytes.
//
// What this tick does NOT cover (deferred):
//   * The actual HTTP POST + retry/backoff loop — lives in a future
//     driver tick on top of `http.rs`.
//   * Trusted-publishing (PEP 740 / OIDC) credentials — needs the
//     OIDC token-exchange flow which is its own surface.
//   * Sigstore signature attachment — separate `--sign` Tick.

use std::collections::BTreeMap;
use std::path::PathBuf;

/// Canonical PyPI upload endpoint.
pub const DEFAULT_PYPI_URL: &str = "https://upload.pypi.org/legacy/";
/// TestPyPI upload endpoint.
pub const TEST_PYPI_URL: &str = "https://test.pypi.org/legacy/";

/// One repository section parsed out of `~/.pypirc`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PypiRepository {
    /// Repository name as it appears in `[<name>]` — e.g. `pypi`, `testpypi`.
    pub name: String,
    /// Upload URL. Normalized to include a trailing slash.
    pub url: String,
    /// Username (typically `__token__` for API-token auth).
    pub username: Option<String>,
    /// Password (the API token body when `username == "__token__"`).
    pub password: Option<String>,
    /// Optional CA bundle path; populated from `ca_cert = ...` in the
    /// INI body for self-hosted indices.
    pub ca_cert: Option<PathBuf>,
}

impl PypiRepository {
    pub fn pypi() -> Self {
        PypiRepository {
            name: "pypi".into(),
            url: DEFAULT_PYPI_URL.into(),
            username: None,
            password: None,
            ca_cert: None,
        }
    }
    pub fn testpypi() -> Self {
        PypiRepository {
            name: "testpypi".into(),
            url: TEST_PYPI_URL.into(),
            username: None,
            password: None,
            ca_cert: None,
        }
    }
}

/// Parse a `.pypirc` INI body into a name -> repository map.
///
/// `.pypirc` predates TOML and Python keeps it on `configparser` (INI).
/// We honor the subset Twine + uv care about:
///
/// ```ini
/// [distutils]
/// index-servers =
///     pypi
///     testpypi
///     internal
///
/// [pypi]
/// username = __token__
/// password = pypi-AgEABC...
///
/// [testpypi]
/// repository = https://test.pypi.org/legacy/
/// username = __token__
/// password = pypi-test-AgE...
///
/// [internal]
/// repository = https://nexus.corp.local/pypi/
/// username = ci
/// password = secret
/// ca_cert = /etc/ssl/corp.pem
/// ```
///
/// The `[distutils].index-servers` list is purely informational; we
/// return every `[section]` we can parse, so extra entries don't get
/// silently dropped.
pub fn parse_pypirc(src: &str) -> BTreeMap<String, PypiRepository> {
    let mut out: BTreeMap<String, PypiRepository> = BTreeMap::new();
    let mut current_section: Option<String> = None;
    let mut current_repo: Option<PypiRepository> = None;
    let mut pending_key: Option<String> = None;
    let mut pending_value: Vec<String> = Vec::new();

    fn flush_kv(
        current_repo: &mut Option<PypiRepository>,
        pending_key: &mut Option<String>,
        pending_value: &mut Vec<String>,
    ) {
        if let (Some(repo), Some(key)) = (current_repo.as_mut(), pending_key.take()) {
            let value = pending_value.join(" ").trim().to_string();
            apply_pair(repo, &key, &value);
        }
        pending_value.clear();
    }

    fn flush_section(
        out: &mut BTreeMap<String, PypiRepository>,
        section: &mut Option<String>,
        repo: &mut Option<PypiRepository>,
    ) {
        if let (Some(name), Some(mut r)) = (section.take(), repo.take()) {
            normalize_repo(&mut r);
            out.insert(name, r);
        }
    }

    for raw_line in src.lines() {
        let line = raw_line.trim_end();
        // Strip comments — `;` or `#` at start of trimmed-left line.
        let trimmed_lhs = line.trim_start();
        if trimmed_lhs.starts_with('#') || trimmed_lhs.starts_with(';') {
            continue;
        }
        if trimmed_lhs.is_empty() {
            // Blank line ends a continuation but not a section.
            flush_kv(&mut current_repo, &mut pending_key, &mut pending_value);
            continue;
        }
        if trimmed_lhs.starts_with('[') && trimmed_lhs.ends_with(']') {
            flush_kv(&mut current_repo, &mut pending_key, &mut pending_value);
            flush_section(&mut out, &mut current_section, &mut current_repo);
            let name = trimmed_lhs[1..trimmed_lhs.len() - 1].trim().to_string();
            if name == "distutils" {
                // We don't carry the index-servers list; skip.
                current_section = None;
                current_repo = None;
            } else {
                current_section = Some(name.clone());
                current_repo = Some(PypiRepository {
                    name,
                    url: String::new(),
                    username: None,
                    password: None,
                    ca_cert: None,
                });
            }
            continue;
        }
        // Key continuation: leading whitespace on a line continues the
        // previous value.
        if (line.starts_with(' ') || line.starts_with('\t')) && pending_key.is_some() {
            pending_value.push(trimmed_lhs.to_string());
            continue;
        }
        // New `key = value` (or `key: value`).
        flush_kv(&mut current_repo, &mut pending_key, &mut pending_value);
        let split = line.splitn(2, |c: char| c == '=' || c == ':').collect::<Vec<_>>();
        if split.len() != 2 {
            continue;
        }
        let key = split[0].trim().to_ascii_lowercase();
        let value = split[1].trim().to_string();
        pending_key = Some(key);
        if !value.is_empty() {
            pending_value.push(value);
        }
    }
    flush_kv(&mut current_repo, &mut pending_key, &mut pending_value);
    flush_section(&mut out, &mut current_section, &mut current_repo);
    out
}

fn apply_pair(repo: &mut PypiRepository, key: &str, value: &str) {
    match key {
        "repository" | "repository-url" => {
            if !value.is_empty() {
                repo.url = value.to_string();
            }
        }
        "username" => repo.username = Some(value.to_string()),
        "password" => repo.password = Some(value.to_string()),
        "ca_cert" | "ca-cert" => repo.ca_cert = Some(PathBuf::from(value)),
        _ => {}
    }
}

fn normalize_repo(repo: &mut PypiRepository) {
    if repo.url.is_empty() {
        repo.url = match repo.name.as_str() {
            "pypi" => DEFAULT_PYPI_URL.into(),
            "testpypi" => TEST_PYPI_URL.into(),
            _ => String::new(),
        };
    }
    if !repo.url.is_empty() && !repo.url.ends_with('/') {
        repo.url.push('/');
    }
}

/// Resolution result for `uv publish`-style credential selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRepository {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ca_cert: Option<PathBuf>,
}

/// CLI-shaped inputs to `resolve_repository`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PublishInputs {
    /// Explicit `--publish-url <url>`. Wins over everything else.
    pub publish_url: Option<String>,
    /// `--repository <name>` — pick a `.pypirc` section.
    pub repository: Option<String>,
    /// `--username` / `UV_PUBLISH_USERNAME` / `TWINE_USERNAME`.
    pub username: Option<String>,
    /// `--password` / `UV_PUBLISH_PASSWORD` / `TWINE_PASSWORD`.
    pub password: Option<String>,
}

/// Compose the final upload target from CLI inputs + .pypirc.
///
/// Precedence per uv's documented rules:
///
///   1. `publish_url` if provided.
///   2. `repository` name looked up in `.pypirc`, falling back to
///      the canonical `pypi` / `testpypi` defaults when missing.
///   3. Bare default → `pypi`.
///
/// CLI-supplied username/password always override `.pypirc` values.
pub fn resolve_repository(
    inputs: &PublishInputs,
    pypirc: &BTreeMap<String, PypiRepository>,
) -> ResolvedRepository {
    // Start from the .pypirc section (if any).
    let base = if let Some(name) = inputs.repository.as_deref() {
        pypirc.get(name).cloned().unwrap_or_else(|| {
            // Even when the user names a non-existent section, fall
            // back to the canonical defaults so `--repository testpypi`
            // works without a .pypirc.
            match name {
                "pypi" => PypiRepository::pypi(),
                "testpypi" => PypiRepository::testpypi(),
                _ => PypiRepository {
                    name: name.to_string(),
                    url: String::new(),
                    username: None,
                    password: None,
                    ca_cert: None,
                },
            }
        })
    } else {
        pypirc
            .get("pypi")
            .cloned()
            .unwrap_or_else(PypiRepository::pypi)
    };

    let url = inputs
        .publish_url
        .clone()
        .filter(|u| !u.is_empty())
        .unwrap_or(base.url);

    ResolvedRepository {
        url,
        username: inputs.username.clone().or(base.username),
        password: inputs.password.clone().or(base.password),
        ca_cert: base.ca_cert,
    }
}

/// One artifact to upload — sdist or wheel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadArtifact {
    /// PEP 503-normalized project name (`name=` form field value).
    pub name: String,
    /// PEP 440 version string (`version=` form field value).
    pub version: String,
    /// PEP 621 `Metadata-Version` header value (e.g. `2.1`).
    pub metadata_version: String,
    /// `bdist_wheel` or `sdist` — the file-type form field.
    pub file_type: ArtifactKind,
    /// Filename to record in the multipart `filename=` parameter
    /// (e.g. `requests-2.31.0-py3-none-any.whl`).
    pub filename: String,
    /// Raw artifact bytes.
    pub data: Vec<u8>,
    /// sha256-hex digest of `data` (PyPI requires this).
    pub sha256_hex: String,
    /// Optional one-line summary from METADATA `Summary:`.
    pub summary: Option<String>,
    /// Optional Python tag (the `pyversion=` form field). For wheels,
    /// the canonical thing to pass is the wheel's `python_tag` (e.g.
    /// `py3`, `cp312`); sdists omit it.
    pub python_tag: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    Sdist,
    Wheel,
}

impl ArtifactKind {
    fn form_value(self) -> &'static str {
        match self {
            ArtifactKind::Sdist => "sdist",
            ArtifactKind::Wheel => "bdist_wheel",
        }
    }
}

/// Build the RFC 7578 multipart/form-data body that the legacy PyPI
/// upload endpoint expects, along with the matching `Content-Type`
/// header value. The returned `body` is ready to PUT/POST as-is.
pub fn build_upload_multipart(
    artifact: &UploadArtifact,
    boundary: &str,
) -> (String, Vec<u8>) {
    let content_type = format!("multipart/form-data; boundary={boundary}");

    let mut out: Vec<u8> = Vec::with_capacity(artifact.data.len() + 1024);
    push_text_field(&mut out, boundary, ":action", "file_upload");
    push_text_field(&mut out, boundary, "protocol_version", "1");
    push_text_field(&mut out, boundary, "metadata_version", &artifact.metadata_version);
    push_text_field(&mut out, boundary, "name", &artifact.name);
    push_text_field(&mut out, boundary, "version", &artifact.version);
    push_text_field(&mut out, boundary, "filetype", artifact.file_type.form_value());
    if let Some(tag) = &artifact.python_tag {
        push_text_field(&mut out, boundary, "pyversion", tag);
    } else {
        // PyPI accepts an empty pyversion for sdists.
        push_text_field(&mut out, boundary, "pyversion", "");
    }
    if let Some(s) = &artifact.summary {
        push_text_field(&mut out, boundary, "summary", s);
    }
    push_text_field(&mut out, boundary, "sha256_digest", &artifact.sha256_hex);

    // File part.
    out.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    out.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"content\"; filename=\"{}\"\r\n",
            artifact.filename
        )
        .as_bytes(),
    );
    out.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    out.extend_from_slice(&artifact.data);
    out.extend_from_slice(b"\r\n");

    out.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (content_type, out)
}

fn push_text_field(out: &mut Vec<u8>, boundary: &str, name: &str, value: &str) {
    out.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    out.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
    );
    out.extend_from_slice(value.as_bytes());
    out.extend_from_slice(b"\r\n");
}

/// Generate a fresh RFC 7578 boundary string. Stable-format,
/// random-suffix; safe for binary file bodies because the boundary
/// alphabet excludes printable ASCII bytes likely to appear in
/// wheel zip archives.
pub fn fresh_boundary() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("----mambaUpload{nanos:032x}")
}

/// Default `.pypirc` location (`~/.pypirc`).
pub fn default_pypirc_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .filter(|h| !h.is_empty())
        .map(|h| PathBuf::from(h).join(".pypirc"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn art(name: &str, version: &str, data: &[u8], kind: ArtifactKind) -> UploadArtifact {
        UploadArtifact {
            name: name.into(),
            version: version.into(),
            metadata_version: "2.1".into(),
            file_type: kind,
            filename: format!("{name}-{version}.{}", if matches!(kind, ArtifactKind::Wheel) { "whl" } else { "tar.gz" }),
            data: data.to_vec(),
            sha256_hex: "deadbeef".into(),
            summary: None,
            python_tag: None,
        }
    }

    #[test]
    fn parse_pypirc_minimal_pypi_section() {
        let src = "\
[pypi]
username = __token__
password = pypi-AgEABC123
";
        let map = parse_pypirc(src);
        let pypi = map.get("pypi").unwrap();
        assert_eq!(pypi.url, DEFAULT_PYPI_URL);
        assert_eq!(pypi.username.as_deref(), Some("__token__"));
        assert_eq!(pypi.password.as_deref(), Some("pypi-AgEABC123"));
    }

    #[test]
    fn parse_pypirc_normalizes_url_with_trailing_slash() {
        let src = "\
[internal]
repository = https://nexus.corp.local/pypi
username = ci
password = secret
";
        let map = parse_pypirc(src);
        let internal = map.get("internal").unwrap();
        assert_eq!(internal.url, "https://nexus.corp.local/pypi/");
    }

    #[test]
    fn parse_pypirc_default_url_when_section_omits_repository() {
        let src = "[testpypi]\nusername = __token__\npassword = x\n";
        let map = parse_pypirc(src);
        assert_eq!(map.get("testpypi").unwrap().url, TEST_PYPI_URL);
    }

    #[test]
    fn parse_pypirc_handles_comments_and_blank_lines() {
        let src = "\
; vendor comment
# another
[pypi]

username = __token__

password = abc
";
        let map = parse_pypirc(src);
        let p = map.get("pypi").unwrap();
        assert_eq!(p.username.as_deref(), Some("__token__"));
        assert_eq!(p.password.as_deref(), Some("abc"));
    }

    #[test]
    fn parse_pypirc_supports_colon_separator() {
        let src = "[pypi]\nusername: __token__\npassword: x\n";
        let map = parse_pypirc(src);
        let p = map.get("pypi").unwrap();
        assert_eq!(p.username.as_deref(), Some("__token__"));
    }

    #[test]
    fn parse_pypirc_picks_up_ca_cert() {
        let src = "[internal]\nrepository = https://x/\nca_cert = /etc/ssl/x.pem\n";
        let map = parse_pypirc(src);
        let r = map.get("internal").unwrap();
        assert_eq!(r.ca_cert.as_ref().unwrap().to_str(), Some("/etc/ssl/x.pem"));
    }

    #[test]
    fn parse_pypirc_drops_distutils_index_list() {
        let src = "\
[distutils]
index-servers =
    pypi
    internal

[pypi]
username = __token__
password = x
";
        let map = parse_pypirc(src);
        assert!(!map.contains_key("distutils"));
        assert!(map.contains_key("pypi"));
    }

    #[test]
    fn parse_pypirc_empty_input_is_empty_map() {
        assert!(parse_pypirc("").is_empty());
    }

    #[test]
    fn resolve_repository_explicit_url_wins() {
        let inputs = PublishInputs {
            publish_url: Some("https://upload.example.com/".into()),
            ..Default::default()
        };
        let pypirc = BTreeMap::new();
        let r = resolve_repository(&inputs, &pypirc);
        assert_eq!(r.url, "https://upload.example.com/");
    }

    #[test]
    fn resolve_repository_falls_back_to_pypi_default() {
        let r = resolve_repository(&PublishInputs::default(), &BTreeMap::new());
        assert_eq!(r.url, DEFAULT_PYPI_URL);
    }

    #[test]
    fn resolve_repository_resolves_named_section() {
        let mut pypirc = BTreeMap::new();
        let mut repo = PypiRepository::pypi();
        repo.username = Some("user1".into());
        repo.password = Some("hunter2".into());
        pypirc.insert("pypi".into(), repo);
        let r = resolve_repository(&PublishInputs::default(), &pypirc);
        assert_eq!(r.username.as_deref(), Some("user1"));
        assert_eq!(r.password.as_deref(), Some("hunter2"));
    }

    #[test]
    fn resolve_repository_cli_overrides_credentials_but_keeps_url() {
        let mut pypirc = BTreeMap::new();
        let mut repo = PypiRepository::pypi();
        repo.username = Some("file-user".into());
        repo.password = Some("file-pw".into());
        pypirc.insert("pypi".into(), repo);
        let inputs = PublishInputs {
            username: Some("cli-user".into()),
            password: Some("cli-pw".into()),
            ..Default::default()
        };
        let r = resolve_repository(&inputs, &pypirc);
        assert_eq!(r.username.as_deref(), Some("cli-user"));
        assert_eq!(r.password.as_deref(), Some("cli-pw"));
        assert_eq!(r.url, DEFAULT_PYPI_URL); // url still from .pypirc default
    }

    #[test]
    fn resolve_repository_unknown_named_falls_back_to_testpypi_default() {
        let inputs = PublishInputs {
            repository: Some("testpypi".into()),
            ..Default::default()
        };
        let r = resolve_repository(&inputs, &BTreeMap::new());
        assert_eq!(r.url, TEST_PYPI_URL);
    }

    #[test]
    fn build_upload_multipart_emits_required_fields() {
        let a = art("requests", "2.31.0", b"WHEEL", ArtifactKind::Wheel);
        let (ct, body) = build_upload_multipart(&a, "BOUNDARY");
        let s = String::from_utf8_lossy(&body);
        assert_eq!(ct, "multipart/form-data; boundary=BOUNDARY");
        assert!(s.contains("name=\":action\""));
        assert!(s.contains("file_upload"));
        assert!(s.contains("name=\"protocol_version\""));
        assert!(s.contains("name=\"name\""));
        assert!(s.contains("name=\"version\""));
        assert!(s.contains("name=\"metadata_version\""));
        assert!(s.contains("name=\"filetype\""));
        assert!(s.contains("bdist_wheel"));
        assert!(s.contains("name=\"sha256_digest\""));
        assert!(s.contains("deadbeef"));
        assert!(s.contains("name=\"content\""));
        assert!(s.contains("requests-2.31.0.whl"));
        // Final boundary marker present.
        assert!(s.contains("--BOUNDARY--"));
    }

    #[test]
    fn build_upload_multipart_includes_payload_bytes() {
        let a = art("x", "1", b"abc123\x00\xff", ArtifactKind::Sdist);
        let (_ct, body) = build_upload_multipart(&a, "B");
        // The raw bytes appear verbatim after the file part header.
        assert!(body.windows(8).any(|w| w == b"abc123\x00\xff"));
    }

    #[test]
    fn build_upload_multipart_sdist_uses_correct_filetype() {
        let a = art("x", "1", b"sd", ArtifactKind::Sdist);
        let (_ct, body) = build_upload_multipart(&a, "B");
        let s = String::from_utf8_lossy(&body);
        assert!(s.contains("\r\n\r\nsdist\r\n"));
    }

    #[test]
    fn build_upload_multipart_includes_python_tag_when_set() {
        let mut a = art("x", "1", b"d", ArtifactKind::Wheel);
        a.python_tag = Some("cp312".into());
        let (_ct, body) = build_upload_multipart(&a, "B");
        let s = String::from_utf8_lossy(&body);
        assert!(s.contains("\r\n\r\ncp312\r\n"));
    }

    #[test]
    fn build_upload_multipart_includes_summary_when_set() {
        let mut a = art("x", "1", b"d", ArtifactKind::Wheel);
        a.summary = Some("A useful tool".into());
        let (_ct, body) = build_upload_multipart(&a, "B");
        let s = String::from_utf8_lossy(&body);
        assert!(s.contains("name=\"summary\""));
        assert!(s.contains("A useful tool"));
    }

    #[test]
    fn fresh_boundary_is_stable_format() {
        let b = fresh_boundary();
        assert!(b.starts_with("----mambaUpload"));
        // Long enough to be RFC-7578 unique.
        assert!(b.len() > 30);
    }

    #[test]
    fn default_pypirc_path_uses_home_dir() {
        // SAFETY: single-threaded test, restores HOME after.
        let prev = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", "/h"); }
        assert_eq!(default_pypirc_path().unwrap(), PathBuf::from("/h/.pypirc"));
        match prev {
            Some(v) => unsafe { std::env::set_var("HOME", v) },
            None => unsafe { std::env::remove_var("HOME") },
        }
    }

    #[test]
    fn pypirc_repository_alias_supports_repository_url_key() {
        let src = "[internal]\nrepository-url = https://x.example/\nusername = u\npassword = p\n";
        let map = parse_pypirc(src);
        assert_eq!(map.get("internal").unwrap().url, "https://x.example/");
    }
}
