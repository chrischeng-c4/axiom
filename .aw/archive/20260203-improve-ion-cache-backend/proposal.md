---
id: improve-ion-cache-backend
type: proposal
version: 1
created_at: 2026-01-30T06:13:23.500703+00:00
updated_at: 2026-01-30T06:13:23.500703+00:00
author: mcp
status: proposed
iteration: 1
summary: "Implement advanced cache features: TTL management, Hash/List ops, and memory eviction."
history:
  - timestamp: 2026-01-30T06:13:23.500703+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-30T06:14:56.448944+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-30T06:15:08.865636+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 10
  new_files: 4
affected_specs:
  - id: ttl-management
    path: specs/ttl-management.md
    depends: []
  - id: hash-ops
    path: specs/hash-ops.md
    depends: []
  - id: list-ops
    path: specs/list-ops.md
    depends: []
  - id: memory-eviction
    path: specs/memory-eviction.md
    depends: []---

<proposal>

# Change: improve-ion-cache-backend

## Summary

Implement advanced cache features: TTL management, Hash/List ops, and memory eviction.

## Why

To support complex caching scenarios and result backend requirements (e.g., for task queues like Celery), cclab-ion needs richer data structure support (Hashes, Lists) and finer-grained control over key expiration. Additionally, production usage requires memory safety mechanisms like eviction policies to prevent OOM errors.

## What Changes

- Add TTL commands: EXPIRE, PEXPIRE, TTL, PTTL, GETEX, PERSIST
- Add Hash commands: HSET, HGET, HMSET, HMGET, HGETALL, HDEL, HEXISTS, HLEN
- Add List commands: LPUSH, RPUSH, LPOP, RPOP, LRANGE, LLEN, BLPOP, BRPOP
- Implement memory eviction policies: maxmemory limit, allkeys-lru, volatile-lru, allkeys-lfu, noeviction
- Update configuration to support maxmemory and eviction policy settings

## Impact

- **Scope**: minor
- **Affected Files**: ~10
- **New Files**: ~4
- Affected specs:
  - `ttl-management` (no dependencies)
  - `hash-ops` (no dependencies)
  - `list-ops` (no dependencies)
  - `memory-eviction` (no dependencies)
- Affected code: `crates/cclab-ion/src/engine.rs`, `crates/cclab-ion/src/types.rs`, `crates/cclab-ion-server/src/protocol.rs`, `crates/cclab-ion-server/src/server.rs`, `crates/cclab-ion/src/config.rs`
- **Breaking Changes**: No. Default configuration will be `maxmemory 0` (unlimited) and `maxmemory-policy noeviction` to maintain backward compatibility.

</proposal>
