// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/types.md#source
// CODEGEN-BEGIN
//! Abstract type system for multi-language codegen.
//!
//! TD specs use abstract types (`integer`, `string`, `list<T>`, etc.) in their
//! YAML frontmatter. This module translates abstract types to per-language concrete types.
//!
//! For MVP, only the Rust translator is implemented. Python/TypeScript translators
//! share the same `AbstractType` enum but have deferred implementations.

// @spec projects/agentic-workflow/tech-design/core/generate/types.md#source

// ---------------------------------------------------------------------------
// Abstract Type Enum
// ---------------------------------------------------------------------------

/// Abstract type used in structural diagram frontmatter.
///
/// YAML uses string notation: `"integer"`, `"string"`, `"list<T>"`, etc.
// @spec projects/agentic-workflow/tech-design/core/generate/types.md#source
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AbstractType {
    /// Integer with optional bit width and sign.
    Integer {
        #[serde(default = "default_int_size")]
        int_size: u8,
        #[serde(default)]
        unsigned: bool,
    },
    /// UTF-8 string.
    String,
    /// Boolean.
    Bool,
    /// Raw bytes.
    Bytes,
    /// Any JSON value.
    Any,
    /// Ordered list of items.
    List { item: Box<AbstractType> },
    /// Key-value map.
    Map {
        key: Box<AbstractType>,
        value: Box<AbstractType>,
    },
    /// Optional (nullable) value.
    Optional { inner: Box<AbstractType> },
    /// Reference to another named type in the same module.
    Ref { name: String },
}

fn default_int_size() -> u8 {
    64
}

// ---------------------------------------------------------------------------
// TypeTranslator trait
// ---------------------------------------------------------------------------

/// Translates abstract types to a target language's concrete type.
/// @spec projects/agentic-workflow/tech-design/core/generate/types.md#source
pub trait TypeTranslator {
    fn translate(&self, t: &AbstractType) -> String;
}

// ---------------------------------------------------------------------------
// Rust type translator
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};

/// Rust codegen configuration (from global config.toml + per-spec x-rust overrides).
/// @spec projects/agentic-workflow/tech-design/core/generate/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustConfig {
    /// Top-level derives applied to generated structs/enums.
    #[serde(default = "default_rust_derives")]
    pub derives: Vec<String>,
    /// Serde `rename_all` strategy applied container-level.
    #[serde(default = "default_serde_rename")]
    pub serde_rename_strategy: String,
    /// Visibility prefix for generated items.
    #[serde(default = "default_visibility")]
    pub visibility: String,
    /// Whether to add Hash to derive list.
    #[serde(default)]
    pub derive_hash: bool,
    /// Whether to add Copy to derive list.
    #[serde(default)]
    pub derive_copy: bool,
}

