---
id: libs-h2c-src-llm-rs
summary: Lossless rust-source-unit coverage for `libs/h2c/src/llm.rs`.
capability_refs:
  - id: http2-cleartext-client-helpers
    role: primary
    claim: http2-cleartext-client-helpers-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the H2c library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/h2c/src/llm.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/h2c/src/llm.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TOPIC` | libs/h2c/src/llm.rs | const | pub | 4 | pub const TOPIC: cli_std::llm::Topic = cli_std::llm::Topic { |
| `topic` | libs/h2c/src/llm.rs | function | pub | 59 | pub fn topic() -> &'static cli_std::llm::Topic { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! LLM topic provider for the shared h2c client/server transport contract.

/// Agent-facing topic describing h2c outbound client pools and server boundary.
pub const TOPIC: cli_std::llm::Topic = cli_std::llm::Topic {
    id: "h2c",
    summary: "Shared HTTP/2 cleartext client helpers, logarithmic connection-pool sizing, and optional server accept loop.",
    body: r#"# h2c shared topic

## Client-side pool sizing
Use the h2c pool on outbound callers: generated clients, adapters, raft peers,
and service-to-service calls. A single HTTP/2 connection multiplexes many
streams, but its framing work still bottlenecks on one task/core under heavy
concurrency. Size the outbound pool from target peak concurrency:

```text
connections = clamp(ceil(ln(concurrency)), 1, cpu_parallelism)
```

`target_concurrency` is a sizing hint, not a throughput promise. The peer
server decides usable request concurrency through stream limits, latency, and
work capacity. Keep agent-facing config protocol-neutral:

```text
max_connections = 128
max_keepalive_connections = 16
max_in_flight_per_origin = target_concurrency or 128
pool_timeout = 5s
```

HTTP/2 runtimes map the target to logarithmic connection count; HTTP/1.1
runtimes use the same abstract knobs but need more sockets. Both modes should
admit only `max_in_flight_per_origin` requests per origin and queue excess work
until `pool_timeout` instead of spawning unbounded client concurrency.

Rust callers can use:

```rust
let pool = h2c::H2cPool::for_concurrency(target_concurrency)?;
let resp = pool.post("http://lumen:7373/collections/products/search").send().await?;
```

Use `recommended_h2c_connections` or `recommended_h2c_connections_for` when a
CLI, generated runtime, or non-Rust client needs the same sizing rule.

## Managed outbound connections
For long-lived service-to-service traffic where GOAWAY, liveness, retry-on-lost
connection, and adaptive grow/shrink matter, use `H2cManager`. `H2cPool` is the
lighter reqwest-level round-robin pool.

## Server boundary
The `server` feature exposes `h2c::serve` so a service can accept HTTP/1.1 and
HTTP/2 cleartext on one port. Server code should not create a connection pool
for inbound traffic; it should accept enough h2 concurrent streams and let each
outbound caller manage its own pool.
"#,
};

/// Return the shared h2c topic for CLI composition.
pub fn topic() -> &'static cli_std::llm::Topic {
    &TOPIC
}

#[cfg(test)]
mod tests {
    #[test]
    fn llm_topic_is_nonempty() {
        let topic = super::topic();
        assert_eq!(topic.id, "h2c");
        assert!(topic.body.contains("ceil(ln(concurrency))"));
        assert!(topic.body.contains("max_in_flight_per_origin"));
        assert!(topic.body.contains("pool_timeout"));
        assert!(topic.body.contains("H2cPool::for_concurrency"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/h2c/src/llm.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/h2c/src/llm.rs` captured during libs codegen standardization.
```
