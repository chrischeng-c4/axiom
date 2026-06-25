// `mamba package` / `mamba publish` - local package artifact build and
// publish-request validation.

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use flate2::read::GzDecoder;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};

use crate::pkgmanage::pkgmgr::extras_spec::normalize_extra;
use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use crate::pkgmanage::pkgmgr::pep621::{License, ProjectTable, Readme};
use crate::pkgmanage::pkgmgr::publish::{
    ArtifactKind, PublishInputs, UploadArtifact, build_upload_multipart, default_pypirc_path,
    parse_pypirc, resolve_repository,
};
use crate::pkgmanage::pkgmgr::requirement_string::Requirement;
use crate::pkgmanage::pkgmgr::sdist_build::SdistBuilder;
use crate::pkgmanage::pkgmgr::wheel_build::{
    CoreMetadata, WheelBuilder, WheelMetadata, compose_filename, parse_core_metadata,
};
use crate::pkgmanage::pkgmgr::wheel_filename::parse_wheel_filename;

const PYPROJECT_FILE: &str = "pyproject.toml";
const DEFAULT_OUT_DIR: &str = "dist";
const DRY_RUN_BOUNDARY: &str = "----mambaPublishDryRunBoundary";

pub fn cmd_package(sub: &ArgMatches) -> Result<()> {
    match sub.subcommand() {
        Some(("build", build)) => cmd_package_build(build),
        Some(("publish", publish)) => cmd_publish(publish),
        _ => bail!("expected subcommand: mamba package build|publish"),
    }
}

