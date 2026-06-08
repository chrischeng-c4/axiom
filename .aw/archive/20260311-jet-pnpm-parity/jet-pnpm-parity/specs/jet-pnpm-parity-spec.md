---
id: jet-pnpm-parity-spec
main_spec_ref: cclab-jet/pkg-manager-pnpm-parity.md
merge_strategy: new
fill_sections: [overview, diagrams, api_spec, changes]
filled_sections: [overview, diagrams, api_spec, changes]
create_complete: true
---

# Jet Pnpm Parity Spec

## Overview

Extend jet package manager to pnpm feature parity: .npmrc config, frozen lockfile, optional/alias deps, overrides, workspace/monorepo support, store GC, audit, and publishing.

### Schemas

#### NpmrcConfig

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "jet://schemas/npmrc-config",
  "type": "object",
  "description": "Merged .npmrc config (project → user → global precedence)",
  "properties": {
    "registry": { "type": "string", "format": "uri", "default": "https://registry.npmjs.org/" },
    "scoped_registries": {
      "type": "object",
      "description": "@scope → registry URL",
      "additionalProperties": { "type": "string", "format": "uri" }
    },
    "auth_tokens": {
      "type": "object",
      "description": "//registry:_authToken mappings",
      "additionalProperties": { "type": "string" }
    },
    "proxy": { "type": ["string", "null"] },
    "https_proxy": { "type": ["string", "null"] },
    "strict_ssl": { "type": "boolean", "default": true }
  }
}
```

#### WorkspaceConfig

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "jet://schemas/workspace-config",
  "type": "object",
  "properties": {
    "packages": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Glob patterns for workspace packages (e.g. ['packages/*', 'apps/*'])"
    },
    "catalog": {
      "type": "object",
      "description": "Shared version definitions: dep_name → version_range",
      "additionalProperties": { "type": "string" }
    },
    "hoisting": {
      "type": "object",
      "properties": {
        "shamefully_hoist": { "type": "boolean", "default": false },
        "public_hoist_pattern": {
          "type": "array",
          "items": { "type": "string" },
          "default": ["*eslint*", "*prettier*"]
        }
      }
    }
  }
}
```

#### WorkspacePackage

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "jet://schemas/workspace-package",
  "type": "object",
  "required": ["name", "path", "package_json"],
  "properties": {
    "name": { "type": "string" },
    "version": { "type": "string" },
    "path": { "type": "string", "description": "Relative path from workspace root" },
    "package_json": { "$ref": "jet://schemas/package-json" },
    "dependencies_on_workspace": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Names of other workspace packages this depends on"
    }
  }
}
```

#### PackageJson (extended)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "jet://schemas/package-json-v2",
  "allOf": [{ "$ref": "jet://schemas/package-json" }],
  "properties": {
    "optionalDependencies": {
      "type": "object",
      "additionalProperties": { "type": "string" }
    },
    "overrides": {
      "type": "object",
      "description": "Force specific versions across dep tree",
      "additionalProperties": { "type": "string" }
    },
    "workspaces": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Workspace package glob patterns"
    },
    "os": { "type": "array", "items": { "type": "string" } },
    "cpu": { "type": "array", "items": { "type": "string" } },
    "libc": { "type": "array", "items": { "type": "string" } }
  }
}
```

#### AuditReport

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "jet://schemas/audit-report",
  "type": "object",
  "properties": {
    "vulnerabilities": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["package", "severity", "title"],
        "properties": {
          "package": { "type": "string" },
          "severity": { "enum": ["critical", "high", "moderate", "low", "info"] },
          "title": { "type": "string" },
          "url": { "type": "string", "format": "uri" },
          "vulnerable_versions": { "type": "string" },
          "patched_versions": { "type": "string" },
          "dependency_chain": { "type": "array", "items": { "type": "string" } }
        }
      }
    },
    "summary": {
      "type": "object",
      "properties": {
        "critical": { "type": "integer" },
        "high": { "type": "integer" },
        "moderate": { "type": "integer" },
        "low": { "type": "integer" },
        "total": { "type": "integer" }
      }
    }
  }
}
```

#### Lockfile v2 (extended)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "jet://schemas/lockfile-v2-ext",
  "allOf": [{ "$ref": "jet://schemas/lockfile" }],
  "properties": {
    "depsHash": { "type": "string", "description": "SHA-256 of sorted package.json deps for frozen lockfile check" },
    "overrides": { "type": "object", "additionalProperties": { "type": "string" } },
    "patchedPackages": {
      "type": "object",
      "description": "package@version → patch file path",
      "additionalProperties": { "type": "string" }
    }
  }
}
```
## Diagrams

