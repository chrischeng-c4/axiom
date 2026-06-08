//! Go type inference: structs, interfaces, channels, generics, composite
//! literals, and type assertion tracking.

use crate::syntax::ParsedFile;
use std::collections::HashMap;

/// Direction of a Go channel
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelDirection {
    Send,
    Receive,
    Bidirectional,
}

/// A Go type representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoType {
    /// Primitive types: int, string, bool, etc.
    Primitive(String),
    /// Named type (struct, interface, alias)
    Named(String),
    /// Pointer type (*T)
    Pointer(Box<GoType>),
    /// Slice type ([]T)
    Slice(Box<GoType>),
    /// Array type ([N]T)
    Array(Box<GoType>, usize),
    /// Map type (map[K]V)
    Map(Box<GoType>, Box<GoType>),
    /// Channel type (chan T, chan<- T, <-chan T)
    Channel {
        element: Box<GoType>,
        direction: ChannelDirection,
    },
    /// Function type
    Func {
        params: Vec<GoType>,
        returns: Vec<GoType>,
    },
    /// Interface type with required method names
    Interface { methods: Vec<String> },
    /// Struct type with field names and types
    Struct { fields: Vec<(String, GoType)> },
    /// Generic instantiation: TypeName[T1, T2]
    Generic(String, Vec<GoType>),
    /// Unknown / unresolved
    Unknown,
}

/// A generic type parameter (Go 1.18+)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericParam {
    pub name: String,
    pub constraint: Option<String>,
}

/// Information about a method defined on a type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodInfo {
    pub name: String,
    pub receiver_type: String,
    pub is_pointer_receiver: bool,
}

/// A recorded type assertion (x.(Type))
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAssertion {
    pub variable: String,
    pub asserted_type: String,
    pub line: u32,
}

/// Go type inference engine: walks a parsed Go file and extracts type
/// information (generics, method sets, channels, composite literals, assertions).
#[derive(Debug)]
pub struct GoTypeInference {
    /// All discovered types: type name -> GoType
    pub types: HashMap<String, GoType>,
    /// Methods defined on types: receiver type name -> Vec<MethodInfo>
    pub methods: HashMap<String, Vec<MethodInfo>>,
    /// Generic parameters per type: type name -> Vec<GenericParam>
    pub generic_params: HashMap<String, Vec<GenericParam>>,
    /// Recorded type assertions for later validation
    pub type_assertions: Vec<TypeAssertion>,
    /// Inferred types from composite literals: variable name -> type name
    pub composite_literals: HashMap<String, String>,
}

