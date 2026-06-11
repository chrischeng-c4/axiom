//! Mermaid diagram parser
//!
//! Parses Mermaid diagrams into SpecIR structures.

use crate::spec::ir::{
    ControlFlowSpec, DataModelSpec, FieldConstraints, FieldDef, FlowEdge, FlowNode, FlowNodeType,
    MethodDef, ModelDef, ParamDef, RelationType, Relationship, StateDef, StateMachineSpec,
    TransitionDef, Visibility,
};
use crate::type_inference::Type;

/// Error type for Mermaid parsing
#[derive(Debug)]
pub enum MermaidError {
    /// Unknown diagram type
    UnknownDiagramType(String),
    /// Invalid syntax
    SyntaxError(String),
    /// Missing required element
    MissingElement(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for MermaidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MermaidError::UnknownDiagramType(s) => write!(f, "unknown diagram type: {}", s),
            MermaidError::SyntaxError(s) => write!(f, "syntax error: {}", s),
            MermaidError::MissingElement(s) => write!(f, "missing element: {}", s),
            MermaidError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for MermaidError {}

/// Detected diagram type
#[derive(Debug, Clone, PartialEq)]
pub enum DiagramType {
    ClassDiagram,
    SequenceDiagram,
    StateDiagram,
    Flowchart,
    ErDiagram,
}

/// Parsed Mermaid result
#[derive(Debug)]
pub enum MermaidSpec {
    DataModel(DataModelSpec),
    StateMachine(StateMachineSpec),
    ControlFlow(ControlFlowSpec),
}

/// Mermaid diagram parser
pub struct MermaidParser;

impl MermaidParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse Mermaid diagram and detect type automatically
    pub fn parse(&self, content: &str) -> Result<MermaidSpec, MermaidError> {
        let content = content.trim();
        let diagram_type = self.detect_diagram_type(content)?;

        match diagram_type {
            DiagramType::ClassDiagram => {
                let spec = self.parse_class_diagram(content)?;
                Ok(MermaidSpec::DataModel(spec))
            }
            DiagramType::StateDiagram => {
                let spec = self.parse_state_diagram(content)?;
                Ok(MermaidSpec::StateMachine(spec))
            }
            DiagramType::Flowchart => {
                let spec = self.parse_flowchart(content)?;
                Ok(MermaidSpec::ControlFlow(spec))
            }
            DiagramType::ErDiagram => {
                let spec = self.parse_er_diagram(content)?;
                Ok(MermaidSpec::DataModel(spec))
            }
            DiagramType::SequenceDiagram => {
                // Sequence diagrams don't map directly to our IR
                // Return empty data model for now
                Ok(MermaidSpec::DataModel(DataModelSpec::default()))
            }
        }
    }

    /// Detect diagram type from content
    fn detect_diagram_type(&self, content: &str) -> Result<DiagramType, MermaidError> {
        let first_line = content.lines().next().unwrap_or("").trim().to_lowercase();

        if first_line.starts_with("classdiagram") {
            Ok(DiagramType::ClassDiagram)
        } else if first_line.starts_with("sequencediagram") {
            Ok(DiagramType::SequenceDiagram)
        } else if first_line.starts_with("statediagram") {
            Ok(DiagramType::StateDiagram)
        } else if first_line.starts_with("flowchart") || first_line.starts_with("graph") {
            Ok(DiagramType::Flowchart)
        } else if first_line.starts_with("erdiagram") {
            Ok(DiagramType::ErDiagram)
        } else {
            Err(MermaidError::UnknownDiagramType(first_line))
        }
    }

    /// Parse classDiagram
    fn parse_class_diagram(&self, content: &str) -> Result<DataModelSpec, MermaidError> {
        let mut spec = DataModelSpec::new();
        let mut current_class: Option<ModelDef> = None;

        for line in content.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() || line.starts_with("%%") {
                continue;
            }

            // Class definition: class ClassName {
            if line.starts_with("class ") {
                // Save previous class
                if let Some(model) = current_class.take() {
                    spec.add_model(model);
                }

                let class_name = self.extract_class_name(line)?;
                current_class = Some(ModelDef {
                    name: class_name,
                    ..Default::default()
                });
            }
            // End of class block
            else if line == "}" {
                if let Some(model) = current_class.take() {
                    spec.add_model(model);
                }
            }
            // Member inside class
            else if let Some(ref mut model) = current_class {
                if let Some(member) = self.parse_class_member(line)? {
                    match member {
                        ClassMember::Field(field) => model.fields.push(field),
                        ClassMember::Method(method) => model.methods.push(method),
                    }
                }
            }
            // Relationship: ClassA --|> ClassB
            else if self.is_relationship(line) {
                if let Some(rel) = self.parse_relationship(line)? {
                    spec.relationships.push(rel);
                }
            }
        }

        // Save last class if not closed
        if let Some(model) = current_class {
            spec.add_model(model);
        }

        Ok(spec)
    }

