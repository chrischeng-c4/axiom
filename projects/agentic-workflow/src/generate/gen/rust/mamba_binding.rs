// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/gen/rust/mamba_binding.md#source
// CODEGEN-BEGIN
//! Mamba FFI binding generator.
//!
//! Reads `x-mamba-binding` + `x-constructor` annotations from a schema YAML
//! and emits the full Rust binding surface — constructor `impl`, `extern "C"`
//! FFI shim, and a `register()` function that hooks into `ModuleRegistrar` via
//! `rt_sym!`. Generated code compiles against `cclab_mamba_registry`.
//!
//! Templates live under `templates/mamba_binding/*.tera` (embedded via
//! `include_str!`) and are rendered with the shared
//! [`crate::generate::engine::TemplateEngine`] (Tera). The generator builds a
//! JSON context, the templates own the Rust code layout.
//!
//! The named-helper concept (`x-mamba-binding.helpers: [http_status_phrase]`)
//! has been retired: mamba PR-4 ships a real `cclab_mamba_registry::http`
//! module with `status_phrase` / `status_name` / `canonical_codes`, which the
//! spec's `default_expr` can call directly.

use serde::Serialize;
use serde_yaml::Value;

use crate::generate::engine::TemplateEngine;

// ── embedded templates ──────────────────────────────────────────────────────

const TPL_CONSTRUCTOR: &str = include_str!("templates/mamba_binding/constructor.tera");
const TPL_EXTERN_SHIM: &str = include_str!("templates/mamba_binding/extern_shim.tera");
const TPL_REGISTER: &str = include_str!("templates/mamba_binding/register.tera");
const TPL_ATTRIBUTE_GETTER: &str = include_str!("templates/mamba_binding/attribute_getter.tera");

// ── public surface ──────────────────────────────────────────────────────────

/// Output from mamba-binding code generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/mamba_binding.md#schema
#[derive(Debug, Clone)]
pub struct MambaBindingGenOutput {
    /// Appended to the end of the schema-generated struct file.
    pub code: String,
    /// Whether binding code was emitted (i.e. x-mamba-binding was present).
    pub emitted: bool,
}
/// Entry point — returns `emitted: false` when `x-mamba-binding` is absent so
/// the caller can skip appending anything.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/mamba_binding.md#source
pub fn generate_mamba_binding(schema_yaml: &Value, struct_name: &str) -> MambaBindingGenOutput {
    generate_mamba_binding_with_provenance(schema_yaml, struct_name, "")
}

/// Same as [`generate_mamba_binding`] but embeds `@spec <spec_path>#<anchor>`
/// provenance markers on each generated item. Pass `""` for `spec_path` to
/// suppress markers.
pub fn generate_mamba_binding_with_provenance(
    schema_yaml: &Value,
    struct_name: &str,
    spec_path: &str,
) -> MambaBindingGenOutput {
    let binding = match schema_yaml.get("x-mamba-binding") {
        Some(v) if v.is_mapping() => v,
        _ => {
            return MambaBindingGenOutput {
                code: String::new(),
                emitted: false,
            }
        }
    };
    let ctor = schema_yaml.get("x-constructor").filter(|v| v.is_mapping());

    let ctx = BindingContext::build(schema_yaml, binding, ctor, struct_name, spec_path);

    let engine = template_engine();
    let constructor = engine
        .render("constructor.tera", &ctx)
        .expect("constructor.tera render (programmer error if this fails)");
    let extern_shim = engine
        .render("extern_shim.tera", &ctx)
        .expect("extern_shim.tera render");
    let register = engine
        .render("register.tera", &ctx)
        .expect("register.tera render");

    let mut blocks: Vec<String> = vec![trim_trailing(&constructor), trim_trailing(&extern_shim)];
    if !ctx.attributes.is_empty() {
        let attr_block = engine
            .render("attribute_getter.tera", &ctx)
            .expect("attribute_getter.tera render");
        blocks.push(trim_trailing(&attr_block));
    }
    blocks.push(trim_trailing(&register));

    MambaBindingGenOutput {
        code: blocks.join("\n\n") + "\n",
        emitted: true,
    }
}

