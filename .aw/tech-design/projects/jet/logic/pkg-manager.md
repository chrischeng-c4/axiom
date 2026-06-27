---
id: projects-jet-logic-pkg-manager-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Jet Package Manager

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/pkg-manager.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# Jet Package Manager

### Overview

Jet package manager is an npm-compatible installer with pnpm-style content
addressing. It resolves package metadata, writes `jet-lock.yaml`, downloads and
extracts tarballs into a global store, symlinks packages into `node_modules`,
supports workspaces, applies platform filters, exposes audit/patch/publish
commands, and prunes unused store entries.

### Source Map

| Concern | Source |
|---------|--------|
| Install orchestration | `crates/jet/src/pkg_manager/mod.rs` |
| Dependency resolution | `crates/jet/src/pkg_manager/resolver.rs` |
| Registry metadata and tarballs | `crates/jet/src/pkg_manager/registry.rs` |
| Lockfile model | `crates/jet/src/pkg_manager/lockfile.rs` |
| Global store and symlinks | `crates/jet/src/pkg_manager/store.rs` |
| Workspace detection | `crates/jet/src/pkg_manager/workspace.rs`, `nx.rs` |
| Platform filters | `crates/jet/src/pkg_manager/platform.rs` |
| Audit | `crates/jet/src/pkg_manager/audit.rs` |
| Store garbage collection | `crates/jet/src/pkg_manager/gc.rs` |
| Patch workflow | `crates/jet/src/pkg_manager/patch.rs` |
| Publishing | `crates/jet/src/pkg_manager/publish.rs` |

### Requirements

```mermaid
---
id: jet-pkg-manager-requirements
entry: R1
---
requirementDiagram
    requirement R1 {
        id: R1
        text: Install must reuse valid lockfile and marker state when possible
        risk: high
        verifymethod: test
    }
    requirement R2 {
        id: R2
        text: Resolver must fetch metadata through L1 and L2 cache before registry HTTP
        risk: high
        verifymethod: test
    }
    requirement R3 {
        id: R3
        text: Store must verify tarball shasum and link packages into node_modules
        risk: high
        verifymethod: test
    }
    requirement R4 {
        id: R4
        text: Lockfile must persist resolved package graph and dependency hash
        risk: high
        verifymethod: test
    }
    requirement R5 {
        id: R5
        text: Workspace detection must prefer Nx then Jet then package workspaces then pnpm
        risk: medium
        verifymethod: test
    }
    requirement R6 {
        id: R6
        text: Platform filters must skip incompatible optional dependencies
        risk: medium
        verifymethod: test
    }
    requirement R7 {
        id: R7
        text: Audit patch publish and store prune commands operate on package metadata and lockfile state
        risk: medium
        verifymethod: test
    }
```

### Scenarios

```yaml
scenarios:
  - id: S1
    requirement: R1
    title: Frozen install rejects dependency hash drift
  - id: S2
    requirement: R2
    title: Resolver uses cached metadata before registry fetch
  - id: S3
    requirement: R3
    title: Verified store package links into node_modules
  - id: S4
    requirement: R4
    title: Full resolve writes jet-lock.yaml with packages map
  - id: S5
    requirement: R5
    title: Workspace protocol resolves to local package version
  - id: S6
    requirement: R6
    title: Optional dependency with mismatched os is skipped
  - id: S7
    requirement: R7
    title: Store prune removes unreferenced global store entries
```

### Install State Machine

```mermaid
---
id: jet-pkg-manager-install-state
entry: ReadPackageJson
---
stateDiagram-v2
    [*] --> ReadPackageJson
    ReadPackageJson --> DetectWorkspace
    DetectWorkspace --> WorkspaceInstall : workspace mode
    DetectWorkspace --> CheckFrozen : single project
    CheckFrozen --> FrozenInstall : frozen or CI
    CheckFrozen --> CheckLockfile : mutable install
    FrozenInstall --> ValidateDepsHash
    ValidateDepsHash --> InstallFromLockfile : hash matches
    ValidateDepsHash --> Error : hash mismatch
    CheckLockfile --> CheckMarker : lockfile valid
    CheckLockfile --> FullResolve : missing or invalid
    CheckMarker --> UpToDate : marker matches
    CheckMarker --> InstallFromLockfile : marker mismatch
    FullResolve --> ResolveDependencyGraph
    ResolveDependencyGraph --> DownloadAndExtract
    DownloadAndExtract --> WriteLockfile
    WriteLockfile --> WriteMarker
    InstallFromLockfile --> LinkPackages
    WriteMarker --> LinkPackages
    LinkPackages --> RunScripts
    WorkspaceInstall --> RunScripts
    RunScripts --> [*]
    UpToDate --> [*]
    Error --> [*]
```

### Resolution Logic

