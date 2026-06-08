---
id: projects-meter-src-parametrize-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/parametrize.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/parametrize.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Parameter` | projects/meter/src/parametrize.rs | struct | pub | 131 |  |
| `ParameterSet` | projects/meter/src/parametrize.rs | struct | pub | 73 |  |
| `ParameterValue` | projects/meter/src/parametrize.rs | enum | pub | 16 |  |
| `ParametrizedTest` | projects/meter/src/parametrize.rs | struct | pub | 165 |  |
| `add` | projects/meter/src/parametrize.rs | function | pub | 88 | add(&mut self, name: String, value: ParameterValue) |
| `add_parameter` | projects/meter/src/parametrize.rs | function | pub | 183 | add_parameter(&mut self, param: Parameter) -> Result<(), String> |
| `count_instances` | projects/meter/src/parametrize.rs | function | pub | 246 | count_instances(&self) -> usize |
| `expand` | projects/meter/src/parametrize.rs | function | pub | 198 | expand(&self) -> Vec<(String, ParameterSet)> |
| `format_for_name` | projects/meter/src/parametrize.rs | function | pub | 36 | format_for_name(&self) -> String |
| `format_for_name` | projects/meter/src/parametrize.rs | function | pub | 98 | format_for_name(&self) -> String |
| `get` | projects/meter/src/parametrize.rs | function | pub | 93 | get(&self, name: &str) -> Option<&ParameterValue> |
| `is_empty` | projects/meter/src/parametrize.rs | function | pub | 111 | is_empty(&self) -> bool |
| `len` | projects/meter/src/parametrize.rs | function | pub | 116 | len(&self) -> usize |
| `new` | projects/meter/src/parametrize.rs | function | pub | 81 | new() -> Self |
| `new` | projects/meter/src/parametrize.rs | function | pub | 141 | new(name: impl Into<String>, values: Vec<ParameterValue>) -> Self |
| `new` | projects/meter/src/parametrize.rs | function | pub | 175 | new(base_name: impl Into<String>) -> Self |
| `validate` | projects/meter/src/parametrize.rs | function | pub | 149 | validate(&self) -> Result<(), String> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/parametrize.rs -->
````rust
//! Parametrize system for test framework
//!
//! Provides parametrization support similar to pytest.mark.parametrize:
//! - Single parameter multiple values
//! - Multiple parameters (Cartesian product)
//! - Test name formatting with parameter values
//! - Integration with TestMeta and fixtures

use std::collections::HashMap;

/// A single parameter value with its type information
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Boolean value
    Bool(bool),
    /// None/null value
    None,
    /// List of values (for nested parametrization)
    List(Vec<ParameterValue>),
    /// Dictionary of values
    Dict(HashMap<String, ParameterValue>),
}

impl ParameterValue {
    /// Format the value for test name display
    pub fn format_for_name(&self) -> String {
        match self {
            ParameterValue::Int(v) => v.to_string(),
            ParameterValue::Float(v) => {
                // Format float with up to 2 decimal places, removing trailing zeros
                let s = format!("{:.2}", v);
                s.trim_end_matches('0').trim_end_matches('.').to_string()
            }
            ParameterValue::String(v) => {
                // Truncate long strings for readability in test names
                if v.len() > 20 {
                    format!("{}...", &v[..17])
                } else {
                    v.clone()
                }
            }
            ParameterValue::Bool(v) => v.to_string(),
            ParameterValue::None => "None".to_string(),
            ParameterValue::List(values) => {
                let formatted: Vec<String> = values.iter().map(|v| v.format_for_name()).collect();
                format!("[{}]", formatted.join(","))
            }
            ParameterValue::Dict(_) => "{...}".to_string(),
        }
    }
}

impl std::fmt::Display for ParameterValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_for_name())
    }
}

/// A set of parameter name-value mappings for a single test instance
#[derive(Debug, Clone)]
pub struct ParameterSet {
    /// Map of parameter names to their values
    pub params: HashMap<String, ParameterValue>,
}

impl ParameterSet {
    /// Create a new empty parameter set
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    /// Add a parameter to the set
    pub fn add(&mut self, name: String, value: ParameterValue) {
        self.params.insert(name, value);
    }

    /// Get a parameter value by name
    pub fn get(&self, name: &str) -> Option<&ParameterValue> {
        self.params.get(name)
    }

    /// Format parameters for test name (e.g., "param1=value1,param2=value2")
    pub fn format_for_name(&self) -> String {
        let mut parts: Vec<String> = self
            .params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v.format_for_name()))
            .collect();

        // Sort for consistent ordering
        parts.sort();
        parts.join(",")
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Number of parameters
    pub fn len(&self) -> usize {
        self.params.len()
    }
}

impl Default for ParameterSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameter definition with name and values
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Possible values for this parameter
    pub values: Vec<ParameterValue>,
}

impl Parameter {
    /// Create a new parameter
    pub fn new(name: impl Into<String>, values: Vec<ParameterValue>) -> Self {
        Self {
            name: name.into(),
            values,
        }
    }

    /// Validate the parameter
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Parameter name cannot be empty".to_string());
        }

        if self.values.is_empty() {
            return Err(format!("Parameter '{}' has no values", self.name));
        }

        Ok(())
    }
}

/// Parametrized test configuration
#[derive(Debug, Clone)]
pub struct ParametrizedTest {
    /// Base test name (without parameter suffix)
    pub base_name: String,
    /// List of parameter definitions
    pub parameters: Vec<Parameter>,
}

impl ParametrizedTest {
    /// Create a new parametrized test
    pub fn new(base_name: impl Into<String>) -> Self {
        Self {
            base_name: base_name.into(),
            parameters: Vec::new(),
        }
    }

    /// Add a parameter to the test
    pub fn add_parameter(&mut self, param: Parameter) -> Result<(), String> {
        // Check for duplicate parameter names
        if self.parameters.iter().any(|p| p.name == param.name) {
            return Err(format!("Duplicate parameter name: '{}'", param.name));
        }

        param.validate()?;
        self.parameters.push(param);
        Ok(())
    }

    /// Expand the parametrized test into multiple test instances
    ///
    /// For single parameter: generates N test cases (N = len(values))
    /// For multiple parameters: generates Cartesian product (N × M × ...)
    pub fn expand(&self) -> Vec<(String, ParameterSet)> {
        if self.parameters.is_empty() {
            return vec![(self.base_name.clone(), ParameterSet::new())];
        }

        // Generate Cartesian product of all parameter values
        let combinations = self.cartesian_product();

        // Create test instances with formatted names
        combinations
            .into_iter()
            .map(|param_set| {
                let name = if param_set.is_empty() {
                    self.base_name.clone()
                } else {
                    format!("{}[{}]", self.base_name, param_set.format_for_name())
                };
                (name, param_set)
            })
            .collect()
    }

    /// Generate Cartesian product of all parameter values
    fn cartesian_product(&self) -> Vec<ParameterSet> {
        if self.parameters.is_empty() {
            return vec![ParameterSet::new()];
        }

        let mut result = vec![ParameterSet::new()];

        for param in &self.parameters {
            let mut new_result = Vec::new();

            for existing_set in &result {
                for value in &param.values {
                    let mut new_set = existing_set.clone();
                    new_set.add(param.name.clone(), value.clone());
                    new_result.push(new_set);
                }
            }

            result = new_result;
        }

        result
    }

    /// Count total test instances that will be generated
    pub fn count_instances(&self) -> usize {
        if self.parameters.is_empty() {
            return 1;
        }

        self.parameters.iter().map(|p| p.values.len()).product()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_value_formatting() {
        assert_eq!(ParameterValue::Int(42).format_for_name(), "42");
        assert_eq!(ParameterValue::Float(3.14).format_for_name(), "3.14");
        assert_eq!(ParameterValue::Float(10.0).format_for_name(), "10");
        assert_eq!(ParameterValue::Bool(true).format_for_name(), "true");
        assert_eq!(
            ParameterValue::String("test".into()).format_for_name(),
            "test"
        );
        assert_eq!(ParameterValue::None.format_for_name(), "None");
    }

    #[test]
    fn test_parameter_value_string_truncation() {
        let long_string = "a".repeat(30);
        let value = ParameterValue::String(long_string);
        let formatted = value.format_for_name();
        assert!(formatted.len() <= 23); // 17 chars + "..."
        assert!(formatted.ends_with("..."));
    }

    #[test]
    fn test_parameter_set_creation() {
        let mut set = ParameterSet::new();
        set.add("batch_size".into(), ParameterValue::Int(100));
        set.add("timeout".into(), ParameterValue::Float(5.0));

        assert_eq!(set.len(), 2);
        assert_eq!(set.get("batch_size"), Some(&ParameterValue::Int(100)));
    }

    #[test]
    fn test_parameter_set_formatting() {
        let mut set = ParameterSet::new();
        set.add("b".into(), ParameterValue::Int(20));
        set.add("a".into(), ParameterValue::Int(10));

        // Should be sorted alphabetically
        let formatted = set.format_for_name();
        assert_eq!(formatted, "a=10,b=20");
    }

    #[test]
    fn test_single_parameter_expansion() {
        let mut test = ParametrizedTest::new("test_insert");
        test.add_parameter(Parameter::new(
            "batch_size",
            vec![
                ParameterValue::Int(10),
                ParameterValue::Int(100),
                ParameterValue::Int(1000),
            ],
        ))
        .unwrap();

        let instances = test.expand();
        assert_eq!(instances.len(), 3);

        let names: Vec<String> = instances.iter().map(|(name, _)| name.clone()).collect();
        assert_eq!(
            names,
            vec![
                "test_insert[batch_size=10]",
                "test_insert[batch_size=100]",
                "test_insert[batch_size=1000]",
            ]
        );
    }

    #[test]
    fn test_multiple_parameters_cartesian_product() {
        let mut test = ParametrizedTest::new("test_http");
        test.add_parameter(Parameter::new(
            "method",
            vec![
                ParameterValue::String("GET".into()),
                ParameterValue::String("POST".into()),
            ],
        ))
        .unwrap();
        test.add_parameter(Parameter::new(
            "auth",
            vec![ParameterValue::Bool(true), ParameterValue::Bool(false)],
        ))
        .unwrap();

        let instances = test.expand();
        assert_eq!(instances.len(), 4); // 2 × 2 = 4

        let names: Vec<String> = instances.iter().map(|(name, _)| name.clone()).collect();

        // Should have all combinations
        assert!(names.contains(&"test_http[auth=true,method=GET]".to_string()));
        assert!(names.contains(&"test_http[auth=true,method=POST]".to_string()));
        assert!(names.contains(&"test_http[auth=false,method=GET]".to_string()));
        assert!(names.contains(&"test_http[auth=false,method=POST]".to_string()));
    }

    #[test]
    fn test_count_instances() {
        let mut test = ParametrizedTest::new("test_example");

        // No parameters
        assert_eq!(test.count_instances(), 1);

        // Single parameter with 3 values
        test.add_parameter(Parameter::new(
            "a",
            vec![
                ParameterValue::Int(1),
                ParameterValue::Int(2),
                ParameterValue::Int(3),
            ],
        ))
        .unwrap();
        assert_eq!(test.count_instances(), 3);

        // Add another parameter with 2 values
        test.add_parameter(Parameter::new(
            "b",
            vec![ParameterValue::Bool(true), ParameterValue::Bool(false)],
        ))
        .unwrap();
        assert_eq!(test.count_instances(), 6); // 3 × 2 = 6
    }

    #[test]
    fn test_duplicate_parameter_name_error() {
        let mut test = ParametrizedTest::new("test_example");

        test.add_parameter(Parameter::new("x", vec![ParameterValue::Int(1)]))
            .unwrap();

        let result = test.add_parameter(Parameter::new("x", vec![ParameterValue::Int(2)]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate parameter name"));
    }

    #[test]
    fn test_empty_values_error() {
        let param = Parameter::new("x", vec![]);
        let result = param.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("has no values"));
    }

    #[test]
    fn test_empty_parameter_name_error() {
        let param = Parameter::new("", vec![ParameterValue::Int(1)]);
        let result = param.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }

    #[test]
    fn test_parameter_set_access() {
        let mut set = ParameterSet::new();
        set.add("x".into(), ParameterValue::Int(42));

        assert_eq!(set.get("x"), Some(&ParameterValue::Int(42)));
        assert_eq!(set.get("y"), None);
    }

    #[test]
    fn test_three_parameter_expansion() {
        let mut test = ParametrizedTest::new("test_bulk");
        test.add_parameter(Parameter::new(
            "size",
            vec![ParameterValue::Int(10), ParameterValue::Int(100)],
        ))
        .unwrap();
        test.add_parameter(Parameter::new(
            "parallel",
            vec![ParameterValue::Bool(true), ParameterValue::Bool(false)],
        ))
        .unwrap();
        test.add_parameter(Parameter::new(
            "validate",
            vec![ParameterValue::Bool(true), ParameterValue::Bool(false)],
        ))
        .unwrap();

        let instances = test.expand();
        assert_eq!(instances.len(), 8); // 2 × 2 × 2 = 8
        assert_eq!(test.count_instances(), 8);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/parametrize.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/parametrize.rs` captured during meter full-codegen standardization.
```
