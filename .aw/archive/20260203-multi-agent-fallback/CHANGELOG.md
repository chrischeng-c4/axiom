# Changelog

## 2026-01-29: Multi-agent fallback per workflow stage (multi-agent-fallback)

### Added
- Per-stage agent selection in `cclab/config.toml` for plan, implementation, and merge executors/reviewers, including ordered fallbacks and `agent:model` specs.
- Automatic quota-aware fallback between agents with a 5-second delay and clear "all agents exhausted" errors when every agent fails.
- Quota detection across stderr/stdout and JSON error payloads to trigger fallback when providers report rate limits.
- Config validation that rejects empty, duplicate, or unknown agent entries in stage lists.

### Changed
- Proposal generation/review and archive changelog generation now use the stage agent lists with fallback behavior.
- Config templates include the new workflow stage sections and `mainthread` option for in-process execution.
