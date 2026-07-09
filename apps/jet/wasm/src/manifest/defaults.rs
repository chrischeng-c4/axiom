// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-manifest.md#schema
// CODEGEN-BEGIN
//! V0 default binding set — pre-bundled `ModuleEntry` values that
//! every project inherits without writing its own `jet.declare.d.ts`.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/binding-manifest.md#schema
//!
//! Per the spec's `DefaultBindings.description`: the JSON Schema
//! `default:` block in `binding-manifest.md` is documentation of the
//! shape, not a runtime constraint. The actual default values live
//! HERE as a Rust constant, unconditionally seeded by `parse_manifest`.
//! Comprehensive WinterCG-aligned default set is a named follow-up
//! issue per R11 of the original enhancement.

use std::sync::LazyLock;

use super::parser::{ExportEntry, ExportKind, JetImpl, ModuleEntry};

/// Helper to build a named export entry with an optional signature.
fn named(name: &str, signature: Option<&str>) -> ExportEntry {
    ExportEntry {
        kind: ExportKind::Named,
        name: name.to_string(),
        signature: signature.map(str::to_string),
    }
}

fn default_export(signature: &str) -> ExportEntry {
    ExportEntry {
        kind: ExportKind::Default,
        name: "default".to_string(),
        signature: Some(signature.to_string()),
    }
}

/// V0 starter set: `fetch`, `console`, `localStorage`, `JSON`.
pub static DEFAULT_BINDINGS: LazyLock<Vec<ModuleEntry>> = LazyLock::new(|| {
    vec![
        ModuleEntry {
            module_name: "fetch".to_string(),
            exports: vec![default_export(
                "(input: RequestInfo, init?: RequestInit): Promise<Response>",
            )],
            jet_impl: JetImpl::Bridge {
                symbol: "jet_bridge_fetch".to_string(),
            },
        },
        ModuleEntry {
            module_name: "console".to_string(),
            exports: vec![
                named("log", Some("(...args: unknown[]): void")),
                named("warn", Some("(...args: unknown[]): void")),
                named("error", Some("(...args: unknown[]): void")),
                named("info", Some("(...args: unknown[]): void")),
            ],
            jet_impl: JetImpl::Bridge {
                symbol: "jet_bridge_console".to_string(),
            },
        },
        ModuleEntry {
            module_name: "localStorage".to_string(),
            exports: vec![
                named("getItem", Some("(key: string): string | null")),
                named("setItem", Some("(key: string, value: string): void")),
                named("removeItem", Some("(key: string): void")),
                named("clear", Some("(): void")),
            ],
            jet_impl: JetImpl::Bridge {
                symbol: "jet_bridge_local_storage".to_string(),
            },
        },
        ModuleEntry {
            module_name: "JSON".to_string(),
            exports: vec![
                named("parse", Some("(text: string): unknown")),
                named("stringify", Some("(value: unknown): string")),
            ],
            jet_impl: JetImpl::Rust {
                symbol: "jet_wasm_json".to_string(),
            },
        },
    ]
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ships_four_modules() {
        assert_eq!(DEFAULT_BINDINGS.len(), 4);
        let names: Vec<&str> = DEFAULT_BINDINGS
            .iter()
            .map(|m| m.module_name.as_str())
            .collect();
        assert_eq!(names, vec!["fetch", "console", "localStorage", "JSON"]);
    }

    #[test]
    fn json_uses_rust_impl() {
        let json = DEFAULT_BINDINGS
            .iter()
            .find(|m| m.module_name == "JSON")
            .unwrap();
        assert!(matches!(&json.jet_impl, JetImpl::Rust { symbol } if symbol == "jet_wasm_json"));
    }

    #[test]
    fn fetch_uses_bridge() {
        let f = DEFAULT_BINDINGS
            .iter()
            .find(|m| m.module_name == "fetch")
            .unwrap();
        assert!(matches!(&f.jet_impl, JetImpl::Bridge { symbol } if symbol == "jet_bridge_fetch"));
    }
}
// CODEGEN-END
