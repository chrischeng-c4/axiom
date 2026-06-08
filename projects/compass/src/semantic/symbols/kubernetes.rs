//! Kubernetes manifest symbol extraction (YAML tree-sitter)
//!
//! Extracts: resources (kind + metadata.name), labels, selectors.

use super::{SymbolKind, SymbolTableBuilder};
use crate::diagnostic::Range;
use crate::syntax::ParsedFile;

impl SymbolTableBuilder {
    /// Walk YAML AST to extract Kubernetes resource symbols
    pub(crate) fn visit_k8s_node(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if node.is_error() || node.is_missing() {
            return;
        }
        match node.kind() {
            "stream" | "document" => {
                let mut c = node.walk();
                for child in node.children(&mut c) {
                    if !child.is_error() && !child.is_missing() {
                        self.visit_k8s_node(&child, file);
                    }
                }
            }
            "block_mapping" => self.visit_k8s_mapping(node, file),
            _ => {
                let mut c = node.walk();
                for child in node.children(&mut c) {
                    if !child.is_error() && !child.is_missing() {
                        self.visit_k8s_node(&child, file);
                    }
                }
            }
        }
    }

    /// Process a top-level YAML mapping to extract K8s resource info
    fn visit_k8s_mapping(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let (mut kind_val, mut name_val, mut ns_val) =
            (String::new(), String::new(), String::new());
        let mut name_range = Range::default();
        let mut labels: Vec<(String, Range)> = Vec::new();
        let mut selectors: Vec<(String, Range)> = Vec::new();

        let mut c = node.walk();
        for child in node.children(&mut c) {
            if child.kind() != "block_mapping_pair" {
                continue;
            }
            let key = match child.child_by_field_name("key") {
                Some(k) => file.node_text(&k).trim().to_string(),
                None => continue,
            };
            let vn = child.child_by_field_name("value");
            match key.as_str() {
                "kind" => {
                    if let Some(v) = &vn {
                        kind_val = file.node_text(v).trim().into();
                    }
                }
                "metadata" => {
                    if let Some(v) = &vn {
                        extract_metadata(
                            v,
                            file,
                            &mut name_val,
                            &mut name_range,
                            &mut ns_val,
                            &mut labels,
                        );
                    }
                }
                "spec" => {
                    if let Some(v) = &vn {
                        extract_spec_selectors(v, file, &mut selectors);
                    }
                }
                _ => {}
            }
        }

        if !name_val.is_empty() {
            let full = if ns_val.is_empty() {
                if kind_val.is_empty() {
                    name_val.clone()
                } else {
                    format!("{}/{}", kind_val, name_val)
                }
            } else {
                format!("{}/{}/{}", ns_val, kind_val, name_val)
            };
            self.table.add_symbol(
                full,
                SymbolKind::Resource,
                name_range,
                None,
                Some(format!("kind: {}", kind_val)),
                self.current_scope,
            );
        }
        for (l, r) in labels {
            self.table.add_symbol(
                l,
                SymbolKind::Label,
                r,
                None,
                Some("metadata label".into()),
                self.current_scope,
            );
        }
        for (s, r) in selectors {
            self.table.add_symbol(
                s,
                SymbolKind::Selector,
                r,
                None,
                Some("selector matchLabel".into()),
                self.current_scope,
            );
        }
    }
}

/// Extract name, namespace, labels from metadata mapping
fn extract_metadata(
    node: &tree_sitter::Node<'_>,
    file: &ParsedFile,
    name: &mut String,
    name_range: &mut Range,
    ns: &mut String,
    labels: &mut Vec<(String, Range)>,
) {
    let Some(mapping) = find_mapping(node) else {
        return;
    };
    let mut c = mapping.walk();
    for child in mapping.children(&mut c) {
        if child.kind() != "block_mapping_pair" {
            continue;
        }
        let key = match child.child_by_field_name("key") {
            Some(k) => file.node_text(&k).trim().to_string(),
            None => continue,
        };
        let vn = child.child_by_field_name("value");
        match key.as_str() {
            "name" => {
                if let Some(v) = &vn {
                    *name = file.node_text(v).trim().into();
                    *name_range = Range::from_node(v);
                }
            }
            "namespace" => {
                if let Some(v) = &vn {
                    *ns = file.node_text(v).trim().into();
                }
            }
            "labels" => {
                if let Some(v) = &vn {
                    collect_mapping_keys(v, file, labels);
                }
            }
            _ => {}
        }
    }
}

