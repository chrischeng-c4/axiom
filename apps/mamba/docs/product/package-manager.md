# Package manager

The `uv`-shaped verbs mamba runs against a project directory. This area is the
README capability `uv-style-package-manager`. Everything here works offline
against a frozen local index, a local wheel path, or an explicit registry URL;
nothing reaches PyPI implicitly.

## Offline project workflow

- Problem: none open as shipped.
- Who: a Python developer keeping a project's dependencies, environment, and
  interpreter pin reproducible from a checkout.
- Promise: `mamba init` scaffolds `mamba.toml`; `add`, `remove`, `lock`,
  `export`, `tree`, and `workspace` keep `mamba.toml` and `mamba.lock`
  consistent from a frozen `--index <DIR>`, a local wheel path, or an explicit
  `--index-url` (a registry base or its `…/simple` URL), pinning the
  transitive closure with a paired `sha256` and `url` or `path` per package;
  `venv` seeds `.venv` from the first `python` on `PATH` or from `--python`;
  `sync` installs the locked wheels into `.venv`'s
  `lib/pythonX.Y/site-packages`, removes what the lock no longer pins, and is
  a no-op the second time; `run -- <cmd>` runs a command inside that
  environment and `run <file>` executes the file on the `.venv` interpreter;
  `python`, `tool`, `version`, `package`, `shell`, `cache`, `hash`, `index`,
  `auth`, `audit`, and `pip` cover the rest of the workflow. Every verb is
  driven offline by the `pkgmgr` test target.
- Limits today: a bare `add <name>` needs `--index`, `--index-url`, or
  `MAMBA_FROZEN_INDEX` / `MAMBA_INDEX_URL`, and `mamba.toml` records no
  index, so the source is repeated on every `add` and `lock`. `sync` compares
  `.venv` to `mamba.lock`, never `mamba.lock` to `mamba.toml`, so an edited
  manifest needs `lock` first. `version` reads `pyproject.toml`, not
  `mamba.toml`. `tool install` resolves only from a frozen index. `publish`
  validates payloads with `--dry-run` and has no upload case; `auth login`
  stores plaintext credentials.
- Non-goals: `uv.lock` byte compatibility; building C extensions from sdist;
  the full `pip` option surface.
- Neighbours: first section of the area.
  [uv workflow parity](#uv-workflow-parity-milestone-130) shipped the real
  install, the transitive lock, and the `run <file>` default this section
  records, and nothing else here.
- Status rows: `project-dependencies`, `environment-and-run`,
  `interpreter-management`, `build-and-version`, `tooling-and-cache`,
  `sources-and-credentials`.

## uv workflow parity (Milestone #130)

- Problem: none open as shipped; the limits below are where the next
  package-manager outcome starts.
- Who: a Python developer with a system or managed CPython who wants to run
  `mamba` where they run `uv`, without installing or learning a mamba
  runtime.
- Promise: inside a project `mamba run <file>` executes the file on the
  `.venv` interpreter, `mamba run --compile <file>` compiles it with mamba,
  and outside a project `run <file>` uses the first `python` on `PATH`.
  `sync` installs the locked wheels into `.venv`'s PEP 405
  `lib/pythonX.Y/site-packages`, is a no-op the second time, removes the
  packages the lock no longer pins, and `sync --check` reports the drift
  without writing. `index build` records each wheel's `Requires-Dist` and
  sha256, so `lock` and `add --index <DIR>` pin the transitive closure with a
  paired `sha256` and `path`; `lock` and `add --index-url <URL>` do the same
  against a registry base or its `…/simple` URL with a paired `sha256` and
  `url`; `remove` keeps the remaining roots' closure and digests; and
  `pkgmgr-validate` accepts the PEP 503 canonical name the lock records. No
  mamba runtime is needed for any of it. Each behaviour is one black-box
  `[[test]]` under `apps/mamba/e2e/`: `pkgmgr_run_file_venv`,
  `pkgmgr_sync_real_install`, `pkgmgr_sync_prune`,
  `pkgmgr_lock_frozen_transitive`, `pkgmgr_index_url_simple`,
  `pkgmgr_lock_registry_sha_pairs_url`, `pkgmgr_add_registry_transitive`,
  `pkgmgr_remove_keeps_remaining_closure`, and
  `pkgmgr_validate_auth_family`.
- Limits today: flag, exit-code, and artifact parity with `uv` is measured
  only for the verbs those cases drive (`add`, `remove`, `lock`, `sync`,
  `run`, `index`, and `pkgmgr-validate`); `init`, `venv`, `python`, `tool`,
  `version`, `tree`, and `export` are measured by the `pkgmgr` target alone.
  The manifest is `mamba.toml`, not `pyproject.toml`, and `version` reads
  `pyproject.toml` only. `tool install` resolves only from a frozen index. A
  bare `add <name>` needs a source flag or variable and `mamba.toml` records
  none; `sync` never compares `mamba.lock` to `mamba.toml`.
- Non-goals: `uv.lock` byte compatibility; resolver speed parity; every `pip`
  option that `uv pip` does not expose; sdist C-extension builds.
- Neighbours: rewrote the `run` limit in
  [Offline project workflow](#offline-project-workflow) and flipped the
  `environment-and-run` STATUS row to Supported;
  [runtime.md](runtime.md) § CPython runtime replacement starts after this
  section and does not touch the package manager's contract.
- Status rows: `environment-and-run`, `project-dependencies`.

## Non-goals in this area

- `sdist-c-extension-builds`: a host toolchain and a build backend mamba does
  not own; wheels are the artifact.
- `resolver-speed-parity-with-uv`: parity is observable behaviour, not
  resolution time.
- `full-pip-option-surface`: `mamba pip` covers the workflow's inspection and
  install verbs, not every `pip` option.
