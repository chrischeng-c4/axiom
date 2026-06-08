---
change: sdd-tdd-gate
group: test-config
date: 2026-04-08
---

# Requirements

PR1 — Add [sdd.test] config parsing to SddConfig (data only, no workflow logic):

1. Add TestConfig struct with optional `setup`, `teardown` commands and a `scope` vec of TestScope entries to `crates/sdd/src/models/change.rs`.
2. Add TestScope struct with fields: `name` (string), `changes` (Vec<String> — GitLab CI-style glob patterns matching file paths), `test_cmd` (string — command to run tests), optional `setup` and `teardown` overrides.
3. Add `test: Option<TestConfig>` field to SddConfig, mapped from `[sdd.test]` TOML section.
4. Update SddConfig::load() to deserialize `[sdd.test]` and `[[sdd.test.scope]]` entries.
5. Add test scope entries in `.score/config.toml` for conductor (changes: ["projects/conductor/**"]) and cclab-queue (changes: ["crates/cclab-queue/**"]).
6. Acceptance: config round-trips through TOML serialize/deserialize, glob patterns in `changes` are stored as strings (matching deferred to PR2).
