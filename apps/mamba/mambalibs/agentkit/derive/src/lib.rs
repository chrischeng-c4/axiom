//! `#[derive(AgentSchema)]` proc-macro for the pydantic-style typed-I/O
//! track in `agent`.
//!
//! Emits `impl <Struct> { pub fn schema() -> ::agent::Schema { ... } }`
//! describing the struct's fields. Field-type support, after #1953:
//!
//! - **Primitives** — `String`, `bool`, `f32`/`f64`, `i8..i64`/`u8..u64`,
//!   `isize`/`usize` — map to `Schema::{string, boolean, number, integer}`.
//! - **`Vec<T>`** — emits `Schema::array(<inner>)` recursively.
//! - **`Option<T>`** — emits `Schema::optional(<inner>)` AND drops the field
//!   from the `required` array.
//! - **Bare path types** (any other path with no generics, e.g.
//!   `struct Foo { addr: Address }`) — emits `<Type>::schema()`, so a
//!   nested user type that *also* derives `AgentSchema` composes
//!   naturally.
//! - **Field metadata** — `#[schema(description = "...", min_length = 2,
//!   max_length = 120, ge = 0, le = 100)]` maps to JSON Schema field
//!   metadata and validation constraints, matching common Pydantic
//!   `Field(...)` usage.
//!
//! Recursion: each `Vec<T>` / `Option<T>` is unwrapped one layer at a
//! time, so `Vec<Option<Vec<String>>>` is well-defined.
//!
//! Unsupported (compile-time error pointing at the offending field):
//! tuple fields, references, slices, function pointers, trait objects.
//! `HashMap<K,V>` / `BTreeMap<K,V>` and additional `#[schema(...)]`
//! customization stay tracked under follow-ups (P6+).
//!
//! @spec gh://chrischeng-c4/cclab/issues/1952
//! @spec gh://chrischeng-c4/cclab/issues/1953
//
// HANDWRITE-BEGIN reason: codegen has no proc-macro generator. Closing
// this gap requires (a) a section type "rust-proc-macro-derive" and
// (b) a generator that emits the `#[proc_macro_derive(...)] pub fn
// <fn_name>(input: TokenStream) -> TokenStream` boilerplate plus a
// per-field-type table (primitive | Vec<T> | Option<T> | bare-path).
// Until that lands, this file is hand-written.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Expr, ExprLit, Fields, GenericArgument,
    Ident, Lit, Meta, PathArguments, Type, TypePath,
};

