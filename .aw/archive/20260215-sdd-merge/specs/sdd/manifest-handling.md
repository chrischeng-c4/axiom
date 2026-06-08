---
id: manifest-handling
type: spec
title: "Manifest Handling in Merge Logic"
version: 1
spec_type: integration
tags: [external]
spec_group: sdd
created_at: 2026-02-15T03:48:01.651714+00:00
updated_at: 2026-02-15T03:48:01.651714+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: sequence
      title: "Archive Flow with Manifests"
depends:
  - crate-unification
changes:
  - file: crates/cclab-sdd/src/cli/archive.rs
    action: MODIFY
  - file: crates/cclab-sdd/src/services/spec_service.rs
    action: MODIFY
history:
  - timestamp: 2026-02-15T03:48:01.651714+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Manifest Handling in Merge Logic

## Overview

This spec updates the change archiving and merging logic to explicitly handle `spec_ir/*.yaml` manifests. This ensures that the intermediate representation of specs is preserved and synchronized along with the markdown specifications, maintaining a complete source of truth.

## Requirements

### R1 - Include Spec IR in Archive

```yaml
id: R1
priority: medium
status: draft
```

Update the archive command to identify and copy `spec_ir` directories and their contents.

### R2 - Validate IR Manifests

```yaml
id: R2
priority: medium
status: draft
```

Implement validation steps to ensure all referenced `spec_ir` files exist and are valid YAML before allowing archive.

### R3 - Sync IR on Merge

```yaml
id: R3
priority: medium
status: draft
```

Ensure that `spec_ir` files are correctly moved to the permanent spec registry during the merge phase.

## Acceptance Criteria

### Scenario: Archive with IR

- **WHEN** archiving a change that includes generated YAML specs
- **THEN** the `spec_ir` folder is present in the archive directory

### Scenario: Merge Verification

- **WHEN** verifying a change with complete manifests
- **THEN** the merge check passes if all IR files are valid

## Diagrams

### Archive Flow with Manifests

```mermaid
sequenceDiagram
    participant User as User
    participant ArchiveCommand as Archive Command
    participant SpecService as Spec Service
    participant FileSystem as File System
    User->>ArchiveCommand: run archive
    ArchiveCommand->>SpecService: collect_artifacts()
    SpecService->>FileSystem: include spec_ir/*.yaml
    ArchiveCommand->>SpecService: verify_manifests()
    ArchiveCommand->>User: complete archive
```

</spec>