fn trim_trailing(s: &str) -> String {
    s.trim_end().to_string()
}

fn template_engine() -> TemplateEngine {
    let mut engine = TemplateEngine::empty();
    engine
        .add_template("constructor.tera", TPL_CONSTRUCTOR)
        .expect("constructor.tera parse");
    engine
        .add_template("extern_shim.tera", TPL_EXTERN_SHIM)
        .expect("extern_shim.tera parse");
    engine
        .add_template("register.tera", TPL_REGISTER)
        .expect("register.tera parse");
    engine
        .add_template("attribute_getter.tera", TPL_ATTRIBUTE_GETTER)
        .expect("attribute_getter.tera parse");
    engine
}

// ── template context ────────────────────────────────────────────────────────

/// Shape handed to the Tera templates. Must stay in sync with the `{{ ... }}`
/// references in `templates/mamba_binding/*.tera`.
#[derive(Debug, Serialize)]
struct BindingContext {
    struct_name: String,
    /// `struct_name` in snake_case (via `heck`), used to disambiguate per-type
    /// `register_<snake>` fn names when a single spec emits multiple types.
    struct_snake: String,
    extern_fn: String,
    symbol: String,
    signature: String,
    args: Vec<ArgContext>,
    validations: Vec<ValidationContext>,
    /// Spec-declared `x-mamba-attributes`. Empty when the spec doesn't declare
    /// any; the register.tera template skips the attribute-registration block
    /// when the list is empty.
    attributes: Vec<AttributeContext>,
    /// True when at least one ctor arg has a primitive reader (int/str/bool/
    /// float/enum). Drives whether the extern shim imports `FromMbValue` +
    /// declares the `read` closure. Ctors whose args all fall through to
    /// literal defaults skip both to keep the output warning-free.
    has_any_primitive_reader: bool,
    /// Relative path to the authoring spec (e.g.
    /// `.aw/tech-design/projects/httpkit/http-exception.md`) used in `@spec`
    /// provenance markers. Empty string suppresses marker emission — useful
    /// in unit tests that drive the generator directly.
    spec_path: String,
}

#[derive(Debug, Serialize)]
struct AttributeContext {
    name: String,
    /// Fully-qualified Rust identifier for the generated extern "C" getter fn.
    /// Derived as `<snake_struct_name>_get_<name>` when the spec omits an
    /// explicit override.
    getter_fn: String,
    /// Rust expression producing the attribute value. Defaults to
    /// `self_.<name>.clone()` when the spec omits `rust_expr`.
    rust_expr: String,
    /// Optional per-attribute docstring sourced from the spec. Rendered as
    /// `/// <doc>` on the getter fn when non-empty.
    doc: String,
    /// Reserved for future setter support. Today always `true`; the generator
    /// skips setter emission regardless until that path ships.
    readonly: bool,
}

#[derive(Debug, Serialize)]
struct ArgContext {
    name: String,
    mb_type: String,
    rust_type: String,
    /// Type of the constructor parameter (may wrap `rust_type` in `Option<..>`).
    param_type: String,
    /// `rust_type` with the outer `Option<..>` stripped — used by the FFI shim
    /// to declare the inner type when the reader produces `Option<T>`.
    stripped_rust_type: String,
    nullable: bool,
    /// Whether the FFI shim has a primitive `FromMbValue` reader for this
    /// `mb_type`. Non-primitive (`dict`, `list`, composite) fall back to the
    /// literal `default` with a `TODO` marker.
    has_primitive_reader: bool,
    /// Rust expression producing the primitive reader result (only meaningful
    /// when `has_primitive_reader` is true).
    reader_expr: String,
    /// The fallback expression used when the primitive reader returns `None`
    /// or there is no reader at all.
    default_fallback: String,
    /// Literal default from `x-constructor.args[].default` (if any).
    default: Option<String>,
    /// Computed default expression from `x-constructor.args[].default_expr`.
    default_expr: Option<String>,
    /// True when the constructor arg is plain `T` but the schema-generated
    /// struct field is `Option<T>` — the `Self { .. }` init must wrap in
    /// `Some(..)`.
    wrap_some_in_init: bool,
}

