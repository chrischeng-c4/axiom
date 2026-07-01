---
id: deploy-cli-keep-spec-spec-gen-dockerfile-render
summary: >
  keep ships the standard service-archetype deploy-CLI verbs: `keep spec`
  (offline OpenAPI, the twin of /openapi.json, plus json-schema / shapes /
  fields views), `keep spec gen --lang ts|py|rust --out` (typed clients from
  keep's own OpenAPI via the shared libs/openapi-codegen), and `keep dockerfile
  render --variant source|release` (renders keep's committed Dockerfile /
  Dockerfile.release). All paths are offline: no server, no network beyond the
  output file.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: deploy-cli-keep-spec-spec-gen-dockerfile-render-contract
entry: cli_parse
nodes:
  cli_parse: { kind: start, label: "keep CLI parses spec and dockerfile subcommands" }
  spec_dispatch: { kind: process, label: "keep spec dispatches print vs gen" }
  spec_print: { kind: process, label: "keep spec resolves openapi/openapi-yaml/json-schema/shapes/fields via keep::spec" }
  spec_openapi: { kind: process, label: "keep::spec::openapi_json reuses http::routes::openapi the /openapi.json accessor" }
  spec_gen: { kind: process, label: "keep spec gen composes cclab_openapi_codegen::generate on keep::spec::openapi_json" }
  gen_write: { kind: process, label: "write generated ts/py/rust client files to --out" }
  dockerfile_render: { kind: process, label: "keep dockerfile render selects source or release variant" }
  render_source: { kind: process, label: "render_source strips ownership markers from the embedded Dockerfile" }
  render_release: { kind: process, label: "render_release substitutes keep@version into the embedded Dockerfile.release" }
  write_or_print: { kind: process, label: "write_or_print emits to --out or stdout" }
  stop: { kind: terminal, label: "offline artifact emitted with no server or network" }
edges:
  - { from: cli_parse, to: spec_dispatch }
  - { from: cli_parse, to: dockerfile_render }
  - { from: spec_dispatch, to: spec_print }
  - { from: spec_dispatch, to: spec_gen }
  - { from: spec_print, to: spec_openapi }
  - { from: spec_openapi, to: stop }
  - { from: spec_gen, to: gen_write }
  - { from: gen_write, to: stop }
  - { from: dockerfile_render, to: render_source }
  - { from: dockerfile_render, to: render_release }
  - { from: render_source, to: write_or_print }
  - { from: render_release, to: write_or_print }
  - { from: write_or_print, to: stop }
---
flowchart TD
    cli_parse([keep CLI parses spec and dockerfile subcommands]) --> spec_dispatch[keep spec dispatches print vs gen]
    cli_parse --> dockerfile_render[keep dockerfile render selects source or release variant]
    spec_dispatch --> spec_print[keep spec resolves openapi openapi-yaml json-schema shapes fields via keep spec]
    spec_dispatch --> spec_gen[keep spec gen composes cclab_openapi_codegen generate on keep spec openapi_json]
    spec_print --> spec_openapi[keep spec openapi_json reuses http routes openapi the openapi.json accessor]
    spec_openapi --> stop([offline artifact emitted with no server or network])
    spec_gen --> gen_write[write generated ts py rust client files to out]
    gen_write --> stop
    dockerfile_render --> render_source[render_source strips ownership markers from the embedded Dockerfile]
    dockerfile_render --> render_release[render_release substitutes keep version into the embedded Dockerfile.release]
    render_source --> write_or_print[write_or_print emits to out or stdout]
    render_release --> write_or_print
    write_or_print --> stop
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: deploy-cli-keep-spec-spec-gen-dockerfile-render-tests
requirements:
  keep_spec_openapi_offline:
    id: R1
    text: "keep spec emits keep's OpenAPI offline (openapi JSON default, openapi-yaml, json-schema, plus shapes/fields), and the openapi format matches the served /openapi.json inventory produced by http::routes::openapi."
    kind: behavior
    risk: high
    verify: test
  keep_spec_gen_clients:
    id: R2
    text: "keep spec gen --lang ts|py|rust --out generates a typed client by composing cclab_openapi_codegen::generate on keep's own OpenAPI, writing non-empty client files for each language with no second codegen path."
    kind: behavior
    risk: high
    verify: test
  keep_dockerfile_render:
    id: R3
    text: "keep dockerfile render --variant source|release renders keep's Dockerfiles: source reproduces the committed Dockerfile byte-for-byte, and release reproduces Dockerfile.release with the keep@version substituted."
    kind: behavior
    risk: medium
    verify: test
elements:
  keep_spec_module_tests:
    kind: test
    path: projects/keep/tests/spec_cli.rs
  keep_deploy_cli_tests:
    kind: test
    path: projects/keep/tests/deploy_cli.rs
relations:
  - { from: keep_spec_module_tests, verifies: keep_spec_openapi_offline }
  - { from: keep_spec_module_tests, verifies: keep_spec_gen_clients }
  - { from: keep_deploy_cli_tests, verifies: keep_spec_openapi_offline }
  - { from: keep_deploy_cli_tests, verifies: keep_spec_gen_clients }
  - { from: keep_deploy_cli_tests, verifies: keep_dockerfile_render }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "keep spec offline OpenAPI == served inventory"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "keep spec gen ts/py/rust via openapi-codegen"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "keep dockerfile render source/release"
      risk: medium
      verifymethod: test
    }
    element keep_spec_module_tests {
      type: "rs/#[test]"
    }
    element keep_deploy_cli_tests {
      type: "rs/#[test]"
    }
    keep_spec_module_tests - verifies -> R1
    keep_spec_module_tests - verifies -> R2
    keep_deploy_cli_tests - verifies -> R1
    keep_deploy_cli_tests - verifies -> R2
    keep_deploy_cli_tests - verifies -> R3
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/keep/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the cclab-openapi-codegen dependency (always linked, for keep spec gen) and make serde_yaml a non-optional dependency (keep spec --format openapi-yaml always works); drop dep:serde_yaml from the operator feature list."
  - path: projects/keep/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Declare pub mod spec so the offline self-description surface is reachable from the binary and tests."
  - path: projects/keep/src/spec.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "New offline spec module: openapi_json/openapi_yaml/json_schema_json reuse http::routes::openapi (the /openapi.json accessor); request_shapes is a keep operation cookbook and value_catalog mirrors the KvValue enum."
  - path: projects/keep/src/bin/keep.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add the Spec and Dockerfile commands (mirroring lumen): Spec dispatches openapi/openapi-yaml/json-schema/shapes/fields and a gen subcommand that composes cclab_openapi_codegen::generate on keep::spec::openapi_json; Dockerfile render renders the source/release variants with marker stripping + keep@version substitution; add clap parse tests."
  - path: projects/keep/Dockerfile.release
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Reconcile the committed KEEP_VERSION / image tag to keep's current version so keep dockerfile render --variant release reproduces the committed file byte-for-byte (render is the source of truth)."
  - path: projects/keep/tests/spec_cli.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Assert the keep::spec surface: openapi_json is valid OpenAPI 3.x with keep data-plane paths, openapi_yaml parses, json_schema exposes component schemas, request_shapes carry request bodies, value_catalog matches the KvValue variants, and cclab_openapi_codegen::generate composes on keep's OpenAPI for ts/py/rust."
  - path: projects/keep/tests/deploy_cli.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Drive the compiled keep binary: keep spec emits the same OpenAPI inventory as http::routes::openapi (the /openapi.json source), keep spec gen --lang ts writes client files to a temp dir, and keep dockerfile render --variant source|release reproduces the committed Dockerfiles (with keep@version substitution)."
```
