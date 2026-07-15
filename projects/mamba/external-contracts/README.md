# mamba external-contracts

mamba's external contract IS CPython 3.12 behavior. The contract artifacts
already exist and are executable — this directory anchors them; it does not
duplicate them as prose.

## Global gates

| Contract | Artifact | Gate command |
|---|---|---|
| C1 functional parity | `tests/cpython/**` corpus (46k+; oracle = python3.12 byte-diff; xfail = acknowledged gaps) | `cargo test -p mamba --release --test conformance` (~3 min) |
| C2 performance | `tests/harness/cpython/config/perf/pins/*.toml` (external CPU/RSS, getrusage / `/usr/bin/time`, ratio asserted — `perf_pin.rs` D5.2) | perf pin sweep |

## Domain contract map (DDD — mirrors tech-design/README context map)

| Domain | Positive contract (must run & match oracle) | Negative contract (must reject) |
|---|---|---|
| type-system | — | `type/` dimension `*_wrong.py` walls; weakening a wall is a contract breach |
| object-model | `_regression/core/{class_system,mro_super,language,descriptors}`, `behavior/core/descr` | — |
| memory | `behavior|surface|type/std-libs/gc`, `_regression/core/stability` soaks; absence of hang/SIGTRAP corpus-wide | — |
| exceptions | `_regression/core/exception*`, `behavior/core/exceptions` | — |
| closures | `pep/572`, capture-introspection fixtures | — |
| stdlib (per module) | `behavior/std-libs/<mod>`, `errors/std-libs/<mod>`, `real_world/std-libs/<mod>` | — |

## Rules

- Dimension rule: fixtures under behavior/errors/real_world/surface/_regression/
  security/concurrency MUST run — a compile reject there is a type-system
  false positive by definition. Only `type/` fixtures are walls.
- A new WI's EC = name the fixture set in its TD's Verification contract.
  Write new fixtures only when the surface has no coverage.
- Gate readings are the only progress signal; per-fix evidence = before/after
  readings on the issue.