### .npmrc Config Resolution

```mermaid
flowchart TD
    A[Start install] --> B[Load project .npmrc]
    B --> C[Load ~/.npmrc]
    C --> D[Load /etc/npmrc]
    D --> E[Merge with precedence<br/>project > user > global]
    E --> F{Scoped package?}
    F -->|Yes| G[Use @scope registry + auth]
    F -->|No| H[Use default registry]
    G --> I[Apply auth token to reqwest]
    H --> I
    I --> J[Fetch metadata]
```

### Frozen Lockfile Flow

```mermaid
flowchart TD
    A[jet install] --> B{CI env detected?<br/>CI=true, GITHUB_ACTIONS, etc.}
    B -->|Yes| C[Auto-enable --frozen-lockfile]
    B -->|No| D{--frozen-lockfile flag?}
    D -->|Yes| C
    D -->|No| E[Normal install]

    C --> F[Compute depsHash<br/>SHA-256 of sorted deps]
    F --> G{depsHash == lockfile.depsHash?}
    G -->|Yes| H[Proceed with lockfile fast-path]
    G -->|No| I[ERROR: Lockfile drift detected<br/>Run jet install locally]

    E --> J[Resolve + install + write lockfile]
```

### Workspace Discovery and Install

```mermaid
sequenceDiagram
    participant CLI as jet install
    participant WS as WorkspaceManager
    participant PM as PackageManager
    participant Store as ~/.jet-store/

    CLI->>WS: discover_workspace(root)
    WS->>WS: Read package.json.workspaces<br/>or jet-workspace.yaml
    WS->>WS: Glob expand patterns<br/>(packages/*, apps/*)
    WS-->>CLI: Vec&lt;WorkspacePackage&gt;

    CLI->>WS: build_dependency_graph()
    WS->>WS: Topological sort workspace packages
    WS-->>CLI: Ordered packages

    loop Each workspace package
        CLI->>PM: resolve(pkg.dependencies)
        PM->>PM: Replace workspace:* with local version
        PM->>PM: Replace catalog refs with catalog versions
        CLI->>Store: install external deps
        CLI->>WS: symlink workspace deps → node_modules/
    end

    CLI->>CLI: Write single jet-lock.yaml at root
```

### Store GC (Garbage Collection)

```mermaid
stateDiagram-v2
    [*] --> ScanProjects: jet store prune

    ScanProjects --> CollectRefs: Find all jet-lock.yaml files
    CollectRefs --> BuildRefSet: Extract referenced packages

    BuildRefSet --> ScanStore: List ~/.jet-store/ entries
    ScanStore --> Compare: referenced vs stored

    Compare --> MarkOrphan: package not in ref set
    Compare --> Keep: package in ref set

    MarkOrphan --> Delete: Remove orphan dirs
    Keep --> [*]
    Delete --> Report: Print reclaimed space
    Report --> [*]
```

### Workspace Protocol Resolution

```mermaid
flowchart LR
    subgraph "Dependency Spec"
        W1["workspace:*"] -->|resolves to| V1["exact local version"]
        W2["workspace:^"] -->|resolves to| V2["^local.version"]
        W3["workspace:~"] -->|resolves to| V3["~local.version"]
    end

    subgraph "Install Behavior"
        V1 --> SYM[Symlink to workspace pkg]
        V2 --> SYM
        V3 --> SYM
    end

    subgraph "Publish Transform"
        V1 --> PUB1["exact version in published pkg"]
        V2 --> PUB2["^version in published pkg"]
        V3 --> PUB3["~version in published pkg"]
    end
```

