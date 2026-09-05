# Mamba status

## Scope

This file records the support state of mamba's package-manager surface: the
`uv`-shaped verbs a Python developer runs against a project directory. Rows
are grouped by workflow rather than by verb, so one row changes when that
workflow's public contract changes. The compiler and runtime capabilities in
the README (`cpython-312-parity`, `cpu-and-memory-under-cpython`, and
`mambalibs-end-to-end`) are measured by their own README gates and have no row
here; their promise is the ROADMAP outcome
[cpython-runtime-replacement](ROADMAP.md#cpython-runtime-replacement). The
mambalibs kits' promise to a CPython user — a PyO3 wheel installed into the
project `.venv` — is the ROADMAP outcome
[mambalibs-cpython-wheels](ROADMAP.md#mambalibs-cpython-wheels); no row
measures it until a wheel exists.

## State definitions

| State | Meaning |
|---|---|
| Supported | The workflow runs offline against a frozen local index, a local wheel path, or an explicit registry URL, and the named gate exercises its verbs. |
| Limited | The workflow runs, and the Limits column names the one ROADMAP outcome that closes the gap. |
| Not supported | The workflow is not offered; the Evidence column names the ROADMAP entry that explains why. |

## Support matrix

| Surface | ID | State | Supported scope | Limits | Evidence |
|---|---|---|---|---|---|
| Project dependencies | `project-dependencies` | Supported | `init`, `add`, `remove`, `lock`, `export`, `tree`, and `workspace` over `mamba.toml` and `mamba.lock`, resolving from a frozen `--index <DIR>`, an explicit `--index-url` (a registry base or its `…/simple` URL), or a local wheel path; `lock` and `add` pin the transitive closure with a paired `sha256` and `url` or `path` per package, `remove` keeps the remaining roots' closure and digests, and `pkgmgr-validate` accepts the PEP 503 canonical name the lock records. | Nothing reaches PyPI implicitly; a bare name needs `--index`, `--index-url`, or `MAMBA_FROZEN_INDEX` / `MAMBA_INDEX_URL`, and `mamba.toml` records no index. `export` emits `requirements.txt` or `pylock.toml`, not `uv.lock`. | `cargo test -p mamba --test pkgmgr`, `cargo test -p mamba --test pkgmgr_lock_frozen_transitive`, `cargo test -p mamba --test pkgmgr_index_url_simple`, `cargo test -p mamba --test pkgmgr_lock_registry_sha_pairs_url`, `cargo test -p mamba --test pkgmgr_add_registry_transitive`, `cargo test -p mamba --test pkgmgr_remove_keeps_remaining_closure`, `cargo test -p mamba --test pkgmgr_validate_auth_family` |
| Environment and run | `environment-and-run` | Supported | `venv` seeds `.venv` from the first `python` on `PATH` or from `--python`; `sync` installs the locked wheels into `.venv`'s `lib/pythonX.Y/site-packages`, is a no-op on the second run, removes the packages `mamba.lock` no longer pins, and `sync --check` reports drift without writing; `run -- <cmd>` runs a command inside the synced environment; inside a project `run <file>` executes the file on the `.venv` interpreter and `run --compile <file>` compiles it with mamba; outside a project `run <file>` uses the first `python` on `PATH`. | `sync` compares `.venv` to `mamba.lock`, not `mamba.lock` to `mamba.toml`, so an edited manifest needs `lock` first. | `cargo test -p mamba --test pkgmgr`, `cargo test -p mamba --test pkgmgr_sync_real_install`, `cargo test -p mamba --test pkgmgr_sync_prune`, `cargo test -p mamba --test pkgmgr_run_file_venv` |
| Interpreter management | `interpreter-management` | Supported | `python list`, `find`, `pin`, `dir`, and `update-shell` over the interpreters on `PATH` and the `.python-version` pin; `python install`, `download`, and `uninstall` over managed interpreters registered from a local source interpreter or a standalone archive. | A managed interpreter comes from the local interpreter or archive the caller names. | `cargo test -p mamba --test pkgmgr` |
| Build and version | `build-and-version` | Supported | `version` reads, sets, or bumps the PEP 621 `[project].version` in `pyproject.toml`; `package build` produces deterministic pure-Python wheel and sdist artifacts. | `version` reads `pyproject.toml`, not `mamba.toml`. `publish` validates upload payloads with `--dry-run`; an actual upload has no case in the gate. | `cargo test -p mamba --test pkgmgr` |
| Tooling and cache | `tooling-and-cache` | Supported | `tool run`, `install`, `upgrade`, `list`, `uninstall`, `dir`, and `update-shell` from a frozen local index; `shell path` and `shell init`; `cache dir`, `size`, `info`, `clean`, and `prune`; `hash` over one or more files. | A tool resolves only from a frozen local index named by `--index` or `MAMBA_FROZEN_INDEX`. | `cargo test -p mamba --test pkgmgr` |
| Sources and credentials | `sources-and-credentials` | Supported | `index build` freezes wheel files or directories into a local index, recording each wheel's `Requires-Dist` and sha256 for the frozen resolver; `auth login`, `logout`, `token`, and `dir`; `audit` of `mamba.lock` against a local advisory database; the `pip` compatibility verbs `compile`, `install`, `sync`, `uninstall`, `list`, `freeze`, `show`, `tree`, and `check`. | `auth login` stores plaintext credentials; `audit` reads only the advisory database it is given. | `cargo test -p mamba --test pkgmgr`, `cargo test -p mamba --test pkgmgr_lock_frozen_transitive` |

## Evidence policy

The command in each row is the required gate for that row's scope. This file
names the gate and does not record a run; the change that alters a row's
public contract runs the gate itself and updates the row in the same change.
`cargo test -p mamba --test pkgmgr` is one `[[test]]` target declared in
`Cargo.toml`; its runner at `tests/pkgmgr/runner.rs` carries one module per
verb, so a verb with no module there is not covered by the row that lists it,
and the Limits column says so. Each `cargo test -p mamba --test pkgmgr_<case>`
gate is one black-box case under `e2e/`, declared as its own `[[test]]`
target with `autotests = false`, so `Cargo.toml` is the inventory of those
cases. `mamba pkgmgr-validate --json` drives the same
workflow families from a built binary; it is the by-hand check behind the
rows, not a gate, because it needs a binary the checkout does not carry.
Promote a ROADMAP outcome into this file only after its implementation and its
executable gate exist.
