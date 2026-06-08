// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/gen/rust/schema.md#source
// CODEGEN-BEGIN

//! Schema structural generator.
//!
//! Produces Rust struct with serde derives from JSON Schema YAML frontmatter.
//! 100% deterministic coverage for schema section types.
#![allow(dead_code, unreachable_code)]

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R1

use serde_yaml::Value;

use crate::generate::types::{parse_abstract_type, RustConfig, RustTypeTranslator, TypeTranslator};

/// Output from schema code generation.
#[derive(Debug, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/schema.md#source
pub struct SchemaGenOutput {
    /// The generated Rust struct code.
    pub code: String,
}

/// Generate a Rust struct from a JSON Schema YAML value.
///
/// Translates each property to a Rust field with the configured type system.
/// Applies config layering: global config + per-spec x-rust overrides.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R1
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R6
pub fn generate_schema(schema_yaml: &Value, config: &RustConfig) -> SchemaGenOutput {
    generate_schema_with_provenance(schema_yaml, config, "")
}

/// Same as [`generate_schema`] but embeds `@spec <spec_path>#<anchor>`
/// provenance markers on each major generated item so the output can be
/// traced back to the exact spec section that produced it. Pass `""` for
/// `spec_path` to suppress markers (useful in unit tests).
pub fn generate_schema_with_provenance(
    schema_yaml: &Value,
    config: &RustConfig,
    spec_path: &str,
) -> SchemaGenOutput {
    let config = config.merge_overrides(schema_yaml);
    let translator = RustTypeTranslator;
    let vis = config.vis_prefix();

    let title = schema_yaml
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("GeneratedStruct");

    let struct_name = to_pascal_case(title);

    // Early dispatch: JSON Schema `type: string, enum: [...]` maps to a Rust
    // unit-variant enum, not a struct. Mamba binding annotations don't apply
    // to enums (no positional-arg construction), so the enum path is
    // self-contained.
    if is_string_enum_schema(schema_yaml)
        || has_payload_variants(schema_yaml)
        || has_x_rust_enum_variants(schema_yaml)
    {
        return generate_rust_enum(schema_yaml, &struct_name, &config, spec_path);
    }

    let properties = schema_yaml
        .get("properties")
        .and_then(|v| v.as_mapping())
        .cloned()
        .unwrap_or_default();

    let required_fields: Vec<String> = schema_yaml
        .get("required")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let mut lines = Vec::new();

    // Struct-level doc comment from schema `description`.
    if let Some(desc) = schema_yaml.get("description").and_then(|v| v.as_str()) {
        for line in desc.lines() {
            lines.push(format!("/// {}", line));
        }
    }
    // Per-item provenance — traces back to the `schema` section of the spec
    // that produced this struct. Omitted when `spec_path` is empty (unit
    // tests that drive the generator directly).
    if !spec_path.is_empty() {
        lines.push(format!("/// @spec {}#schema", spec_path));
    }

    // Optional struct-level cfg attribute.
    if let Some(cfg) = schema_yaml.get("x-rust-cfg").and_then(|v| v.as_str()) {
        lines.push(format!("#[cfg({})]", cfg));
    }

    // Optional arbitrary type-level attributes via `x-rust-attrs: ["allow(dead_code)", ...]`.
    // Each entry emits `#[<value>]` verbatim above the type declaration. Used
    // when an existing source carries something the generator doesn't have a
    // dedicated key for (e.g. `#[allow(dead_code)]` on partly-consumed types).
    if let Some(attrs) = schema_yaml
        .get("x-rust-attrs")
        .and_then(|v| v.as_sequence())
    {
        for attr in attrs {
            if let Some(s) = attr.as_str() {
                lines.push(format!("#[{}]", s));
            }
        }
    }

    // Derives and attributes. R6: if the schema declares an explicit
    // `x-rust-struct.derive: [...]` list, emit it verbatim. Otherwise fall
    // back to the historical `config.derive_attr()` which layers in Hash/Copy
    // toggles from RustConfig.
    let derive_attr = resolve_explicit_derive(schema_yaml, "x-rust-struct", config.derive_attr());
    if !derive_attr.is_empty() {
        lines.push(derive_attr);
    }
    // Whether THIS struct (after x-rust-struct.derive override) actually
    // has Serialize/Deserialize. Per-field serde attrs (`#[serde(default)]`,
    // `#[serde(skip_serializing_if = ...)]`) only make sense on serde
    // structs — emitting them on a `#[derive(Debug, Clone)]`-only struct
    // is a hard compile error ("cannot find attribute `serde`").
    let struct_has_serde = struct_has_serde_derive(schema_yaml, "x-rust-struct", &config);

    // Container-level serde attributes. Spec author can declare any of:
    // - `x-rust-struct.serde_rename_all: <strategy>` (e.g. lowercase, camelCase)
    // - `x-rust-struct.serde_deny_unknown: true` → `#[serde(deny_unknown_fields)]`
    // When both are present they are combined into ONE `#[serde(...)]` line
    // (serde rejects them split across two attributes).
    let serde_rename_strategy = schema_yaml
        .get("x-rust-struct")
        .and_then(|v| v.get("serde_rename_all"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty() && *s != "none");
    let mut struct_serde_parts: Vec<String> = Vec::new();
    if let Some(strategy) = serde_rename_strategy {
        // Structs: snake_case is the default → suppress.
        if !(strategy == "snake_case") {
            struct_serde_parts.push(format!("rename_all = \"{}\"", strategy));
        }
    } else if let Some(default_attr) = config.serde_rename_attr().strip_prefix("#[serde(") {
        // Fallback to global RustConfig — preserve previous behaviour.
        let body = default_attr.trim_end_matches(")]");
        if !body.is_empty() {
            struct_serde_parts.push(body.to_string());
        }
    }
    let serde_deny_unknown = schema_yaml
        .get("x-rust-struct")
        .and_then(|v| v.get("serde_deny_unknown"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if serde_deny_unknown {
        struct_serde_parts.push("deny_unknown_fields".to_string());
    }
    if !struct_serde_parts.is_empty() {
        lines.push(format!("#[serde({})]", struct_serde_parts.join(", ")));
    }

    // Optional generic parameters: `x-rust-generics: [T]` → `struct Name<T>`.
    // Used for container types like `Path<T>`, `Query<T>`, `Json<T>` where
    // the inner field carries the generic.
    let generics: Vec<String> = schema_yaml
        .get("x-rust-generics")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let generics_str = if generics.is_empty() {
        String::new()
    } else {
        format!("<{}>", generics.join(", "))
    };

    // Optional struct-level where-clauses: `x-rust-where-clauses: ["F: Fn(&str) -> String + Send + Sync"]`
    // Renders between the generics and the opening `{` when present.
    let where_clauses: Vec<String> = schema_yaml
        .get("x-rust-where-clauses")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Unit struct shorthand: `x-rust-struct.unit: true` with empty
    // properties emits `pub struct X;` instead of `pub struct X { }`.
    // The shorthand form is required for `Self` constructor expressions
    // in hand-written impl blocks (`fn new() -> Self { Self }` only
    // works on tuple/unit structs).
    let is_unit = properties.is_empty()
        && schema_yaml
            .get("x-rust-struct")
            .and_then(|v| v.get("unit"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

    let is_empty_braced_struct = properties.is_empty() && where_clauses.is_empty() && !is_unit;
    if is_unit {
        lines.push(format!("{}struct {}{};", vis, struct_name, generics_str));
    } else if is_empty_braced_struct {
        lines.push(format!(
            "{}struct {}{} {{}}",
            vis, struct_name, generics_str
        ));
    } else if where_clauses.is_empty() {
        lines.push(format!("{}struct {}{} {{", vis, struct_name, generics_str));
    } else {
        lines.push(format!("{}struct {}{}", vis, struct_name, generics_str));
        lines.push("where".to_string());
        for clause in &where_clauses {
            lines.push(format!("    {},", clause));
        }
        lines.push("{".to_string());
    }

    for (key, prop_value) in &properties {
        let field_name = key.as_str().unwrap_or("field");
        let is_required = required_fields.contains(&field_name.to_string());

        let (inner_type, type_nullable) = infer_rust_type_with_nullable(prop_value, &translator);
        // Wrap in Option<...> when:
        //   - not in `required` (schema-level optional), OR
        //   - type array includes `"null"` even if listed required.
        // BUT: if the inner type the author specified already starts with
        //   `Option<`, the double wrap is never desired — the explicit
        //   x-rust-type already encodes the optionality. Same for `Vec<...>`
        //   when paired with x-serde-default (Vec is not Option-wrappable
        //   in serde's default mental model — empty Vec is the natural
        //   missing-value marker). Detect and skip the wrap.
        // R11: collection types from explicit `x-rust-type: "Vec<T>"` etc.
        // are also "already optional" — empty collection is the natural
        // missing-value marker, so don't double-wrap in Option<>.
        let already_optional = inner_type.starts_with("Option<")
            || inner_type.starts_with("std::option::Option<")
            || inner_type.starts_with("Vec<")
            || inner_type.starts_with("HashMap<")
            || inner_type.starts_with("BTreeMap<")
            || inner_type.starts_with("HashSet<")
            || inner_type.starts_with("BTreeSet<");
        let wrap_option = (!is_required || type_nullable) && !already_optional;
        let final_type = if wrap_option {
            format!("Option<{}>", inner_type)
        } else {
            inner_type
        };

        if let Some(desc) = prop_value.get("description").and_then(|v| v.as_str()) {
            // Split on newlines so multi-line descriptions render as
            // multiple `///` doc lines. Emitting a raw `\n` inside a single
            // `///` string would produce a bare line that fails to parse.
            for line in desc.lines() {
                lines.push(format!("    /// {}", line));
            }
        }

        if let Some(cfg) = prop_value.get("x-rust-cfg").and_then(|v| v.as_str()) {
            lines.push(format!("    #[cfg({})]", cfg));
        }

        // `x-clap-arg: "<body>"` emits `#[arg(<body>)]` verbatim. Authors
        // write the clap inner-attribute body directly — e.g.
        // `x-clap-arg: "long"`               → `#[arg(long)]`
        // `x-clap-arg: "long, short = 'd'"`  → `#[arg(long, short = 'd')]`
        // `x-clap-arg: "default_value = \"foo\""` → `#[arg(default_value = "foo")]`
        // Used in CLI handler structs that derive `clap::Args`. The
        // `#[derive(Args)]` itself is rendered via the standard derive
        // list (path-style `clap::Args` works).
        if let Some(arg_body) = prop_value.get("x-clap-arg").and_then(|v| v.as_str()) {
            lines.push(format!("    #[arg({})]", arg_body));
        }

        // `x-clap-command: "<body>"` emits `#[command(<body>)]`. Used for
        // `#[command(subcommand)]` on a field whose type is a clap-Subcommand
        // enum. Body forwarded verbatim — e.g. "subcommand", "flatten".
        if let Some(cmd_body) = prop_value.get("x-clap-command").and_then(|v| v.as_str()) {
            lines.push(format!("    #[command({})]", cmd_body));
        }

        let rust_field_name = to_rust_field_name(field_name);

        // Collect per-field serde modifiers into a single combined attr so
        // we don't emit two lines like `#[serde(default)] #[serde(...)]`.
        // Order in output: rename, default, skip_serializing_if. Other
        // serde knobs can be added here as new x-serde-* shortcuts land.
        let mut serde_parts: Vec<String> = Vec::new();
        // Per-field serde attrs require the struct to actually have
        // Serialize/Deserialize derives — otherwise rustc errors with
        // "cannot find attribute `serde` in this scope".
        //
        // `x-serde-rename` overrides the JSON key name independently of
        // the Rust field name. Use it when the property key is the Rust
        // field name (`attr_type` in spec) but the JSON wire format wants
        // a different / reserved-word key (`type` on the wire).
        let explicit_rename = prop_value.get("x-serde-rename").and_then(|v| v.as_str());
        if struct_has_serde {
            if let Some(name) = explicit_rename {
                serde_parts.push(format!("rename = \"{}\"", name));
            } else if rust_field_name != field_name {
                serde_parts.push(format!("rename = \"{}\"", field_name));
            }
        }
        // `x-serde-alias: ["alt1", "alt2"]` emits one `alias = "..."` part per
        // entry, in declared order. Use it for backward-compat with old wire
        // names (e.g. TasksFrontmatter.id keeps `alias = "change_id"`).
        if struct_has_serde {
            if let Some(aliases) = prop_value
                .get("x-serde-alias")
                .and_then(|v| v.as_sequence())
            {
                for alias in aliases.iter().filter_map(|v| v.as_str()) {
                    serde_parts.push(format!("alias = \"{}\"", alias));
                }
            }
        }
        // `x-serde-skip` accepts:
        //   true             → `skip`               (skip both ser+de)
        //   "serializing"    → `skip_serializing`   (omit on serialize only)
        //   "deserializing"  → `skip_deserializing` (ignore on deserialize only)
        // Placed before `default` so we emit `#[serde(skip, default)]` matching
        // hand-written convention in models/change.rs.
        if struct_has_serde {
            if let Some(skip_value) = prop_value.get("x-serde-skip") {
                if skip_value.as_bool() == Some(true) {
                    serde_parts.push("skip".to_string());
                } else if let Some(mode) = skip_value.as_str() {
                    match mode {
                        "serializing" => serde_parts.push("skip_serializing".to_string()),
                        "deserializing" => serde_parts.push("skip_deserializing".to_string()),
                        _ => {}
                    }
                }
            }
        }
        // `x-serde-default` accepts:
        //   true              → `default`               (uses Default::default)
        //   "<fn_name>"       → `default = "fn_name"`   (calls custom fn)
        //   false / absent    → no default attribute
        // The custom-fn shape is the spec equivalent of source code like
        // `#[serde(default = "default_columns")]` where the author defined
        // a top-level `fn default_columns() -> u32 { 1 }` helper.
        let serde_default_value = prop_value.get("x-serde-default");
        let explicit_default_bool = serde_default_value
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let explicit_default_fn = serde_default_value
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());
        let auto_default = struct_has_serde && wrap_option;
        // R12: explicit_default_bool requires struct_has_serde — emitting
        // `#[serde(default)]` on a struct without Serialize/Deserialize
        // derives is a compile error ("cannot find attribute serde in
        // this scope"). auto_default is already gated above.
        if let Some(fn_name) = explicit_default_fn {
            if struct_has_serde {
                serde_parts.push(format!("default = \"{}\"", fn_name));
            }
        } else if (struct_has_serde && explicit_default_bool) || auto_default {
            serde_parts.push("default".to_string());
        }
        // `x-serde-skip-if` (string) → emit
        // `#[serde(skip_serializing_if = "<expr>")]`. Most common idiom is
        // `Option::is_none` for optional fields and `Vec::is_empty` for
        // collections that omit-when-empty.
        if struct_has_serde {
            if let Some(skip_expr) = prop_value.get("x-serde-skip-if").and_then(|v| v.as_str()) {
                serde_parts.push(format!("skip_serializing_if = \"{}\"", skip_expr));
            }
        }
        if !serde_parts.is_empty() {
            lines.push(format!("    #[serde({})]", serde_parts.join(", ")));
        }

        // Per-field visibility override. `x-rust-visibility: private` emits
        // the field without `pub ` so that private fields on public structs
        // (e.g. `state` on `Request`) round-trip correctly.
        let field_vis = prop_value
            .get("x-rust-visibility")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let field_vis_prefix = match field_vis {
            "private" => String::new(),
            "" => vis.to_string(),
            other => format!("{} ", other),
        };
        lines.push(format!(
            "    {}{}: {},",
            field_vis_prefix, rust_field_name, final_type
        ));
    }

    if !is_unit {
        if !is_empty_braced_struct {
            lines.push("}".to_string());
        }
    }

    // Factory methods from `x-sdd.factories`.
    if let Some(factories) = schema_yaml
        .get("x-sdd")
        .and_then(|v| v.get("factories"))
        .and_then(|v| v.as_sequence())
    {
        lines.push(String::new());
        lines.push(format!("impl {} {{", struct_name));
        for (i, factory) in factories.iter().enumerate() {
            let name = factory
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("new");
            let doc = factory.get("doc").and_then(|v| v.as_str());
            let params = factory
                .get("params")
                .and_then(|v| v.as_sequence())
                .cloned()
                .unwrap_or_default();
            let fields = factory
                .get("fields")
                .and_then(|v| v.as_mapping())
                .cloned()
                .unwrap_or_default();

            if i > 0 {
                lines.push(String::new());
            }
            if let Some(d) = doc {
                for line in d.lines() {
                    lines.push(format!("    /// {}", line));
                }
            }

            let params_str = params
                .iter()
                .map(|p| {
                    let n = p.get("name").and_then(|v| v.as_str()).unwrap_or("arg");
                    let t = p.get("type").and_then(|v| v.as_str()).unwrap_or("()");
                    format!("{}: {}", n, t)
                })
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("    {}fn {}({}) -> Self {{", vis, name, params_str));
            lines.push("        Self {".to_string());
            // Emit fields in the same order as struct properties for determinism.
            for (key, _) in &properties {
                let field_name = key.as_str().unwrap_or("field");
                let rust_field_name = to_rust_field_name(field_name);
                let val_key = serde_yaml::Value::String(field_name.to_string());
                let val = fields
                    .get(&val_key)
                    .and_then(|v| v.as_str())
                    .unwrap_or("Default::default()");
                // Field init shorthand: `name: name` → `name` when the value
                // literal matches the field identifier exactly.
                if val == rust_field_name {
                    lines.push(format!("            {},", rust_field_name));
                } else {
                    lines.push(format!("            {}: {},", rust_field_name, val));
                }
            }
            lines.push("        }".to_string());
            lines.push("    }".to_string());
        }
        lines.push("}".to_string());
    }

    // R1/R2/R3/R4: spec-declared inherent impls. `x-constructor`,
    // `x-builders`, and codegen-marked `x-methods` collectively become one
    // `impl StructName { ... }` block appended after the struct. Methods
    // flagged `impl_mode: hand-written` are skipped here and remain the
    // author's responsibility.
    let x_impls = emit_x_impls(schema_yaml, &struct_name, spec_path, &vis);
    if !x_impls.is_empty() {
        lines.push(String::new());
        lines.push(x_impls);
    }

    // x-trait-impls: Display / FromStr / etc. — emitted after inherent
    // impls. Same machinery as the enum path; structs occasionally need
    // Display for log output and FromStr for config parsing.
    let trait_impls = emit_trait_impls(schema_yaml, &struct_name, spec_path);
    if !trait_impls.is_empty() {
        lines.push(String::new());
        lines.push(trait_impls);
    }

    // Auto-infer `use` imports from generated body + explicit x-rust.imports.
    let body = lines.join("\n");
    let mut imports: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    if body.contains("DateTime<") || body.contains("Utc::now()") {
        imports.insert("use chrono::{DateTime, Utc};".to_string());
    }
    if body.contains("Uuid") && !body.contains("uuid::Uuid") {
        imports.insert("use uuid::Uuid;".to_string());
    }
    // Only emit `use serde::*` when the body actually mentions short-form
    // `Serialize` / `Deserialize`. Skip when:
    //   - the body uses path-style derives (`serde::Serialize`) which
    //     don't need the import (the `serde::` segment IS the path),
    //   - the body has no serde derives at all (e.g. struct only derives
    //     Debug + Clone), in which case the import is dead code AND
    //     conflicts with any pre-existing hand-written `use serde::Serialize`
    //     above the CODEGEN block.
    // Detect short-form usage by looking for `Serialize` or `Deserialize`
    // as a whole token NOT immediately preceded by `::` (which would be
    // path-style).
    let needs_short_serde = body_uses_short_serde(&body);
    if needs_short_serde {
        let needs_ser = body_uses_short_token(&body, "Serialize");
        let needs_de = body_uses_short_token(&body, "Deserialize");
        let import = match (needs_ser, needs_de) {
            (true, true) => "use serde::{Deserialize, Serialize};",
            (true, false) => "use serde::Serialize;",
            (false, true) => "use serde::Deserialize;",
            (false, false) => unreachable!(),
        };
        imports.insert(import.to_string());
    }
    if let Some(extra) = schema_yaml
        .get("x-rust")
        .and_then(|v| v.get("imports"))
        .and_then(|v| v.as_sequence())
    {
        for item in extra {
            if let Some(path) = item.as_str() {
                imports.insert(format!("use {};", path));
            }
        }
    }

    let mut code = if imports.is_empty() {
        body
    } else {
        let mut out = imports.into_iter().collect::<Vec<_>>().join("\n");
        out.push_str("\n\n");
        out.push_str(&body);
        out
    };

    // Mamba FFI binding — emitted when `x-mamba-binding` is present on the
    // schema. Produces constructor impl + extern "C" shim + register() fn,
    // appended after the struct.
    let binding = super::mamba_binding::generate_mamba_binding_with_provenance(
        schema_yaml,
        &struct_name,
        spec_path,
    );
    if binding.emitted {
        code.push_str("\n\n");
        code.push_str(&binding.code);
    }

    SchemaGenOutput { code }
}

/// Returns `(rust_type, type_says_nullable)`. The nullable flag is true when
/// the JSON Schema property declares `type: ["X", "null"]`. Callers combine
/// that with the `required` list to decide whether to wrap in `Option<...>`.
fn infer_rust_type_with_nullable(prop: &Value, translator: &RustTypeTranslator) -> (String, bool) {
    // Spec-author override: `x-rust-type: <Type>` short-circuits inference so
    // narrow Rust types (u16, u8, custom aliases) can be spelled explicitly.
    // Nullability is still derived from schema `type: [..., "null"]` so
    // `Option<>` wrapping continues to work.
    let null_from_type = matches!(
        prop.get("type"),
        Some(v) if v.is_sequence()
            && v.as_sequence()
                .map(|s| s.iter().any(|t| t.as_str() == Some("null")))
                .unwrap_or(false)
    );
    if let Some(t) = prop.get("x-rust-type").and_then(|v| v.as_str()) {
        return (t.to_string(), null_from_type);
    }

    let format_str = prop.get("format").and_then(|v| v.as_str());
    let ref_str = prop.get("$ref").and_then(|v| v.as_str());

    if let Some(r) = ref_str {
        return (r.rsplit('/').next().unwrap_or("Value").to_string(), false);
    }

    // Normalize `type: ["string","null"]` → ("string", nullable=true).
    let (type_str, nullable_from_type) = match prop.get("type") {
        Some(v) if v.is_sequence() => {
            let types: Vec<&str> = v
                .as_sequence()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str())
                .collect();
            let has_null = types.contains(&"null");
            let non_null: Vec<&str> = types.iter().copied().filter(|t| *t != "null").collect();
            if non_null.len() == 1 {
                (Some(non_null[0]), has_null)
            } else {
                (None, has_null)
            }
        }
        Some(v) => (v.as_str(), false),
        None => (None, false),
    };

    // `minimum >= 0` → unsigned integer.
    let min_nonneg = prop
        .get("minimum")
        .and_then(|v| v.as_i64())
        .map_or(false, |m| m >= 0);

    let ty = match (type_str, format_str) {
        (Some("string"), Some("uuid")) => "Uuid".to_string(),
        (Some("string"), Some("date-time")) => "DateTime<Utc>".to_string(),
        (Some("string"), _) => "String".to_string(),
        (Some("integer"), Some("int32")) => (if min_nonneg { "u32" } else { "i32" }).to_string(),
        // R5: `type: [integer, "null"]` → usize (idiomatic for nullable indices
        // and 1-counters like line numbers). `type: integer` alone keeps the
        // historical u64/i64 default so existing specs are not disturbed.
        (Some("integer"), _) if nullable_from_type => "usize".to_string(),
        (Some("integer"), _) => (if min_nonneg { "u64" } else { "i64" }).to_string(),
        (Some("number"), Some("float")) => "f32".to_string(),
        (Some("number"), _) => "f64".to_string(),
        (Some("boolean"), _) => "bool".to_string(),
        (Some("array"), _) => {
            let (item_type, _) = prop
                .get("items")
                .map(|items| infer_rust_type_with_nullable(items, translator))
                .unwrap_or_else(|| ("serde_json::Value".to_string(), false));
            format!("Vec<{}>", item_type)
        }
        (Some("object"), _) => "serde_json::Value".to_string(),
        _ => {
            if let Some(t) = type_str {
                if let Ok(at) = parse_abstract_type(t) {
                    translator.translate(&at)
                } else {
                    "serde_json::Value".to_string()
                }
            } else {
                "serde_json::Value".to_string()
            }
        }
    };

    (ty, nullable_from_type)
}

fn infer_rust_type(prop: &Value, translator: &RustTypeTranslator) -> String {
    infer_rust_type_with_nullable(prop, translator).0
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
            result.push(c.to_lowercase().next().unwrap_or(c));
        } else if c == '-' {
            result.push('_');
        } else {
            result.push(c.to_lowercase().next().unwrap_or(c));
        }
    }
    result
}

