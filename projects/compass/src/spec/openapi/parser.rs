//! OpenAPI 3.x to RestApiSpec parser
//!
//! Parses OpenAPI 3.0.x and 3.1.x specifications.

use std::collections::HashMap;

use serde_json::Value;

use crate::spec::ir::{
    DataModelSpec, EndpointDef, FieldConstraints, FieldDef, HttpMethod, ModelDef, ParamDef,
    QueryParam, RequestBody, ResponseDef, RestApiSpec, SecurityScheme, SecuritySchemeType,
    ServerDef, StringFormat,
};
use crate::type_inference::Type;

/// Error type for OpenAPI parsing
#[derive(Debug)]
pub enum OpenApiError {
    /// Invalid JSON/YAML
    ParseError(String),
    /// Missing required field
    MissingField(String),
    /// Unsupported OpenAPI version
    UnsupportedVersion(String),
    /// Invalid reference
    InvalidRef(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for OpenApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenApiError::ParseError(s) => write!(f, "parse error: {}", s),
            OpenApiError::MissingField(s) => write!(f, "missing field: {}", s),
            OpenApiError::UnsupportedVersion(s) => write!(f, "unsupported version: {}", s),
            OpenApiError::InvalidRef(s) => write!(f, "invalid reference: {}", s),
            OpenApiError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for OpenApiError {}

/// OpenAPI parser
pub struct OpenApiParser {
    /// Parsed components for reference resolution
    components: HashMap<String, Value>,
}

impl OpenApiParser {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Parse OpenAPI from JSON string
    pub fn parse_json(&mut self, json: &str) -> Result<RestApiSpec, OpenApiError> {
        let value: Value =
            serde_json::from_str(json).map_err(|e| OpenApiError::ParseError(e.to_string()))?;
        self.parse_value(&value)
    }

    /// Parse OpenAPI from YAML string
    pub fn parse_yaml(&mut self, yaml: &str) -> Result<RestApiSpec, OpenApiError> {
        let value: Value =
            serde_yaml::from_str(yaml).map_err(|e| OpenApiError::ParseError(e.to_string()))?;
        self.parse_value(&value)
    }

    /// Parse OpenAPI from Value
    pub fn parse_value(&mut self, value: &Value) -> Result<RestApiSpec, OpenApiError> {
        // Check version
        let version = value
            .get("openapi")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OpenApiError::MissingField("openapi".into()))?;

        if !version.starts_with("3.") {
            return Err(OpenApiError::UnsupportedVersion(version.to_string()));
        }

        // Extract components for reference resolution
        self.extract_components(value);

        // Parse info
        let info = value
            .get("info")
            .ok_or_else(|| OpenApiError::MissingField("info".into()))?;

        let title = info
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OpenApiError::MissingField("info.title".into()))?;

        let api_version = info
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OpenApiError::MissingField("info.version".into()))?;

        let mut spec = RestApiSpec::new(title, api_version);
        spec.description = info
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Parse servers
        if let Some(servers) = value.get("servers").and_then(|v| v.as_array()) {
            for server in servers {
                if let Some(url) = server.get("url").and_then(|v| v.as_str()) {
                    spec.servers.push(ServerDef {
                        url: url.to_string(),
                        description: server
                            .get("description")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                    });
                }
            }
        }

        // Parse paths (endpoints)
        if let Some(paths) = value.get("paths").and_then(|v| v.as_object()) {
            for (path, path_item) in paths {
                self.parse_path_item(path, path_item, &mut spec)?;
            }
        }

        // Parse component schemas
        spec.schemas = self.parse_component_schemas()?;

        // Parse security schemes
        if let Some(sec_schemes) = value
            .pointer("/components/securitySchemes")
            .and_then(|v| v.as_object())
        {
            for (name, scheme) in sec_schemes {
                if let Some(parsed) = self.parse_security_scheme(name, scheme) {
                    spec.security_schemes.push(parsed);
                }
            }
        }

