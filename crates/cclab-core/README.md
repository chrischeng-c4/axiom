# Cclab Core

## Brief

Cclab Core is the small shared Rust support crate for ecosystem-wide error,
HTTP, and utility contracts.

It owns the common `DataBridgeError` / `Result` surface, production-safe error
sanitization helpers, and lightweight HTTP request/response abstractions used
by higher-level cclab crates.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Unified Error Contract | - | implemented | passing | smoke | not_ready | shared `DataBridgeError` and `Result` contract for cclab crates |
| Sanitized Error Reporting | - | implemented | passing | conformance | not_ready | redacts connection strings, credentials, internal IPs, and tokens before production logging |
| HTTP Helper Contracts | - | implemented | passing | smoke | not_ready | shared HTTP method, status, request, and response helper traits |

### Unified Error Contract

ID: unified-error-contract
Type: RuntimeTool
Surfaces: Rust API: `cclab_core::DataBridgeError`, `Result`, `is_retryable`, `is_constraint_violation`
EC Dimensions: behavior: `cargo test -p cclab-core` - shared error display, conversion, retry, and constraint behavior
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Core gives Rust crates one shared error/result contract with stable display text, serde conversion, retry classification, and constraint violation classification.
Gate Inventory: `cargo test -p cclab-core`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Error display and classification contract | epic | - | implemented | passing | smoke | `cargo test -p cclab-core` |

### Sanitized Error Reporting

ID: sanitized-error-reporting
Type: SecurityTool
Surfaces: Rust API: `sanitize_error`, `sanitize_error_message`, `categorize_error`, `ErrorCategory`
EC Dimensions: security: `cargo test -p cclab-core` - connection string, credential, IP, and token redaction behavior
Root WI: -
Status: confirmed
Required Verification: smoke, conformance
Promise:
Cclab Core provides production-safe error sanitization and categorization so crates can log failures without leaking connection strings, credentials, internal IPs, or auth tokens.
Gate Inventory: `cargo test -p cclab-core`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Error sanitization contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-core` |

### HTTP Helper Contracts

ID: http-helper-contracts
Type: DeveloperTool
Surfaces: Rust API: `HttpMethod`, `HttpStatus`, `HttpRequestLike`, `HttpResponseLike`
EC Dimensions: behavior: `cargo test -p cclab-core` - HTTP method parsing, status classification, and trait helper behavior
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Core provides lightweight HTTP helper types and traits that let ecosystem crates share method parsing, status classification, header lookup, and response/request behavior.
Gate Inventory: `cargo test -p cclab-core`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| HTTP method and status helper contract | epic | - | implemented | passing | smoke | `cargo test -p cclab-core` |
