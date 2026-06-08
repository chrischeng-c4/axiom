//! JSON Schema builder API for typed I/O.
//!
//! Mirrors pydantic's `TypeAdapter` construction surface: callers describe a
//! JSON shape programmatically, then call [`Schema::to_json_schema`] to emit
//! draft-07 compatible output. Foundation primitive for the pydantic-style
//! typed I/O track — validator engine (#1950) and derive macro (#1952) layer
//! on top.
//!
//! ```
//! use agent::Schema;
//!
//! let s = Schema::object()
//!     .field("name", Schema::string())
//!     .field("age", Schema::integer())
//!     .required(&["name"])
//!     .build();
//! let json = s.to_json_schema();
//! assert_eq!(json["type"], "object");
//! ```
//!
//! Scope (deliberately small for the first slice):
//!
//! - Primitives: string / integer / number / boolean / null
//! - Compounds: array (homogeneous), object (named fields + required list)
//! - Output: draft-07 shape with `type`, `properties`, `required`, `items`
//! - Field metadata: `description`, `minLength`, `maxLength`,
//!   `minimum`, `maximum` for
//!   Pydantic-like `Field(...)`
//!
//! Out of scope (deferred to follow-ups): broader constraint vocabulary
//! (`pattern`, `enum`, …), union / oneOf, $ref,
//! tuple-typed arrays, additionalProperties tuning.
//!
//! @spec gh://chrischeng-c4/cclab/issues/1949 — feat(agent): Schema builder
//! API + primitive field types. The spec lives on the GitHub issue body
//! until a `.aw/tech-design/projects/agentkit/specs/schema-builder.md`
//! spec gets authored alongside the rest of the typed-I/O track.

// HANDWRITE-BEGIN reason: codegen has no generator for fresh Rust modules
// that define a primitive vocabulary (enum + builder + JSON emitter). The
// existing `schema` section type targets DB / API schemas, not in-language
// type-system primitives. Closing this gap requires a new section type
// ("rust-primitive-vocab" or similar) plus a generator that emits an enum
// per primitive + a builder per compound. Tracked under the typed-I/O
// track epic — issue link will be created when the SDD pipeline can host
// a spec for this file (see todo for P-track spec authoring).
//
// Until the generator lands: this file is the temporary state; once codegen
// can emit the same surface byte-equivalently, the markers below get
// rewritten to CODEGEN-BEGIN / CODEGEN-END and the source of truth becomes
// the spec file.

use serde_json::{json, Map, Number, Value};

/// JSON Schema node.
///
/// Each variant maps to one of JSON Schema draft-07's primitive `type`
/// values, plus the two compound shapes (`array`, `object`). The
/// representation is intentionally non-`serde`-derived so that
/// [`to_json_schema`](Self::to_json_schema) can emit the exact draft-07
/// shape (e.g. an empty `required` list is omitted; `items` is recursive)
/// without leaking the internal layout.
#[derive(Debug, Clone, PartialEq)]
pub enum Schema {
    String,
    Integer,
    Number,
    Boolean,
    Null,
    Array(Box<Schema>),
    Object {
        properties: Vec<(String, Schema)>,
        required: Vec<String>,
    },
    /// `null | inner`. Validates either a JSON null OR a value matching
    /// `inner`. Used for pydantic-style `Option<T>` fields.
    Optional(Box<Schema>),
    /// `inner` plus JSON Schema field metadata. Descriptions enrich
    /// OpenAPI/tool schemas; constraint keywords also participate in
    /// validation.
    Annotated {
        schema: Box<Schema>,
        metadata: SchemaMetadata,
    },
}

impl Schema {
    /// Build a `string` schema.
    pub fn string() -> Self {
        Self::String
    }

    /// Build an `integer` schema.
    pub fn integer() -> Self {
        Self::Integer
    }

    /// Build a `number` schema (any JSON number, integer or fractional).
    pub fn number() -> Self {
        Self::Number
    }

    /// Build a `boolean` schema.
    pub fn boolean() -> Self {
        Self::Boolean
    }

    /// Build a `null` schema.
    pub fn null() -> Self {
        Self::Null
    }

    /// Build an `array` schema with a homogeneous item type.
    pub fn array(item: Schema) -> Self {
        Self::Array(Box::new(item))
    }