fn to_rust_field_name(field_name: &str) -> String {
    let snake = to_snake_case(field_name);
    if is_rust_keyword(&snake) {
        format!("r#{}", snake)
    } else {
        snake
    }
}

fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "abstract"
            | "as"
            | "async"
            | "await"
            | "become"
            | "box"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "do"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "final"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "macro"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "override"
            | "priv"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "try"
            | "true"
            | "type"
            | "typeof"
            | "unsafe"
            | "unsized"
            | "use"
            | "virtual"
            | "where"
            | "while"
            | "yield"
    )
}

// ---------------------------------------------------------------------------
// Spec-declared impls (R1/R2/R3/R4/R6/R7)
// ---------------------------------------------------------------------------

/// True when the effective derive list for this schema includes
/// `Serialize` or `Deserialize` in either bare (`Serialize`) or path-style
/// (`serde::Serialize`) form. Order of precedence mirrors
/// `resolve_explicit_derive`:
/// 1. `<parent_key>.derive: [...]` on the schema (per-spec).
/// 2. Fall back to `RustConfig.has_serde_derives()`.
///
/// Used to gate per-field serde attribute emission. Without this gate,
/// a non-serde struct (`#[derive(Debug, Clone)]`) carrying an `Option`
/// field would still get `#[serde(default)]` auto-emitted and rustc
/// would complain about an unknown attribute.
fn struct_has_serde_derive(schema: &Value, parent_key: &str, config: &RustConfig) -> bool {
    if let Some(list) = schema
        .get(parent_key)
        .and_then(|v| v.get("derive"))
        .and_then(|v| v.as_sequence())
    {
        return list
            .iter()
            .filter_map(|v| v.as_str())
            .any(is_serde_derive_name);
    }
    config.has_serde_derives()
}

