# transport-h2c Contributing

## Brief

Use this guide to change `libs/transport-h2c`. The [README](README.md) owns the
library promises. The root [CONTRIBUTING.md](../../CONTRIBUTING.md) owns
repository-wide authoring rules.

## Authoritative Inputs

Read these sources in order for the part you change:

1. [README.md](README.md) for the public workflow, capabilities, sources, and
   gates.
2. [STATUS.md](STATUS.md) for current support and limits.
3. [ROADMAP.md](ROADMAP.md) for explicit non-goals.
4. `libs/transport-h2c/src/lib.rs` for the public client and pool surface.
5. `manager.rs`, `conn.rs`, and `error.rs` for managed connection behavior.
6. `server.rs`, `e2e/`, and `Cargo.toml` for the optional server and test
   inventory.

## Local Workflow

Keep the crate transport-focused. It can manage h2c connections and one
accepted stream. It must not bind a listener, choose TLS identity, or define
application routes.

Treat mutation replay as unsafe after dispatch. Add a failure test whenever a
change touches GOAWAY, retry, drain, deadline, or ambiguity accounting.

When README, STATUS, or ROADMAP changes, treat them as one document set. Run
the deterministic check.

## Verification

### Product documents

```bash
python3 scripts/meta/test_readme_contract.py
python3 scripts/meta/test_project_docs_contract.py
python3 scripts/meta/project_docs_contract.py check libs/transport-h2c --format json
```

### Library behavior

```bash
cargo test -p transport-h2c
```

A docs-only change records only the document checks it ran.