```mermaid
---
id: jet-pkg-manager-resolution-logic
entry: A
---
flowchart TD
    A[Read direct dependencies] --> B[Apply overrides]
    B --> C[Queue packages for BFS resolve]
    C --> D{Metadata in L1 DashMap?}
    D -->|yes| E[Use metadata]
    D -->|no| F{Metadata in disk cache and fresh?}
    F -->|yes| E
    F -->|no| G[Fetch from registry]
    G --> H[Write L1 and L2 cache]
    H --> E
    E --> I[Pick highest semver-compatible version]
    I --> J{Alias or workspace protocol?}
    J -->|alias| K[Resolve alias target]
    J -->|workspace| L[Resolve local workspace package]
    J -->|normal| M[Resolve package tarball]
    K --> N[Record ResolvedPackage]
    L --> N
    M --> N
    N --> O{Version conflict?}
    O -->|yes| P[Mark nested_in parent]
    O -->|no| Q[Top-level package]
    P --> R[Enqueue transitive dependencies]
    Q --> R
    R --> C
```

### Dependency Model

```mermaid
---
id: jet-pkg-manager-dependencies
entry: PackageManager
---
classDiagram
    class PackageManager {
        +install() Result
        +add(package) Result
        +remove(package) Result
        +update(package) Result
    }
    class DependencyResolver {
        +resolve(deps) ResolvedGraph
    }
    class RegistryClient {
        +fetch_metadata(package) PackageMetadata
        +download_tarball(url) Bytes
    }
    class StoreManager {
        +install_package(name, version, tarball, shasum) PathBuf
        +link_package(name, version, target) Result
        +link_bins(package) Result
    }
    class Lockfile {
        +lockfileVersion
        +depsHash
        +packages
        +patchedPackages
    }
    class WorkspaceManager {
        +detect(root) WorkspaceMode
        +resolve_workspace_protocol(specifier) String
    }
    class Audit {
        +audit(lockfile) AuditReport
    }

    PackageManager --> DependencyResolver
    PackageManager --> StoreManager
    PackageManager --> Lockfile
    DependencyResolver --> RegistryClient
    DependencyResolver --> WorkspaceManager
    StoreManager --> Lockfile
    PackageManager --> Audit
```

### Schema

```yaml
PackageMetadata:
  source: crates/jet/src/pkg_manager/registry.rs
  fields:
    name: string
    dist_tags: "HashMap<String, String>"
    versions: "HashMap<String, VersionMetadata>"
VersionMetadata:
  fields:
    version: string
    dist:
      tarball: string
      shasum: string
      integrity: optional string
    dependencies: "HashMap<String, String>"
    peerDependencies: "HashMap<String, String>"
    optionalDependencies: "HashMap<String, String>"
Lockfile:
  source: crates/jet/src/pkg_manager/lockfile.rs
  persisted_as: jet-lock.yaml
  fields:
    lockfileVersion: "2.0"
    depsHash: string
    overrides: "HashMap<String, String>"
    patchedPackages: "HashMap<String, String>"
    packages: "HashMap<String, LockPackage>"
LockPackage:
  fields:
    version: string
    resolution:
      tarball: string
      shasum: string
      integrity: optional string
    workspace: boolean
    localPath: optional string
    dependencies: "HashMap<String, String>"
    peerDependencies: "HashMap<String, String>"
    bin: "HashMap<String, String>"
    nestedIn: optional string
AuditReport:
  source: crates/jet/src/pkg_manager/audit.rs
  fields:
    vulnerabilities: array
    summary:
      critical: integer
      high: integer
      moderate: integer
      low: integer
      total: integer
```

### Test Plan

```mermaid
---
id: jet-pkg-manager-test-plan
entry: T1
---
requirementDiagram
    requirement R2 {
        id: R2
        text: resolver cache and metadata
        risk: high
        verifymethod: test
    }
    requirement R4 {
        id: R4
        text: lockfile persistence
        risk: high
        verifymethod: test
    }
    requirement R5 {
        id: R5
        text: workspace detection
        risk: medium
        verifymethod: test
    }
    requirement R6 {
        id: R6
        text: platform filtering
        risk: medium
        verifymethod: test
    }
    element T1 {
        type: test
        docref: cargo test -p jet pkg_manager::
    }
    element T2 {
        type: test
        docref: cargo test -p jet pkg_manager::nx_test
    }
```

### Execution

```bash
cargo test -p jet pkg_manager::
cargo test -p jet pkg_manager::nx_test
```

### Changes

```yaml
files:
  - path: .aw/tech-design/crates/jet/logic/pkg-manager.md
    action: MODIFY
    section: doc
    impl_mode: hand-written
    desc: Replace legacy package-manager architecture prose with a checker-compliant current-state contract.

  - path: crates/jet/src/pkg_manager/mod.rs
    action: NONE
    section: doc
    impl_mode: hand-written
    desc: Existing implementation owns install orchestration and command surface.

  - path: crates/jet/src/pkg_manager/resolver.rs
    action: NONE
    section: doc
    impl_mode: hand-written
    desc: Existing implementation owns dependency resolution, metadata cache, aliases, and conflicts.

  - path: crates/jet/src/pkg_manager/store.rs
    action: NONE
    section: doc
    impl_mode: hand-written
    desc: Existing implementation owns global store extraction, verification, symlinks, and bins.

  - path: crates/jet/src/pkg_manager/workspace.rs
    action: NONE
    section: doc
    impl_mode: hand-written
    desc: Existing implementation owns Jet and pnpm workspace detection and local dependency links.
```
