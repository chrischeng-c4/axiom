# service-auth Contributing

## Brief

Use this guide to change `libs/service-auth`. The [README](README.md) owns the
library promises. The root [CONTRIBUTING.md](../../CONTRIBUTING.md) owns
repository-wide authoring rules.

## Authoritative Inputs

Read these sources in order for the part you change:

1. [README.md](README.md) for the public workflow, capabilities, sources, and
   gates.
2. [STATUS.md](STATUS.md) for current support and limits.
3. [ROADMAP.md](ROADMAP.md) for future outcomes and non-goals.
4. `libs/service-auth/src/lib.rs` and the owning module under
   `libs/service-auth/src/` for the Rust contract.
5. Colocated tests for review, failure, rotation, and redaction behavior.

## Local Workflow

Keep identity transport and verification mechanisms service-neutral. The app
supplies its audience, protected route policy, and Kubernetes resource mapping.
Do not add a Fleet or service-specific RBAC policy to this crate.

Keep the current Rust JWT preflight separate from a portable opaque-token
source. A client-side check can fail early, but it never replaces server-side
TokenReview.

Add a failure test before changing a fail-closed or redaction rule. A test
credential must be recognizable so the test can prove that no error or event
contains it.

When README, STATUS, or ROADMAP changes, treat them as one document set. Run
the deterministic check.

## Verification

### Product documents

```bash
python3 scripts/meta/test_readme_contract.py
python3 scripts/meta/test_project_docs_contract.py
python3 scripts/meta/project_docs_contract.py check libs/service-auth --format json
```

### Library behavior

```bash
cargo test -p service-auth
```

A docs-only change records only the document checks it ran.