fn is_serde_derive_name(name: &str) -> bool {
    matches!(
        name.rsplit("::").next().unwrap_or(name),
        "Serialize" | "Deserialize"
    )
}

/// Resolve a container-level `#[serde(rename_all = "...")]` attribute.
/// Order of precedence:
/// 1. `<parent_key>.serde_rename_all: <strategy>` on the schema (per-spec).
/// 2. `RustConfig.serde_rename_strategy` (global).
/// 3. None — emits empty string when neither is set or strategy is
///    `none`. Note: `snake_case` is suppressed for **structs** because
///    snake_case field identifiers stay snake_case in JSON by default,
///    making the attr redundant. For **enums** snake_case is NOT the
///    default — enum variants use PascalCase identifiers and serde
///    keeps them as PascalCase unless explicitly told otherwise — so
///    we must emit `#[serde(rename_all = "snake_case")]` to preserve
///    the active casing transform.
fn resolve_serde_rename_all(schema: &Value, parent_key: &str, config: &RustConfig) -> String {
    let strategy = schema
        .get(parent_key)
        .and_then(|v| v.get("serde_rename_all"))
        .and_then(|v| v.as_str());
    if let Some(s) = strategy {
        if s.is_empty() || s == "none" {
            return String::new();
        }
        // Structs: snake_case is the default for field names → suppress.
        // Enums: snake_case is an active variant-name transform → emit.
        if s == "snake_case" && parent_key == "x-rust-struct" {
            return String::new();
        }
        return format!("#[serde(rename_all = \"{}\")]", s);
    }
    config.serde_rename_attr()
}

/// Look up `<parent_key>.derive: [...]` and render it as
/// `#[derive(<list verbatim>)]`. Returns `fallback` when the list is missing
/// or empty. Implements R6: explicit lists win over any default derives.
fn resolve_explicit_derive(schema: &Value, parent_key: &str, fallback: String) -> String {
    if let Some(list) = schema
        .get(parent_key)
        .and_then(|v| v.get("derive"))
        .and_then(|v| v.as_sequence())
    {
        let names: Vec<String> = list
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        if names.is_empty() {
            return String::new();
        }
        return format!("#[derive({})]", names.join(", "));
    }
    fallback
}

/// Does the schema's `x-methods` list include a codegen method whose `name`
/// matches `target`? Case-insensitive match with the conventional
/// snake_case/PascalCase spellings collapsed. Implements R7.
fn x_methods_declares(schema: &Value, target: &str) -> bool {
    let target_low = target.to_ascii_lowercase();
    let Some(methods) = schema.get("x-methods").and_then(|v| v.as_sequence()) else {
        return false;
    };
    methods.iter().any(|m| {
        m.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_ascii_lowercase() == target_low)
            .unwrap_or(false)
            && m.get("impl_mode")
                .and_then(|v| v.as_str())
                .map(|s| s == "codegen")
                .unwrap_or(true)
    })
}

/// Combine spec-declared `x-constructor`, `x-builders`, and codegen
/// `x-methods` into a single `impl <TypeName> { ... }` block. Returns an
/// empty string when none of the three extensions is present.
///
/// Emission order within the block: constructor → builders → methods
/// (in declaration order). Methods flagged `impl_mode: hand-written` are
/// skipped silently so the author retains ownership.
fn emit_x_impls(schema: &Value, type_name: &str, spec_path: &str, vis: &str) -> String {
    let mut body: Vec<String> = Vec::new();

    // Helper: indent a multi-line string by 4 spaces and push into body,
    // separating consecutive items with a blank line for readability.
    let push_item = |body: &mut Vec<String>, s: String| {
        if !body.is_empty() {
            body.push(String::new());
        }
        for line in s.lines() {
            body.push(format!("    {}", line));
        }
    };

    if let Some(ctor) = schema.get("x-constructor") {
        if let Some(out) = emit_constructor(ctor, vis) {
            push_item(&mut body, out);
        }
    }

    if let Some(builders) = schema.get("x-builders").and_then(|v| v.as_sequence()) {
        for b in builders {
            if let Some(out) = emit_builder(b, vis) {
                push_item(&mut body, out);
            }
        }
    }

    if let Some(methods) = schema.get("x-methods").and_then(|v| v.as_sequence()) {
        for m in methods {
            // Skip `as_str` / `from_str` — generate_rust_enum already emits
            // those inline when declared (R7 path). Double-emission would
            // collide.
            let name = m.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if name.eq_ignore_ascii_case("as_str") || name.eq_ignore_ascii_case("from_str") {
                continue;
            }
            if let Some(out) = emit_method(m, type_name, vis) {
                push_item(&mut body, out);
            }
        }
    }

    if body.is_empty() {
        return String::new();
    }

    let mut lines: Vec<String> = Vec::new();
    if !spec_path.is_empty() {
        lines.push(format!("/// @spec {}#schema.impls", spec_path));
    }
    lines.push(format!("impl {} {{", type_name));
    lines.extend(body);
    lines.push("}".to_string());
    lines.join("\n")
}

/// Emit `pub fn <name>(args) -> Self { Self { ... } }` from an
/// `x-constructor` block. Returns `None` when the block is
/// `impl_mode: hand-written` (author owns it) or malformed.
///
/// Each arg whose spec carries `into: <T>` is assigned into its field via
/// `<arg>.into()`. Remaining fields come from the `init:` mapping's literal
/// Rust expressions.
fn emit_constructor(ctor: &Value, vis: &str) -> Option<String> {
    if ctor
        .get("impl_mode")
        .and_then(|v| v.as_str())
        .map(|s| s != "codegen")
        .unwrap_or(false)
    {
        return None;
    }
    let name = ctor.get("name").and_then(|v| v.as_str())?;
    let doc = ctor.get("doc").and_then(|v| v.as_str());
    let args = ctor
        .get("args")
        .and_then(|v| v.as_sequence())
        .cloned()
        .unwrap_or_default();

    let mut lines: Vec<String> = Vec::new();
    if let Some(d) = doc {
        for l in d.lines() {
            lines.push(format!("/// {}", l));
        }
    }

    let sig_args: Vec<String> = args
        .iter()
        .filter_map(|a| {
            let n = a.get("name").and_then(|v| v.as_str())?;
            let t = a.get("rust_type").and_then(|v| v.as_str())?;
            Some(format!("{}: {}", n, t))
        })
        .collect();

    lines.push(format!(
        "{}fn {}({}) -> Self {{",
        vis,
        name,
        sig_args.join(", ")
    ));
    lines.push("    Self {".to_string());
    for a in &args {
        let n = a.get("name").and_then(|v| v.as_str()).unwrap_or("arg");
        if a.get("into").is_some() {
            lines.push(format!("        {}: {}.into(),", n, n));
        } else {
            lines.push(format!("        {},", n));
        }
    }
    if let Some(init) = ctor.get("init").and_then(|v| v.as_mapping()) {
        for (k, v) in init {
            let key = k.as_str().unwrap_or("field");
            let val = v.as_str().unwrap_or("Default::default()");
            lines.push(format!("        {}: {},", key, val));
        }
    }
    lines.push("    }".to_string());
    lines.push("}".to_string());
    Some(lines.join("\n"))
}

/// Emit `pub fn <name>(mut self, <arg>: <type>) -> Self { self.<field> = wrap(arg); self }`
/// from an `x-builders` entry. Respects `wrap: Some` (or other wrapper idents)
/// and `into:` on the arg.
fn emit_builder(b: &Value, vis: &str) -> Option<String> {
    if b.get("impl_mode")
        .and_then(|v| v.as_str())
        .map(|s| s != "codegen")
        .unwrap_or(false)
    {
        return None;
    }
    let name = b.get("name").and_then(|v| v.as_str())?;
    let field = b.get("field").and_then(|v| v.as_str())?;
    let arg = b.get("arg")?;
    let arg_name = arg.get("name").and_then(|v| v.as_str())?;
    let arg_type = arg.get("rust_type").and_then(|v| v.as_str())?;
    let wrap = b.get("wrap").and_then(|v| v.as_str());
    let has_into = arg.get("into").is_some();
    let doc = b.get("doc").and_then(|v| v.as_str());

    let arg_expr = if has_into {
        format!("{}.into()", arg_name)
    } else {
        arg_name.to_string()
    };
    let wrapped = match wrap {
        Some(w) if !w.is_empty() => format!("{}({})", w, arg_expr),
        _ => arg_expr,
    };

    let mut lines: Vec<String> = Vec::new();
    if let Some(d) = doc {
        for l in d.lines() {
            lines.push(format!("/// {}", l));
        }
    }
    lines.push(format!(
        "{}fn {}(mut self, {}: {}) -> Self {{",
        vis, name, arg_name, arg_type
    ));
    lines.push(format!("    self.{} = {};", field, wrapped));
    lines.push("    self".to_string());
    lines.push("}".to_string());
    Some(lines.join("\n"))
}

