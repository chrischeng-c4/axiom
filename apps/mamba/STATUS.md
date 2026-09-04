# Mamba status

## Scope

This file records the support state of mamba's package-manager surface: the
`uv`-shaped verbs a Python developer runs against a project directory. Rows
are grouped by workflow rather than by verb, so one row changes when that
workflow's public contract changes. The compiler and runtime capabilities in
the README (`cpython-312-parity`, `cpu-and-memory-under-cpython`, and
`mambalibs-end-to-end`) are measured by their own README gates and have no row
here; their promise is the ROADMAP outcome
[cpython-runtime-replacement](ROADMAP.md#cpython-runtime-replacement).

## State definitions

| State | Meaning |
|---|---|
| Supported | The workflow runs offline against a frozen local index, a local wheel path, or an explicit registry URL, and the named gate exercises its verbs. |
| Limited | The workflow runs, and the Limits column names the one ROADMAP outcome that closes the gap. |
| Not supported | The workflow is not offered; the Evidence column names the ROADMAP entry that explains why. |

## Support matrix

| Surface | ID | State | Supported scope | Limits | Evidence |
|---|---|---|---|---|---|
| Project dependencies | `project-dependencies` | Supported | `init`, `add`, `remove`, `lock`, `export`, `tree`, and `workspace` over `mamba.toml` and `mamba.lock`, resolving from a frozen `--index <DIR>`, an explicit `--index-url`, or a local wheel path. | Nothing reaches PyPI implicitly; a bare name needs `--index` or `--index-url`. `export` emits `requirements.txt` or `pylock.toml`, not `uv.lock`. | `cargo test -p mamba --test pkgmgr` |
| Environment and run | `environment-and-run` | Limited | `venv` seeds `.venv` from the first `python` on `PATH` or from `--python`; `sync` converges `.venv` to `mamba.lock` and is a no-op on the second run; `run -- <cmd>` runs a command inside the synced environment. | `mamba run <file>` compiles the file with mamba instead of executing it on the `.venv` interpreter, so a program that needs CPython semantics runs through `run -- python <file>` today: [uv-workflow-parity](ROADMAP.md#uv-workflow-parity). | `cargo test -p mamba --test pkgmgr` |
| Interpreter management | `interpreter-management` | Supported | `python list`, `find`, `pin`, `dir`, and `update-shell` over the interpreters on `PATH` and the `.python-version` pin; `python install`, `download`, and `uninstall` over managed interpreters registered from a local source interpreter or a standalone archive. | A managed interpreter comes from the local interpreter or archive the caller names. | `cargo test -p mamba --test pkgmgr` |
| Build and version | `build-and-version` | Supported | `version` reads, sets, or bumps the PEP 621 `[project].version` in `pyproject.toml`; `package build` produces deterministic pure-Python wheel and sdist artifacts. | `publish` validates upload payloads with `--dry-run`; an actual upload has no case in the gate. | `cargo test -p mamba --test pkgmgr` |
| Tooling and cache | `tooling-and-cache` | Supported | `tool run`, `install`, `upgrade`, `list`, `uninstall`, `dir`, and `update-shell` from a frozen local index; `shell path` and `shell init`; `cache dir`, `size`, `info`, `clean`, and `prune`; `hash` over one or more files. | A tool resolves only from a frozen local index named by `--index` or `MAMBA_FROZEN_INDEX`. | `cargo test -p mamba --test pkgmgr` |
| Sources and credentials | `sources-and-credentials` | Supported | `index build` freezes wheel files or directories into a local index; `auth login`, `logout`, `token`, and `dir`; `audit` of `mamba.lock` against a local advisory database; the `pip` compatibility verbs `compile`, `install`, `sync`, `uninstall`, `list`, `freeze`, `show`, `tree`, and `check`. | `auth login` stores plaintext credentials; `audit` reads only the advisory database it is given. | `cargo test -p mamba --test pkgmgr` |

## Evidence policy

The command in each row is the required gate for that row's scope. This file
names the gate and does not record a run; the change that alters a row's
public contract runs the gate itself and updates the row in the same change.
`cargo test -p mamba --test pkgmgr` is one `[[test]]` target declared in
`Cargo.toml`; its runner at `tests/pkgmgr/runner.rs` carries one module per
verb, so a verb with no module there is not covered by the row that lists it,
and the Limits column says so. `mamba pkgmgr-validate --json` drives the same
workflow families from a built binary; it is the by-hand check behind the
rows, not a gate, because it needs a binary the checkout does not carry.
Promote a ROADMAP outcome into this file only after its implementation and its
executable gate exist.
