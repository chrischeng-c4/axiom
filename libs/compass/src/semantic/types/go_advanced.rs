//! Advanced Go type analysis
//!
//! Provides higher-level analysis built on top of GoTypeInference:
//! - Interface satisfaction checking
//! - Cross-type method set comparison
//! - Type assertion validation

use super::go::{GoType, GoTypeInference};

/// Result of checking whether a struct satisfies an interface
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SatisfactionResult {
    pub struct_name: String,
    pub interface_name: String,
    pub satisfied: bool,
    /// Method names required by the interface but missing from the struct
    pub missing_methods: Vec<String>,
}

/// Check which structs satisfy which interfaces based on method name matching.
///
/// Given a `GoTypeInference` that has already been populated (via `infer()`),
/// this function iterates over all struct and interface types and checks whether
/// each struct's method set includes all methods required by each interface.
pub fn check_interface_satisfaction(inference: &GoTypeInference) -> Vec<SatisfactionResult> {
    let mut results = Vec::new();

    // Collect interface definitions (name -> required method names)
    let mut interfaces: Vec<(String, Vec<String>)> = Vec::new();
    for (name, ty) in &inference.types {
        if let GoType::Interface { methods } = ty {
            interfaces.push((name.clone(), methods.clone()));
        }
    }

    // Collect struct names
    let mut struct_names: Vec<String> = Vec::new();
    for (name, ty) in &inference.types {
        if matches!(ty, GoType::Struct { .. }) {
            struct_names.push(name.clone());
        }
    }

    // For each struct, check against each interface
    for struct_name in &struct_names {
        let method_set = inference.collect_method_set(struct_name);

        for (iface_name, required_methods) in &interfaces {
            let missing: Vec<String> = required_methods
                .iter()
                .filter(|m| !method_set.contains(m))
                .cloned()
                .collect();

            results.push(SatisfactionResult {
                struct_name: struct_name.clone(),
                interface_name: iface_name.clone(),
                satisfied: missing.is_empty(),
                missing_methods: missing,
            });
        }
    }

    results
}

/// Validate recorded type assertions against known types.
///
/// Returns a list of (assertion_index, is_valid) pairs.
/// An assertion is considered valid if the asserted type exists in
/// the inference's type map.
pub fn validate_type_assertions(inference: &GoTypeInference) -> Vec<(usize, bool)> {
    inference
        .type_assertions
        .iter()
        .enumerate()
        .map(|(i, assertion)| {
            let exists = inference.types.contains_key(&assertion.asserted_type)
                || is_builtin_type(&assertion.asserted_type);
            (i, exists)
        })
        .collect()
}