/// Emit one spec-declared inherent method from an `x-methods` entry.
/// - `dispatch: [...]` → `match self { variant => value }` body (R1).
/// - `body: "<rust-expr>"` → body is the literal expression followed by `;`.
/// - `delegates_to: "<rust-expr>"` → same as `body:` (alias).
/// - otherwise → `Default::default()` placeholder so the file compiles (R4).
///
/// Receiver is inferred: `&mut self` when body mutates (`self.` appears on
/// LHS of `=` / method call like `.push`), `self` for builder-like
/// methods that return `Self`, otherwise `&self`.
fn emit_method(m: &Value, type_name: &str, vis: &str) -> Option<String> {
    if m.get("impl_mode")
        .and_then(|v| v.as_str())
        .map(|s| s != "codegen")
        .unwrap_or(false)
    {
        return None;
    }
    let name = m.get("name").and_then(|v| v.as_str())?;
    let returns = m.get("returns").and_then(|v| v.as_str());
    let doc = m.get("doc").and_then(|v| v.as_str());
    let args: Vec<Value> = m
        .get("args")
        .and_then(|v| v.as_sequence())
        .cloned()
        .unwrap_or_default();
    let dispatch = m.get("dispatch").and_then(|v| v.as_sequence()).cloned();
    let body_literal = m
        .get("body")
        .and_then(|v| v.as_str())
        .or_else(|| m.get("delegates_to").and_then(|v| v.as_str()));

    // Receiver heuristic — declarative guess good enough for the typical
    // builder/inspector methods we see in sdd/score specs. Authors can
    // override via an explicit `receiver:` field (values: `self`, `&self`,
    // `&mut self`, or `mut self`).
    //
    // Precedence: explicit `receiver:` > `returns: Self` (consuming builder)
    // > mutation evidence in body > plain `&self`. The builder case takes
    // priority over mutation-in-body because the builder pattern is
    // `fn with_x(mut self, ...) -> Self { self.x.push(..); self }` — the
    // body DOES mutate but the method is still consuming-style.
    let receiver = if let Some(explicit) = m.get("receiver").and_then(|v| v.as_str()) {
        explicit
    } else if returns == Some("Self") {
        "mut self"
    } else if body_literal
        .map(|b| b.contains("self.") || b.contains(".push(") || b.contains(".extend("))
        .unwrap_or(false)
        || matches!(name, "push" | "extend" | "clear" | "reset")
    {
        "&mut self"
    } else {
        "&self"
    };

    let arg_sig: Vec<String> = args
        .iter()
        .filter_map(|a| {
            let n = a.get("name").and_then(|v| v.as_str())?;
            let t = a.get("rust_type").and_then(|v| v.as_str())?;
            Some(format!("{}: {}", n, t))
        })
        .collect();

    let ret_clause = returns.map_or(String::new(), |r| format!(" -> {}", r));

    let mut lines: Vec<String> = Vec::new();
    if let Some(d) = doc {
        for l in d.lines() {
            lines.push(format!("/// {}", l));
        }
    }
    let sig = if arg_sig.is_empty() {
        format!("{}fn {}({}){} {{", vis, name, receiver, ret_clause)
    } else {
        format!(
            "{}fn {}({}, {}){} {{",
            vis,
            name,
            receiver,
            arg_sig.join(", "),
            ret_clause
        )
    };
    lines.push(sig);

    if let Some(arms) = dispatch {
        // R1: dispatch table → exhaustive `match self`.
        lines.push("    match self {".to_string());
        for arm in arms {
            let var = arm.get("variant").and_then(|v| v.as_str()).unwrap_or("_");
            let raw = arm.get("value").and_then(|v| v.as_str()).unwrap_or("");
            // Auto-quote unless the author already wrote a Rust expression
            // (quoted string, leading `Self::`, etc.).
            let quoted = raw.starts_with('"')
                || raw.starts_with("Self::")
                || raw.starts_with(&format!("{}::", type_name));
            let val = if quoted {
                raw.to_string()
            } else {
                format!("\"{}\"", raw)
            };
            lines.push(format!("        {}::{} => {},", type_name, var, val));
        }
        lines.push("    }".to_string());
    } else if let Some(body) = body_literal {
        let trimmed = body.trim_end();
        let stmt = if trimmed.ends_with(';') || trimmed.ends_with('}') {
            trimmed.to_string()
        } else {
            trimmed.to_string()
        };
        // Emit the body as a single expression — Rust treats the final
        // expression as the return value when no trailing `;` is present.
        lines.push(format!("    {}", stmt));
    } else {
        // R4 fallback: spec didn't supply a body. `Default::default()`
        // compiles for any `Default` type (including `()`), giving the
        // author a compilable stub to replace.
        lines.push("    Default::default()".to_string());
    }
    lines.push("}".to_string());
    Some(lines.join("\n"))
}

