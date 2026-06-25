// `uv pip compile`-style offline requirements compiler.
//
// This local-first layer consumes requirements roots plus a frozen wheel index,
// reads wheel dist-info/METADATA for dependency edges, and renders the resolved
// closure through the same requirements/pylock exporters used by `mamba export`.

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::lockfile::{Lockfile, Package, SourceRef, SourceRefKind};
use crate::pkgmanage::pkgmgr::name_normalize::pep503_normalize;
use crate::pkgmanage::pkgmgr::pip_check::version_satisfies_all;
use crate::pkgmanage::pkgmgr::pip_compile::CompileFormat::{PylockToml, RequirementsTxt};
use crate::pkgmanage::pkgmgr::pip_install::{
    find_index_wheel, is_extra_marker, read_wheel_metadata,
};
use crate::pkgmanage::pkgmgr::pylock_export::{PylockOptions, render_pylock_toml};
use crate::pkgmanage::pkgmgr::requirements_export::{ExportOptions, export_requirements_txt};
use crate::pkgmanage::pkgmgr::requirements_loader::{
    LoadedRequirements, load_requirements_file, load_requirements_text,
};
use crate::pkgmanage::pkgmgr::requirements_parse::{
    PackageRequirement, RequirementLine, parse_one_line,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileFormat {
    RequirementsTxt,
    PylockToml,
}

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub index: PathBuf,
    pub output_file: Option<PathBuf>,
    pub format: CompileFormat,
    pub include_header: bool,
    pub generate_hashes: bool,
    pub annotate: bool,
    pub no_deps: bool,
    pub no_emit_packages: Vec<String>,
}

#[derive(Default)]
struct CompileState {
    packages: BTreeMap<String, Package>,
}

pub fn compile_sources(src_files: &[PathBuf], opts: &CompileOptions) -> Result<String> {
    if src_files.is_empty() {
        bail!("pip compile requires at least one source file");
    }

    let loaded = load_all_sources(src_files)?;
    reject_unsupported_loaded_requirements(&loaded)?;
    if loaded.primary.is_empty() {
        bail!("pip compile found no package requirements");
    }

    let constraints = constraints_by_name(&loaded.constraints)?;
    let mut state = CompileState::default();
    for req in &loaded.primary {
        resolve_requirement(req, &constraints, opts, &mut state)?;
    }

    let packages = state.packages.into_values().collect::<Vec<_>>();
    let lockfile = Lockfile {
        format_version: 1,
        input_hash: input_hash(&loaded),
        packages,
    };

    Ok(match opts.format {
        RequirementsTxt => export_requirements_txt(
            &lockfile,
            &ExportOptions {
                include_hashes: opts.generate_hashes,
                include_header: opts.include_header,
                exclude_packages: opts.no_emit_packages.clone(),
                global_marker: None,
                annotate: opts.annotate,
            },
        ),
        PylockToml => render_pylock_toml(&lockfile, &PylockOptions::default()),
    })
}

pub fn write_compile_output(output_file: Option<&PathBuf>, body: &str) -> Result<()> {
    match output_file {
        Some(path) => {
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            atomic_write(path, body.as_bytes())
        }
        None => {
            print!("{body}");
            Ok(())
        }
    }
}

pub fn parse_compile_format(
    raw: Option<&String>,
    output: Option<&PathBuf>,
) -> Result<CompileFormat> {
    if let Some(raw) = raw {
        return match raw.as_str() {
            "requirements.txt" | "requirements-txt" | "requirements" => Ok(RequirementsTxt),
            "pylock.toml" | "pylock" => Ok(PylockToml),
            other => bail!("unsupported pip compile format `{other}`"),
        };
    }

    if let Some(path) = output {
        if path.file_name().and_then(|s| s.to_str()) == Some("pylock.toml") {
            return Ok(PylockToml);
        }
    }
    Ok(RequirementsTxt)
}

fn load_all_sources(src_files: &[PathBuf]) -> Result<LoadedRequirements> {
    let mut out = LoadedRequirements::default();
    let cwd = std::env::current_dir().context("read current directory")?;
    for src in src_files {
        let loaded = if src.as_os_str() == "-" {
            let mut body = String::new();
            std::io::stdin()
                .read_to_string(&mut body)
                .context("read requirements from stdin")?;
            load_requirements_text(&body, &cwd)?
        } else {
            load_requirements_file(src)?
        };
        out.primary.extend(loaded.primary);
        out.constraints.extend(loaded.constraints);
        out.editables.extend(loaded.editables);
        out.index_flags.extend(loaded.index_flags);
        out.unknown.extend(loaded.unknown);
        out.visited.extend(loaded.visited);
    }
    Ok(out)
}

fn reject_unsupported_loaded_requirements(loaded: &LoadedRequirements) -> Result<()> {
    if !loaded.editables.is_empty() {
        bail!("mamba pip compile does not compile editable requirements yet");
    }
    if !loaded.index_flags.is_empty() {
        bail!(
            "mamba pip compile currently uses --index DIR; requirements-file index flags are not applied yet"
        );
    }
    if !loaded.unknown.is_empty() {
        bail!(
            "mamba pip compile found unsupported requirement directives: {}",
            loaded.unknown.join(", ")
        );
    }
    Ok(())
}