    /// Wrap a schema as nullable. The resulting schema accepts JSON `null`
    /// OR any value matching `inner`. Mirrors pydantic's `Optional[T]`.
    pub fn optional(inner: Schema) -> Self {
        Self::Optional(Box::new(inner))
    }

    /// Attach a JSON Schema `description` annotation to this node.
    ///
    /// Mirrors Pydantic's `Field(description=...)` metadata. The annotation
    /// is emitted by [`to_json_schema`](Self::to_json_schema) but is ignored by
    /// [`validate`](Self::validate), because descriptions are documentation,
    /// not validation constraints.
    pub fn description(self, description: impl Into<String>) -> Self {
        let description = description.into();
        if description.is_empty() {
            return self;
        }
        self.with_metadata(|metadata| {
            metadata.description = Some(description);
        })
    }

    /// Require string values to contain at least `min_length` Unicode scalar
    /// values. Emits JSON Schema `minLength` and validates present strings.
    ///
    /// Non-string values are still handled by the inner schema. This means
    /// nullable schemas such as `Schema::optional(Schema::string()).min_length(2)`
    /// continue to accept `null`, matching JSON Schema keyword semantics.
    pub fn min_length(self, min_length: usize) -> Self {
        self.with_metadata(|metadata| {
            metadata.min_length = Some(min_length);
        })
    }

    /// Require string values to contain at most `max_length` Unicode scalar
    /// values. Emits JSON Schema `maxLength` and validates present strings.
    pub fn max_length(self, max_length: usize) -> Self {
        self.with_metadata(|metadata| {
            metadata.max_length = Some(max_length);
        })
    }

    /// Require numeric values to be greater than or equal to `minimum`.
    /// Emits JSON Schema `minimum` and validates present numbers.
    pub fn minimum(self, minimum: f64) -> Self {
        self.with_metadata(|metadata| {
            metadata.minimum = Some(minimum);
        })
    }

    /// Require numeric values to be less than or equal to `maximum`.
    /// Emits JSON Schema `maximum` and validates present numbers.
    pub fn maximum(self, maximum: f64) -> Self {
        self.with_metadata(|metadata| {
            metadata.maximum = Some(maximum);
        })
    }

    /// Pydantic-style alias for [`minimum`](Self::minimum).
    pub fn ge(self, minimum: f64) -> Self {
        self.minimum(minimum)
    }

    /// Pydantic-style alias for [`maximum`](Self::maximum).
    pub fn le(self, maximum: f64) -> Self {
        self.maximum(maximum)
    }

    /// Start an object schema builder. Fields are added via
    /// [`ObjectSchemaBuilder::field`], required keys via
    /// [`ObjectSchemaBuilder::required`], and the builder finalised via
    /// [`ObjectSchemaBuilder::build`].
    pub fn object() -> ObjectSchemaBuilder {
        ObjectSchemaBuilder::default()
    }

    /// Emit the draft-07 JSON Schema for this node.
    ///
    /// Output shape:
    ///
    /// - Primitives: `{"type": "<name>"}`.
    /// - Array: `{"type": "array", "items": <recursive>}`.
    /// - Object: `{"type": "object", "properties": {<k>: <recursive>...}, "required": [...]?}`.
    ///   The `required` key is **omitted** when the list is empty so the
    ///   output matches the shape pydantic produces for fully-optional
    ///   objects.
    pub fn to_json_schema(&self) -> Value {
        match self {
            Self::String => json!({"type": "string"}),
            Self::Integer => json!({"type": "integer"}),
            Self::Number => json!({"type": "number"}),
            Self::Boolean => json!({"type": "boolean"}),
            Self::Null => json!({"type": "null"}),
            Self::Array(item) => json!({
                "type": "array",
                "items": item.to_json_schema(),
            }),
            Self::Optional(inner) => json!({
                "anyOf": [{"type": "null"}, inner.to_json_schema()],
            }),
            Self::Annotated { schema, metadata } => {
                let mut out = schema.to_json_schema();
                apply_metadata(&mut out, metadata);
                out
            }
            Self::Object {
                properties,
                required,
            } => {
                let mut props = Map::new();
                for (name, schema) in properties {
                    props.insert(name.clone(), schema.to_json_schema());
                }
                let mut out = Map::new();
                out.insert("type".to_string(), Value::String("object".to_string()));
                out.insert("properties".to_string(), Value::Object(props));
                if !required.is_empty() {
                    out.insert(
                        "required".to_string(),
                        Value::Array(required.iter().map(|s| Value::String(s.clone())).collect()),
                    );
                }
                Value::Object(out)
            }
        }
    }

