---
id: projects-meter-src-report-schema-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/report/schema.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/report/schema.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `catalog` | projects/meter/src/report/schema.rs | function | pub | 152 | catalog() -> Value |
| `json_schema` | projects/meter/src/report/schema.rs | function | pub | 20 | json_schema() -> Value |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Offline self-describers for `meter spec`.
//!
//! [`json_schema`] returns a hand-written, deterministic JSON-Schema for
//! [`MeterReport`](super::MeterReport) (no `schemars` dependency — the shape is stable
//! and byte-comparable). [`catalog`] lists the severities, kinds, and per-kind
//! evidence shapes so an agent can discover the closed sets without reading code.

use serde_json::{json, Value};

use super::envelope::SCHEMA_VERSION;
use super::finding::{Kind, Severity};

/// A hand-written JSON-Schema (draft 2020-12) for [`MeterReport`](super::MeterReport).
///
/// Deterministic by construction: built from object literals in a fixed order,
/// so repeated `--compact` emissions are byte-identical.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-schema-rs.md#source
pub fn json_schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://cclab.dev/schema/meter.report/1",
        "title": "MeterReport",
        "description": "The one self-describing document every meter verb prints to stdout.",
        "type": "object",
        "x-schema-version": SCHEMA_VERSION,
        "required": [
            "schema_version", "tool_version", "verb", "target", "status",
            "clean", "exit_code", "terminal", "summary", "findings",
            "environment", "completion", "agent_prompt"
        ],
        "properties": {
            "schema_version": { "type": "string", "const": SCHEMA_VERSION },
            "tool_version": { "type": "string" },
            "verb": {
                "type": "string",
                "enum": ["report", "profile", "bench", "test", "run", "spec", "llm"]
            },
            "target": { "type": "string" },
            "status": { "$ref": "#/$defs/OverallStatus" },
            "clean": { "type": "boolean", "description": "mirror of status==clean" },
            "exit_code": { "type": "integer", "description": "the process exit code" },
            "terminal": { "type": "boolean" },
            "last_run": { "$ref": "#/$defs/RunnerRecord" },
            "summary": { "$ref": "#/$defs/FindingsSummary" },
            "findings": { "type": "array", "items": { "$ref": "#/$defs/Finding" } },
            "environment": { "$ref": "#/$defs/EnvBlock" },
            "completion": { "$ref": "#/$defs/Completion" },
            "agent_prompt": { "type": "string" },
            "requires_hitl": { "type": "boolean" }
        },
        "$defs": {
            "OverallStatus": {
                "type": "object",
                "description": "Tagged union on `state`; SOLE source of exit_code/clean/terminal.",
                "required": ["state"],
                "properties": {
                    "state": { "type": "string", "enum": ["clean", "findings", "regression", "tool_error"] },
                    "count": { "type": "integer" },
                    "code": { "type": "integer" },
                    "message": { "type": "string" }
                }
            },
            "FindingsSummary": {
                "type": "object",
                "required": ["critical", "high", "medium", "low", "info", "total", "truncated", "sample"],
                "properties": {
                    "critical": { "type": "integer" },
                    "high": { "type": "integer" },
                    "medium": { "type": "integer" },
                    "low": { "type": "integer" },
                    "info": { "type": "integer" },
                    "total": { "type": "integer" },
                    "truncated": { "type": "boolean" },
                    "sample": { "type": "array", "items": { "$ref": "#/$defs/Finding" } },
                    "payload_path": { "type": ["string", "null"] }
                }
            },
            "Completion": {
                "type": "object",
                "required": ["clean", "criteria", "missing"],
                "properties": {
                    "clean": { "type": "boolean" },
                    "criteria": { "type": "array", "items": { "type": "string" } },
                    "missing": { "type": "array", "items": { "type": "string" } }
                }
            },
            "RunnerRecord": {
                "type": "object",
                "required": ["command", "kind", "started_at"],
                "properties": {
                    "command": { "type": "array", "items": { "type": "string" } },
                    "kind": { "type": "string" },
                    "started_at": { "type": "string", "format": "date-time" },
                    "finished_at": { "type": ["string", "null"], "format": "date-time" },
                    "exit_code": { "type": ["integer", "null"], "description": "FORWARDED child exit code" },
                    "duration_ms": { "type": ["integer", "null"] },
                    "delegated": { "type": "boolean" }
                }
            },
            "EnvBlock": {
                "type": "object",
                "required": ["os", "arch", "nextest_present", "sampler_backend", "note"],
                "properties": {
                    "os": { "type": "string" },
                    "arch": { "type": "string" },
                    "nextest_present": { "type": "boolean" },
                    "sampler_backend": { "type": "string", "enum": ["macos-sample", "linux-perf", "none"] },
                    "rustc_version": { "type": ["string", "null"] },
                    "note": { "type": "string" }
                }
            },
            "Finding": {
                "type": "object",
                "required": ["id", "severity", "kind", "title", "detail", "remediation", "invoke", "evidence"],
                "properties": {
                    "id": { "type": "string", "description": "stable {kind}:{slug}" },
                    "severity": { "type": "string", "enum": severity_enum() },
                    "kind": { "type": "string", "enum": kind_enum() },
                    "title": { "type": "string" },
                    "detail": { "type": "string" },
                    "remediation": { "type": "string" },
                    "invoke": { "$ref": "#/$defs/Invoke" },
                    "evidence": { "description": "per-kind structured proof; see meter spec --catalog" },
                    "location": { "$ref": "#/$defs/Location" }
                }
            },
            "Invoke": {
                "type": "object",
                "required": ["command"],
                "properties": {
                    "command": { "type": "string" },
                    "args": { "description": "free-form structured args" }
                }
            },
            "Location": {
                "type": "object",
                "properties": {
                    "file": { "type": ["string", "null"] },
                    "line": { "type": ["integer", "null"] },
                    "symbol": { "type": ["string", "null"] }
                }
            }
        }
    })
}

