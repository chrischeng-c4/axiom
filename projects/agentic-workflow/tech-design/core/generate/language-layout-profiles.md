---
id: aw-language-layout-profiles
summary: "Define cross-language layout profiles behind section type v2 roles."
fill_sections: [overview, schema, scenarios, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# AW Language Layout Profiles

## Overview
<!-- type: overview lang: markdown -->

Section type v2 is the semantic contract for a TD section: `schema` describes
data shapes, `logic` describes deterministic behavior, `cli` describes command
surfaces, `deployment` describes deployable runtime artifacts, and so on.
Language layout profiles must not invent a second taxonomy. Their job is only
to decide how a section type v2 role lands in a concrete language or artifact
family.

The Rust layout profile is the first stabilized instance. This spec defines the
shared profile interface and the initial backlog for Python, TypeScript,
YAML/Kustomize, and SQL/Alembic. Each profile has the same shape: target
detection, section-role placement rules, ownership boundaries, validation
hooks, unsupported-role handling, and examples.

Unsupported or ambiguous placement is never claimed regenerable. A generator
may emit CODEGEN only when exactly one profile and one placement rule apply to
the target path and section role. Otherwise the lifecycle must leave the region
as HANDWRITE, record a concrete blocker, and point at the profile gap that
would make the region regenerable later.

## Schema
<!-- type: schema lang: yaml -->

```yaml
language_layout_profiles:
  version: 1
  role_source_of_truth:
    spec: projects/agentic-workflow/tech-design/surface/specs/score-section-type-registry.md
    rule: "Profiles consume approved section type v2 roles; they do not add language-specific section types."
    role_fields:
      - section_type
      - family
      - generator
      - heading_match
      - use_for

  profile_interface:
    required_fields:
      - id
      - version
      - target_family
      - detects
      - section_role_mapping
      - ownership_boundaries
      - validation_hooks
      - unsupported_policy
      - examples
    fields:
      id:
        type: string
        rule: "Stable profile id such as rust, python, typescript, yaml-kustomize, or sql-alembic."
      version:
        type: integer
        rule: "Increment only when placement semantics change incompatibly."
      target_family:
        type: string
        allowed:
          - language
          - artifact
      detects:
        required:
          - extensions
          - path_globs
          - negative_globs
        rule: "Target detection must be deterministic from path and project metadata."
      section_role_mapping:
        key: "approved section type v2 name"
        value:
          required:
            - placement_family
            - allowed_paths
            - owns
            - must_not_own
            - generated_claim_requirements
          rule: "Each mapping answers how a section role lands for this profile."
      ownership_boundaries:
        rule: "Boundaries name domain, adapter, CLI, manifest, migration, test, and compatibility zones when relevant."
      validation_hooks:
        required:
          - resolve_profile
          - resolve_role
          - validate_target_path
          - validate_boundary
          - validate_codegen_claim
      unsupported_policy:
        rule: "Unsupported or ambiguous roles become HANDWRITE/blocked, not CODEGEN/regenerable."
      examples:
        rule: "Every profile includes one positive placement and one blocked placement."

  validation_hooks:
    resolve_profile:
      input: ["project_root", "target_path", "project_metadata"]
      success: "exactly one profile id"
      block_when:
        - "no profile matches"
        - "multiple profiles match the same target path"
    resolve_role:
      input: ["section_type", "section_family", "profile_id"]
      success: "one section_role_mapping entry"
      block_when:
        - "section type is deprecated"
        - "profile has no rule for this role"
    validate_target_path:
      input: ["profile_id", "section_type", "target_path", "changes_entry"]
      success: "target path is inside allowed paths for the role"
      block_when:
        - "path is only allowed for another role"
        - "path is a compatibility or migration zone without explicit role support"
    validate_boundary:
      input: ["profile_id", "section_type", "target_path", "generated_preview"]
      success: "generated content respects ownership boundaries"
      block_when:
        - "domain code invokes external process, network, git, or tracker adapters directly"
        - "CLI code owns durable business rules"
        - "manifest or migration code embeds runtime logic"
    validate_codegen_claim:
      input: ["profile_id", "section_type", "target_path", "impl_mode", "coverage_claim"]
      success: "CODEGEN claim carries profile id, role id, target path, and deterministic replacement rule"
      block_when:
        - "coverage is claimed without a profile id"
        - "coverage is claimed for unsupported or ambiguous placement"
        - "replacement range cannot be resolved deterministically"

  unsupported_policy:
    generated_result: "blocked"
    impl_mode: "hand-written"
    marker: "HANDWRITE"
    required_evidence:
      - profile_id_or_missing_profile_reason
      - section_type
      - target_path
      - blocker_id
      - tracker_or_follow_up_wi
    message: "No regenerable claim may be emitted until a profile rule and deterministic replacement rule exist."

  profiles:
    rust:
      status: stable
      spec: projects/agentic-workflow/tech-design/core/generate/rust-layout-profile.md
      notes:
        - "First complete profile; owns crate, module, CLI, adapter, generated, compatibility, migration, and test placement."

    python:
      status: planned
      target_family: language
      detects:
        extensions: [".py"]
        path_globs: ["**/*.py", "pyproject.toml", "setup.cfg"]
        negative_globs: ["**/.venv/**", "**/__pycache__/**"]
      section_role_mapping:
        schema:
          placement_family: "package schema module"
          allowed_paths: ["<package>/models.py", "<package>/schemas.py", "<package>/types.py"]
          owns: ["dataclasses", "pydantic models", "typed value objects"]
          must_not_own: ["I/O clients", "CLI parsing", "migration side effects"]
          generated_claim_requirements: ["profile_id", "class_or_type_name", "replace_range"]
        logic:
          placement_family: "service or domain module"
          allowed_paths: ["<package>/services/*.py", "<package>/domain/*.py"]
          owns: ["pure rules", "state transitions", "deterministic transformations"]
          must_not_own: ["subprocess calls", "network clients", "database sessions"]
          generated_claim_requirements: ["profile_id", "function_or_class_name", "adapter_inputs"]
        cli:
          placement_family: "command module"
          allowed_paths: ["<package>/cli.py", "<package>/cli/*.py"]
          owns: ["arg parsing", "stdout/stderr envelopes", "exit code mapping"]
          must_not_own: ["durable business rules", "backend protocol details"]
          generated_claim_requirements: ["profile_id", "command_name", "delegated_service"]
        tests:
          placement_family: "pytest module"
          allowed_paths: ["tests/**/*.py"]
          owns: ["fixtures", "behavioral assertions", "regression coverage"]
          must_not_own: ["production runtime behavior"]
          generated_claim_requirements: ["profile_id", "test_target", "assertion_scope"]

    typescript:
      status: planned
      target_family: language
      detects:
        extensions: [".ts", ".tsx"]
        path_globs: ["src/**/*.ts", "src/**/*.tsx", "packages/*/src/**/*.ts", "packages/*/src/**/*.tsx"]
        negative_globs: ["**/node_modules/**", "**/dist/**", "**/.next/**"]
      section_role_mapping:
        schema:
          placement_family: "types or schema module"
          allowed_paths: ["src/types/*.ts", "src/schemas/*.ts", "packages/*/src/types/*.ts"]
          owns: ["interfaces", "zod schemas", "DTO types"]
          must_not_own: ["React rendering", "network transport", "business side effects"]
          generated_claim_requirements: ["profile_id", "export_name", "replace_range"]
        logic:
          placement_family: "domain or service module"
          allowed_paths: ["src/domain/**/*.ts", "src/services/**/*.ts", "packages/*/src/domain/**/*.ts"]
          owns: ["pure functions", "state transitions", "workflow decisions"]
          must_not_own: ["fetch clients", "DOM access", "CLI process calls"]
          generated_claim_requirements: ["profile_id", "export_name", "adapter_inputs"]
        component:
          placement_family: "UI component module"
          allowed_paths: ["src/components/**/*.tsx", "packages/*/src/components/**/*.tsx"]
          owns: ["component props", "render structure", "local interaction state"]
          must_not_own: ["backend protocol decisions", "global app routing"]
          generated_claim_requirements: ["profile_id", "component_name", "props_schema"]
        tests:
          placement_family: "unit or component test"
          allowed_paths: ["src/**/*.test.ts", "src/**/*.test.tsx", "tests/**/*.ts", "tests/**/*.tsx"]
          owns: ["unit assertions", "component behavior", "fixture setup"]
          must_not_own: ["production logic"]
          generated_claim_requirements: ["profile_id", "test_target", "runner"]

    yaml_kustomize:
      status: planned
      target_family: artifact
      detects:
        extensions: [".yaml", ".yml"]
        path_globs: ["**/kustomization.yaml", "k8s/**/*.yaml", "deploy/**/*.yaml", "charts/**/*.yaml"]
        negative_globs: ["**/node_modules/**", "**/target/**"]
      section_role_mapping:
        deployment:
          placement_family: "kubernetes resource or kustomize overlay"
          allowed_paths: ["k8s/base/*.yaml", "k8s/overlays/*/*.yaml", "deploy/**/*.yaml"]
          owns: ["resource metadata", "spec blocks", "overlay patches"]
          must_not_own: ["application business logic", "secret values", "runtime code"]
          generated_claim_requirements: ["profile_id", "kind", "resource_name", "overlay_scope"]
        runtime-image:
          placement_family: "image metadata or deployment reference"
          allowed_paths: ["k8s/**/*.yaml", "deploy/**/*.yaml"]
          owns: ["image reference wiring", "container command/env surface"]
          must_not_own: ["Dockerfile body when target is YAML", "secret material"]
          generated_claim_requirements: ["profile_id", "container_name", "image_field_path"]
        config:
          placement_family: "ConfigMap or plain config manifest"
          allowed_paths: ["config/**/*.yaml", "k8s/**/config*.yaml", "deploy/**/config*.yaml"]
          owns: ["non-secret config keys", "schema-shaped static config"]
          must_not_own: ["secrets", "dynamic runtime decisions"]
          generated_claim_requirements: ["profile_id", "config_key_scope", "resource_name"]

    sql_alembic:
      status: planned
      target_family: artifact
      detects:
        extensions: [".sql", ".py"]
        path_globs: ["migrations/**/*.sql", "alembic/versions/*.py", "**/versions/*.py"]
        negative_globs: ["**/.venv/**", "**/__pycache__/**"]
      section_role_mapping:
        db-model:
          placement_family: "DDL or Alembic revision"
          allowed_paths: ["migrations/**/*.sql", "alembic/versions/*.py", "**/versions/*.py"]
          owns: ["table shape", "indexes", "constraints", "upgrade/downgrade operations"]
          must_not_own: ["application query logic", "runtime data backfill without explicit migration role"]
          generated_claim_requirements: ["profile_id", "revision_id_or_sql_unit", "object_names"]
        schema:
          placement_family: "typed database schema description"
          allowed_paths: ["migrations/**/*.sql", "alembic/versions/*.py"]
          owns: ["column definitions", "enum definitions", "constraint declarations"]
          must_not_own: ["ORM runtime logic", "service-layer decisions"]
          generated_claim_requirements: ["profile_id", "object_names", "replace_range"]
        tests:
          placement_family: "migration verification"
          allowed_paths: ["tests/**/*migration*.py", "tests/**/*.sql"]
          owns: ["migration idempotency checks", "schema assertion fixtures"]
          must_not_own: ["production migration execution"]
          generated_claim_requirements: ["profile_id", "migration_target", "assertion_scope"]

  initial_backlog:
    - id: profile-python
      title: "Stabilize Python layout profile"
      depends_on: ["aw-language-layout-profiles"]
      scope: "Package/module placement, adapter-free domain rules, pytest tests, FastAPI/backend examples."
    - id: profile-typescript
      title: "Stabilize TypeScript layout profile"
      depends_on: ["aw-language-layout-profiles"]
      scope: "Type/schema placement, service/domain boundaries, component modules, TS test placement."
    - id: profile-yaml-kustomize
      title: "Stabilize YAML/Kustomize artifact profile"
      depends_on: ["aw-language-layout-profiles"]
      scope: "Deployment/config/runtime-image placement for base, component, and overlay manifests."
    - id: profile-sql-alembic
      title: "Stabilize SQL/Alembic artifact profile"
      depends_on: ["aw-language-layout-profiles"]
      scope: "DDL, revision, migration verification, and blocked data-backfill policy."
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: aw-language-layout-profiles
scenarios:
  - id: S1
    title: "Profile resolves from target path and section role"
    given:
      - "a changes entry targets src/services/work_items.py"
      - "the section type is logic"
    then:
      - "the python profile is selected"
      - "the logic role maps to a service or domain module"
      - "the generated claim records profile_id=python, section_type=logic, and target_path"

  - id: S2
    title: "Ambiguous placement is blocked"
    given:
      - "a changes entry targets generated.yaml"
      - "both generic YAML and Kustomize deployment rules could match"
    then:
      - "the generator does not emit a CODEGEN/regenerable claim"
      - "the lifecycle leaves the region HANDWRITE"
      - "the blocked evidence names the missing disambiguation rule"

  - id: S3
    title: "Section type v2 remains the semantic source"
    given:
      - "a TypeScript component target uses section type component"
    then:
      - "the TypeScript profile maps the component role to a UI component module"
      - "no typescript-component section type is introduced"
      - "validation rejects attempts to add language-specific section aliases"

  - id: S4
    title: "Unsupported SQL side effect is not regenerable"
    given:
      - "a db-model section describes a migration that also backfills production data"
    then:
      - "the sql_alembic profile accepts the DDL placement only"
      - "the data backfill remains HANDWRITE/blocked unless a migration role explicitly supports it"
      - "project health does not count the backfill as regenerable"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tech-design/core/generate/language-layout-profiles.md
    action: create
    section: overview
    impl_mode: hand-written
    description: |
      Define the shared layout profile interface, validation hooks,
      unsupported-placement policy, and initial Python, TypeScript,
      YAML/Kustomize, and SQL/Alembic profile backlog behind section type v2.
  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```
