---
id: platform-sync
type: tasks
version: 1
created_at: 2026-02-03T04:00:00.000000+00:00
updated_at: 2026-02-03T04:00:00.000000+00:00
source: proposal
total_tasks: 5
completed_tasks: 5
---

# Implementation Tasks

## T1: Create platform sync service with payload builder

**Priority**: high
**Complexity**: medium
**Files**: `crates/cclab-genesis/src/services/mod.rs`, `crates/cclab-genesis/src/services/platform_sync.rs`

### Description

Create the core platform sync service that:
- Reads change artifacts (proposal.md, specs/*.md, tasks.md)
- Builds a deterministic markdown payload
- Computes SHA256 hash for idempotent updates
- Defines the `SyncProvider` trait for platform backends

### Acceptance Criteria

- [x] `PlatformSyncService` struct with `build_payload()` method
- [x] `SyncProvider` trait with `sync()` method
- [x] Payload includes proposal summary, specs list, tasks status
- [x] Hash computed from payload content (includes title, body, labels)

---

## T2: Implement GitHub provider using gh CLI

**Priority**: high
**Complexity**: medium
**Files**: `crates/cclab-genesis/src/services/platform_sync/github.rs`

### Description

Create GitHub provider that:
- Uses `gh` CLI for authentication (no separate tokens)
- Creates GitHub issue if not exists
- Updates existing issue if payload changed
- Checks `gh auth status` before operations

### Acceptance Criteria

- [x] `GitHubProvider` implements `SyncProvider`
- [x] Creates issue with `gh issue create`
- [x] Updates issue with `gh issue edit`
- [x] Clear error when `gh` not installed or not authenticated

---

## T3: Add sync metadata persistence

**Priority**: medium
**Complexity**: low
**Files**: `crates/cclab-genesis/src/services/platform_sync.rs`

### Description

Persist sync state in change directory:
- Store in `cclab/changes/<id>/SYNC.yaml`
- Track: platform, issue_url, issue_number, last_sync, payload_hash

### Acceptance Criteria

- [x] `SyncMetadata` struct with serde serialization
- [x] Read/write to SYNC.yaml
- [x] Skip sync if payload hash unchanged (also checks platform and repo match)

---

## T4: Expose MCP tool genesis_platform_sync

**Priority**: high
**Complexity**: medium
**Files**: `crates/cclab-genesis/src/mcp/tools/mod.rs`, `crates/cclab-genesis/src/mcp/tools/platform_sync.rs`

### Description

Add new MCP tool:
- Tool name: `genesis_platform_sync`
- Inputs: project_path, change_id, platform (default: "github"), repo (optional)
- Returns: issue URL, sync status

### Acceptance Criteria

- [x] Tool registered in genesis MCP server
- [x] Validates change exists before sync
- [x] Returns created/updated/unchanged status

---

## T5: Add unit tests

**Priority**: medium
**Complexity**: low
**Files**: `crates/cclab-genesis/src/services/platform_sync.rs` (inline tests)

### Description

Add tests for:
- Payload building from mock change artifacts
- Metadata read/write
- Hash computation stability
- Mock command runner for gh CLI tests

### Acceptance Criteria

- [x] Test payload includes all sections
- [x] Test metadata serialization roundtrip
- [x] Test hash changes when content changes