    /// Validate a JSON value against this schema.
    ///
    /// Returns `Ok(())` on a structural match. On mismatch, returns the
    /// first error encountered as the engine walks the value. The error
    /// carries a JSONPath-style breadcrumb in `ValidationError::path`
    /// (e.g. `["users", "0", "email"]`).
    ///
    /// Semantics (matches the P2 spec):
    ///
    /// - Primitives are strict — no coercion. `"30"` does not satisfy
    ///   `integer`. Integer requires `n.is_i64() || n.is_u64()`; number
    ///   accepts any JSON number.
    /// - Object: every required field must be present; extra fields are
    ///   tolerated (consistent with pydantic's default `extra='ignore'`).
    /// - Array: every element must match `items`. Element index becomes
    ///   the path segment.
    /// - Optional: `null` always passes; otherwise the inner schema is
    ///   applied (without adding a path segment).
    pub fn validate(&self, value: &Value) -> Result<(), ValidationError> {
        let mut path = Vec::new();
        walk(self, value, &mut path)
    }

    fn with_metadata<F>(self, update: F) -> Self
    where
        F: FnOnce(&mut SchemaMetadata),
    {
        match self {
            Self::Annotated {
                schema,
                mut metadata,
            } => {
                update(&mut metadata);
                Self::Annotated { schema, metadata }
            }
            inner => {
                let mut metadata = SchemaMetadata::default();
                update(&mut metadata);
                Self::Annotated {
                    schema: Box::new(inner),
                    metadata,
                }
            }
        }
    }
}

/// JSON Schema field metadata attached to a [`Schema`] node.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SchemaMetadata {
    pub description: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
}

fn apply_metadata(out: &mut Value, metadata: &SchemaMetadata) {
    let Some(object) = out.as_object_mut() else {
        return;
    };
    if let Some(description) = &metadata.description {
        object.insert(
            "description".to_string(),
            Value::String(description.clone()),
        );
    }
    if let Some(min_length) = metadata.min_length {
        object.insert("minLength".to_string(), Value::from(min_length as u64));
    }
    if let Some(max_length) = metadata.max_length {
        object.insert("maxLength".to_string(), Value::from(max_length as u64));
    }
    if let Some(minimum) = metadata.minimum {
        insert_f64_keyword(object, "minimum", minimum);
    }
    if let Some(maximum) = metadata.maximum {
        insert_f64_keyword(object, "maximum", maximum);
    }
}

fn insert_f64_keyword(object: &mut Map<String, Value>, key: &str, value: f64) {
    if let Some(number) = Number::from_f64(value) {
        object.insert(key.to_string(), Value::Number(number));
    }
}

fn validate_metadata(
    value: &Value,
    metadata: &SchemaMetadata,
    path: &[String],
) -> Result<(), ValidationError> {
    let Value::String(text) = value else {
        return Ok(());
    };
    let len = text.chars().count();
    if let Some(min_length) = metadata.min_length {
        if len < min_length {
            return Err(ValidationError::at(
                path,
                format!("expected at least {min_length} characters"),
            ));
        }
    }
    if let Some(max_length) = metadata.max_length {
        if len > max_length {
            return Err(ValidationError::at(
                path,
                format!("expected at most {max_length} characters"),
            ));
        }
    }
    Ok(())
}

fn validate_numeric_metadata(
    value: &Value,
    metadata: &SchemaMetadata,
    path: &[String],
) -> Result<(), ValidationError> {
    let Value::Number(number) = value else {
        return Ok(());
    };
    let Some(actual) = number.as_f64() else {
        return Ok(());
    };
    if let Some(minimum) = metadata.minimum {
        if actual < minimum {
            return Err(ValidationError::at(
                path,
                format!("expected >= {}", format_bound(minimum)),
            ));
        }
    }
    if let Some(maximum) = metadata.maximum {
        if actual > maximum {
            return Err(ValidationError::at(
                path,
                format!("expected <= {}", format_bound(maximum)),
            ));
        }
    }
    Ok(())
}

