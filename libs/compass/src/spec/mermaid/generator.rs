//! Mermaid diagram generator
//!
//! Generates Mermaid diagrams from SpecIR structures.

use crate::spec::ir::{
    ControlFlowSpec, DataModelSpec, EventApiSpec, FlowNodeType, ModelDef, RelationType,
    RestApiSpec, StateMachineSpec, Visibility,
};

/// Mermaid diagram generator
pub struct MermaidGenerator;

impl MermaidGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate classDiagram from DataModelSpec
    pub fn generate_class_diagram(&self, spec: &DataModelSpec) -> String {
        let mut lines = vec!["classDiagram".to_string()];

        // Generate classes
        for model in &spec.models {
            lines.push(self.generate_class(model));
        }

        // Generate relationships
        for rel in &spec.relationships {
            let arrow = match rel.rel_type {
                RelationType::OneToOne => "--",
                RelationType::OneToMany => "--*",
                RelationType::ManyToOne => "*--",
                RelationType::ManyToMany => "*--*",
            };
            lines.push(format!("    {} {} {}", rel.from_model, arrow, rel.to_model));
        }

        lines.join("\n")
    }

    /// Generate class definition
    fn generate_class(&self, model: &ModelDef) -> String {
        let mut lines = vec![format!("    class {} {{", model.name)];

        // Generate fields
        for field in &model.fields {
            let visibility = "+"; // Default to public for simplicity
            let ty = self.type_to_string(&field.ty);
            lines.push(format!("        {}{} {}", visibility, ty, field.name));
        }

        // Generate methods
        for method in &model.methods {
            let visibility = match method.visibility {
                Visibility::Public => "+",
                Visibility::Private => "-",
                Visibility::Protected => "#",
            };

            let params: Vec<String> = method
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, self.type_to_string(&p.ty)))
                .collect();

            let return_ty = self.type_to_string(&method.return_type);
            lines.push(format!(
                "        {}{}({}) {}",
                visibility,
                method.name,
                params.join(", "),
                return_ty
            ));
        }

        lines.push("    }".to_string());
        lines.join("\n")
    }

    /// Generate erDiagram from DataModelSpec
    pub fn generate_er_diagram(&self, spec: &DataModelSpec) -> String {
        let mut lines = vec!["erDiagram".to_string()];

        // Generate relationships
        for rel in &spec.relationships {
            let marker = match rel.rel_type {
                RelationType::OneToOne => "||--||",
                RelationType::OneToMany => "||--o{",
                RelationType::ManyToOne => "}o--||",
                RelationType::ManyToMany => "}o--o{",
            };
            lines.push(format!(
                "    {} {} {} : \"{}\"",
                rel.from_model, marker, rel.to_model, rel.from_field
            ));
        }

        // Generate entities
        for model in &spec.models {
            lines.push(format!("    {} {{", model.name));
            for field in &model.fields {
                let ty = self.type_to_er_type(&field.ty);
                let constraints = if field.primary_key {
                    " PK"
                } else if field.foreign_key.is_some() {
                    " FK"
                } else if field.unique {
                    " UK"
                } else {
                    ""
                };
                lines.push(format!("        {} {}{}", ty, field.name, constraints));
            }
            lines.push("    }".to_string());
        }

        lines.join("\n")
    }

    /// Generate sequenceDiagram from RestApiSpec
    pub fn generate_sequence_diagram(&self, spec: &RestApiSpec) -> String {
        let mut lines = vec!["sequenceDiagram".to_string()];

        // Add participants
        lines.push("    participant Client".to_string());
        let server_name = spec.title.replace(" ", "");
        lines.push(format!("    participant {} as Server", server_name));

        // Generate sequence for each endpoint
        for endpoint in &spec.endpoints {
            let method = format!("{:?}", endpoint.method).to_uppercase();
            let summary = endpoint.summary.as_deref().unwrap_or(&endpoint.path);

            lines.push(format!(
                "    Client->>Server: {} {} ({})",
                method, endpoint.path, summary
            ));

            // Response
            if let Some(resp) = endpoint.responses.first() {
                lines.push(format!(
                    "    Server-->>Client: {} {}",
                    resp.status_code, resp.description
                ));
            }
        }

        lines.join("\n")
    }

    /// Generate sequenceDiagram from EventApiSpec
    pub fn generate_event_sequence(&self, spec: &EventApiSpec) -> String {
        let mut lines = vec!["sequenceDiagram".to_string()];

        // Add participants
        lines.push("    participant Publisher".to_string());
        lines.push("    participant MessageBroker".to_string());
        lines.push("    participant Subscriber".to_string());

        // Generate sequence for each channel
        for channel in &spec.channels {
            if let Some(pub_op) = &channel.publish {
                let msg = pub_op.summary.as_deref().unwrap_or(&channel.name);
                lines.push(format!(
                    "    Publisher->>MessageBroker: publish to {} ({})",
                    channel.name, msg
                ));
            }

            if let Some(sub_op) = &channel.subscribe {
                let msg = sub_op.summary.as_deref().unwrap_or(&channel.name);
                lines.push(format!(
                    "    MessageBroker->>Subscriber: deliver from {} ({})",
                    channel.name, msg
                ));
            }
        }

        lines.join("\n")
    }

    /// Generate stateDiagram from StateMachineSpec
    pub fn generate_state_diagram(&self, spec: &StateMachineSpec) -> String {
        let mut lines = vec!["stateDiagram-v2".to_string()];

        // Initial state
        if let Some(initial) = &spec.initial_state {
            lines.push(format!("    [*] --> {}", initial));
        }

        // States with descriptions
        for state in &spec.states {
            if let Some(desc) = &state.description {
                lines.push(format!("    state \"{}\" as {}", desc, state.name));
            }
        }

        // Transitions
        for transition in &spec.transitions {
            let label = if let Some(event) = &transition.event {
                format!(" : {}", event)
            } else {
                String::new()
            };
            lines.push(format!(
                "    {} --> {}{}",
                transition.from, transition.to, label
            ));
        }

        // Final states
        for final_state in &spec.final_states {
            lines.push(format!("    {} --> [*]", final_state));
        }

        lines.join("\n")
    }

    /// Generate flowchart from ControlFlowSpec
    pub fn generate_flowchart(&self, spec: &ControlFlowSpec, direction: &str) -> String {
        let mut lines = vec![format!("flowchart {}", direction)];

        // Generate nodes
        for node in &spec.nodes {
            let shape = match node.node_type {
                FlowNodeType::Start => format!("{}([{}])", node.id, node.label),
                FlowNodeType::End => format!("{}(({})) ", node.id, node.label),
                FlowNodeType::Process => format!("{}[{}]", node.id, node.label),
                FlowNodeType::Decision => format!("{}{{{}}}", node.id, node.label),
                FlowNodeType::InputOutput => format!("{}[/{}\\]", node.id, node.label),
                FlowNodeType::SubProcess => format!("{}[[{}]]", node.id, node.label),
                FlowNodeType::Database => format!("{}[({})]", node.id, node.label),
            };
            lines.push(format!("    {}", shape));
        }

        // Generate edges
        for edge in &spec.edges {
            let label = if let Some(lbl) = &edge.label {
                format!("|{}|", lbl)
            } else {
                String::new()
            };
            lines.push(format!("    {} -->{} {}", edge.from, label, edge.to));
        }

        lines.join("\n")
    }

    /// Convert Type to string for Mermaid
    fn type_to_string(&self, ty: &crate::type_inference::Type) -> String {
        use crate::type_inference::Type;
        match ty {
            Type::Str => "String".to_string(),
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Bool => "bool".to_string(),
            Type::None => "void".to_string(),
            Type::List(inner) => format!("List~{}~", self.type_to_string(inner)),
            Type::Optional(inner) => format!("{}?", self.type_to_string(inner)),
            Type::Instance { name, .. } => name.clone(),
            Type::Any => "any".to_string(),
            _ => "any".to_string(),
        }
    }

    /// Convert Type to ER diagram type
    fn type_to_er_type(&self, ty: &crate::type_inference::Type) -> String {
        use crate::type_inference::Type;
        match ty {
            Type::Str => "string".to_string(),
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Bool => "boolean".to_string(),
            Type::None => "null".to_string(),
            Type::Instance { name, .. } => name.to_lowercase(),
            _ => "any".to_string(),
        }
    }
}

