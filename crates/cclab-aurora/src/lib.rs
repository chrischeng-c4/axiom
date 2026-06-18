//! Diagram and specification generation primitives.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AuroraError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuroraError {
    #[error("diagram title must not be empty")]
    EmptyDiagramTitle,
    #[error("diagram must contain at least one node")]
    EmptyDiagram,
    #[error("specification title must not be empty")]
    EmptySpecificationTitle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagramNode {
    pub id: String,
    pub label: String,
}

impl DiagramNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagramEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

impl DiagramEdge {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: None,
        }
    }

    pub fn labeled(
        from: impl Into<String>,
        to: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: Some(label.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagramSpec {
    pub title: String,
    pub direction: MermaidDirection,
    pub nodes: Vec<DiagramNode>,
    pub edges: Vec<DiagramEdge>,
}

impl DiagramSpec {
    pub fn flowchart(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            direction: MermaidDirection::TopDown,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn with_direction(mut self, direction: MermaidDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_node(mut self, node: DiagramNode) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn with_edge(mut self, edge: DiagramEdge) -> Self {
        self.edges.push(edge);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MermaidDirection {
    TopDown,
    LeftRight,
}

impl MermaidDirection {
    fn as_mermaid(self) -> &'static str {
        match self {
            Self::TopDown => "TD",
            Self::LeftRight => "LR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecificationDoc {
    pub title: String,
    pub sections: Vec<SpecificationSection>,
}

impl SpecificationDoc {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            sections: Vec::new(),
        }
    }

    pub fn with_section(mut self, heading: impl Into<String>, body: impl Into<String>) -> Self {
        self.sections.push(SpecificationSection {
            heading: heading.into(),
            body: body.into(),
        });
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecificationSection {
    pub heading: String,
    pub body: String,
}

pub fn render_mermaid_flowchart(spec: &DiagramSpec) -> Result<String> {
    if spec.title.trim().is_empty() {
        return Err(AuroraError::EmptyDiagramTitle);
    }
    if spec.nodes.is_empty() {
        return Err(AuroraError::EmptyDiagram);
    }

    let mut output = format!(
        "---\ntitle: {}\n---\nflowchart {}\n",
        spec.title,
        spec.direction.as_mermaid()
    );
    for node in &spec.nodes {
        output.push_str(&format!(
            "    {}[\"{}\"]\n",
            node.id,
            escape_mermaid(&node.label)
        ));
    }
    for edge in &spec.edges {
        match &edge.label {
            Some(label) => output.push_str(&format!(
                "    {} -->|{}| {}\n",
                edge.from,
                escape_mermaid(label),
                edge.to
            )),
            None => output.push_str(&format!("    {} --> {}\n", edge.from, edge.to)),
        }
    }
    Ok(output)
}

pub fn render_markdown_spec(doc: &SpecificationDoc) -> Result<String> {
    if doc.title.trim().is_empty() {
        return Err(AuroraError::EmptySpecificationTitle);
    }

    let mut output = format!("# {}\n", doc.title);
    for section in &doc.sections {
        output.push_str(&format!("\n## {}\n\n{}\n", section.heading, section.body));
    }
    Ok(output)
}

fn escape_mermaid(input: &str) -> String {
    input.replace('"', "#quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_mermaid_flowchart_from_structured_input() {
        let diagram = DiagramSpec::flowchart("Request Flow")
            .with_direction(MermaidDirection::LeftRight)
            .with_node(DiagramNode::new("client", "Client"))
            .with_node(DiagramNode::new("api", "API"))
            .with_edge(DiagramEdge::labeled("client", "api", "HTTP"));

        let rendered = render_mermaid_flowchart(&diagram).unwrap();

        assert!(rendered.contains("title: Request Flow"));
        assert!(rendered.contains("flowchart LR"));
        assert!(rendered.contains("client[\"Client\"]"));
        assert!(rendered.contains("client -->|HTTP| api"));
    }

    #[test]
    fn renders_markdown_spec_from_sections() {
        let doc = SpecificationDoc::new("Payment API")
            .with_section("Request", "POST /payments")
            .with_section("Response", "201 Created");

        let rendered = render_markdown_spec(&doc).unwrap();

        assert!(rendered.contains("# Payment API"));
        assert!(rendered.contains("## Request"));
        assert!(rendered.contains("POST /payments"));
        assert!(rendered.contains("## Response"));
    }

    #[test]
    fn validates_required_inputs() {
        assert_eq!(
            render_mermaid_flowchart(&DiagramSpec::flowchart("")).unwrap_err(),
            AuroraError::EmptyDiagramTitle
        );
        assert_eq!(
            render_mermaid_flowchart(&DiagramSpec::flowchart("empty")).unwrap_err(),
            AuroraError::EmptyDiagram
        );
        assert_eq!(
            render_markdown_spec(&SpecificationDoc::new("")).unwrap_err(),
            AuroraError::EmptySpecificationTitle
        );
    }
}
