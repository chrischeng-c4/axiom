---
id: projects-rig-src-scenario-interp-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/scenario/interp.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/scenario/interp.rs`, captured as a rust-source-unit (td_ast) item-tree
during rig standardization onto the codegen ladder.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `{{var}}` interpolation over a flat variable store.
//!
//! Vars come from three layers, later wins: scenario `[env]` defaults,
//! `RIG_VAR_<NAME>` OS-environment overrides, then values captured by
//! steps at run time. Values are stored as JSON values; interpolation
//! renders them as bare strings (no quotes around strings, plain digits
//! for numbers).

use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct VarStore {
    vars: BTreeMap<String, Value>,
}

impl VarStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed from a scenario's `[env]` table, then apply `RIG_VAR_<NAME>`
    /// overrides (name uppercased).
    pub fn seed(env: &BTreeMap<String, String>) -> Self {
        let mut store = Self::new();
        for (k, v) in env {
            store.set(k, Value::String(v.clone()));
        }
        for (k, v) in std::env::vars() {
            if let Some(name) = k.strip_prefix("RIG_VAR_") {
                store.set(name.to_lowercase(), Value::String(v));
            }
        }
        store
    }

    pub fn set(&mut self, name: impl Into<String>, value: Value) {
        self.vars.insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }

    /// A var as f64 for assertions (numbers directly; numeric strings parsed).
    pub fn get_f64(&self, name: &str) -> Option<f64> {
        match self.vars.get(name)? {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => s.parse().ok(),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Replace every `{{name}}` in `template`. Unknown vars are an error —
    /// a silently-empty substitution would corrupt URLs/bodies.
    pub fn interpolate(&self, template: &str) -> Result<String, String> {
        let mut out = String::with_capacity(template.len());
        let mut rest = template;
        while let Some(start) = rest.find("{{") {
            out.push_str(&rest[..start]);
            let after = &rest[start + 2..];
            let Some(end) = after.find("}}") else {
                return Err(format!("unclosed `{{{{` in template: `{template}`"));
            };
            let name = after[..end].trim();
            let Some(value) = self.vars.get(name) else {
                return Err(format!("unknown var `{name}` in template: `{template}`"));
            };
            match value {
                Value::String(s) => out.push_str(s),
                Value::Number(n) => out.push_str(&n.to_string()),
                Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
                other => out.push_str(&other.to_string()),
            }
            rest = &after[end + 2..];
        }
        out.push_str(rest);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> VarStore {
        let mut s = VarStore::new();
        s.set("upstream", Value::String("127.0.0.1:7373".into()));
        s.set("p99", serde_json::json!(12.5));
        s
    }

    #[test]
    fn interpolates_strings_and_numbers() {
        let s = store();
        assert_eq!(
            s.interpolate("http://{{upstream}}/x?p={{p99}}").unwrap(),
            "http://127.0.0.1:7373/x?p=12.5"
        );
    }

    #[test]
    fn unknown_var_is_error() {
        let s = store();
        let e = s.interpolate("{{missing}}").unwrap_err();
        assert!(e.contains("unknown var `missing`"));
    }

    #[test]
    fn unclosed_brace_is_error() {
        let s = store();
        assert!(s.interpolate("{{upstream").is_err());
    }

    #[test]
    fn no_vars_passthrough() {
        let s = store();
        assert_eq!(s.interpolate("plain").unwrap(), "plain");
    }

    #[test]
    fn get_f64_coerces() {
        let mut s = store();
        s.set("n", Value::String("42".into()));
        assert_eq!(s.get_f64("n"), Some(42.0));
        assert_eq!(s.get_f64("p99"), Some(12.5));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/scenario/interp.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/scenario/interp.rs` captured during rig
      standardization onto the codegen ladder.
```
