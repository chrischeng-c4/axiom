---
id: lens-index-storage
type: spec
title: "Lens Index Storage & Resolution"
version: 1
spec_type: utility
created_at: 2026-02-10T06:42:05.027180+00:00
updated_at: 2026-02-10T06:42:05.027180+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Index Path Resolution Flow"
    - type: class
      title: "Storage Components"
history:
  - timestamp: 2026-02-10T06:42:05.027180+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "This analysis/standardization logic TD supports brownfield semantic coverage and takeover readiness gates."
---

# Lens Index Storage & Resolution

## Overview
<!-- type: doc lang: markdown -->

A utility specification for resolving and managing the persistent storage location of Lens code indexes. It ensures that indexes are stored in a consistent, project-local directory structure (`{project_root}/cclab/.index/`), derived from the canonical path of the project root. This enables index persistence across server restarts and supports multiple projects and monorepo modules without collision.

## Requirements
<!-- type: doc lang: markdown -->

### R1 - Persistent Storage Path

```yaml
id: R1
priority: medium
status: draft
```

The system must resolve the persistent storage root for a project at `{project_root}/cclab/.index/`, where `{project_root}` is the canonicalized project root directory.

### R2 - Path Canonicalization

```yaml
id: R2
priority: medium
status: draft
```

The project root path must be canonicalized (resolving symlinks and relative paths) before hashing to ensure that the same project always maps to the same storage location.

### R3 - Path Hashing

```yaml
id: R3
priority: medium
status: draft
```

The canonical project root path must be hashed using a stable algorithm (e.g., SHA256) to generate the `{path_hash}` segment of the storage path.

### R4 - Module Index Separation

```yaml
id: R4
priority: medium
status: draft
```

The storage structure must support separate index files or subdirectories for distinct modules defined in the project configuration, preventing conflicts in monorepos.

## Acceptance Criteria
<!-- type: doc lang: markdown -->

### Scenario: Resolve New Project Path

- **WHEN** The index storage path is requested for a new project at `/user/dev/my-project`.
- **THEN** The canonicalized project path is used to construct `{project_root}/cclab/.index/` and returned.

### Scenario: Resolve Existing Project Path

- **WHEN** The index storage path is requested again for the same project root.
- **THEN** The same hash and storage path are returned as the first request.

### Scenario: Module Sub-path Resolution

- **WHEN** The index path for a specific module named 'backend' is requested.
- **THEN** The returned path includes the module's identifier (e.g., `cclab/.index/backend.idx`).

## Diagrams
<!-- type: doc lang: markdown -->

### Index Path Resolution Flow

```mermaid
flowchart TB
    start([Start: Resolve Path])
    canon[Canonicalize Project Root]
    hash[Compute SHA256 Hash]
    construct[Construct {project_root}/cclab/.index/]
    ensure[Ensure Directory Exists]
    end([Return PathBuf])
    start --> canon
    canon --> hash
    hash --> construct
    construct --> ensure
    ensure --> end
```

### Storage Components

```mermaid
classDiagram
    class IndexStorageManager {
        <<service>>
        +resolve_root_path(String project_path) PathBuf
        +resolve_module_path(String project_path, String module_name) PathBuf
    }
    class PathHasher {
        <<service>>
        +hash_path(PathBuf path) String
    }
    IndexStorageManager ..> PathHasher : uses
```
