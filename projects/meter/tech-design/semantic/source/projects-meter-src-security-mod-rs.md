---
id: projects-meter-src-security-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: legacy-carried-internals
    role: primary
    gap: seeded-fuzz-and-injection-finding-generation
    claim: seeded-fuzz-and-injection-finding-generation
    coverage: full
    rationale: "Source template implements meter security, fuzzing, injection, or audit surfaces."
---

# Standardized projects/meter/src/security/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/security/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Security testing framework for cclab
//!
//! Provides security-focused testing utilities:
//! - SQL injection detection and prevention testing
//! - Fuzzing framework for input validation (sync and async)
//! - Payload databases for security tests
//!
//! # Example
//! ```rust,ignore
//! use meter::security::{PayloadDatabase, Fuzzer, FuzzConfig};
//!
//! // Test SQL injection prevention
//! let payloads = PayloadDatabase::new();
//! for payload in payloads.sql_injection() {
//!     let result = validate_identifier(payload);
//!     assert!(result.is_err(), "Should block: {}", payload);
//! }
//!
//! // Fuzz test an input validator (sync)
//! let config = FuzzConfig::default().with_iterations(1000);
//! let fuzzer = Fuzzer::new(config);
//! let result = fuzzer.fuzz(|input| validate_input(input));
//! assert!(result.crashes.is_empty());
//!
//! // Async fuzzing
//! use meter::security::{AsyncFuzzer, AsyncFuzzConfig};
//!
//! # async fn async_example() {
//! let config = AsyncFuzzConfig::new().with_iterations(1000);
//! let mut fuzzer = AsyncFuzzer::new(config);
//! let result = fuzzer.fuzz_async(|input| async move {
//!     validate_async(input).await
//! }).await;
//! # }
//! ```

mod async_fuzzer;
mod fuzzer;
mod payloads;
mod sql_injection;

pub use async_fuzzer::{AsyncFuzzConfig, AsyncFuzzer};
pub use fuzzer::{FuzzConfig, FuzzCrash, FuzzResult, Fuzzer, MutationStrategy};
pub use payloads::{PayloadCategory, PayloadDatabase};
pub use sql_injection::{InjectionResult, InjectionTest, SqlInjectionTester};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/security/mod.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/security/mod.rs` captured during meter full-codegen standardization.
```
