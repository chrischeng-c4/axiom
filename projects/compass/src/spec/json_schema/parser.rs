//! JSON Schema to SpecIR parser
//!
//! Converts JSON Schema definitions into DataModelSpec.

use std::collections::HashMap;

use serde_json::Value;

use crate::spec::ir::{
    DataModelSpec, EnumDef, EnumValue, EnumVariant, FieldConstraints, FieldDef, ModelDef,
    StringFormat,
};
use crate::type_inference::Type;

/// Error type for JSON Schema parsing
#[derive(Debug)]
pub enum JsonSchemaError {
    /// Invalid JSON
    InvalidJson(String),
    /// Missing required field
    MissingField(String),
    /// Unsupported schema feature
    UnsupportedFeature(String),
    /// Invalid type
    InvalidType(String),
    /// Reference error
    InvalidRef(String),
}

impl std::fmt::Display for JsonSchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonSchemaError::InvalidJson(s) => write!(f, "invalid JSON: {}", s),
            JsonSchemaError::MissingField(s) => write!(f, "missing field: {}", s),
            JsonSchemaError::UnsupportedFeature(s) => write!(f, "unsupported feature: {}", s),
            JsonSchemaError::InvalidType(s) => write!(f, "invalid type: {}", s),
            JsonSchemaError::InvalidRef(s) => write!(f, "invalid reference: {}", s),
        }
    }
}

impl std::error::Error for JsonSchemaError {}

/// JSON Schema parser
pub struct JsonSchemaParser {
    /// Resolved definitions (for $ref handling)
    definitions: HashMap<String, Value>,
    /// Root schema
    root: Option<Value>,
}