#[derive(Debug, Serialize)]
struct ValidationContext {
    /// Raw Rust boolean expression — true when the field passes the rule.
    /// Constructor returns `Err` when this evaluates to `false`.
    check_expr: String,
    /// Human-readable failure message. Escaped for safe inclusion inside
    /// `format!("...")`.
    message: String,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/mamba_binding.md#source
impl BindingContext {
    fn build(
        schema_yaml: &Value,
        binding: &Value,
        ctor: Option<&Value>,
        struct_name: &str,
        spec_path: &str,
    ) -> Self {
        let extern_fn = binding
            .get("extern_fn")
            .and_then(|v| v.as_str())
            .unwrap_or("mb_binding_new")
            .to_string();
        let symbol = binding
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or(struct_name)
            .to_string();
        let signature = binding
            .get("signature")
            .and_then(|v| v.as_str())
            .unwrap_or(struct_name)
            .to_string();
        let field_nullable = parse_field_nullability(schema_yaml);
        let args = parse_args(ctor, &field_nullable);
        let has_any_primitive_reader = args.iter().any(|a| a.has_primitive_reader);
        let validations = parse_validations(ctor);
        let attributes = parse_attributes(schema_yaml, struct_name);

        Self {
            struct_name: struct_name.to_string(),
            struct_snake: to_snake_case(struct_name),
            extern_fn,
            symbol,
            signature,
            args,
            validations,
            attributes,
            has_any_primitive_reader,
            spec_path: spec_path.to_string(),
        }
    }
}

// ── attribute parsing ───────────────────────────────────────────────────────

/// Parse `x-mamba-attributes` into `AttributeContext`s ready for the templates.
///
/// Defaults:
/// - `rust_expr` omitted → `self_.<name>.clone()` (covers the common
///   "read field, clone if non-Copy" case)
/// - `getter_fn` never overridden by the spec; derived as
///   `<snake_struct_name>_get_<name>` for determinism
/// - `doc` omitted → empty string; template falls back to a generic stub doc
/// - `readonly` defaults to `true`; setter emission is TBD after mamba PR-5
fn parse_attributes(schema_yaml: &Value, struct_name: &str) -> Vec<AttributeContext> {
    let Some(seq) = schema_yaml
        .get("x-mamba-attributes")
        .and_then(|v| v.as_sequence())
    else {
        return Vec::new();
    };
    let struct_snake = to_snake_case(struct_name);
    seq.iter()
        .filter_map(|entry| {
            let m = entry.as_mapping()?;
            let name = m.get("name").and_then(|v| v.as_str())?.to_string();
            let rust_expr = m
                .get("rust_expr")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| format!("self_.{}.clone()", name));
            let doc = m
                .get("doc")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let readonly = m.get("readonly").and_then(|v| v.as_bool()).unwrap_or(true);
            let getter_fn = format!("{}_get_{}", struct_snake, name);
            Some(AttributeContext {
                name,
                getter_fn,
                rust_expr,
                doc,
                readonly,
            })
        })
        .collect()
}

/// `HTTPException` → `http_exception` (acronym-aware snake_case via `heck`).
fn to_snake_case(s: &str) -> String {
    use heck::ToSnakeCase;
    s.to_snake_case()
}

// ── field-nullability helper ────────────────────────────────────────────────