### Audit Flow

```mermaid
sequenceDiagram
    participant CLI as jet audit
    participant LF as jet-lock.yaml
    participant API as npm Advisory API
    participant Report as AuditReport

    CLI->>LF: Read all resolved packages
    CLI->>CLI: Build dependency tree payload
    CLI->>API: POST /-/npm/v1/security/audits
    API-->>CLI: Advisory response (CVEs)

    CLI->>Report: Group by severity
    CLI->>Report: Map dependency chains
    CLI-->>CLI: Print table (critical/high/moderate/low)
    CLI->>CLI: Exit code 1 if critical/high found
```

### Optional Dependency Platform Check

```mermaid
flowchart TD
    PKG[Package with os/cpu/libc] --> OS{os includes current?}
    OS -->|Yes| CPU{cpu includes current?}
    OS -->|No| SKIP[Skip: platform mismatch]
    CPU -->|Yes| LIBC{libc matches?}
    CPU -->|No| SKIP
    LIBC -->|Yes| INSTALL[Install normally]
    LIBC -->|No| SKIP
    SKIP --> WARN[Log: skipping optional dep]
```

### Override Resolution

```mermaid
flowchart TD
    R[BFS Resolver] --> CHECK{overrides in package.json?}
    CHECK -->|Yes| LOAD[Load overrides map]
    CHECK -->|No| NORMAL[Normal resolution]

    LOAD --> RESOLVE[For each dep in tree]
    RESOLVE --> MATCH{dep name in overrides?}
    MATCH -->|Yes| FORCE[Force override version]
    MATCH -->|No| NORMAL
    FORCE --> CONTINUE[Continue BFS with forced version]
    NORMAL --> CONTINUE
```
## API Spec

### OpenAPI 3.1 (Internal API)

```yaml
openapi: 3.1.0
info:
  title: Jet Package Manager — pnpm Parity Extensions
  version: 3.0.0

paths:
  /install:
    post:
      operationId: PackageManager::install
      summary: Install with frozen lockfile and workspace support
      parameters:
        - name: frozen_lockfile
          in: query
          schema: { type: boolean, default: false }
          description: Fail if lockfile drift detected (auto-enabled in CI)
        - name: filter
          in: query
          schema: { type: string }
          description: Workspace filter pattern (e.g. "pkg-a", "apps/*")
        - name: recursive
          in: query
          schema: { type: boolean, default: false }
          description: Run across all workspace packages
      responses:
        200: { description: All packages installed }
        409: { description: Version conflict }
        422: { description: "Frozen lockfile drift: depsHash mismatch" }

  /update:
    post:
      operationId: PackageManager::update
      summary: Update packages to latest matching versions
      parameters:
        - name: package
          in: query
          schema: { type: string }
          description: "Specific package to update (omit for all)"
        - name: latest
          in: query
          schema: { type: boolean, default: false }
          description: "Ignore semver range, update to absolute latest"
      responses:
        200: { description: Packages updated, lockfile rewritten }

  /audit:
    get:
      operationId: PackageManager::audit
      summary: Check for known security vulnerabilities
      responses:
        200:
          description: No critical/high vulnerabilities
          content:
            application/json:
              schema: { $ref: "jet://schemas/audit-report" }
        1:
          description: Critical or high severity vulnerabilities found (exit code 1)

  /patch:
    post:
      operationId: PackageManager::patch
      summary: Create editable copy of installed package for patching
      parameters:
        - name: package
          in: query
          required: true
          schema: { type: string }
      responses:
        200: { description: "Package copied to patches/{name}@{version}/" }

  /patch/commit:
    post:
      operationId: PackageManager::patch_commit
      summary: Generate .patch file from modified package
      parameters:
        - name: package
          in: query
          required: true
          schema: { type: string }
      responses:
        200: { description: "Patch file written to patches/{name}@{version}.patch" }

  /publish:
    post:
      operationId: PackageManager::publish
      summary: Publish package to npm registry
      description: Resolves workspace:* protocols to real versions before publish
      parameters:
        - name: tag
          in: query
          schema: { type: string, default: "latest" }
        - name: access
          in: query
          schema: { type: string, enum: [public, restricted] }
      responses:
        200: { description: Package published }
        401: { description: Auth token missing or invalid }

  /pack:
    post:
      operationId: PackageManager::pack
      summary: Create tarball without publishing
      responses:
        200: { description: "Tarball created: {name}-{version}.tgz" }

  /store/prune:
    post:
      operationId: StoreManager::prune
      summary: Remove unreferenced packages from global store
      responses:
        200: { description: "Pruned N packages, reclaimed X MB" }

  /workspace/discover:
    get:
      operationId: WorkspaceManager::discover
      summary: Discover workspace packages from config
      responses:
        200:
          content:
            application/json:
              schema:
                type: array
                items: { $ref: "jet://schemas/workspace-package" }

  /config/npmrc:
    get:
      operationId: NpmrcParser::load
      summary: Load and merge .npmrc from all levels
      responses:
        200:
          content:
            application/json:
              schema: { $ref: "jet://schemas/npmrc-config" }
```
## Changes


