# Lumen Contributing

## Brief

Use this guide to change `apps/lumen`. The [README](README.md) owns product
promises. The root [CONTRIBUTING.md](../../CONTRIBUTING.md) owns repository-wide
authoring rules.

## Authoritative Inputs

Read these sources in order for the part you change:

1. [README.md](README.md) for product purpose, public workflow, capabilities,
   sources, and gates.
2. [STATUS.md](STATUS.md) for current support and limits.
3. [ROADMAP.md](ROADMAP.md) for future outcomes and non-goals.
4. [Architecture](ARCHITECTURE.md) for source-of-truth, caller, data-plane,
   control-plane, and shared-library boundaries.
5. [Indexing](docs/indexing.md) for schema, write, durability, rebuild, and
   activation semantics.
6. [Querying](docs/querying.md) for selection, scoring, result, facet, metric,
   limit, and hydration semantics.
7. [0.5 search migration](docs/migration-0.5-search.md) for the versioned
   compatibility window and caller actions.
8. [Protocol](docs/protocol.md) for the canonical HTTP and runtime-behavior
   source map.
9. [Generated clients](clients/README.md) for the artifact model, language
   matrix, connection inputs, and current client limits.
10. [GKE](docs/gke.md) for support tiers, topology, Kubernetes-native
    placement, the Standard Regional profile, and its acceptance contract.
11. [Client integration](docs/client-integration.md) for connection profiles,
    workload projection, request mechanics, and source hydration helpers.
12. `apps/lumen/src/` and `apps/lumen/e2e/` for behavior and executable
   contracts.
13. `apps/lumen/src/operator/crd.rs` and
   `apps/lumen/src/operator/fleet.rs` for the generated Kubernetes API.
14. [Deployment](docs/deployment.md), [configuration](docs/configuration.md), and
   [authentication](docs/authentication.md) for maintained operating context.

Generated `apps/lumen/k8s/operator/crd.yaml` follows the Rust CRD types. Do not
edit generated schema as a substitute for changing its source.

## Local Workflow

Application behavior follows the repository phase ladder `wi → e2e → impl`.
Write the failing black-box case in `apps/lumen/e2e/` before changing
`apps/lumen/src/`. Use `/aw-e2e-for-wi` for the e2e phase and
`/aw-impl-for-wi` for the impl phase when the work is tied to a work item.

Keep Lumen-specific search, schema, shard, and health policy in `apps/lumen`.
Put reusable Kubernetes mechanisms in `libs/service-k8s` after their shared
contract is defined. Do not move Lumen's protected reshard fields into a
generic Fleet policy.

Keep the Lumen audience, token path, anonymous routes, typed access API, and
SubjectAccessReview resource mapping in this app. Put token and review
mechanisms in `libs/service-auth`, generic request-time client providers in
`libs/openapi-codegen`, and Kubernetes object mechanics in
`libs/service-k8s`. Never place a bearer token in a CRD, argument, environment
variable, status, Event, or log.

When a Rust CRD type changes, regenerate the checked-in schema:

```bash
cargo run -p lumen --features operator --bin lumen -- \
  k8s crd render --out apps/lumen/k8s/operator
```

When README, STATUS, ROADMAP, protocol, generated-client, indexing, querying,
GKE, client-integration, or migration documentation changes, update the adopted
product-document set when its claims or links overlap. Keep current behavior
separate from target behavior. Run the checker from `apps/lumen`, not from a
nested supporting-doc directory. Then use `$project-readme-check` after the
deterministic command passes.

## Verification

### Product documents

```bash
python3 scripts/meta/test_readme_contract.py
python3 scripts/meta/test_project_docs_contract.py
python3 scripts/meta/project_docs_contract.py check apps/lumen --format json
```

The first two commands test the validators. The third checks the three core
documents plus every adopted protocol, generated-client, indexing, querying,
GKE, client-integration, and linked migration guide. After it passes,
`$project-readme-check` runs one clean-context reader over those exact files. A
docs-only change does not claim that product gates ran.

### Product behavior

| Gate | Command |
|---|---|
| default features and refusal paths | `cargo test -p lumen` |
| operator and delegated-auth e2e targets | `cargo test -p lumen --features "operator delegated-auth"` |
| release feature set | `cargo test -p lumen --locked --features release --test release_feature_set` |
| landed-main release candidate oracle | `cargo test -p lumen --test release_candidate` |
| protected-tag promotion oracle | `cargo test -p lumen --test release_promotion` |
| full candidate verifier | `apps/lumen/scripts/verify-release-candidate.sh --repo chrischeng-c4/axiom --version <version> --commit <commit> --run-id <id> --run-attempt <attempt> --manifest <path> --manifest-sidecar <path> --artifacts-dir <path> --image <image> --candidate-tag <tag> --amd64-digest <digest> --arm64-digest <digest> --mode full` |
| public release verifier | `apps/lumen/scripts/verify-release-artifacts.sh --repo chrischeng-c4/axiom --tag lumen@<version> --commit <commit> --candidate-run-id <id> --mode public --output <path>` |
| standalone container bind smoke | `bash apps/lumen/scripts/standalone-container-smoke.sh bind` |

All eight rows are required for a full Lumen behavior claim. None is a superset
of the others. Do not replace the second or third row with `--all-features`;
enabling `jieba` changes the behavior that the fallback test is meant to check.

Run narrower named tests during development. Run each of the eight gates at its
applicable candidate or promotion lifecycle stage before claiming that a release
is complete. Run the public verifier only after publication. Use the exact
capability gates in the README when the claim is narrower or requires a
live-cluster script.
