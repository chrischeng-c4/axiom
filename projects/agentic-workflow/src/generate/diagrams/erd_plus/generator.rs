// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/generator.md#source
// CODEGEN-BEGIN
//! ERD+ generator

use super::schema::{Cardinality, ERDDef, KeyType};
use super::validator::ERDValidationResult;
use indexmap::IndexMap;
use std::collections::{HashSet, VecDeque};

use serde::Serialize;

/// ERD+ generator output bundle.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/generator.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct ERDPlusOutput {
    /// YAML frontmatter.
    pub frontmatter: String,
    /// Mermaid ERD diagram.
    pub diagram: String,
    /// Validation result.
    pub validation: ERDValidationResult,
    /// Combined frontmatter + diagram output.
    pub combined: String,
}

/// ERD+ generator (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/generator.md#schema
pub struct ERDPlusGenerator;
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/generator.md#source
impl ERDPlusGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(
        &self,
        erd: &ERDDef,
        validation: ERDValidationResult,
    ) -> Result<ERDPlusOutput, String> {
        let frontmatter = self.generate_frontmatter(erd)?;
        let mermaid = self.generate_mermaid(erd)?;

        // Combine into Mermaid+ format (frontmatter inside code block per Mermaid spec)
        let mut combined = String::new();
        combined.push_str("```mermaid\n");
        combined.push_str("---\n");
        combined.push_str(&frontmatter);
        combined.push_str("---\n");
        combined.push_str(&mermaid);
        combined.push_str("```\n");

        if !validation.warnings.is_empty() {
            combined.push_str("\n<!-- Validation Warnings:\n");
            for w in &validation.warnings {
                combined.push_str(&format!("  - {}: {} (at {})\n", w.code, w.message, w.path));
            }
            combined.push_str("-->\n");
        }

        Ok(ERDPlusOutput {
            frontmatter,
            diagram: mermaid,
            validation,
            combined,
        })
    }

    fn generate_frontmatter(&self, erd: &ERDDef) -> Result<String, String> {
        let reordered = self.reorder_entities(erd);
        let yaml = serde_yaml::to_string(&reordered).map_err(|e| format!("YAML error: {}", e))?;
        Ok(yaml.strip_prefix("---\n").unwrap_or(&yaml).to_string())
    }

    /// Return a copy of the ERD with entities reordered by dependency (referenced entities first)
    fn reorder_entities(&self, erd: &ERDDef) -> ERDDef {
        let sorted_keys = self.topo_sort_entity_keys(erd);
        let mut ordered = IndexMap::with_capacity(erd.entities.len());
        for key in &sorted_keys {
            if let Some(entity) = erd.entities.get(key) {
                ordered.insert(key.clone(), entity.clone());
            }
        }
        ERDDef {
            id: erd.id.clone(),
            entities: ordered,
            relationships: erd.relationships.clone(),
            description: erd.description.clone(),
        }
    }

    /// Topological sort of entity keys based on FK references.
    /// Entities that are referenced by FK come first (they are dependencies).
    fn topo_sort_entity_keys(&self, erd: &ERDDef) -> Vec<String> {
        let all_keys: Vec<String> = erd.entities.keys().cloned().collect();
        if all_keys.is_empty() {
            return vec![];
        }

        // Build adjacency: if entity A has FK referencing entity B, then B -> A (B must come first)
        let mut adj: IndexMap<String, Vec<String>> = IndexMap::new();
        let mut in_degree: IndexMap<String, usize> = IndexMap::new();
        for key in &all_keys {
            adj.entry(key.clone()).or_default();
            in_degree.entry(key.clone()).or_insert(0);
        }
        for (entity_name, entity_def) in &erd.entities {
            for attr in &entity_def.attributes {
                if let Some(ref reference) = attr.references {
                    if let Some((ref_entity, _)) = reference.split_once('.') {
                        if erd.entities.contains_key(ref_entity) && ref_entity != entity_name {
                            adj.entry(ref_entity.to_string())
                                .or_default()
                                .push(entity_name.clone());
                            *in_degree.entry(entity_name.clone()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(k, _)| k.clone())
            .collect();
        let mut visited: HashSet<String> = HashSet::new();
        let mut result: Vec<String> = Vec::with_capacity(all_keys.len());

        while let Some(node) = queue.pop_front() {
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node.clone());
            result.push(node.clone());

            if let Some(neighbors) = adj.get(&node) {
                for next in neighbors {
                    if !visited.contains(next) {
                        let deg = in_degree.get_mut(next).unwrap();
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(next.clone());
                        }
                    }
                }
            }
        }

        // Append any remaining (cycles / orphans)
        for key in &all_keys {
            if !visited.contains(key) {
                result.push(key.clone());
            }
        }

        result
    }

    pub fn generate_mermaid(&self, erd: &ERDDef) -> Result<String, String> {
        let mut mermaid = String::new();
        mermaid.push_str("erDiagram\n");

        // Generate entities in dependency order
        let sorted_keys = self.topo_sort_entity_keys(erd);

        for entity_name in &sorted_keys {
            let entity_def = match erd.entities.get(entity_name) {
                Some(e) => e,
                None => continue,
            };
            mermaid.push_str(&format!("    {} {{\n", entity_name));
            for attr in &entity_def.attributes {
                let key_str = attr.key.as_ref().map_or("", |k| match k {
                    KeyType::PK => " PK",
                    KeyType::FK => " FK",
                    KeyType::UK => " UK",
                });
                let comment = attr
                    .comment
                    .as_ref()
                    .map_or(String::new(), |c| format!(" \"{}\"", c));
                mermaid.push_str(&format!(
                    "        {}{} {}{}\n",
                    attr.data_type, key_str, attr.name, comment
                ));
            }
            mermaid.push_str("    }\n");
        }

        // Generate relationships
        for rel in &erd.relationships {
            let (left, right) = self.cardinality_to_symbols(&rel.cardinality);
            let label = rel.label.as_deref().unwrap_or("");
            mermaid.push_str(&format!(
                "    {} {}--{} {} : {}\n",
                rel.from, left, right, rel.to, label
            ));
        }

        Ok(mermaid)
    }

    fn cardinality_to_symbols(&self, cardinality: &Cardinality) -> (&'static str, &'static str) {
        match cardinality {
            Cardinality::OneToOne => ("||", "||"),
            Cardinality::OneToMany => ("||", "o{"),
            Cardinality::ManyToOne => ("}o", "||"),
            Cardinality::ManyToMany => ("}o", "o{"),
            Cardinality::OneOrMoreToOne => ("|{", "||"),
            Cardinality::OneToOneOrMore => ("||", "}|"),
            Cardinality::ZeroOrOneToOne => ("|o", "||"),
            Cardinality::OneToZeroOrOne => ("||", "o|"),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/generator.md#source
impl Default for ERDPlusGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::validator::ERDValidator;
    use super::*;
    use serde_json::json;

    fn parse_erd(json: serde_json::Value) -> ERDDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_generate_erd() {
        let erd = parse_erd(json!({
            "id": "test",
            "entities": {
                "User": {
                    "attributes": [
                        { "name": "id", "type": "int", "key": "PK" },
                        { "name": "email", "type": "string" }
                    ]
                },
                "Order": {
                    "attributes": [
                        { "name": "id", "type": "int", "key": "PK" },
                        { "name": "user_id", "type": "int", "key": "FK" }
                    ]
                }
            },
            "relationships": [
                { "from": "User", "to": "Order", "cardinality": "one-to-many", "label": "places" }
            ]
        }));

        let validation = ERDValidator::new().validate(&erd);
        let output = ERDPlusGenerator::new().generate(&erd, validation).unwrap();

        assert!(output.diagram.contains("erDiagram"));
        assert!(output.diagram.contains("User {"));
        assert!(output.diagram.contains("int PK id"));
        assert!(output.diagram.contains("User ||--o{ Order : places"));
    }

    #[test]
    fn test_topological_entity_order() {
        // Order deliberately has dependent entity first in JSON
        let erd = parse_erd(json!({
            "id": "order-test",
            "entities": {
                "OrderItem": {
                    "attributes": [
                        { "name": "id", "type": "UUID", "key": "PK" },
                        { "name": "order_id", "type": "UUID", "key": "FK", "references": "Order.id" },
                        { "name": "product_id", "type": "UUID", "key": "FK", "references": "Product.id" }
                    ]
                },
                "Order": {
                    "attributes": [
                        { "name": "id", "type": "UUID", "key": "PK" },
                        { "name": "user_id", "type": "UUID", "key": "FK", "references": "User.id" }
                    ]
                },
                "User": {
                    "attributes": [
                        { "name": "id", "type": "UUID", "key": "PK" },
                        { "name": "email", "type": "VARCHAR" }
                    ]
                },
                "Product": {
                    "attributes": [
                        { "name": "id", "type": "UUID", "key": "PK" },
                        { "name": "name", "type": "VARCHAR" }
                    ]
                }
            }
        }));

        let gen = ERDPlusGenerator::new();
        let sorted = gen.topo_sort_entity_keys(&erd);

        // User and Product have no FK deps, should come before Order
        // Order depends on User, should come before OrderItem
        // OrderItem depends on Order and Product, should be last
        let user_pos = sorted.iter().position(|k| k == "User").unwrap();
        let product_pos = sorted.iter().position(|k| k == "Product").unwrap();
        let order_pos = sorted.iter().position(|k| k == "Order").unwrap();
        let item_pos = sorted.iter().position(|k| k == "OrderItem").unwrap();

        assert!(user_pos < order_pos, "User should come before Order");
        assert!(
            product_pos < item_pos,
            "Product should come before OrderItem"
        );
        assert!(order_pos < item_pos, "Order should come before OrderItem");
    }
}

// CODEGEN-END
