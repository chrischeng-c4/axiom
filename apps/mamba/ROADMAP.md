# Mamba roadmap

## Purpose

This file orders the outcomes that change what mamba promises. Near-term
outcomes are listed in delivery order: the first is the next release
Milestone's candidate. An outcome moves into the README and STATUS only when
its implementation and its executable gate exist. The product prose behind
each outcome lives under [docs/product/README.md](docs/product/README.md).

## Near-term outcomes

### CPython runtime replacement

- ID: `cpython-runtime-replacement`
- Outcome: A CPython 3.12 program compiled by mamba gives the same observable
  result as under CPython and uses less CPU time and less memory, delivered
  in the tier order T1 to T7 recorded in the README section
  [Runtime replacement order](README.md#runtime-replacement-order).
- Boundary: The compiler and runtime work the tiers describe, plus the
  `MambaModule` carrier for each kit that already ships as a wheel
  ([STATUS](STATUS.md) `mambalibs-cpython-wheels`), matching that wheel's
  Python API. It starts after the shipped uv workflow parity
  ([STATUS](STATUS.md) `environment-and-run`) and the shipped mambalibs
  CPython wheels, and does not change the package manager's contract.
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
