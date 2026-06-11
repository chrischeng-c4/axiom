//! Dockerfile symbol extraction (line-based)
//!
//! Extracts symbols from Dockerfile source without tree-sitter:
//! - FROM stages: `FROM image AS name`
//! - ENV vars: `ENV KEY=value` or `ENV KEY value`
//! - ARG declarations: `ARG NAME=default`
//! - EXPOSE ports: `EXPOSE 8080`
//! - LABEL keys: `LABEL key=value`

use crate::diagnostic::{Position, Range};

use super::{SymbolKind, SymbolTableBuilder};

impl SymbolTableBuilder {
    /// Parse Dockerfile source line-by-line to extract symbols
    pub(crate) fn visit_dockerfile_lines(&mut self, source: &str) {
        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let line_num = line_idx as u32;

            // Find instruction (first word, case-insensitive)
            let (instruction, rest) = match trimmed.split_once(char::is_whitespace) {
                Some((inst, rest)) => (inst.to_uppercase(), rest.trim()),
                None => continue,
            };

            match instruction.as_str() {
                "FROM" => self.extract_dockerfile_from(rest, line_num),
                "ENV" => self.extract_dockerfile_env(rest, line_num),
                "ARG" => self.extract_dockerfile_arg(rest, line_num),
                "EXPOSE" => self.extract_dockerfile_expose(rest, line_num),
                "LABEL" => self.extract_dockerfile_label(rest, line_num),
                _ => {}
            }
        }
    }

    /// Extract FROM stage alias: `FROM image:tag AS name`
    fn extract_dockerfile_from(&mut self, rest: &str, line: u32) {
        // Look for "AS name" (case-insensitive)
        let parts: Vec<&str> = rest.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if part.eq_ignore_ascii_case("AS") {
                if let Some(&name) = parts.get(i + 1) {
                    let col = rest.find(name).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        name.to_string(),
                        SymbolKind::Stage,
                        make_range(line, col, name.len()),
                        None,
                        Some(format!("Build stage: {}", rest)),
                        self.current_scope,
                    );
                }
                return;
            }
        }
    }

    /// Extract ENV variables: `ENV KEY=value` or `ENV KEY value`
    fn extract_dockerfile_env(&mut self, rest: &str, line: u32) {
        // Handle `ENV KEY=value KEY2=value2`
        if rest.contains('=') {
            for pair in rest.split_whitespace() {
                if let Some((key, _)) = pair.split_once('=') {
                    if !key.is_empty() {
                        let col = rest.find(key).unwrap_or(0) as u32;
                        self.table.add_symbol(
                            key.to_string(),
                            SymbolKind::Variable,
                            make_range(line, col, key.len()),
                            None,
                            Some("ENV variable".to_string()),
                            self.current_scope,
                        );
                    }
                }
            }
        } else {
            // Handle `ENV KEY value` (legacy single-var form)
            if let Some((key, _)) = rest.split_once(char::is_whitespace) {
                if !key.is_empty() {
                    self.table.add_symbol(
                        key.to_string(),
                        SymbolKind::Variable,
                        make_range(line, 0, key.len()),
                        None,
                        Some("ENV variable".to_string()),
                        self.current_scope,
                    );
                }
            }
        }
    }

    /// Extract ARG declarations: `ARG NAME=default`
    fn extract_dockerfile_arg(&mut self, rest: &str, line: u32) {
        let name = rest.split('=').next().unwrap_or("").trim();
        if !name.is_empty() {
            self.table.add_symbol(
                name.to_string(),
                SymbolKind::Variable,
                make_range(line, 0, name.len()),
                None,
                Some("ARG declaration".to_string()),
                self.current_scope,
            );
        }
    }

    /// Extract EXPOSE ports: `EXPOSE 8080 443/tcp`
    fn extract_dockerfile_expose(&mut self, rest: &str, line: u32) {
        for port_spec in rest.split_whitespace() {
            let port = port_spec.split('/').next().unwrap_or(port_spec);
            if port.chars().all(|c| c.is_ascii_digit()) && !port.is_empty() {
                let col = rest.find(port_spec).unwrap_or(0) as u32;
                self.table.add_symbol(
                    port_spec.to_string(),
                    SymbolKind::Port,
                    make_range(line, col, port_spec.len()),
                    None,
                    Some(format!("Exposed port: {}", port_spec)),
                    self.current_scope,
                );
            }
        }
    }

    /// Extract LABEL keys: `LABEL key=value key2=value2`
    fn extract_dockerfile_label(&mut self, rest: &str, line: u32) {
        for pair in rest.split_whitespace() {
            if let Some((key, _)) = pair.split_once('=') {
                if !key.is_empty() {
                    let col = rest.find(pair).unwrap_or(0) as u32;
                    self.table.add_symbol(
                        key.to_string(),
                        SymbolKind::Label,
                        make_range(line, col, key.len()),
                        None,
                        Some("LABEL".to_string()),
                        self.current_scope,
                    );
                }
            }
        }
    }
}

/// Create a range for a token at a given line and column
fn make_range(line: u32, col: u32, len: usize) -> Range {
    Range::new(
        Position::new(line, col),
        Position::new(line, col + len as u32),
    )
}

#[cfg(test)]
mod tests {
    use super::super::SymbolTableBuilder;

    #[test]
    fn test_dockerfile_from_stage() {
        let source = "FROM python:3.12 AS builder\nFROM alpine:latest";
        let table = SymbolTableBuilder::new().build_dockerfile_from_source(source);
        let names: Vec<&str> = table
            .all_symbols()
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            names.contains(&"builder"),
            "missing stage 'builder', got: {:?}",
            names
        );
    }

    #[test]
    fn test_dockerfile_env_and_arg() {
        let source = "ARG VERSION=1.0\nENV APP_HOME=/app PORT=8080\nENV LEGACY_VAR value";
        let table = SymbolTableBuilder::new().build_dockerfile_from_source(source);
        let names: Vec<&str> = table
            .all_symbols()
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            names.contains(&"VERSION"),
            "missing 'VERSION', got: {:?}",
            names
        );
        assert!(
            names.contains(&"APP_HOME"),
            "missing 'APP_HOME', got: {:?}",
            names
        );
        assert!(names.contains(&"PORT"), "missing 'PORT', got: {:?}", names);
        assert!(
            names.contains(&"LEGACY_VAR"),
            "missing 'LEGACY_VAR', got: {:?}",
            names
        );
    }

    #[test]
    fn test_dockerfile_expose() {
        let source = "EXPOSE 8080 443/tcp";
        let table = SymbolTableBuilder::new().build_dockerfile_from_source(source);
        let names: Vec<&str> = table
            .all_symbols()
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            names.contains(&"8080"),
            "missing port '8080', got: {:?}",
            names
        );
        assert!(
            names.contains(&"443/tcp"),
            "missing port '443/tcp', got: {:?}",
            names
        );
    }

    #[test]
    fn test_dockerfile_label() {
        let source = r#"LABEL maintainer="user@example.com" version="1.0""#;
        let table = SymbolTableBuilder::new().build_dockerfile_from_source(source);
        let names: Vec<&str> = table
            .all_symbols()
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            names.contains(&"maintainer"),
            "missing 'maintainer', got: {:?}",
            names
        );
        assert!(
            names.contains(&"version"),
            "missing 'version', got: {:?}",
            names
        );
    }
}
