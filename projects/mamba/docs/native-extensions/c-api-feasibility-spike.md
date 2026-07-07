# C-API Subset Emulation Feasibility Spike

Issue: #1121

## Purpose

This spike records whether mamba should pursue a partial CPython C-API
emulation layer as the first path for the current native-extension backlog.
It is a bounded decision artifact, not a prototype C-extension loader.

## Scope

- Evaluate the `numpy` / `psycopg` / `lxml` class of packages against mamba's
  current runtime, package-manager, and ecosystem posture.
- Capture repo-grounded evidence that already exists today.
- Produce a reviewable decision table for the next issue split.

## Non-Goals

- No CPython ABI loader is implemented by this issue.
- No `PyObject*` compatibility layer, `PyTypeObject` emulation, or extension
  import hook is implemented by this issue.
- No promise of `numpy`, `lxml`, `grpcio`, or `psycopg2` compatibility is made
  by this issue.
- No claim that the existing `ctypes` surface is sufficient for loading Python
  extension modules.

## Current Repo Evidence

### README posture

`projects/mamba/README.md` still states that C-extension packages cannot load
and frames the supported treatments as `Hack`, `Bridge`, or `Native kit`.
That remains directionally correct for package compatibility: there is still no
CPython extension-module loader and no general-purpose emulation of the Python
C-API import boundary.

### Existing `ctypes` is not a C-extension ABI

The runtime now has a real `ctypes` substrate in
`projects/mamba/src/runtime/stdlib/ctypes_mod.rs`, including `CDLL`,
`dlsym`-style lookup, and limited direct-call marshalling. That matters, but it
does not solve the native-extension problem:

- it is a stdlib-facing FFI surface, not an import-time extension-module loader
- it does not reconstruct CPython object layouts or refcount semantics
- it does not provide the module-init contract expected by `PyO3`, Cython,
  CPython C extensions, or mixed Rust/C extension wheels

So the repo does have some low-level FFI machinery, but it does not yet have a
viable C-extension ABI.

### Ecosystem manifest blockers already point at loader gaps

`projects/mamba/ecosystem_fixture_manifest.toml` already records native
extension blockers for:

- `numpy` -> `#2526 mamba native-extension loader not ready for numpy C core`
- `pandas` -> transitive dependency on the same loader gap
- `cryptography` -> `#2526 mamba native-extension loader not ready for cryptography Rust _rust extension`

That is strong evidence that current failures are classified as loader / ABI
gaps rather than ordinary stdlib parity gaps.

### Package-manager metadata already recognizes PyO3-shaped packages

`projects/mamba/src/pkgmanage/pkgmgr/maturin_compat.rs` parses
`[tool.maturin]` metadata and classifies `bindings = "pyo3"` projects as
Python extensions. This is useful for detection, diagnostics, and future
package-manager routing, but it is not a backend: metadata awareness does not
make the runtime able to import the resulting extension module.

### Existing ecosystem accounting already leans toward native kits or bridge

`projects/mamba/tests/harness/cpython/tools/third_party_readiness.py` already
classifies mandatory C-extension packages toward `mambalibs.*` replacements or
`native_extension_bridge`, including:

- `numpy` -> `mambalibs.array`, `mambalibs.sci`, `native_extension_bridge`
- `psycopg` -> `mambalibs.pg`, `native_extension_bridge`
- `orjson` / `grpcio` / `cryptography` -> `native_extension_bridge`

`projects/mamba/tests/harness/cpython/tools/mambalibs_readiness.py` also maps
native-kit replacements such as `mambalibs.pg` for `psycopg` and
`mambalibs.array` / `mambalibs.sci` for `numpy`.

## Feasibility Read

The core problem with "partial C-API emulation" is not just symbol count. The
first hard packages (`numpy`, `lxml`, `grpcio`, `psycopg2`) rely on deep,
package-specific assumptions around:

- object layout and lifetime
- import-time module initialization
- buffer / memory / capsule / descriptor contracts
- error propagation and thread / GIL expectations
- Cython- or PyO3-generated glue that assumes real CPython runtime behavior

That means a small, generic subset is unlikely to unlock the highest-value
packages first. The packages that look easiest to detect are not the packages
that look safest to emulate.

## Decision Table