impl GoTypeInference {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            methods: HashMap::new(),
            generic_params: HashMap::new(),
            type_assertions: Vec::new(),
            composite_literals: HashMap::new(),
        }
    }

    /// Run type inference on a parsed Go file
    pub fn infer(&mut self, file: &ParsedFile) {
        let root = file.root_node();
        self.visit_node(&root, file);
    }

    /// Collect all method names defined on a type (value and pointer receivers)
    pub fn collect_method_set(&self, type_name: &str) -> Vec<String> {
        self.methods
            .get(type_name)
            .map(|methods| methods.iter().map(|m| m.name.clone()).collect())
            .unwrap_or_default()
    }

    fn visit_node(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if node.is_error() || node.is_missing() {
            return;
        }

        match node.kind() {
            "type_declaration" => {
                self.visit_type_declaration(node, file);
            }
            "method_declaration" => {
                self.visit_method_declaration(node, file);
            }
            "short_var_declaration" | "assignment_statement" => {
                self.visit_assignment(node, file);
            }
            "expression_statement" => {
                self.visit_expression_statement(node, file);
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if !child.is_error() && !child.is_missing() {
                self.visit_node(&child, file);
            }
        }
    }

    fn visit_type_declaration(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_spec" {
                self.visit_type_spec(&child, file);
            }
        }
    }

    fn visit_type_spec(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let name = node
            .child_by_field_name("name")
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();

        if name.is_empty() {
            return;
        }

        // Extract generic type parameters if present
        self.extract_generic_params(node, file, &name);

        // Parse the type definition
        let type_node = node.child_by_field_name("type");
        if let Some(tn) = type_node {
            let go_type = self.parse_type_node(&tn, file);
            self.types.insert(name, go_type);
        }
    }

    fn extract_generic_params(
        &mut self,
        node: &tree_sitter::Node<'_>,
        file: &ParsedFile,
        type_name: &str,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_parameter_list" {
                let params = self.parse_type_parameter_list(&child, file);
                if !params.is_empty() {
                    self.generic_params.insert(type_name.to_string(), params);
                }
                return;
            }
        }
    }

    fn parse_type_parameter_list(
        &self,
        node: &tree_sitter::Node<'_>,
        file: &ParsedFile,
    ) -> Vec<GenericParam> {
        let mut params = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            // type_parameter_declaration contains name + constraint
            if child.kind() == "type_parameter_declaration" {
                let name = child
                    .child_by_field_name("name")
                    .map(|n| file.node_text(&n).to_string())
                    .unwrap_or_default();
                let constraint = child
                    .child_by_field_name("type")
                    .map(|n| file.node_text(&n).to_string());

                if !name.is_empty() {
                    params.push(GenericParam { name, constraint });
                }
            }
        }
        params
    }

    fn parse_type_node(&self, node: &tree_sitter::Node<'_>, file: &ParsedFile) -> GoType {
        match node.kind() {
            "struct_type" => self.parse_struct_type(node, file),
            "interface_type" => self.parse_interface_type(node, file),
            "channel_type" => self.parse_channel_type(node, file),
            "pointer_type" => {
                if let Some(inner) = node.child(1) {
                    GoType::Pointer(Box::new(self.parse_type_node(&inner, file)))
                } else {
                    GoType::Pointer(Box::new(GoType::Unknown))
                }
            }
            "slice_type" => {
                if let Some(elem) = node.child_by_field_name("element") {
                    GoType::Slice(Box::new(self.parse_type_node(&elem, file)))
                } else {
                    GoType::Slice(Box::new(GoType::Unknown))
                }
            }
            "map_type" => {
                let key = node
                    .child_by_field_name("key")
                    .map(|n| self.parse_type_node(&n, file))
                    .unwrap_or(GoType::Unknown);
                let value = node
                    .child_by_field_name("value")
                    .map(|n| self.parse_type_node(&n, file))
                    .unwrap_or(GoType::Unknown);
                GoType::Map(Box::new(key), Box::new(value))
            }
            "type_identifier" | "qualified_type" => {
                let text = file.node_text(node).to_string();
                parse_primitive_or_named(&text)
            }
            _ => {
                let text = file.node_text(node).trim().to_string();
                if text.is_empty() {
                    GoType::Unknown
                } else {
                    parse_primitive_or_named(&text)
                }
            }
        }
    }

    fn parse_struct_type(&self, node: &tree_sitter::Node<'_>, file: &ParsedFile) -> GoType {
        let mut fields = Vec::new();
        // struct_type -> field_declaration_list -> field_declaration*
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "field_declaration_list" {
                let mut inner = child.walk();
                for field_node in child.children(&mut inner) {
                    if field_node.kind() == "field_declaration" {
                        let fname = field_node
                            .child_by_field_name("name")
                            .map(|n| file.node_text(&n).to_string())
                            .unwrap_or_default();
                        let ftype = field_node
                            .child_by_field_name("type")
                            .map(|n| self.parse_type_node(&n, file))
                            .unwrap_or(GoType::Unknown);
                        if !fname.is_empty() {
                            fields.push((fname, ftype));
                        }
                    }
                }
                break;
            }
        }
        GoType::Struct { fields }
    }

    fn parse_interface_type(&self, node: &tree_sitter::Node<'_>, file: &ParsedFile) -> GoType {
        let mut methods = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            // tree-sitter-go 0.25 uses "method_elem" for interface methods
            if child.kind() == "method_elem" || child.kind() == "method_spec" {
                let name = child
                    .child_by_field_name("name")
                    .map(|n| file.node_text(&n).to_string())
                    .unwrap_or_default();
                if !name.is_empty() {
                    methods.push(name);
                }
            }
        }
        GoType::Interface { methods }
    }

    fn parse_channel_type(&self, node: &tree_sitter::Node<'_>, file: &ParsedFile) -> GoType {
        let text = file.node_text(node);
        let direction = if text.starts_with("<-chan") {
            ChannelDirection::Receive
        } else if text.starts_with("chan<-") {
            ChannelDirection::Send
        } else {
            ChannelDirection::Bidirectional
        };

        // The element type is the last child (the value type after chan keyword)
        let element = node
            .child_by_field_name("value")
            .map(|n| self.parse_type_node(&n, file))
            .unwrap_or_else(|| {
                // Fallback: try last named child
                let count = node.named_child_count();
                if count > 0 {
                    node.named_child(count - 1)
                        .map(|n| self.parse_type_node(&n, file))
                        .unwrap_or(GoType::Unknown)
                } else {
                    GoType::Unknown
                }
            });

        GoType::Channel {
            element: Box::new(element),
            direction,
        }
    }

    fn visit_method_declaration(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let name = node
            .child_by_field_name("name")
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();

        if name.is_empty() {
            return;
        }

        // Extract receiver type and pointer-ness
        if let Some(receiver) = node.child_by_field_name("receiver") {
            let (recv_type, is_pointer) = self.extract_receiver_info(&receiver, file);
            if !recv_type.is_empty() {
                let info = MethodInfo {
                    name,
                    receiver_type: recv_type.clone(),
                    is_pointer_receiver: is_pointer,
                };
                self.methods.entry(recv_type).or_default().push(info);
            }
        }
    }

    fn extract_receiver_info(
        &self,
        receiver: &tree_sitter::Node<'_>,
        file: &ParsedFile,
    ) -> (String, bool) {
        let text = file.node_text(receiver);
        // Receiver looks like (s *Server) or (s Server)
        let is_pointer = text.contains('*');
        // Extract the type name from the receiver text
        let type_name = text
            .trim_matches(|c: char| c == '(' || c == ')')
            .split_whitespace()
            .last()
            .unwrap_or("")
            .trim_start_matches('*')
            .to_string();
        (type_name, is_pointer)
    }

    /// Visit assignments to detect composite literals
    fn visit_assignment(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        // Look for: x := TypeName{...} or x = TypeName{...}
        let lhs = node.child_by_field_name("left");
        let rhs = node.child_by_field_name("right");
        if let (Some(lhs_node), Some(rhs_node)) = (lhs, rhs) {
            let var_name = file.node_text(&lhs_node).to_string();
            self.try_extract_composite_literal(&var_name, &rhs_node, file);
        }
    }

    fn visit_expression_statement(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        // Look for type assertions: x.(Type)
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_assertion_expression" {
                self.visit_type_assertion(&child, file);
            }
        }
    }

    fn try_extract_composite_literal(
        &mut self,
        var_name: &str,
        node: &tree_sitter::Node<'_>,
        file: &ParsedFile,
    ) {
        if node.kind() == "composite_literal" {
            // The type name is the first child (before the literal body)
            if let Some(type_node) = node.child_by_field_name("type") {
                let type_name = file.node_text(&type_node).to_string();
                if !type_name.is_empty() && !var_name.is_empty() {
                    self.composite_literals
                        .insert(var_name.to_string(), type_name);
                }
            }
        }
    }

    fn visit_type_assertion(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        // type_assertion_expression: operand "." "(" type ")"
        let operand = node
            .child_by_field_name("operand")
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();
        let asserted = node
            .child_by_field_name("type")
            .map(|n| file.node_text(&n).to_string())
            .unwrap_or_default();

        if !operand.is_empty() && !asserted.is_empty() {
            self.type_assertions.push(TypeAssertion {
                variable: operand,
                asserted_type: asserted,
                line: node.start_position().row as u32,
            });
        }
    }
}

impl Default for GoTypeInference {
    fn default() -> Self {
        Self::new()
    }
}

/// Classify a type string as primitive or named
fn parse_primitive_or_named(type_str: &str) -> GoType {
    match type_str {
        "int" | "int8" | "int16" | "int32" | "int64" | "uint" | "uint8" | "uint16" | "uint32"
        | "uint64" | "uintptr" | "float32" | "float64" | "complex64" | "complex128" | "bool"
        | "byte" | "rune" | "string" => GoType::Primitive(type_str.to_string()),
        _ => GoType::Named(type_str.to_string()),
    }
}