fn format_bound(bound: f64) -> String {
    if bound.is_finite() && bound.fract() == 0.0 {
        format!("{bound:.0}")
    } else {
        bound.to_string()
    }
}

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(n) if n.is_i64() || n.is_u64() => "integer",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn walk(schema: &Schema, value: &Value, path: &mut Vec<String>) -> Result<(), ValidationError> {
    match schema {
        Schema::String => match value {
            Value::String(_) => Ok(()),
            other => Err(ValidationError::at(
                path,
                format!("expected string, got {}", type_name(other)),
            )),
        },
        Schema::Integer => match value {
            Value::Number(n) if n.is_i64() || n.is_u64() => Ok(()),
            other => Err(ValidationError::at(
                path,
                format!("expected integer, got {}", type_name(other)),
            )),
        },
        Schema::Number => match value {
            Value::Number(_) => Ok(()),
            other => Err(ValidationError::at(
                path,
                format!("expected number, got {}", type_name(other)),
            )),
        },
        Schema::Boolean => match value {
            Value::Bool(_) => Ok(()),
            other => Err(ValidationError::at(
                path,
                format!("expected boolean, got {}", type_name(other)),
            )),
        },
        Schema::Null => match value {
            Value::Null => Ok(()),
            other => Err(ValidationError::at(
                path,
                format!("expected null, got {}", type_name(other)),
            )),
        },
        Schema::Array(item) => match value {
            Value::Array(items) => {
                for (idx, v) in items.iter().enumerate() {
                    path.push(idx.to_string());
                    let r = walk(item, v, path);
                    path.pop();
                    r?;
                }
                Ok(())
            }
            other => Err(ValidationError::at(
                path,
                format!("expected array, got {}", type_name(other)),
            )),
        },
        Schema::Optional(inner) => match value {
            Value::Null => Ok(()),
            v => walk(inner, v, path),
        },
        Schema::Annotated { schema, metadata } => {
            walk(schema, value, path)?;
            validate_metadata(value, metadata, path)?;
            validate_numeric_metadata(value, metadata, path)
        }
        Schema::Object {
            properties,
            required,
        } => match value {
            Value::Object(map) => {
                for name in required {
                    if !map.contains_key(name) {
                        path.push(name.clone());
                        let err =
                            ValidationError::at(path, format!("missing required field '{name}'"));
                        path.pop();
                        return Err(err);
                    }
                }
                for (name, sub_schema) in properties {
                    if let Some(v) = map.get(name) {
                        path.push(name.clone());
                        let r = walk(sub_schema, v, path);
                        path.pop();
                        r?;
                    }
                }
                Ok(())
            }
            other => Err(ValidationError::at(
                path,
                format!("expected object, got {}", type_name(other)),
            )),
        },
    }
}

/// Validation failure produced by [`Schema::validate`].
///
/// `path` is the JSON pointer-ish breadcrumb to the offending node
/// (empty for a root-level type mismatch). `message` is a short human
/// description, e.g. `"expected integer, got string"` or
/// `"missing required field 'email'"`.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub path: Vec<String>,
    pub message: String,
}

impl ValidationError {
    fn at(path: &[String], message: String) -> Self {
        Self {
            path: path.to_vec(),
            message,
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "{} (at /{})", self.message, self.path.join("/"))
        }
    }
}

impl std::error::Error for ValidationError {}

/// Fluent builder for [`Schema::Object`].
///
/// Field insertion order is preserved in the emitted JSON (relies on
/// `serde_json`'s `preserve_order` feature, enabled at the workspace
/// root). Required-key order matches insertion order across all calls
/// to [`Self::required`].
#[derive(Debug, Default, Clone)]
pub struct ObjectSchemaBuilder {
    properties: Vec<(String, Schema)>,
    required: Vec<String>,
}

impl ObjectSchemaBuilder {
    /// Add a named field with a value schema. Re-adding the same field
    /// name appends a second entry rather than replacing — callers
    /// shouldn't add duplicates.
    pub fn field(mut self, name: &str, schema: Schema) -> Self {
        self.properties.push((name.to_string(), schema));
        self
    }

