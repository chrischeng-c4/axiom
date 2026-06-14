//! Mermaid diagram symbol extraction (line-based)
//!
//! Extracts symbols from Mermaid diagram source:
//! - Diagram type (first non-empty line) as Module
//! - Node IDs in flowchart/graph as Variable
//! - Edge labels (`A -->|label| B`) as Label
//! - Subgraph names (`subgraph Name`) as Module

use super::{SymbolKind, SymbolTableBuilder};
use crate::diagnostic::{Position, Range};

const FLOWCHART_TYPES: &[&str] = &["graph", "flowchart"];

impl SymbolTableBuilder {
    pub(crate) fn visit_mermaid_lines(&mut self, source: &str) {
        let mut diagram_type: Option<String> = None;
        let mut is_flowchart = false;

        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            let line_num = line_idx as u32;

            // Skip blank lines and comments
            if trimmed.is_empty() || trimmed.starts_with("%%") {
                continue;
            }

            // First meaningful line — diagram type declaration
            if diagram_type.is_none() {
                let dtype = trimmed.split_whitespace().next().unwrap_or(trimmed);
                let col = line.find(dtype).unwrap_or(0) as u32;
                self.table.add_symbol(
                    dtype.to_string(),
                    SymbolKind::Module,
                    mk_range(line_num, col, dtype.len()),
                    None,
                    Some("Mermaid diagram type".to_string()),
                    self.current_scope,
                );
                is_flowchart = FLOWCHART_TYPES.contains(&dtype);
                diagram_type = Some(dtype.to_string());
                continue;
            }

            // Subgraph declaration: `subgraph Name` or `subgraph Name[Title]`
            if trimmed.starts_with("subgraph") {
                self.extract_mermaid_subgraph(line, trimmed, line_num);
                continue;
            }

            // Skip `end` keyword and class/style directives
            if trimmed == "end"
                || trimmed.starts_with("style ")
                || trimmed.starts_with("classDef ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("click ")
            {
                continue;
            }

            // Flowchart-specific: extract node IDs and edge labels
            if is_flowchart {
                self.extract_mermaid_flowchart_line(line, trimmed, line_num);
            }
        }
    }

    /// Extract subgraph name from `subgraph Name` or `subgraph Name[Display Title]`
    fn extract_mermaid_subgraph(&mut self, raw_line: &str, trimmed: &str, line_num: u32) {
        // Format: `subgraph <id>` or `subgraph <id>[<title>]` or `subgraph [<title>]`
        let rest = trimmed["subgraph".len()..].trim();
        if rest.is_empty() {
            return;
        }
        // Name is the part before `[` (if any)
        let name = if let Some(bracket_pos) = rest.find('[') {
            rest[..bracket_pos].trim()
        } else {
            rest.trim()
        };
        if name.is_empty() {
            return;
        }
        let col = raw_line.find(name).unwrap_or(0) as u32;
        self.table.add_symbol(
            name.to_string(),
            SymbolKind::Module,
            mk_range(line_num, col, name.len()),
            None,
            Some("Mermaid subgraph".to_string()),
            self.current_scope,
        );
    }

    /// Extract node IDs and edge labels from a flowchart line
    fn extract_mermaid_flowchart_line(&mut self, raw_line: &str, trimmed: &str, line_num: u32) {
        let arrow_patterns = ["-->", "---", "-.-", "==>", "-.->", "--o", "--x", "<-->"];
        let has_arrow = arrow_patterns.iter().any(|a| trimmed.contains(a));

        if has_arrow {
            // Extract edge labels: `-->|label|` or `-- label -->`
            self.extract_mermaid_edge_labels(raw_line, trimmed, line_num);

            // Extract node IDs from both sides of arrows
            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            let mut i = 0;
            while i < tokens.len() {
                let tok = tokens[i];
                // Skip arrow tokens and edge label tokens `|...|`
                if tok.starts_with('|') || tok.ends_with('|') {
                    i += 1;
                    continue;
                }
                if arrow_patterns
                    .iter()
                    .any(|a| tok.contains(a) || tok.contains("->") || tok.contains("--"))
                {
                    i += 1;
                    continue;
                }
                if let Some(id) = extract_node_id(tok) {
                    // Only record IDs that look like node definitions (have bracket notation)
                    if tok.contains('[') || tok.contains('(') || tok.contains('{') {
                        let col = raw_line.find(id).unwrap_or(0) as u32;
                        self.table.add_symbol(
                            id.to_string(),
                            SymbolKind::Variable,
                            mk_range(line_num, col, id.len()),
                            None,
                            Some("Mermaid node".to_string()),
                            self.current_scope,
                        );
                    }
                }
                i += 1;
            }
        } else {
            // Standalone node definition line
            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            if let Some(&first_tok) = tokens.first() {
                if let Some(id) = extract_node_id(first_tok) {
                    // Only register as symbol if it has a label bracket
                    if first_tok.contains('[') || first_tok.contains('(') || first_tok.contains('{')
                    {
                        let col = raw_line.find(id).unwrap_or(0) as u32;
                        self.table.add_symbol(
                            id.to_string(),
                            SymbolKind::Variable,
                            mk_range(line_num, col, id.len()),
                            None,
                            Some("Mermaid node".to_string()),
                            self.current_scope,
                        );
                    }
                }
            }
        }
    }