    /// Extract class name from "class ClassName" or "class ClassName {"
    fn extract_class_name(&self, line: &str) -> Result<String, MermaidError> {
        let without_class = line.strip_prefix("class ").unwrap_or(line);
        let name = without_class
            .split(|c: char| c == '{' || c == ':' || c.is_whitespace())
            .next()
            .ok_or_else(|| MermaidError::SyntaxError("Invalid class definition".into()))?;
        Ok(name.trim().to_string())
    }

    /// Parse class member (field or method)
    fn parse_class_member(&self, line: &str) -> Result<Option<ClassMember>, MermaidError> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(None);
        }

        // Determine visibility
        let (visibility, rest) = if line.starts_with('+') {
            (Visibility::Public, &line[1..])
        } else if line.starts_with('-') {
            (Visibility::Private, &line[1..])
        } else if line.starts_with('#') {
            (Visibility::Protected, &line[1..])
        } else if line.starts_with('~') {
            // Package/internal visibility - map to Protected
            (Visibility::Protected, &line[1..])
        } else {
            (Visibility::Public, line)
        };

        let rest = rest.trim();

        // Check if it's a method (contains parentheses)
        if rest.contains('(') {
            let method = self.parse_method(rest, visibility)?;
            Ok(Some(ClassMember::Method(method)))
        } else {
            // It's a field: type name or name: type
            let field = self.parse_field(rest, visibility)?;
            Ok(Some(ClassMember::Field(field)))
        }
    }

    /// Parse method definition
    fn parse_method(&self, line: &str, visibility: Visibility) -> Result<MethodDef, MermaidError> {
        // Format: methodName(params) returnType or methodName(params): returnType
        let paren_start = line.find('(').unwrap();
        let paren_end = line.find(')').unwrap_or(line.len());

        let name = line[..paren_start].trim().to_string();
        let params_str = &line[paren_start + 1..paren_end];

        // Parse parameters
        let params: Vec<ParamDef> = params_str
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|p| {
                let p = p.trim();
                // Format: name: Type or Type name
                if let Some(colon) = p.find(':') {
                    ParamDef {
                        name: p[..colon].trim().to_string(),
                        ty: self.parse_type(&p[colon + 1..]),
                        default: None,
                    }
                } else {
                    let parts: Vec<&str> = p.split_whitespace().collect();
                    if parts.len() >= 2 {
                        ParamDef {
                            name: parts[1].to_string(),
                            ty: self.parse_type(parts[0]),
                            default: None,
                        }
                    } else {
                        ParamDef {
                            name: parts.get(0).unwrap_or(&"arg").to_string(),
                            ty: Type::Any,
                            default: None,
                        }
                    }
                }
            })
            .collect();

        // Parse return type
        let return_type = if let Some(colon) = line[paren_end..].find(':') {
            self.parse_type(&line[paren_end + colon + 1..])
        } else if line.len() > paren_end + 1 {
            self.parse_type(&line[paren_end + 1..])
        } else {
            Type::None
        };

        Ok(MethodDef {
            name,
            params,
            return_type,
            visibility,
            is_static: false,
            is_async: false,
            description: None,
        })
    }

    /// Parse field definition
    fn parse_field(&self, line: &str, visibility: Visibility) -> Result<FieldDef, MermaidError> {
        // Format: Type name or name: Type
        let (name, ty) = if let Some(colon) = line.find(':') {
            let n = line[..colon].trim();
            let t = line[colon + 1..].trim();
            (n.to_string(), self.parse_type(t))
        } else {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                (parts[1].to_string(), self.parse_type(parts[0]))
            } else {
                (parts.get(0).unwrap_or(&"field").to_string(), Type::Any)
            }
        };

        let required = matches!(visibility, Visibility::Public);

        Ok(FieldDef {
            name,
            ty,
            required,
            default: None,
            description: None,
            constraints: FieldConstraints::default(),
            column_name: None,
            primary_key: false,
            unique: false,
            indexed: false,
            foreign_key: None,
            alias: None,
        })
    }

    /// Parse type string to Type
    fn parse_type(&self, s: &str) -> Type {
        let s = s.trim();
        match s.to_lowercase().as_str() {
            "string" | "str" => Type::Str,
            "int" | "integer" | "i32" | "i64" => Type::Int,
            "float" | "double" | "f32" | "f64" | "number" => Type::Float,
            "bool" | "boolean" => Type::Bool,
            "void" | "none" | "()" => Type::None,
            "any" => Type::Any,
            _ => {
                // Check for generic types like List<T>
                if let Some(bracket_start) = s.find('<') {
                    let base = &s[..bracket_start];
                    let inner = &s[bracket_start + 1..s.len() - 1];
                    match base.to_lowercase().as_str() {
                        "list" | "array" | "vec" => Type::List(Box::new(self.parse_type(inner))),
                        "optional" | "option" => Type::Optional(Box::new(self.parse_type(inner))),
                        _ => Type::Instance {
                            name: s.to_string(),
                            module: None,
                            type_args: vec![],
                        },
                    }
                } else if s.ends_with("[]") {
                    let inner = &s[..s.len() - 2];
                    Type::List(Box::new(self.parse_type(inner)))
                } else if s.ends_with('?') {
                    let inner = &s[..s.len() - 1];
                    Type::Optional(Box::new(self.parse_type(inner)))
                } else {
                    Type::Instance {
                        name: s.to_string(),
                        module: None,
                        type_args: vec![],
                    }
                }
            }
        }
    }

    /// Check if line is a relationship
    fn is_relationship(&self, line: &str) -> bool {
        line.contains("--|>")
            || line.contains("<|--")
            || line.contains("--*")
            || line.contains("*--")
            || line.contains("--o")
            || line.contains("o--")
            || line.contains("-->")
            || line.contains("<--")
            || line.contains("--") && !line.starts_with("class ")
    }

    /// Parse relationship line
    fn parse_relationship(&self, line: &str) -> Result<Option<Relationship>, MermaidError> {
        // Find relationship marker
        let (from, to, rel_type) = if line.contains("--|>") {
            let parts: Vec<&str> = line.split("--|>").collect();
            (parts[0].trim(), parts[1].trim(), RelationType::OneToOne) // Inheritance
        } else if line.contains("<|--") {
            let parts: Vec<&str> = line.split("<|--").collect();
            (parts[1].trim(), parts[0].trim(), RelationType::OneToOne) // Inheritance (reverse)
        } else if line.contains("--*") {
            let parts: Vec<&str> = line.split("--*").collect();
            (parts[0].trim(), parts[1].trim(), RelationType::OneToMany) // Composition
        } else if line.contains("*--") {
            let parts: Vec<&str> = line.split("*--").collect();
            (parts[1].trim(), parts[0].trim(), RelationType::ManyToOne)
        } else if line.contains("--o") {
            let parts: Vec<&str> = line.split("--o").collect();
            (parts[0].trim(), parts[1].trim(), RelationType::OneToMany) // Aggregation
        } else if line.contains("o--") {
            let parts: Vec<&str> = line.split("o--").collect();
            (parts[1].trim(), parts[0].trim(), RelationType::ManyToOne)
        } else if line.contains("--") {
            let parts: Vec<&str> = line.split("--").collect();
            if parts.len() >= 2 {
                (parts[0].trim(), parts[1].trim(), RelationType::OneToOne)
            } else {
                return Ok(None);
            }
        } else {
            return Ok(None);
        };

        // Clean up names (remove labels like : "label")
        let from_clean = from.split(':').next().unwrap_or(from).trim();
        let to_clean = to.split(':').next().unwrap_or(to).trim();

        Ok(Some(Relationship {
            from_model: from_clean.to_string(),
            from_field: "id".to_string(),
            to_model: to_clean.to_string(),
            to_field: "id".to_string(),
            rel_type,
        }))
    }

    /// Parse stateDiagram
    fn parse_state_diagram(&self, content: &str) -> Result<StateMachineSpec, MermaidError> {
        let mut spec = StateMachineSpec::default();

        for line in content.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() || line.starts_with("%%") {
                continue;
            }

            // State definition: state "State Name" as alias
            if line.starts_with("state ") {
                if let Some(state) = self.parse_state_def(line)? {
                    spec.states.push(state);
                }
            }
            // Transition: State1 --> State2 : event
            else if line.contains("-->") {
                if let Some(transition) = self.parse_transition(line)? {
                    // Check for initial state
                    if transition.from == "[*]" {
                        spec.initial_state = Some(transition.to.clone());
                    } else if transition.to == "[*]" {
                        spec.final_states.push(transition.from.clone());
                    }
                    spec.transitions.push(transition);
                }
            }
        }

        Ok(spec)
    }

    /// Parse state definition
    fn parse_state_def(&self, line: &str) -> Result<Option<StateDef>, MermaidError> {
        // state "Description" as StateName
        let without_state = line.strip_prefix("state ").unwrap_or(line).trim();

        let (description, name) = if without_state.starts_with('"') {
            // Has description
            if let Some(end_quote) = without_state[1..].find('"') {
                let desc = &without_state[1..end_quote + 1];
                let rest = without_state[end_quote + 2..].trim();
                let name = rest.strip_prefix("as ").unwrap_or(rest).trim();
                (Some(desc.to_string()), name.to_string())
            } else {
                (None, without_state.to_string())
            }
        } else {
            (
                None,
                without_state
                    .split_whitespace()
                    .next()
                    .unwrap_or(without_state)
                    .to_string(),
            )
        };

        Ok(Some(StateDef {
            name,
            description,
            on_enter: None,
            on_exit: None,
            nested: None,
        }))
    }

    /// Parse transition
    fn parse_transition(&self, line: &str) -> Result<Option<TransitionDef>, MermaidError> {
        let parts: Vec<&str> = line.split("-->").collect();
        if parts.len() < 2 {
            return Ok(None);
        }

        let from = parts[0].trim().to_string();
        let to_and_event = parts[1].trim();

        let (to, event) = if let Some(colon) = to_and_event.find(':') {
            (
                to_and_event[..colon].trim().to_string(),
                Some(to_and_event[colon + 1..].trim().to_string()),
            )
        } else {
            (to_and_event.to_string(), None)
        };

        Ok(Some(TransitionDef {
            from,
            to,
            event,
            guard: None,
            action: None,
        }))
    }

    /// Parse flowchart
    fn parse_flowchart(&self, content: &str) -> Result<ControlFlowSpec, MermaidError> {
        let mut spec = ControlFlowSpec::default();

        for line in content.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() || line.starts_with("%%") {
                continue;
            }

            // Parse node definitions and edges
            if line.contains("-->") || line.contains("---") {
                self.parse_flow_line(line, &mut spec)?;
            }
        }

        Ok(spec)
    }

    /// Parse flowchart line (node and edge)
    fn parse_flow_line(&self, line: &str, spec: &mut ControlFlowSpec) -> Result<(), MermaidError> {
        // Split by arrow
        let parts: Vec<&str> = if line.contains("-->") {
            line.split("-->").collect()
        } else {
            line.split("---").collect()
        };

        for (i, part) in parts.iter().enumerate() {
            let (node_id, label, node_type) = self.parse_flow_node(part.trim())?;

            // Add node if not exists
            if !spec.nodes.iter().any(|n| n.id == node_id) {
                spec.nodes.push(FlowNode {
                    id: node_id.clone(),
                    label,
                    node_type,
                });
            }

            // Add edge to next node
            if i < parts.len() - 1 {
                let (next_id, _, _) = self.parse_flow_node(parts[i + 1].trim())?;
                spec.edges.push(FlowEdge {
                    from: node_id.clone(),
                    to: next_id,
                    label: None,
                    condition: None,
                });
            }
        }

        Ok(())
    }

    /// Parse flow node
    fn parse_flow_node(&self, s: &str) -> Result<(String, String, FlowNodeType), MermaidError> {
        // Formats:
        // A[Label] - process
        // A{Label} - decision
        // A([Label]) - start/end
        // A((Label)) - circle
        // A>Label] - flag

        let s = s.trim();

        // Extract edge label if present (|label|)
        let s = if let Some(pipe) = s.find('|') {
            &s[..pipe]
        } else {
            s
        };

        let s = s.trim();

        // Find bracket type
        if let Some(bracket_start) = s.find(|c| c == '[' || c == '{' || c == '(') {
            let id = s[..bracket_start].trim().to_string();
            let bracket_char = s.chars().nth(bracket_start).unwrap();

            let (label, node_type) = match bracket_char {
                '[' => {
                    let end = s.rfind(']').unwrap_or(s.len());
                    (s[bracket_start + 1..end].to_string(), FlowNodeType::Process)
                }
                '{' => {
                    let end = s.rfind('}').unwrap_or(s.len());
                    (
                        s[bracket_start + 1..end].to_string(),
                        FlowNodeType::Decision,
                    )
                }
                '(' => {
                    let end = s.rfind(')').unwrap_or(s.len());
                    let inner = &s[bracket_start + 1..end];
                    if inner.starts_with('(') && inner.ends_with(')') {
                        (inner[1..inner.len() - 1].to_string(), FlowNodeType::End)
                    } else if inner.starts_with('[') && inner.ends_with(']') {
                        (inner[1..inner.len() - 1].to_string(), FlowNodeType::Start)
                    } else {
                        (inner.to_string(), FlowNodeType::Start)
                    }
                }
                _ => (s.to_string(), FlowNodeType::Process),
            };

            Ok((id, label, node_type))
        } else {
            Ok((s.to_string(), s.to_string(), FlowNodeType::Process))
        }
    }

    /// Parse erDiagram (Entity-Relationship)
    fn parse_er_diagram(&self, content: &str) -> Result<DataModelSpec, MermaidError> {
        let mut spec = DataModelSpec::new();
        let mut current_entity: Option<ModelDef> = None;

        for line in content.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() || line.starts_with("%%") {
                continue;
            }

            // Relationship: Entity1 ||--o{ Entity2 : "relationship"
            if self.is_er_relationship(line) {
                // Save current entity first
                if let Some(entity) = current_entity.take() {
                    spec.add_model(entity);
                }

                if let Some(rel) = self.parse_er_relationship(line)? {
                    spec.relationships.push(rel);
                }
            }
            // Entity definition: EntityName {
            else if line.ends_with('{') {
                if let Some(entity) = current_entity.take() {
                    spec.add_model(entity);
                }
                let name = line.trim_end_matches('{').trim().to_string();
                current_entity = Some(ModelDef {
                    name,
                    table_name: None,
                    ..Default::default()
                });
            }
            // End of entity
            else if line == "}" {
                if let Some(entity) = current_entity.take() {
                    spec.add_model(entity);
                }
            }
            // Field inside entity: type name PK/FK "comment"
            else if let Some(ref mut entity) = current_entity {
                if let Some(field) = self.parse_er_field(line)? {
                    entity.fields.push(field);
                }
            }
        }

        if let Some(entity) = current_entity {
            spec.add_model(entity);
        }

        Ok(spec)
    }

    /// Check if line is an ER relationship
    fn is_er_relationship(&self, line: &str) -> bool {
        line.contains("||")
            || line.contains("}|")
            || line.contains("|{")
            || line.contains("}o")
            || line.contains("o{")
            || line.contains("--")
    }

    /// Parse ER relationship
    fn parse_er_relationship(&self, line: &str) -> Result<Option<Relationship>, MermaidError> {
        // Format: Entity1 ||--o{ Entity2 : "label"
        // ||--|| one to one
        // ||--o{ one to many
        // }o--o{ many to many

        // Find the relationship marker
        let rel_markers = [
            "||--||", "||--o{", "}o--||", "}o--o{", "||--|{", "}|--||", "--",
        ];

        for marker in rel_markers {
            if let Some(pos) = line.find(marker) {
                let left = line[..pos].trim();
                let right_and_label = line[pos + marker.len()..].trim();

                let right = right_and_label
                    .split(':')
                    .next()
                    .unwrap_or(right_and_label)
                    .trim();

                let rel_type = match marker {
                    "||--||" => RelationType::OneToOne,
                    "||--o{" | "||--|{" => RelationType::OneToMany,
                    "}o--||" | "}|--||" => RelationType::ManyToOne,
                    "}o--o{" => RelationType::ManyToMany,
                    _ => RelationType::OneToOne,
                };

                return Ok(Some(Relationship {
                    from_model: left.to_string(),
                    from_field: "id".to_string(),
                    to_model: right.to_string(),
                    to_field: format!("{}_id", left.to_lowercase()),
                    rel_type,
                }));
            }
        }

        Ok(None)
    }

    /// Parse ER field
    fn parse_er_field(&self, line: &str) -> Result<Option<FieldDef>, MermaidError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Ok(None);
        }

        let ty = self.parse_type(parts[0]);
        let name = parts[1].to_string();

        let primary_key = parts.iter().any(|&p| p == "PK");
        let is_foreign_key = parts.iter().any(|&p| p == "FK");
        let unique = parts.iter().any(|&p| p == "UK");

        Ok(Some(FieldDef {
            name,
            ty,
            required: primary_key || !line.contains("NULL"),
            default: None,
            description: None,
            constraints: FieldConstraints::default(),
            column_name: None,
            primary_key,
            unique,
            indexed: primary_key || is_foreign_key,
            foreign_key: None,
            alias: None,
        }))
    }
}