/// For each schema property, compute whether the corresponding struct field
/// is wrapped in `Option<...>` by the schema generator. Mirrors the rule in
/// `schema.rs::infer_rust_type_with_nullable`: the field is Option iff it is
/// not in `required` OR its JSON-schema `type` list contains `"null"`.
fn parse_field_nullability(schema_yaml: &Value) -> std::collections::HashMap<String, bool> {
    use std::collections::HashMap;
    let mut out: HashMap<String, bool> = HashMap::new();
    let required: Vec<String> = schema_yaml
        .get("required")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let Some(props) = schema_yaml.get("properties").and_then(|v| v.as_mapping()) else {
        return out;
    };
    for (key, prop) in props {
        let Some(name) = key.as_str() else { continue };
        let type_says_null = match prop.get("type") {
            Some(Value::Sequence(seq)) => seq.iter().any(|t| t.as_str() == Some("null")),
            _ => false,
        };
        let wrapped_in_option = !required.contains(&name.to_string()) || type_says_null;
        out.insert(name.to_string(), wrapped_in_option);
    }
    out
}

// ── arg + validation parsing ────────────────────────────────────────────────

fn parse_args(
    ctor: Option<&Value>,
    field_nullable: &std::collections::HashMap<String, bool>,
) -> Vec<ArgContext> {
    let Some(ctor) = ctor else { return Vec::new() };
    let Some(seq) = ctor.get("args").and_then(|v| v.as_sequence()) else {
        return Vec::new();
    };
    seq.iter()
        .enumerate()
        .filter_map(|(idx, entry)| {
            let m = entry.as_mapping()?;
            let name = m.get("name").and_then(|v| v.as_str())?.to_string();
            let mb_type = m
                .get("mb_type")
                .and_then(|v| v.as_str())
                .unwrap_or("any")
                .to_string();
            let rust_type = m
                .get("rust_type")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| mb_to_rust_default(&mb_type));
            let nullable = m.get("nullable").and_then(|v| v.as_bool()).unwrap_or(false);
            let default = m.get("default").map(yaml_literal);
            let default_expr = m
                .get("default_expr")
                .and_then(|v| v.as_str())
                .map(String::from);

            let param_type = if nullable && !rust_type.starts_with("Option<") {
                format!("Option<{}>", rust_type)
            } else {
                rust_type.clone()
            };
            let stripped_rust_type = strip_option(&rust_type).to_string();

            let default_fallback = match (&default_expr, &default, nullable) {
                (Some(_), _, true) => "None".to_string(),
                (Some(expr), _, false) => expr.clone(),
                (_, Some(lit), _) => lit.clone(),
                (None, None, true) => "None".to_string(),
                (None, None, false) => "Default::default()".to_string(),
            };

            let (has_primitive_reader, reader_expr) =
                primitive_reader_expr(idx, &mb_type, &rust_type);

            let field_is_option = field_nullable.get(&name).copied().unwrap_or(false);
            let arg_is_option = nullable || rust_type.starts_with("Option<");
            let wrap_some_in_init = field_is_option && !arg_is_option;

            Some(ArgContext {
                name,
                mb_type,
                rust_type,
                param_type,
                stripped_rust_type,
                nullable,
                has_primitive_reader,
                reader_expr,
                default_fallback,
                default,
                default_expr,
                wrap_some_in_init,
            })
        })
        .collect()
}

