//! Smoke tests for `#[derive(AgentSchema)]` (P4 / #1952 + P5 / #1953).
//!
//! Covers:
//! - Primitive field mapping (String, integer family, f32/f64, bool)
//! - `Vec<T>` and nested `Vec<Vec<T>>`
//! - `Option<T>` (dropped from `required`)
//! - Nested user-defined struct (recursion through `<T>::schema()`)
//! - Roundtrip: derive → schema → validate JSON payload
//
// HANDWRITE-BEGIN reason: codegen has no rust-proc-macro-derive section
// type yet. Once that lands, the `tests/` smoke harness should be one of
// the templated outputs of the derive generator.

use agent::{AgentSchema, Schema};
use serde_json::{json, Value};

#[derive(AgentSchema)]
#[allow(dead_code)] // fields are only inspected via the generated `schema()` impl
struct User {
    name: String,
    age: i64,
    active: bool,
}

#[derive(AgentSchema)]
#[allow(dead_code)]
struct Mixed {
    s: String,
    i: i32,
    u: u32,
    f: f64,
    b: bool,
}

// ── Nested + Vec + Option (P5 / #1953) ─────────────────────────────────────

#[derive(AgentSchema)]
#[allow(dead_code)]
struct Address {
    city: String,
    zip: String,
}

#[derive(AgentSchema)]
#[allow(dead_code)]
struct Person {
    name: String,
    tags: Vec<String>,
    addr: Option<Address>,
}

#[derive(AgentSchema)]
#[allow(dead_code)]
struct DeeplyNested {
    // 3-level recursion: Vec<Option<Vec<i64>>>
    matrix: Vec<Option<Vec<i64>>>,
    // Nested user type composed via Option
    maybe_person: Option<Person>,
}

#[derive(AgentSchema)]
#[allow(dead_code)]
struct DescribedInput {
    #[schema(description = "Public display name", min_length = 2, max_length = 40)]
    name: String,
    #[schema(
        description = "Optional nickname shown in profile cards",
        min_length = 2
    )]
    nickname: Option<String>,
}

#[derive(AgentSchema)]
#[allow(dead_code)]
struct NumericInput {
    #[schema(description = "Percent score", ge = 0, le = 100)]
    score: i32,
    #[schema(
        description = "Confidence from zero to one",
        minimum = 0.0,
        maximum = 1.0
    )]
    confidence: f64,
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[test]
fn user_schema_shape_matches_expected_json_schema() {
    let js = User::schema().to_json_schema();
    let want = json!({
        "type": "object",
        "properties": {
            "name":   { "type": "string"  },
            "age":    { "type": "integer" },
            "active": { "type": "boolean" }
        },
        "required": ["name", "age", "active"]
    });
    assert_eq!(js, want);
}

#[test]
fn mixed_primitive_mapping_is_correct() {
    let js = Mixed::schema().to_json_schema();
    let props = js.get("properties").and_then(Value::as_object).unwrap();
    assert_eq!(props.get("s").unwrap(), &json!({"type": "string"}));
    assert_eq!(props.get("i").unwrap(), &json!({"type": "integer"}));
    assert_eq!(props.get("u").unwrap(), &json!({"type": "integer"}));
    assert_eq!(props.get("f").unwrap(), &json!({"type": "number"}));
    assert_eq!(props.get("b").unwrap(), &json!({"type": "boolean"}));

    // Required is field-declaration order (insertion-order map).
    let required: Vec<&str> = js
        .get("required")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(required, vec!["s", "i", "u", "f", "b"]);
}

#[test]
fn derived_schema_validates_a_well_formed_value() {
    let schema = User::schema();
    let ok = json!({ "name": "alice", "age": 30, "active": true });
    schema
        .validate(&ok)
        .expect("well-formed value should validate");
}

