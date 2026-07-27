// HANDWRITE-BEGIN gap="missing-generator:python-artifact-protocol" tracker="#2290" reason="The CPython protocol runner owns process isolation, digest validation, and evidence-path safety until the Python artifact generator can emit this shared adapter."
//! CPython-first project protocol shared by future EC and TD adapters.
//!
//! The runner discovers configuration only from `pyproject.toml`; it never
//! imports project modules while deciding how to run them. The project process
//! receives authoritative digests through environment variables, and AW
//! validates its one JSON result envelope before a later lifecycle layer can
//! consume it.
//!
//! @spec apps/agentic-workflow/tech-design/core/logic/python-artifact-project-protocol.md#logic

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use walkdir::WalkDir;

pub const PYTHON_ARTIFACT_PROTOCOL: &str = "aw.python-artifact.v1";
pub const PYTHON_ARTIFACT_RESULT_SCHEMA: &str = "aw.python-artifact.result.v1";

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const CPYTHON_IMPLEMENTATION_PROBE: &str =
    "import platform; print(platform.python_implementation())";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonArtifactProject {
    root: PathBuf,
    entrypoint: PathBuf,
    evidence_dir: PathBuf,
    source_digest: String,
    dependency_lock_digest: String,
    input_files: Vec<PythonArtifactInput>,
}

impl PythonArtifactProject {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn source_digest(&self) -> &str {
        &self.source_digest
    }

    pub fn dependency_lock_digest(&self) -> &str {
        &self.dependency_lock_digest
    }