/// Translates abstract types to Rust type expressions.
/// @spec projects/agentic-workflow/tech-design/core/generate/types.md#schema
pub struct RustTypeTranslator;
// @spec projects/agentic-workflow/tech-design/core/generate/types.md#source
impl TypeTranslator for RustTypeTranslator {
    fn translate(&self, t: &AbstractType) -> String {
        match t {
            AbstractType::Integer { int_size, unsigned } => {
                let prefix = if *unsigned { "u" } else { "i" };
                format!("{}{}", prefix, int_size)
            }
            AbstractType::String => "String".to_string(),
            AbstractType::Bool => "bool".to_string(),
            AbstractType::Bytes => "Vec<u8>".to_string(),
            AbstractType::Any => "serde_json::Value".to_string(),
            AbstractType::List { item } => {
                format!("Vec<{}>", self.translate(item))
            }
            AbstractType::Map { key, value } => {
                format!(
                    "HashMap<{}, {}>",
                    self.translate(key),
                    self.translate(value)
                )
            }
            AbstractType::Optional { inner } => {
                format!("Option<{}>", self.translate(inner))
            }
            AbstractType::Ref { name } => name.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Parser: YAML type string → AbstractType
// ---------------------------------------------------------------------------

/// Parse an abstract type from a YAML type string.
///
/// Supports forms: `"integer"`, `"string"`, `"bool"`, `"bytes"`, `"any"`,
/// `"optional<T>"`, `"list<T>"`, `"map<K,V>"`, `"ref<Name>"`.
// @spec projects/agentic-workflow/tech-design/core/generate/types.md#source
pub fn parse_abstract_type(s: &str) -> Result<AbstractType, String> {
    let s = s.trim();

    // Parameterized types
    if let Some(inner) = strip_wrapper(s, "optional<", ">") {
        let inner_type = parse_abstract_type(inner)?;
        return Ok(AbstractType::Optional {
            inner: Box::new(inner_type),
        });
    }
    if let Some(inner) = strip_wrapper(s, "list<", ">") {
        let item = parse_abstract_type(inner)?;
        return Ok(AbstractType::List {
            item: Box::new(item),
        });
    }
    if let Some(inner) = strip_wrapper(s, "map<", ">") {
        // Split on the first comma not inside angle brackets
        let (key_str, val_str) =
            split_map_params(inner).ok_or_else(|| format!("Invalid map type params: {}", inner))?;
        let key = parse_abstract_type(key_str.trim())?;
        let value = parse_abstract_type(val_str.trim())?;
        return Ok(AbstractType::Map {
            key: Box::new(key),
            value: Box::new(value),
        });
    }
    if let Some(name) = strip_wrapper(s, "ref<", ">") {
        return Ok(AbstractType::Ref {
            name: name.to_string(),
        });
    }

    // Scalar types
    match s {
        "integer" | "int" | "i64" => Ok(AbstractType::Integer {
            int_size: 64,
            unsigned: false,
        }),
        "u64" => Ok(AbstractType::Integer {
            int_size: 64,
            unsigned: true,
        }),
        "i32" | "integer32" => Ok(AbstractType::Integer {
            int_size: 32,
            unsigned: false,
        }),
        "u32" => Ok(AbstractType::Integer {
            int_size: 32,
            unsigned: true,
        }),
        "i16" => Ok(AbstractType::Integer {
            int_size: 16,
            unsigned: false,
        }),
        "u16" => Ok(AbstractType::Integer {
            int_size: 16,
            unsigned: true,
        }),
        "i8" => Ok(AbstractType::Integer {
            int_size: 8,
            unsigned: false,
        }),
        "u8" => Ok(AbstractType::Integer {
            int_size: 8,
            unsigned: true,
        }),
        "string" | "str" | "text" | "String" => Ok(AbstractType::String),
        "bool" | "boolean" => Ok(AbstractType::Bool),
        "bytes" => Ok(AbstractType::Bytes),
        "any" | "object" | "json" => Ok(AbstractType::Any),
        other => {
            // Treat unknown types as Ref (e.g. named struct types)
            if other
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
            {
                Ok(AbstractType::Ref {
                    name: other.to_string(),
                })
            } else {
                Err(format!("Unknown abstract type: {}", other))
            }
        }
    }
}

fn strip_wrapper<'a>(s: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
    s.strip_prefix(prefix)?.strip_suffix(suffix)
}

/// Split "K,V" at the top-level comma (not inside nested angle brackets).
fn split_map_params(s: &str) -> Option<(&str, &str)> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => return Some((&s[..i], &s[i + 1..])),
            _ => {}
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Codegen config structures
// ---------------------------------------------------------------------------

/// Rust codegen configuration (from global config.toml + per-spec x-rust overrides).
// @spec projects/agentic-workflow/tech-design/core/generate/types.md#source
fn default_rust_derives() -> Vec<String> {
    vec![
        "Debug".into(),
        "Clone".into(),
        "PartialEq".into(),
        "Serialize".into(),
        "Deserialize".into(),
    ]
}

fn default_serde_rename() -> String {
    "snake_case".into()
}

fn default_visibility() -> String {
    "pub".into()
}

/// @spec projects/agentic-workflow/tech-design/core/generate/types.md#source
impl Default for RustConfig {
    fn default() -> Self {
        Self {
            derives: default_rust_derives(),
            serde_rename_strategy: default_serde_rename(),
            visibility: default_visibility(),
            derive_hash: false,
            derive_copy: false,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/types.md#source
impl RustConfig {
    /// Whether `Serialize` or `Deserialize` are in the active derives.
    /// Used to gate `#[serde(...)]` attribute emission — emitting them
    /// without the derive turns them into unused attribute noise (or
    /// compile errors on strict configs).
    pub fn has_serde_derives(&self) -> bool {
        self.derives
            .iter()
            .any(|d| d == "Serialize" || d == "Deserialize")
    }

    /// Merge per-spec x-rust overrides from frontmatter YAML value.
    pub fn merge_overrides(&self, frontmatter: &serde_yaml::Value) -> Self {
        let x_rust = match frontmatter.get("x-rust") {
            Some(v) => v,
            None => return self.clone(),
        };
        let mut merged = self.clone();
        if let Some(derives) = x_rust.get("derives").and_then(|v| v.as_sequence()) {
            merged.derives = derives
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }
        if let Some(srs) = x_rust.get("serde_rename_strategy").and_then(|v| v.as_str()) {
            merged.serde_rename_strategy = srs.to_string();
        }
        if let Some(vis) = x_rust.get("visibility").and_then(|v| v.as_str()) {
            merged.visibility = vis.to_string();
        }
        if let Some(hash) = x_rust.get("derive_hash").and_then(|v| v.as_bool()) {
            merged.derive_hash = hash;
        }
        if let Some(copy) = x_rust.get("derive_copy").and_then(|v| v.as_bool()) {
            merged.derive_copy = copy;
        }
        merged
    }

    /// Build the `#[derive(...)]` attribute string.
    pub fn derive_attr(&self) -> String {
        let mut derives = self.derives.clone();
        if self.derive_hash && !derives.contains(&"Hash".to_string()) {
            derives.push("Hash".into());
        }
        if self.derive_copy && !derives.contains(&"Copy".to_string()) {
            derives.push("Copy".into());
        }
        if derives.is_empty() {
            String::new()
        } else {
            format!("#[derive({})]", derives.join(", "))
        }
    }

    /// Build the `#[serde(rename_all = "...")]` attribute string (if needed).
    /// `snake_case` matches Rust's field naming and is serde's de-facto default,
    /// so we skip emitting it to avoid noisy redundant attributes.
    pub fn serde_rename_attr(&self) -> String {
        if self.serde_rename_strategy.is_empty()
            || self.serde_rename_strategy == "none"
            || self.serde_rename_strategy == "snake_case"
        {
            String::new()
        } else {
            format!("#[serde(rename_all = \"{}\")]", self.serde_rename_strategy)
        }
    }

    /// Visibility prefix (`"pub "`, `"pub(crate) "`, or `""`).
    pub fn vis_prefix(&self) -> String {
        if self.visibility.is_empty() {
            String::new()
        } else {
            format!("{} ", self.visibility)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer() {
        let t = parse_abstract_type("integer").unwrap();
        assert_eq!(
            t,
            AbstractType::Integer {
                int_size: 64,
                unsigned: false
            }
        );
    }

    #[test]
    fn test_parse_string() {
        let t = parse_abstract_type("string").unwrap();
        assert_eq!(t, AbstractType::String);
    }

    #[test]
    fn test_parse_optional() {
        let t = parse_abstract_type("optional<string>").unwrap();
        assert_eq!(
            t,
            AbstractType::Optional {
                inner: Box::new(AbstractType::String)
            }
        );
    }

    #[test]
    fn test_parse_list() {
        let t = parse_abstract_type("list<integer>").unwrap();
        assert_eq!(
            t,
            AbstractType::List {
                item: Box::new(AbstractType::Integer {
                    int_size: 64,
                    unsigned: false
                })
            }
        );
    }

    #[test]
    fn test_parse_map() {
        let t = parse_abstract_type("map<string,integer>").unwrap();
        match t {
            AbstractType::Map { key, value } => {
                assert_eq!(*key, AbstractType::String);
                assert_eq!(
                    *value,
                    AbstractType::Integer {
                        int_size: 64,
                        unsigned: false
                    }
                );
            }
            other => panic!("expected Map, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_ref() {
        let t = parse_abstract_type("ref<MyStruct>").unwrap();
        assert_eq!(
            t,
            AbstractType::Ref {
                name: "MyStruct".to_string()
            }
        );
    }

    #[test]
    fn test_uppercase_fallback_to_ref() {
        let t = parse_abstract_type("MyCustomType").unwrap();
        assert_eq!(
            t,
            AbstractType::Ref {
                name: "MyCustomType".to_string()
            }
        );
    }

    #[test]
    fn test_rust_translate_integer() {
        let translator = RustTypeTranslator;
        let t = AbstractType::Integer {
            int_size: 64,
            unsigned: false,
        };
        assert_eq!(translator.translate(&t), "i64");
    }

    #[test]
    fn test_rust_translate_u32() {
        let translator = RustTypeTranslator;
        let t = AbstractType::Integer {
            int_size: 32,
            unsigned: true,
        };
        assert_eq!(translator.translate(&t), "u32");
    }

    #[test]
    fn test_rust_translate_string() {
        let translator = RustTypeTranslator;
        assert_eq!(translator.translate(&AbstractType::String), "String");
    }

    #[test]
    fn test_rust_translate_optional() {
        let translator = RustTypeTranslator;
        let t = AbstractType::Optional {
            inner: Box::new(AbstractType::String),
        };
        assert_eq!(translator.translate(&t), "Option<String>");
    }

    #[test]
    fn test_rust_translate_vec() {
        let translator = RustTypeTranslator;
        let t = AbstractType::List {
            item: Box::new(AbstractType::String),
        };
        assert_eq!(translator.translate(&t), "Vec<String>");
    }

    #[test]
    fn test_rust_config_derive_attr() {
        let cfg = RustConfig::default();
        assert_eq!(
            cfg.derive_attr(),
            "#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]"
        );
    }

    #[test]
    fn test_rust_config_derive_hash() {
        let cfg = RustConfig {
            derive_hash: true,
            ..Default::default()
        };
        let attr = cfg.derive_attr();
        assert!(attr.contains("Hash"));
    }
}

// CODEGEN-END