    /// Mark the listed field names as required. Multiple calls append.
    /// Names are not validated against `properties` — passing an
    /// unknown name produces a `required` entry pointing at a missing
    /// property, which is itself a valid draft-07 schema (vacuously
    /// unsatisfiable).
    pub fn required(mut self, names: &[&str]) -> Self {
        self.required.extend(names.iter().map(|s| s.to_string()));
        self
    }

    /// Finalise the builder into a [`Schema::Object`].
    pub fn build(self) -> Schema {
        Schema::Object {
            properties: self.properties,
            required: self.required,
        }
    }
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn primitives_emit_type_only() {
        assert_eq!(Schema::string().to_json_schema(), json!({"type": "string"}));
        assert_eq!(
            Schema::integer().to_json_schema(),
            json!({"type": "integer"})
        );
        assert_eq!(Schema::number().to_json_schema(), json!({"type": "number"}));
        assert_eq!(
            Schema::boolean().to_json_schema(),
            json!({"type": "boolean"})
        );
        assert_eq!(Schema::null().to_json_schema(), json!({"type": "null"}));
    }

    #[test]
    fn array_of_primitive() {
        let s = Schema::array(Schema::integer());
        assert_eq!(
            s.to_json_schema(),
            json!({"type": "array", "items": {"type": "integer"}})
        );
    }

    #[test]
    fn object_with_required_matches_p1_acceptance_criterion() {
        // Verbatim from P1's acceptance criteria.
        let s = Schema::object()
            .field("name", Schema::string())
            .required(&["name"])
            .build();
        assert_eq!(
            s.to_json_schema(),
            json!({
                "type": "object",
                "properties": {"name": {"type": "string"}},
                "required": ["name"],
            })
        );
    }

    #[test]
    fn object_without_required_omits_required_key() {
        let s = Schema::object().field("age", Schema::integer()).build();
        let out = s.to_json_schema();
        assert_eq!(out["type"], "object");
        assert!(
            out.get("required").is_none(),
            "empty required must be omitted: {out}"
        );
    }

    #[test]
    fn nested_object() {
        let inner = Schema::object()
            .field("street", Schema::string())
            .field("zip", Schema::string())
            .required(&["street"])
            .build();
        let outer = Schema::object()
            .field("name", Schema::string())
            .field("address", inner)
            .required(&["name", "address"])
            .build();
        let out = outer.to_json_schema();
        assert_eq!(out["type"], "object");
        assert_eq!(out["properties"]["address"]["type"], "object");
        assert_eq!(
            out["properties"]["address"]["properties"]["street"]["type"],
            "string"
        );
        assert_eq!(out["properties"]["address"]["required"], json!(["street"]));
        assert_eq!(out["required"], json!(["name", "address"]));
    }

    #[test]
    fn array_of_objects() {
        let item = Schema::object()
            .field("id", Schema::integer())
            .required(&["id"])
            .build();
        let s = Schema::array(item);
        let out = s.to_json_schema();
        assert_eq!(out["type"], "array");
        assert_eq!(out["items"]["type"], "object");
        assert_eq!(out["items"]["properties"]["id"]["type"], "integer");
        assert_eq!(out["items"]["required"], json!(["id"]));
    }

    #[test]
    fn field_insertion_order_preserved() {
        let s = Schema::object()
            .field("z", Schema::string())
            .field("a", Schema::string())
            .field("m", Schema::string())
            .build();
        let out = s.to_json_schema();
        let keys: Vec<&str> = out["properties"]
            .as_object()
            .unwrap()
            .keys()
            .map(|s| s.as_str())
            .collect();
        assert_eq!(keys, vec!["z", "a", "m"]);
    }

    #[test]
    fn required_appends_across_calls() {
        let s = Schema::object()
            .field("a", Schema::string())
            .field("b", Schema::string())
            .required(&["a"])
            .required(&["b"])
            .build();
        assert_eq!(s.to_json_schema()["required"], json!(["a", "b"]));
    }

    #[test]
    fn description_annotation_emits_json_schema_keyword() {
        let s = Schema::object()
            .field(
                "email",
                Schema::string().description("User email address for notifications"),
            )
            .required(&["email"])
            .build();

        let out = s.to_json_schema();
        assert_eq!(
            out["properties"]["email"]["description"],
            "User email address for notifications"
        );
        assert_eq!(out["properties"]["email"]["type"], "string");
    }