    /// Normalized, content-addressed inputs included in this artifact
    /// declaration. Callers that need a durable lock can retain the exact
    /// source and dependency files rather than trusting only aggregate
    /// digests.
    pub fn input_files(&self) -> &[PythonArtifactInput] {
        &self.input_files
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonArtifactInput {
    pub path: PathBuf,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonArtifactRunOptions {
    pub python_executable: PathBuf,
    pub timeout: Duration,
}

impl Default for PythonArtifactRunOptions {
    fn default() -> Self {
        Self {
            python_executable: PathBuf::from("python3"),
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PythonArtifactStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonArtifactRun {
    pub status: PythonArtifactStatus,
    pub exit_code: i32,
    pub evidence_paths: Vec<PathBuf>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Deserialize)]
struct PyprojectDocument {
    tool: PyprojectTool,
}

#[derive(Debug, Deserialize)]
struct PyprojectTool {
    aw: PyprojectAw,
}

#[derive(Debug, Deserialize)]
struct PyprojectAw {
    #[serde(rename = "python-artifact")]
    python_artifact: PythonArtifactConfig,
}

#[derive(Debug, Deserialize)]
struct PythonArtifactConfig {
    protocol: String,
    entrypoint: String,
    source_roots: Vec<String>,
    dependency_files: Vec<String>,
    evidence_dir: String,
}

#[derive(Debug, Deserialize)]
struct PythonArtifactEnvelope {
    schema_version: String,
    status: PythonArtifactStatus,
    source_digest: String,
    dependency_lock_digest: String,
    evidence: Vec<String>,
}

/// Discover a `python-v1` artifact project by reading its declarative
/// `pyproject.toml`. Discovery deliberately does not execute Python.
pub fn discover_python_artifact_project(project_root: &Path) -> Result<PythonArtifactProject> {
    let root = project_root.canonicalize().with_context(|| {
        format!(
            "canonicalize Python artifact root {}",
            project_root.display()
        )
    })?;
    let pyproject = root.join("pyproject.toml");
    let pyproject_content = fs::read_to_string(&pyproject)
        .with_context(|| format!("read Python artifact configuration {}", pyproject.display()))?;
    let document: PyprojectDocument = toml::from_str(&pyproject_content).with_context(|| {
        format!(
            "parse Python artifact configuration {}",
            pyproject.display()
        )
    })?;
    let config = document.tool.aw.python_artifact;

    if config.protocol != PYTHON_ARTIFACT_PROTOCOL {
        bail!(
            "unsupported Python artifact protocol `{}`; expected `{PYTHON_ARTIFACT_PROTOCOL}`",
            config.protocol
        );
    }
    if config.source_roots.is_empty() {
        bail!("Python artifact protocol requires at least one source_roots entry");
    }
    if config.dependency_files.is_empty() {
        bail!("Python artifact protocol requires dependency_files including pyproject.toml");
    }
    if !config
        .dependency_files
        .iter()
        .any(|path| Path::new(path) == Path::new("pyproject.toml"))
    {
        bail!("Python artifact dependency_files must include pyproject.toml");
    }

    let entrypoint = resolve_project_path(&root, &config.entrypoint, "entrypoint")?;
    require_regular_file(&entrypoint, "entrypoint")?;
    if entrypoint
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("py")
    {
        bail!(
            "Python artifact entrypoint must be a .py file, got {}",
            entrypoint.display()
        );
    }

    let evidence_dir = resolve_project_path(&root, &config.evidence_dir, "evidence_dir")?;
    require_directory(&evidence_dir, "evidence_dir")?;

    let mut source_roots = Vec::with_capacity(config.source_roots.len());
    for source_root in &config.source_roots {
        let source_root = resolve_project_path(&root, source_root, "source_roots")?;
        require_directory(&source_root, "source_roots")?;
        source_roots.push(source_root);
    }
    let source_files = collect_python_sources(&source_roots)?;
    let source_digest = digest_files(&root, &source_files)?;

    let mut dependency_files = BTreeSet::new();
    for dependency_file in &config.dependency_files {
        let dependency_file = resolve_project_path(&root, dependency_file, "dependency_files")?;
        require_regular_file(&dependency_file, "dependency_files")?;
        dependency_files.insert(dependency_file);
    }
    let dependency_lock_digest = digest_files(&root, &dependency_files)?;
    let input_files = normalized_input_files(&source_files, &dependency_files)?;

    Ok(PythonArtifactProject {
        root,
        entrypoint,
        evidence_dir,
        source_digest,
        dependency_lock_digest,
        input_files,
    })
}

/// Run one protocol command and fail closed unless process exit, JSON result,
/// digests, and evidence paths agree.
pub fn run_python_artifact_project(
    project: &PythonArtifactProject,
    command: &str,
    options: &PythonArtifactRunOptions,
) -> Result<PythonArtifactRun> {
    if command.is_empty()
        || !command
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        bail!("Python artifact command must be one ASCII command token");
    }
    if options.timeout.is_zero() {
        bail!("Python artifact timeout must be greater than zero");
    }
    ensure_cpython(options)?;

    let mut process = Command::new(&options.python_executable);
    process
        .arg("-I")
        .arg(&project.entrypoint)
        .arg(command)
        .current_dir(&project.root)
        .env("AW_PYTHON_ARTIFACT_PROTOCOL", PYTHON_ARTIFACT_PROTOCOL)
        .env("AW_PYTHON_ARTIFACT_SOURCE_DIGEST", &project.source_digest)
        .env(
            "AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST",
            &project.dependency_lock_digest,
        )
        .env("AW_PYTHON_ARTIFACT_EVIDENCE_DIR", &project.evidence_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = process.spawn().with_context(|| {
        format!(
            "spawn CPython artifact command `{}` for {}",
            command,
            project.root.display()
        )
    })?;

    let deadline = Instant::now() + options.timeout;
    while child.try_wait()?.is_none() {
        if Instant::now() >= deadline {
            child
                .kill()
                .context("terminate timed out Python artifact command")?;
            let _ = child.wait_with_output();
            bail!(
                "Python artifact command `{command}` timed out after {:?}",
                options.timeout
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = child
        .wait_with_output()
        .context("collect Python artifact command output")?;
    let exit_code = output
        .status
        .code()
        .context("Python artifact command terminated without an exit code")?;
    let stdout =
        String::from_utf8(output.stdout).context("Python artifact stdout was not UTF-8")?;
    let stderr =
        String::from_utf8(output.stderr).context("Python artifact stderr was not UTF-8")?;
    let envelope: PythonArtifactEnvelope = serde_json::from_str(&stdout)
        .context("malformed Python artifact result envelope; stdout must be one JSON document")?;

    if envelope.schema_version != PYTHON_ARTIFACT_RESULT_SCHEMA {
        bail!(
            "unsupported Python artifact result schema `{}`; expected `{PYTHON_ARTIFACT_RESULT_SCHEMA}`",
            envelope.schema_version
        );
    }
    if envelope.source_digest != project.source_digest {
        bail!(
            "Python artifact source digest mismatch: runner reported `{}`, AW computed `{}`",
            envelope.source_digest,
            project.source_digest
        );
    }
    if envelope.dependency_lock_digest != project.dependency_lock_digest {
        bail!(
            "Python artifact dependency lock digest mismatch: runner reported `{}`, AW computed `{}`",
            envelope.dependency_lock_digest,
            project.dependency_lock_digest
        );
    }
    let expected_exit_code = match envelope.status {
        PythonArtifactStatus::Passed => 0,
        PythonArtifactStatus::Failed => 1,
    };
    if exit_code != expected_exit_code {
        bail!(
            "Python artifact status {:?} requires exit code {expected_exit_code}, observed {exit_code}",
            envelope.status
        );
    }

    let evidence_paths = resolve_evidence_paths(project, &envelope.evidence)?;
    Ok(PythonArtifactRun {
        status: envelope.status,
        exit_code,
        evidence_paths,
        stdout,
        stderr,
    })
}

fn ensure_cpython(options: &PythonArtifactRunOptions) -> Result<()> {
    let output = Command::new(&options.python_executable)
        .arg("-I")
        .arg("-c")
        .arg(CPYTHON_IMPLEMENTATION_PROBE)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| {
            format!(
                "run CPython implementation probe via {}",
                options.python_executable.display()
            )
        })?;
    if !output.status.success() {
        bail!(
            "Python artifact runner requires CPython; implementation probe exited {:?}",
            output.status.code()
        );
    }
    let implementation = String::from_utf8(output.stdout)
        .context("CPython implementation probe stdout was not UTF-8")?;
    if implementation.trim() != "CPython" {
        bail!(
            "Python artifact runner requires CPython, observed `{}`",
            implementation.trim()
        );
    }
    Ok(())
}

fn resolve_project_path(root: &Path, declared: &str, field: &str) -> Result<PathBuf> {
    let relative = Path::new(declared);
    if declared.trim().is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        bail!("Python artifact {field} must be a safe project-relative path: `{declared}`");
    }
    let candidate = root.join(relative);
    require_no_symlink_components(root, relative, field)?;
    let canonical = candidate
        .canonicalize()
        .with_context(|| format!("resolve Python artifact {field} `{declared}`"))?;
    if !canonical.starts_with(root) {
        bail!("Python artifact {field} escapes the project root: `{declared}`");
    }
    Ok(canonical)
}

fn require_no_symlink_components(root: &Path, relative: &Path, field: &str) -> Result<()> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component);
        let metadata = fs::symlink_metadata(&current).with_context(|| {
            format!(
                "read Python artifact {field} path component {}",
                current.display()
            )
        })?;
        if metadata.file_type().is_symlink() {
            bail!(
                "Python artifact {field} may not traverse a symlink: {}",
                current.display()
            );
        }
    }
    Ok(())
}

fn require_regular_file(path: &Path, field: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("read Python artifact {field} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!(
            "Python artifact {field} must be a regular file: {}",
            path.display()
        );
    }
    Ok(())
}

fn require_directory(path: &Path, field: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("read Python artifact {field} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!(
            "Python artifact {field} must be a directory: {}",
            path.display()
        );
    }
    Ok(())
}

fn collect_python_sources(source_roots: &[PathBuf]) -> Result<BTreeSet<PathBuf>> {
    let mut files = BTreeSet::new();
    for source_root in source_roots {
        for entry in WalkDir::new(source_root)
            .follow_links(false)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|entry| {
                !entry.file_type().is_dir()
                    || !is_ignored_python_artifact_directory(entry.file_name())
            })
        {
            let entry = entry?;
            if entry.file_type().is_symlink() {
                bail!(
                    "Python artifact source roots may not contain symlinks: {}",
                    entry.path().display()
                );
            }
            if entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|extension| extension.to_str())
                    == Some("py")
            {
                files.insert(entry.path().canonicalize()?);
            }
        }
    }
    if files.is_empty() {
        bail!("Python artifact source_roots contain no Python source files");
    }
    Ok(files)
}