/// A discovery catalog: closed severity/kind sets and per-kind evidence shapes.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-schema-rs.md#source
pub fn catalog() -> Value {
    json!({
        "schema_version": SCHEMA_VERSION,
        "severities": severity_enum(),
        "kinds": kind_catalog(),
        "exit_codes": {
            "0": "clean — no findings",
            "1": "findings — issues surfaced",
            "2": "regression — performance regression(s)",
            "3": "tool_error — usage/bad-flag",
            "4": "tool_error — required tool missing",
            "5": "tool_error — io / spawn failure"
        }
    })
}

/// The snake_case severity labels, most-severe first.
fn severity_enum() -> Vec<&'static str> {
    Severity::all().iter().map(|s| s.as_str()).collect()
}

/// The snake_case kind labels, in declaration order.
fn kind_enum() -> Vec<&'static str> {
    Kind::all().iter().map(|k| k.as_str()).collect()
}

/// Per-kind entries: producing verb + evidence field names.
fn kind_catalog() -> Value {
    json!([
        { "kind": "hotspot", "verb": "profile", "evidence": ["symbol", "self_ns", "total_ns", "pct", "samples", "rank"] },
        { "kind": "boundary_cost", "verb": "profile", "evidence": ["phase", "self_ns", "total_ns", "pct", "samples"] },
        { "kind": "regression", "verb": "bench", "evidence": ["bench", "baseline_ms", "current_ms", "percent_change", "severity", "ci_overlap"] },
        { "kind": "test_failure", "verb": "test", "evidence": ["name", "stdout_tail"] },
        { "kind": "vital", "verb": "profile", "evidence": ["cpu_time_ms", "wall_time_ms", "peak_rss_bytes"] }
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_title_is_qcreport() {
        let s = json_schema();
        assert_eq!(s["title"], "MeterReport");
        assert!(s["properties"]["findings"].is_object());
    }

    #[test]
    fn schema_is_deterministic() {
        let a = serde_json::to_string(&json_schema()).unwrap();
        let b = serde_json::to_string(&json_schema()).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn catalog_lists_all_kinds() {
        let c = catalog();
        let kinds = c["kinds"].as_array().unwrap();
        assert_eq!(kinds.len(), Kind::all().len());
    }

    #[test]
    fn severity_enum_matches_all() {
        assert_eq!(severity_enum().len(), 5);
        assert_eq!(severity_enum()[0], "critical");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/report/schema.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/report/schema.rs` captured during meter full-codegen standardization.
```
