// `mamba init` — uv-style project scaffolding.
//
// Acceptance (tests/governance/gates/pkgmgr/init/manifest.toml, schema gate
// pkgmgr_init_fixture_2679.rs):
//
//   - First run in an empty directory creates: mamba.toml, .python-version,
//     .gitignore, README.md, src/__init__.py.
//   - Re-running is idempotent: keep-existing policy, exit 0, stderr says
//     "already initialized", and mamba.toml / src/__init__.py / README.md
//     are preserved byte-for-byte.
//   - Offline; no $HOME or global-cache reads.

use anyhow::{Context, Result};
use clap::ArgMatches;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_PYTHON_REQUIRES: &str = ">=3.12";
const DEFAULT_PYTHON_VERSION_FILE: &str = "3.12\n";

pub fn cmd_init(sub: &ArgMatches) -> Result<()> {
    let dir_arg = sub.get_one::<String>("path").cloned();
    let project_dir: PathBuf = match dir_arg {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().context("read current directory")?,
    };

    if !project_dir.exists() {
        fs::create_dir_all(&project_dir)
            .with_context(|| format!("create project directory {}", project_dir.display()))?;
    }

    let manifest_path = project_dir.join("mamba.toml");
    if manifest_path.exists() {
        eprintln!("mamba: already initialized at {}", project_dir.display());
        return Ok(());
    }

    let project_name = project_name_from_dir(&project_dir);

    write_new(&manifest_path, &render_manifest(&project_name))?;
    write_new(&project_dir.join(".python-version"), DEFAULT_PYTHON_VERSION_FILE)?;
    write_new(&project_dir.join(".gitignore"), DEFAULT_GITIGNORE)?;
    write_new(&project_dir.join("README.md"), &render_readme(&project_name))?;

    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir)
        .with_context(|| format!("create {}", src_dir.display()))?;
    write_new(&src_dir.join("__init__.py"), "")?;

    Ok(())
}

fn project_name_from_dir(dir: &Path) -> String {
    dir.canonicalize()
        .ok()
        .as_deref()
        .and_then(Path::file_name)
        .or_else(|| dir.file_name())
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "mamba-project".to_string())
}

fn render_manifest(name: &str) -> String {
    format!(
        "[project]\n\
         name = \"{name}\"\n\
         version = \"0.1.0\"\n\
         python-requires = \"{DEFAULT_PYTHON_REQUIRES}\"\n\
         dependencies = []\n\
         dev-dependencies = []\n"
    )
}

fn render_readme(name: &str) -> String {
    format!("# {name}\n\nA mamba project.\n")
}

const DEFAULT_GITIGNORE: &str = "\
# mamba
.venv/
mamba.lock.lock

# python
__pycache__/
*.pyc
*.pyo
*.egg-info/

# build artifacts
target/
*.so
*.dylib
*.dll
";

fn write_new(path: &Path, body: &str) -> Result<()> {
    fs::write(path, body).with_context(|| format!("write {}", path.display()))
}
