# mamba Test Architecture Index

Canonical rules live in `../../../CONTRIBUTING.md` under "Mamba test
architecture: DDD, boundary-first". This file is only a local index for the
`projects/mamba/tests/` tree.

## Domain Map

```text
tests/
├── cpython/     CPython replacement contract: runtime parity, strict type
│                deltas, speed, memory, security, and compatibility fixtures.
├── mambalibs/   Mamba-native library contracts with no CPython oracle.
├── pkgmgr/      `mamba` CLI and package-manager behavior.
└── governance/  Meta-gates over manifests, release profiles, CI policy, and
                 test-inventory shape.
```

Cargo does not auto-discover Rust test files under subdirectories, so
`projects/mamba/Cargo.toml` lists the domain entrypoints explicitly. Keep new
Rust integration tests under a domain directory and either register a domain
entrypoint in `Cargo.toml` or add a module to the existing umbrella runner.

Domain-local helper scripts belong under their domain as well. For example,
CPython golden regeneration lives at `cpython/tools/regen_golden.py`, not at
the tests root.

## Boundary-First Rules

- `cpython/` defines what a CPython replacement must do. It answers:
  `surface` has the API, `behavior` matches CPython, `errors` raises where
  CPython raises, `real_world` works in user-shaped flows, `security` does not
  crash or hang, and `bench` is faster/lower-memory than CPython.
- `type-strict` is the deliberate incompatibility lane: CPython may accept a
  case, but mamba must reject it with the explicit inverse markers.
- `mambalibs/` is not CPython compatibility. It covers mamba-only native library
  loading, ABI, generated stubs, and local artifact behavior.
- `pkgmgr/` owns package-manager CLI behavior. It should spawn the built
  `mamba` binary and pin observable CLI outcomes.
- `governance/` owns contracts about the test system itself: manifest schemas,
  release gates, profile definitions, skip debt, and structural lints.

## Agent-Optimized Layout

Follow the repository-wide rule from `CONTRIBUTING.md`:

```text
<area>/<subject>/<concern>/<artifact>
```

For CPython fixtures, the concrete grammar is:

```text
tests/cpython/fixtures/<bucket>/<lib>/<dimension>/<case>.py
```

This tree is generator/linter backed, so maximal one-case-per-file granularity
is intentional. A fixture name should brief the observable boundary without
opening the file.

For manifest-backed Rust gates, keep the same pairing visible:

```text
tests/governance/gates/<scope>/<gate>/manifest.toml
tests/governance/schema_gates/<scope_or_gate>_fixture_<issue>.rs
```

For mambalibs:

```text
tests/mambalibs/fixtures/<gate>/manifest.toml
tests/mambalibs/mambalibs_<gate>_fixture_<issue>.rs
```

## Current Migration Debt

The target CPython fixture layout is the dimension-directory shape above. Some
legacy monolith fixtures still exist as `<lib>/surface.py`,
`<lib>/behavior.py`, and `<lib>/errors.py`. They are discovered for backward
compatibility, but new fixtures should not use that shape. `ci_guard` locks the
current count as a ceiling so the debt can only stay flat or shrink.

The detailed CPython fixture mechanics live in
`tests/cpython/conventions/FIXTURE-LAYOUT.md`.
