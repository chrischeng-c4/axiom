//! Consolidated umbrella binary for the `mamba` package-manager CLI
//! verbs (C4 of the four-capability MVP: uv-like package manager).
//!
//! Each former top-level `tests/cli_<verb>.rs` is registered below via
//! `#[path]` and lives as `tests/pkgmgr/<verb>.rs`. cargo only treats
//! `tests/*.rs` as integration binaries, so the verb files are no
//! longer separate binaries — this umbrella is the sole binary that
//! links them all.
//!
//! Each verb file spawns the in-repo `mamba` CLI via
//! `env!("CARGO_BIN_EXE_mamba")` and pins the acceptance contract from
//! the matching `tests/governance/gates/pkgmgr/<verb>/manifest.toml`.
//!
//! Selector: `cargo test -p mamba --test pkgmgr`.

#[path = "add.rs"]
mod add;

#[path = "audit.rs"]
mod audit;

#[path = "auth.rs"]
mod auth;

#[path = "cache.rs"]
mod cache;

#[path = "export.rs"]
mod export;

#[path = "hash.rs"]
mod hash;

#[path = "init.rs"]
mod init;

#[path = "index.rs"]
mod index;

#[path = "install.rs"]
mod install;

#[path = "lock.rs"]
mod lock;

#[path = "package.rs"]
mod package;

#[path = "pip.rs"]
mod pip;

#[path = "python.rs"]
mod python;

#[path = "remove.rs"]
mod remove;

#[path = "run_preflight.rs"]
mod run_preflight;

#[path = "run_stdin.rs"]
mod run_stdin;

#[path = "shell.rs"]
mod shell;

#[path = "sync.rs"]
mod sync;

#[path = "tool.rs"]
mod tool;

#[path = "tree.rs"]
mod tree;

#[path = "validate.rs"]
mod validate;

#[path = "venv.rs"]
mod venv;

#[path = "version.rs"]
mod version;

#[path = "workspace.rs"]
mod workspace;
