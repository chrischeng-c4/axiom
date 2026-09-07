# mambalibs

The Rust kits under `mambalibs/` and their Python surface. This area holds the
README capability `mambalibs-end-to-end`. A kit reaches a Python program through
one of two carriers, in this order: a PyO3 extension wheel installed into a
CPython `.venv` first, and a `MambaModule` inside the mamba runtime second. The
wheel's Python API is the contract the runtime carrier must match.

## mambalibs CPython wheels (Milestone #129)

- Problem: none open as shipped; the limits below are where the remaining
  kits and the runtime carrier start.
- Who: a Python developer on CPython 3.12 with a project `.venv` who wants the
  data-science kits (arrays, scientific routines, plotting) without a mamba
  runtime.
- Promise: the three data-science kits ship as standard wheels built with PyO3
  and maturin from each kit's existing core crate: `mambalibs-arraykit`
  (`import mambalibs.array`), `mambalibs-scikit` (`import mambalibs.sci`, with
  `sci.stats`, `sci.fft`, `sci.signal`, `sci.interpolate`, `sci.optimize`,
  `sci.ts`, `sci.spatial`, `sci.sparse`, and `sci.integrate` as attributes),
  and `mambalibs-plotkit` (`import mambalibs.plot`). Each is one `cp312-abi3`
  wheel per platform, so a single build serves CPython 3.12 and later, and
  the `mambalibs` package is a PEP 420 namespace — no wheel ships
  `mambalibs/__init__.py` — so the three install side by side in one `.venv`.
  A wheel built by `maturin build` is frozen by `mamba index build`, installed
  by `mamba add mambalibs-<kit>` followed by `mamba sync`, and the kit's
  Python cases pass under pytest on the `.venv` interpreter. Each kit is one
  black-box `[[test]]` under `apps/mamba/e2e/`: `mambalibs_array_wheel`,
  `mambalibs_sci_wheel`, and `mambalibs_plot_wheel`. The wheel's Python API is
  the contract the later `MambaModule` carrier must match.
- Limits today: the wheels are served only from a `mamba index` frozen index;
  publication to PyPI has no case. The host running the gates needs `maturin`
  on `PATH` and a `python3.12` that can `import pytest`; a missing tool fails
  the case rather than skipping it. Only arraykit, scikit, and plotkit have a
  wheel; cryptokit, mediakit, and mongokit still have no Python surface.
  `mamba package` does not build these wheels — the cases drive
  `maturin build` themselves.
- Non-goals: a `MambaModule` carrier for the same kit in this outcome (that is
  `cpython-runtime-replacement`); a CPython C-API emulation layer or a bridged
  CPython as a carrier (`cpython-c-api-emulation`); building a kit from an sdist
  on the user's machine (`sdist-c-extension-builds`).
- Neighbours: after [package-manager.md](package-manager.md) § uv workflow
  parity, whose `mamba add`, `mamba sync` and `.venv` path is the install
  route; before [runtime.md](runtime.md) § CPython runtime replacement, whose
  `MambaModule` carrier must match this wheel's Python API.
- Status rows: `mambalibs-cpython-wheels`.

## Non-goals in this area

- `cpython-c-api-emulation`: a kit reaches CPython through PyO3's extension
  contract, not through an emulated CPython C-API inside the mamba runtime and
  not through a bridged CPython.
- `sdist-c-extension-builds`: the kits ship as wheels; nothing compiles them
  from source on the user's machine.
