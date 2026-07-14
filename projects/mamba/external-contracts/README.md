# mamba external-contracts

mamba's external contract IS CPython 3.12 behavior. The contract artifacts
already exist and are executable — this directory anchors them; it does not
duplicate them as prose.

| Contract | Artifact | Gate command |
|---|---|---|
| C1 functional parity | `tests/cpython/**` fixture corpus (46k+, oracle = python3.12 byte-diff; `type/` dimension = strict-type walls, xfail markers = acknowledged gaps) | `cargo test -p mamba --release --test conformance` (~3 min) |
| C2 performance | `tests/harness/cpython/config/perf/pins/*.toml` (external CPU/RSS via getrusage / `/usr/bin/time`, mamba/cpython ratio asserted — see `perf_pin.rs` D5.2) | perf pin sweep / `cargo test -p mamba --test conformance_pipeline` |
| Per-change EC | the fixture set named in the change's TD "Verification contract" section | direct fixture runs vs oracle + focused sweep |

Rules:

- A new WI's EC = name the fixture set (existing or added fixtures) in its TD's
  Verification contract. Only write new fixture files when the surface has no
  coverage; never restate fixture content as prose here.
- Gate readings are the only progress signal (goal frame); per-wave/per-fix
  evidence lives on the issue as before/after readings.
