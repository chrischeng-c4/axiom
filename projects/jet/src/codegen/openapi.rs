// SPEC-MANAGED: .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
// HANDWRITE-BEGIN gap="standardize:projects-jet-src-codegen-openapi-rs" tracker="pending-tracker" reason="Existing hand-written code in projects/jet/src/codegen/openapi.rs requires tracked generator coverage."
//! OpenAPI 3.0 / 3.1 document model.
//!
//! A pragmatic deserialization subset: only the keywords the TypeScript
//! generator consumes are modeled. Unknown fields are tolerated (no
//! `deny_unknown_fields`) because real specs carry many keywords we ignore.
//! 3.0 (`nullable: true`) and 3.1 (`type: ["T", "null"]`) nullability are both
//! captured and reconciled by [`Schema::is_nullable`] / [`Schema::type_names`].

use serde::Deserialize;
use std::collections::BTreeMap;

/// Root OpenAPI document.
///
/// @spec .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
#[derive(Debug, Default, Deserialize)]
pub struct Spec {
    #[serde(default)]
    pub openapi: String,
    #[serde(default)]
    pub info: Info,
    /// Paths keyed by URL template; `BTreeMap` keeps output deterministic.
    #[serde(default)]
    pub paths: BTreeMap<String, PathItem>,
    #[serde(default)]
    pub components: Components,
}

#[derive(Debug, Default, Deserialize)]
pub struct Info {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Components {
    #[serde(default)]
    pub schemas: BTreeMap<String, RefOr<Schema>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct PathItem {
    #[serde(default)]
    pub get: Option<Operation>,
    #[serde(default)]
    pub put: Option<Operation>,
    #[serde(default)]
    pub post: Option<Operation>,
    #[serde(default)]
    pub delete: Option<Operation>,
    #[serde(default)]
    pub patch: Option<Operation>,
    /// Path-level parameters merged into every operation under this path.
    #[serde(default)]
    pub parameters: Vec<RefOr<Parameter>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Operation {
    #[serde(rename = "operationId", default)]
    pub operation_id: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub parameters: Vec<RefOr<Parameter>>,
    #[serde(rename = "requestBody", default)]
    pub request_body: Option<RefOr<RequestBody>>,
    #[serde(default)]
    pub responses: BTreeMap<String, RefOr<Response>>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Parameter {
    pub name: String,
    /// `query` | `path` | `header` | `cookie`. Kept as a string for tolerance.
    #[serde(rename = "in")]
    pub location: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub schema: Option<RefOr<Schema>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RequestBody {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub content: BTreeMap<String, MediaType>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Response {
    #[serde(default)]
    pub content: BTreeMap<String, MediaType>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct MediaType {
    #[serde(default)]
    pub schema: Option<RefOr<Schema>>,
}

/// Either a JSON Reference (`{"$ref": "..."}`) or an inline value `T`.
///
/// serde's untagged enum tries [`RefOr::Ref`] first; the `$ref` field is
/// required there, so a plain object without `$ref` falls through to
/// [`RefOr::Item`].
///
/// @spec .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RefOr<T> {
    Ref(RefObj),
    Item(Box<T>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefObj {
    #[serde(rename = "$ref")]
    pub reference: String,
}

/// `additionalProperties` may be a boolean or a schema.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum AdditionalProperties {
    Bool(bool),
    Schema(Box<RefOr<Schema>>),
}

/// JSON-Schema subset used for TypeScript type mapping.
///
/// @spec .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Schema {
    #[serde(default, rename = "type")]
    pub ty: TypeField,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub properties: BTreeMap<String, RefOr<Schema>>,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub items: Option<Box<RefOr<Schema>>>,
    #[serde(default, rename = "enum")]
    pub enum_values: Vec<serde_json::Value>,
    #[serde(default, rename = "oneOf")]
    pub one_of: Vec<RefOr<Schema>>,
    #[serde(default, rename = "anyOf")]
    pub any_of: Vec<RefOr<Schema>>,
    #[serde(default, rename = "allOf")]
    pub all_of: Vec<RefOr<Schema>>,
    /// OpenAPI 3.0 nullability. 3.1 expresses this via a `"null"` type entry.
    #[serde(default)]
    pub nullable: Option<bool>,
    #[serde(default, rename = "additionalProperties")]
    pub additional_properties: Option<AdditionalProperties>,
    #[serde(default)]
    pub description: Option<String>,
}

impl Schema {
    /// Declared types with the 3.1 `"null"` sentinel removed.
    pub fn type_names(&self) -> Vec<String> {
        self.ty
            .0
            .iter()
            .filter(|t| t.as_str() != "null")
            .cloned()
            .collect()
    }

    /// True when the schema permits `null` under either 3.0 or 3.1 rules.
    pub fn is_nullable(&self) -> bool {
        self.nullable == Some(true) || self.ty.0.iter().any(|t| t == "null")
    }
}

/// `type` keyword: a single string (3.0) or an array of strings (3.1).
/// Normalized to a `Vec<String>`; absent → empty.
#[derive(Debug, Default, Clone)]
pub struct TypeField(pub Vec<String>);

impl<'de> Deserialize<'de> for TypeField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            One(String),
            Many(Vec<String>),
        }
        let raw = Option::<Raw>::deserialize(deserializer)?;
        Ok(match raw {
            None => TypeField(Vec::new()),
            Some(Raw::One(s)) => TypeField(vec![s]),
            Some(Raw::Many(v)) => TypeField(v),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ref_or_picks_ref_when_dollar_ref_present() {
        let r: RefOr<Schema> =
            serde_json::from_str(r##"{"$ref":"#/components/schemas/Pet"}"##).unwrap();
        match r {
            RefOr::Ref(o) => assert_eq!(o.reference, "#/components/schemas/Pet"),
            RefOr::Item(_) => panic!("expected ref"),
        }
    }

    #[test]
    fn ref_or_picks_item_for_inline_schema() {
        let r: RefOr<Schema> = serde_json::from_str(r##"{"type":"string"}"##).unwrap();
        match r {
            RefOr::Item(s) => assert_eq!(s.type_names(), vec!["string".to_string()]),
            RefOr::Ref(_) => panic!("expected item"),
        }
    }

    #[test]
    fn nullable_30_and_31_converge() {
        let v30: Schema = serde_json::from_str(r##"{"type":"string","nullable":true}"##).unwrap();
        let v31: Schema = serde_json::from_str(r##"{"type":["string","null"]}"##).unwrap();
        assert!(v30.is_nullable());
        assert!(v31.is_nullable());
        assert_eq!(v30.type_names(), vec!["string".to_string()]);
        assert_eq!(v31.type_names(), vec!["string".to_string()]);
    }

    #[test]
    fn tolerates_unknown_keywords_and_exclusive_minimum_variants() {
        // exclusiveMinimum is a bool in 3.0 and a number in 3.1; both ignored.
        let a: Schema =
            serde_json::from_str(r##"{"type":"integer","exclusiveMinimum":true,"x-foo":1}"##)
                .unwrap();
        let b: Schema =
            serde_json::from_str(r##"{"type":"integer","exclusiveMinimum":0,"deprecated":true}"##)
                .unwrap();
        assert_eq!(a.type_names(), vec!["integer".to_string()]);
        assert_eq!(b.type_names(), vec!["integer".to_string()]);
    }
}
// HANDWRITE-END