/// @spec #2774
fn is_ignored_python_artifact_directory(name: &std::ffi::OsStr) -> bool {
    matches!(
        name.to_str(),
        Some(
            "__pycache__"
                | ".venv"
                | "venv"
                | ".pytest_cache"
                | ".mypy_cache"
                | ".ruff_cache"
                | ".tox"
                | "build"
                | "dist"
                | ".eggs"
        )
    )
}

fn digest_files(root: &Path, files: &BTreeSet<PathBuf>) -> Result<String> {
    let mut hasher = Sha256::new();
    for path in files {
        let relative = path.strip_prefix(root).with_context(|| {
            format!("digest path {} escapes {}", path.display(), root.display())
        })?;
        let relative = relative
            .to_str()
            .context("Python artifact digest paths must be UTF-8")?;
        let bytes =
            fs::read(path).with_context(|| format!("read digest input {}", path.display()))?;
        hasher.update(relative.as_bytes());
        hasher.update([0]);
        hasher.update((bytes.len() as u64).to_be_bytes());
        hasher.update([0]);
        hasher.update(bytes);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn normalized_input_files(
    source_files: &BTreeSet<PathBuf>,
    dependency_files: &BTreeSet<PathBuf>,
) -> Result<Vec<PythonArtifactInput>> {
    let mut paths = source_files.clone();
    paths.extend(dependency_files.iter().cloned());
    paths
        .into_iter()
        .map(|path| {
            let bytes = fs::read(&path)
                .with_context(|| format!("read Python artifact input {}", path.display()))?;
            Ok(PythonArtifactInput {
                path,
                digest: digest_content(&bytes),
            })
        })
        .collect()
}

fn digest_content(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

fn resolve_evidence_paths(
    project: &PythonArtifactProject,
    declared: &[String],
) -> Result<Vec<PathBuf>> {
    if declared.is_empty() {
        bail!("Python artifact result envelope must name at least one evidence path");
    }
    let mut paths = BTreeSet::new();
    for evidence in declared {
        let path = resolve_project_path(&project.root, evidence, "result evidence")?;
        if !path.starts_with(&project.evidence_dir) {
            bail!(
                "Python artifact evidence must live under {}: `{evidence}`",
                project.evidence_dir.display()
            );
        }
        require_regular_file(&path, "result evidence")?;
        if fs::metadata(&path)
            .with_context(|| format!("read Python artifact result evidence {}", path.display()))?
            .len()
            == 0
        {
            bail!(
                "Python artifact result evidence must be non-empty: {}",
                path.display()
            );
        }
        if !paths.insert(path) {
            bail!("Python artifact result envelope names duplicate evidence `{evidence}`");
        }
    }
    Ok(paths.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::{collect_python_sources, digest_files};
    use std::fs;

    /// @spec #2774
    #[test]
    fn python_artifact_source_digest_ignores_cache_and_build_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let source_root = root.join("src");
        fs::create_dir_all(&source_root).unwrap();
        fs::write(source_root.join("contract.py"), b"CONTRACT = True\n").unwrap();

        let baseline_sources = collect_python_sources(std::slice::from_ref(&source_root)).unwrap();
        let baseline_digest = digest_files(&root, &baseline_sources).unwrap();

        let artifacts = [
            (
                "__pycache__/contract.cpython-312.pyc",
                b"\0\xff\xfe".as_slice(),
            ),
            ("build/generated.py", b"\0\xff\xfe".as_slice()),
            ("dist/generated.py", b"\0\xff\xfe".as_slice()),
            (".eggs/generated.py", b"\0\xff\xfe".as_slice()),
        ];
        for (relative, bytes) in artifacts {
            let path = source_root.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }

        let after_sources = collect_python_sources(std::slice::from_ref(&source_root)).unwrap();
        let after_digest = digest_files(&root, &after_sources).unwrap();

        assert_eq!(after_sources, baseline_sources);
        assert_eq!(after_digest, baseline_digest);
        for (relative, bytes) in artifacts {
            assert_eq!(fs::read(source_root.join(relative)).unwrap(), bytes);
        }
    }
}
// HANDWRITE-END
