// pkgmanage::lockfile — top-level mamba.lock (project-wide).
//
// Empty placeholder for Wave 1. Distinct from `pkgmanage::pkgmgr::lockfile`,
// which is the PyPI resolver's per-resolution lockfile inside the index client.
// This module will own the user-facing `mamba.lock` written next to
// `mamba.toml`, tying `[crates.*]` entries to resolved versions + hashes.