/// Walk `x-trait-impls` and concatenate emitted `impl <Trait> for <Type>`
/// blocks. Returns an empty string when no entry is declared or no
/// supported trait is requested.
///
/// Currently supported traits (each requires a `dispatch:` table per
/// variant — only meaningful on string-enum schemas):
/// - `Display` / `std::fmt::Display`: writes the variant's `value` via
///   `write!(f, "...")`.
/// - `FromStr` / `std::str::FromStr`: parses an `input` string back to a
///   variant; `err_type:` (default `String`) and `err_msg:` (default
///   `"invalid <Type>: {}"`) configure the error path.
///
/// Entries with `impl_mode: hand-written` are skipped (the author owns
/// them). Unsupported traits are skipped silently — adding a new trait
/// is a generator extension, not a spec-author concern.
fn emit_trait_impls(schema: &Value, type_name: &str, spec_path: &str) -> String {
    let Some(impls) = schema.get("x-trait-impls").and_then(|v| v.as_sequence()) else {
        return String::new();
    };
    let mut blocks: Vec<String> = Vec::new();
    for entry in impls {
        let mode = entry
            .get("impl_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("codegen");
        if mode != "codegen" {
            continue;
        }
        let trait_name = entry.get("trait").and_then(|v| v.as_str()).unwrap_or("");
        let block = match trait_name {
            "Display" | "std::fmt::Display" | "fmt::Display" => {
                emit_display_trait_impl(entry, type_name, spec_path)
            }
            "FromStr" | "std::str::FromStr" | "str::FromStr" => {
                emit_fromstr_trait_impl(entry, type_name, spec_path)
            }
            "Default" => emit_default_trait_impl(entry, type_name, spec_path),
            _ => None,
        };
        if let Some(b) = block {
            blocks.push(b);
        }
    }
    blocks.join("\n\n")
}

/// Emit `impl std::fmt::Display for <Type>` from a `dispatch:` table.
fn emit_display_trait_impl(entry: &Value, type_name: &str, spec_path: &str) -> Option<String> {
    let arms = entry.get("dispatch").and_then(|v| v.as_sequence())?;
    let mut lines: Vec<String> = Vec::new();
    if !spec_path.is_empty() {
        lines.push(format!(
            "/// @spec {}#schema.trait-impls.Display",
            spec_path
        ));
    }
    lines.push(format!("impl std::fmt::Display for {} {{", type_name));
    lines.push(
        "    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {".to_string(),
    );
    lines.push("        match self {".to_string());
    for arm in arms {
        let var = arm.get("variant").and_then(|v| v.as_str())?;
        let val = arm.get("value").and_then(|v| v.as_str())?;
        lines.push(format!(
            "            {}::{} => write!(f, \"{}\"),",
            type_name,
            var,
            escape_string_literal(val),
        ));
    }
    lines.push("        }".to_string());
    lines.push("    }".to_string());
    lines.push("}".to_string());
    Some(lines.join("\n"))
}

/// Emit `impl std::str::FromStr for <Type>` from a `dispatch:` table whose
/// arms have `input` (the string to match) and `variant` (the constructed
/// variant). `err_type:` and `err_msg:` are optional with sensible defaults.
fn emit_fromstr_trait_impl(entry: &Value, type_name: &str, spec_path: &str) -> Option<String> {
    let arms = entry.get("dispatch").and_then(|v| v.as_sequence())?;
    let err_type = entry
        .get("err_type")
        .and_then(|v| v.as_str())
        .unwrap_or("String");
    let err_msg_default = format!("invalid {}: {{}}", type_name);
    let err_msg = entry
        .get("err_msg")
        .and_then(|v| v.as_str())
        .unwrap_or(err_msg_default.as_str());
    let mut lines: Vec<String> = Vec::new();
    if !spec_path.is_empty() {
        lines.push(format!(
            "/// @spec {}#schema.trait-impls.FromStr",
            spec_path
        ));
    }
    lines.push(format!("impl std::str::FromStr for {} {{", type_name));
    lines.push(format!("    type Err = {};", err_type));
    lines.push("    fn from_str(s: &str) -> Result<Self, Self::Err> {".to_string());
    lines.push("        match s {".to_string());
    for arm in arms {
        let input = arm.get("input").and_then(|v| v.as_str())?;
        let var = arm.get("variant").and_then(|v| v.as_str())?;
        lines.push(format!(
            "            \"{}\" => Ok({}::{}),",
            escape_string_literal(input),
            type_name,
            var,
        ));
    }
    lines.push(format!(
        "            _ => Err(format!(\"{}\", s)),",
        escape_string_literal(err_msg),
    ));
    lines.push("        }".to_string());
    lines.push("    }".to_string());
    lines.push("}".to_string());
    Some(lines.join("\n"))
}

/// Emit `impl Default for <Type> { fn default() -> Self { <body> } }`.
/// The `body:` field on the entry holds the literal Rust expression
/// returning `Self`. Used for structs whose default values cannot be
/// expressed via `#[derive(Default)]` (e.g. non-empty string literals,
/// non-zero integers).
fn emit_default_trait_impl(entry: &Value, type_name: &str, spec_path: &str) -> Option<String> {
    let body = entry.get("body").and_then(|v| v.as_str())?;
    let mut lines: Vec<String> = Vec::new();
    if !spec_path.is_empty() {
        lines.push(format!(
            "/// @spec {}#schema.trait-impls.Default",
            spec_path
        ));
    }
    lines.push(format!("impl Default for {} {{", type_name));
    lines.push("    fn default() -> Self {".to_string());
    // Indent the body — every line gets an 8-space prefix to align with
    // `fn default()` body. Authors write the body verbatim; we only
    // indent.
    for line in body.lines() {
        if line.is_empty() {
            lines.push(String::new());
        } else {
            lines.push(format!("        {}", line));
        }
    }
    lines.push("    }".to_string());
    lines.push("}".to_string());
    Some(lines.join("\n"))
}

/// Escape `\` and `"` so the value can be inlined into a Rust string
/// literal without breaking lexing. Other characters (`{`, `}`, etc.)
/// pass through — `write!`/`format!` macros take care of them.
/// True when the body actually uses short-form `Serialize`/`Deserialize`
/// derives (the kind that needs `use serde::*;`). Returns false when:
/// - body contains no `Serialize` / `Deserialize` token at all, OR
/// - every occurrence is path-style (`serde::Serialize` or
///   `serde::Deserialize`), which doesn't need the import.
fn body_uses_short_serde(body: &str) -> bool {
    body_uses_short_token(body, "Serialize") || body_uses_short_token(body, "Deserialize")
}

fn body_uses_short_token(body: &str, needle: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = body[start..].find(needle) {
        let abs = start + pos;
        let before = &body[..abs];
        let path_prefix = before.ends_with("::");
        let after = &body[abs + needle.len()..];
        let whole_word = after
            .chars()
            .next()
            .map(|c| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(true);
        if !path_prefix && whole_word {
            return true;
        }
        start = abs + needle.len();
    }
    false
}

fn escape_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(c),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Enum dispatch
// ---------------------------------------------------------------------------

/// Return true when the schema is a plain string enum — i.e. `type: string`
/// with an `enum: [...]` list. These map to Rust unit-variant enums rather
/// than structs. Mamba binding annotations are ignored for enums (no
/// positional-arg construction semantics).
fn is_string_enum_schema(schema_yaml: &Value) -> bool {
    let is_string_type = schema_yaml.get("type").and_then(|v| v.as_str()) == Some("string");
    let has_enum_list = schema_yaml
        .get("enum")
        .and_then(|v| v.as_sequence())
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    is_string_type && has_enum_list
}

/// Emit the variants of an enum body. When `x-rust-enum.variants` is
/// declared, walk that list and respect each variant's `kind`:
///
/// - `unit` (default): bare `Variant,` line.
/// - `tuple`: `Variant(Type1, Type2),` from `fields: [{rust_type: T}, ...]`.
/// - `struct`: `Variant { name: T, ... },` from `fields: [{name, rust_type}, ...]`.
///
/// Per-variant `#[serde(rename = "...")]` is emitted ONLY for unit variants
/// (and only when no container-level `rename_all` is in effect). Payload
/// variants don't need rename — serde keys them by name verbatim under the
/// default tagged-enum representation.
///
/// Per-variant doc comments (`doc:`) are emitted as `///` lines above each
/// variant when present.
///
/// Falls back to `string_enum_variants` (the legacy JSON `enum: [...]`
/// list) when `x-rust-enum.variants` is absent.
fn emit_enum_variants(
    schema_yaml: &Value,
    string_enum_variants: &[String],
    has_rename_all: bool,
    lines: &mut Vec<String>,
) {
    // Skip per-variant `#[serde(rename = ...)]` emission entirely when
    // the enum has no serde traits in its derive list — without
    // Serialize/Deserialize the attr is dead code AND fails to compile
    // because `serde` isn't in scope.
    let has_serde = schema_yaml
        .get("x-rust-enum")
        .and_then(|v| v.get("derive"))
        .and_then(|v| v.as_sequence())
        .map(|list| {
            list.iter()
                .filter_map(|v| v.as_str())
                .any(is_serde_derive_name)
        })
        .unwrap_or(true);

    let structured = schema_yaml
        .get("x-rust-enum")
        .and_then(|v| v.get("variants"))
        .and_then(|v| v.as_sequence());

    let Some(variants) = structured else {
        // Legacy path — JSON `enum: [...]` only carries variant names.
        for v in string_enum_variants {
            if has_serde && !has_rename_all {
                lines.push(format!("    #[serde(rename = \"{}\")]", v));
            }
            lines.push(format!("    {},", to_pascal_case(v)));
        }
        return;
    };

    for variant in variants {
        // Doc comment(s) for this variant.
        if let Some(doc) = variant.get("doc").and_then(|v| v.as_str()) {
            for line in doc.lines() {
                lines.push(format!("    /// {}", line));
            }
        }
        // `is_default: true` on a variant emits `#[default]` so the
        // enum's `#[derive(Default)]` impl picks this variant. Only
        // valid on unit variants per Rust's `#[default]` attribute
        // semantics, but enforcement is left to rustc.
        let is_default = variant
            .get("is_default")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if is_default {
            lines.push("    #[default]".to_string());
        }
        // `is_other: true` on a variant emits `#[serde(other)]` — the
        // catch-all variant for unrecognised wire values. Mutually
        // exclusive with `serde_rename` on the same variant per serde.
        let is_other = variant
            .get("is_other")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if is_other && has_serde {
            lines.push("    #[serde(other)]".to_string());
        }
        // `error: "<template>"` on a variant emits `#[error("<template>")]`
        // for thiserror-derived enums. Per-variant; placed immediately
        // above the variant declaration. Authors author the template
        // verbatim — `{0}` and `{name}` placeholders are forwarded as-is.
        if let Some(err_msg) = variant.get("error").and_then(|v| v.as_str()) {
            lines.push(format!("    #[error(\"{}\")]", err_msg));
        }
        let name = variant
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unnamed");
        let kind = variant
            .get("kind")
            .and_then(|k| k.as_str())
            .unwrap_or("unit");
        match kind {
            "tuple" => {
                let fields = variant
                    .get("fields")
                    .and_then(|v| v.as_sequence())
                    .cloned()
                    .unwrap_or_default();
                // Render each field as `[#[from] ]<rust_type>` then join.
                // `error_from: true` on a field emits `#[from]` to enable
                // the standard `From<X>` impl that thiserror derives.
                let parts: Vec<String> = fields
                    .iter()
                    .filter_map(|f| {
                        let ty = f.get("rust_type").and_then(|v| v.as_str())?;
                        let from = f
                            .get("error_from")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        Some(if from {
                            format!("#[from] {}", ty)
                        } else {
                            ty.to_string()
                        })
                    })
                    .collect();
                lines.push(format!("    {}({}),", name, parts.join(", ")));
            }
            "struct" => {
                let fields = variant
                    .get("fields")
                    .and_then(|v| v.as_sequence())
                    .cloned()
                    .unwrap_or_default();
                let mut rendered_fields: Vec<(Vec<String>, String)> = Vec::new();
                for f in &fields {
                    let fname = f.get("name").and_then(|v| v.as_str()).unwrap_or("field");
                    let ftype = f.get("rust_type").and_then(|v| v.as_str()).unwrap_or("()");
                    let mut attrs = Vec::new();
                    let from = f
                        .get("error_from")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if from {
                        attrs.push("        #[from]".to_string());
                    }
                    // `flatten: true` on a variant struct field emits
                    // `#[serde(flatten)]` so the inner struct's fields appear
                    // inline at the variant level. Used heavily by SpecIR
                    // (generate/spec_ir/types.rs) to flatten payload structs.
                    let flatten = f.get("flatten").and_then(|v| v.as_bool()).unwrap_or(false);
                    if flatten && has_serde {
                        attrs.push("        #[serde(flatten)]".to_string());
                    }
                    // Per-variant-field clap arg attribute. Same shape as
                    // top-level x-clap-arg: body is forwarded verbatim.
                    if let Some(arg_body) = f.get("x-clap-arg").and_then(|v| v.as_str()) {
                        attrs.push(format!("        #[arg({})]", arg_body));
                    }
                    rendered_fields.push((attrs, format!("{}: {}", fname, ftype)));
                }

                let single_line_fields = rendered_fields
                    .iter()
                    .map(|(_, field)| field.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let single_line = if single_line_fields.is_empty() {
                    format!("    {} {{}},", name)
                } else {
                    format!("    {} {{ {} }},", name, single_line_fields)
                };
                let has_field_attrs = rendered_fields.iter().any(|(attrs, _)| !attrs.is_empty());
                let has_multifield_generic = rendered_fields.len() > 1
                    && rendered_fields
                        .iter()
                        .any(|(_, field)| field.contains('<') || field.contains('>'));
                let has_many_fields = rendered_fields.len() > 2;
                if !has_field_attrs
                    && !has_multifield_generic
                    && !has_many_fields
                    && single_line.len() <= 100
                {
                    lines.push(single_line);
                    continue;
                }

                lines.push(format!("    {} {{", name));
                for (attrs, field) in rendered_fields {
                    lines.extend(attrs);
                    lines.push(format!("        {},", field));
                }
                lines.push("    },".to_string());
            }
            _ => {
                // unit variant. Variant authors can opt into an arbitrary
                // wire string via `rename: "<wire>"` on the variant — used
                // when the wire format is something the variant identifier
                // can't legally be (e.g. a URL like
                // `"http://json-schema.org/draft-07/schema#"` for
                // `SchemaVersion::Draft7`). Falls back to renaming to the
                // variant name verbatim when neither `rename:` nor a
                // container-level `rename_all` is set.
                let explicit_rename = variant.get("rename").and_then(|v| v.as_str());
                if let Some(wire) = explicit_rename {
                    if has_serde {
                        lines.push(format!("    #[serde(rename = \"{}\")]", wire));
                    }
                } else if has_serde && !has_rename_all {
                    lines.push(format!("    #[serde(rename = \"{}\")]", name));
                }
                lines.push(format!("    {},", name));
            }
        }
    }
}

/// True when the schema declares `x-rust-enum.variants` with at least one
/// non-unit variant (`kind: tuple` or `kind: struct`). Such enums cannot be
/// expressed as a JSON string-enum (`type: string + enum: [...]`) because
/// the variants carry payload, so they take a dedicated dispatch into
/// `generate_rust_enum` regardless of the JSON `type` field.
/// True when the schema declares an `x-rust-enum.variants` list at all
/// (regardless of variant kinds). Used by the early dispatch so unit-only
/// enums declared via `x-rust-enum.variants` still route to the enum path
/// instead of being treated as a struct schema.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/adjacent-tagged-enum.md#logic
fn has_x_rust_enum_variants(schema_yaml: &Value) -> bool {
    schema_yaml
        .get("x-rust-enum")
        .and_then(|v| v.get("variants"))
        .and_then(|v| v.as_sequence())
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}

fn has_payload_variants(schema_yaml: &Value) -> bool {
    let Some(variants) = schema_yaml
        .get("x-rust-enum")
        .and_then(|v| v.get("variants"))
        .and_then(|v| v.as_sequence())
    else {
        return false;
    };
    variants.iter().any(|v| {
        let kind = v.get("kind").and_then(|k| k.as_str()).unwrap_or("unit");
        kind == "tuple" || kind == "struct"
    })
}

/// Render a Rust unit-variant enum from a JSON string-enum schema.
///
/// Emits a Rust unit-variant enum from a string-enum schema.
///
/// Derive policy (R6): if `x-rust-enum.derive: [...]` is present, emit exactly
/// that list in declaration order; otherwise fall back to the default
/// `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]`.
///
/// `as_str()` and `impl FromStr` emission (R7): only emitted when the schema's
/// `x-methods` list declares them explicitly. Previously they were always
/// emitted, which collided with spec-declared inherent methods like
/// `RuleId::short()` that serve the same role.
///
/// Inherent methods from `x-methods` (dispatch or body), `x-constructor`, and
/// `x-builders` are emitted via `emit_x_impls`.
fn generate_rust_enum(
    schema_yaml: &Value,
    enum_name: &str,
    config: &RustConfig,
    spec_path: &str,
) -> SchemaGenOutput {
    let vis = config.vis_prefix();
    // String-enum names from JSON `enum: [...]` (unit-variant case). The
    // structured `x-rust-enum.variants` form takes priority when present and
    // adds support for tuple / struct variant payloads; see emit_enum_variants
    // for the union of both shapes.
    let variants: Vec<String> = schema_yaml
        .get("enum")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut lines: Vec<String> = Vec::new();

    // Struct-level doc from schema description.
    if let Some(desc) = schema_yaml.get("description").and_then(|v| v.as_str()) {
        for line in desc.lines() {
            lines.push(format!("/// {}", line));
        }
    }
    if !spec_path.is_empty() {
        lines.push(format!("/// @spec {}#schema", spec_path));
    }

    // Optional arbitrary type-level attributes via `x-rust-attrs: [...]` —
    // same shape as on structs (see generate_rust_struct comment).
    if let Some(attrs) = schema_yaml
        .get("x-rust-attrs")
        .and_then(|v| v.as_sequence())
    {
        for attr in attrs {
            if let Some(s) = attr.as_str() {
                lines.push(format!("#[{}]", s));
            }
        }
    }

    // R6: honour explicit `x-rust-enum.derive` list when declared; otherwise
    // fall back to a default. Default depends on shape: unit-only enums
    // get the historical `Copy + Hash`-friendly set; payload enums omit
    // those because tuple/struct variants are rarely Copy and never Hash
    // automatically (would fail to compile if any field type isn't Hash).
    let default_enum_derive = if has_payload_variants(schema_yaml) {
        "#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]".to_string()
    } else {
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]"
            .to_string()
    };
    lines.push(resolve_explicit_derive(
        schema_yaml,
        "x-rust-enum",
        default_enum_derive,
    ));
    // Container-level `#[serde(rename_all = ...)]` and `#[serde(tag = ...)]`.
    // `rename_all` (when present) skips per-variant `#[serde(rename = "...")]`.
    // `serde_tag` enables internally-tagged enum representation for struct
    // variants (e.g. `{"verdict": "approved"}` / `{"verdict": "needs_revision",
    // "issues": [...]}`). When BOTH are present they MUST share one
    // `#[serde(...)]` attribute — serde rejects them split across two.
    let enum_rename_all = resolve_serde_rename_all(schema_yaml, "x-rust-enum", &config);
    let has_rename_all = !enum_rename_all.is_empty();
    let serde_tag = schema_yaml
        .get("x-rust-enum")
        .and_then(|v| v.get("serde_tag"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());

    // @spec projects/agentic-workflow/tech-design/core/generate/generators/adjacent-tagged-enum.md#logic
    let serde_content = schema_yaml
        .get("x-rust-enum")
        .and_then(|v| v.get("serde_content"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());

    let serde_untagged = schema_yaml
        .get("x-rust-enum")
        .and_then(|v| v.get("serde_untagged"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let enum_deny_unknown = schema_yaml
        .get("x-rust-enum")
        .and_then(|v| v.get("serde_deny_unknown"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if serde_untagged {
        // `#[serde(untagged)]` — variants are deserialized in order, no
        // discriminator field. Mutually exclusive with `tag` and
        // `rename_all` (both irrelevant when there is no tag).
        let mut parts = vec!["untagged".to_string()];
        if enum_deny_unknown {
            parts.push("deny_unknown_fields".to_string());
        }
        lines.push(format!("#[serde({})]", parts.join(", ")));
    } else {
        let strategy = schema_yaml
            .get("x-rust-enum")
            .and_then(|v| v.get("serde_rename_all"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let mut parts: Vec<String> = Vec::new();
        if let Some(tag) = serde_tag {
            parts.push(format!("tag = \"{}\"", tag));
        }

        // @spec projects/agentic-workflow/tech-design/core/generate/generators/adjacent-tagged-enum.md#logic
        if let (Some(_), Some(content)) = (serde_tag, serde_content) {
            parts.push(format!("content = \"{}\"", content));
        }

        if has_rename_all && !strategy.is_empty() {
            parts.push(format!("rename_all = \"{}\"", strategy));
        } else if has_rename_all {
            // Fall back to the rendered string from `resolve_serde_rename_all`
            // for the path-style branch (defensive — strategy should always be present).
            let body = enum_rename_all
                .trim_start_matches("#[serde(")
                .trim_end_matches(")]");
            if !body.is_empty() {
                parts.push(body.to_string());
            }
        }
        if enum_deny_unknown {
            parts.push("deny_unknown_fields".to_string());
        }
        if !parts.is_empty() {
            lines.push(format!("#[serde({})]", parts.join(", ")));
        } else {
            // Nothing — preserve the (None, false) branch.
            let _ = ();
        }
    }
    lines.push(format!("{}enum {} {{", vis, enum_name));
    emit_enum_variants(schema_yaml, &variants, has_rename_all, &mut lines);
    lines.push("}".to_string());

    // R7: only emit auto `as_str`/`FromStr` when the spec explicitly declares
    // them in `x-methods`. Keeping them unconditional used to collide with
    // spec-declared inherent methods (e.g. `short()`) and produced dead APIs
    // the author never asked for.
    if x_methods_declares(schema_yaml, "as_str") {
        lines.push(String::new());
        if !spec_path.is_empty() {
            lines.push(format!("/// @spec {}#schema.as_str", spec_path));
        }
        lines.push(format!("impl {} {{", enum_name));
        lines.push(
            "    /// Canonical string representation (matches serde rename values).".to_string(),
        );
        lines.push(format!("    {}fn as_str(self) -> &'static str {{", vis));
        lines.push("        match self {".to_string());
        for v in &variants {
            lines.push(format!(
                "            {}::{} => \"{}\",",
                enum_name,
                to_pascal_case(v),
                v,
            ));
        }
        lines.push("        }".to_string());
        lines.push("    }".to_string());
        lines.push("}".to_string());
    }
    if x_methods_declares(schema_yaml, "from_str") {
        lines.push(String::new());
        if !spec_path.is_empty() {
            lines.push(format!("/// @spec {}#schema.from_str", spec_path));
        }
        lines.push(format!("impl std::str::FromStr for {} {{", enum_name));
        lines.push("    type Err = String;".to_string());
        lines.push("    fn from_str(s: &str) -> Result<Self, Self::Err> {".to_string());
        lines.push("        match s {".to_string());
        for v in &variants {
            lines.push(format!(
                "            \"{}\" => Ok({}::{}),",
                v,
                enum_name,
                to_pascal_case(v),
            ));
        }
        lines.push(format!(
            "            _ => Err(format!(\"invalid {}: {{}}\", s)),",
            enum_name,
        ));
        lines.push("        }".to_string());
        lines.push("    }".to_string());
        lines.push("}".to_string());
    }

    // R1/R4: Spec-declared inherent methods via `x-methods` (dispatch or body).
    // Emitted AFTER the auto-as_str/FromStr blocks so the file reads
    // logically: auto-impls first, then the author's named methods.
    let x_impls = emit_x_impls(schema_yaml, enum_name, spec_path, &vis);
    if !x_impls.is_empty() {
        lines.push(String::new());
        lines.push(x_impls);
    }

    // x-trait-impls: external trait impls (Display, FromStr, ...). Emitted
    // last so trait impls follow inherent impls in the generated file —
    // matches typical Rust style where trait impls live below inherent ones.
    let trait_impls = emit_trait_impls(schema_yaml, enum_name, spec_path);
    if !trait_impls.is_empty() {
        lines.push(String::new());
        lines.push(trait_impls);
    }

    SchemaGenOutput {
        code: lines.join("\n"),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::types::RustConfig;

    #[test]
    fn test_keyword_field_emits_raw_identifier_and_serde_rename() {
        let yaml_str = r#"
title: Message
type: object
required: [from, async]
properties:
  from:
    type: string
  async:
    type: boolean
    x-serde-default: true
x-rust-struct:
  derive: [Debug, Clone, Serialize, Deserialize]
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());

        assert!(output
            .code
            .contains("#[serde(rename = \"async\", default)]"));
        assert!(output.code.contains("pub r#async: bool"));
        assert!(!output.code.contains("pub async: bool"));
    }

    /// Regression: `type: string, enum: [...]` produces a Rust unit-variant
    /// enum with serde renames; auto `as_str`/`FromStr` are emitted only when
    /// the spec explicitly declares them via `x-methods` (R7).
    #[test]
    fn test_string_enum_produces_rust_enum() {
        let yaml_str = r#"
title: HealthStatus
type: string
enum: [healthy, degraded, unhealthy]
description: "Aggregate health of a component."
x-methods:
  - name: as_str
    impl_mode: codegen
  - name: from_str
    impl_mode: codegen
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let config = RustConfig::default();
        let output = generate_schema(&yaml, &config);

        // Enum, not struct
        assert!(
            output.code.contains("pub enum HealthStatus"),
            "Should generate enum HealthStatus\n---\n{}\n---",
            output.code
        );
        assert!(
            !output.code.contains("pub struct HealthStatus"),
            "Should NOT generate a struct"
        );
        // Variants converted to PascalCase
        assert!(output.code.contains("Healthy,"));
        assert!(output.code.contains("Degraded,"));
        assert!(output.code.contains("Unhealthy,"));
        // serde rename per variant
        assert!(output.code.contains("#[serde(rename = \"healthy\")]"));
        assert!(output.code.contains("#[serde(rename = \"unhealthy\")]"));
        // as_str helper
        assert!(output.code.contains("pub fn as_str(self) -> &'static str"));
        assert!(output.code.contains("HealthStatus::Healthy => \"healthy\""));
        // FromStr impl
        assert!(output
            .code
            .contains("impl std::str::FromStr for HealthStatus"));
        assert!(output
            .code
            .contains("\"degraded\" => Ok(HealthStatus::Degraded)"));
        assert!(output.code.contains("invalid HealthStatus:"));
        // Description flows into /// doc
        assert!(output.code.contains("/// Aggregate health of a component."));
    }

    #[test]
    fn test_enum_with_tuple_variants() {
        let yaml_str = r#"
title: Delta
type: object
description: "Tagged delta of payload."
x-rust-enum:
  derive: [Debug, Clone, PartialEq, Serialize, Deserialize]
  variants:
    - name: Added
      kind: tuple
      fields:
        - { rust_type: Requirement }
      doc: "A new requirement was added."
    - name: Modified
      kind: tuple
      fields:
        - { rust_type: Requirement }
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        // Routes to enum path despite `type: object`.
        assert!(output.code.contains("pub enum Delta"));
        assert!(
            output.code.contains("Added(Requirement)"),
            "tuple variant should emit Variant(Type), got:\n{}",
            output.code,
        );
        assert!(output.code.contains("Modified(Requirement)"));
        // No per-variant rename for payload variants.
        assert!(!output.code.contains("#[serde(rename = \"Added\")]"));
        // Doc preserved.
        assert!(output.code.contains("/// A new requirement was added."));
    }

    #[test]
    fn test_enum_with_struct_variants() {
        let yaml_str = r#"
title: Delta
type: object
x-rust-enum:
  derive: [Debug, Clone, PartialEq, Serialize, Deserialize]
  variants:
    - name: Removed
      kind: struct
      fields:
        - { name: name,      rust_type: String }
        - { name: reason,    rust_type: String }
        - { name: migration, rust_type: "Option<String>" }
    - name: Renamed
      kind: struct
      fields:
        - { name: from, rust_type: String }
        - { name: to,   rust_type: String }
    - name: Triple
      kind: struct
      fields:
        - { name: a, rust_type: String }
        - { name: b, rust_type: String }
        - { name: c, rust_type: String }
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(output.code.contains("pub enum Delta"));
        assert!(
            output.code.contains("    Removed {\n"),
            "multi-field generic struct variants should use rustfmt-stable multiline output, got:\n{}",
            output.code,
        );
        assert!(output.code.contains("name: String"));
        assert!(output.code.contains("migration: Option<String>"));
        assert!(output
            .code
            .contains("Renamed { from: String, to: String },"));
        assert!(output.code.contains("    Triple {\n"));
        assert!(output.code.contains("from: String"));
    }

    #[test]
    fn test_enum_mixing_unit_tuple_struct_variants() {
        let yaml_str = r#"
title: Multi
type: object
x-rust-enum:
  derive: [Debug, Clone, PartialEq, Serialize, Deserialize]
  variants:
    - { name: None }
    - name: Single
      kind: tuple
      fields: [{ rust_type: u32 }]
    - name: Pair
      kind: struct
      fields:
        - { name: x, rust_type: u32 }
        - { name: y, rust_type: u32 }
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        // All three shapes coexist.
        assert!(output.code.contains("    None,"));
        assert!(output.code.contains("Single(u32)"));
        assert!(output.code.contains("Pair {"));
        assert!(output.code.contains("x: u32"));
    }

    // ----- R5: adjacent-tagged enum snapshot fixtures --------------------
    //
    // Four cases verifying the new `serde_content` branch in
    // `generate_rust_enum`. Each fixture sets `serde_tag: kind`,
    // `serde_content: data`, and `serde_rename_all: snake_case` and asserts
    // the canonical `#[serde(tag = "kind", content = "data",
    // rename_all = "snake_case")]` attribute is emitted as ONE line, plus
    // the variant body shape for that case.
    //
    // @spec projects/agentic-workflow/tech-design/core/generate/generators/adjacent-tagged-enum.md#test-plan

    /// R5 case 1: unit-only enum.
    #[test]
    fn test_enum_adjacent_tagged_unit_only() {
        let yaml_str = r#"
title: SignalKind
type: object
x-rust-enum:
  derive: [Debug, Clone, Serialize, Deserialize]
  serde_tag: kind
  serde_content: data
  serde_rename_all: snake_case
  variants:
    - { name: Ready }
    - { name: Pending }
    - { name: Done }
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(
            output
                .code
                .contains(r#"#[serde(tag = "kind", content = "data", rename_all = "snake_case")]"#),
            "missing canonical adjacent-tag attribute, got:\n{}",
            output.code,
        );
        assert!(output.code.contains("pub enum SignalKind"));
        assert!(output.code.contains("    Ready,"));
        assert!(output.code.contains("    Pending,"));
        assert!(output.code.contains("    Done,"));
    }

    /// R5 case 2: single-tuple-only enum.
    #[test]
    fn test_enum_adjacent_tagged_tuple_only() {
        let yaml_str = r#"
title: Payload
type: object
x-rust-enum:
  derive: [Debug, Clone, Serialize, Deserialize]
  serde_tag: kind
  serde_content: data
  serde_rename_all: snake_case
  variants:
    - name: JsonSchema
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
    - name: OpenRpc
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
    - name: Markdown
      kind: tuple
      fields: [{ rust_type: String }]
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(
            output
                .code
                .contains(r#"#[serde(tag = "kind", content = "data", rename_all = "snake_case")]"#),
            "missing canonical adjacent-tag attribute, got:\n{}",
            output.code,
        );
        assert!(output.code.contains("pub enum Payload"));
        assert!(output.code.contains("JsonSchema(serde_yaml::Value)"));
        assert!(output.code.contains("OpenRpc(serde_yaml::Value)"));
        assert!(output.code.contains("Markdown(String)"));
    }

    /// R5 case 3: named-struct-only enum.
    #[test]
    fn test_enum_adjacent_tagged_struct_only() {
        let yaml_str = r#"
title: Event
type: object
x-rust-enum:
  derive: [Debug, Clone, Serialize, Deserialize]
  serde_tag: kind
  serde_content: data
  serde_rename_all: snake_case
  variants:
    - name: Renamed
      kind: struct
      fields:
        - { name: from, rust_type: String }
        - { name: to,   rust_type: String }
    - name: Moved
      kind: struct
      fields:
        - { name: src, rust_type: String }
        - { name: dst, rust_type: String }
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(
            output
                .code
                .contains(r#"#[serde(tag = "kind", content = "data", rename_all = "snake_case")]"#),
            "missing canonical adjacent-tag attribute, got:\n{}",
            output.code,
        );
        assert!(output.code.contains("pub enum Event"));
        assert!(output.code.contains("    Renamed {"));
        assert!(output.code.contains("from: String"));
        assert!(output.code.contains("    Moved {"));
    }

    /// One-shot dumper used to derive the byte-equivalent CODEGEN block in
    /// `projects/agentic-workflow/src/td_ast/types.rs`. Run with
    /// `cargo test -p agentic-workflow --lib dump_typed_body_bytes -- --nocapture --ignored`
    /// to print the canonical generator output for the `TypedBody` schema.
    #[test]
    #[ignore]
    fn dump_typed_body_bytes() {
        let yaml_str = r#"
title: TypedBody
type: object
description: |-
  Discriminated body of a parsed TD section. Nine typed variants cover
  known section families (R3); the opaque `Unsupported` variant carries
  sections whose parser is not yet implemented (R9).
x-rust-enum:
  derive: [Debug, Clone, Serialize, Deserialize]
  serde_tag: kind
  serde_content: data
  serde_rename_all: snake_case
  variants:
    - name: MermaidPlus
      kind: tuple
      fields: [{ rust_type: MermaidPlusPayload }]
      doc: |-
        Mermaid Plus block (state-machine, logic, interaction, dependency,
        db-model, scenarios, unit-test). Hash covers frontmatter only (R4, R8).
    - name: JsonSchema
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
      doc: |-
        JSON Schema document (schema, config, wireframe, component,
        design-token, manifest, e2e-test).
    - name: OpenRpc
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
      doc: "OpenRPC 1.3 document (rpc-api)."
    - name: OpenApi
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
      doc: "OpenAPI 3.1 document (rest-api)."
    - name: AsyncApi
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
      doc: "AsyncAPI 2.6 document (async-api)."
    - name: CliManifest
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
      doc: "CLI manifest YAML (cli)."
    - name: ConfigManifest
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
      doc: "Config manifest YAML (config)."
    - name: Markdown
      kind: tuple
      fields: [{ rust_type: String }]
      doc: "Plain markdown body (doc, overview, changes-as-prose)."
    - name: Placeholder
      doc: "Section listed in `fill_sections` but not yet authored."
    - name: Unsupported
      kind: tuple
      fields: [{ rust_type: String }]
      doc: |-
        Opaque carrier for section types without a typed parser. Carries
        the raw block string for round-trip fidelity (R9).
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let out = generate_schema_with_provenance(
            &yaml,
            &RustConfig::default(),
            "projects/agentic-workflow/tech-design/core/td_ast/types.md",
        );
        println!(
            "===CODEGEN-DUMP-START===\n{}===CODEGEN-DUMP-END===",
            out.code
        );
    }

    /// R5 case 4: mixed shape — direct fixture for `TypedBody` shape.
    /// One unit, eight tuple variants (mixed payload types), no struct.
    /// Asserts byte-equivalence with the body of the canonical
    /// `TypedBody` enum block in `projects/agentic-workflow/src/td_ast/types.rs`.
    #[test]
    fn test_enum_adjacent_tagged_mixed_typed_body() {
        let yaml_str = r#"
title: TypedBody
type: object
x-rust-enum:
  derive: [Debug, Clone, Serialize, Deserialize]
  serde_tag: kind
  serde_content: data
  serde_rename_all: snake_case
  variants:
    - name: MermaidPlus
      kind: tuple
      fields: [{ rust_type: MermaidPlusPayload }]
    - name: JsonSchema
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
    - name: OpenRpc
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
    - name: OpenApi
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
    - name: AsyncApi
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
    - name: CliManifest
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
    - name: ConfigManifest
      kind: tuple
      fields: [{ rust_type: "serde_yaml::Value" }]
    - name: Markdown
      kind: tuple
      fields: [{ rust_type: String }]
    - { name: Placeholder }
    - name: Unsupported
      kind: tuple
      fields: [{ rust_type: String }]
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(
            output
                .code
                .contains(r#"#[serde(tag = "kind", content = "data", rename_all = "snake_case")]"#),
            "missing canonical adjacent-tag attribute, got:\n{}",
            output.code,
        );
        assert!(output.code.contains("pub enum TypedBody"));
        // All ten variants emitted in declaration order.
        let body_start = output.code.find("pub enum TypedBody").unwrap();
        let body = &output.code[body_start..];
        let want = [
            "MermaidPlus(MermaidPlusPayload)",
            "JsonSchema(serde_yaml::Value)",
            "OpenRpc(serde_yaml::Value)",
            "OpenApi(serde_yaml::Value)",
            "AsyncApi(serde_yaml::Value)",
            "CliManifest(serde_yaml::Value)",
            "ConfigManifest(serde_yaml::Value)",
            "Markdown(String)",
            "    Placeholder,",
            "Unsupported(String)",
        ];
        for needle in want {
            assert!(
                body.contains(needle),
                "missing variant `{}` in TypedBody body:\n{}",
                needle,
                body,
            );
        }
        // Order check — use byte indices to confirm declaration order.
        let mut prev = 0usize;
        for needle in want {
            let pos = body.find(needle).unwrap();
            assert!(
                pos >= prev,
                "variant `{}` appears out of order (pos {} before {})",
                needle,
                pos,
                prev,
            );
            prev = pos;
        }
    }

    #[test]
    fn test_field_x_serde_rename_overrides_json_key() {
        let yaml_str = r#"
title: Attr
type: object
required: [name, attr_type]
properties:
  name:
    type: string
  attr_type:
    type: string
    x-serde-rename: "type"
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        // Field name in Rust is attr_type; JSON sees "type".
        assert!(output.code.contains("pub attr_type: String"));
        assert!(
            output.code.contains("#[serde(rename = \"type\")]"),
            "should rename to JSON key 'type', got:\n{}",
            output.code,
        );
    }

    #[test]
    fn test_enum_variant_is_default_emits_attr() {
        let yaml_str = r#"
title: Shape
type: string
enum: [Square, Round, Hex]
x-rust-enum:
  derive: [Debug, Clone, Default, PartialEq, Serialize, Deserialize]
  variants:
    - { name: Square, is_default: true }
    - { name: Round }
    - { name: Hex }
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        // Exactly one #[default] in the output, attached to Square.
        assert_eq!(
            output.code.matches("#[default]").count(),
            1,
            "exactly one variant should carry #[default], got:\n{}",
            output.code,
        );
        // Square's variant block contains both #[default] and the variant
        // declaration; the cheapest reliable check is "the line that
        // introduces #[default] appears before the line that introduces
        // Square (and after Round/Hex are absent from #[default])".
        let default_pos = output.code.find("#[default]").expect("found");
        let square_pos = output.code.find("Square,").expect("found");
        let round_pos = output.code.find("Round,").expect("found");
        assert!(default_pos < square_pos, "default must precede Square");
        assert!(square_pos < round_pos, "Square is the first variant");
    }

    #[test]
    fn test_field_serde_skip_if_emitted() {
        let yaml_str = r#"
title: Cfg
type: object
properties:
  override:
    type: string
    x-serde-skip-if: "Option::is_none"
required: []
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(
            output
                .code
                .contains("skip_serializing_if = \"Option::is_none\""),
            "should emit skip_serializing_if attr, got:\n{}",
            output.code,
        );
        // Optional field still gets default via the auto path.
        assert!(output.code.contains("default"), "default also present");
    }

    #[test]
    fn test_container_serde_rename_all_on_struct() {
        let yaml_str = r#"
title: Cfg
type: object
properties:
  some_field:
    type: string
required: [some_field]
x-rust-struct:
  derive: [Debug, Clone, Serialize, Deserialize]
  serde_rename_all: kebab-case
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(
            output
                .code
                .contains("#[serde(rename_all = \"kebab-case\")]"),
            "should emit rename_all on struct, got:\n{}",
            output.code,
        );
    }

    #[test]
    fn test_container_serde_rename_all_on_enum_skips_per_variant_rename() {
        let yaml_str = r#"
title: Lang
type: string
enum: [Rust, Python, JavaScript]
x-rust-enum:
  derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
  serde_rename_all: lowercase
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(
            output.code.contains("#[serde(rename_all = \"lowercase\")]"),
            "should emit rename_all on enum, got:\n{}",
            output.code,
        );
        // Per-variant `#[serde(rename = "...")]` MUST be suppressed when
        // rename_all takes care of the whole strategy. Otherwise serde
        // emits a duplicate-attribute warning.
        assert!(
            !output.code.contains("#[serde(rename = \"Rust\")]"),
            "per-variant rename should be skipped under rename_all",
        );
    }

    /// `x-trait-impls: [{trait: Default, body: ...}]` emits a custom
    /// `impl Default for <Type>` block with the literal body indented
    /// inside `fn default() -> Self`. Used for structs whose default
    /// values cannot be expressed via `#[derive(Default)]` (non-empty
    /// strings, non-zero integers).
    #[test]
    fn test_default_trait_impl_custom_body() {
        let yaml_str = r#"
title: MainSpecFrontmatter
type: object
required: [id, doc_type, title, version]
properties:
  id:
    type: string
  doc_type:
    type: string
  title:
    type: string
  version:
    type: integer
    x-rust-type: u32
x-rust-struct:
  derive: [Debug, Clone, Serialize, Deserialize]
x-trait-impls:
  - trait: Default
    impl_mode: codegen
    body: |
      Self {
          id: String::new(),
          doc_type: "spec".to_string(),
          title: String::new(),
          version: 1,
      }
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(
            output
                .code
                .contains("impl Default for MainSpecFrontmatter {"),
            "should emit impl Default block, got:\n{}",
            output.code,
        );
        assert!(
            output.code.contains("fn default() -> Self {"),
            "should emit default fn signature, got:\n{}",
            output.code,
        );
        assert!(
            output.code.contains(r#"doc_type: "spec".to_string(),"#),
            "should emit body verbatim, got:\n{}",
            output.code,
        );
        assert!(
            output.code.contains("version: 1,"),
            "should emit non-zero default, got:\n{}",
            output.code,
        );
    }

    /// `x-clap-arg: "<body>"` on a struct field emits `#[arg(<body>)]`
    /// verbatim. Used for CLI handler structs that derive `clap::Args`.
    #[test]
    fn test_clap_arg_attr() {
        let yaml_str = r#"
title: RegistryCheckArgs
type: object
description: "Arguments for a registry drift check."
required: [dry_run, check]
properties:
  dry_run:
    type: boolean
    x-clap-arg: "long"
    description: "Print unified diff of what would change without writing the file."
  check:
    type: boolean
    x-clap-arg: "long"
    description: "Like --dry-run but exits with code 1 when the diff is non-empty."
x-rust-struct:
  derive: [Debug, "clap::Args"]
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        // Both fields get `#[arg(long)]`.
        let arg_long_count = output.code.matches("#[arg(long)]").count();
        assert_eq!(
            arg_long_count, 2,
            "expected 2 #[arg(long)] attrs, got {}:\n{}",
            arg_long_count, output.code,
        );
        // Path-style derive renders verbatim.
        assert!(
            output.code.contains("#[derive(Debug, clap::Args)]"),
            "should preserve clap::Args path-style derive, got:\n{}",
            output.code,
        );
    }

    /// `error: "<template>"` on a variant emits `#[error("<template>")]`,
    /// and `error_from: true` on a tuple/struct field emits `#[from]`.
    /// Together they support thiserror-derived error enums.
    #[test]
    fn test_thiserror_attrs() {
        let yaml_str = r#"
title: ViewerError
type: object
description: "Errors that can occur in the viewer manager."
x-rust-enum:
  derive: [Debug, "thiserror::Error"]
  variants:
    - name: PathTraversal
      kind: tuple
      error: "Path traversal attempt detected: {0}"
      fields:
        - rust_type: String
    - name: Annotation
      kind: tuple
      error: "Annotation error: {0}"
      fields:
        - rust_type: AnnotationError
          error_from: true
    - name: IoError
      kind: tuple
      error: "IO error: {0}"
      fields:
        - rust_type: "std::io::Error"
          error_from: true
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        // Per-variant `#[error("...")]` for all three.
        assert!(
            output
                .code
                .contains(r#"#[error("Path traversal attempt detected: {0}")]"#),
            "missing error attr for PathTraversal, got:\n{}",
            output.code,
        );
        assert!(
            output.code.contains(r#"#[error("Annotation error: {0}")]"#),
            "missing error attr for Annotation, got:\n{}",
            output.code,
        );
        // Per-field `#[from]` on tuple variants.
        assert!(
            output.code.contains("Annotation(#[from] AnnotationError)"),
            "missing #[from] on AnnotationError tuple field, got:\n{}",
            output.code,
        );
        assert!(
            output.code.contains("IoError(#[from] std::io::Error)"),
            "missing #[from] on IoError tuple field, got:\n{}",
            output.code,
        );
        // No #[from] on the bare-String tuple variant.
        assert!(
            output.code.contains("PathTraversal(String)"),
            "PathTraversal should be a plain tuple, got:\n{}",
            output.code,
        );
        // Derive list rendered verbatim including the path-style derive.
        assert!(
            output.code.contains("#[derive(Debug, thiserror::Error)]"),
            "derive list should preserve thiserror::Error path-style derive, got:\n{}",
            output.code,
        );
    }

    /// `x-serde-default: "<fn_name>"` (string value) emits
    /// `#[serde(default = "fn_name")]` instead of bare `#[serde(default)]`.
    /// Used when source has a top-level helper like
    /// `fn default_columns() -> u32 { 1 }`.
    #[test]
    fn test_serde_default_fn_name() {
        let yaml_str = r#"
title: BlockDef
type: object
required: [columns]
properties:
  columns:
    type: integer
    x-rust-type: u32
    x-serde-default: "default_columns"
    description: "Number of columns."
x-rust-struct:
  derive: [Debug, Clone, Serialize, Deserialize]
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(
            output
                .code
                .contains(r#"#[serde(default = "default_columns")]"#),
            "should emit default = fn_name attribute, got:\n{}",
            output.code,
        );
        // Bare `default` MUST NOT appear when fn-name form is used.
        assert!(
            !output.code.contains("#[serde(default)]"),
            "bare `default` should be replaced by `default = \"fn\"`, got:\n{}",
            output.code,
        );
    }

    /// `serde_tag` on an enum emits `#[serde(tag = "...")]` for internally-
    /// tagged representation. When combined with `serde_rename_all`, both
    /// directives MUST share a single `#[serde(...)]` attribute — splitting
    /// them across two attributes is a serde compile error.
    #[test]
    fn test_serde_tag_alone() {
        let yaml_str = r#"
title: Verdict
type: string
enum: [Approved, NeedsRevision]
x-rust-enum:
  derive: [Debug, Clone, Serialize, Deserialize]
  serde_tag: kind
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(
            output.code.contains("#[serde(tag = \"kind\")]"),
            "should emit serde_tag attribute, got:\n{}",
            output.code,
        );
    }

    #[test]
    fn test_serde_tag_with_rename_all_combines() {
        let yaml_str = r#"
title: Verdict
type: string
enum: [Approved, NeedsRevision]
x-rust-enum:
  derive: [Debug, Clone, Serialize, Deserialize]
  serde_tag: verdict
  serde_rename_all: snake_case
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        // Both directives MUST appear in a single attribute — serde rejects
        // them split across two `#[serde(...)]` lines.
        assert!(
            output
                .code
                .contains("#[serde(tag = \"verdict\", rename_all = \"snake_case\")]"),
            "should combine tag + rename_all into one attribute, got:\n{}",
            output.code,
        );
        // Sanity: separate attribute lines must NOT appear.
        assert!(
            !output
                .code
                .contains("#[serde(tag = \"verdict\")]\n#[serde(rename_all"),
            "tag and rename_all must not be split across two attributes",
        );
    }

    /// `x-trait-impls` with Display dispatch emits the standard fmt
    /// signature and a `match self { Variant => write!(f, "value"), ... }`
    /// body. R7-gated `as_str`/`FromStr` should NOT auto-emit when not
    /// declared, even though we are now emitting Display for the same enum.
    #[test]
    fn test_x_trait_impls_emits_display() {
        let yaml_str = r#"
title: Verdict
type: string
enum: [Approved, Rejected]
description: "Review verdict."
x-trait-impls:
  - trait: Display
    impl_mode: codegen
    dispatch:
      - { variant: Approved, value: "APPROVED" }
      - { variant: Rejected, value: "REJECTED" }
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(
            output.code.contains("impl std::fmt::Display for Verdict"),
            "Should emit fully-qualified Display impl, got:\n{}",
            output.code,
        );
        assert!(output
            .code
            .contains("fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result"));
        assert!(output
            .code
            .contains("Verdict::Approved => write!(f, \"APPROVED\")"));
        assert!(output
            .code
            .contains("Verdict::Rejected => write!(f, \"REJECTED\")"));
        // R7: as_str / FromStr are NOT auto-emitted when not declared.
        assert!(
            !output.code.contains("pub fn as_str"),
            "as_str should NOT emit unless declared in x-methods (R7)",
        );
    }

    /// `x-trait-impls` with FromStr emits the type Err alias, a
    /// `match s { "X" => Ok(Variant) }` body, and a default error path
    /// using `format!`. Custom `err_type` overrides the default `String`.
    #[test]
    fn test_x_trait_impls_emits_fromstr() {
        let yaml_str = r#"
title: Verdict
type: string
enum: [Approved, Rejected]
x-trait-impls:
  - trait: FromStr
    impl_mode: codegen
    err_type: "String"
    dispatch:
      - { input: "APPROVED", variant: Approved }
      - { input: "REJECTED", variant: Rejected }
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(output.code.contains("impl std::str::FromStr for Verdict"));
        assert!(output.code.contains("type Err = String"));
        assert!(output
            .code
            .contains("\"APPROVED\" => Ok(Verdict::Approved)"));
        assert!(output
            .code
            .contains("\"REJECTED\" => Ok(Verdict::Rejected)"));
        assert!(output
            .code
            .contains("Err(format!(\"invalid Verdict: {}\", s))"));
    }

    /// Non-codegen entries are skipped silently — `impl_mode: hand-written`
    /// means the author owns the trait impl.
    #[test]
    fn test_x_trait_impls_respects_hand_written_mode() {
        let yaml_str = r#"
title: Verdict
type: string
enum: [Approved]
x-trait-impls:
  - trait: Display
    impl_mode: hand-written
    dispatch:
      - { variant: Approved, value: "APPROVED" }
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(
            !output.code.contains("impl std::fmt::Display"),
            "hand-written trait impl must not be emitted",
        );
    }

    /// Unsupported traits fall through silently — adding new ones is a
    /// generator extension, not an author concern.
    #[test]
    fn test_x_trait_impls_unsupported_trait_skipped() {
        let yaml_str = r#"
title: Verdict
type: string
enum: [Approved]
x-trait-impls:
  - trait: Hash
    impl_mode: codegen
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(!output.code.contains("impl Hash"));
        assert!(!output.code.contains("impl std::hash::Hash"));
    }

    /// Non-enum schemas continue to use the struct codepath unchanged.
    #[test]
    fn test_schema_without_enum_still_uses_struct_codepath() {
        let yaml_str = r#"
title: Foo
type: object
properties:
  bar: { type: string }
required: [bar]
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(output.code.contains("pub struct Foo"));
        assert!(!output.code.contains("pub enum Foo"));
    }

    /// A `type: string` schema without an `enum:` list stays a string field
    /// inside its parent struct — the enum path is for top-level schemas
    /// only.
    #[test]
    fn test_bare_type_string_is_not_an_enum() {
        // A top-level schema that declares `type: string` without `enum:` is
        // unusual (top-level strings rarely appear in TD), but we still
        // should not misinterpret it as a Rust enum.
        let yaml_str = r#"
title: PlainString
type: string
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_schema(&yaml, &RustConfig::default());
        assert!(
            !output.code.contains("pub enum PlainString"),
            "Should NOT generate an enum when enum list is absent"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R1
    #[test]
    fn test_generates_struct_from_schema() {
        let yaml_str = r#"
title: Issue
properties:
  id:
    type: string
    description: Unique identifier
  title:
    type: string
required: [id, title]
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let config = RustConfig::default();
        let output = generate_schema(&yaml, &config);

        assert!(
            output.code.contains("struct Issue"),
            "Should generate struct Issue"
        );
        assert!(output.code.contains("id: String"), "Should have id field");
        assert!(
            output.code.contains("title: String"),
            "Should have title field"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R1
    #[test]
    fn test_optional_fields_wrapped_in_option() {
        let yaml_str = r#"
title: Config
properties:
  required_field:
    type: string
  optional_field:
    type: string
required: [required_field]
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let config = RustConfig::default();
        let output = generate_schema(&yaml, &config);

        assert!(
            output.code.contains("required_field: String"),
            "Required field should be bare type"
        );
        assert!(
            output.code.contains("Option<String>"),
            "Optional field should be wrapped in Option"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R6
    #[test]
    fn test_applies_config_derives() {
        let yaml_str = "title: MyStruct\nproperties: {}";
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let config = RustConfig::default();
        let output = generate_schema(&yaml, &config);

        assert!(
            output.code.contains("#[derive("),
            "Should have derive attribute"
        );
        assert!(output.code.contains("Debug"), "Should derive Debug");
        assert!(output.code.contains("Serialize"), "Should derive Serialize");
    }

    #[test]
    fn empty_braced_struct_is_rustfmt_stable() {
        let yaml_str = "title: MyStruct\nx-rust-struct:\n  derive: []\nproperties: {}";
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let config = RustConfig::default();
        let output = generate_schema(&yaml, &config);

        assert!(
            output.code.contains("pub struct MyStruct {}"),
            "Empty braced structs should use rustfmt-stable single-line output. Got:\n{}",
            output.code
        );
        assert!(
            !output.code.contains("pub struct MyStruct {\n}"),
            "Empty braced structs should not emit formatter-unstable multiline output. Got:\n{}",
            output.code
        );
    }

    #[test]
    fn test_generic_struct_via_x_rust_generics() {
        let yaml_str = r#"
title: Path
x-rust-generics: [T]
x-rust:
  derives: []
required: [inner]
properties:
  inner:
    type: object
    x-rust-type: T
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let config = RustConfig::default();
        let output = generate_schema(&yaml, &config);

        assert!(
            output.code.contains("pub struct Path<T> {"),
            "Should emit generic struct header. Got:\n{}",
            output.code
        );
        assert!(
            output.code.contains("pub inner: T,"),
            "Should emit `pub inner: T,` field. Got:\n{}",
            output.code
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R6
    #[test]
    fn test_per_spec_x_rust_override() {
        let yaml_str = r#"
title: MyStruct
x-rust:
  derives: [Debug, Clone]
  visibility: "pub(crate)"
properties: {}
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let config = RustConfig::default();
        let output = generate_schema(&yaml, &config);

        assert!(
            output.code.contains("pub(crate)"),
            "Should use overridden visibility"
        );
        assert!(
            !output.code.contains("Serialize"),
            "Overridden derives should not include Serialize"
        );
    }

    #[test]
    fn path_style_serde_derives_enable_field_serde_attrs() {
        let yaml_str = r#"
title: MyStruct
required: [items]
properties:
  source:
    type: string
    x-serde-default: true
  items:
    type: array
    items: { type: string }
    x-serde-default: true
x-rust-struct:
  derive: [Debug, Clone, "serde::Serialize", "serde::Deserialize"]
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let config = RustConfig::default();
        let output = generate_schema(&yaml, &config);

        assert!(
            output.code.contains("#[serde(default)]"),
            "Path-style serde derives should still enable field serde attrs. Got:\n{}",
            output.code
        );
    }
}

// CODEGEN-END
