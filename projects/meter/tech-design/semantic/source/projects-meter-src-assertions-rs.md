---
id: projects-meter-src-assertions-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/assertions.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/assertions.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AssertionError` | projects/meter/src/assertions.rs | struct | pub | 19 |  |
| `AssertionResult` | projects/meter/src/assertions.rs | type | pub | 13 |  |
| `Expectation` | projects/meter/src/assertions.rs | struct | pub | 58 |  |
| `expect` | projects/meter/src/assertions.rs | function | pub | 595 | expect(value: T) -> Expectation<T> |
| `is_negated` | projects/meter/src/assertions.rs | function | pub | 85 | is_negated(&self) -> bool |
| `new` | projects/meter/src/assertions.rs | function | pub | 33 | new(message: impl Into<String>, assertion_type: impl Into<String>) -> Self |
| `new` | projects/meter/src/assertions.rs | function | pub | 65 | new(value: T) -> Self |
| `not` | projects/meter/src/assertions.rs | function | pub | 74 | not(mut self) -> Self |
| `to_be_at_least` | projects/meter/src/assertions.rs | function | pub | 264 | to_be_at_least(&self, expected: &T) -> AssertionResult |
| `to_be_at_most` | projects/meter/src/assertions.rs | function | pub | 279 | to_be_at_most(&self, expected: &T) -> AssertionResult |
| `to_be_between` | projects/meter/src/assertions.rs | function | pub | 296 | to_be_between(&self, low: T, high: T) -> AssertionResult |
| `to_be_empty` | projects/meter/src/assertions.rs | function | pub | 409 | to_be_empty(&self) -> AssertionResult |
| `to_be_empty` | projects/meter/src/assertions.rs | function | pub | 501 | to_be_empty(&self) -> AssertionResult |
| `to_be_false` | projects/meter/src/assertions.rs | function | pub | 154 | to_be_false(&self) -> AssertionResult |
| `to_be_greater_than` | projects/meter/src/assertions.rs | function | pub | 217 | to_be_greater_than(&self, expected: &T) -> AssertionResult |
| `to_be_less_than` | projects/meter/src/assertions.rs | function | pub | 242 | to_be_less_than(&self, expected: &T) -> AssertionResult |
| `to_be_none` | projects/meter/src/assertions.rs | function | pub | 194 | to_be_none(&self) -> AssertionResult |
| `to_be_some` | projects/meter/src/assertions.rs | function | pub | 177 | to_be_some(&self) -> AssertionResult |
| `to_be_true` | projects/meter/src/assertions.rs | function | pub | 137 | to_be_true(&self) -> AssertionResult |
| `to_contain` | projects/meter/src/assertions.rs | function | pub | 319 | to_contain(&self, substring: &str) -> AssertionResult |
| `to_contain` | projects/meter/src/assertions.rs | function | pub | 429 | to_contain(&self, substring: &str) -> AssertionResult |
| `to_contain_item` | projects/meter/src/assertions.rs | function | pub | 469 | to_contain_item(&self, item: &T) -> AssertionResult |
| `to_end_with` | projects/meter/src/assertions.rs | function | pub | 356 | to_end_with(&self, suffix: &str) -> AssertionResult |
| `to_equal` | projects/meter/src/assertions.rs | function | pub | 96 | to_equal(&self, expected: &T) -> AssertionResult |
| `to_have_key` | projects/meter/src/assertions.rs | function | pub | 525 | to_have_key(&self, key: &str) -> AssertionResult |
| `to_have_keys` | projects/meter/src/assertions.rs | function | pub | 538 | to_have_keys(&self, keys: &[&str]) -> AssertionResult |
| `to_have_length` | projects/meter/src/assertions.rs | function | pub | 390 | to_have_length(&self, length: usize) -> AssertionResult |
| `to_have_length` | projects/meter/src/assertions.rs | function | pub | 482 | to_have_length(&self, length: usize) -> AssertionResult |
| `to_have_path_value` | projects/meter/src/assertions.rs | function | pub | 558 | to_have_path_value(&self, path: &str, expected: &JsonValue) -> AssertionResult |
| `to_match` | projects/meter/src/assertions.rs | function | pub | 371 | to_match(&self, pattern: &str) -> AssertionResult |
| `to_match` | projects/meter/src/assertions.rs | function | pub | 444 | to_match(&self, pattern: &str) -> AssertionResult |
| `to_not_equal` | projects/meter/src/assertions.rs | function | pub | 115 | to_not_equal(&self, expected: &T) -> AssertionResult |
| `to_start_with` | projects/meter/src/assertions.rs | function | pub | 341 | to_start_with(&self, prefix: &str) -> AssertionResult |
| `value` | projects/meter/src/assertions.rs | function | pub | 80 | value(&self) -> &T |
| `with_actual` | projects/meter/src/assertions.rs | function | pub | 49 | with_actual(mut self, actual: impl fmt::Debug) -> Self |
| `with_expected` | projects/meter/src/assertions.rs | function | pub | 43 | with_expected(mut self, expected: impl fmt::Debug) -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/assertions.rs -->
````rust
//! Assertion engine - expect-style assertions

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fmt;
use thiserror::Error;

/// Result type for assertions
pub type AssertionResult = Result<(), AssertionError>;

/// Assertion error with context
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("{message}")]
pub struct AssertionError {
    /// Error message
    pub message: String,
    /// Expected value (stringified)
    pub expected: Option<String>,
    /// Actual value (stringified)
    pub actual: Option<String>,
    /// Assertion type (e.g., "to_equal", "to_contain")
    pub assertion_type: String,
}

impl AssertionError {
    /// Create a new assertion error
    pub fn new(message: impl Into<String>, assertion_type: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            expected: None,
            actual: None,
            assertion_type: assertion_type.into(),
        }
    }

    /// Add expected value
    pub fn with_expected(mut self, expected: impl fmt::Debug) -> Self {
        self.expected = Some(format!("{:?}", expected));
        self
    }

    /// Add actual value
    pub fn with_actual(mut self, actual: impl fmt::Debug) -> Self {
        self.actual = Some(format!("{:?}", actual));
        self
    }
}

/// Expectation builder for fluent assertions
#[derive(Debug, Clone)]
pub struct Expectation<T> {
    value: T,
    negated: bool,
}

impl<T> Expectation<T> {
    /// Create a new expectation
    pub fn new(value: T) -> Self {
        Self {
            value,
            negated: false,
        }
    }

    /// Negate the assertion
    #[allow(clippy::should_implement_trait)]
    pub fn not(mut self) -> Self {
        self.negated = !self.negated;
        self
    }

    /// Get the value
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Check if negated
    pub fn is_negated(&self) -> bool {
        self.negated
    }
}

// =====================
// Equality Assertions
// =====================

impl<T: PartialEq + fmt::Debug> Expectation<T> {
    /// Assert value equals expected
    pub fn to_equal(&self, expected: &T) -> AssertionResult {
        let result = self.value == *expected;
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = if self.negated {
                format!("Expected value not to equal {:?}, but it did", expected)
            } else {
                format!("Expected {:?} to equal {:?}", self.value, expected)
            };
            Err(AssertionError::new(msg, "to_equal")
                .with_expected(expected)
                .with_actual(&self.value))
        }
    }

    /// Assert value does not equal expected (convenience for not().to_equal())
    pub fn to_not_equal(&self, expected: &T) -> AssertionResult {
        let result = self.value != *expected;
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!("Expected {:?} not to equal {:?}", self.value, expected);
            Err(AssertionError::new(msg, "to_not_equal")
                .with_expected(expected)
                .with_actual(&self.value))
        }
    }
}

// =====================
// Boolean Assertions
// =====================

impl Expectation<bool> {
    /// Assert value is true
    pub fn to_be_true(&self) -> AssertionResult {
        let result = self.value;
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = if self.negated {
                "Expected value to be false, but was true"
            } else {
                "Expected value to be true, but was false"
            };
            Err(AssertionError::new(msg, "to_be_true").with_actual(self.value))
        }
    }

    /// Assert value is false
    pub fn to_be_false(&self) -> AssertionResult {
        let result = !self.value;
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = if self.negated {
                "Expected value to be true, but was false"
            } else {
                "Expected value to be false, but was true"
            };
            Err(AssertionError::new(msg, "to_be_false").with_actual(self.value))
        }
    }
}

// =====================
// Option Assertions
// =====================

impl<T: fmt::Debug> Expectation<Option<T>> {
    /// Assert option is Some
    pub fn to_be_some(&self) -> AssertionResult {
        let result = self.value.is_some();
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = if self.negated {
                "Expected None, but got Some"
            } else {
                "Expected Some, but got None"
            };
            Err(AssertionError::new(msg, "to_be_some"))
        }
    }

    /// Assert option is None
    pub fn to_be_none(&self) -> AssertionResult {
        let result = self.value.is_none();
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = if self.negated {
                "Expected Some, but got None".to_string()
            } else {
                format!("Expected None, but got {:?}", self.value)
            };
            Err(AssertionError::new(msg, "to_be_none"))
        }
    }
}

// =====================
// Numeric Assertions
// =====================

impl<T: PartialOrd + fmt::Debug> Expectation<T> {
    /// Assert value is greater than expected
    pub fn to_be_greater_than(&self, expected: &T) -> AssertionResult {
        let result = self.value > *expected;
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = if self.negated {
                format!(
                    "Expected {:?} not to be greater than {:?}",
                    self.value, expected
                )
            } else {
                format!(
                    "Expected {:?} to be greater than {:?}",
                    self.value, expected
                )
            };
            Err(AssertionError::new(msg, "to_be_greater_than")
                .with_expected(expected)
                .with_actual(&self.value))
        }
    }

    /// Assert value is less than expected
    pub fn to_be_less_than(&self, expected: &T) -> AssertionResult {
        let result = self.value < *expected;
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = if self.negated {
                format!(
                    "Expected {:?} not to be less than {:?}",
                    self.value, expected
                )
            } else {
                format!("Expected {:?} to be less than {:?}", self.value, expected)
            };
            Err(AssertionError::new(msg, "to_be_less_than")
                .with_expected(expected)
                .with_actual(&self.value))
        }
    }

    /// Assert value is greater than or equal to expected
    pub fn to_be_at_least(&self, expected: &T) -> AssertionResult {
        let result = self.value >= *expected;
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!("Expected {:?} to be at least {:?}", self.value, expected);
            Err(AssertionError::new(msg, "to_be_at_least")
                .with_expected(expected)
                .with_actual(&self.value))
        }
    }

    /// Assert value is less than or equal to expected
    pub fn to_be_at_most(&self, expected: &T) -> AssertionResult {
        let result = self.value <= *expected;
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!("Expected {:?} to be at most {:?}", self.value, expected);
            Err(AssertionError::new(msg, "to_be_at_most")
                .with_expected(expected)
                .with_actual(&self.value))
        }
    }
}

impl<T: PartialOrd + fmt::Debug + Copy> Expectation<T> {
    /// Assert value is between low and high (inclusive)
    pub fn to_be_between(&self, low: T, high: T) -> AssertionResult {
        let result = self.value >= low && self.value <= high;
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!(
                "Expected {:?} to be between {:?} and {:?}",
                self.value, low, high
            );
            Err(AssertionError::new(msg, "to_be_between").with_actual(self.value))
        }
    }
}

// =====================
// String Assertions
// =====================

impl Expectation<String> {
    /// Assert string contains substring
    pub fn to_contain(&self, substring: &str) -> AssertionResult {
        let result = self.value.contains(substring);
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = if self.negated {
                format!("Expected string not to contain {:?}", substring)
            } else {
                format!(
                    "Expected string to contain {:?}, but it was {:?}",
                    substring, self.value
                )
            };
            Err(AssertionError::new(msg, "to_contain")
                .with_expected(substring)
                .with_actual(&self.value))
        }
    }

    /// Assert string starts with prefix
    pub fn to_start_with(&self, prefix: &str) -> AssertionResult {
        let result = self.value.starts_with(prefix);
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!("Expected {:?} to start with {:?}", self.value, prefix);
            Err(AssertionError::new(msg, "to_start_with")
                .with_expected(prefix)
                .with_actual(&self.value))
        }
    }

    /// Assert string ends with suffix
    pub fn to_end_with(&self, suffix: &str) -> AssertionResult {
        let result = self.value.ends_with(suffix);
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!("Expected {:?} to end with {:?}", self.value, suffix);
            Err(AssertionError::new(msg, "to_end_with")
                .with_expected(suffix)
                .with_actual(&self.value))
        }
    }

    /// Assert string matches regex pattern
    pub fn to_match(&self, pattern: &str) -> AssertionResult {
        let regex = Regex::new(pattern).map_err(|e| {
            AssertionError::new(format!("Invalid regex pattern: {}", e), "to_match")
        })?;

        let result = regex.is_match(&self.value);
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!("Expected {:?} to match pattern {:?}", self.value, pattern);
            Err(AssertionError::new(msg, "to_match")
                .with_expected(pattern)
                .with_actual(&self.value))
        }
    }

    /// Assert string has exact length
    pub fn to_have_length(&self, length: usize) -> AssertionResult {
        let actual_len = self.value.len();
        let result = actual_len == length;
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!(
                "Expected string length to be {}, but was {}",
                length, actual_len
            );
            Err(AssertionError::new(msg, "to_have_length")
                .with_expected(length)
                .with_actual(actual_len))
        }
    }

    /// Assert string is empty
    pub fn to_be_empty(&self) -> AssertionResult {
        let result = self.value.is_empty();
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = if self.negated {
                "Expected string not to be empty"
            } else {
                "Expected string to be empty"
            };
            Err(AssertionError::new(msg, "to_be_empty").with_actual(&self.value))
        }
    }
}

impl Expectation<&str> {
    /// Assert &str contains substring
    pub fn to_contain(&self, substring: &str) -> AssertionResult {
        let result = self.value.contains(substring);
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!("Expected {:?} to contain {:?}", self.value, substring);
            Err(AssertionError::new(msg, "to_contain")
                .with_expected(substring)
                .with_actual(self.value))
        }
    }

    /// Assert &str matches regex
    pub fn to_match(&self, pattern: &str) -> AssertionResult {
        let regex = Regex::new(pattern).map_err(|e| {
            AssertionError::new(format!("Invalid regex pattern: {}", e), "to_match")
        })?;

        let result = regex.is_match(self.value);
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!("Expected {:?} to match pattern {:?}", self.value, pattern);
            Err(AssertionError::new(msg, "to_match")
                .with_expected(pattern)
                .with_actual(self.value))
        }
    }
}

// =====================
// Collection Assertions
// =====================

impl<T: PartialEq + fmt::Debug> Expectation<Vec<T>> {
    /// Assert vector contains item
    pub fn to_contain_item(&self, item: &T) -> AssertionResult {
        let result = self.value.contains(item);
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!("Expected collection to contain {:?}", item);
            Err(AssertionError::new(msg, "to_contain").with_expected(item))
        }
    }

    /// Assert vector has exact length
    pub fn to_have_length(&self, length: usize) -> AssertionResult {
        let actual_len = self.value.len();
        let result = actual_len == length;
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!(
                "Expected collection length to be {}, but was {}",
                length, actual_len
            );
            Err(AssertionError::new(msg, "to_have_length")
                .with_expected(length)
                .with_actual(actual_len))
        }
    }

    /// Assert vector is empty
    pub fn to_be_empty(&self) -> AssertionResult {
        let result = self.value.is_empty();
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = if self.negated {
                "Expected collection not to be empty"
            } else {
                "Expected collection to be empty"
            };
            Err(AssertionError::new(msg, "to_be_empty"))
        }
    }
}

// =====================
// JSON Assertions
// =====================

impl Expectation<JsonValue> {
    /// Assert JSON has key
    pub fn to_have_key(&self, key: &str) -> AssertionResult {
        let result = self.value.get(key).is_some();
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!("Expected JSON to have key {:?}", key);
            Err(AssertionError::new(msg, "to_have_key").with_expected(key))
        }
    }

    /// Assert JSON has multiple keys
    pub fn to_have_keys(&self, keys: &[&str]) -> AssertionResult {
        let missing: Vec<_> = keys
            .iter()
            .filter(|k| self.value.get(*k).is_none())
            .collect();
        let result = missing.is_empty();
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!(
                "Expected JSON to have keys {:?}, missing {:?}",
                keys, missing
            );
            Err(AssertionError::new(msg, "to_have_keys"))
        }
    }

    /// Assert JSON value at path equals expected
    pub fn to_have_path_value(&self, path: &str, expected: &JsonValue) -> AssertionResult {
        let actual = json_path(&self.value, path);
        let result = actual.map(|v| v == expected).unwrap_or(false);
        let passed = if self.negated { !result } else { result };

        if passed {
            Ok(())
        } else {
            let msg = format!(
                "Expected JSON path {:?} to have value {:?}, but was {:?}",
                path, expected, actual
            );
            Err(AssertionError::new(msg, "to_have_path_value")
                .with_expected(expected)
                .with_actual(actual))
        }
    }
}

/// Simple JSON path lookup (supports dot notation like "user.name")
fn json_path<'a>(value: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        current = current.get(part)?;
    }

    Some(current)
}

// =====================
// Helper Functions
// =====================

/// Create an expectation (entry point)
pub fn expect<T>(value: T) -> Expectation<T> {
    Expectation::new(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_equal() {
        assert!(expect(42).to_equal(&42).is_ok());
        assert!(expect(42).to_equal(&43).is_err());
        assert!(expect(42).not().to_equal(&43).is_ok());
    }

    #[test]
    fn test_to_be_greater_than() {
        assert!(expect(10).to_be_greater_than(&5).is_ok());
        assert!(expect(5).to_be_greater_than(&10).is_err());
    }

    #[test]
    fn test_string_assertions() {
        assert!(expect("hello world".to_string())
            .to_contain("world")
            .is_ok());
        assert!(expect("hello".to_string()).to_start_with("he").is_ok());
        assert!(expect("hello".to_string()).to_end_with("lo").is_ok());
        assert!(expect("test123".to_string()).to_match(r"\d+").is_ok());
    }

    #[test]
    fn test_option_assertions() {
        assert!(expect(Some(42)).to_be_some().is_ok());
        assert!(expect(None::<i32>).to_be_none().is_ok());
    }

    #[test]
    fn test_collection_assertions() {
        let vec = vec![1, 2, 3];
        assert!(expect(vec.clone()).to_contain_item(&2).is_ok());
        assert!(expect(vec.clone()).to_have_length(3).is_ok());
        assert!(expect(Vec::<i32>::new()).to_be_empty().is_ok());
    }

    #[test]
    fn test_json_assertions() {
        let json: JsonValue = serde_json::json!({
            "user": {
                "name": "Alice",
                "age": 30
            }
        });

        assert!(expect(json.clone()).to_have_key("user").is_ok());
        assert!(expect(json.clone())
            .to_have_path_value("user.name", &serde_json::json!("Alice"))
            .is_ok());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/assertions.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/assertions.rs` captured during meter full-codegen standardization.
```