impl JsonSchemaParser {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            root: None,
        }
    }

    /// Parse JSON Schema from string
    pub fn parse_str(&mut self, json: &str) -> Result<DataModelSpec, JsonSchemaError> {
        let value: Value =
            serde_json::from_str(json).map_err(|e| JsonSchemaError::InvalidJson(e.to_string()))?;
        self.parse_value(&value)
    }

    /// Parse JSON Schema from Value
    pub fn parse_value(&mut self, value: &Value) -> Result<DataModelSpec, JsonSchemaError> {
        self.root = Some(value.clone());

        // Extract definitions (draft-07: definitions, draft-2020-12: $defs)
        self.extract_definitions(value);

        let mut spec = DataModelSpec::new();

        // Parse root schema if it's an object type
        if let Some(root_model) = self.parse_root_object(value)? {
            spec.add_model(root_model);
        }

        // Parse all definitions as models
        let def_keys: Vec<String> = self.definitions.keys().cloned().collect();
        for name in def_keys {
            if let Some(def_value) = self.definitions.get(&name).cloned() {
                if let Some(model) = self.parse_definition(&name, &def_value)? {
                    spec.add_model(model);
                }
            }
        }

        Ok(spec)
    }

    /// Extract definitions from schema
    fn extract_definitions(&mut self, value: &Value) {
        // Draft-07 style: definitions
        if let Some(defs) = value.get("definitions").and_then(|v| v.as_object()) {
            for (name, def) in defs {
                self.definitions.insert(name.clone(), def.clone());
            }
        }

        // Draft-2020-12 style: $defs
        if let Some(defs) = value.get("$defs").and_then(|v| v.as_object()) {
            for (name, def) in defs {
                self.definitions.insert(name.clone(), def.clone());
            }
        }
    }

    /// Parse root object as a model
    fn parse_root_object(&self, value: &Value) -> Result<Option<ModelDef>, JsonSchemaError> {
        let type_val = value.get("type").and_then(|v| v.as_str());

        if type_val != Some("object") {
            return Ok(None);
        }

        // Get title or use "Root" as default
        let name = value
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Root")
            .to_string();

        self.parse_object_schema(&name, value)
    }

    /// Parse a definition into a model
    fn parse_definition(
        &self,
        name: &str,
        value: &Value,
    ) -> Result<Option<ModelDef>, JsonSchemaError> {
        let type_val = value.get("type").and_then(|v| v.as_str());

        // Handle enum type
        if value.get("enum").is_some() {
            // Enums are handled separately
            return Ok(None);
        }

        match type_val {
            Some("object") => self.parse_object_schema(name, value),
            _ => Ok(None),
        }
    }

    /// Parse object schema into ModelDef
    fn parse_object_schema(
        &self,
        name: &str,
        value: &Value,
    ) -> Result<Option<ModelDef>, JsonSchemaError> {
        let mut model = ModelDef::new(to_pascal_case(name));

        // Description
        if let Some(desc) = value.get("description").and_then(|v| v.as_str()) {
            model.description = Some(desc.to_string());
        }

        // Required fields
        let required_fields: Vec<String> = value
            .get("required")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // Properties
        if let Some(props) = value.get("properties").and_then(|v| v.as_object()) {
            for (prop_name, prop_schema) in props {
                let field = self.parse_field(prop_name, prop_schema, &required_fields)?;
                model.fields.push(field);
            }
        }

        Ok(Some(model))
    }

    /// Parse a property into FieldDef
    fn parse_field(
        &self,
        name: &str,
        schema: &Value,
        required_fields: &[String],
    ) -> Result<FieldDef, JsonSchemaError> {
        let ty = self.parse_type(schema)?;
        let is_required = required_fields.contains(&name.to_string());

        let mut field = FieldDef::new(to_snake_case(name), ty);
        field.required = is_required;

        // Description
        if let Some(desc) = schema.get("description").and_then(|v| v.as_str()) {
            field.description = Some(desc.to_string());
        }

        // Default value
        if let Some(default) = schema.get("default") {
            field.default = Some(default.to_string());
            field.required = false;
        }

        // Constraints
        field.constraints = self.parse_constraints(schema);

        Ok(field)
    }

    /// Parse JSON Schema type into Type IR
    fn parse_type(&self, schema: &Value) -> Result<Type, JsonSchemaError> {
        // Handle $ref
        if let Some(ref_str) = schema.get("$ref").and_then(|v| v.as_str()) {
            return self.resolve_ref(ref_str);
        }

        // Handle allOf (intersection/extends)
        if let Some(all_of) = schema.get("allOf").and_then(|v| v.as_array()) {
            return self.parse_all_of(all_of);
        }

        // Handle oneOf/anyOf (union)
        if let Some(one_of) = schema.get("oneOf").and_then(|v| v.as_array()) {
            return self.parse_union(one_of);
        }
        if let Some(any_of) = schema.get("anyOf").and_then(|v| v.as_array()) {
            return self.parse_union(any_of);
        }

        // Handle enum
        if let Some(enum_vals) = schema.get("enum").and_then(|v| v.as_array()) {
            return self.parse_enum_type(enum_vals);
        }

        // Handle const
        if let Some(const_val) = schema.get("const") {
            return self.parse_literal(const_val);
        }

        // Handle type array (e.g., ["string", "null"])
        if let Some(types) = schema.get("type").and_then(|v| v.as_array()) {
            return self.parse_type_array(schema, types);
        }

        // Handle single type
        let type_str = schema.get("type").and_then(|v| v.as_str()).unwrap_or("any");

        self.parse_single_type(schema, type_str)
    }

    /// Parse a single type string
    fn parse_single_type(&self, schema: &Value, type_str: &str) -> Result<Type, JsonSchemaError> {
        match type_str {
            "string" => {
                // Check for format
                if let Some(format) = schema.get("format").and_then(|v| v.as_str()) {
                    return Ok(self.format_to_type(format));
                }
                Ok(Type::Str)
            }
            "integer" | "number" => {
                if type_str == "integer" {
                    Ok(Type::Int)
                } else {
                    Ok(Type::Float)
                }
            }
            "boolean" => Ok(Type::Bool),
            "null" => Ok(Type::None),
            "array" => {
                let item_type = if let Some(items) = schema.get("items") {
                    self.parse_type(items)?
                } else {
                    Type::Any
                };
                Ok(Type::List(Box::new(item_type)))
            }
            "object" => {
                // Check for additionalProperties (dict type)
                if let Some(add_props) = schema.get("additionalProperties") {
                    if add_props.is_boolean() && add_props.as_bool() == Some(true) {
                        return Ok(Type::Dict(Box::new(Type::Str), Box::new(Type::Any)));
                    }
                    if add_props.is_object() {
                        let value_type = self.parse_type(add_props)?;
                        return Ok(Type::Dict(Box::new(Type::Str), Box::new(value_type)));
                    }
                }
                // Named object reference
                if let Some(title) = schema.get("title").and_then(|v| v.as_str()) {
                    return Ok(Type::Instance {
                        name: to_pascal_case(title),
                        module: None,
                        type_args: vec![],
                    });
                }
                Ok(Type::Dict(Box::new(Type::Str), Box::new(Type::Any)))
            }
            _ => Err(JsonSchemaError::InvalidType(type_str.to_string())),
        }
    }

    /// Parse type array (union with null)
    fn parse_type_array(&self, schema: &Value, types: &[Value]) -> Result<Type, JsonSchemaError> {
        let mut parsed_types = Vec::new();

        for type_val in types {
            if let Some(type_str) = type_val.as_str() {
                let ty = self.parse_single_type(schema, type_str)?;
                parsed_types.push(ty);
            }
        }

        // Special case: ["type", "null"] -> Optional<type>
        if parsed_types.len() == 2 && parsed_types.contains(&Type::None) {
            let non_null = parsed_types
                .into_iter()
                .find(|t| t != &Type::None)
                .unwrap_or(Type::Any);
            return Ok(Type::Optional(Box::new(non_null)));
        }

        if parsed_types.len() == 1 {
            return Ok(parsed_types.remove(0));
        }

        Ok(Type::Union(parsed_types))
    }

    /// Parse allOf (intersection)
    fn parse_all_of(&self, items: &[Value]) -> Result<Type, JsonSchemaError> {
        // If single item, just parse it
        if items.len() == 1 {
            return self.parse_type(&items[0]);
        }

        // For multiple items, try to find a $ref and merge
        let mut base_ref = None;
        for item in items {
            if item.get("$ref").is_some() {
                base_ref = Some(self.parse_type(item)?);
                break;
            }
        }

        if let Some(base) = base_ref {
            // Return the base type - properties are merged in model parsing
            return Ok(base);
        }

        // Fall back to first item
        self.parse_type(&items[0])
    }

    /// Parse oneOf/anyOf (union)
    fn parse_union(&self, items: &[Value]) -> Result<Type, JsonSchemaError> {
        let types: Vec<Type> = items
            .iter()
            .map(|item| self.parse_type(item))
            .collect::<Result<Vec<_>, _>>()?;

        if types.len() == 1 {
            return Ok(types.into_iter().next().unwrap());
        }

        Ok(Type::Union(types))
    }

    /// Parse enum values as literal union
    fn parse_enum_type(&self, values: &[Value]) -> Result<Type, JsonSchemaError> {
        let literals: Vec<Type> = values
            .iter()
            .filter_map(|v| self.value_to_literal(v))
            .collect();

        if literals.len() == 1 {
            return Ok(literals.into_iter().next().unwrap());
        }

        Ok(Type::Union(literals))
    }

    /// Convert JSON value to Literal type
    fn value_to_literal(&self, value: &Value) -> Option<Type> {
        use crate::type_inference::LiteralValue;

        match value {
            Value::String(s) => Some(Type::Literal(LiteralValue::Str(s.clone()))),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Some(Type::Literal(LiteralValue::Int(i)))
                } else if let Some(f) = n.as_f64() {
                    Some(Type::Literal(LiteralValue::Float(f)))
                } else {
                    None
                }
            }
            Value::Bool(b) => Some(Type::Literal(LiteralValue::Bool(*b))),
            Value::Null => Some(Type::Literal(LiteralValue::None)),
            _ => None,
        }
    }

    /// Parse const value as literal
    fn parse_literal(&self, value: &Value) -> Result<Type, JsonSchemaError> {
        self.value_to_literal(value)
            .ok_or_else(|| JsonSchemaError::InvalidType("unsupported const value".to_string()))
    }

    /// Resolve $ref to a type
    fn resolve_ref(&self, ref_str: &str) -> Result<Type, JsonSchemaError> {
        // Handle local references: #/definitions/Name or #/$defs/Name
        if let Some(name) = ref_str.strip_prefix("#/definitions/") {
            return Ok(Type::Instance {
                name: to_pascal_case(name),
                module: None,
                type_args: vec![],
            });
        }
        if let Some(name) = ref_str.strip_prefix("#/$defs/") {
            return Ok(Type::Instance {
                name: to_pascal_case(name),
                module: None,
                type_args: vec![],
            });
        }

        Err(JsonSchemaError::InvalidRef(ref_str.to_string()))
    }

    /// Convert format string to Type
    fn format_to_type(&self, format: &str) -> Type {
        // For now, all formats map to Str but we record the format
        // The constraint will be used for validation
        match format {
            "date-time" | "datetime" => Type::Str, // Could be a special DateTime type
            "date" => Type::Str,
            "time" => Type::Str,
            "email" => Type::Str,
            "uri" | "url" => Type::Str,
            "uuid" => Type::Str,
            "hostname" => Type::Str,
            "ipv4" => Type::Str,
            "ipv6" => Type::Str,
            _ => Type::Str,
        }
    }

    /// Parse constraints from schema
    fn parse_constraints(&self, schema: &Value) -> FieldConstraints {
        let mut constraints = FieldConstraints::default();

        // String constraints
        if let Some(min) = schema.get("minLength").and_then(|v| v.as_u64()) {
            constraints.min_length = Some(min as usize);
        }
        if let Some(max) = schema.get("maxLength").and_then(|v| v.as_u64()) {
            constraints.max_length = Some(max as usize);
        }
        if let Some(pattern) = schema.get("pattern").and_then(|v| v.as_str()) {
            constraints.pattern = Some(pattern.to_string());
        }
        if let Some(format) = schema.get("format").and_then(|v| v.as_str()) {
            constraints.format = parse_string_format(format);
        }

        // Numeric constraints
        if let Some(min) = schema.get("minimum").and_then(|v| v.as_f64()) {
            constraints.minimum = Some(min);
        }
        if let Some(max) = schema.get("maximum").and_then(|v| v.as_f64()) {
            constraints.maximum = Some(max);
        }
        if let Some(min) = schema.get("exclusiveMinimum").and_then(|v| v.as_f64()) {
            constraints.exclusive_minimum = Some(min);
        }
        if let Some(max) = schema.get("exclusiveMaximum").and_then(|v| v.as_f64()) {
            constraints.exclusive_maximum = Some(max);
        }
        if let Some(mult) = schema.get("multipleOf").and_then(|v| v.as_f64()) {
            constraints.multiple_of = Some(mult);
        }

        // Array constraints
        if let Some(min) = schema.get("minItems").and_then(|v| v.as_u64()) {
            constraints.min_items = Some(min as usize);
        }
        if let Some(max) = schema.get("maxItems").and_then(|v| v.as_u64()) {
            constraints.max_items = Some(max as usize);
        }
        if let Some(unique) = schema.get("uniqueItems").and_then(|v| v.as_bool()) {
            constraints.unique_items = unique;
        }

        constraints
    }

    /// Parse enums from schema
    pub fn parse_enums(&self, value: &Value) -> Result<Vec<EnumDef>, JsonSchemaError> {
        let mut enums = Vec::new();

        // Check definitions
        let defs_key = if value.get("$defs").is_some() {
            "$defs"
        } else {
            "definitions"
        };

        if let Some(defs) = value.get(defs_key).and_then(|v| v.as_object()) {
            for (name, def) in defs {
                if let Some(enum_vals) = def.get("enum").and_then(|v| v.as_array()) {
                    let enum_def = parse_enum_def(name, def, enum_vals);
                    enums.push(enum_def);
                }
            }
        }

        Ok(enums)
    }
}

