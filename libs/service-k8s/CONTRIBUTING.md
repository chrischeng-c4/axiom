# service-k8s Contributing

## Brief

Use this guide to change `libs/service-k8s`. The [README](README.md) owns the
library promises. The root [CONTRIBUTING.md](../../CONTRIBUTING.md) owns
repository-wide authoring rules.

## Authoritative Inputs

Read these sources in order for the part you change:

1. [README.md](README.md) for the public workflow, capabilities, sources, and
   gates.
2. [STATUS.md](STATUS.md) for current support and limits.
3. [ROADMAP.md](ROADMAP.md) for shared future outcomes and non-goals.
4. `libs/service-k8s/src/lib.rs` for the exported Rust surface.
5. The owning semantic module under `libs/service-k8s/src/` and its colocated
   tests.
6. `libs/service-k8s/e2e/` and `Cargo.toml` for external behavior and the test
   target inventory.

For identity work, read `render/projected_token.rs` and `render/rbac.rs`
together. The first keeps token mount and read paths aligned. The second owns
RBAC object shape, not an app's permission meaning.

## Local Workflow

This library has no app phase ladder. Make one bounded library change with its
test. Put externally observable behavior in `libs/service-k8s/e2e/` and declare
the target in `Cargo.toml`. Keep internal rules in colocated unit tests.

Keep shared mechanisms service-neutral. A caller supplies its CRD schema,
access resource mapping, domain topology, protected paths, health mapping, and
external provider policy. Do not move an app policy into this crate only
because two controllers use Kubernetes.

For placement, the app supplies the failure-domain group and quorum meaning.
For certificate work, the app supplies identities and readiness. For public
trust, this library handles only public ConfigMap data and ownership. For
StatefulSet rollout, never treat a PDB as an update gate. A PDB controls
voluntary eviction.

When README, STATUS, or ROADMAP changes, treat them as one document set. Run
the deterministic check.

## Verification

### Product documents

```bash
python3 scripts/meta/test_readme_contract.py
python3 scripts/meta/test_project_docs_contract.py
python3 scripts/meta/project_docs_contract.py check libs/service-k8s --format json
```

### Library behavior

```bash
cargo test -p service-k8s
```

Run the full library gate before claiming a behavior or public API change is
complete. A docs-only change records only the document checks it actually ran.