impl Default for MermaidParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Enum for class members
enum ClassMember {
    Field(FieldDef),
    Method(MethodDef),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_class_diagram() {
        let mermaid = r#"
classDiagram
    class User {
        +String name
        +String email
        -String password
        +login(username: String, password: String) bool
        +logout() void
    }
    class Order {
        +int id
        +float total
        +createOrder(items: List) Order
    }
    User --* Order : places
"#;

        let parser = MermaidParser::new();
        let result = parser.parse(mermaid).unwrap();

        match result {
            MermaidSpec::DataModel(spec) => {
                assert_eq!(spec.models.len(), 2);

                let user = spec.models.iter().find(|m| m.name == "User").unwrap();
                assert_eq!(user.fields.len(), 3);
                assert_eq!(user.methods.len(), 2);

                let order = spec.models.iter().find(|m| m.name == "Order").unwrap();
                assert_eq!(order.fields.len(), 2);

                assert_eq!(spec.relationships.len(), 1);
            }
            _ => panic!("Expected DataModel"),
        }
    }

    #[test]
    fn test_parse_state_diagram() {
        let mermaid = r#"
stateDiagram-v2
    [*] --> Idle
    Idle --> Processing : start
    Processing --> Complete : finish
    Processing --> Error : fail
    Complete --> [*]
    Error --> Idle : retry
"#;

        let parser = MermaidParser::new();
        let result = parser.parse(mermaid).unwrap();

        match result {
            MermaidSpec::StateMachine(spec) => {
                assert_eq!(spec.initial_state, Some("Idle".to_string()));
                assert!(spec.final_states.contains(&"Complete".to_string()));
                assert!(spec.transitions.len() >= 5);
            }
            _ => panic!("Expected StateMachine"),
        }
    }

