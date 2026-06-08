---
id: sdd-models-project
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Project Model

## Overview
<!-- type: overview lang: markdown -->

Project and workspace data model for `projects/agentic-workflow/src/models/project.rs` — the canonical
registry types shared between `project_discovery` (writes) and `project_registry` (reads/merges).

Six structs declared in this spec:

- `Project` — a discovered or manually declared project entry in `.aw/projects.toml`.
  Four fields: `name`, `path` (PathBuf), optional `tech_design_dir`, and `workspaces: Vec<Workspace>`.
- `Workspace` — a single language workspace within a project.
  Five fields: optional `name`, `paths: Vec<String>`, `target: Language`, optional `test_cmd`, optional `codegen: CodegenProfile`.
- `CodegenProfile` — codegen configuration for a workspace.
  Six fields: optional `target: Language`, optional `profile`, optional `framework`,
  optional `runtime`, optional `bundler`, and `default_derives: Vec<String>` with
  `#[serde(default, skip_serializing_if = "Vec::is_empty")]`.
- `WorkspaceDefaults` — fallback values applied when a workspace field is absent.
  One field: optional `codegen: CodegenProfile`.
- `ProjectsToml` — top-level document structure for `.aw/projects.toml`.
  Two fields: optional `defaults: ProjectsDefaults`, and `projects: Vec<Project>` with `#[serde(default)]`.
- `ProjectsDefaults` — container for the `[defaults]` table in `projects.toml`.
  One field: optional `workspace: WorkspaceDefaults`.

This spec is the dogfood exercise for `x-serde-skip-if` (Option::is_none and Vec::is_empty)
and `x-serde-default` extensions introduced in commit 88b58ebc. No impl blocks exist in the
source file — codegen replaces the serde import and all six struct declarations.
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  Project:
    type: object
    required: [name, path, workspaces]
    description: |
      A discovered or manually declared project entry in `.aw/projects.toml`.
      Each project maps to a top-level directory under `crates/`, `projects/`, or `packages/`.
    properties:
      name:
        type: string
        description: "Project identifier derived from directory name."
      path:
        type: string
        x-rust-type: PathBuf
        description: "Path relative to repo root (e.g. `projects/agentic-workflow`, `projects/conductor`)."
      tech_design_dir:
        type: string
        description: "Override for `.aw/tech-design` sub-path. Defaults to the discovered path when absent."
        x-serde-skip-if: "Option::is_none"
      workspaces:
        type: array
        items:
          $ref: "#/definitions/Workspace"
        description: "Non-empty list of workspaces contained in this project."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]

  Workspace:
    type: object
    required: [paths, target]
    description: |
      A single language workspace within a project.
      Single-language projects have one workspace; `be`/`fe` projects have two.
    properties:
      name:
        type: string
        description: "Short identifier (e.g. `be`, `fe`, `cli`, or same as project name)."
        x-serde-skip-if: "Option::is_none"
      paths:
        type: array
        items:
          type: string
        description: "Glob path patterns relative to repo root (e.g. `[\"projects/agentic-workflow/**\"]`)."
      target:
        type: string
        x-rust-type: Language
        description: "Language/runtime target inferred from manifest files."
      test_cmd:
        type: string
        description: "Shell command to run the workspace test suite. Omitted when the required tool/lock file is not present."
        x-serde-skip-if: "Option::is_none"
      codegen:
        $ref: "#/definitions/CodegenProfile"
        description: "Optional codegen profile override for this workspace."
        x-serde-skip-if: "Option::is_none"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]

  # NOTE: Vec fields (default_derives, projects) are listed in `required` so
  # the generator doesn't wrap them in Option<Vec<...>>. Vec already has a
  # natural empty-default; combined with x-serde-default + skip_if=Vec::is_empty
  # they round-trip as `Vec<T>` with the same serde shape as the source.
  CodegenProfile:
    type: object
    required: [default_derives]
    description: |
      Codegen configuration for a workspace.
      Used in both per-workspace overrides and `[defaults.workspace]`.
    properties:
      target:
        type: string
        x-rust-type: Language
        description: "Optional language/runtime target override for code generation. Defaults to the workspace target when absent."
        x-serde-skip-if: "Option::is_none"
      profile:
        type: string
        description: "Named generator/template profile for this workspace."
        x-serde-skip-if: "Option::is_none"
      framework:
        type: string
        description: "Optional web/app framework (e.g. `axum-service`, `react-component`)."
        x-serde-skip-if: "Option::is_none"
      runtime:
        type: string
        description: "Optional runtime identifier (e.g. `tokio`, `uvicorn`)."
        x-serde-skip-if: "Option::is_none"
      bundler:
        type: string
        description: "Optional bundler (e.g. `vite`, `webpack`)."
        x-serde-skip-if: "Option::is_none"
      default_derives:
        type: array
        items:
          type: string
        description: "Default `#[derive(...)]` attributes for generated structs."
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq]

  WorkspaceDefaults:
    type: object
    description: |
      Fallback values applied when a workspace field is absent in both
      auto-discovery and `config.toml` overrides.
    properties:
      codegen:
        $ref: "#/definitions/CodegenProfile"
        description: "Default codegen profile applied to every workspace missing one."
        x-serde-skip-if: "Option::is_none"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Default]

  ProjectsToml:
    type: object
    required: [projects]
    description: "Top-level document structure for `.aw/projects.toml`."
    properties:
      defaults:
        $ref: "#/definitions/ProjectsDefaults"
        description: "Workspace-level fallback defaults."
        x-serde-skip-if: "Option::is_none"
      projects:
        type: array
        items:
          $ref: "#/definitions/Project"
        description: "Ordered list of discovered/declared project entries."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Default]

  ProjectsDefaults:
    type: object
    description: "Container for the `[defaults]` table in `projects.toml`."
    properties:
      workspace:
        $ref: "#/definitions/WorkspaceDefaults"
        description: "Default values applied to every workspace."
        x-serde-skip-if: "Option::is_none"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Default]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/project.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - Project
      - Workspace
      - CodegenProfile
      - WorkspaceDefaults
      - ProjectsToml
      - ProjectsDefaults
    description: |
      Codegen replaces all 6 struct decls. No impl blocks in source.
      Hand-written: just `// @spec` comment plus `use std::path::PathBuf` and
      `use crate::models::tech_stack::Language` imports (untouched).
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [schema] All 6 structs present and correctly declared; Option fields consistently annotated with `x-serde-skip-if: "Option::is_none"`; `CodegenProfile.default_derives` correctly carries both `x-serde-default: true` and `x-serde-skip-if: "Vec::is_empty"`; `Default` derive present on `WorkspaceDefaults`, `ProjectsToml`, and `ProjectsDefaults`; `replaces:` contains only the 6 struct names with no impl entries. Spec is implementation-ready.
