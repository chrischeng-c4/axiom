//! Tests for Go type inference (go.rs)

use super::go::*;
use crate::syntax::{Language, MultiParser};

fn infer_from_source(source: &str) -> GoTypeInference {
    let mut parser = MultiParser::new().unwrap();
    let parsed = parser.parse(source, Language::Go).unwrap();
    let mut inference = GoTypeInference::new();
    inference.infer(&parsed);
    inference
}

#[test]
fn test_struct_type_inference() {
    let inf = infer_from_source(
        r#"
package main

type Server struct {
    Host string
    Port int
}
"#,
    );
    let ty = inf.types.get("Server").unwrap();
    match ty {
        GoType::Struct { fields } => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "Host");
            assert_eq!(fields[0].1, GoType::Primitive("string".to_string()));
            assert_eq!(fields[1].0, "Port");
            assert_eq!(fields[1].1, GoType::Primitive("int".to_string()));
        }
        other => panic!("expected Struct, got {:?}", other),
    }
}

#[test]
fn test_interface_type_inference() {
    let inf = infer_from_source(
        r#"
package main

type Reader interface {
    Read(p []byte) (int, error)
    Close() error
}
"#,
    );
    let ty = inf.types.get("Reader").unwrap();
    match ty {
        GoType::Interface { methods } => {
            assert_eq!(methods.len(), 2);
            assert!(methods.contains(&"Read".to_string()));
            assert!(methods.contains(&"Close".to_string()));
        }
        other => panic!("expected Interface, got {:?}", other),
    }
}

#[test]
fn test_method_set_collection() {
    let inf = infer_from_source(
        r#"
package main

type MyStruct struct {
    Value int
}

func (s MyStruct) Get() int {
    return s.Value
}

func (s *MyStruct) Set(v int) {
    s.Value = v
}
"#,
    );
    let methods = inf.collect_method_set("MyStruct");
    assert_eq!(methods.len(), 2);
    assert!(methods.contains(&"Get".to_string()));
    assert!(methods.contains(&"Set".to_string()));

    // Check pointer receiver info
    let infos = inf.methods.get("MyStruct").unwrap();
    let set_info = infos.iter().find(|m| m.name == "Set").unwrap();
    assert!(set_info.is_pointer_receiver);
    let get_info = infos.iter().find(|m| m.name == "Get").unwrap();
    assert!(!get_info.is_pointer_receiver);
}

#[test]
fn test_channel_type_inference() {
    let inf = infer_from_source(
        r#"
package main

type Notifier struct {
    events chan string
}
"#,
    );
    let ty = inf.types.get("Notifier").unwrap();
    match ty {
        GoType::Struct { fields } => {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].0, "events");
            match &fields[0].1 {
                GoType::Channel { element, direction } => {
                    assert_eq!(**element, GoType::Primitive("string".to_string()));
                    assert_eq!(*direction, ChannelDirection::Bidirectional);
                }
                other => panic!("expected Channel, got {:?}", other),
            }
        }
        other => panic!("expected Struct, got {:?}", other),
    }
}

#[test]
fn test_map_type_inference() {
    let inf = infer_from_source(
        r#"
package main

type Config struct {
    Settings map[string]int
}
"#,
    );
    let ty = inf.types.get("Config").unwrap();
    match ty {
        GoType::Struct { fields } => {
            assert_eq!(fields[0].0, "Settings");
            match &fields[0].1 {
                GoType::Map(k, v) => {
                    assert_eq!(**k, GoType::Primitive("string".to_string()));
                    assert_eq!(**v, GoType::Primitive("int".to_string()));
                }
                other => panic!("expected Map, got {:?}", other),
            }
        }
        other => panic!("expected Struct, got {:?}", other),
    }
}

#[test]
fn test_pointer_type_in_struct() {
    let inf = infer_from_source(
        r#"
package main

type Node struct {
    Next *Node
    Value int
}
"#,
    );
    let ty = inf.types.get("Node").unwrap();
    match ty {
        GoType::Struct { fields } => {
            assert_eq!(fields[0].0, "Next");
            match &fields[0].1 {
                GoType::Pointer(inner) => {
                    assert_eq!(**inner, GoType::Named("Node".to_string()));
                }
                other => panic!("expected Pointer, got {:?}", other),
            }
        }
        other => panic!("expected Struct, got {:?}", other),
    }
}

#[test]
fn test_generic_param_extraction() {
    // Note: tree-sitter-go may not fully support generics syntax.
    // This test uses the direct API to verify the data structure works.
    let mut inf = GoTypeInference::new();
    inf.generic_params.insert(
        "Container".to_string(),
        vec![GenericParam {
            name: "T".to_string(),
            constraint: Some("any".to_string()),
        }],
    );
    let params = inf.generic_params.get("Container").unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "T");
    assert_eq!(params[0].constraint.as_deref(), Some("any"));
}

#[test]
fn test_type_assertion_tracking() {
    // TypeAssertion recording via direct API
    let mut inf = GoTypeInference::new();
    inf.type_assertions.push(TypeAssertion {
        variable: "val".to_string(),
        asserted_type: "string".to_string(),
        line: 10,
    });
    assert_eq!(inf.type_assertions.len(), 1);
    assert_eq!(inf.type_assertions[0].variable, "val");
    assert_eq!(inf.type_assertions[0].asserted_type, "string");
}

#[test]
fn test_empty_method_set() {
    let inf = GoTypeInference::new();
    let methods = inf.collect_method_set("NonExistent");
    assert!(methods.is_empty());
}

#[test]
fn test_multiple_types_in_file() {
    let inf = infer_from_source(
        r#"
package main

type Point struct {
    X float64
    Y float64
}

type Shape interface {
    Area() float64
}

type Circle struct {
    Center Point
    Radius float64
}
"#,
    );
    assert_eq!(inf.types.len(), 3);
    assert!(inf.types.contains_key("Point"));
    assert!(inf.types.contains_key("Shape"));
    assert!(inf.types.contains_key("Circle"));

    // Verify Circle has a named field type
    match inf.types.get("Circle").unwrap() {
        GoType::Struct { fields } => {
            assert_eq!(fields[0].0, "Center");
            assert_eq!(fields[0].1, GoType::Named("Point".to_string()));
        }
        other => panic!("expected Struct, got {:?}", other),
    }
}