impl Default for JsonSchemaParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse enum definition
fn parse_enum_def(name: &str, schema: &Value, values: &[Value]) -> EnumDef {
    let description = schema
        .get("description")
        .and_then(|v| v.as_str())
        .map(String::from);

    let variants: Vec<EnumVariant> = values
        .iter()
        .filter_map(|v| {
            let (variant_name, value) = match v {
                Value::String(s) => (to_pascal_case(s), Some(EnumValue::String(s.clone()))),
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        (format!("Value{}", i), Some(EnumValue::Int(i)))
                    } else {
                        return None;
                    }
                }
                _ => return None,
            };

            Some(EnumVariant {
                name: variant_name,
                value,
                description: None,
            })
        })
        .collect();

    EnumDef {
        name: to_pascal_case(name),
        description,
        variants,
    }
}

/// Parse string format
fn parse_string_format(format: &str) -> Option<StringFormat> {
    match format {
        "email" => Some(StringFormat::Email),
        "uri" | "url" => Some(StringFormat::Url),
        "uuid" => Some(StringFormat::Uuid),
        "date-time" | "datetime" => Some(StringFormat::DateTime),
        "date" => Some(StringFormat::Date),
        "time" => Some(StringFormat::Time),
        "duration" => Some(StringFormat::Duration),
        "hostname" => Some(StringFormat::Hostname),
        "ipv4" => Some(StringFormat::Ipv4),
        "ipv6" => Some(StringFormat::Ipv6),
        "regex" => Some(StringFormat::Regex),
        "json-pointer" => Some(StringFormat::JsonPointer),
        other => Some(StringFormat::Custom(other.to_string())),
    }
}