/// Produce the Rust expression that reads arg `idx` into an `Option<T>` when a
/// primitive `FromMbValue` impl exists. Dict (`HashMap<String, V>`) and list
/// (`Vec<T>`) are now backed by mamba PR-2's ops-table-driven conversion —
/// their readers use `<T>::from_mb_value(..).ok()` directly on the declared
/// `rust_type`. Composite types with no primitive reader return
/// `(false, String::new())` so the template emits a TODO + falls back to the
/// literal `default`.
fn primitive_reader_expr(idx: usize, mb_type: &str, rust_type: &str) -> (bool, String) {
    match mb_type {
        "int" => {
            let cast = if rust_type != "i64" {
                format!(" as {}", rust_type)
            } else {
                String::new()
            };
            (
                true,
                format!(
                    "i64::from_mb_value(read({idx})).ok().map(|v| v{cast})",
                    idx = idx,
                    cast = cast,
                ),
            )
        }
        "str" => (true, format!("String::from_mb_value(read({})).ok()", idx)),
        "enum" => {
            // Read the mamba arg as a string then parse into the Rust enum via
            // `FromStr`. The schema-generated enum provides `FromStr` with
            // `Err = String`, so `.ok()` collapses to `Option<EnumType>`.
            (
                true,
                format!(
                    "String::from_mb_value(read({idx})).ok().and_then(|s| s.parse::<{ty}>().ok())",
                    idx = idx,
                    ty = rust_type,
                ),
            )
        }
        "bool" => (true, format!("bool::from_mb_value(read({})).ok()", idx)),
        "float" => (true, format!("f64::from_mb_value(read({})).ok()", idx)),
        // Dict / list can read through mamba PR-2's ops-table impls, but
        // only when the element type itself implements `FromMbValue`. The
        // registry ships impls for primitives (String, i64/i32/u32/usize,
        // bool, f64/f32). Anything else — `Vec<Cookie>`, `Vec<u8>`,
        // `HashMap<String, SomeStruct>` — has no `FromMbValue` impl, so
        // we fall back to the TODO+default path.
        "dict" | "list" if has_primitive_from_mb_value(rust_type) => (
            true,
            format!(
                "<{ty}>::from_mb_value(read({idx})).ok()",
                ty = rust_type,
                idx = idx
            ),
        ),
        _ => (false, String::new()),
    }
}

/// Heuristic — does the given concrete Rust type parse as
/// `Vec<P>` / `HashMap<String, P>` / `P` where `P` is a primitive with a
/// mamba-registry `FromMbValue` impl? Element type is extracted by
/// pattern-matching the string — we don't have a real type resolver here.
fn has_primitive_from_mb_value(rust_type: &str) -> bool {
    // Strip `Vec<..>` or `HashMap<String, ..>` wrappers to find the leaf.
    let leaf = if let Some(inner) = rust_type
        .strip_prefix("Vec<")
        .and_then(|s| s.strip_suffix(">"))
    {
        inner.trim()
    } else if let Some(inner) = rust_type
        .strip_prefix("HashMap<String, ")
        .or_else(|| rust_type.strip_prefix("std::collections::HashMap<String, "))
        .and_then(|s| s.strip_suffix(">"))
    {
        inner.trim()
    } else {
        rust_type.trim()
    };
    // Handle fully-qualified `std::string::String` etc. by taking the last
    // path segment.
    let leaf_short = leaf.rsplit("::").next().unwrap_or(leaf);
    matches!(
        leaf_short,
        "String" | "i64" | "i32" | "u32" | "usize" | "bool" | "f64" | "f32"
    )
}

