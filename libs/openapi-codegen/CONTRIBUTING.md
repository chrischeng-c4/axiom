# openapi-codegen Contributing

## Brief

Use this guide to change `libs/openapi-codegen`. The [README](README.md) owns
the library promises. The root [CONTRIBUTING.md](../../CONTRIBUTING.md) owns
repository-wide authoring rules.

## Authoritative Inputs

Read these sources in order for the part you change:

1. [README.md](README.md) for the public workflow, capabilities, sources, and
   gates.
2. [STATUS.md](STATUS.md) for current support and limits.
3. [ROADMAP.md](ROADMAP.md) for future outcomes and non-goals.
4. `libs/openapi-codegen/src/ir/` for operations, media types, and the
   language-neutral schema contract.
5. The owning emitter under `libs/openapi-codegen/src/emit/` and its tests for
   language-specific request, response, error, and type behavior.
6. `libs/openapi-codegen/src/target.rs` and `lib.rs` for target, dependency,
   and artifact
   compatibility.

## Local Workflow

Keep the generated client surface service-neutral. An app supplies its API
document, audience, credential source, and identity policy. Do not add KSA,
Fleet, or service-specific RBAC policy to this library.

When a shared IR rule changes, test all three emitters. Do the same for media
types, streaming, errors, schema types, and target dependencies. When
request-time auth changes, prove that TypeScript, Python, and Rust call the
provider for every request and stop before transport on provider failure.

Keep retry mechanics separate from service retry policy. The generator can
carry operation metadata and provide hooks. The app decides whether a read or
write is safe. A required cross-language gate must fail when a selected
toolchain is missing.

When README, STATUS, or ROADMAP changes, treat them as one document set. Run
the deterministic check.

## Verification

### Product documents

```bash
python3 scripts/meta/test_readme_contract.py
python3 scripts/meta/test_project_docs_contract.py
python3 scripts/meta/project_docs_contract.py check libs/openapi-codegen --format json
```

### Library behavior

```bash
cargo test -p openapi-codegen
```

A docs-only change records only the document checks it ran.
