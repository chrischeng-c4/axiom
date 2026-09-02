# server-http Contributing

## Brief

Use this guide to change `libs/server-http`. The [README](README.md) owns the
library promises. The root [CONTRIBUTING.md](../../CONTRIBUTING.md) owns
repository-wide authoring rules.

## Authoritative Inputs

Read these sources in order for the part you change:

1. [README.md](README.md) for the public workflow, capabilities, sources, and
   gates.
2. [STATUS.md](STATUS.md) for current support and limits.
3. [ROADMAP.md](ROADMAP.md) for explicit non-goals.
4. `libs/server-http/src/lib.rs` for listener composition and reporting.
5. `libs/server-http/src/tls.rs` for TLS selection and refusal behavior.
6. `libs/server-http/e2e/` and `Cargo.toml` for external behavior and the test
   target inventory.
7. The public contracts of `server-tcp`, `server-lifecycle`, `transport-h2c`,
   `service-http`, and `peer-tls` for each composition boundary.

## Local Workflow

Keep this crate at the listener boundary. It can compose TCP admission,
per-connection HTTP serving, lifecycle drain, and TLS handshakes. It must not
take ownership of routes, middleware policy, certificate parsing, or identity
policy.

Keep TLS rotation fail-closed. Read the active configuration once per accept.
Do not replace it under an existing connection or downgrade to cleartext when
no valid material exists.

When README, STATUS, or ROADMAP changes, treat them as one document set. Run
the deterministic check.

## Verification

### Product documents

```bash
python3 scripts/meta/test_readme_contract.py
python3 scripts/meta/test_project_docs_contract.py
python3 scripts/meta/project_docs_contract.py check libs/server-http --format json
```

### Library behavior

```bash
cargo test -p server-http
```

A docs-only change records only the document checks it ran.
