// `mamba remove` — uv-style dependency removal.
//
// Acceptance (tests/governance/gates/pkgmgr/remove/manifest.toml, schema gate
// pkgmgr_remove_fixture_2680.rs):
//
//   - Removed dep no longer appears in mamba.toml.
//   - mamba.lock is updated deterministically (byte-identical on replay).
//   - Other deps and project metadata fields are preserved.
//   - Offline.
//
// Removing a dep that isn't recorded is a soft no-op success (idempotent
// re-removal), matching uv's behavior.

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use std::fs;

use crate::pkgmanage::add::{ManifestState, atomic_write, render_lockfile_from_deps};

const MANIFEST_FILE: &str = "mamba.toml";
const LOCKFILE_FILE: &str = "mamba.lock";

pub fn cmd_remove(sub: &ArgMatches) -> Result<()> {
    let name = sub
        .get_one::<String>("name")
        .context("missing required argument <name>")?;
    if name.trim().is_empty() {
        bail!("empty dependency name");
    }

    let project_dir = std::env::current_dir().context("read current directory")?;
    let manifest_path = project_dir.join(MANIFEST_FILE);
    if !manifest_path.exists() {
        bail!(
            "no {MANIFEST_FILE} in {} — run `mamba init` first",
            project_dir.display()
        );
    }

    let manifest_src = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let mut state = ManifestState::parse(&manifest_src)?;

    let before = state.dependencies.clone();
    state.remove_dependency(name);
    let removed = before.len() != state.dependencies.len();

    let new_manifest = state.render();
    let new_lockfile = render_lockfile_from_deps(&state.dependencies);

    atomic_write(&manifest_path, new_manifest.as_bytes())?;
    let lock_path = project_dir.join(LOCKFILE_FILE);
    atomic_write(&lock_path, new_lockfile.as_bytes())?;

    if !removed {
        eprintln!("mamba: `{name}` was not recorded as a dependency (no-op)");
    }

    Ok(())
}