| package | implementation shape | repo evidence | recommended route | rationale | next issue refs |
|---------|----------------------|---------------|-------------------|-----------|-----------------|
| `numpy` | deep C core, array protocol, buffer semantics, heavy CPython ABI assumptions | README marks it as a hack candidate; ecosystem manifest already blocks it on `#2526`; native-kit readiness already points to `mambalibs.array` / `mambalibs.sci` | `native mambalib replacement` | Not a good first candidate for partial CPython C-API emulation. A tiny ABI subset will not cover ndarray internals, ufunc dispatch, or the ecosystem expectation surface. Prefer native array/science kits; bridge only for bounded compatibility stopgaps. | `#1119`, `#1071` only when competitiveness/perf becomes the decision bar |
| `psycopg` | mixed story: `psycopg` 3 has pure-Python wrappers plus binary / `libpq` / optional speedups; `psycopg2` is classic C-extension territory | README ecosystem matrix already tracks `psycopg`; third-party readiness points to `mambalibs.pg` or bridge; mambalibs readiness maps `psycopg` to `mambalibs.pg` | `native mambalib replacement` for primary path, `bridge/subprocess CPython` for compatibility path | For IO-bound database client work, ABI emulation is a poor first investment. Prefer Rust-native `pg`/mambalib surfaces or a CPython bridge before chasing CPython ABI compatibility. `psycopg2` is especially unfavorable as an emulation-first target. | `#1119` |
| `psycopg2` | C extension over `libpq`, strong DB-API compatibility expectations | No dedicated repo-first path beyond `psycopg` native-kit and bridge posture | `bridge/subprocess CPython` | Too tied to the classic CPython extension path to justify as the first emulation target. Use bridge for compatibility cases; steer product work toward `mambalibs.pg`. | `#1119` |
| `lxml` | Cython-heavy binding over `libxml2` / `libxslt`, large object-model and parser surface | wheel filename parsing knows `lxml` wheels exist, but repo has no native replacement plan or ecosystem unblocker yet | `no-go/defer` | Not a good first-path package for partial C-API emulation. The Cython-generated surface and native dependency stack make it a bad proving ground for a minimal ABI subset. | `#1119` |
| `cryptography` | Rust core exposed through Python extension glue (`cryptography._rust`) | ecosystem manifest already blocks on `#2526`; README lists it as a foundation hack target; third-party readiness sends it to bridge | `PyO3-native backend` | This is a better backend-design target than a generic C-API emulation target because the implementation already centers on Rust. If mamba grows native-extension support, PyO3-shaped imports are more tractable than `numpy`-class C internals. | `#1120`, `#1119` |
| `pydantic-core` | Rust extension backing mostly Python-facing `pydantic` APIs | ecosystem manifest notes `pydantic-core`; README lists it as already-Rust-upstream | `PyO3-native backend` | Another good candidate for a PyO3-oriented path, not for broad CPython C-API subset emulation. The packaging/runtime problem is narrower than `numpy` or `lxml`. | `#1120`, `#1119` |
| `orjson` | Rust extension with narrow encode/decode surface | README lists `orjson` / `msgpack` as serde-backed candidates; third-party readiness currently leans bridge | `PyO3-native backend` | Narrower than `numpy`, but still better modeled as a PyO3-native backend problem than as a general C-API emulation problem. | `#1120`, `#1119` |
| `protobuf` | mixed ecosystem: Python package can run on pure-Python path, optimized path uses native code | README already treats `protobuf` as a top foundation and suggests `prost`; third-party readiness already allows `pure_python_fallback` | `native mambalib replacement` | This is exactly the kind of package where a native kit beats ABI emulation: mature Rust backing exists, API edges are clearer, and it unblocks downstream ecosystems. | `#1119` |
| `grpcio` | C/C++ wrapper stack over gRPC core | README lists `grpcio` as depending on `protobuf`; cloud SDK governance already records it as deferred / out of scope in the umbrella | `bridge/subprocess CPython` | Poor first emulation target. It is mostly IO-bound and already has a cleaner replacement direction via `tonic`-class native kits or non-`grpcio` Python transport choices. | `#1119` |

## Requested `numpy` / `psycopg` / `lxml` Conclusion

- `numpy` is not a good candidate for partial CPython C-API emulation as the
  first path.
- `lxml` is also not a good candidate for partial CPython C-API emulation as
  the first path.
- `psycopg` depends on the selected package generation:
  - `psycopg` 3 has wrapper choices and product space that can often be served
    better by `mambalibs.pg`
  - `psycopg2` is much closer to the "classic CPython extension" problem and
    should not be the proving ground for an emulation-first effort
- for IO-bound Postgres client work, prefer Rust-native `pg` / mambalib or a
  bridge/subprocess CPython route before ABI emulation

## Recommendation

Do not open the next implementation issue as "build a partial CPython C-API
subset for `numpy`/`lxml`/`psycopg2`."

Prefer a split like this instead:

1. `#1119` keeps the ecosystem-native replacement and bridge routing explicit.
2. `#1120` owns any future PyO3-oriented import/backend investigation for the
   narrower Rust-extension class (`cryptography`, `pydantic-core`, `orjson`).
3. Native-kit product work continues where mamba already has clear replacement
   stories (`mambalibs.pg`, `mambalibs.array`, `mambalibs.sci`, `prost`/`tonic`-class kits).

The bounded conclusion from this spike is:

> partial CPython C-API emulation is not the recommended first path for the
> `numpy` / `psycopg` / `lxml` class; use native replacements, PyO3-specific
> backend work, or CPython bridge routing depending on package shape.