/// Extract matchLabels from spec.selector
fn extract_spec_selectors(
    node: &tree_sitter::Node<'_>,
    file: &ParsedFile,
    selectors: &mut Vec<(String, Range)>,
) {
    let Some(mapping) = find_mapping(node) else {
        return;
    };
    let mut c = mapping.walk();
    for child in mapping.children(&mut c) {
        if child.kind() != "block_mapping_pair" {
            continue;
        }
        let key = match child.child_by_field_name("key") {
            Some(k) => file.node_text(&k).trim().to_string(),
            None => continue,
        };
        if key != "selector" {
            continue;
        }
        if let Some(v) = child.child_by_field_name("value") {
            if let Some(sm) = find_mapping(&v) {
                let mut sc = sm.walk();
                for pair in sm.children(&mut sc) {
                    if pair.kind() != "block_mapping_pair" {
                        continue;
                    }
                    let pk = match pair.child_by_field_name("key") {
                        Some(k) => file.node_text(&k).trim().to_string(),
                        None => continue,
                    };
                    if pk == "matchLabels" {
                        if let Some(ml) = pair.child_by_field_name("value") {
                            collect_mapping_keys(&ml, file, selectors);
                        }
                    }
                }
            }
        }
    }
}

/// Find a block_mapping child (may be nested under block_node)
fn find_mapping<'a>(node: &tree_sitter::Node<'a>) -> Option<tree_sitter::Node<'a>> {
    if node.kind() == "block_mapping" {
        return Some(*node);
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        if child.kind() == "block_mapping" {
            return Some(child);
        }
        if child.kind() == "block_node" {
            let mut ic = child.walk();
            for gc in child.children(&mut ic) {
                if gc.kind() == "block_mapping" {
                    return Some(gc);
                }
            }
        }
    }
    None
}

/// Collect key names from a flat YAML mapping
fn collect_mapping_keys(
    node: &tree_sitter::Node<'_>,
    file: &ParsedFile,
    out: &mut Vec<(String, Range)>,
) {
    let Some(mapping) = find_mapping(node) else {
        return;
    };
    let mut c = mapping.walk();
    for child in mapping.children(&mut c) {
        if child.kind() == "block_mapping_pair" {
            if let Some(k) = child.child_by_field_name("key") {
                let kn = file.node_text(&k).trim().to_string();
                if !kn.is_empty() {
                    out.push((kn, Range::from_node(&k)));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{SymbolKind, SymbolTableBuilder};
    use crate::syntax::{Language, MultiParser};

    #[test]
    fn test_k8s_deployment() {
        let source = "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: nginx-deploy\n  namespace: production\n  labels:\n    app: nginx\n    env: prod\nspec:\n  selector:\n    matchLabels:\n      app: nginx\n";
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Yaml).unwrap();
        let table = SymbolTableBuilder::new().build_kubernetes(&parsed);
        let syms = table.all_symbols();

        let res: Vec<&str> = syms
            .iter()
            .filter(|s| s.kind == SymbolKind::Resource)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            res.contains(&"production/Deployment/nginx-deploy"),
            "got: {:?}",
            res
        );

        let lbl: Vec<&str> = syms
            .iter()
            .filter(|s| s.kind == SymbolKind::Label)
            .map(|s| s.name.as_str())
            .collect();
        assert!(lbl.contains(&"app"), "got: {:?}", lbl);
        assert!(lbl.contains(&"env"), "got: {:?}", lbl);
    }
}
