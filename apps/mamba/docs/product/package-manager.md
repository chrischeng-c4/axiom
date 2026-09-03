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
  `--index-url`; `venv` seeds `.venv` from the first `python` on `PATH` or
  from `--python`; `sync` converges `.venv` to `mamba.lock` and is a no-op the
  second time; `run -- <cmd>` runs a command inside that environment;
  `python`, `tool`, `version`, `package`, `shell`, `cache`, `hash`, `index`,
  `auth`, `audit`, and `pip` cover the rest of the workflow. Every verb is
  driven offline by the `pkgmgr` test target.
- Limits today: `mamba run <file>` compiles the file with mamba rather than
  executing it on the `.venv` interpreter; `publish` validates payloads with
  `--dry-run` and has no upload case; `auth login` stores plaintext
  credentials.
- Non-goals: `uv.lock` byte compatibility; building C extensions from sdist;
  the full `pip` option surface.
- Neighbours: first section of the area.
  [uv workflow parity](#uv-workflow-parity) changes the `run` default this
  section records as a limit and nothing else here.
- Status rows: `project-dependencies`, `environment-and-run`,
  `interpreter-management`, `build-and-version`, `tooling-and-cache`,
  `sources-and-credentials`.

## uv workflow parity

- Problem: a Python developer who reaches for mamba as a `uv` replacement
  hits a different `run`: inside a project `mamba run <file>` compiles the
  file through mamba, so a program that relies on CPython semantics or on a
  C extension in `.venv` fails where `uv run <file>` succeeds. The other
  common verbs differ from `uv` in flags, exit codes, and output in ways no
  case measures today.
- Who: a Python developer with a system or managed CPython who wants to run
  `mamba` where they run `uv`, without installing or learning a mamba
  runtime.
- Promise: the common `uv` subcommands (`init`, `add`, `remove`, `lock`,
  `sync`, `run`, `venv`, `python`, `tool`, `version`, `tree`, and `export`)
  accept the same flags, exit with the same codes, and leave the same
  `pyproject.toml`, lockfile, and `.venv` artifacts as `uv`, measured by a
  black-box case per verb under `apps/mamba/e2e/`. Inside a project,
  `mamba run <file>` executes the file on the `.venv` interpreter by default;
  compiling through mamba is an explicit opt-in flag. No mamba runtime is
  needed for any of it.
- Non-goals: `uv.lock` byte compatibility; resolver speed parity; every `pip`
  option that `uv pip` does not expose; sdist C-extension builds.
- Open: the name of the opt-in compiler flag on `mamba run`. Which
  subcommands and flags count as common: the list above is the interview's
  answer, and the Milestone's issue set fixes it. Whether `mamba run <file>`
  outside a project falls back to the `python` on `PATH` or keeps compiling.
- Neighbours: rewrites the `run` limit in
  [Offline project workflow](#offline-project-workflow) and flips the
  `environment-and-run` STATUS row to Supported;
  [runtime.md](runtime.md) § CPython runtime replacement starts after this
  outcome and does not touch the package manager's contract.
- Outcome: `uv-workflow-parity`. Tracking: Not assigned.

## Non-goals in this area

- `sdist-c-extension-builds`: a host toolchain and a build backend mamba does
  not own; wheels are the artifact.
- `resolver-speed-parity-with-uv`: parity is observable behaviour, not
  resolution time.
- `full-pip-option-surface`: `mamba pip` covers the workflow's inspection and
  install verbs, not every `pip` option.
