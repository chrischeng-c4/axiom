// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/generator.md#source
// CODEGEN-BEGIN
//! Flowchart+ generator
//!
//! Generates Mermaid+ output from validated flowchart definitions.
//! Mermaid+ = YAML frontmatter (structured definition) + Mermaid diagram

use super::schema::{
    EdgeDef, EdgeStyle, FlowDirection, FlowchartDef, NodeDef, NodeShape, SemanticType,
};
use super::validator::FlowchartValidationResult;
use indexmap::IndexMap;
use std::collections::{HashSet, VecDeque};

use serde::Serialize;

/// Mermaid+ generator output bundle.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/generator.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct FlowchartPlusOutput {
    /// YAML frontmatter content (without --- markers).
    pub frontmatter: String,
    /// Mermaid diagram content (without ```mermaid``` markers).
    pub diagram: String,
    /// Validation result.
    pub validation: FlowchartValidationResult,
    /// Combined Mermaid+ format (ready to embed in markdown).
    pub combined: String,
}

/// Mermaid+ generator (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/generator.md#schema
pub struct FlowchartPlusGenerator;
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/generator.md#source
impl FlowchartPlusGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate Mermaid+ output from a flowchart definition
    pub fn generate(
        &self,
        flowchart: &FlowchartDef,
        validation: FlowchartValidationResult,
    ) -> Result<FlowchartPlusOutput, String> {
        // Generate YAML frontmatter
        let frontmatter = self.generate_frontmatter(flowchart)?;

        // Generate Mermaid diagram
        let diagram = self.generate_mermaid(flowchart)?;

        // Combine into Mermaid+ format (frontmatter inside code block per Mermaid spec)
        let mut combined = String::new();
        combined.push_str("```mermaid\n");
        combined.push_str("---\n");
        combined.push_str(&frontmatter);
        combined.push_str("---\n");
        combined.push_str(&diagram);
        combined.push_str("```\n");

        // Add validation warnings as HTML comments
        if !validation.warnings.is_empty() {
            combined.push_str("\n<!-- Validation Warnings:\n");
            for w in &validation.warnings {
                combined.push_str(&format!("  - {}: {} (at {})\n", w.code, w.message, w.path));
            }
            combined.push_str("-->\n");
        }

        Ok(FlowchartPlusOutput {
            frontmatter,
            diagram,
            validation,
            combined,
        })
    }

    /// Generate YAML frontmatter from flowchart definition (nodes in topological order)
    fn generate_frontmatter(&self, flowchart: &FlowchartDef) -> Result<String, String> {
        let ordered = self.reorder_nodes(flowchart);
        let yaml = serde_yaml::to_string(&ordered)
            .map_err(|e| format!("YAML serialization error: {}", e))?;

        // Strip leading "---\n" if present
        let yaml = yaml.strip_prefix("---\n").unwrap_or(&yaml);

        Ok(yaml.to_string())
    }

    /// Return a copy of the flowchart with nodes reordered by topological sort
    fn reorder_nodes(&self, flowchart: &FlowchartDef) -> FlowchartDef {
        let sorted_keys = self.topo_sort_node_keys(flowchart);
        let mut ordered_nodes = IndexMap::with_capacity(flowchart.nodes.len());
        for key in &sorted_keys {
            if let Some(node) = flowchart.nodes.get(key) {
                ordered_nodes.insert(key.clone(), node.clone());
            }
        }
        FlowchartDef {
            id: flowchart.id.clone(),
            direction: flowchart.direction.clone(),
            nodes: ordered_nodes,
            edges: flowchart.edges.clone(),
            subgraphs: flowchart.subgraphs.clone(),
            description: flowchart.description.clone(),
        }
    }

    /// Topological sort of node keys derived from edges.
    /// Roots: nodes with Start semantic, else nodes with no incoming edges.
    /// Nodes not reached by edges are appended at the end.
    fn topo_sort_node_keys(&self, flowchart: &FlowchartDef) -> Vec<String> {
        let all_keys: Vec<String> = flowchart.nodes.keys().cloned().collect();
        if all_keys.is_empty() {
            return vec![];
        }

        // Build adjacency list and in-degree from edges
        let mut adj: IndexMap<String, Vec<String>> = IndexMap::new();
        let mut in_degree: IndexMap<String, usize> = IndexMap::new();
        for key in &all_keys {
            adj.entry(key.clone()).or_default();
            in_degree.entry(key.clone()).or_insert(0);
        }
        for edge in &flowchart.edges {
            if flowchart.nodes.contains_key(&edge.from) && flowchart.nodes.contains_key(&edge.to) {
                adj.entry(edge.from.clone())
                    .or_default()
                    .push(edge.to.clone());
                *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
            }
        }

        // Find roots: prefer nodes with Start semantic
        let mut roots: Vec<String> = flowchart
            .nodes
            .iter()
            .filter(|(_, n)| matches!(n.semantic, Some(SemanticType::Start)))
            .map(|(k, _)| k.clone())
            .collect();

        // Fallback: nodes with in-degree 0
        if roots.is_empty() {
            roots = in_degree
                .iter()
                .filter(|(_, &deg)| deg == 0)
                .map(|(k, _)| k.clone())
                .collect();
        }

        // BFS topological sort (Kahn's algorithm)
        let mut queue: VecDeque<String> = roots.into_iter().collect();
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

        // Append any remaining nodes not reached (orphans / cycles)
        for key in &all_keys {
            if !visited.contains(key) {
                result.push(key.clone());
            }
        }

        result
    }

    /// Generate Mermaid flowchart from definition (nodes in topological order)
    pub fn generate_mermaid(&self, flowchart: &FlowchartDef) -> Result<String, String> {
        let mut mermaid = String::new();

        // Direction
        let direction = match flowchart.direction {
            FlowDirection::TB => "TB",
            FlowDirection::BT => "BT",
            FlowDirection::LR => "LR",
            FlowDirection::RL => "RL",
        };
        mermaid.push_str(&format!("flowchart {}\n", direction));

        // Collect nodes in subgraphs
        let mut subgraph_nodes: HashSet<String> = HashSet::new();
        for sg in &flowchart.subgraphs {
            for node_id in &sg.nodes {
                subgraph_nodes.insert(node_id.clone());
            }
        }

        // Generate subgraphs
        for sg in &flowchart.subgraphs {
            mermaid.push_str(&format!("    subgraph {}[\"{}\"]\n", sg.id, sg.label));
            for node_id in &sg.nodes {
                if let Some(node) = flowchart.nodes.get(node_id) {
                    mermaid.push_str(&format!("        {}\n", self.format_node(node_id, node)?));
                }
            }
            mermaid.push_str("    end\n");
        }

        // Generate standalone nodes in topological order
        let sorted_keys = self.topo_sort_node_keys(flowchart);
        for node_id in &sorted_keys {
            if subgraph_nodes.contains(node_id) {
                continue;
            }
            if let Some(node) = flowchart.nodes.get(node_id) {
                mermaid.push_str(&format!("    {}\n", self.format_node(node_id, node)?));
            }
        }

        // Generate edges
        for edge in &flowchart.edges {
            mermaid.push_str(&format!("    {}\n", self.format_edge(edge)?));
        }

        Ok(mermaid)
    }

    /// Format a node based on its shape
    fn format_node(&self, id: &str, node: &NodeDef) -> Result<String, String> {
        let escaped_label = node.label.replace('"', "#quot;");

        let node_str = match node.shape {
            NodeShape::Rectangle => format!("{}[{}]", id, escaped_label),
            NodeShape::Rounded => format!("{}({})", id, escaped_label),
            NodeShape::Stadium => format!("{}([{}])", id, escaped_label),
            NodeShape::Subroutine => format!("{}[[{}]]", id, escaped_label),
            NodeShape::Cylinder => format!("{}[({})]", id, escaped_label),
            NodeShape::Circle => format!("{}(({}))", id, escaped_label),
            NodeShape::Diamond => format!("{}{{{}}} ", id, escaped_label),
            NodeShape::Hexagon => format!("{}{{{{{}}}}}", id, escaped_label),
            NodeShape::Parallelogram => format!("{}[/{}\\]", id, escaped_label),
            NodeShape::Trapezoid => format!("{}[\\{}/]", id, escaped_label),
        };

        Ok(node_str)
    }

    /// Format an edge based on its style
    fn format_edge(&self, edge: &EdgeDef) -> Result<String, String> {
        let edge_str = match (&edge.style, &edge.label) {
            (EdgeStyle::Arrow, Some(lbl)) => {
                format!("{} -->|{}| {}", edge.from, lbl, edge.to)
            }
            (EdgeStyle::Arrow, None) => format!("{} --> {}", edge.from, edge.to),
            (EdgeStyle::Thick, Some(lbl)) => {
                format!("{} ==>|{}| {}", edge.from, lbl, edge.to)
            }
            (EdgeStyle::Thick, None) => format!("{} ==> {}", edge.from, edge.to),
            (EdgeStyle::Dotted, Some(lbl)) => {
                format!("{} -.->|{}| {}", edge.from, lbl, edge.to)
            }
            (EdgeStyle::Dotted, None) => format!("{} -.-> {}", edge.from, edge.to),
        };

        Ok(edge_str)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/generator.md#source
impl Default for FlowchartPlusGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::validator::FlowchartValidator;
    use super::*;
    use serde_json::json;

    fn parse_flowchart(json: serde_json::Value) -> FlowchartDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_generate_simple_mermaid() {
        let flowchart = parse_flowchart(json!({
            "id": "test",
            "direction": "TB",
            "nodes": {
                "start": { "label": "Start", "shape": "rounded" },
                "process": { "label": "Process", "shape": "rectangle" },
                "end": { "label": "End", "shape": "rounded" }
            },
            "edges": [
                { "from": "start", "to": "process" },
                { "from": "process", "to": "end" }
            ]
        }));

        let validation = FlowchartValidator::new().validate(&flowchart);
        let output = FlowchartPlusGenerator::new()
            .generate(&flowchart, validation)
            .unwrap();

        assert!(output.diagram.contains("flowchart TB"));
        assert!(output.diagram.contains("start(Start)"));
        assert!(output.diagram.contains("process[Process]"));
        assert!(output.diagram.contains("end(End)"));
        assert!(output.diagram.contains("start --> process"));
        assert!(output.diagram.contains("process --> end"));
    }

    #[test]
    fn test_generate_with_subgraph() {
        let flowchart = parse_flowchart(json!({
            "id": "grouped",
            "direction": "LR",
            "nodes": {
                "a": { "label": "A" },
                "b": { "label": "B" },
                "c": { "label": "C" }
            },
            "edges": [
                { "from": "a", "to": "b" },
                { "from": "b", "to": "c" }
            ],
            "subgraphs": [
                { "id": "sg1", "label": "Group 1", "nodes": ["a", "b"] }
            ]
        }));

        let validation = FlowchartValidator::new().validate(&flowchart);
        let output = FlowchartPlusGenerator::new()
            .generate(&flowchart, validation)
            .unwrap();

        assert!(output.diagram.contains("subgraph sg1[\"Group 1\"]"));
        assert!(output.diagram.contains("end"));
    }

    #[test]
    fn test_generate_with_edge_labels() {
        let flowchart = parse_flowchart(json!({
            "id": "labeled",
            "nodes": {
                "a": { "label": "A" },
                "b": { "label": "B" },
                "c": { "label": "C" }
            },
            "edges": [
                { "from": "a", "to": "b", "label": "yes", "style": "arrow" },
                { "from": "a", "to": "c", "label": "no", "style": "dotted" }
            ]
        }));

        let validation = FlowchartValidator::new().validate(&flowchart);
        let output = FlowchartPlusGenerator::new()
            .generate(&flowchart, validation)
            .unwrap();

        assert!(output.diagram.contains("a -->|yes| b"));
        assert!(output.diagram.contains("a -.->|no| c"));
    }

    #[test]
    fn test_topological_node_order() {
        // Nodes deliberately in reverse order in JSON
        let flowchart = parse_flowchart(json!({
            "id": "order-test",
            "nodes": {
                "end": { "label": "End", "semantic": { "type": "end" } },
                "process": { "label": "Process" },
                "start": { "label": "Start", "semantic": { "type": "start" } }
            },
            "edges": [
                { "from": "start", "to": "process" },
                { "from": "process", "to": "end" }
            ]
        }));

        let gen = FlowchartPlusGenerator::new();
        let sorted = gen.topo_sort_node_keys(&flowchart);
        assert_eq!(sorted, vec!["start", "process", "end"]);

        // Verify Mermaid output order
        let validation = FlowchartValidator::new().validate(&flowchart);
        let output = gen.generate(&flowchart, validation).unwrap();
        let lines: Vec<&str> = output.diagram.lines().collect();
        // lines[0] = "flowchart TB", lines[1..] = nodes + edges
        let node_lines: Vec<&str> = lines
            .iter()
            .filter(|l| !l.contains("-->") && !l.contains("flowchart"))
            .copied()
            .collect();
        // start should come before process, process before end
        let start_pos = node_lines.iter().position(|l| l.contains("start")).unwrap();
        let process_pos = node_lines
            .iter()
            .position(|l| l.contains("process"))
            .unwrap();
        let end_pos = node_lines.iter().position(|l| l.contains("end")).unwrap();
        assert!(start_pos < process_pos);
        assert!(process_pos < end_pos);
    }

    #[test]
    fn test_mermaid_plus_format() {
        let flowchart = parse_flowchart(json!({
            "id": "simple",
            "nodes": {
                "a": { "label": "A", "semantic": { "type": "start" } },
                "b": { "label": "B", "semantic": { "type": "end" } }
            },
            "edges": [
                { "from": "a", "to": "b" }
            ]
        }));

        let validation = FlowchartValidator::new().validate(&flowchart);
        let output = FlowchartPlusGenerator::new()
            .generate(&flowchart, validation)
            .unwrap();

        // Check combined format (frontmatter inside code block)
        assert!(output.combined.starts_with("```mermaid\n---\n"));
        assert!(output.combined.contains("id: simple"));
        assert!(output.combined.contains("flowchart TB"));
    }
}

// CODEGEN-END