impl Default for MermaidGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::ir::{
        EndpointDef, FieldDef, FlowEdge, FlowNode, HttpMethod, MethodDef, Relationship,
        ResponseDef, StateDef, TransitionDef,
    };
    use crate::type_inference::Type;

    #[test]
    fn test_generate_class_diagram() {
        let spec = DataModelSpec {
            models: vec![ModelDef {
                name: "User".to_string(),
                fields: vec![
                    FieldDef {
                        name: "id".to_string(),
                        ty: Type::Int,
                        ..Default::default()
                    },
                    FieldDef {
                        name: "name".to_string(),
                        ty: Type::Str,
                        ..Default::default()
                    },
                ],
                methods: vec![MethodDef {
                    name: "save".to_string(),
                    params: vec![],
                    return_type: Type::None,
                    visibility: Visibility::Public,
                    is_static: false,
                    is_async: false,
                    description: None,
                }],
                ..Default::default()
            }],
            enums: vec![],
            relationships: vec![],
        };

        let gen = MermaidGenerator::new();
        let result = gen.generate_class_diagram(&spec);

        assert!(result.contains("classDiagram"));
        assert!(result.contains("class User"));
        assert!(result.contains("int id"));
        assert!(result.contains("String name"));
        assert!(result.contains("+save()"));
    }

    #[test]
    fn test_generate_er_diagram() {
        let spec = DataModelSpec {
            models: vec![
                ModelDef {
                    name: "Customer".to_string(),
                    fields: vec![
                        FieldDef {
                            name: "id".to_string(),
                            ty: Type::Int,
                            primary_key: true,
                            ..Default::default()
                        },
                        FieldDef {
                            name: "name".to_string(),
                            ty: Type::Str,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                },
                ModelDef {
                    name: "Order".to_string(),
                    fields: vec![FieldDef {
                        name: "id".to_string(),
                        ty: Type::Int,
                        primary_key: true,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ],
            enums: vec![],
            relationships: vec![Relationship {
                from_model: "Customer".to_string(),
                from_field: "id".to_string(),
                to_model: "Order".to_string(),
                to_field: "customer_id".to_string(),
                rel_type: RelationType::OneToMany,
            }],
        };

        let gen = MermaidGenerator::new();
        let result = gen.generate_er_diagram(&spec);

        assert!(result.contains("erDiagram"));
        assert!(result.contains("Customer ||--o{ Order"));
        assert!(result.contains("int id PK"));
    }

    #[test]
    fn test_generate_state_diagram() {
        let spec = StateMachineSpec {
            name: "OrderState".to_string(),
            states: vec![
                StateDef {
                    name: "Pending".to_string(),
                    description: Some("Order is pending".to_string()),
                    on_enter: None,
                    on_exit: None,
                    nested: None,
                },
                StateDef {
                    name: "Processing".to_string(),
                    description: None,
                    on_enter: None,
                    on_exit: None,
                    nested: None,
                },
            ],
            transitions: vec![TransitionDef {
                from: "Pending".to_string(),
                to: "Processing".to_string(),
                event: Some("confirm".to_string()),
                guard: None,
                action: None,
            }],
            initial_state: Some("Pending".to_string()),
            final_states: vec!["Completed".to_string()],
        };

        let gen = MermaidGenerator::new();
        let result = gen.generate_state_diagram(&spec);

        assert!(result.contains("stateDiagram-v2"));
        assert!(result.contains("[*] --> Pending"));
        assert!(result.contains("Pending --> Processing : confirm"));
        assert!(result.contains("Completed --> [*]"));
    }

    #[test]
    fn test_generate_flowchart() {
        let spec = ControlFlowSpec {
            name: "LoginFlow".to_string(),
            nodes: vec![
                FlowNode {
                    id: "A".to_string(),
                    label: "Start".to_string(),
                    node_type: FlowNodeType::Start,
                },
                FlowNode {
                    id: "B".to_string(),
                    label: "Valid?".to_string(),
                    node_type: FlowNodeType::Decision,
                },
                FlowNode {
                    id: "C".to_string(),
                    label: "Login".to_string(),
                    node_type: FlowNodeType::Process,
                },
            ],
            edges: vec![
                FlowEdge {
                    from: "A".to_string(),
                    to: "B".to_string(),
                    label: None,
                    condition: None,
                },
                FlowEdge {
                    from: "B".to_string(),
                    to: "C".to_string(),
                    label: Some("Yes".to_string()),
                    condition: None,
                },
            ],
        };

        let gen = MermaidGenerator::new();
        let result = gen.generate_flowchart(&spec, "TD");

        assert!(result.contains("flowchart TD"));
        assert!(result.contains("A([Start])"));
        assert!(result.contains("B{Valid?}"));
        assert!(result.contains("C[Login]"));
        assert!(result.contains("B -->|Yes| C"));
    }

    #[test]
    fn test_generate_sequence_diagram() {
        let spec = RestApiSpec {
            title: "User API".to_string(),
            version: "1.0".to_string(),
            description: None,
            servers: vec![],
            endpoints: vec![EndpointDef {
                path: "/users".to_string(),
                method: HttpMethod::Get,
                operation_id: None,
                summary: Some("List users".to_string()),
                description: None,
                tags: vec![],
                path_params: vec![],
                query_params: vec![],
                request_body: None,
                responses: vec![ResponseDef {
                    status_code: 200,
                    description: "OK".to_string(),
                    schema: None,
                    content_type: None,
                }],
                security: vec![],
                deprecated: false,
            }],
            schemas: DataModelSpec::default(),
            security_schemes: vec![],
        };

        let gen = MermaidGenerator::new();
        let result = gen.generate_sequence_diagram(&spec);

        assert!(result.contains("sequenceDiagram"));
        assert!(result.contains("participant Client"));
        assert!(result.contains("Client->>Server: GET /users"));
        assert!(result.contains("Server-->>Client: 200 OK"));
    }
}