pub fn cmd_package_build(sub: &ArgMatches) -> Result<()> {
    let project_dir = sub
        .get_one::<String>("project")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir().context("read current directory")?);
    let out_dir = sub
        .get_one::<String>("out-dir")
        .map(PathBuf::from)
        .unwrap_or_else(|| project_dir.join(DEFAULT_OUT_DIR));
    let wants_wheel = sub.get_flag("wheel") || !sub.get_flag("sdist");
    let wants_sdist = sub.get_flag("sdist") || !sub.get_flag("wheel");
    let json_out = sub.get_flag("json");

    let project = read_project_table(&project_dir)?;
    let version = project
        .version
        .clone()
        .context("dynamic [project].version is not supported by mamba package build yet")?;
    let core_meta = core_metadata_from_project(&project_dir, &project, &version)?;
    let sources = collect_source_files(&project_dir, &project)?;

    fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;
    let mut artifacts = Vec::new();

    if wants_wheel {
        let mut wheel_meta = WheelMetadata::new("mamba package build");
        wheel_meta.tags.push("py3-none-any".to_string());
        let filename = compose_filename(&project.raw_name, &version, "py3", "none", "any");
        let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta.clone());
        for src in &sources {
            builder.add_file(src.wheel_path.clone(), src.data.clone());
        }
        let path = builder.build_to_dir(&out_dir)?;
        artifacts.push(path);
    }

    if wants_sdist {
        let mut builder = SdistBuilder::new(&project.raw_name, &version).metadata(core_meta);
        let pyproject = fs::read(project_dir.join(PYPROJECT_FILE))
            .with_context(|| format!("read {}", project_dir.join(PYPROJECT_FILE).display()))?;
        builder = builder.add_file(PYPROJECT_FILE, pyproject);
        for src in &sources {
            builder = builder.add_file(src.sdist_path.clone(), src.data.clone());
        }
        let path = builder.build_to_dir(&out_dir)?;
        artifacts.push(path);
    }

    if json_out {
        let payload = json!({
            "artifacts": artifacts.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        for artifact in artifacts {
            println!("built {}", artifact.display());
        }
    }
    Ok(())
}

pub fn cmd_publish(sub: &ArgMatches) -> Result<()> {
    if !sub.get_flag("dry-run") {
        bail!("network upload is not implemented yet; use --dry-run to validate publish payloads");
    }

    let pypirc = load_pypirc(sub)?;
    let inputs = publish_inputs(sub);
    let repo = resolve_repository(&inputs, &pypirc);
    if repo.url.trim().is_empty() {
        bail!("publish repository URL is empty; pass --publish-url or configure .pypirc");
    }

    let artifacts = artifact_paths(sub)?;
    if artifacts.is_empty() {
        bail!("no artifacts found; pass files or build into dist/ first");
    }

    let mut summaries = Vec::new();
    for path in artifacts {
        let upload = read_upload_artifact(&path)?;
        let (content_type, body) = build_upload_multipart(&upload, DRY_RUN_BOUNDARY);
        summaries.push(PublishDryRun {
            path,
            filename: upload.filename,
            name: upload.name,
            version: upload.version,
            file_type: upload.file_type,
            sha256_hex: upload.sha256_hex,
            content_type,
            body_len: body.len(),
        });
    }

    if sub.get_flag("json") {
        let payload = json!({
            "repository_url": repo.url,
            "username": repo.username,
            "password_present": repo.password.is_some(),
            "ca_cert": repo.ca_cert.map(|p| p.to_string_lossy().to_string()),
            "artifacts": summaries.iter().map(|s| json!({
                "path": s.path.to_string_lossy(),
                "filename": &s.filename,
                "name": &s.name,
                "version": &s.version,
                "file_type": s.file_type.label(),
                "sha256": &s.sha256_hex,
                "content_type": &s.content_type,
                "body_bytes": s.body_len,
            })).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("publish dry-run target {}", repo.url);
        println!(
            "credentials username={} password_present={}",
            repo.username.as_deref().unwrap_or("<none>"),
            repo.password.is_some()
        );
        for s in summaries {
            println!(
                "artifact {} {} {} sha256:{} body_bytes:{}",
                s.filename, s.name, s.version, s.sha256_hex, s.body_len
            );
        }
    }
    Ok(())
}

struct SourceFile {
    wheel_path: String,
    sdist_path: String,
    data: Vec<u8>,
}

struct PublishDryRun {
    path: PathBuf,
    filename: String,
    name: String,
    version: String,
    file_type: ArtifactKind,
    sha256_hex: String,
    content_type: String,
    body_len: usize,
}

trait ArtifactKindLabel {
    fn label(self) -> &'static str;
}

impl ArtifactKindLabel for ArtifactKind {
    fn label(self) -> &'static str {
        match self {
            ArtifactKind::Sdist => "sdist",
            ArtifactKind::Wheel => "wheel",
        }
    }
}

fn read_project_table(project_dir: &Path) -> Result<ProjectTable> {
    let pyproject = project_dir.join(PYPROJECT_FILE);
    let src =
        fs::read_to_string(&pyproject).with_context(|| format!("read {}", pyproject.display()))?;
    ProjectTable::parse(&src).map_err(anyhow::Error::from)
}

fn core_metadata_from_project(
    project_dir: &Path,
    project: &ProjectTable,
    version: &str,
) -> Result<CoreMetadata> {
    let mut meta = CoreMetadata::new(&project.raw_name, version);
    meta.summary = project.description.clone();
    meta.requires_python = project.requires_python.clone();
    meta.requires_dist = project
        .dependencies
        .iter()
        .map(render_requirement)
        .collect();
    for (extra_raw, deps) in &project.optional_dependencies {
        let extra = normalize_extra(extra_raw);
        meta.provides_extras.push(extra.clone());
        for dep in deps {
            meta.requires_dist
                .push(render_requirement_for_extra(dep, &extra));
        }
    }
    meta.provides_extras.sort();
    meta.provides_extras.dedup();
    meta.classifiers = project.classifiers.clone();
    meta.keywords = project.keywords.clone();
    if let Some(rp) = &project.readme {
        apply_readme(project_dir, rp, &mut meta)?;
    }
    if let Some(license) = &project.license {
        meta.license = Some(match license {
            License::File(path) => format!("file: {path}"),
            License::Text(text) | License::SpdxExpression(text) => text.clone(),
        });
    }
    if let Some(author) = project.authors.first() {
        meta.author = author.name.clone();
        meta.author_email = author.email.clone();
    }
    Ok(meta)
}

fn apply_readme(project_dir: &Path, readme: &Readme, meta: &mut CoreMetadata) -> Result<()> {
    match readme {
        Readme::Inline { text, content_type } => {
            meta.description = Some(text.clone());
            meta.description_content_type = Some(content_type.clone());
        }
        Readme::File(path) => {
            let full = project_dir.join(path);
            let body =
                fs::read_to_string(&full).with_context(|| format!("read {}", full.display()))?;
            meta.description = Some(body);
            meta.description_content_type = Some(readme_content_type(path).to_string());
        }
    }
    Ok(())
}

fn readme_content_type(path: &str) -> &'static str {
    match Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "md" => "text/markdown",
        "rst" => "text/x-rst",
        _ => "text/plain",
    }
}

fn render_requirement(req: &Requirement) -> String {
    let mut out = render_requirement_head(req);
    if let Some(marker) = &req.marker {
        out.push_str("; ");
        out.push_str(marker);
    }
    out
}

fn render_requirement_for_extra(req: &Requirement, extra: &str) -> String {
    let mut out = render_requirement_head(req);
    out.push_str("; ");
    match &req.marker {
        Some(marker) => out.push_str(&format!("({marker}) and extra == \"{extra}\"")),
        None => out.push_str(&format!("extra == \"{extra}\"")),
    }
    out
}

