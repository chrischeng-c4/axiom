# Mamba roadmap

## Purpose

This file orders the outcomes that change what mamba promises. Near-term
outcomes are listed in delivery order: the first is the next release
Milestone's candidate. An outcome moves into the README and STATUS only when
its implementation and its executable gate exist. The product prose behind
each outcome lives under [docs/product/README.md](docs/product/README.md).

## Near-term outcomes

### uv workflow parity

- ID: `uv-workflow-parity`
- Outcome: A Python developer with a system or managed CPython uses `mamba` in
  place of `uv` for the common project workflow (`init`, `add`, `remove`,
  `lock`, `sync`, `run`, `venv`, `python`, `tool`, `version`, `tree`, and
  `export`) and gets the same observable behaviour: the same flags, the same
  exit codes, and the same on-disk artifacts. Inside a project,
  `mamba run <file>` executes the file on the `.venv` interpreter by default,
  and compiling through mamba is an explicit opt-in flag. No mamba runtime is
  needed to use mamba this way.
- Boundary: Observable behaviour of the common subcommands only: flags, exit
  codes, stdout shape, and the `pyproject.toml`, lockfile, and `.venv`
  artifacts. Byte compatibility of `uv.lock`, resolver speed, and every `pip`
  option that `uv pip` does not expose stay outside it.
- Completion evidence: Black-box cases under `apps/mamba/e2e/` run each parity
  verb against a project fixture and compare exit code and artifacts with the
  documented `uv` behaviour; `cargo test -p mamba --test pkgmgr` stays green;
  and `mamba run <file>` inside a project observably executes on the `.venv`
  interpreter with no mamba runtime present.
- Tracking: Not assigned.

### CPython runtime replacement

- ID: `cpython-runtime-replacement`
- Outcome: A CPython 3.12 program compiled by mamba gives the same observable
  result as under CPython and uses less CPU time and less memory, delivered
  in the tier order T1 to T7 recorded in the README section
  [Runtime replacement order](README.md#runtime-replacement-order).
- Boundary: The compiler, runtime, and mambalibs work the tiers describe. It
  starts after `uv-workflow-parity` and does not change the package manager's
  contract.
- Completion evidence: `cargo test -p mamba --test conformance_contract` and
  `cargo test -p mamba --release --test perf_pin` exit zero over the tier's
  fixture set, with the tier's exit gate written and named in the README
  before the tier is claimed.
- Tracking: Not assigned.

## Later outcomes

No items.

## Non-goals

### sdist C-extension builds

- ID: `sdist-c-extension-builds`
- Reason: Building a C extension from an sdist needs a host compiler toolchain
  and a build backend that mamba does not own; wheels are the supported
  artifact.

### resolver speed parity with uv

- ID: `resolver-speed-parity-with-uv`
- Reason: Parity is measured on observable behaviour, not on resolution time;
  a speed target would trade correctness work for benchmark work.

### full pip option surface

- ID: `full-pip-option-surface`
- Reason: `mamba pip` covers the inspection and install verbs a project
  workflow needs; reproducing every `pip` option is a maintenance surface
  with no user behind it.