#[test]
fn derived_schema_rejects_missing_required_field() {
    let schema = User::schema();
    let bad = json!({ "name": "alice", "age": 30 }); // missing `active`
    let err = schema.validate(&bad).unwrap_err();
    assert!(
        err.to_string().contains("active"),
        "error should mention missing field `active`: {err}"
    );
}

#[test]
fn derived_schema_rejects_wrong_type() {
    let schema = User::schema();
    let bad = json!({ "name": "alice", "age": "thirty", "active": true });
    let err = schema.validate(&bad).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("integer"),
        "error should mention integer type: {err}"
    );
}

// ── P5 / #1953: Vec / Option / nested ──────────────────────────────────────

#[test]
fn vec_of_string_emits_array_of_strings() {
    // Hand-rolled equivalent for direct comparison.
    let want = Schema::object()
        .field("name", Schema::string())
        .field("tags", Schema::array(Schema::string()))
        .field(
            "addr",
            Schema::optional(
                Schema::object()
                    .field("city", Schema::string())
                    .field("zip", Schema::string())
                    .required(&["city", "zip"])
                    .build(),
            ),
        )
        .required(&["name", "tags"])
        .build()
        .to_json_schema();
    assert_eq!(Person::schema().to_json_schema(), want);
}

#[test]
fn option_drops_field_from_required() {
    let js = Person::schema().to_json_schema();
    let required: Vec<&str> = js
        .get("required")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(required, vec!["name", "tags"]);
    assert!(
        !required.contains(&"addr"),
        "Option<T> field must not be required"
    );
}

#[test]
fn option_emits_any_of_with_null() {
    let js = Person::schema().to_json_schema();
    let addr = js
        .get("properties")
        .and_then(|p| p.get("addr"))
        .expect("addr field present");
    let any_of = addr.get("anyOf").and_then(Value::as_array).unwrap();
    assert_eq!(any_of.len(), 2);
    assert_eq!(any_of[0], json!({"type": "null"}));
    // any_of[1] is the inner Address object schema
    assert_eq!(any_of[1].get("type").unwrap(), "object");
}