    #[test]
    fn description_annotation_does_not_change_validation() {
        let s = Schema::integer().description("Database primary key");

        assert!(s.validate(&json!(42)).is_ok());
        let err = s.validate(&json!("42")).unwrap_err();
        assert_eq!(err.message, "expected integer, got string");
    }

    #[test]
    fn string_length_constraints_emit_json_schema_keywords() {
        let s = Schema::string().min_length(2).max_length(5);
        assert_eq!(
            s.to_json_schema(),
            json!({
                "type": "string",
                "minLength": 2,
                "maxLength": 5,
            })
        );
    }

    #[test]
    fn string_length_constraints_validate_present_strings() {
        let s = Schema::string().min_length(2).max_length(5);

        assert!(s.validate(&json!("ab")).is_ok());
        assert!(s.validate(&json!("abcde")).is_ok());

        let too_short = s.validate(&json!("a")).unwrap_err();
        assert_eq!(too_short.path, Vec::<String>::new());
        assert_eq!(too_short.message, "expected at least 2 characters");

        let too_long = s.validate(&json!("abcdef")).unwrap_err();
        assert_eq!(too_long.message, "expected at most 5 characters");
    }

    #[test]
    fn string_length_constraints_allow_nullable_null() {
        let s = Schema::optional(Schema::string()).min_length(2);

        assert!(s.validate(&json!(null)).is_ok());
        assert!(s.validate(&json!("ok")).is_ok());
        let err = s.validate(&json!("x")).unwrap_err();
        assert_eq!(err.message, "expected at least 2 characters");
    }

    #[test]
    fn numeric_bounds_emit_json_schema_keywords() {
        let s = Schema::number().minimum(0.5).maximum(10.0);
        assert_eq!(
            s.to_json_schema(),
            json!({
                "type": "number",
                "minimum": 0.5,
                "maximum": 10.0,
            })
        );
    }

    #[test]
    fn numeric_bounds_validate_numbers() {
        let s = Schema::integer().ge(1.0).le(5.0);

        assert!(s.validate(&json!(1)).is_ok());
        assert!(s.validate(&json!(5)).is_ok());

        let too_small = s.validate(&json!(0)).unwrap_err();
        assert_eq!(too_small.message, "expected >= 1");

        let too_large = s.validate(&json!(6)).unwrap_err();
        assert_eq!(too_large.message, "expected <= 5");
    }

    #[test]
    fn numeric_bounds_allow_nullable_null() {
        let s = Schema::optional(Schema::number()).minimum(0.0);

        assert!(s.validate(&json!(null)).is_ok());
        assert!(s.validate(&json!(0.25)).is_ok());
        let err = s.validate(&json!(-0.25)).unwrap_err();
        assert_eq!(err.message, "expected >= 0");
    }

    // ─── validator engine tests (P2 / #1950) ───────────────────────────

    mod validate {
        use super::*;

        #[test]
        fn primitive_happy_path() {
            assert!(Schema::string().validate(&json!("hi")).is_ok());
            assert!(Schema::integer().validate(&json!(30)).is_ok());
            assert!(Schema::number().validate(&json!(1.5)).is_ok());
            assert!(Schema::number().validate(&json!(30)).is_ok()); // integer is a number
            assert!(Schema::boolean().validate(&json!(true)).is_ok());
            assert!(Schema::null().validate(&json!(null)).is_ok());
        }

        #[test]
        fn primitive_type_mismatch_reports_actual_type() {
            let err = Schema::integer().validate(&json!("30")).unwrap_err();
            assert_eq!(err.path, Vec::<String>::new());
            assert_eq!(err.message, "expected integer, got string");
        }

        #[test]
        fn integer_strict_rejects_float() {
            let err = Schema::integer().validate(&json!(1.5)).unwrap_err();
            assert_eq!(err.message, "expected integer, got number");
        }

        #[test]
        fn p2_acceptance_criterion_happy() {
            let s = Schema::object()
                .field("age", Schema::integer())
                .required(&["age"])
                .build();
            assert!(s.validate(&json!({"age": 30})).is_ok());
        }

