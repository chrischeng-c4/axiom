---
id: aw-rust-layout-profile
summary: "Define Rust generator placement and ownership rules."
fill_sections: [overview, schema, scenarios, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# AW Rust Layout Profile

## Overview
<!-- type: overview lang: markdown -->

Rust code generation needs a concrete placement contract, not only general
guidance about coherent modules. The Rust layout profile maps TD section types
and generated artifacts to crate roots, module files, adapters, CLI surfaces,
generated implementation modules, compatibility shims, migrations, and tests.

The profile is intentionally ownership-oriented. Domain modules own business
rules and pure model behavior; CLI modules own argument parsing, envelope
formatting, and user-facing command orchestration; adapter modules own external
systems such as GitHub, GitLab, git, process execution, filesystem stores, and
network APIs. Generated modules must not blur these boundaries just because a
single TD section references several concerns.

Dogfood names from the Score/SDD era map to current Agentic Workflow surfaces:
`projects/score` corresponds to the repo-local CLI and work-item lifecycle
surface under `projects/agentic-workflow/src/cli/`, while `projects/sdd`
corresponds to the spec compiler, TD AST, generator, and validation surfaces
under `projects/agentic-workflow/src/{td_ast,generate,validate,validator}/`.
The examples keep the historical names because existing issues and specs still
use them as shorthand.

## Schema
<!-- type: schema lang: yaml -->

```yaml
rust_layout_profile:
  id: aw-rust-layout-profile
  language: rust
  crate_policy:
    lib_rs:
      owns:
        - public module declarations
        - stable crate-level re-exports
        - feature-gated module exposure
      must_not_own:
        - business logic
        - CLI command bodies
        - backend protocol calls
        - generated algorithm bodies
    root_modules:
      rule: "One top-level module per ownership family."
      examples:
        cli: "repo-local command verbs and envelope UX"
        issues: "work-item domain model and backend traits"
        runtime: "runtime/session/process adapters"
        generate: "spec-to-source generation pipeline"
        td_ast: "TD AST parse/query/validate model"
        validate: "TD validation rule execution"

  section_type_mapping:
    schema:
      primary_location: "src/<domain>/types.rs or src/<domain>/schema.rs"
      owns: ["data structures", "serde shapes", "typed value objects"]
      reexport: "from src/<domain>/mod.rs; lib.rs only for stable public API"
    logic:
      primary_location: "src/<domain>/<capability>.rs"
      owns: ["pure domain decisions", "state transitions", "deterministic algorithms"]
      adapters: "call through traits or service interfaces, never direct external CLI"
    cli:
      primary_location: "src/cli/<verb>.rs"
      owns: ["clap args", "stdout/stderr envelopes", "exit-code mapping", "command routing"]
      delegates_to: ["domain services", "adapter traits"]
    config:
      primary_location: "src/config.rs or src/<domain>/config.rs"
      owns: ["configuration structs", "config file parsing", "defaults"]
    manifest:
      primary_location: "Cargo.toml or generated manifest file"
      owns: ["crate features", "dependencies", "binary registration metadata"]
    tests:
      primary_location: "tests/<feature>_test.rs or #[cfg(test)] module beside target"
      owns: ["behavioral regression", "layout rule enforcement", "dogfood examples"]
    source:
      primary_location: "existing target file named by changes[].path"
      owns: ["explicit replaces ranges", "module preamble/trailer", "handwrite gaps"]

  placement_families:
    crate_root:
      examples: ["src/lib.rs", "src/main.rs", "src/bin/<bin>.rs"]
      rule: "Expose modules and binaries only; delegate immediately."
    domain_module:
      examples: ["src/issues/types.rs", "src/td_ast/types.rs", "src/generate/apply.rs"]
      rule: "Own data, rules, and pure transformations."
    cli_module:
      examples: ["src/cli/issues.rs", "src/cli/td.rs", "src/cli/cb.rs"]
      rule: "Own command UX, validation envelopes, and orchestration glue."
    adapter_module:
      examples: ["src/runtime/github_backend.rs", "src/git.rs", "src/runtime/score_process.rs"]
      rule: "Own external systems and side effects behind stable boundaries."
    generated_module:
      examples: ["src/generate/gen/rust/*.rs", "src/generate/diagrams/*"]
      rule: "Own deterministic emitters and generated helper code."
    compatibility_module:
      examples: ["deprecated aliases", "legacy score/sdd naming bridges"]
      rule: "Isolate old names and route to canonical AW concepts."
    migration_module:
      examples: ["src/cli/td_migrate.rs", "src/generate/diagrams/mermaid_plus/migrate.rs"]
      rule: "Own one-shot or repeatable migrations, never steady-state domain rules."

  public_reexport_policy:
    stable_api:
      location: "src/lib.rs or src/<domain>/mod.rs"
      allowed_for:
        - public domain types
        - public traits
        - generator entrypoints intentionally used cross-crate
    denied_for:
      - CLI arg structs unless explicitly public surface
      - backend implementation details
      - migration helpers
      - compatibility aliases
    rule: "Prefer module-local visibility until another crate has a real API need."

  negative_rules:
    - id: rust-layout-no-domain-gh-cli
      rule: "Domain modules must not invoke gh, gitlab, git, shell commands, or process APIs directly."
      route: "Use adapter traits or runtime backend modules."
    - id: rust-layout-no-cli-business-logic
      rule: "CLI modules must not own durable business rules when a domain/service module can own them."
      route: "Parse args in CLI, call domain/service, render result."
    - id: rust-layout-no-lib-rs-logic
      rule: "lib.rs must not become an implementation dumping ground."
      route: "Declare/re-export modules only."
    - id: rust-layout-no-generated-compat-sprawl
      rule: "Generated code must not create legacy Score/SDD aliases outside compatibility modules."
      route: "Canonical names are Agentic Workflow / aw unless compatibility is explicitly scoped."
    - id: rust-layout-no-adapter-leak
      rule: "GitHub/GitLab/git/process details must not leak into TD AST, generator IR, or domain models."
      route: "Represent external effects as evidence, backend traits, or adapter results."

  dogfood_examples:
    projects_score:
      current_path: "projects/agentic-workflow/src/cli/"
      mapping:
        work_item_cli: "src/cli/issues.rs"
        td_cli: "src/cli/td.rs"
        cb_cli: "src/cli/cb.rs"
        standardize_cli: "src/cli/standardize.rs"
      rule: "CLI files may orchestrate AW lifecycle verbs but must delegate durable state and backend effects."
    projects_sdd:
      current_path: "projects/agentic-workflow/src/{td_ast,generate,validate,validator}/"
      mapping:
        td_ast: "src/td_ast/"
        generator_pipeline: "src/generate/"
        validation_rules: "src/validate/ and src/validator/"
      rule: "Generator and validator modules own spec semantics and deterministic emission, not tracker UX."
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: aw-rust-layout-profile
scenarios:
  - id: S1
    title: "CLI section generates command surface only"
    given:
      - "a TD changes entry has section: cli and target src/cli/issues.rs"
    then:
      - "generated code may add args, dispatch, envelope rendering, and command help"
      - "business validation rules move to a service/domain module or helper with explicit ownership"
      - "GitHub CLI calls remain behind runtime/backend adapters"

  - id: S2
    title: "logic section stays adapter-free"
    given:
      - "a TD logic section describes work-item acceptance rules"
    then:
      - "generated Rust lands in a domain/service module"
      - "the module receives inputs as typed values or traits"
      - "the module does not spawn gh, git, shell, or network clients directly"

  - id: S3
    title: "Score and SDD dogfood stay separated"
    given:
      - "a generator change touches both aw wi CLI and TD AST parsing"
    then:
      - "CLI code lands in the historical projects/score surface: src/cli/*"
      - "AST/generator code lands in the historical projects/sdd surface: src/td_ast or src/generate"
      - "compatibility aliases are isolated and do not become the canonical module model"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tech-design/core/generate/rust-layout-profile.md
    action: create
    section: overview
    impl_mode: hand-written
    description: |
      Define Rust-specific generator placement, public API, ownership, negative
      rule, compatibility, migration, and Score/SDD dogfood mapping policy.
  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```
