//! Terraform/HCL symbol extraction via tree-sitter-hcl
//!
//! Extracts symbols from HCL ASTs:
//! - Resources: `resource "type" "name" { ... }`
//! - Data sources: `data "type" "name" { ... }`
//! - Variables: `variable "name" { ... }`
//! - Outputs: `output "name" { ... }`
//! - Locals: `locals { name = ... }`
//! - Modules: `module "name" { ... }`

use crate::diagnostic::Range;
use crate::syntax::ParsedFile;

use super::{SymbolKind, SymbolTableBuilder};

impl SymbolTableBuilder {
    /// Walk HCL AST to extract Terraform symbols
    pub(crate) fn visit_hcl_node(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        if node.is_error() || node.is_missing() {
            return;
        }

        if node.kind() == "block" {
            self.visit_hcl_block(node, file);
            return;
        }

        // Recurse children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if !child.is_error() && !child.is_missing() {
                self.visit_hcl_node(&child, file);
            }
        }
    }

    /// Process an HCL block: `block_type label1 label2 { body }`
    fn visit_hcl_block(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        // Collect block type and string labels
        let mut cursor = node.walk();
        let children: Vec<tree_sitter::Node<'_>> = node.children(&mut cursor).collect();

        if children.is_empty() {
            return;
        }

        // First child is the block type identifier
        let block_type_node = &children[0];
        let block_type = file.node_text(block_type_node);

        // Collect string literal labels (removing quotes)
        let labels: Vec<String> = children[1..]
            .iter()
            .filter(|c| c.kind() == "string_lit" || c.kind() == "identifier")
            .map(|c| strip_quotes(file.node_text(c)))
            .collect();

        match block_type {
            "resource" => {
                // resource "aws_instance" "web" { ... }
                if labels.len() >= 2 {
                    let resource_type = &labels[0];
                    let resource_name = &labels[1];
                    let full_name = format!("{}.{}", resource_type, resource_name);
                    self.table.add_symbol(
                        full_name,
                        SymbolKind::Resource,
                        Range::from_node(node),
                        None,
                        Some(format!(
                            "resource \"{}\" \"{}\"",
                            resource_type, resource_name
                        )),
                        self.current_scope,
                    );
                }
            }
            "data" => {
                // data "aws_ami" "ubuntu" { ... }
                if labels.len() >= 2 {
                    let data_type = &labels[0];
                    let data_name = &labels[1];
                    let full_name = format!("data.{}.{}", data_type, data_name);
                    self.table.add_symbol(
                        full_name,
                        SymbolKind::Resource,
                        Range::from_node(node),
                        None,
                        Some(format!("data \"{}\" \"{}\"", data_type, data_name)),
                        self.current_scope,
                    );
                }
            }
            "variable" => {
                // variable "instance_type" { ... }
                if let Some(var_name) = labels.first() {
                    self.table.add_symbol(
                        format!("var.{}", var_name),
                        SymbolKind::Variable,
                        Range::from_node(node),
                        None,
                        Some(format!("variable \"{}\"", var_name)),
                        self.current_scope,
                    );
                }
            }
            "output" => {
                // output "instance_ip" { ... }
                if let Some(out_name) = labels.first() {
                    self.table.add_symbol(
                        format!("output.{}", out_name),
                        SymbolKind::Variable,
                        Range::from_node(node),
                        None,
                        Some(format!("output \"{}\"", out_name)),
                        self.current_scope,
                    );
                }
            }
            "module" => {
                // module "vpc" { ... }
                if let Some(mod_name) = labels.first() {
                    self.table.add_symbol(
                        format!("module.{}", mod_name),
                        SymbolKind::Module,
                        Range::from_node(node),
                        None,
                        Some(format!("module \"{}\"", mod_name)),
                        self.current_scope,
                    );
                }
            }
            "locals" => {
                // locals { name = value }
                self.visit_hcl_locals_body(node, file);
                return; // locals body handled separately
            }
            "terraform" | "provider" => {
                // terraform { ... } and provider "name" { ... } — skip
            }
            _ => {}
        }

        // Recurse into block body for nested blocks
        self.push_scope();
        for child in &children {
            if child.kind() == "block" || child.kind() == "body" {
                self.visit_hcl_node(child, file);
            }
        }
        self.pop_scope();
    }

    /// Extract local variable names from a `locals` block body
    fn visit_hcl_locals_body(&mut self, node: &tree_sitter::Node<'_>, file: &ParsedFile) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "body" {
                let mut body_cursor = child.walk();
                for attr in child.children(&mut body_cursor) {
                    if attr.kind() == "attribute" {
                        if let Some(name_node) = attr.child(0) {
                            let name = file.node_text(&name_node);
                            if !name.is_empty() {
                                self.table.add_symbol(
                                    format!("local.{}", name),
                                    SymbolKind::Variable,
                                    Range::from_node(&name_node),
                                    None,
                                    Some("local value".to_string()),
                                    self.current_scope,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Strip surrounding quotes from a string literal
fn strip_quotes(s: &str) -> String {
    s.trim_matches('"').trim_matches('\'').to_string()
}

#[cfg(test)]
mod tests {
    use super::super::SymbolTableBuilder;
    use crate::syntax::{Language, MultiParser};

    #[test]
    fn test_terraform_resource() {
        let source = r#"
resource "aws_instance" "web" {
  ami           = "ami-abc123"
  instance_type = "t2.micro"
}
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Hcl).unwrap();
        let table = SymbolTableBuilder::new().build_terraform(&parsed);
        let names: Vec<&str> = table
            .all_symbols()
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            names.contains(&"aws_instance.web"),
            "missing resource, got: {:?}",
            names
        );
    }

    #[test]
    fn test_terraform_variable_and_output() {
        let source = r#"
variable "instance_type" {
  default = "t2.micro"
}

output "public_ip" {
  value = aws_instance.web.public_ip
}
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Hcl).unwrap();
        let table = SymbolTableBuilder::new().build_terraform(&parsed);
        let names: Vec<&str> = table
            .all_symbols()
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            names.contains(&"var.instance_type"),
            "missing variable, got: {:?}",
            names
        );
        assert!(
            names.contains(&"output.public_ip"),
            "missing output, got: {:?}",
            names
        );
    }

    #[test]
    fn test_terraform_data_and_module() {
        let source = r#"
data "aws_ami" "ubuntu" {
  most_recent = true
}

module "vpc" {
  source = "./modules/vpc"
}
"#;
        let mut parser = MultiParser::new().unwrap();
        let parsed = parser.parse(source, Language::Hcl).unwrap();
        let table = SymbolTableBuilder::new().build_terraform(&parsed);
        let names: Vec<&str> = table
            .all_symbols()
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            names.contains(&"data.aws_ami.ubuntu"),
            "missing data source, got: {:?}",
            names
        );
        assert!(
            names.contains(&"module.vpc"),
            "missing module, got: {:?}",
            names
        );
    }
}