#[proc_macro_derive(AgentSchema, attributes(schema))]
pub fn derive_agent_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(input) {
        Ok(ts) => ts.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn expand(input: DeriveInput) -> syn::Result<TokenStream2> {
    let struct_ident = input.ident.clone();

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => &named.named,
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(_),
            ..
        }) => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "AgentSchema can only be derived for structs with named fields (tuple structs are not supported)",
            ));
        }
        Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "AgentSchema can only be derived for structs with named fields (unit structs are not supported)",
            ));
        }
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "AgentSchema can only be derived for structs (enums are not supported in this slice — tracked under the P-track follow-ups)",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "AgentSchema can only be derived for structs (unions are not supported)",
            ));
        }
    };

    let mut field_inits = Vec::new();
    let mut required_names = Vec::new();
    for field in fields {
        let name = field.ident.as_ref().expect("named field has ident").clone();
        let name_str = name.to_string();
        let field_attrs = parse_schema_field_attrs(field)?;
        let (mut expr, is_required) = schema_expr_for(&field.ty)?;
        if let Some(description) = field_attrs.description {
            expr = quote! { #expr.description(#description) };
        }
        if let Some(min_length) = field_attrs.min_length {
            expr = quote! { #expr.min_length(#min_length) };
        }
        if let Some(max_length) = field_attrs.max_length {
            expr = quote! { #expr.max_length(#max_length) };
        }
        if let Some(minimum) = field_attrs.minimum {
            let minimum = proc_macro2::Literal::f64_unsuffixed(minimum);
            expr = quote! { #expr.minimum(#minimum) };
        }
        if let Some(maximum) = field_attrs.maximum {
            let maximum = proc_macro2::Literal::f64_unsuffixed(maximum);
            expr = quote! { #expr.maximum(#maximum) };
        }
        field_inits.push(quote! {
            .field(#name_str, #expr)
        });
        if is_required {
            required_names.push(quote! { #name_str });
        }
    }

    let out = quote! {
        impl #struct_ident {
            /// Auto-generated by `#[derive(AgentSchema)]`. Returns a runtime
            /// `agent::Schema` describing this struct's fields.
            pub fn schema() -> ::agent::Schema {
                ::agent::Schema::object()
                    #(#field_inits)*
                    .required(&[#(#required_names),*])
                    .build()
            }
        }
    };
    Ok(out)
}

#[derive(Default)]
struct FieldSchemaAttrs {
    description: Option<String>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    minimum: Option<f64>,
    maximum: Option<f64>,
}

fn parse_schema_field_attrs(field: &syn::Field) -> syn::Result<FieldSchemaAttrs> {
    let mut out = FieldSchemaAttrs::default();
    for attr in &field.attrs {
        if !attr.path().is_ident("schema") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            let key = meta
                .path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_default();
            match key.as_str() {
                "description" => out.description = Some(parse_schema_string_arg(&meta)?),
                "min_length" => out.min_length = Some(parse_schema_usize_arg(&meta)?),
                "max_length" => out.max_length = Some(parse_schema_usize_arg(&meta)?),
                "minimum" | "ge" => out.minimum = Some(parse_schema_f64_arg(&meta)?),
                "maximum" | "le" => out.maximum = Some(parse_schema_f64_arg(&meta)?),
                other => {
                    return Err(meta.error(format!(
                        "unsupported #[schema(...)] key `{other}` — expected `description`, `min_length`, `max_length`, `minimum`/`ge`, or `maximum`/`le`"
                    )));
                }
            }
            Ok(())
        })?;
    }
    Ok(out)
}

fn parse_schema_string_arg(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<String> {
    let value: Expr = meta.value()?.parse()?;
    match value {
        Expr::Lit(ExprLit {
            lit: Lit::Str(lit), ..
        }) => Ok(lit.value()),
        _ => Err(meta.error("#[schema(...)] `description` must be a string literal")),
    }
}

fn parse_schema_usize_arg(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<usize> {
    let value: Expr = meta.value()?.parse()?;
    match value {
        Expr::Lit(ExprLit {
            lit: Lit::Int(lit), ..
        }) => lit.base10_parse::<usize>(),
        _ => Err(meta.error("#[schema(...)] length constraints must be unsigned integer literals")),
    }
}

fn parse_schema_f64_arg(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<f64> {
    let value: Expr = meta.value()?.parse()?;
    match value {
        Expr::Lit(ExprLit {
            lit: Lit::Int(lit), ..
        }) => lit.base10_parse::<f64>(),
        Expr::Lit(ExprLit {
            lit: Lit::Float(lit),
            ..
        }) => lit.base10_parse::<f64>(),
        _ => Err(meta.error("#[schema(...)] numeric constraints must be numeric literals")),
    }
}

/// Build the per-field schema expression and decide whether the field
/// belongs in the parent object's `required` list.
///
/// `Option<T>` is the only shape that returns `is_required = false` —
/// every other supported type (primitive, `Vec<T>`, bare path) is
/// required by default, matching the P1 builder behaviour.
fn schema_expr_for(ty: &Type) -> syn::Result<(TokenStream2, bool)> {
    if let Some(inner) = extract_generic_inner(ty, "Option") {
        let inner_expr = inner_schema_expr(inner)?;
        return Ok((quote! { ::agent::Schema::optional(#inner_expr) }, false));
    }
    let expr = inner_schema_expr(ty)?;
    Ok((expr, true))
}

/// Build the schema expression for a non-`Option` type. Recursively
/// unwraps `Vec<T>`, looks up primitives, and falls back to
/// `<Type>::schema()` for nested user-defined types.
fn inner_schema_expr(ty: &Type) -> syn::Result<TokenStream2> {
    if let Some(inner) = extract_generic_inner(ty, "Vec") {
        let inner_expr = inner_schema_expr(inner)?;
        return Ok(quote! { ::agent::Schema::array(#inner_expr) });
    }
    // Nested Option<T> inside Vec<Option<T>> etc. is still meaningful.
    if let Some(inner) = extract_generic_inner(ty, "Option") {
        let inner_expr = inner_schema_expr(inner)?;
        return Ok(quote! { ::agent::Schema::optional(#inner_expr) });
    }
    if let Some(prim) = primitive_for(ty) {
        let prim_ident = Ident::new(prim, proc_macro2::Span::call_site());
        return Ok(quote! { ::agent::Schema::#prim_ident() });
    }
    // Bare path (no generic args other than potentially lifetimes) — assume
    // the user type also derives AgentSchema and call its inherent
    // `schema()` method.
    if let Type::Path(TypePath { qself: None, path }) = ty {
        // Reject path types that still have unsupported generic arguments
        // (e.g. `HashMap<K, V>`). Only Vec / Option were unwrapped above —
        // anything else with `<...>` is currently unsupported.
        if let Some(seg) = path.segments.last() {
            if !matches!(seg.arguments, PathArguments::None) {
                return Err(syn::Error::new_spanned(
                    ty,
                    "AgentSchema: generic field types other than `Vec<T>` / `Option<T>` are not supported in this slice (HashMap / BTreeMap / custom generics are tracked under P6+).",
                ));
            }
        }
        return Ok(quote! { <#path>::schema() });
    }
    Err(syn::Error::new_spanned(
        ty,
        "AgentSchema: unsupported field type. Supported: primitives (String, i*, u*, f32, f64, bool), `Vec<T>`, `Option<T>`, and named user-defined types that also derive `AgentSchema`. Unsupported: tuples, references, slices, function pointers, trait objects.",
    ))
}

/// If `ty` is a path whose last segment matches `wrapper` and carries
/// exactly one type argument, return that argument.
///
/// Used to peel `Vec<T>` and `Option<T>`. Matches on the last segment
/// only, so `std::option::Option<T>` works as well as bare `Option<T>`.
fn extract_generic_inner<'a>(ty: &'a Type, wrapper: &str) -> Option<&'a Type> {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return None;
    };
    let seg = path.segments.last()?;
    if seg.ident != wrapper {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    // Find the first type argument; ignore lifetimes/const generics.
    args.args.iter().find_map(|arg| match arg {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    })
}

/// Map a Rust type to the `Schema::<primitive>` constructor name.
///
/// Returns `None` if the type isn't one of the supported primitives —
/// callers fall through to the `Vec` / `Option` / bare-path arms.
fn primitive_for(ty: &Type) -> Option<&'static str> {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return None;
    };
    let seg = path.segments.last()?;
    // Primitives never carry generic arguments — reject `String<...>`-style
    // nonsense early so it falls through to the bare-path arm and produces
    // a clearer error.
    if !matches!(seg.arguments, PathArguments::None) {
        return None;
    }
    let ident = seg.ident.to_string();
    match ident.as_str() {
        "String" => Some("string"),
        "bool" => Some("boolean"),
        "f32" | "f64" => Some("number"),
        "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize" => {
            Some("integer")
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// `#[derive(AgentTool)]` (#2031) and `#[derive(AgentOutput)]` (#2032).
//
// Both layer on top of `AgentSchema` — the struct must also derive
// `AgentSchema` (and typically `serde::Deserialize`) so the JSON-Schema and
// the typed deserialization are both available.
//
// HANDWRITE rationale (same as the `AgentSchema` block above): codegen has
// no proc-macro generator. Until it does, the markers stay HANDWRITE and
// this file is the source of truth.
// ---------------------------------------------------------------------------

/// Extract `#[tool(name = "...", description = "...")]` from a struct's
/// outer attributes. Both fields are required.
fn parse_tool_attr(input: &DeriveInput) -> syn::Result<(String, String)> {
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("tool") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            let key = meta
                .path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_default();
            let value: Expr = meta.value()?.parse()?;
            let s = match value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(lit), ..
                }) => lit.value(),
                _ => {
                    return Err(meta.error(
                        "#[tool(...)] arguments must be string literals (name = \"...\", description = \"...\")",
                    ));
                }
            };
            match key.as_str() {
                "name" => name = Some(s),
                "description" => description = Some(s),
                other => {
                    return Err(meta.error(format!(
                        "unsupported #[tool(...)] key `{other}` — expected `name` or `description`"
                    )));
                }
            }
            Ok(())
        })?;
    }

    let name = name.ok_or_else(|| {
        syn::Error::new_spanned(
            &input.ident,
            "AgentTool: missing `#[tool(name = \"...\", description = \"...\")]` attribute on the struct",
        )
    })?;
    let description = description.ok_or_else(|| {
        syn::Error::new_spanned(
            &input.ident,
            "AgentTool: missing `description = \"...\"` in `#[tool(...)]`",
        )
    })?;
    Ok((name, description))
}

/// `#[derive(AgentTool)]` — emits typed-tool metadata + an `into_tool_spec`
/// helper that wraps a strongly-typed handler `Fn(Deps, Self) -> Future` into
/// an `agent::ToolSpec<Deps>` whose JSON args are deserialized into `Self`
/// at dispatch time. The struct must also derive `AgentSchema` (for the
/// parameter schema) and `serde::Deserialize` (for arg parsing).
#[proc_macro_derive(AgentTool, attributes(tool))]
pub fn derive_agent_tool(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_agent_tool(input) {
        Ok(ts) => ts.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn expand_agent_tool(input: DeriveInput) -> syn::Result<TokenStream2> {
    // Reject non-named-struct shapes so the error message is friendly.
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(_),
            ..
        }) => {}
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "AgentTool can only be derived for structs with named fields",
            ));
        }
    }
    let (tool_name, tool_description) = parse_tool_attr(&input)?;
    let struct_ident = input.ident;

    let out = quote! {
        impl #struct_ident {
            /// Tool name advertised to the LLM. Source: `#[tool(name = "...")]`.
            pub const TOOL_NAME: &'static str = #tool_name;
            /// Tool description advertised to the LLM. Source: `#[tool(description = "...")]`.
            pub const TOOL_DESCRIPTION: &'static str = #tool_description;

            /// JSON-Schema (draft-07) for this tool's parameters. Sourced
            /// from the companion `#[derive(AgentSchema)]`.
            pub fn tool_parameters_json() -> ::serde_json::Value {
                <Self>::schema().to_json_schema()
            }

            /// Convert a typed handler `Fn(Deps, Self) -> Future` into a
            /// `ToolSpec<Deps>` that the agent runtime can dispatch. The
            /// handler's `Self` argument is parsed from the LLM-supplied
            /// JSON; its return value is serialized back to JSON.
            pub fn into_tool_spec<Deps, F, Fut, R>(
                handler: F,
            ) -> ::agent::ToolSpec<Deps>
            where
                Deps: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
                F: ::std::ops::Fn(Deps, Self) -> Fut
                    + ::std::marker::Send
                    + ::std::marker::Sync
                    + 'static,
                Fut: ::std::future::Future<Output = ::agent::NovaResult<R>>
                    + ::std::marker::Send
                    + 'static,
                R: ::serde::Serialize + ::std::marker::Send + 'static,
                Self: ::serde::de::DeserializeOwned + ::std::marker::Send + 'static,
            {
                let handler = ::std::sync::Arc::new(handler);
                ::agent::ToolSpec::new(
                    Self::TOOL_NAME,
                    Self::TOOL_DESCRIPTION,
                    Self::tool_parameters_json(),
                    move |deps: Deps, args: ::serde_json::Value| {
                        let handler = handler.clone();
                        async move {
                            let parsed: Self = ::serde_json::from_value(args)
                                .map_err(|e| ::agent::NovaError::InvalidArguments(
                                    ::std::format!("{}: {}", Self::TOOL_NAME, e)
                                ))?;
                            let result = handler(deps, parsed).await?;
                            ::serde_json::to_value(result)
                                .map_err(::agent::NovaError::SerializationError)
                        }
                    },
                )
            }
        }
    };
    Ok(out)
}

/// `#[derive(AgentOutput)]` — emits a `parse_response(raw)` helper that
/// validates and deserializes the LLM's final JSON reply against the
/// schema produced by `#[derive(AgentSchema)]`. The struct must also
/// derive `AgentSchema` and `serde::Deserialize`.
#[proc_macro_derive(AgentOutput)]
pub fn derive_agent_output(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_agent_output(input) {
        Ok(ts) => ts.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn expand_agent_output(input: DeriveInput) -> syn::Result<TokenStream2> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(_),
            ..
        }) => {}
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "AgentOutput can only be derived for structs with named fields",
            ));
        }
    }
    // Reject `#[tool(...)]` on an output type — likely a mix-up between the two
    // derives. Silent acceptance would let stale attributes drift.
    for attr in &input.attrs {
        if attr.path().is_ident("tool") {
            return Err(syn::Error::new_spanned(
                attr.path(),
                "AgentOutput: `#[tool(...)]` does not apply to output types (did you mean #[derive(AgentTool)]?)",
            ));
        }
    }
    let _ = Meta::List; // touch `Meta` to keep the import live across rustc versions

    let struct_ident = input.ident;

    let out = quote! {
        impl #struct_ident {
            /// Same as `Self::schema()` — repeated here so callers can use a
            /// single name (`output_schema`) regardless of whether the type is
            /// being used as a tool param or an agent output.
            pub fn output_schema() -> ::agent::Schema {
                <Self>::schema()
            }

            /// Parse + validate a raw JSON string against this type's
            /// schema. Returns `NovaError::MalformedLLMResponse` for
            /// non-JSON input, `NovaError::ValidationFailed` for shape
            /// mismatches or deserialization failures.
            pub fn parse_response(raw: &str) -> ::agent::NovaResult<Self>
            where
                Self: ::serde::de::DeserializeOwned,
            {
                if raw.trim().is_empty() {
                    return ::std::result::Result::Err(
                        ::agent::NovaError::MalformedLLMResponse(
                            "empty final reply".to_string(),
                        ),
                    );
                }
                let value: ::serde_json::Value = ::serde_json::from_str(raw)
                    .map_err(|e| ::agent::NovaError::MalformedLLMResponse(
                        ::std::format!("final reply is not valid JSON: {}", e),
                    ))?;
                <Self>::schema()
                    .validate(&value)
                    .map_err(|e| ::agent::NovaError::ValidationFailed(e.to_string()))?;
                ::serde_json::from_value(value)
                    .map_err(|e| ::agent::NovaError::ValidationFailed(
                        ::std::format!("structured output deserialization failed: {}", e),
                    ))
            }
        }
    };
    Ok(out)
}

// HANDWRITE-END