        Ok(spec)
    }

    /// Extract components for reference resolution
    fn extract_components(&mut self, value: &Value) {
        if let Some(components) = value.get("components").and_then(|v| v.as_object()) {
            if let Some(schemas) = components.get("schemas").and_then(|v| v.as_object()) {
                for (name, schema) in schemas {
                    self.components
                        .insert(format!("#/components/schemas/{}", name), schema.clone());
                }
            }
        }
    }

    /// Parse a path item (all operations for a path)
    fn parse_path_item(
        &self,
        path: &str,
        item: &Value,
        spec: &mut RestApiSpec,
    ) -> Result<(), OpenApiError> {
        // Shared parameters for all operations in this path
        let shared_params = item
            .get("parameters")
            .and_then(|v| v.as_array())
            .map(|arr| self.parse_parameters(arr))
            .unwrap_or_default();

        let methods = [
            ("get", HttpMethod::Get),
            ("post", HttpMethod::Post),
            ("put", HttpMethod::Put),
            ("patch", HttpMethod::Patch),
            ("delete", HttpMethod::Delete),
            ("head", HttpMethod::Head),
            ("options", HttpMethod::Options),
        ];

        for (method_str, method) in methods {
            if let Some(op) = item.get(method_str) {
                let endpoint = self.parse_operation(path, method, op, &shared_params)?;
                spec.endpoints.push(endpoint);
            }
        }

        Ok(())
    }

    /// Parse an operation
    fn parse_operation(
        &self,
        path: &str,
        method: HttpMethod,
        op: &Value,
        shared_params: &[ParsedParam],
    ) -> Result<EndpointDef, OpenApiError> {
        let operation_id = op
            .get("operationId")
            .and_then(|v| v.as_str())
            .map(String::from);
        let summary = op.get("summary").and_then(|v| v.as_str()).map(String::from);
        let description = op
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);

        let tags: Vec<String> = op
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // Parse operation-specific parameters
        let op_params = op
            .get("parameters")
            .and_then(|v| v.as_array())
            .map(|arr| self.parse_parameters(arr))
            .unwrap_or_default();

        // Combine shared and operation params
        let all_params: Vec<_> = shared_params.iter().chain(op_params.iter()).collect();

        // Separate path and query params
        let mut path_params = Vec::new();
        let mut query_params = Vec::new();

        for param in all_params {
            match param.location.as_str() {
                "path" => path_params.push(ParamDef {
                    name: param.name.clone(),
                    ty: param.ty.clone(),
                    default: param.default.clone(),
                }),
                "query" => query_params.push(QueryParam {
                    name: param.name.clone(),
                    ty: param.ty.clone(),
                    required: param.required,
                    description: param.description.clone(),
                    default: param.default.clone(),
                }),
                _ => {}
            }
        }

        // Parse request body
        let request_body = op
            .get("requestBody")
            .map(|rb| self.parse_request_body(rb))
            .transpose()?;

        // Parse responses
        let responses = self.parse_responses(op.get("responses"));

        // Security
        let security: Vec<String> = op
            .get("security")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_object())
                    .flat_map(|obj| obj.keys().cloned())
                    .collect()
            })
            .unwrap_or_default();

        let deprecated = op
            .get("deprecated")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(EndpointDef {
            path: path.to_string(),
            method,
            operation_id,
            summary,
            description,
            tags,
            path_params,
            query_params,
            request_body,
            responses,
            security,
            deprecated,
        })
    }

    /// Parse parameters
    fn parse_parameters(&self, params: &[Value]) -> Vec<ParsedParam> {
        params
            .iter()
            .filter_map(|p| self.parse_parameter(p))
            .collect()
    }

    /// Parse a single parameter
    fn parse_parameter(&self, param: &Value) -> Option<ParsedParam> {
        let name = param.get("name")?.as_str()?.to_string();
        let location = param.get("in")?.as_str()?.to_string();
        let required = param
            .get("required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let description = param
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);

        let schema = param.get("schema").unwrap_or(&Value::Null);
        let ty = self.schema_to_type(schema);

        let default = param.pointer("/schema/default").map(|v| v.to_string());

        Some(ParsedParam {
            name,
            location,
            ty,
            required,
            description,
            default,
        })
    }

    /// Parse request body
    fn parse_request_body(&self, rb: &Value) -> Result<RequestBody, OpenApiError> {
        let required = rb
            .get("required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let description = rb
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Get the first content type (usually application/json)
        let content = rb.get("content").and_then(|v| v.as_object());

        let (content_type, schema) = if let Some(c) = content {
            if let Some((ct, media)) = c.iter().next() {
                let schema = media.get("schema").unwrap_or(&Value::Null);
                (ct.clone(), self.schema_to_type(schema))
            } else {
                ("application/json".to_string(), Type::Any)
            }
        } else {
            ("application/json".to_string(), Type::Any)
        };

        Ok(RequestBody {
            content_type,
            schema,
            required,
            description,
        })
    }

    /// Parse responses
    fn parse_responses(&self, responses: Option<&Value>) -> Vec<ResponseDef> {
        let Some(responses) = responses.and_then(|v| v.as_object()) else {
            return vec![];
        };

        responses
            .iter()
            .filter_map(|(status, resp)| {
                let status_code: u16 = status.parse().ok()?;
                let description = resp
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Response")
                    .to_string();

                let content = resp.get("content").and_then(|v| v.as_object());
                let (content_type, schema) = if let Some(c) = content {
                    if let Some((ct, media)) = c.iter().next() {
                        let schema = media.get("schema").map(|s| self.schema_to_type(s));
                        (Some(ct.clone()), schema)
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };

                Some(ResponseDef {
                    status_code,
                    description,
                    schema,
                    content_type,
                })
            })
            .collect()
    }

    /// Parse security scheme
    fn parse_security_scheme(&self, name: &str, scheme: &Value) -> Option<SecurityScheme> {
        let scheme_type = scheme.get("type")?.as_str()?;

        let parsed_type = match scheme_type {
            "apiKey" => {
                let key_name = scheme.get("name")?.as_str()?.to_string();
                let location = scheme.get("in")?.as_str()?;
                SecuritySchemeType::ApiKey {
                    in_header: location == "header",
                    key_name,
                }
            }
            "http" => {
                let auth_scheme = scheme.get("scheme")?.as_str()?.to_string();
                let bearer_format = scheme
                    .get("bearerFormat")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                SecuritySchemeType::Http {
                    scheme: auth_scheme,
                    bearer_format,
                }
            }
            "oauth2" => {
                let flows: Vec<String> = scheme
                    .get("flows")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.keys().cloned().collect())
                    .unwrap_or_default();
                SecuritySchemeType::OAuth2 { flows }
            }
            "openIdConnect" => {
                let url = scheme.get("openIdConnectUrl")?.as_str()?.to_string();
                SecuritySchemeType::OpenIdConnect { url }
            }
            _ => return None,
        };

        Some(SecurityScheme {
            name: name.to_string(),
            scheme_type: parsed_type,
            description: scheme
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
        })
    }

    /// Parse component schemas into DataModelSpec
    fn parse_component_schemas(&self) -> Result<DataModelSpec, OpenApiError> {
        let mut spec = DataModelSpec::new();

        for (ref_path, schema) in &self.components {
            // Extract name from ref path
            let name = ref_path
                .strip_prefix("#/components/schemas/")
                .unwrap_or(ref_path);

            if let Some(model) = self.parse_schema_as_model(name, schema)? {
                spec.add_model(model);
            }
        }

        Ok(spec)
    }

    /// Parse a schema as a model definition
    fn parse_schema_as_model(
        &self,
        name: &str,
        schema: &Value,
    ) -> Result<Option<ModelDef>, OpenApiError> {
        let schema_type = schema.get("type").and_then(|v| v.as_str());

        // Only parse object types as models
        if schema_type != Some("object") {
            return Ok(None);
        }

        let mut model = ModelDef::new(name);
        model.description = schema
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Parse properties
        let required_fields: Vec<String> = schema
            .get("required")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        if let Some(props) = schema.get("properties").and_then(|v| v.as_object()) {
            for (prop_name, prop_schema) in props {
                let field = self.parse_property(prop_name, prop_schema, &required_fields);
                model.add_field(field);
            }
        }

        Ok(Some(model))
    }

    /// Parse a property into FieldDef
    fn parse_property(&self, name: &str, schema: &Value, required: &[String]) -> FieldDef {
        let ty = self.schema_to_type(schema);
        let is_required = required.contains(&name.to_string());

        let mut field = FieldDef::new(to_snake_case(name), ty);
        field.required = is_required;
        field.description = schema
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);

        if let Some(default) = schema.get("default") {
            field.default = Some(default.to_string());
        }

        // Parse constraints
        field.constraints = self.parse_constraints(schema);

        field
    }

    /// Convert OpenAPI schema to Type
    fn schema_to_type(&self, schema: &Value) -> Type {
        // Handle $ref
        if let Some(ref_str) = schema.get("$ref").and_then(|v| v.as_str()) {
            return self.resolve_ref(ref_str);
        }

        // Handle allOf
        if let Some(all_of) = schema.get("allOf").and_then(|v| v.as_array()) {
            if let Some(first) = all_of.first() {
                return self.schema_to_type(first);
            }
        }

        // Handle oneOf/anyOf
        if let Some(one_of) = schema.get("oneOf").and_then(|v| v.as_array()) {
            let types: Vec<Type> = one_of.iter().map(|s| self.schema_to_type(s)).collect();
            return Type::Union(types);
        }
        if let Some(any_of) = schema.get("anyOf").and_then(|v| v.as_array()) {
            let types: Vec<Type> = any_of.iter().map(|s| self.schema_to_type(s)).collect();
            return Type::Union(types);
        }

        // Get type
        let type_str = schema.get("type").and_then(|v| v.as_str()).unwrap_or("any");
        let format = schema.get("format").and_then(|v| v.as_str());
        let nullable = schema
            .get("nullable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let base_type = match type_str {
            "string" => match format {
                Some("date-time") => Type::Str, // Could use special DateTime type
                Some("date") => Type::Str,
                Some("time") => Type::Str,
                Some("email") => Type::Str,
                Some("uri") | Some("url") => Type::Str,
                Some("uuid") => Type::Str,
                Some("binary") => Type::Bytes,
                Some("byte") => Type::Bytes,
                _ => Type::Str,
            },
            "integer" => Type::Int,
            "number" => Type::Float,
            "boolean" => Type::Bool,
            "array" => {
                let items = schema.get("items").unwrap_or(&Value::Null);
                let item_type = self.schema_to_type(items);
                Type::List(Box::new(item_type))
            }
            "object" => {
                // Check for additionalProperties (dict)
                if let Some(add_props) = schema.get("additionalProperties") {
                    if add_props.is_boolean() && add_props.as_bool() == Some(true) {
                        return Type::Dict(Box::new(Type::Str), Box::new(Type::Any));
                    }
                    if add_props.is_object() {
                        let value_type = self.schema_to_type(add_props);
                        return Type::Dict(Box::new(Type::Str), Box::new(value_type));
                    }
                }

                // Check for title (named object)
                if let Some(title) = schema.get("title").and_then(|v| v.as_str()) {
                    return Type::Instance {
                        name: to_pascal_case(title),
                        module: None,
                        type_args: vec![],
                    };
                }

                Type::Dict(Box::new(Type::Str), Box::new(Type::Any))
            }
            "null" => Type::None,
            _ => Type::Any,
        };

        if nullable {
            Type::Optional(Box::new(base_type))
        } else {
            base_type
        }
    }

    /// Resolve a $ref to a Type
    fn resolve_ref(&self, ref_str: &str) -> Type {
        // Extract name from ref
        let name = ref_str
            .strip_prefix("#/components/schemas/")
            .unwrap_or(ref_str);

        Type::Instance {
            name: to_pascal_case(name),
            module: None,
            type_args: vec![],
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
            constraints.format = parse_format(format);
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
}

impl Default for OpenApiParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed parameter (intermediate)
#[derive(Debug)]
struct ParsedParam {
    name: String,
    location: String,
    ty: Type,
    required: bool,
    description: Option<String>,
    default: Option<String>,
}

/// Parse format string to StringFormat
fn parse_format(format: &str) -> Option<StringFormat> {
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
        other => Some(StringFormat::Custom(other.to_string())),
    }
}

