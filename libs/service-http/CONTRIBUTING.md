# service-http Contributing

## Brief

Use this guide to change `libs/service-http`. The [README](README.md) owns the
library promises. The root [CONTRIBUTING.md](../../CONTRIBUTING.md) owns
repository-wide authoring rules.

## Authoritative Inputs

Read these sources in order for the part you change:

1. [README.md](README.md) for the public workflow, capabilities, sources, and
   gates.
2. [STATUS.md](STATUS.md) for current support and limits.
3. [ROADMAP.md](ROADMAP.md) for future outcomes and non-goals.
4. `libs/service-http/src/lib.rs` for the exported Rust surface.
5. The owning module under `libs/service-http/src/` and its colocated tests.
6. `libs/service-http/e2e/` and `Cargo.toml` for external behavior and the test
   target inventory.

## Local Workflow

Keep shared HTTP policy service-neutral. The app owns domain routes, domain
errors, authentication policy, listener settings, and TLS identity. This crate
can provide a reusable adapter without taking ownership of those decisions.

Test middleware at its real request and response boundary. Keep private keys,
credentials, and raw admission keys out of errors, logs, metrics, and timing
names.

When README, STATUS, or ROADMAP changes, treat them as one document set. Run
the deterministic check.

## Verification

### Product documents

```bash
python3 scripts/meta/test_readme_contract.py
python3 scripts/meta/test_project_docs_contract.py
python3 scripts/meta/project_docs_contract.py check libs/service-http --format json
```

### Library behavior

```bash
cargo test -p service-http
cargo test -p service-http --features otlp --test otlp_tracing
```

A docs-only change records only the document checks it ran.
