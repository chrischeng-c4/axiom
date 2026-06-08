// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-security-mod-rs.md#source
// CODEGEN-BEGIN
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
// CODEGEN-END
