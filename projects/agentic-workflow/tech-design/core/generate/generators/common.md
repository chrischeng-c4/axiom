---
id: sdd-generators-common-types
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Generators Common Types

## Overview
<!-- type: overview lang: markdown -->

6 types in generators/common.rs. Mix of thiserror enum + various struct shapes.

Codegen replaces the common type declarations. A companion source template owns
GeneratedFile and Manifest impl helpers plus the Generator and SpecIRGenerator
traits.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  GeneratorError:
    type: object
    description: Generator error types.
    x-rust-enum:
      derive: [Debug, "thiserror::Error"]
      variants:
        - name: TemplateSetMissing
          kind: tuple
          error: "Template set missing at: {0}"
          fields:
            - { rust_type: PathBuf }
        - name: TemplateRenderError
          kind: struct
          error: "Template render error in '{template}': {message}"
          fields:
            - { name: template, rust_type: String }
            - { name: message, rust_type: String }
        - name: OverwriteNotAllowed
          kind: tuple
          error: "Overwrite not allowed for file: {0}"
          fields:
            - { rust_type: PathBuf }
        - name: IoError
          kind: struct
          error: "IO error for '{path}': {message}"
          fields:
            - { name: path, rust_type: PathBuf }
            - { name: message, rust_type: String }
        - name: SchemaError
          kind: tuple
          error: "Schema error: {0}"
          fields:
            - { rust_type: String }

  OverwritePolicy:
    type: string
    enum: [Error, Skip, Overwrite]
    description: Overwrite policy for generated files.
    x-rust-enum:
      derive: [Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: lowercase
      variants:
        - { name: Error, is_default: true, doc: "Error on overwrite (default)." }
        - { name: Skip,                   doc: "Skip on overwrite." }
        - { name: Overwrite,              doc: "Always overwrite." }

  FileStatus:
    type: string
    enum: [Written, Skipped, Error]
    description: Status of a generated file.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: lowercase

  GeneratorSettings:
    type: object
    required: [name, version, lang, output_dir, overwrite_policy]
    description: Generator settings.
    properties:
      name:
        type: string
      version:
        type: string
      lang:
        type: string
        x-serde-default: true
      output_dir:
        type: object
        x-rust-type: PathBuf
        x-serde-default: true
      overwrite_policy:
        $ref: "#/definitions/OverwritePolicy"
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
    x-trait-impls:
      - trait: Default
        impl_mode: codegen
        body: |
          Self {
              name: "app".to_string(),
              version: "0.1.0".to_string(),
              lang: "".to_string(),
              output_dir: PathBuf::from("."),
              overwrite_policy: OverwritePolicy::default(),
          }

  GeneratedFile:
    type: object
    required: [path, status]
    description: A generated file entry.
    properties:
      path:
        type: object
        x-rust-type: PathBuf
      status:
        $ref: "#/definitions/FileStatus"
      content_hash:
        type: string
        x-serde-skip-if: "Option::is_none"
      error:
        type: string
        x-serde-skip-if: "Option::is_none"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Manifest:
    type: object
    required: [files]
    description: Manifest of generated files (sorted by path for determinism).
    properties:
      files:
        type: object
        x-rust-type: "BTreeMap<PathBuf, GeneratedFile>"
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/generators/common.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - GeneratorError
      - OverwritePolicy
      - GeneratorSettings
      - FileStatus
      - GeneratedFile
      - Manifest
    description: Codegen replaces the serde import, 6 type declarations, impl Default for OverwritePolicy, and impl Default for GeneratorSettings.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