/// Check if a type name is a Go builtin
fn is_builtin_type(name: &str) -> bool {
    matches!(
        name,
        "int"
            | "int8"
            | "int16"
            | "int32"
            | "int64"
            | "uint"
            | "uint8"
            | "uint16"
            | "uint32"
            | "uint64"
            | "uintptr"
            | "float32"
            | "float64"
            | "complex64"
            | "complex128"
            | "bool"
            | "byte"
            | "rune"
            | "string"
            | "error"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::types::go::{GoTypeInference, MethodInfo, TypeAssertion};

    fn build_inference_with_types() -> GoTypeInference {
        let mut inf = GoTypeInference::new();

        // Define a struct
        inf.types.insert(
            "Server".to_string(),
            GoType::Struct {
                fields: vec![
                    ("Host".to_string(), GoType::Primitive("string".to_string())),
                    ("Port".to_string(), GoType::Primitive("int".to_string())),
                ],
            },
        );

        // Define methods on Server
        inf.methods.insert(
            "Server".to_string(),
            vec![
                MethodInfo {
                    name: "Start".to_string(),
                    receiver_type: "Server".to_string(),
                    is_pointer_receiver: true,
                },
                MethodInfo {
                    name: "Stop".to_string(),
                    receiver_type: "Server".to_string(),
                    is_pointer_receiver: true,
                },
                MethodInfo {
                    name: "Status".to_string(),
                    receiver_type: "Server".to_string(),
                    is_pointer_receiver: false,
                },
            ],
        );

        // Define an interface that Server satisfies
        inf.types.insert(
            "Service".to_string(),
            GoType::Interface {
                methods: vec!["Start".to_string(), "Stop".to_string()],
            },
        );

        // Define an interface that Server does NOT satisfy
        inf.types.insert(
            "Persistent".to_string(),
            GoType::Interface {
                methods: vec!["Save".to_string(), "Load".to_string()],
            },
        );

        inf
    }

    #[test]
    fn test_interface_satisfaction_passes() {
        let inf = build_inference_with_types();
        let results = check_interface_satisfaction(&inf);

        let service_result = results
            .iter()
            .find(|r| r.struct_name == "Server" && r.interface_name == "Service")
            .unwrap();

        assert!(service_result.satisfied);
        assert!(service_result.missing_methods.is_empty());
    }

    #[test]
    fn test_interface_satisfaction_fails() {
        let inf = build_inference_with_types();
        let results = check_interface_satisfaction(&inf);

        let persistent_result = results
            .iter()
            .find(|r| r.struct_name == "Server" && r.interface_name == "Persistent")
            .unwrap();

        assert!(!persistent_result.satisfied);
        assert_eq!(persistent_result.missing_methods.len(), 2);
        assert!(persistent_result
            .missing_methods
            .contains(&"Save".to_string()));
        assert!(persistent_result
            .missing_methods
            .contains(&"Load".to_string()));
    }

    #[test]
    fn test_partial_interface_satisfaction() {
        let mut inf = GoTypeInference::new();

        inf.types
            .insert("Writer".to_string(), GoType::Struct { fields: vec![] });
        inf.methods.insert(
            "Writer".to_string(),
            vec![MethodInfo {
                name: "Write".to_string(),
                receiver_type: "Writer".to_string(),
                is_pointer_receiver: false,
            }],
        );
        inf.types.insert(
            "ReadWriter".to_string(),
            GoType::Interface {
                methods: vec!["Read".to_string(), "Write".to_string()],
            },
        );

        let results = check_interface_satisfaction(&inf);
        let result = results
            .iter()
            .find(|r| r.struct_name == "Writer" && r.interface_name == "ReadWriter")
            .unwrap();

        assert!(!result.satisfied);
        assert_eq!(result.missing_methods, vec!["Read".to_string()]);
    }

    #[test]
    fn test_empty_interface_always_satisfied() {
        let mut inf = GoTypeInference::new();

        inf.types
            .insert("Anything".to_string(), GoType::Struct { fields: vec![] });
        inf.types
            .insert("Empty".to_string(), GoType::Interface { methods: vec![] });

        let results = check_interface_satisfaction(&inf);
        assert_eq!(results.len(), 1);
        assert!(results[0].satisfied);
    }

    #[test]
    fn test_no_structs_no_results() {
        let mut inf = GoTypeInference::new();
        inf.types.insert(
            "Reader".to_string(),
            GoType::Interface {
                methods: vec!["Read".to_string()],
            },
        );
        let results = check_interface_satisfaction(&inf);
        assert!(results.is_empty());
    }

    #[test]
    fn test_validate_type_assertions_builtin() {
        let mut inf = GoTypeInference::new();
        inf.type_assertions.push(TypeAssertion {
            variable: "x".to_string(),
            asserted_type: "string".to_string(),
            line: 5,
        });
        let results = validate_type_assertions(&inf);
        assert_eq!(results.len(), 1);
        assert!(results[0].1); // string is builtin, valid
    }

    #[test]
    fn test_validate_type_assertions_named() {
        let mut inf = GoTypeInference::new();
        inf.types
            .insert("MyType".to_string(), GoType::Struct { fields: vec![] });
        inf.type_assertions.push(TypeAssertion {
            variable: "x".to_string(),
            asserted_type: "MyType".to_string(),
            line: 10,
        });
        inf.type_assertions.push(TypeAssertion {
            variable: "y".to_string(),
            asserted_type: "UnknownType".to_string(),
            line: 15,
        });

        let results = validate_type_assertions(&inf);
        assert_eq!(results.len(), 2);
        assert!(results[0].1); // MyType exists
        assert!(!results[1].1); // UnknownType does not exist
    }

    #[test]
    fn test_interface_satisfaction_with_tree_sitter() {
        use crate::syntax::{Language, MultiParser};

        let source = r#"
package main

type Animal interface {
    Speak() string
    Name() string
}

type Dog struct {
    name string
}

func (d Dog) Speak() string {
    return "Woof"
}

func (d Dog) Name() string {
    return d.name
}

type Cat struct {
    name string
}

func (c Cat) Speak() string {
    return "Meow"
}
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Go).unwrap();
        let mut inf = GoTypeInference::new();
        inf.infer(&parsed);

        let results = check_interface_satisfaction(&inf);

        // Dog should satisfy Animal
        let dog_result = results
            .iter()
            .find(|r| r.struct_name == "Dog" && r.interface_name == "Animal")
            .unwrap();
        assert!(
            dog_result.satisfied,
            "Dog should satisfy Animal, missing: {:?}",
            dog_result.missing_methods
        );

        // Cat should NOT satisfy Animal (missing Name)
        let cat_result = results
            .iter()
            .find(|r| r.struct_name == "Cat" && r.interface_name == "Animal")
            .unwrap();
        assert!(!cat_result.satisfied);
        assert_eq!(cat_result.missing_methods, vec!["Name".to_string()]);
    }

    #[test]
    fn test_composite_literal_tracking() {
        let mut inf = GoTypeInference::new();
        inf.composite_literals
            .insert("srv".to_string(), "Server".to_string());
        assert_eq!(
            inf.composite_literals.get("srv"),
            Some(&"Server".to_string())
        );
    }

    #[test]
    fn test_method_set_value_and_pointer_receivers() {
        let mut inf = GoTypeInference::new();
        inf.methods.insert(
            "Buf".to_string(),
            vec![
                MethodInfo {
                    name: "Len".to_string(),
                    receiver_type: "Buf".to_string(),
                    is_pointer_receiver: false,
                },
                MethodInfo {
                    name: "Reset".to_string(),
                    receiver_type: "Buf".to_string(),
                    is_pointer_receiver: true,
                },
                MethodInfo {
                    name: "Write".to_string(),
                    receiver_type: "Buf".to_string(),
                    is_pointer_receiver: true,
                },
            ],
        );
        let set = inf.collect_method_set("Buf");
        assert_eq!(set.len(), 3);
        assert!(set.contains(&"Len".to_string()));
        assert!(set.contains(&"Reset".to_string()));
        assert!(set.contains(&"Write".to_string()));
    }
}
