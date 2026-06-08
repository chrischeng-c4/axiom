---
change: platform-sync
date: 2026-02-03
---

# Clarifications

## Q1: Changelog
- **Question**: CHANGELOG.md content
- **Answer**: ## 2026-02-03: Platform Sync MCP Tool (platform-sync)
Add a publish-only Platform Sync MCP tool that syncs Genesis change artifacts to GitHub using gh CLI auth.
- **Platform Sync Tool**: New `genesis_platform_sync` MCP tool that syncs change artifacts (proposal, specs, tasks) to GitHub.
- **Configurable Sync**: Project-level configuration in `cclab/config.yaml` for platform type, repo, and token source.
- **Deterministic Payloads**: Generates consistent markdown payloads with hash verification for idempotent updates.
- **Metadata Persistence**: Tracks sync status in `SYNC.yaml` within the change directory.
- **GitHub Provider**: Supports both token-based and `gh` CLI authentication for seamless integration.
- Related specs: None
- **Rationale**: User requested changelog generation

