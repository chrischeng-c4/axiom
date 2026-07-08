---
id: libs-compass-src-server-protocol-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/server/protocol.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/server/protocol.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/server/protocol.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Request` | libs/compass/src/server/protocol.rs | struct | pub | 7 | pub struct Request { |
| `new` | libs/compass/src/server/protocol.rs | function | pub | 16 | pub fn new(id: impl Into<RequestId>, method: &str, params: Option<serde_json::Value>) -> Self { |
| `Response` | libs/compass/src/server/protocol.rs | struct | pub | 28 | pub struct Response { |
| `success` | libs/compass/src/server/protocol.rs | function | pub | 38 | pub fn success(id: RequestId, result: serde_json::Value) -> Self { |
| `error` | libs/compass/src/server/protocol.rs | function | pub | 47 | pub fn error(id: RequestId, error: RpcError) -> Self { |
| `RpcError` | libs/compass/src/server/protocol.rs | struct | pub | 59 | pub struct RpcError { |
| `parse_error` | libs/compass/src/server/protocol.rs | function | pub | 67 | pub fn parse_error(msg: impl Into<String>) -> Self { |
| `invalid_request` | libs/compass/src/server/protocol.rs | function | pub | 75 | pub fn invalid_request(msg: impl Into<String>) -> Self { |
| `method_not_found` | libs/compass/src/server/protocol.rs | function | pub | 83 | pub fn method_not_found(method: &str) -> Self { |
| `invalid_params` | libs/compass/src/server/protocol.rs | function | pub | 91 | pub fn invalid_params(msg: impl Into<String>) -> Self { |
| `internal_error` | libs/compass/src/server/protocol.rs | function | pub | 99 | pub fn internal_error(msg: impl Into<String>) -> Self { |
| `RequestId` | libs/compass/src/server/protocol.rs | enum | pub | 111 | pub enum RequestId { |
| `CheckParams` | libs/compass/src/server/protocol.rs | struct | pub | 140 | pub struct CheckParams { |
| `TypeAtParams` | libs/compass/src/server/protocol.rs | struct | pub | 146 | pub struct TypeAtParams { |
| `SymbolsParams` | libs/compass/src/server/protocol.rs | struct | pub | 154 | pub struct SymbolsParams { |
| `DiagnosticsParams` | libs/compass/src/server/protocol.rs | struct | pub | 160 | pub struct DiagnosticsParams { |
| `HoverParams` | libs/compass/src/server/protocol.rs | struct | pub | 167 | pub struct HoverParams { |
| `DefinitionParams` | libs/compass/src/server/protocol.rs | struct | pub | 175 | pub struct DefinitionParams { |
| `ReferencesParams` | libs/compass/src/server/protocol.rs | struct | pub | 183 | pub struct ReferencesParams { |
| `DiagnosticInfo` | libs/compass/src/server/protocol.rs | struct | pub | 197 | pub struct DiagnosticInfo { |
| `SymbolInfo` | libs/compass/src/server/protocol.rs | struct | pub | 210 | pub struct SymbolInfo { |
| `LocationInfo` | libs/compass/src/server/protocol.rs | struct | pub | 221 | pub struct LocationInfo { |
| `IndexStatus` | libs/compass/src/server/protocol.rs | struct | pub | 231 | pub struct IndexStatus { |
| `CheckResult` | libs/compass/src/server/protocol.rs | struct | pub | 240 | pub struct CheckResult { |
| `PdgParams` | libs/compass/src/server/protocol.rs | struct | pub | 253 | pub struct PdgParams { |
| `SliceParams` | libs/compass/src/server/protocol.rs | struct | pub | 261 | pub struct SliceParams { |
| `ImpactParams` | libs/compass/src/server/protocol.rs | struct | pub | 271 | pub struct ImpactParams { |
| `TaintParams` | libs/compass/src/server/protocol.rs | struct | pub | 278 | pub struct TaintParams { |
| `SliceNodeInfo` | libs/compass/src/server/protocol.rs | struct | pub | 290 | pub struct SliceNodeInfo { |
| `SliceResult` | libs/compass/src/server/protocol.rs | struct | pub | 298 | pub struct SliceResult { |
| `ImpactNode` | libs/compass/src/server/protocol.rs | struct | pub | 307 | pub struct ImpactNode { |
| `ImpactResult` | libs/compass/src/server/protocol.rs | struct | pub | 322 | pub struct ImpactResult { |
| `TaintPathInfo` | libs/compass/src/server/protocol.rs | struct | pub | 333 | pub struct TaintPathInfo { |
| `TaintResult` | libs/compass/src/server/protocol.rs | struct | pub | 345 | pub struct TaintResult { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! JSON-RPC protocol definitions for Argus daemon

use serde::{Deserialize, Serialize};

/// JSON-RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl Request {
    pub fn new(id: impl Into<RequestId>, method: &str, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method: method.to_string(),
            params,
        }
    }
}

/// JSON-RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl Response {
    pub fn success(id: RequestId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: RequestId, error: RpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl RpcError {
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self {
            code: -32700,
            message: msg.into(),
            data: None,
        }
    }

    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: msg.into(),
            data: None,
        }
    }

    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method),
            data: None,
        }
    }

    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: msg.into(),
            data: None,
        }
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: msg.into(),
            data: None,
        }
    }
}

/// Request ID (can be string or number)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

impl From<i64> for RequestId {
    fn from(n: i64) -> Self {
        RequestId::Number(n)
    }
}

impl From<&str> for RequestId {
    fn from(s: &str) -> Self {
        RequestId::String(s.to_string())
    }
}

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        RequestId::String(s)
    }
}

// ============================================================================
// Method-specific types
// ============================================================================

/// Parameters for check method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckParams {
    pub path: String,
}

/// Parameters for type_at method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeAtParams {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Parameters for symbols method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolsParams {
    pub file: String,
}

/// Parameters for diagnostics method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

/// Parameters for hover method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverParams {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Parameters for definition method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionParams {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Parameters for references method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferencesParams {
    pub file: String,
    pub line: u32,
    pub column: u32,
    #[serde(default)]
    pub include_declaration: bool,
}

// ============================================================================
// Response types
// ============================================================================

/// Diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticInfo {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub severity: String,
    pub code: String,
    pub message: String,
}

/// Symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String,
    pub line: u32,
    pub column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_info: Option<String>,
}

/// Location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

/// Index status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatus {
    pub indexed_files: usize,
    pub total_symbols: usize,
    pub last_updated: Option<String>,
    pub is_ready: bool,
}

/// Check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub diagnostics: Vec<DiagnosticInfo>,
    pub files_checked: usize,
    pub errors: usize,
    pub warnings: usize,
}

// ============================================================================
// PDG method types
// ============================================================================

/// Parameters for pdg method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdgParams {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
}

/// Parameters for slice method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceParams {
    pub file: String,
    /// Line number (0-indexed)
    pub line: usize,
    /// "forward" or "backward"
    pub direction: String,
}

/// Parameters for impact method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactParams {
    pub file: String,
    pub changed_lines: Vec<usize>,
}

/// Parameters for taint method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintParams {
    pub file: String,
    /// Optional explicit source lines; if empty, auto-detects from code patterns
    #[serde(default)]
    pub sources: Vec<usize>,
    /// Optional explicit sink lines; if empty, auto-detects from code patterns
    #[serde(default)]
    pub sinks: Vec<usize>,
}

/// A slice node (statement) in the result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceNodeInfo {
    pub line: usize,
    pub text: String,
    pub kind: String,
}

/// Slice result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceResult {
    pub direction: String,
    pub criterion_line: usize,
    pub nodes: Vec<SliceNodeInfo>,
    pub line_count: usize,
}

/// A node in the impact dependency tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactNode {
    pub line: usize,
    pub text: String,
    /// "data", "control", or "transitive"
    pub reason: String,
    /// Variable name for data dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variable: Option<String>,
    /// Further downstream impacts
    #[serde(default)]
    pub children: Vec<ImpactNode>,
}

/// Impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactResult {
    pub changed_lines: Vec<usize>,
    /// Flat list of affected lines
    pub affected_lines: Vec<usize>,
    /// Tree showing WHY each line is affected
    pub impact_tree: Vec<ImpactNode>,
    pub total_affected: usize,
}

/// A taint path from source to sink
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintPathInfo {
    pub source_line: usize,
    pub source_text: String,
    pub source_kind: String,
    pub sink_line: usize,
    pub sink_text: String,
    pub sink_kind: String,
    pub path: Vec<usize>,
}

/// Taint analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintResult {
    pub source_lines: Vec<usize>,
    pub sink_lines: Vec<usize>,
    pub taint_paths: Vec<TaintPathInfo>,
    pub has_vulnerabilities: bool,
    pub auto_detected: bool,
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/server/protocol.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/server/protocol.rs` captured during libs codegen standardization.
```