fn constraints_by_name(
    constraints: &[PackageRequirement],
) -> Result<BTreeMap<String, Vec<String>>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for constraint in constraints {
        if constraint.direct_url.is_some() {
            bail!(
                "mamba pip compile does not support direct URL constraints for `{}`",
                constraint.name
            );
        }
        out.entry(pep503_normalize(&constraint.name))
            .or_default()
            .extend(constraint.specifiers.clone());
    }
    Ok(out)
}

fn resolve_requirement(
    req: &PackageRequirement,
    constraints: &BTreeMap<String, Vec<String>>,
    opts: &CompileOptions,
    state: &mut CompileState,
) -> Result<()> {
    let req = constrained_requirement(req, constraints);
    let wheel = wheel_for_requirement(&req, &opts.index)?;
    let metadata = read_wheel_metadata(&wheel)?;
    let key = metadata.canonical_name.clone();

    if !req.specifiers.is_empty() && !version_satisfies_all(&metadata.version, &req.specifiers) {
        bail!(
            "resolved {}=={} does not satisfy {:?}",
            metadata.name,
            metadata.version,
            req.specifiers
        );
    }
    if let Some(existing) = state.packages.get(&key) {
        if version_satisfies_all(&existing.version, &req.specifiers) {
            return Ok(());
        }
        bail!(
            "conflicting requirements for `{}`: already resolved {} but new specifiers are {:?}",
            metadata.name,
            existing.version,
            req.specifiers
        );
    }

    let mut dep_requirements = Vec::new();
    let mut dependencies = Vec::new();
    if !opts.no_deps {
        for dep in &metadata.requires {
            if is_extra_marker(dep) {
                continue;
            }
            match parse_one_line(dep)? {
                RequirementLine::Package(dep_req) => {
                    dependencies.push(dep_req.name.clone());
                    dep_requirements.push(dep_req);
                }
                other => bail!(
                    "unsupported dependency metadata for {}: {other:?}",
                    metadata.name
                ),
            }
        }
    }

    state.packages.insert(
        key,
        Package {
            name: metadata.name.clone(),
            version: metadata.version.clone(),
            sha256: sha256_file(&wheel)?,
            source: source_for(&metadata.name, &metadata.version, &req),
            dependencies,
            markers: None,
            source_ref: source_ref_for(&wheel, &req)?,
        },
    );

    for dep in dep_requirements {
        resolve_requirement(&dep, constraints, opts, state)?;
    }
    Ok(())
}

fn constrained_requirement(
    req: &PackageRequirement,
    constraints: &BTreeMap<String, Vec<String>>,
) -> PackageRequirement {
    let mut out = req.clone();
    if let Some(extra) = constraints.get(&pep503_normalize(&req.name)) {
        out.specifiers.extend(extra.clone());
    }
    out
}

fn wheel_for_requirement(req: &PackageRequirement, index: &Path) -> Result<PathBuf> {
    if let Some(url) = &req.direct_url {
        let Some(path) = url.strip_prefix("file://") else {
            bail!(
                "mamba pip compile only supports local file:// direct URLs for `{}`",
                req.name
            );
        };
        return absolute_path(Path::new(path));
    }
    find_index_wheel(req, index)
}

fn source_for(name: &str, version: &str, req: &PackageRequirement) -> String {
    if let Some(url) = &req.direct_url {
        return url.clone();
    }
    format!("pypi://{name}/{version}")
}

fn source_ref_for(wheel: &Path, req: &PackageRequirement) -> Result<Option<SourceRef>> {
    if req.direct_url.is_none() {
        return Ok(Some(SourceRef {
            kind: SourceRefKind::Registry,
            path: None,
            url: None,
            rev: None,
        }));
    }
    Ok(Some(SourceRef {
        kind: SourceRefKind::Path,
        path: Some(absolute_path(wheel)?.to_string_lossy().into_owned()),
        url: None,
        rev: None,
    }))
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(std::env::current_dir()?.join(path))
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    Ok(out)
}

fn input_hash(loaded: &LoadedRequirements) -> String {
    let mut hasher = Sha256::new();
    for req in &loaded.primary {
        hasher.update(req.name.as_bytes());
        hasher.update(b"\0");
        for spec in &req.specifiers {
            hasher.update(spec.as_bytes());
            hasher.update(b"\0");
        }
    }
    for req in &loaded.constraints {
        hasher.update(b"constraint\0");
        hasher.update(req.name.as_bytes());
        hasher.update(b"\0");
        for spec in &req.specifiers {
            hasher.update(spec.as_bytes());
            hasher.update(b"\0");
        }
    }
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn atomic_write(dest: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = dest.with_extension("tmp");
    std::fs::write(&tmp, bytes).with_context(|| format!("write {}", tmp.display()))?;
    std::fs::rename(&tmp, dest).with_context(|| format!("rename {}", dest.display()))?;
    Ok(())
}