| File | Action | Description |
|------|--------|-------------|
| `pkg_manager/npmrc.rs` (~200L) | create | `.npmrc` parser: 3-level merge (project → user → global), scoped registries, auth tokens, proxy settings |
| `pkg_manager/workspace.rs` (~350L) | create | Workspace discovery (`package.json` workspaces + `jet-workspace.yaml`), glob expansion, topological sort, `workspace:*`/`^`/`~` protocol resolution, catalog shared versions |
| `pkg_manager/audit.rs` (~150L) | create | npm advisory API client (`/-/npm/v1/security/audits`), `AuditReport` builder, severity grouping, exit code logic |
| `pkg_manager/patch.rs` (~120L) | create | `jet patch <pkg>` copy-to-edit, `jet patch-commit` diff generation, `.patch` file writer, `patchedPackages` lockfile integration |
| `pkg_manager/publish.rs` (~180L) | create | `jet publish` / `jet pack`: tarball creation, `workspace:*` → real version transform, `.npmrc` auth token injection, `npm` registry PUT |
| `pkg_manager/gc.rs` (~100L) | create | Store GC: scan all `jet-lock.yaml` files, build ref set, diff against `~/.jet-store/`, delete orphans, report reclaimed space |
| `pkg_manager/mod.rs` (315L) | modify | Add `--frozen-lockfile` flag (auto-enable in CI), `--filter` workspace filter, `--recursive` flag, `depsHash` computation and check, wire new submodules |
| `pkg_manager/resolver.rs` (343L) | modify | `optionalDependencies` with `os`/`cpu`/`libc` platform check, `overrides` map forcing, alias deps (`npm:` protocol), `workspace:*` local resolution |
| `pkg_manager/registry.rs` (185L) | modify | Use `NpmrcConfig` for registry URL + auth token per request, scoped registry routing, proxy support |
| `pkg_manager/store.rs` (417L) | modify | File-level content-addressable dedup (SHA-256 per file, hardlink identical files across packages), hoisting strategies (`shamefully-hoist`, `public-hoist-pattern`) |
| `pkg_manager/lockfile.rs` (295L) | modify | Add `depsHash`, `overrides`, `patchedPackages` fields to lockfile v2; import from `package-lock.json` / `yarn.lock` |
| `cli.rs` (249L) | modify | Add subcommands: `update`, `audit`, `patch`, `patch-commit`, `publish`, `pack`, `store prune`; add `--frozen-lockfile`, `--filter`, `-r` flags to `install` |
| `lib.rs` (12L) | modify | Re-export new modules: `npmrc`, `workspace`, `audit`, `patch`, `publish`, `gc` |
| `Cargo.toml` | modify | Add deps: `glob` (workspace patterns), `ini` (.npmrc parsing), `ignore` (gitignore-style matching) |
# Reviews