    /// Extract edge labels from patterns like `-->|label|` and `-- text -->`
    fn extract_mermaid_edge_labels(&mut self, raw_line: &str, trimmed: &str, line_num: u32) {
        // Pattern 1: `-->|label|` — label between pipe characters
        let mut search = trimmed;
        while let Some(pipe_open) = search.find('|') {
            let after_open = &search[pipe_open + 1..];
            if let Some(pipe_close) = after_open.find('|') {
                let label = after_open[..pipe_close].trim();
                if !label.is_empty() {
                    let col = raw_line.find(label).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        label.to_string(),
                        SymbolKind::Label,
                        mk_range(line_num, col, label.len()),
                        None,
                        Some("Mermaid edge label".to_string()),
                        self.current_scope,
                    );
                }
                // Advance past closing pipe
                let advance = pipe_open + 1 + pipe_close + 1;
                if advance >= search.len() {
                    break;
                }
                search = &search[advance..];
            } else {
                break;
            }
        }
    }
}

/// Extract a node ID from a token like `A`, `A[text]`, `A(text)`, `A{text}`
fn extract_node_id(token: &str) -> Option<&str> {
    let end = token.find(|c| matches!(c, '[' | '(' | '{' | '>'));
    let id = match end {
        Some(pos) => &token[..pos],
        None => token,
    };
    if id.is_empty() {
        return None;
    }
    // Valid node IDs are alphanumeric + underscore/hyphen
    if id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        Some(id)
    } else {
        None
    }
}

fn mk_range(line: u32, col: u32, len: usize) -> Range {
    Range::new(
        Position::new(line, col),
        Position::new(line, col + len as u32),
    )
}

#[cfg(test)]
mod tests {
    use super::super::{SymbolKind, SymbolTableBuilder};

    fn build(source: &str) -> super::super::SymbolTable {
        SymbolTableBuilder::new().build_mermaid_from_source(source)
    }

    #[test]
    fn test_diagram_type_as_module() {
        let src = "flowchart LR\n    A[Start] --> B[End]\n";
        let table = build(src);
        let modules: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Module)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            modules.contains(&"flowchart"),
            "missing diagram type, got: {:?}",
            modules
        );
    }

    #[test]
    fn test_node_ids_extracted() {
        let src = "graph TD\n    A[Alpha]\n    B(Beta)\n    A --> B\n";
        let table = build(src);
        let nodes: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Variable)
            .map(|s| s.name.as_str())
            .collect();
        assert!(nodes.contains(&"A"), "missing node 'A', got: {:?}", nodes);
        assert!(nodes.contains(&"B"), "missing node 'B', got: {:?}", nodes);
    }

    #[test]
    fn test_edge_labels() {
        let src = "flowchart LR\n    A -->|yes| B\n    A -->|no| C\n";
        let table = build(src);
        let labels: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Label)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            labels.contains(&"yes"),
            "missing label 'yes', got: {:?}",
            labels
        );
        assert!(
            labels.contains(&"no"),
            "missing label 'no', got: {:?}",
            labels
        );
    }

    #[test]
    fn test_subgraph_names() {
        let src = "graph LR\n    subgraph Frontend\n        A[Page]\n    end\n    subgraph Backend\n        B[API]\n    end\n";
        let table = build(src);
        let subgraphs: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| {
                s.kind == SymbolKind::Module && s.doc.as_deref() == Some("Mermaid subgraph")
            })
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            subgraphs.contains(&"Frontend"),
            "missing 'Frontend', got: {:?}",
            subgraphs
        );
        assert!(
            subgraphs.contains(&"Backend"),
            "missing 'Backend', got: {:?}",
            subgraphs
        );
    }

    #[test]
    fn test_sequence_diagram_no_node_symbols() {
        // sequenceDiagram participants are not extracted as Variable nodes
        let src = "sequenceDiagram\n    Alice->>Bob: Hello\n    Bob-->>Alice: Hi\n";
        let table = build(src);
        let nodes: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Variable)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            nodes.is_empty(),
            "no Variable nodes expected for sequenceDiagram, got: {:?}",
            nodes
        );
        // But the diagram type should be recorded
        let modules: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Module)
            .map(|s| s.name.as_str())
            .collect();
        assert!(modules.contains(&"sequenceDiagram"), "got: {:?}", modules);
    }
}
