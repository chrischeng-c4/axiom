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

### mambalibs CPython wheels

- ID: `mambalibs-cpython-wheels`
- Outcome: A Python developer on CPython 3.12 installs a mambalibs kit into a
  project `.venv` as a standard wheel, `mambalibs-<kit>`, built with PyO3 and
  maturin from the kit's existing core crate, published through
  `mamba publish`, served from a `mamba index` frozen index, and installed by
  `mamba add mambalibs-<kit>` followed by `mamba sync`; `import mambalibs.<name>`
  then works under the `.venv` interpreter and the kit's Python cases pass
  under pytest there. The data-science kits go first — arraykit, scikit,
  plotkit — and every other kit follows the same route. The wheel's Python API
  is the contract the later `MambaModule` carrier must match.
- Boundary: The PyO3 binding crate, the maturin manifest, and the wheel build,
  publish, index, install, import, and pytest path for each promised kit,
  starting with the data-science kits. The `MambaModule` carrier for the same
  kit, a CPython C-API emulation layer, and sdist builds on the user's machine
  stay outside it. It starts after `uv-workflow-parity` and does not change
  the package manager's contract.
- Completion evidence: A `[[test]]` target under `apps/mamba/e2e/` builds a
  kit wheel with maturin, serves it through `mamba index`, installs it with
  `mamba add` and `mamba sync` into a fresh `.venv`, and runs the kit's pytest
  cases on that interpreter, exiting zero for each promised kit; and
  `cargo test -p mamba --test mambalibs` stays green.
- Tracking: [Milestone #129](https://github.com/chrischeng-c4/axiom/milestone/129).

### CPython runtime replacement

- ID: `cpython-runtime-replacement`
- Outcome: A CPython 3.12 program compiled by mamba gives the same observable
  result as under CPython and uses less CPU time and less memory, delivered
  in the tier order T1 to T7 recorded in the README section
  [Runtime replacement order](README.md#runtime-replacement-order).
- Boundary: The compiler and runtime work the tiers describe, plus the
  `MambaModule` carrier for each kit that already ships as a wheel under
  `mambalibs-cpython-wheels`, matching that wheel's Python API. It starts
  after `uv-workflow-parity` and `mambalibs-cpython-wheels` and does not
  change the package manager's contract.
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

### CPython C-API emulation

- ID: `cpython-c-api-emulation`
- Reason: A mambalibs kit reaches CPython through PyO3's extension contract,
  not through an emulated CPython C-API inside the mamba runtime or a bridged
  CPython. The feasibility spike at
  [docs/native-extensions/c-api-feasibility-spike.md](docs/native-extensions/c-api-feasibility-spike.md)
  recommends native kits over a partial C-API emulation layer.
