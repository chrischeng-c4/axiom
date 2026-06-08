// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/mcp/tools.md#source
// CODEGEN-BEGIN
//! SDD MCP Tool Schemas
//!
//! Defines tool schemas for diagram and spec generation.

use serde_json::{json, Value};

use serde::{Deserialize, Serialize};

/// Tool schema for MCP protocol.
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/tools.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Tool name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema for tool input.
    pub input_schema: Value,
}

/// SDD generate tool registry.
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/tools.md#schema
pub struct SddTools;
/// @spec projects/agentic-workflow/tech-design/core/generate/mcp/tools.md#source
impl SddTools {
    /// List all SDD generate tools
    pub fn list() -> Vec<ToolSchema> {
        vec![
            // Simple diagram tools
            Self::sdd_generate_flowchart(),
            Self::sdd_generate_sequence(),
            Self::sdd_generate_class(),
            Self::sdd_generate_state(),
            Self::sdd_generate_erd(),
            Self::sdd_generate_mindmap(),
            Self::sdd_generate_requirement(),
            Self::sdd_generate_journey(),
            // Plus diagram tools (validated + YAML frontmatter)
            Self::sdd_generate_mermaid_plus(),
            Self::sdd_generate_flowchart_plus(),
            Self::sdd_generate_sequence_plus(),
            Self::sdd_generate_class_plus(),
            Self::sdd_generate_erd_plus(),
            Self::sdd_generate_requirement_plus(),
            Self::sdd_generate_mindmap_plus(),
            Self::sdd_generate_journey_plus(),
            Self::sdd_generate_block_plus(),
            // Spec tools
            Self::sdd_generate_openapi(),
            Self::sdd_generate_asyncapi(),
            Self::sdd_generate_openrpc(),
            Self::sdd_generate_serverless_workflow(),
        ]
    }

    /// Get tool count
    pub fn count() -> usize {
        21
    }

    // === Diagram Tools ===