    #[test]
    fn test_parse_flowchart() {
        let mermaid = r#"
flowchart TD
    A[Start] --> B{Is valid?}
    B -->|Yes| C[Process]
    B -->|No| D[Error]
    C --> E[End]
    D --> E
"#;

        let parser = MermaidParser::new();
        let result = parser.parse(mermaid).unwrap();

        match result {
            MermaidSpec::ControlFlow(spec) => {
                assert!(spec.nodes.len() >= 4);
                assert!(spec.edges.len() >= 4);

                // Check node types
                let decision = spec.nodes.iter().find(|n| n.id == "B").unwrap();
                assert!(matches!(decision.node_type, FlowNodeType::Decision));
            }
            _ => panic!("Expected ControlFlow"),
        }
    }

    #[test]
    fn test_parse_er_diagram() {
        let mermaid = r#"
erDiagram
    CUSTOMER ||--o{ ORDER : places
    ORDER ||--|{ LINE_ITEM : contains
    CUSTOMER {
        string name
        string email PK
        int age
    }
    ORDER {
        int id PK
        float total
        string status
    }
"#;

        let parser = MermaidParser::new();
        let result = parser.parse(mermaid).unwrap();

        match result {
            MermaidSpec::DataModel(spec) => {
                assert_eq!(spec.models.len(), 2);
                assert_eq!(spec.relationships.len(), 2);

                let customer = spec.models.iter().find(|m| m.name == "CUSTOMER").unwrap();
                assert_eq!(customer.fields.len(), 3);

                let email_field = customer.fields.iter().find(|f| f.name == "email").unwrap();
                assert!(email_field.primary_key);
            }
            _ => panic!("Expected DataModel"),
        }
    }

    #[test]
    fn test_detect_diagram_type() {
        let parser = MermaidParser::new();

        assert_eq!(
            parser.detect_diagram_type("classDiagram\n...").unwrap(),
            DiagramType::ClassDiagram
        );
        assert_eq!(
            parser.detect_diagram_type("stateDiagram-v2\n...").unwrap(),
            DiagramType::StateDiagram
        );
        assert_eq!(
            parser.detect_diagram_type("flowchart TD\n...").unwrap(),
            DiagramType::Flowchart
        );
        assert_eq!(
            parser.detect_diagram_type("erDiagram\n...").unwrap(),
            DiagramType::ErDiagram
        );
    }
}
