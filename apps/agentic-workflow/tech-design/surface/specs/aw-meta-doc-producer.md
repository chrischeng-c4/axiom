---
id: aw-meta-doc-producer
summary: Provide one marker-safe init, sync, and read-only check control plane for repository and project META-docs.
fill_sections: [scenarios, schema, logic, unit-test, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: meta-doc-init-sync-check
    claim: meta-doc-init-sync-check
    coverage: full
    rationale: "Agent-first iteration must begin from deterministic META-doc skeletons and one non-conflicting producer/checker surface."
---
<!-- HANDWRITE-BEGIN gap="missing-generator:logic:aw-meta-doc-producer" tracker="#1498" reason="Marker preservation and legacy projector convergence require an explicit owner-approved producer registry." -->

# META-Doc Producer Control Plane

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: aw-meta-doc-producer-scenarios
scenarios:
  - id: S1
    title: initialize a fresh repository and project
    when:
      - "an agent runs aw meta init with a project path or configured project"
    then:
      - "repo AGENTS.md, CLAUDE.md, README.md, and CONTRIBUTING.md skeletons exist"
      - "project README.md, CONTRIBUTING.md, and CAPABILITIES.md skeletons exist"
      - "stdout names an executable aw meta check command"
  - id: S2
    title: synchronize marker-owned blocks without touching human bytes
    given:
      - "human content exists before or after an AW marker pair"
    when:
      - "aw meta sync reconciles the document"
    then:
      - "only bytes from the owned start marker through the owned end marker change"
      - "a second sync is byte-idempotent"
  - id: S3
    title: check detects exact drift without writing
    given:
      - "one or more registered managed blocks are stale or malformed"
    when:
      - "aw meta check runs"
    then:
      - "the command exits non-zero"
      - "each finding names the file, block id, and aw meta sync remediation"
      - "no file byte changes"
  - id: S4
    title: root agent guidance has one semantic source
    when:
      - "CLAUDE.md and AGENTS.md are initialized or synchronized"
    then:
      - "both derive from the compiled CLAUDE template"
      - "AGENTS.md differs only by the explicit Codex whitelist projection"
  - id: S5
    title: legacy initialization delegates instead of competing
    when:
      - "aw new installs or refreshes project assets"
    then:
      - "the same META_DOC_PRODUCERS registry writes all single-product META-docs"
      - "legacy projects-table, trait-table, and ownership-matrix blocks use the registry"
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: aw-meta-doc-producer
type: object
additionalProperties: false
required: [schema_version, action, status, repository_root, projects, changes, findings, next, terminal]
properties:
  schema_version:
    const: aw.meta.v1
  action:
    enum: [meta_init, meta_sync, meta_check]
  status:
    enum: [initialized, synchronized, clean, drift]
  repository_root:
    type: string
  repository_is_product:
    type: boolean
  projects:
    type: array
    items: { type: string }
  changes:
    type: array
    items:
      type: object
      additionalProperties: false
      required: [path, block, status]
      properties:
        path: { type: string }
        block: { type: string, minLength: 1 }
        status: { enum: [created, updated, unchanged] }
  findings:
    type: array
    items:
      type: object
      required: [code, path, message, remediation]
      properties:
        code:
          enum: [managed_block_missing, managed_block_stale, managed_block_malformed, meta_doc_missing, meta_doc_unreadable, meta_doc_repository_unreadable, meta_doc_section_missing, project_agent_doc_forbidden, root_capabilities_requires_product, unexpected_root_meta_doc]
        path: { type: string }
        message: { type: string }
        remediation: { type: string }
  next:
    oneOf:
      - type: "null"
      - type: object
        required: [command]
        properties:
          command: { type: string, pattern: '^aw meta (check|sync)' }
  terminal:
    oneOf:
      - type: "null"
      - type: object
        required: [status]
        properties:
          status: { const: done }
```

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: aw-meta-doc-producer-loop
entry: scope
nodes:
  scope: { kind: start, label: "Resolve repo/product/project scope" }
  registry: { kind: process, label: "Walk one META_DOC_PRODUCERS registry" }
  existing: { kind: decision, label: "Managed marker pair exists?" }
  reconcile: { kind: process, label: "Render and replace only owned marker span" }
  skeleton: { kind: process, label: "Create or append canonical skeleton block" }
  mode: { kind: decision, label: "Check mode?" }
  report: { kind: process, label: "Report exact file/block drift without writing" }
  write: { kind: process, label: "Write reconciled bytes" }
  validate: { kind: process, label: "Run ownership matrix validation" }
  next: { kind: terminal, label: "Emit executable next or done terminal" }
edges:
  - { from: scope, to: registry }
  - { from: registry, to: existing }
  - { from: existing, to: reconcile, label: "yes" }
  - { from: existing, to: skeleton, label: "no and required" }
  - { from: existing, to: validate, label: "no and optional" }
  - { from: reconcile, to: mode }
  - { from: skeleton, to: mode }
  - { from: mode, to: report, label: "yes" }
  - { from: mode, to: write, label: "no" }
  - { from: report, to: validate }
  - { from: write, to: validate }
  - { from: validate, to: next }
---
flowchart TD
    scope([Resolve repo/product/project scope]) --> registry[Walk one META_DOC_PRODUCERS registry]
    registry --> existing{Managed marker pair exists?}
    existing -->|yes| reconcile[Render and replace only owned marker span]
    existing -->|no and required| skeleton[Create or append canonical skeleton block]
    existing -->|no and optional| validate[Run ownership matrix validation]
    reconcile --> mode{Check mode?}
    skeleton --> mode
    mode -->|yes| report[Report exact file/block drift without writing]
    mode -->|no| write[Write reconciled bytes]
    report --> validate
    write --> validate
    validate --> next([Emit executable next or done terminal])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: aw-meta-doc-producer-unit-test
coverage_kind: unit
evidence:
  command: "cargo test -p agentic-workflow --lib cli::meta::tests:: -- --nocapture"
---
requirementDiagram
  requirement fresh_init {
    id: UT1
    text: "fresh repo and project fixtures receive all layer skeletons"
    risk: high
    verifymethod: test
  }
  requirement idempotent_sync {
    id: UT2
    text: "sync is byte-idempotent and preserves human-owned regions"
    risk: high
    verifymethod: test
  }
  requirement drift_repair {
    id: UT3
    text: "tampered managed blocks are detected by check and repaired by sync"
    risk: high
    verifymethod: test
  }
  requirement read_only_check {
    id: UT4
    text: "check never mutates a drifting document"
    risk: high
    verifymethod: test
  }
  requirement shared_agent_projection {
    id: UT5
    text: "aw new and aw meta use one root-agent template plus runtime whitelist"
    risk: medium
    verifymethod: test
  }
  requirement chain_surface {
    id: UT6
    text: "meta init, sync, and check are classified in the real CLI leaf registry"
    risk: medium
    verifymethod: test
  }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/meta.rs
    action: create
    section: logic
    impl_mode: codegen
    description: Register and implement the init/sync/check producer engine, marker-preserving reconciliation, structured output, and focused fixtures.
  - path: apps/agentic-workflow/src/cli/commands.rs
    action: modify
    section: logic
    impl_mode: codegen
    description: Register and dispatch the public aw meta namespace.
  - path: apps/agentic-workflow/src/cli/init.rs
    action: modify
    section: logic
    impl_mode: codegen
    description: Delete the independent root-doc projectors and delegate aw new to the shared META-doc registry.
  - path: apps/agentic-workflow/src/cli/doc_mirror.rs
    action: modify
    section: schema
    impl_mode: codegen
    description: Include aw meta in the generated workflow command table.
  - path: apps/agentic-workflow/src/cli/chain.rs
    action: modify
    section: schema
    impl_mode: codegen
    description: Classify meta init/sync as mutating core verbs and meta check as read-only.
  - path: AGENTS.md
    action: modify
    section: scenarios
    impl_mode: hand-written
    description: Project the new public META-doc surface into Codex guidance.
  - path: CLAUDE.md
    action: modify
    section: scenarios
    impl_mode: hand-written
    description: Project the new public META-doc surface into Claude guidance.
  - path: apps/agentic-workflow/templates/cli/mainthread/CLAUDE.md.tmpl
    action: modify
    section: scenarios
    impl_mode: hand-written
    description: Keep the semantic root-agent template aligned with both runtime projections.
  - path: CONTRIBUTING.md
    action: modify
    section: schema
    impl_mode: hand-written
    description: Name aw meta as the sole writer/checker for the ownership matrix contract.
  - path: apps/agentic-workflow/README.md
    action: modify
    section: scenarios
    impl_mode: hand-written
    description: Start greenfield and brownfield lifecycle guidance at the META-doc control plane.
  - path: apps/agentic-workflow/CONTRIBUTING.md
    action: modify
    section: schema
    impl_mode: hand-written
    description: Classify aw meta as part of the core CLI lifecycle surface.
  - path: apps/agentic-workflow/CAPABILITIES.md
    action: modify
    section: scenarios
    impl_mode: hand-written
    description: Register issue 1498 and its verification evidence under the agent-first CLI capability.
```
<!-- HANDWRITE-END -->