/// Convert string to PascalCase
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else if c == '-' || c == ' ' {
            result.push('_');
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let schema = r#"{
            "type": "object",
            "title": "User",
            "properties": {
                "id": { "type": "integer" },
                "name": { "type": "string" },
                "email": { "type": "string", "format": "email" }
            },
            "required": ["id", "name"]
        }"#;

        let mut parser = JsonSchemaParser::new();
        let spec = parser.parse_str(schema).unwrap();

        assert_eq!(spec.models.len(), 1);
        let model = &spec.models[0];
        assert_eq!(model.name, "User");
        assert_eq!(model.fields.len(), 3);

        let id_field = model.fields.iter().find(|f| f.name == "id").unwrap();
        assert!(id_field.required);
        assert_eq!(id_field.ty, Type::Int);

        let email_field = model.fields.iter().find(|f| f.name == "email").unwrap();
        assert!(!email_field.required);
        assert_eq!(email_field.constraints.format, Some(StringFormat::Email));
    }

    #[test]
    fn test_parse_with_definitions() {
        let schema = r##"{
            "definitions": {
                "Address": {
                    "type": "object",
                    "properties": {
                        "street": { "type": "string" },
                        "city": { "type": "string" }
                    }
                }
            },
            "type": "object",
            "title": "Person",
            "properties": {
                "name": { "type": "string" },
                "address": { "$ref": "#/definitions/Address" }
            }
        }"##;

        let mut parser = JsonSchemaParser::new();
        let spec = parser.parse_str(schema).unwrap();

        assert_eq!(spec.models.len(), 2);

        let person = spec.get_model("Person").unwrap();
        let addr_field = person.fields.iter().find(|f| f.name == "address").unwrap();
        assert!(matches!(&addr_field.ty, Type::Instance { name, .. } if name == "Address"));
    }

    #[test]
    fn test_parse_nullable_type() {
        let schema = r#"{
            "type": "object",
            "title": "Test",
            "properties": {
                "maybe_name": { "type": ["string", "null"] }
            }
        }"#;

        let mut parser = JsonSchemaParser::new();
        let spec = parser.parse_str(schema).unwrap();

        let model = &spec.models[0];
        let field = &model.fields[0];
        assert!(matches!(&field.ty, Type::Optional(inner) if **inner == Type::Str));
    }

    #[test]
    fn test_parse_array_type() {
        let schema = r#"{
            "type": "object",
            "title": "Container",
            "properties": {
                "items": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "maxItems": 10
                }
            }
        }"#;

        let mut parser = JsonSchemaParser::new();
        let spec = parser.parse_str(schema).unwrap();

        let model = &spec.models[0];
        let field = &model.fields[0];
        assert!(matches!(&field.ty, Type::List(inner) if **inner == Type::Str));
        assert_eq!(field.constraints.min_items, Some(1));
        assert_eq!(field.constraints.max_items, Some(10));
    }

    #[test]
    fn test_parse_constraints() {
        let schema = r#"{
            "type": "object",
            "title": "Validated",
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 150
                },
                "username": {
                    "type": "string",
                    "minLength": 3,
                    "maxLength": 50,
                    "pattern": "^[a-z]+$"
                }
            }
        }"#;

        let mut parser = JsonSchemaParser::new();
        let spec = parser.parse_str(schema).unwrap();

        let model = &spec.models[0];

        let age = model.fields.iter().find(|f| f.name == "age").unwrap();
        assert_eq!(age.constraints.minimum, Some(0.0));
        assert_eq!(age.constraints.maximum, Some(150.0));

        let username = model.fields.iter().find(|f| f.name == "username").unwrap();
        assert_eq!(username.constraints.min_length, Some(3));
        assert_eq!(username.constraints.max_length, Some(50));
        assert_eq!(username.constraints.pattern, Some("^[a-z]+$".to_string()));
    }

    #[test]
    fn test_case_conversion() {
        assert_eq!(to_pascal_case("user_name"), "UserName");
        assert_eq!(to_pascal_case("user-name"), "UserName");
        assert_eq!(to_pascal_case("userName"), "UserName");

        assert_eq!(to_snake_case("UserName"), "user_name");
        assert_eq!(to_snake_case("userName"), "user_name");
        assert_eq!(to_snake_case("user-name"), "user_name");
    }
}