/// Parse `x-constructor.validations` into pre-rendered `check_expr` + message
/// pairs. Every supported rule shape compiles down to a Rust boolean
/// expression that is TRUE when the field is valid — the constructor emits
/// `if !(check_expr) { return Err(...) }`.
///
/// Supported rules:
///
/// - `range { field, min, max, message? }` — integer range check
///   (`(min..=max).contains(&(field as i64))`). Use for `u16 status_code`
///   or `i64 age`-style bounds.
/// - `min_length { field, min, message? }` — string / Vec length floor
///   (`field.len() >= min`). Use for required non-empty names.
/// - `max_length { field, max, message? }` — string / Vec length ceiling.
/// - `expr { expr, message? }` — escape hatch. `expr` is a raw Rust boolean
///   expression evaluated against the constructor's locals (arg names are
///   in scope). Use for one-off checks a general rule doesn't express —
///   e.g. `"email.contains('@')"`. Prefer a named rule when the check
///   recurs across specs.
fn parse_validations(ctor: Option<&Value>) -> Vec<ValidationContext> {
    let Some(ctor) = ctor else { return Vec::new() };
    let Some(seq) = ctor.get("validations").and_then(|v| v.as_sequence()) else {
        return Vec::new();
    };
    seq.iter()
        .filter_map(|entry| {
            let m = entry.as_mapping()?;
            let rule = m.get("rule").and_then(|v| v.as_str())?;
            let custom_message = m.get("message").and_then(|v| v.as_str()).map(String::from);

            let (check_expr, default_message) = match rule {
                "range" => {
                    let field = m.get("field").and_then(|v| v.as_str())?;
                    let min = m.get("min").and_then(|v| v.as_i64())?;
                    let max = m.get("max").and_then(|v| v.as_i64())?;
                    (
                        format!("({}..={}).contains(&({} as i64))", min, max, field),
                        format!("{} out of range [{}, {}]", field, min, max),
                    )
                }
                "min_length" => {
                    let field = m.get("field").and_then(|v| v.as_str())?;
                    let min = m.get("min").and_then(|v| v.as_i64())?;
                    (
                        format!("{}.len() >= {}", field, min),
                        format!("{} must have at least {} chars", field, min),
                    )
                }
                "max_length" => {
                    let field = m.get("field").and_then(|v| v.as_str())?;
                    let max = m.get("max").and_then(|v| v.as_i64())?;
                    (
                        format!("{}.len() <= {}", field, max),
                        format!("{} must have at most {} chars", field, max),
                    )
                }
                "expr" => {
                    let expr = m.get("expr").and_then(|v| v.as_str())?;
                    (expr.to_string(), format!("validation failed: {}", expr))
                }
                _ => return None,
            };

            let message = custom_message
                .unwrap_or(default_message)
                .replace('"', "\\\"");
            Some(ValidationContext {
                check_expr,
                message,
            })
        })
        .collect()
}

fn yaml_literal(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "None".to_string(),
        _ => "Default::default()".to_string(),
    }
}

fn mb_to_rust_default(mb_type: &str) -> String {
    match mb_type {
        "int" => "i64".to_string(),
        "str" => "String".to_string(),
        "bool" => "bool".to_string(),
        "float" => "f64".to_string(),
        "dict" => "std::collections::HashMap<String, String>".to_string(),
        "list" => "Vec<String>".to_string(),
        _ => "cclab_mamba_registry::MbValue".to_string(),
    }
}