    fn sdd_generate_flowchart() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_flowchart".to_string(),
            description: "Generate Mermaid flowchart from structured input. Use for algorithms, business logic, decision trees.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "nodes": {
                        "type": "array",
                        "description": "List of flowchart nodes",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "label": { "type": "string" },
                                "shape": { "type": "string", "enum": ["rectangle", "rounded", "stadium", "subroutine", "cylinder", "circle", "rhombus", "hexagon", "parallelogram", "trapezoid"] }
                            },
                            "required": ["id", "label"]
                        }
                    },
                    "edges": {
                        "type": "array",
                        "description": "List of connections between nodes",
                        "items": {
                            "type": "object",
                            "properties": {
                                "from": { "type": "string" },
                                "to": { "type": "string" },
                                "label": { "type": "string" },
                                "style": { "type": "string", "enum": ["solid", "dotted", "thick"] }
                            },
                            "required": ["from", "to"]
                        }
                    },
                    "direction": { "type": "string", "enum": ["TB", "BT", "LR", "RL"], "default": "TB" },
                    "subgraphs": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "label": { "type": "string" },
                                "nodes": { "type": "array", "items": { "type": "string" } }
                            }
                        }
                    }
                },
                "required": ["nodes", "edges"]
            }),
        }
    }

    fn sdd_generate_sequence() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_sequence".to_string(),
            description: "Generate Mermaid sequence diagram from structured input. Use for API flows, service interactions.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "participants": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "label": { "type": "string" },
                                "type": { "type": "string", "enum": ["participant", "actor"] }
                            },
                            "required": ["id", "label"]
                        }
                    },
                    "messages": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "from": { "type": "string" },
                                "to": { "type": "string" },
                                "text": { "type": "string", "description": "Message text" },
                                "type": { "type": "string", "enum": ["solid", "dotted", "solid_open", "dotted_open"] },
                                "activate": { "type": "boolean", "default": false },
                                "deactivate": { "type": "boolean", "default": false }
                            },
                            "required": ["from", "to", "text"]
                        }
                    }
                },
                "required": ["participants", "messages"]
            }),
        }
    }

    fn sdd_generate_class() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_class".to_string(),
            description: "Generate Mermaid class diagram from structured input. Use for data models, OOP design.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "classes": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "stereotype": { "type": "string", "enum": ["interface", "abstract", "enumeration", "service"] },
                                "attributes": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "name": { "type": "string" },
                                            "type": { "type": "string" },
                                            "visibility": { "type": "string", "enum": ["public", "private", "protected", "package"], "default": "public" },
                                            "static_attr": { "type": "boolean", "default": false }
                                        },
                                        "required": ["name", "type"]
                                    }
                                },
                                "methods": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "name": { "type": "string" },
                                            "parameters": { "type": "array", "items": { "type": "object", "properties": { "name": { "type": "string" }, "type": { "type": "string" } } } },
                                            "return_type": { "type": "string" },
                                            "visibility": { "type": "string", "enum": ["public", "private", "protected", "package"], "default": "public" },
                                            "static_method": { "type": "boolean", "default": false },
                                            "abstract_method": { "type": "boolean", "default": false }
                                        },
                                        "required": ["name"]
                                    }
                                }
                            },
                            "required": ["name"]
                        }
                    },
                    "relationships": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "from": { "type": "string" },
                                "to": { "type": "string" },
                                "type": { "type": "string", "enum": ["inheritance", "composition", "aggregation", "association", "dependency", "realization"] },
                                "label": { "type": "string" },
                                "multiplicity_from": { "type": "string" },
                                "multiplicity_to": { "type": "string" }
                            },
                            "required": ["from", "to", "type"]
                        }
                    },
                    "namespaces": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "classes": { "type": "array", "items": { "type": "string" } }
                            },
                            "required": ["name", "classes"]
                        }
                    }
                },
                "required": ["classes"]
            }),
        }
    }

    fn sdd_generate_state() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_state".to_string(),
            description: "Generate Mermaid state diagram from structured input. Use for simple state machines, UI states.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "states": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "label": { "type": "string" },
                                "type": { "type": "string", "enum": ["normal", "start", "end", "choice", "fork", "join"] }
                            },
                            "required": ["id", "label"]
                        }
                    },
                    "transitions": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "from": { "type": "string" },
                                "to": { "type": "string" },
                                "label": { "type": "string" }
                            },
                            "required": ["from", "to"]
                        }
                    },
                    "direction": { "type": "string", "enum": ["TB", "BT", "LR", "RL"] }
                },
                "required": ["states"]
            }),
        }
    }

    fn sdd_generate_erd() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_erd".to_string(),
            description: "Generate Mermaid ERD from structured input. Use for database design, data modeling.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entities": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "attributes": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "name": { "type": "string" },
                                            "type": { "type": "string" },
                                            "key": { "type": "string", "enum": ["PK", "FK", "UK"] },
                                            "nullable": { "type": "boolean", "default": false },
                                            "comment": { "type": "string" }
                                        },
                                        "required": ["name", "type"]
                                    }
                                }
                            },
                            "required": ["name"]
                        }
                    },
                    "relationships": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "from": { "type": "string" },
                                "to": { "type": "string" },
                                "cardinality": {
                                    "type": "string",
                                    "enum": ["OneToOne", "OneToMany", "ManyToOne", "ManyToMany", "OneOrMoreToOne", "OneToOneOrMore", "ZeroOrOneToOne", "OneToZeroOrOne"],
                                    "description": "Relationship cardinality"
                                },
                                "label": { "type": "string" },
                                "identifying": { "type": "boolean", "default": false }
                            },
                            "required": ["from", "to", "cardinality"]
                        }
                    }
                },
                "required": ["entities"]
            }),
        }
    }

    fn sdd_generate_mindmap() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_mindmap".to_string(),
            description: "Generate Mermaid mindmap from structured input. Use for brainstorming, concept organization.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "root": {
                        "type": "object",
                        "properties": {
                            "label": { "type": "string" },
                            "children": { "type": "array" }
                        },
                        "required": ["label"]
                    }
                },
                "required": ["root"]
            }),
        }
    }

    fn sdd_generate_requirement() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_requirement".to_string(),
            description: "Generate Mermaid requirement diagram from structured input. Use for requirement traceability.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "requirements": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "text": { "type": "string", "description": "Requirement text description" },
                                "type": { "type": "string", "enum": ["requirement", "functionalRequirement", "interfaceRequirement", "performanceRequirement", "physicalRequirement", "designConstraint"], "default": "requirement" },
                                "risk": { "type": "string", "enum": ["Low", "Medium", "High"] },
                                "verification": { "type": "string", "enum": ["Analysis", "Inspection", "Test", "Demonstration"] }
                            },
                            "required": ["id", "text", "risk", "verification"]
                        }
                    },
                    "elements": {
                        "type": "array",
                        "description": "Design elements that satisfy/verify requirements",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "text": { "type": "string" },
                                "type": { "type": "string", "description": "Element type (e.g., 'module', 'component', 'test')" },
                                "docref": { "type": "string", "description": "Documentation reference (optional at spec stage)" },
                                "test_type": { "type": "string", "enum": ["unit", "integration", "e2e"], "description": "Test type — for test elements" },
                                "given": { "type": "string", "description": "Given precondition (BDD) — for test elements" },
                                "when": { "type": "string", "description": "When action (BDD) — for test elements" },
                                "then": { "type": "string", "description": "Then expected outcome (BDD) — for test elements" }
                            },
                            "required": ["id", "text", "type"]
                        }
                    },
                    "relationships": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "from": { "type": "string" },
                                "to": { "type": "string" },
                                "type": { "type": "string", "enum": ["satisfies", "verifies", "refines", "traces", "contains", "copies", "derives"] }
                            },
                            "required": ["from", "to", "type"]
                        }
                    }
                },
                "required": ["requirements"]
            }),
        }
    }

    fn sdd_generate_journey() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_journey".to_string(),
            description: "Generate Mermaid user journey diagram from structured input. Use for UX flows, service blueprints.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "sections": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "tasks": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "name": { "type": "string" },
                                            "score": { "type": "integer", "minimum": 1, "maximum": 5, "description": "Satisfaction score (1=bad, 5=great)" },
                                            "actors": { "type": "array", "items": { "type": "string" }, "minItems": 1, "description": "List of actors involved" }
                                        },
                                        "required": ["name", "score", "actors"]
                                    }
                                }
                            },
                            "required": ["name", "tasks"]
                        }
                    }
                },
                "required": ["title", "sections"]
            }),
        }
    }

    fn sdd_generate_mermaid_plus() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_mermaid_plus".to_string(),
            description: "Generate Mermaid+ stateDiagram-v2 from XState-compatible state machine definition. Includes YAML frontmatter with metadata, guards, and actions. Supports nested and parallel states. Input is validated before generation.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Machine identifier" },
                    "initial": { "type": "string", "description": "Initial state ID" },
                    "states": {
                        "type": "object",
                        "description": "State definitions keyed by state ID",
                        "additionalProperties": {
                            "type": "object",
                            "properties": {
                                "type": { "type": "string", "enum": ["atomic", "compound", "parallel", "final"] },
                                "initial": { "type": "string" },
                                "states": { "type": "object" },
                                "on": { "type": "object" },
                                "entry": {},
                                "exit": {},
                                "description": { "type": "string" }
                            }
                        }
                    },
                    "guards": {
                        "type": "object",
                        "description": "Guard condition definitions",
                        "additionalProperties": {
                            "type": "object",
                            "properties": {
                                "condition": { "type": "string" },
                                "description": { "type": "string" }
                            }
                        }
                    },
                    "actions": {
                        "type": "object",
                        "description": "Action definitions",
                        "additionalProperties": {
                            "type": "object",
                            "properties": {
                                "effect": { "type": "string" },
                                "description": { "type": "string" }
                            }
                        }
                    },
                    "description": { "type": "string" }
                },
                "required": ["id", "initial", "states"]
            }),
        }
    }

    // === Plus Diagram Tools ===

    fn sdd_generate_flowchart_plus() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_flowchart_plus".to_string(),
            description: "Generate Mermaid+ flowchart with YAML frontmatter and validation. Supports semantic types for code generation (validation, db_query, api_call, etc.).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Diagram identifier" },
                    "direction": { "type": "string", "enum": ["TB", "BT", "LR", "RL"], "default": "TB" },
                    "nodes": {
                        "type": "object",
                        "description": "Node definitions keyed by node ID",
                        "additionalProperties": {
                            "type": "object",
                            "properties": {
                                "label": { "type": "string" },
                                "shape": { "type": "string", "enum": ["rectangle", "rounded", "stadium", "subroutine", "cylinder", "circle", "diamond", "hexagon", "parallelogram", "trapezoid"] },
                                "semantic": { "type": "object", "description": "Semantic type for code generation" }
                            },
                            "required": ["label"]
                        }
                    },
                    "edges": { "type": "array", "items": { "type": "object", "properties": { "from": { "type": "string" }, "to": { "type": "string" }, "label": { "type": "string" }, "style": { "type": "string", "enum": ["arrow", "thick", "dotted"] }, "condition": { "type": "string", "description": "Condition expression for conditional branches" }, "is_error_path": { "type": "boolean", "description": "Whether this edge represents an error handling path" } }, "required": ["from", "to"] } },
                    "subgraphs": { "type": "array" },
                    "description": { "type": "string" }
                },
                "required": ["id", "nodes"]
            }),
        }
    }

    fn sdd_generate_sequence_plus() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_sequence_plus".to_string(),
            description: "Generate Mermaid+ sequence diagram with YAML frontmatter and validation. Supports loops, alt/opt blocks, and activation.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Diagram identifier" },
                    "title": { "type": "string" },
                    "participants": {
                        "type": "object",
                        "description": "Participant definitions keyed by participant ID",
                        "additionalProperties": {
                            "type": "object",
                            "properties": {
                                "label": { "type": "string" },
                                "type": { "type": "string", "enum": ["participant", "actor"] }
                            },
                            "required": ["label"]
                        }
                    },
                    "messages": { "type": "array", "items": { "type": "object", "properties": { "from": { "type": "string" }, "to": { "type": "string" }, "text": { "type": "string" }, "type": { "type": "string" }, "activate": { "type": "boolean" }, "deactivate": { "type": "boolean" } }, "required": ["from", "to", "text"] } },
                    "loops": { "type": "array" },
                    "alts": { "type": "array" },
                    "notes": { "type": "array" }
                },
                "required": ["id", "participants", "messages"]
            }),
        }
    }

    fn sdd_generate_class_plus() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_class_plus".to_string(),
            description: "Generate Mermaid+ class diagram with YAML frontmatter and validation. Supports DDD stereotypes (entity, valueObject, aggregate).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Diagram identifier" },
                    "classes": {
                        "type": "object",
                        "description": "Class definitions keyed by class name",
                        "additionalProperties": {
                            "type": "object",
                            "properties": {
                                "stereotype": { "type": "string", "enum": ["interface", "abstract", "enumeration", "service", "entity", "valueObject", "aggregate"] },
                                "attributes": { "type": "array" },
                                "methods": { "type": "array" }
                            }
                        }
                    },
                    "relationships": { "type": "array" },
                    "namespaces": { "type": "array" }
                },
                "required": ["id", "classes"]
            }),
        }
    }

    fn sdd_generate_erd_plus() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_erd_plus".to_string(),
            description: "Generate Mermaid+ ERD with YAML frontmatter and validation. Validates FK references and PK existence.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Diagram identifier" },
                    "entities": {
                        "type": "object",
                        "description": "Entity definitions keyed by entity name",
                        "additionalProperties": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string", "description": "Display name (optional, defaults to key)" },
                                "description": { "type": "string", "description": "Entity description" },
                                "attributes": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "name": { "type": "string", "description": "Attribute name" },
                                            "type": { "type": "string", "description": "Data type (e.g. UUID, VARCHAR, INTEGER)" },
                                            "key": { "type": "string", "enum": ["PK", "FK", "UK"], "description": "Key type" },
                                            "nullable": { "type": "boolean", "description": "Whether the attribute is nullable" },
                                            "references": { "type": "string", "description": "FK reference in Entity.attribute format" },
                                            "comment": { "type": "string", "description": "Attribute comment/description" }
                                        },
                                        "required": ["name", "type"]
                                    }
                                }
                            }
                        }
                    },
                    "relationships": {
                        "type": "array",
                        "description": "Relationships between entities",
                        "items": {
                            "type": "object",
                            "properties": {
                                "from": { "type": "string", "description": "Source entity name" },
                                "to": { "type": "string", "description": "Target entity name" },
                                "cardinality": {
                                    "type": "string",
                                    "enum": ["one-to-one", "one-to-many", "many-to-one", "many-to-many", "one-or-more-to-one", "one-to-one-or-more", "zero-or-one-to-one", "one-to-zero-or-one"],
                                    "description": "Relationship cardinality"
                                },
                                "label": { "type": "string", "description": "Relationship label (verb phrase)" },
                                "identifying": { "type": "boolean", "description": "Whether this is an identifying relationship" }
                            },
                            "required": ["from", "to", "cardinality"]
                        }
                    },
                    "description": { "type": "string", "description": "Diagram description" }
                },
                "required": ["id", "entities"]
            }),
        }
    }

    fn sdd_generate_requirement_plus() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_requirement_plus".to_string(),
            description: "Generate Mermaid+ requirement diagram with YAML frontmatter and validation. Supports SysML v1.6 types, risk levels, verification methods, and layout direction.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Diagram identifier" },
                    "title": { "type": "string", "description": "Diagram title" },
                    "direction": { "type": "string", "enum": ["TB", "BT", "LR", "RL"], "description": "Layout direction" },
                    "requirements": {
                        "type": "object",
                        "description": "Requirement definitions keyed by requirement ID",
                        "additionalProperties": {
                            "type": "object",
                            "properties": {
                                "text": { "type": "string" },
                                "type": { "type": "string", "enum": ["requirement", "functionalRequirement", "interfaceRequirement", "performanceRequirement", "physicalRequirement", "designConstraint"] },
                                "risk": { "type": "string", "enum": ["Low", "Medium", "High"] },
                                "verification": { "type": "string", "enum": ["Analysis", "Inspection", "Test", "Demonstration"] }
                            },
                            "required": ["text", "risk", "verification"]
                        }
                    },
                    "elements": {
                        "type": "object",
                        "description": "Design/test elements keyed by element ID",
                        "additionalProperties": {
                            "type": "object",
                            "properties": {
                                "text": { "type": "string" },
                                "type": { "type": "string", "description": "Element type (e.g., 'module', 'component', 'test')" },
                                "docref": { "type": "string", "description": "Documentation reference (optional at spec stage)" },
                                "test_type": { "type": "string", "enum": ["unit", "integration", "e2e"], "description": "Test type — for test elements" },
                                "given": { "type": "string", "description": "Given precondition (BDD) — for test elements" },
                                "when": { "type": "string", "description": "When action (BDD) — for test elements" },
                                "then": { "type": "string", "description": "Then expected outcome (BDD) — for test elements" }
                            },
                            "required": ["text", "type"]
                        }
                    },
                    "relationships": { "type": "array" }
                },
                "required": ["id", "requirements"]
            }),
        }
    }

    fn sdd_generate_mindmap_plus() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_mindmap_plus".to_string(),
            description: "Generate Mermaid+ mindmap with YAML frontmatter and validation. Supports recursive node structure with shapes and icons.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Diagram identifier" },
                    "root": {
                        "type": "object",
                        "description": "Root node with recursive children",
                        "properties": {
                            "label": { "type": "string" },
                            "shape": { "type": "string", "enum": ["square", "rounded", "circle", "bang", "cloud", "hexagon"] },
                            "icon": { "type": "string" },
                            "children": { "type": "array" }
                        },
                        "required": ["label"]
                    }
                },
                "required": ["id", "root"]
            }),
        }
    }

    fn sdd_generate_journey_plus() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_journey_plus".to_string(),
            description: "Generate Mermaid+ user journey diagram with YAML frontmatter and validation. Validates score range (1-5) and actor requirements.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Diagram identifier" },
                    "title": { "type": "string" },
                    "sections": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "tasks": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "name": { "type": "string" },
                                            "score": { "type": "integer", "minimum": 1, "maximum": 5 },
                                            "actors": { "type": "array", "items": { "type": "string" }, "minItems": 1 }
                                        },
                                        "required": ["name", "score", "actors"]
                                    }
                                }
                            },
                            "required": ["name", "tasks"]
                        }
                    }
                },
                "required": ["id", "title", "sections"]
            }),
        }
    }

    fn sdd_generate_block_plus() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_block_plus".to_string(),
            description: "Generate Mermaid+ block-beta diagram with YAML frontmatter and validation. Supports column grid layout, nested/composite blocks, shapes, and edges.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Diagram identifier" },
                    "title": { "type": "string", "description": "Diagram title" },
                    "columns": { "type": "integer", "minimum": 1, "default": 1, "description": "Number of columns in grid layout" },
                    "blocks": {
                        "type": "array",
                        "description": "Block definitions (order determines grid position)",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string" },
                                "label": { "type": "string" },
                                "shape": { "type": "string", "enum": ["default", "round", "stadium", "diamond", "cylinder", "hexagon", "circle", "subroutine"] },
                                "width": { "type": "integer", "minimum": 1, "default": 1, "description": "Column span" },
                                "children": { "type": "array", "description": "Nested blocks for composite block" },
                                "child_columns": { "type": "integer", "description": "Column count for nested blocks" },
                                "metadata": { "type": "object", "description": "Arbitrary metadata for LLM routing" }
                            },
                            "required": ["id", "label"]
                        }
                    },
                    "edges": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "from": { "type": "string" },
                                "to": { "type": "string" },
                                "label": { "type": "string" },
                                "style": { "type": "string", "enum": ["arrow", "thick", "dotted"] }
                            },
                            "required": ["from", "to"]
                        }
                    }
                },
                "required": ["id", "blocks"]
            }),
        }
    }

    // === Spec Tools ===

    fn sdd_generate_openapi() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_openapi".to_string(),
            description: "Generate OpenAPI 3.1 specification from structured input. Use for REST API documentation.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "version": { "type": "string" },
                    "description": { "type": "string" },
                    "servers": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "url": { "type": "string" },
                                "description": { "type": "string" }
                            }
                        }
                    },
                    "paths": { "type": "object" },
                    "components": { "type": "object" }
                },
                "required": ["title", "version"]
            }),
        }
    }

    fn sdd_generate_asyncapi() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_asyncapi".to_string(),
            description: "Generate AsyncAPI 2.6 specification from structured input. Use for event-driven API documentation.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "version": { "type": "string" },
                    "description": { "type": "string" },
                    "servers": { "type": "object" },
                    "channels": { "type": "object" },
                    "components": { "type": "object" }
                },
                "required": ["title", "version"]
            }),
        }
    }

    fn sdd_generate_openrpc() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_openrpc".to_string(),
            description: "Generate OpenRPC 1.3 specification from structured input. Use for JSON-RPC and MCP tool documentation.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "version": { "type": "string" },
                    "description": { "type": "string" },
                    "methods": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "summary": { "type": "string" },
                                "params": { "type": "array" },
                                "result": { "type": "object" }
                            },
                            "required": ["name"]
                        }
                    }
                },
                "required": ["title", "version", "methods"]
            }),
        }
    }

    fn sdd_generate_serverless_workflow() -> ToolSchema {
        ToolSchema {
            name: "sdd_generate_serverless_workflow".to_string(),
            description: "Generate Serverless Workflow 0.8 specification from structured input. Use for workflow orchestration.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Workflow identifier" },
                    "name": { "type": "string", "description": "Workflow name" },
                    "start": { "type": "string", "description": "Name of the starting state" },
                    "states": {
                        "type": "array",
                        "description": "Workflow state definitions",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "type": { "type": "string", "enum": ["operation", "switch", "sleep", "parallel", "foreach", "inject", "event", "callback"] }
                            },
                            "required": ["name", "type"]
                        }
                    },
                    "version": { "type": "string", "description": "Workflow version (optional, defaults to 1.0.0)" },
                    "description": { "type": "string" },
                    "functions": { "type": "array", "description": "Function definitions" },
                    "errors": { "type": "array", "description": "Error definitions" }
                },
                "required": ["id", "name", "start", "states"]
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_count() {
        assert_eq!(SddTools::list().len(), SddTools::count());
    }

    #[test]
    fn test_all_tools_have_names() {
        for tool in SddTools::list() {
            assert!(tool.name.starts_with("sdd_generate_"));
            assert!(!tool.description.is_empty());
        }
    }
}

// CODEGEN-END