/// Convert to snake_case
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

/// Convert to PascalCase
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

#[cfg(test)]
mod tests {
    use super::*;

    const PETSTORE_YAML: &str = r#"
openapi: "3.0.0"
info:
  title: Petstore API
  version: "1.0.0"
  description: A sample API for pets
servers:
  - url: https://api.example.com/v1
    description: Production
paths:
  /pets:
    get:
      operationId: listPets
      summary: List all pets
      tags:
        - pets
      parameters:
        - name: limit
          in: query
          description: Maximum number of pets to return
          required: false
          schema:
            type: integer
            minimum: 1
            maximum: 100
      responses:
        "200":
          description: A list of pets
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/Pet'
    post:
      operationId: createPet
      summary: Create a pet
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/NewPet'
      responses:
        "201":
          description: Pet created
  /pets/{petId}:
    get:
      operationId: getPet
      summary: Get a pet by ID
      parameters:
        - name: petId
          in: path
          required: true
          schema:
            type: string
      responses:
        "200":
          description: A pet
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Pet'
components:
  schemas:
    Pet:
      type: object
      required:
        - id
        - name
      properties:
        id:
          type: integer
        name:
          type: string
        tag:
          type: string
    NewPet:
      type: object
      required:
        - name
      properties:
        name:
          type: string
          minLength: 1
          maxLength: 100
        tag:
          type: string
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
"#;

