// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/engine/filters.md#source
// CODEGEN-BEGIN
//! Custom Tera filters for case conversion

use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use std::collections::HashMap;
use tera::{Result, Value};

/// Convert string to PascalCase
/// @spec projects/agentic-workflow/tech-design/core/generate/engine/filters.md#source
pub fn pascal_case(value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
    match value {
        Value::String(s) => Ok(Value::String(s.to_pascal_case())),
        _ => Err(tera::Error::msg("pascal_case filter requires a string")),
    }
}

/// Convert string to camelCase
/// @spec projects/agentic-workflow/tech-design/core/generate/engine/filters.md#source
pub fn camel_case(value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
    match value {
        Value::String(s) => Ok(Value::String(s.to_lower_camel_case())),
        _ => Err(tera::Error::msg("camel_case filter requires a string")),
    }
}

/// Convert string to snake_case
/// @spec projects/agentic-workflow/tech-design/core/generate/engine/filters.md#source
pub fn snake_case(value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
    match value {
        Value::String(s) => Ok(Value::String(s.to_snake_case())),
        _ => Err(tera::Error::msg("snake_case filter requires a string")),
    }
}

/// Convert string to kebab-case
/// @spec projects/agentic-workflow/tech-design/core/generate/engine/filters.md#source
pub fn kebab_case(value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
    match value {
        Value::String(s) => Ok(Value::String(s.to_kebab_case())),
        _ => Err(tera::Error::msg("kebab_case filter requires a string")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pascal_case() {
        let val = Value::String("my_module_name".to_string());
        let result = pascal_case(&val, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("MyModuleName".to_string()));
    }

    #[test]
    fn test_camel_case() {
        let val = Value::String("my_module_name".to_string());
        let result = camel_case(&val, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("myModuleName".to_string()));
    }

    #[test]
    fn test_snake_case() {
        let val = Value::String("MyModuleName".to_string());
        let result = snake_case(&val, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("my_module_name".to_string()));
    }

    #[test]
    fn test_kebab_case() {
        let val = Value::String("MyModuleName".to_string());
        let result = kebab_case(&val, &HashMap::new()).unwrap();
        assert_eq!(result, Value::String("my-module-name".to_string()));
    }
}
// CODEGEN-END
