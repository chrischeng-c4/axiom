# mambalibs

The Rust kits under `mambalibs/` and their Python surface. This area holds the
README capability `mambalibs-end-to-end`. A kit reaches a Python program through
one of two carriers, in this order: a PyO3 extension wheel installed into a
CPython `.venv` first, and a `MambaModule` inside the mamba runtime second. The
wheel's Python API is the contract the runtime carrier must match.

## mambalibs CPython wheels (Milestone #129)

- Problem: no kit reaches a CPython user today. The workspace declares no `pyo3`
  dependency and no maturin `pyproject.toml`; the package manager parses
  `[tool.maturin]` (`src/pkgmanage/pkgmgr/maturin_compat.rs`) but never runs
  maturin; the `native-modules` feature that links the `MambaModule` bindings is
  outside every release build; and `cargo test -p mamba --test mambalibs` parses
  kit manifests without importing a kit. Six kits — arraykit, cryptokit,
  mediakit, mongokit, plotkit, scikit — have an empty binding, so the data stack
  the README names as the numpy chokepoint has no Python surface at all.
- Who: a Python developer on CPython 3.12 with a project `.venv` who wants the
  data-science kits (arrays, scientific routines, plotting) without a mamba
  runtime.
- Promise: each promised kit ships as its own standard wheel, `mambalibs-<kit>`,
  built with PyO3 and maturin from the kit's existing core crate and sharing the
  `mambalibs` namespace package. The wheel is published through `mamba publish`,
  served from a `mamba index` frozen index, and installed by
  `mamba add mambalibs-<kit>` followed by `mamba sync`; after that,
  `import mambalibs.<name>` works under the `.venv` interpreter and the kit's
  Python cases pass under pytest there. The data-science kits go first —
  arraykit (`mambalibs.array`), scikit, plotkit; every other kit follows the same
  route. The wheel's Python API is the contract the later `MambaModule` carrier
  must match.
- Non-goals: a `MambaModule` carrier for the same kit in this outcome (that is
  `cpython-runtime-replacement`); a CPython C-API emulation layer or a bridged
  CPython as a carrier (`cpython-c-api-emulation`); building a kit from an sdist
  on the user's machine (`sdist-c-extension-builds`).
- Open: `abi3-py312` or one wheel per CPython minor. Whether the wheels also go
  to PyPI or only to a mamba-managed index. How much of the numpy API
  `mambalibs.array` covers in the first Milestone, and the Python module names
  for scikit and plotkit. Whether the six empty `MambaModule` bindings are
  removed or kept until the runtime carrier round. Whether `mamba package`
  learns to drive maturin or maturin builds the wheel directly.
- Neighbours: after [package-manager.md](package-manager.md) § uv workflow
  parity, whose `mamba add`, `mamba sync` and `.venv` path is the install
  route; before [runtime.md](runtime.md) § CPython runtime replacement, whose
  `MambaModule` carrier must match this wheel's Python API.
- Outcome: `mambalibs-cpython-wheels`. Tracking: [Milestone #129](https://github.com/chrischeng-c4/axiom/milestone/129).

## Non-goals in this area

- `cpython-c-api-emulation`: a kit reaches CPython through PyO3's extension
  contract, not through an emulated CPython C-API inside the mamba runtime and
  not through a bridged CPython.
- `sdist-c-extension-builds`: the kits ship as wheels; nothing compiles them
  from source on the user's machine.