fn render_requirement_head(req: &Requirement) -> String {
    let mut out = req.raw_name.clone();
    if !req.extras.is_empty() {
        out.push_str(&req.extras.render());
    }
    if let Some(url) = &req.url {
        out.push_str(" @ ");
        out.push_str(url);
    } else if let Some(specifier) = &req.specifier {
        out.push_str(specifier);
    }
    out
}

fn collect_source_files(project_dir: &Path, project: &ProjectTable) -> Result<Vec<SourceFile>> {
    let src_dir = project_dir.join("src");
    let mut out = Vec::new();
    if src_dir.is_dir() {
        collect_tree(project_dir, &src_dir, &src_dir, true, &mut out)?;
    } else {
        let module_name = pep503_normalize(&project.raw_name).replace('-', "_");
        let package_dir = project_dir.join(&module_name);
        let module_file = project_dir.join(format!("{module_name}.py"));
        if package_dir.is_dir() {
            collect_tree(project_dir, &package_dir, project_dir, false, &mut out)?;
        }
        if module_file.is_file() {
            push_source_file(project_dir, &module_file, project_dir, false, &mut out)?;
        }
    }
    out.sort_by(|a, b| a.sdist_path.cmp(&b.sdist_path));
    if out.is_empty() {
        bail!("no package sources found under src/ or a top-level module matching [project].name");
    }
    Ok(out)
}

fn collect_tree(
    project_dir: &Path,
    dir: &Path,
    wheel_base: &Path,
    src_layout: bool,
    out: &mut Vec<SourceFile>,
) -> Result<()> {
    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("read dir {}", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("read entries {}", dir.display()))?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if skip_source_entry(&name, &path) {
            continue;
        }
        let ty = entry
            .file_type()
            .with_context(|| format!("stat {}", path.display()))?;
        if ty.is_dir() {
            collect_tree(project_dir, &path, wheel_base, src_layout, out)?;
        } else if ty.is_file() {
            push_source_file(project_dir, &path, wheel_base, src_layout, out)?;
        }
    }
    Ok(())
}

fn push_source_file(
    project_dir: &Path,
    path: &Path,
    wheel_base: &Path,
    src_layout: bool,
    out: &mut Vec<SourceFile>,
) -> Result<()> {
    let wheel_path = rel_slash(path, wheel_base)?;
    let sdist_path = if src_layout {
        format!("src/{wheel_path}")
    } else {
        rel_slash(path, project_dir)?
    };
    let data = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    out.push(SourceFile {
        wheel_path,
        sdist_path,
        data,
    });
    Ok(())
}

fn skip_source_entry(name: &str, path: &Path) -> bool {
    if name == "__pycache__"
        || name == ".DS_Store"
        || name == "dist"
        || name == "build"
        || name == ".venv"
        || name == ".git"
        || name == "target"
        || name.ends_with(".dist-info")
        || name.ends_with(".egg-info")
    {
        return true;
    }
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("pyc") | Some("pyo")
    )
}

fn rel_slash(path: &Path, base: &Path) -> Result<String> {
    let rel = path
        .strip_prefix(base)
        .with_context(|| format!("{} is not under {}", path.display(), base.display()))?;
    let mut parts = Vec::new();
    for component in rel.components() {
        match component {
            Component::Normal(s) => parts.push(
                s.to_str()
                    .with_context(|| format!("non-UTF-8 path {}", path.display()))?
                    .to_string(),
            ),
            _ => bail!("unsupported archive path component in {}", path.display()),
        }
    }
    if parts.is_empty() {
        bail!("empty archive path for {}", path.display());
    }
    Ok(parts.join("/"))
}

fn load_pypirc(
    sub: &ArgMatches,
) -> Result<std::collections::BTreeMap<String, crate::pkgmanage::pkgmgr::publish::PypiRepository>> {
    let path = sub
        .get_one::<String>("pypirc")
        .map(PathBuf::from)
        .or_else(default_pypirc_path);
    let Some(path) = path else {
        return Ok(Default::default());
    };
    if !path.exists() {
        return Ok(Default::default());
    }
    let src = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(parse_pypirc(&src))
}

fn publish_inputs(sub: &ArgMatches) -> PublishInputs {
    PublishInputs {
        publish_url: cli_or_env(
            sub,
            "publish-url",
            &["UV_PUBLISH_URL", "TWINE_REPOSITORY_URL"],
        ),
        repository: cli_or_env(
            sub,
            "repository",
            &["UV_PUBLISH_REPOSITORY", "TWINE_REPOSITORY"],
        ),
        username: cli_or_env(sub, "username", &["UV_PUBLISH_USERNAME", "TWINE_USERNAME"]),
        password: cli_or_env(sub, "password", &["UV_PUBLISH_PASSWORD", "TWINE_PASSWORD"]),
    }
}