fn strip_option(ty: &str) -> &str {
    ty.strip_prefix("Option<")
        .and_then(|s| s.strip_suffix(">"))
        .unwrap_or(ty)
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Value {
        serde_yaml::from_str(s).unwrap()
    }

    #[test]
    fn no_binding_annotation_emits_nothing() {
        let schema = parse("title: Foo\ntype: object\nproperties: {}");
        let out = generate_mamba_binding(&schema, "Foo");
        assert!(!out.emitted);
        assert!(out.code.is_empty());
    }

    #[test]
    fn emits_constructor_with_range_validation() {
        let schema = parse(
            r#"
title: HTTPException
type: object
x-constructor:
  args:
    - { name: status_code, mb_type: int, rust_type: u16, default: 500 }
    - { name: detail, mb_type: str, rust_type: String, nullable: true,
        default_expr: "cclab_mamba_registry::http::status_phrase(status_code).to_string()" }
  validations:
    - { field: status_code, rule: range, min: 100, max: 599 }
x-mamba-binding:
  symbol: HTTPException
  extern_fn: http_exception_new
  signature: "HTTPException(status_code, detail)"
"#,
        );
        let out = generate_mamba_binding(&schema, "HTTPException");
        assert!(out.emitted);
        assert!(out.code.contains("impl HTTPException {"));
        assert!(out.code.contains("pub fn new("));
        assert!(out.code.contains("status_code: u16"));
        assert!(out.code.contains("detail: Option<String>"));
        assert!(out.code.contains("(100..=599).contains"));
        assert!(out.code.contains("detail.unwrap_or_else"));
    }

    #[test]
    fn validation_rules_render_length_and_expr_checks() {
        let schema = parse(
            r#"
title: UserPayload
type: object
x-constructor:
  args:
    - { name: name,  mb_type: str, rust_type: String }
    - { name: email, mb_type: str, rust_type: String }
    - { name: age,   mb_type: int, rust_type: i64, default: "0" }
  validations:
    - { field: name,  rule: min_length, min: 1 }
    - { field: name,  rule: max_length, max: 64 }
    - { field: email, rule: expr, expr: "email.contains('@')", message: "email must contain @" }
    - { field: age,   rule: range, min: 0, max: 150 }
x-mamba-binding:
  symbol: UserPayload
  extern_fn: user_payload_new
  signature: "UserPayload(name, email, age)"
"#,
        );
        let out = generate_mamba_binding(&schema, "UserPayload");
        // min_length rule renders as `.len() >= N`
        assert!(out.code.contains("name.len() >= 1"));
        // max_length rule renders as `.len() <= N`
        assert!(out.code.contains("name.len() <= 64"));
        // expr rule embeds the raw expression verbatim
        assert!(out.code.contains("email.contains('@')"));
        // Custom message flows through unchanged after quote escape
        assert!(out.code.contains("email must contain @"));
        // range rule still works alongside the new rules
        assert!(out.code.contains("(0..=150).contains(&(age as i64))"));
    }

    #[test]
    fn emits_extern_shim_with_arg_readers() {
        let schema = parse(
            r#"
title: HTTPException
type: object
x-constructor:
  args:
    - { name: status_code, mb_type: int, rust_type: u16, default: 500 }
x-mamba-binding:
  extern_fn: http_exception_new
  symbol: HTTPException
  signature: "HTTPException(status_code)"
"#,
        );
        let out = generate_mamba_binding(&schema, "HTTPException");
        assert!(out
            .code
            .contains("pub unsafe extern \"C\" fn http_exception_new"));
        assert!(out.code.contains("i64::from_mb_value(read(0))"));
        assert!(out.code.contains("HTTPException::new("));
        assert!(out.code.contains("mb_wrap_native_typed(\"HTTPException\""));
    }

    #[test]
    fn emits_register_fn() {
        let schema = parse(
            r#"
title: HTTPException
type: object
x-mamba-binding:
  extern_fn: http_exception_new
  symbol: HTTPException
  signature: "HTTPException(status_code: int)"
"#,
        );
        let out = generate_mamba_binding(&schema, "HTTPException");
        // Per-type register is `register_<snake>`; the aggregate `pub fn
        // register(r)` is emitted by apply.rs::try_generate_schema, not by
        // this generator (multi-type spec support).
        assert!(out.code.contains(
            "pub fn register_http_exception(r: &mut cclab_mamba_registry::ModuleRegistrar)"
        ));
        assert!(out
            .code
            .contains("rt_sym!(\"HTTPException\", http_exception_new"));
    }

    // [removed] `emits_http_status_phrase_helper_when_requested` — the
    // `helpers: [http_status_phrase]` path has been retired. Mamba PR-4 ships
    // `cclab_mamba_registry::http::status_phrase(u16) -> &'static str`, and
    // specs now call it directly via `default_expr` rather than asking the
    // generator to inline a phrase table.

    #[test]
    fn emits_attribute_getters_when_spec_declares_attributes() {
        let schema = parse(
            r#"
title: HTTPException
type: object
required: [status_code, detail]
properties:
  status_code: { type: integer, x-rust-type: u16 }
  detail:     { type: string }
x-mamba-attributes:
  - name: status_code
    rust_expr: "self_.status_code as i64"
    doc: "HTTP status code."
  - name: detail
    # rust_expr omitted → defaults to `self_.detail.clone()`
x-mamba-binding:
  symbol: HTTPException
  extern_fn: http_exception_new
  signature: "HTTPException(status_code, detail)"
"#,
        );
        let out = generate_mamba_binding(&schema, "HTTPException");
        assert!(out.emitted);
        // Both getter fns are emitted with the expected names.
        assert!(out
            .code
            .contains("pub unsafe extern \"C\" fn http_exception_get_status_code"));
        assert!(out
            .code
            .contains("pub unsafe extern \"C\" fn http_exception_get_detail"));
        // Explicit rust_expr is used verbatim.
        assert!(out.code.contains("self_.status_code as i64"));
        // Default rust_expr kicks in when omitted.
        assert!(out.code.contains("self_.detail.clone()"));
        // Doc is wired into a `///` comment.
        assert!(out.code.contains("/// HTTP status code."));
        // register() registers both getters into the ops table.
        assert!(out.code.contains("(o.register_getter)(\"HTTPException\", \"status_code\", http_exception_get_status_code)"));
        assert!(out.code.contains(
            "(o.register_getter)(\"HTTPException\", \"detail\", http_exception_get_detail)"
        ));
        // mb_unwrap_native_ref is used (borrow, not consume).
        assert!(out.code.contains("mb_unwrap_native_ref"));
    }

    #[test]
    fn no_attribute_block_when_spec_omits_x_mamba_attributes() {
        let schema = parse(
            r#"
title: Foo
type: object
x-mamba-binding:
  symbol: Foo
  extern_fn: foo_new
  signature: "Foo()"
"#,
        );
        let out = generate_mamba_binding(&schema, "Foo");
        assert!(!out.code.contains("register_getter"));
        assert!(!out.code.contains("mb_unwrap_native_ref"));
    }

    #[test]
    fn extern_shim_raises_value_error_via_ops_table() {
        let schema = parse(
            r#"
title: HTTPException
type: object
x-constructor:
  args:
    - { name: status_code, mb_type: int, rust_type: u16, default: 500 }
  validations:
    - { field: status_code, rule: range, min: 100, max: 599 }
x-mamba-binding:
  symbol: HTTPException
  extern_fn: http_exception_new
  signature: "HTTPException(status_code)"
"#,
        );
        let out = generate_mamba_binding(&schema, "HTTPException");
        // The `Err` branch now calls through the ops table. The
        // `OBJECT_OPS.get()` fallback keeps unit tests happy when the runtime
        // hasn't installed the table.
        assert!(out
            .code
            .contains("cclab_mamba_registry::ops::OBJECT_OPS.get()"));
        assert!(out.code.contains("(o.raise)(\"ValueError\", &msg)"));
    }

    #[test]
    fn wraps_some_when_arg_non_option_but_field_is_option() {
        // headers is Option<HashMap> on the struct (nullable via type: [object, null])
        // but the ctor arg is non-nullable HashMap with a literal default.
        let schema = parse(
            r#"
title: HTTPException
type: object
required: [status_code]
properties:
  status_code: { type: integer, x-rust-type: u16 }
  headers:
    type: [object, null]
    x-rust-type: "std::collections::HashMap<String, String>"
x-constructor:
  args:
    - { name: status_code, mb_type: int, rust_type: u16, default: 500 }
    - { name: headers, mb_type: dict, rust_type: "std::collections::HashMap<String, String>", default: "std::collections::HashMap::new()" }
x-mamba-binding:
  extern_fn: f
  symbol: S
  signature: "S()"
"#,
        );
        let out = generate_mamba_binding(&schema, "HTTPException");
        assert!(
            out.code.contains("headers: Some(headers),"),
            "Self init should wrap the non-Option arg with Some(..) to match the Option<..> struct field\n---\n{}",
            out.code,
        );
    }
}

// CODEGEN-END