#[test]
fn nested_user_type_composes_via_schema_method() {
    let js = Person::schema().to_json_schema();
    // Inside anyOf[1] is the nested Address schema.
    let addr_schema = &js["properties"]["addr"]["anyOf"][1];
    assert_eq!(addr_schema["type"], "object");
    let props = addr_schema["properties"].as_object().unwrap();
    assert!(props.contains_key("city"));
    assert!(props.contains_key("zip"));
    let required: Vec<&str> = addr_schema["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(required, vec!["city", "zip"]);
}

#[test]
fn deeply_nested_vec_option_vec_is_well_formed() {
    let js = DeeplyNested::schema().to_json_schema();
    let matrix = &js["properties"]["matrix"];
    // Vec<Option<Vec<i64>>> → array(optional(array(integer)))
    assert_eq!(matrix["type"], "array");
    let outer_items = &matrix["items"];
    // outer_items is the Option wrapper (anyOf with null + inner array)
    let any_of = outer_items["anyOf"].as_array().unwrap();
    assert_eq!(any_of[0], json!({"type": "null"}));
    assert_eq!(any_of[1]["type"], "array");
    assert_eq!(any_of[1]["items"], json!({"type": "integer"}));
}

#[test]
fn nested_user_type_inside_option_composes() {
    let js = DeeplyNested::schema().to_json_schema();
    let mp = &js["properties"]["maybe_person"];
    // Option<Person> → anyOf [null, Person object schema]
    let any_of = mp["anyOf"].as_array().unwrap();
    assert_eq!(any_of[0], json!({"type": "null"}));
    assert_eq!(any_of[1]["type"], "object");
    // Person has name + tags + addr properties
    assert!(any_of[1]["properties"]["name"].is_object());
    assert!(any_of[1]["properties"]["tags"].is_object());
    assert!(any_of[1]["properties"]["addr"].is_object());
}

#[test]
fn roundtrip_validate_real_json_payload_against_derived_schema() {
    let schema = Person::schema();
    let payload = json!({
        "name": "alice",
        "tags": ["admin", "founder"],
        "addr": { "city": "Singapore", "zip": "048942" }
    });
    schema
        .validate(&payload)
        .expect("payload with Vec<String> + nested Option<Address> should validate");

    // null for Option<Address> is also accepted.
    let payload_null = json!({
        "name": "bob",
        "tags": [],
        "addr": null
    });
    schema
        .validate(&payload_null)
        .expect("Option<T> field set to null must validate");

    // Omitting the optional field entirely is allowed too (Optional only
    // applies when the key is present; missing optional keys are fine
    // because the field is not in `required`).
    let payload_omitted = json!({ "name": "carol", "tags": ["solo"] });
    schema
        .validate(&payload_omitted)
        .expect("omitting an Option<T> field must validate");
}

#[test]
fn missing_required_vec_field_is_rejected() {
    let schema = Person::schema();
    let bad = json!({ "name": "x" }); // missing `tags`
    let err = schema.validate(&bad).unwrap_err();
    assert!(err.to_string().contains("tags"));
}

#[test]
fn schema_description_attribute_emits_field_description() {
    let js = DescribedInput::schema().to_json_schema();

    assert_eq!(
        js["properties"]["name"]["description"],
        "Public display name"
    );
    assert_eq!(js["properties"]["name"]["minLength"], 2);
    assert_eq!(js["properties"]["name"]["maxLength"], 40);
    assert_eq!(
        js["properties"]["nickname"]["description"],
        "Optional nickname shown in profile cards"
    );
    assert_eq!(js["properties"]["nickname"]["minLength"], 2);
    assert_eq!(
        js["properties"]["nickname"]["anyOf"][0],
        json!({"type": "null"})
    );

    let required: Vec<&str> = js["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(required, vec!["name"]);
}

#[test]
fn schema_length_attributes_validate_string_fields() {
    let schema = DescribedInput::schema();

    schema
        .validate(&json!({"name": "Alice", "nickname": null}))
        .expect("null optional nickname remains valid");
    schema
        .validate(&json!({"name": "Al", "nickname": "Ace"}))
        .expect("values inside length bounds validate");

    let name_err = schema
        .validate(&json!({"name": "A", "nickname": null}))
        .unwrap_err();
    assert_eq!(name_err.path, vec!["name".to_string()]);
    assert_eq!(name_err.message, "expected at least 2 characters");

    let nickname_err = schema
        .validate(&json!({"name": "Alice", "nickname": "A"}))
        .unwrap_err();
    assert_eq!(nickname_err.path, vec!["nickname".to_string()]);
    assert_eq!(nickname_err.message, "expected at least 2 characters");
}

#[test]
fn schema_numeric_attributes_emit_bounds() {
    let js = NumericInput::schema().to_json_schema();

    assert_eq!(js["properties"]["score"]["minimum"], 0.0);
    assert_eq!(js["properties"]["score"]["maximum"], 100.0);
    assert_eq!(js["properties"]["confidence"]["minimum"], 0.0);
    assert_eq!(js["properties"]["confidence"]["maximum"], 1.0);
}

#[test]
fn schema_numeric_attributes_validate_bounds() {
    let schema = NumericInput::schema();

    schema
        .validate(&json!({"score": 90, "confidence": 0.9}))
        .expect("values inside numeric bounds validate");

    let score_err = schema
        .validate(&json!({"score": 101, "confidence": 0.9}))
        .unwrap_err();
    assert_eq!(score_err.path, vec!["score".to_string()]);
    assert_eq!(score_err.message, "expected <= 100");

    let confidence_err = schema
        .validate(&json!({"score": 90, "confidence": -0.1}))
        .unwrap_err();
    assert_eq!(confidence_err.path, vec!["confidence".to_string()]);
    assert_eq!(confidence_err.message, "expected >= 0");
}

// HANDWRITE-END
