# service-executor

## Brief

`service-executor` supplies application-neutral bounded asynchronous execution.
It deliberately does not own durable assignment, fencing, retry, external
target semantics, or outcome persistence; those remain committed domain state.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Bounded Async Execution | #1854 | implemented | verified | conformance | ready | completion-order execution with a hard in-flight ceiling |

### Bounded Async Execution

ID: bounded-async-execution
Type: Runtime
Root WI: 1854
Status: verified
Surfaces: Rust API: `service_executor::run_bounded`.
EC Dimensions: behavior: `cargo test -p service-executor` - every item runs, configured concurrency is never exceeded, and zero normalizes to one
Required Verification: conformance
Promise:
Services can execute a finite set of already-authorized work concurrently
without cloning concurrency plumbing. The helper returns every result in
completion order and enforces a non-zero concurrency ceiling. Durable ownership
and permission to cause an external effect must be established by the caller
before submitting work.
Gate Inventory: `cargo test -p service-executor`; libs/service-executor/src/lib.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| bounded-async-execution | change | #1854 | implemented | verified | conformance | `cargo test -p service-executor` |
