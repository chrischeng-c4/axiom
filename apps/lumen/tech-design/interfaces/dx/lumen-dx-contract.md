---
id: lumen-dx-contract
title: Lumen Developer and Agent Experience Contract
project: lumen
capability_refs:
  - id: agent-task-navigation
    role: primary
    claim: lumen-llm-v2-task-navigation
    coverage: full
    rationale: Typed offline task navigation is the capability's primary contract.
  - id: cli-interface
    role: primary
    claim: lumen-llm-v2-task-navigation
    coverage: full
    rationale: The same typed contract defines the public `lumen llm` CLI surface.
  - id: developer-agent-experience
    role: primary
    claim: lumen-llm-v2-task-navigation
    coverage: full
    rationale: The source-backed manifest and runbooks are the DX task-navigation contract.
fill_sections:
  - dx-contract
  - unit-test
  - changes
---

# Lumen Developer and Agent Experience Contract

## DX Contract
<!-- type: dx-contract lang: yaml -->

```yaml
version: 1
authority:
  runtime: "Rust FieldType, runtime validation, CLI registration, and aw.toml traits establish structural behavior."
  decisions: "This dx-contract owns task classification, narrative, preconditions, typed inputs, templates, and artifact selection."
  verification: "External contracts prove generated artifacts and public runtime behavior remain aligned."
field_catalog:
  source: "FieldType::capabilities"
  rules:
    text: "BM25 only; range and sort are unsupported."
    keyword: "Exact match plus byte/lexicographic range and sort."
    number: "Numeric range and sort."
artifacts:
  field_catalog: "lumen spec --fields"
  task_manifest: "lumen llm --topic outline --format json"
  runbooks: "lumen llm --topic <id> [--format md|json]"
  openapi: "lumen spec --format openapi, /openapi.json, clients/openapi.json, and spec gen share canonical bytes"
llm_protocol:
  protocol: cclab.llm.v2
  tasks:
    - id: local-search
      use_when: "inspect the offline Lumen search contract before issuing a request"
      requires: []
      reads: ["lumen spec --fields", "lumen spec --shapes"]
      produces: ["field and query selection"]
      risk: inspect
      purpose: "Select fields and query operations from the installed binary without a running server."
      preconditions: ["The installed lumen binary is available."]
      inputs: []
      constraints: ["Only fully-bound commands are runnable; templates require their typed inputs.", "Use lumen spec for exact wire shapes and --help for CLI grammar."]
      instruction: "Read the local field catalogue."
      command: "lumen spec --fields"
      verification: ["Confirm the chosen operation is advertised by the field catalogue."]
    - id: model-schema
      use_when: "declare or review a collection schema"
      requires: ["A reachable Lumen base URL and a collection identifier."]
      reads: ["lumen spec --fields"]
      produces: ["collection schema and index request"]
      risk: local_write
      purpose: "Choose text for BM25 long text; choose keyword for exact, range, and sort semantics."
      preconditions: ["Read the field catalogue before creating the collection schema."]
      inputs:
        - { name: url, type: url, description: "Lumen base URL", required: true }
        - { name: collection, type: string, description: "collection id", required: true }
        - { name: item, type: string, description: "EXTERNAL_ID:FIELD=VALUE index item", required: true }
      constraints: ["text cannot be sorted or ranged; keyword and number can."]
      instruction: "Index one validated field value after schema creation."
      command_template: "lumen query index --url {url} --collection {collection} --item {item}"
      verification: ["Confirm every selected operation is advertised for the selected field type."]
    - id: select-query
      use_when: "choose a search, filter, range, sort, kNN, or duplicate query"
      requires: []
      reads: ["lumen spec --shapes", "lumen spec --fields"]
      produces: ["supported request body"]
      risk: inspect
      purpose: "Use the canonical query cookbook before composing an HTTP or CLI query."
      preconditions: ["The installed lumen binary is available."]
      inputs: []
      constraints: ["Do not infer unsupported field operations from a string-like type name."]
      instruction: "Read the query-shape cookbook."
      command: "lumen spec --shapes"
      verification: ["Confirm the request body and field operations are both advertised."]
    - id: integrate-source-db
      use_when: "connect Postgres, AlloyDB, CDC, or an outbox to Lumen"
      requires: ["An authoritative source system and an output directory."]
      reads: ["lumen spec --format openapi"]
      produces: ["adapter contract"]
      risk: local_write
      purpose: "Keep the source database authoritative; deliver idempotent writes through Lumen's public HTTP contract."
      preconditions: ["Choose an idempotency key before wiring a CDC or outbox adapter."]
      inputs:
        - { name: lang, type: enum, description: "ts, py, or rust", required: true }
        - { name: out, type: path, description: "generated client output directory", required: true }
      constraints: ["Generated clients are derived from canonical OpenAPI; no hand-maintained SDK wrapper."]
      instruction: "Generate the client used by the adapter boundary."
      command_template: "lumen spec gen --lang {lang} --out {out}"
      verification: ["Confirm generated client source is produced from the canonical OpenAPI document."]
    - id: authenticate
      use_when: "configure bearer-token access or inspect the token registry shape"
      requires: []
      reads: ["lumen spec --format json-schema"]
      produces: ["auth configuration"]
      risk: inspect
      purpose: "Read the operational token-registry schema before creating secrets or client headers."
      preconditions: ["Know the intended collection and minimum role."]
      inputs: []
      constraints: ["Never put a bearer token in a generated runbook or committed client source."]
      instruction: "Read the token-registry schema."
      command: "lumen spec --format json-schema"
      verification: ["Confirm the selected role covers only the intended collection scope."]
    - id: connect-kubernetes
      use_when: "run a command through a temporary Kubernetes port-forward"
      requires: ["kubectl access to the target namespace and service."]
      reads: ["lumen connect --help"]
      produces: ["authenticated local connection"]
      risk: remote_write
      purpose: "Use lumen connect so port-forward lifecycle and bearer-token lookup are bounded to one command."
      preconditions: ["Resolve the target namespace and service name."]
      inputs:
        - { name: namespace, type: string, description: "Kubernetes namespace", required: true }
        - { name: service, type: string, description: "Lumen service name", required: true }
        - { name: command, type: command, description: "wrapped local command", required: true }
      constraints: ["The wrapped command is explicit; LLM navigation does not execute it automatically."]
      instruction: "Run an explicit client command through the temporary port-forward."
      command_template: "lumen connect --namespace {namespace} --service {service} -- {command}"
      verification: ["Confirm the wrapped command observes LUMEN_URL and terminates with the port-forward."]
    - id: deploy-kubernetes
      use_when: "render image, CRD, operator, or Lumen instance manifests"
      requires: ["A deployment profile and output path."]
      reads: ["lumen k8s --help"]
      produces: ["deployment artifact"]
      risk: local_write
      purpose: "Render one deployment layer at a time; image creation remains outside the Kubernetes command group."
      preconditions: ["Choose the deployment layer before rendering."]
      inputs:
        - { name: profile, type: enum, description: "dev, staging, prod, or template", required: true }
        - { name: out, type: path, description: "rendered manifest path", required: true }
      constraints: ["Do not collapse CRD, operator, and instance artifacts into one command."]
      instruction: "Render an instance manifest for the chosen profile."
      command_template: "lumen k8s instance render --profile {profile} --out {out}"
      verification: ["Confirm the rendered artifact corresponds to exactly one deployment layer."]
    - id: backup-restore
      use_when: "create or restore an administrative Lumen snapshot"
      requires: ["Admin authorization and a supported backup destination."]
      reads: ["lumen backup --help"]
      produces: ["backup or restore evidence"]
      risk: remote_write
      purpose: "Use the admin backup surface with an admin bearer token; a local snapshot is not a replacement for scheduled object storage backups."
      preconditions: ["Confirm the destination URI and retention policy."]
      inputs:
        - { name: url, type: url, description: "Lumen base URL", required: true }
        - { name: destination, type: url, description: "backup destination URL", required: true }
      constraints: ["A restore changes live service state and requires an explicit operator decision."]
      instruction: "Create an administrative backup at the supplied destination."
      command_template: "lumen backup --url {url} --dest {destination}"
      verification: ["Confirm backup output reports its object location and explicit restore follow-up."]
    - id: generate-client
      use_when: "generate a typed Rust, Python, or TypeScript client"
      requires: ["An output directory and target language."]
      reads: ["lumen spec --format openapi"]
      produces: ["typed client source"]
      risk: local_write
      purpose: "Generate from the canonical OpenAPI document; never hand-maintain an SDK wrapper."
      preconditions: ["Choose the consuming language and an empty or dedicated output directory."]
      inputs:
        - { name: lang, type: enum, description: "ts, py, or rust", required: true }
        - { name: out, type: path, description: "client output directory", required: true }
      constraints: ["The generated output is disposable and must be regenerated after an API release."]
      instruction: "Generate the selected typed client."
      command_template: "lumen spec gen --lang {lang} --out {out}"
      verification: ["Confirm the generated entrypoint is the CLI-reported next artifact."]
    - id: diagnose
      use_when: "inspect readiness, metrics, OpenAPI, or version data from a running service"
      requires: ["A reachable Lumen base URL."]
      reads: ["/healthz", "/readyz", "/metrics", "/openapi.json"]
      produces: ["diagnostic evidence"]
      risk: inspect
      purpose: "Start with the standard operational surface; do not infer liveness from a data-plane request."
      preconditions: ["Identify whether a bearer token is required for the collection listing."]
      inputs:
        - { name: url, type: url, description: "Lumen base URL", required: true }
      constraints: ["Readiness and liveness are separate signals; do not treat a 200 healthz as write readiness."]
      instruction: "List visible collections after checking operational endpoints."
      command_template: "lumen query collections list --url {url}"
      verification: ["Record health, readiness, metrics, and canonical OpenAPI evidence separately."]
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
kind: requirementDiagram
id: lumen-dx-contract-tests
---
requirementDiagram
  requirement task_manifest {
    id: DX-1
    text: "All ten task ids parse as typed LLM v2 runbooks."
    risk: medium
    verifymethod: test
  }
  requirement canonical_openapi {
    id: DX-2
    text: "Offline, live, snapshot, and generated clients consume canonical OpenAPI bytes."
    risk: high
    verifymethod: test
  }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/dx.rs
    action: modify
    section: dx-contract
    impl_mode: hand-written
    description: "Bind the typed TD task decisions to runtime FieldType capabilities and deterministic LLM v2 rendering; the missing cross-source emitter remains tracked by #1683."
  - path: apps/lumen/tests/spec_cli.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Verify task-manifest/runbook typing, field capability projection, and canonical OpenAPI bytes from the public CLI surface."
```