    #[test]
    fn test_parse_petstore() {
        let mut parser = OpenApiParser::new();
        let spec = parser.parse_yaml(PETSTORE_YAML).unwrap();

        assert_eq!(spec.title, "Petstore API");
        assert_eq!(spec.version, "1.0.0");
        assert_eq!(spec.servers.len(), 1);
        assert_eq!(spec.endpoints.len(), 3);

        // Check list pets endpoint
        let list_pets = spec
            .endpoints
            .iter()
            .find(|e| e.operation_id == Some("listPets".into()));
        assert!(list_pets.is_some());
        let list_pets = list_pets.unwrap();
        assert_eq!(list_pets.method, HttpMethod::Get);
        assert_eq!(list_pets.path, "/pets");
        assert_eq!(list_pets.query_params.len(), 1);

        // Check create pet endpoint
        let create_pet = spec
            .endpoints
            .iter()
            .find(|e| e.operation_id == Some("createPet".into()));
        assert!(create_pet.is_some());
        let create_pet = create_pet.unwrap();
        assert!(create_pet.request_body.is_some());

        // Check get pet endpoint
        let get_pet = spec
            .endpoints
            .iter()
            .find(|e| e.operation_id == Some("getPet".into()));
        assert!(get_pet.is_some());
        let get_pet = get_pet.unwrap();
        assert_eq!(get_pet.path_params.len(), 1);
        assert_eq!(get_pet.path_params[0].name, "petId");
    }

    #[test]
    fn test_parse_schemas() {
        let mut parser = OpenApiParser::new();
        let spec = parser.parse_yaml(PETSTORE_YAML).unwrap();

        assert_eq!(spec.schemas.models.len(), 2);

        let pet = spec.schemas.get_model("Pet").unwrap();
        assert_eq!(pet.fields.len(), 3);

        let new_pet = spec.schemas.get_model("NewPet").unwrap();
        let name_field = new_pet.fields.iter().find(|f| f.name == "name").unwrap();
        assert!(name_field.required);
        assert_eq!(name_field.constraints.min_length, Some(1));
        assert_eq!(name_field.constraints.max_length, Some(100));
    }

    #[test]
    fn test_parse_security_schemes() {
        let mut parser = OpenApiParser::new();
        let spec = parser.parse_yaml(PETSTORE_YAML).unwrap();

        assert_eq!(spec.security_schemes.len(), 1);
        let auth = &spec.security_schemes[0];
        assert_eq!(auth.name, "bearerAuth");
        assert!(
            matches!(&auth.scheme_type, SecuritySchemeType::Http { scheme, .. } if scheme == "bearer")
        );
    }
}