        #[test]
        fn p2_acceptance_criterion_type_mismatch_path() {
            let s = Schema::object()
                .field("age", Schema::integer())
                .required(&["age"])
                .build();
            let err = s.validate(&json!({"age": "30"})).unwrap_err();
            assert_eq!(err.path, vec!["age".to_string()]);
            assert_eq!(err.message, "expected integer, got string");
        }

        #[test]
        fn missing_required_field() {
            let s = Schema::object()
                .field("name", Schema::string())
                .field("age", Schema::integer())
                .required(&["name", "age"])
                .build();
            let err = s.validate(&json!({"name": "alice"})).unwrap_err();
            assert_eq!(err.path, vec!["age".to_string()]);
            assert_eq!(err.message, "missing required field 'age'");
        }

        #[test]
        fn extra_fields_are_allowed() {
            let s = Schema::object()
                .field("name", Schema::string())
                .required(&["name"])
                .build();
            assert!(s
                .validate(&json!({"name": "alice", "extra": 1, "more": [1, 2]}))
                .is_ok());
        }

        #[test]
        fn nested_object_error_carries_full_path() {
            let inner = Schema::object()
                .field("email", Schema::string())
                .required(&["email"])
                .build();
            let outer = Schema::object()
                .field("user", inner)
                .required(&["user"])
                .build();
            let err = outer.validate(&json!({"user": {"email": 42}})).unwrap_err();
            assert_eq!(err.path, vec!["user".to_string(), "email".to_string()]);
            assert_eq!(err.message, "expected string, got integer");
        }

        #[test]
        fn array_of_objects_index_in_path() {
            let item = Schema::object()
                .field("email", Schema::string())
                .required(&["email"])
                .build();
            let s = Schema::array(item);
            let err = s
                .validate(&json!([
                    {"email": "a@b"},
                    {"email": 7},
                ]))
                .unwrap_err();
            assert_eq!(err.path, vec!["1".to_string(), "email".to_string()]);
        }

        #[test]
        fn deeply_nested_path() {
            let s = Schema::object()
                .field(
                    "users",
                    Schema::array(
                        Schema::object()
                            .field(
                                "contact",
                                Schema::object()
                                    .field("email", Schema::string())
                                    .required(&["email"])
                                    .build(),
                            )
                            .required(&["contact"])
                            .build(),
                    ),
                )
                .required(&["users"])
                .build();
            let err = s
                .validate(&json!({
                    "users": [
                        {"contact": {"email": "a@b"}},
                        {"contact": {"email": 42}},
                    ]
                }))
                .unwrap_err();
            assert_eq!(
                err.path,
                vec![
                    "users".to_string(),
                    "1".to_string(),
                    "contact".to_string(),
                    "email".to_string(),
                ]
            );
            // Display formatting includes the breadcrumb.
            assert_eq!(
                err.to_string(),
                "expected string, got integer (at /users/1/contact/email)"
            );
        }

        #[test]
        fn optional_accepts_null_or_inner() {
            let s = Schema::object()
                .field("middle_name", Schema::optional(Schema::string()))
                .build();
            assert!(s.validate(&json!({"middle_name": null})).is_ok());
            assert!(s.validate(&json!({"middle_name": "Quinn"})).is_ok());
            // Optional + omitted (not in required list) → OK.
            assert!(s.validate(&json!({})).is_ok());
            // Type mismatch inside optional → error.
            let err = s.validate(&json!({"middle_name": 99})).unwrap_err();
            assert_eq!(err.path, vec!["middle_name".to_string()]);
            assert_eq!(err.message, "expected string, got integer");
        }

        #[test]
        fn optional_to_json_schema_emits_any_of() {
            let s = Schema::optional(Schema::string());
            assert_eq!(
                s.to_json_schema(),
                json!({
                    "anyOf": [{"type": "null"}, {"type": "string"}],
                })
            );
        }

        #[test]
        fn root_type_mismatch_has_empty_path() {
            let s = Schema::object().field("x", Schema::string()).build();
            let err = s.validate(&json!([1, 2, 3])).unwrap_err();
            assert!(err.path.is_empty());
            assert_eq!(err.message, "expected object, got array");
        }
    }
}