fn cli_or_env(sub: &ArgMatches, arg: &str, envs: &[&str]) -> Option<String> {
    sub.get_one::<String>(arg)
        .cloned()
        .or_else(|| envs.iter().find_map(|key| std::env::var(key).ok()))
        .filter(|s| !s.is_empty())
}

fn artifact_paths(sub: &ArgMatches) -> Result<Vec<PathBuf>> {
    if let Some(paths) = sub.get_many::<String>("artifact") {
        return Ok(paths.map(PathBuf::from).collect());
    }
    let dist = PathBuf::from(DEFAULT_OUT_DIR);
    if !dist.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = fs::read_dir(&dist)
        .with_context(|| format!("read dir {}", dist.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|s| s.to_str())
                .map(is_artifact_filename)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    out.sort();
    Ok(out)
}

fn is_artifact_filename(name: &str) -> bool {
    name.ends_with(".whl") || name.ends_with(".tar.gz")
}

fn read_upload_artifact(path: &Path) -> Result<UploadArtifact> {
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .with_context(|| format!("artifact path has no UTF-8 filename: {}", path.display()))?
        .to_string();
    let data = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let sha256_hex = sha256_hex(&data);
    if filename.ends_with(".whl") {
        upload_from_wheel(path, filename, data, sha256_hex)
    } else if filename.ends_with(".tar.gz") {
        upload_from_sdist(path, filename, data, sha256_hex)
    } else {
        bail!("unsupported publish artifact `{}`", path.display())
    }
}

fn upload_from_wheel(
    path: &Path,
    filename: String,
    data: Vec<u8>,
    sha256_hex: String,
) -> Result<UploadArtifact> {
    let wheel_name = parse_wheel_filename(&filename).map_err(anyhow::Error::from)?;
    let meta = wheel_core_metadata(path, &data)?;
    Ok(UploadArtifact {
        name: pep503_normalize(&meta.name),
        version: meta.version,
        metadata_version: meta.metadata_version,
        file_type: ArtifactKind::Wheel,
        filename,
        data,
        sha256_hex,
        summary: meta.summary,
        python_tag: Some(wheel_name.python_tag),
    })
}

fn upload_from_sdist(
    path: &Path,
    filename: String,
    data: Vec<u8>,
    sha256_hex: String,
) -> Result<UploadArtifact> {
    let meta = sdist_core_metadata(path, &data)?;
    Ok(UploadArtifact {
        name: pep503_normalize(&meta.name),
        version: meta.version,
        metadata_version: meta.metadata_version,
        file_type: ArtifactKind::Sdist,
        filename,
        data,
        sha256_hex,
        summary: meta.summary,
        python_tag: None,
    })
}

fn wheel_core_metadata(path: &Path, data: &[u8]) -> Result<CoreMetadata> {
    let mut zip = zip::ZipArchive::new(Cursor::new(data))
        .with_context(|| format!("open wheel {}", path.display()))?;
    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .with_context(|| format!("read wheel entry #{i} in {}", path.display()))?;
        let name = file.name().to_string();
        if name.ends_with(".dist-info/METADATA") {
            let mut body = String::new();
            file.read_to_string(&mut body)
                .with_context(|| format!("read {name} in {}", path.display()))?;
            return parse_core_metadata(&body).map_err(anyhow::Error::from);
        }
    }
    bail!("wheel {} is missing .dist-info/METADATA", path.display())
}

fn sdist_core_metadata(path: &Path, data: &[u8]) -> Result<CoreMetadata> {
    let gz = GzDecoder::new(Cursor::new(data));
    let mut archive = tar::Archive::new(gz);
    let entries = archive
        .entries()
        .with_context(|| format!("read sdist {}", path.display()))?;
    for entry in entries {
        let mut entry = entry.with_context(|| format!("read sdist entry {}", path.display()))?;
        let entry_path = entry
            .path()
            .with_context(|| format!("read sdist entry path {}", path.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        if entry_path == "PKG-INFO" || entry_path.ends_with("/PKG-INFO") {
            let mut body = String::new();
            entry
                .read_to_string(&mut body)
                .with_context(|| format!("read {entry_path} in {}", path.display()))?;
            return parse_core_metadata(&body).map_err(anyhow::Error::from);
        }
    }
    bail!("sdist {} is missing PKG-INFO", path.display())
}

fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    let mut out = String::with_capacity(digest.len() * 2);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}
