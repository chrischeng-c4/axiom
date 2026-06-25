// pkgmanage — mamba's own package-management bounded context.
//
// Owns:
//   - manifest/  : mamba.toml schema + parsing
//   - pkgmgr/    : PyPI-compatible index client + resolver + installer + lockfile
//   - builder/   : `mamba build` orchestration + native-module force-link table
//   - source/    : path / git / registry fetch (B1 vendor-at-install landing zone)
//   - lockfile/  : mamba.lock (top-level, separate from pkgmgr internal lockfile)
//
// Lives alongside `cpython::*` (language + stdlib) and `mambalibs::*`
// (PyPI-equivalents). See projects/mamba/PLAN.md.

pub mod add;
pub mod builder;
pub mod cache;
pub mod export;
pub mod hash;
pub mod index;
pub mod init;
pub mod install;
pub mod lock;
pub mod lockfile;
pub mod manifest;
pub mod pip;
pub mod pkgmgr;
pub mod python;
pub mod remove;
pub mod run;
pub mod shell;
pub mod source;
pub mod sync;
pub mod tree;
pub mod validate;
pub mod venv;
pub mod version;
pub mod workspace;
