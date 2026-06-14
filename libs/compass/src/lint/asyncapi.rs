//! AsyncAPI 2.x/3.x lint checker (source-line analysis on YAML)

use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use crate::syntax::{Language, ParsedFile};

/// AsyncAPI 2.x/3.x checker — line-based YAML analysis
pub struct AsyncApiChecker;

impl AsyncApiChecker {
    pub fn new() -> Self {
        Self
    }

    /// Detect whether the source is an AsyncAPI document.
    pub fn is_asyncapi(source: &str) -> bool {
        source
            .lines()
            .take(10)
            .any(|line| line.trim().starts_with("asyncapi:"))
    }

    /// AA001: Missing required fields — info, channels; info.title, info.version
    fn check_required_fields(lines: &[&str]) -> Vec<Diagnostic> {
        let mut has_info = false;
        let mut has_channels = false;
        let mut has_info_title = false;
        let mut has_info_version = false;
        let mut in_info = false;

        for line in lines.iter() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let indent = line.len() - line.trim_start().len();
            if indent == 0 {
                if in_info {
                    in_info = false;
                }
                if trimmed.starts_with("info:") {
                    has_info = true;
                    in_info = true;
                } else if trimmed.starts_with("channels:") {
                    has_channels = true;
                }
            } else if in_info {
                if trimmed.starts_with("title:") {
                    has_info_title = true;
                } else if trimmed.starts_with("version:") {
                    has_info_version = true;
                }
            }
        }

        let mut diags = Vec::new();
        let r = Range::new(Position::new(0, 0), Position::new(0, 1));
        if !has_info {
            diags.push(Diagnostic::error(
                r,
                "AA001",
                DiagnosticCategory::Logic,
                "Missing required field 'info' in AsyncAPI document",
            ));
        }
        if !has_channels {
            diags.push(Diagnostic::error(
                r,
                "AA001",
                DiagnosticCategory::Logic,
                "Missing required field 'channels' in AsyncAPI document",
            ));
        }
        if has_info && !has_info_title {
            diags.push(Diagnostic::error(
                r,
                "AA001",
                DiagnosticCategory::Logic,
                "Missing required field 'info.title' in AsyncAPI document",
            ));
        }
        if has_info && !has_info_version {
            diags.push(Diagnostic::error(
                r,
                "AA001",
                DiagnosticCategory::Logic,
                "Missing required field 'info.version' in AsyncAPI document",
            ));
        }
        diags
    }

    /// AA002: Channel name must start with `/`
    fn check_channel_names(lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut in_channels = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let indent = line.len() - line.trim_start().len();

            if indent == 0 {
                in_channels = trimmed.starts_with("channels:");
                continue;
            }

            if in_channels && indent == 2 && trimmed.ends_with(':') {
                let channel = trimmed.trim_end_matches(':');
                if !channel.is_empty() && !channel.starts_with('$') && !channel.starts_with('/') {
                    diagnostics.push(Diagnostic::warning(
                        Range::new(
                            Position::new(i as u32, 0),
                            Position::new(i as u32, line.len() as u32),
                        ),
                        "AA002",
                        DiagnosticCategory::Style,
                        format!(
                            "Channel '{}' does not start with '/' — use a path-style channel name",
                            channel
                        ),
                    ));
                }
            }
        }

        diagnostics
    }

    /// AA003: Message block without payload or schema
    fn check_message_without_schema(lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut msg_block: Option<(usize, usize)> = None; // (line, indent)
        let mut has_schema = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let indent = line.len() - line.trim_start().len();

            if trimmed.starts_with("message:") {
                if let Some((ml, _)) = msg_block.take() {
                    if !has_schema {
                        push_no_schema(&mut diagnostics, ml, lines);
                    }
                }
                msg_block = Some((i, indent));
                has_schema = false;
                continue;
            }

            if let Some((ml, mi)) = msg_block {
                if indent <= mi && !trimmed.is_empty() {
                    msg_block = None;
                    if !has_schema {
                        push_no_schema(&mut diagnostics, ml, lines);
                    }
                } else if trimmed.starts_with("payload:") || trimmed.starts_with("schema:") {
                    has_schema = true;
                }
            }
        }

        if let Some((ml, _)) = msg_block {
            if !has_schema {
                push_no_schema(&mut diagnostics, ml, lines);
            }
        }

        diagnostics
    }

    /// AA004: Server entry missing required `protocol` field
    fn check_server_protocol(lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut in_servers = false;
        let mut servers_indent: usize = 0;
        let mut current_server: Option<(usize, String, usize)> = None;
        let mut server_has_protocol = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let indent = line.len() - line.trim_start().len();

            if indent == 0 {
                if let Some((sl, sn, _)) = current_server.take() {
                    if !server_has_protocol {
                        push_no_protocol(&mut diagnostics, sl, sn, lines);
                    }
                }
                if trimmed.starts_with("servers:") {
                    in_servers = true;
                    servers_indent = indent;
                } else {
                    in_servers = false;
                }
                continue;
            }

            if !in_servers {
                continue;
            }

            if indent == servers_indent + 2 && trimmed.ends_with(':') {
                if let Some((sl, sn, _)) = current_server.take() {
                    if !server_has_protocol {
                        push_no_protocol(&mut diagnostics, sl, sn, lines);
                    }
                }
                let sname = trimmed.trim_end_matches(':').to_string();
                current_server = Some((i, sname, indent));
                server_has_protocol = false;
            } else if let Some((_, _, si)) = current_server {
                if indent > si && trimmed.starts_with("protocol:") {
                    server_has_protocol = true;
                }
            }
        }

        if let Some((sl, sn, _)) = current_server {
            if !server_has_protocol {
                push_no_protocol(&mut diagnostics, sl, sn, lines);
            }
        }

        diagnostics
    }

    /// AA005: Description heading level skip
    fn check_description_markdown(lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("description:") {
                let value = rest.trim().trim_matches('"').trim_matches('\'');
                if value.contains("# ") && value.contains("### ") && !value.contains("## ") {
                    diagnostics.push(Diagnostic::new(
                        Range::new(Position::new(i as u32, 0), Position::new(i as u32, line.len() as u32)),
                        DiagnosticSeverity::Hint,
                        "AA005",
                        DiagnosticCategory::Style,
                        "Description skips heading level (h1 to h3) — use sequential heading levels",
                    ));
                }
            }
        }
        diagnostics
    }
}

fn push_no_schema(diagnostics: &mut Vec<Diagnostic>, line: usize, lines: &[&str]) {
    let col = lines.get(line).map(|l| l.len()).unwrap_or(0);
    diagnostics.push(Diagnostic::warning(
        Range::new(
            Position::new(line as u32, 0),
            Position::new(line as u32, col as u32),
        ),
        "AA003",
        DiagnosticCategory::Logic,
        "Message block is missing 'payload' or 'schema' definition",
    ));
}

fn push_no_protocol(diagnostics: &mut Vec<Diagnostic>, line: usize, name: String, lines: &[&str]) {
    let col = lines.get(line).map(|l| l.len()).unwrap_or(0);
    diagnostics.push(Diagnostic::error(
        Range::new(
            Position::new(line as u32, 0),
            Position::new(line as u32, col as u32),
        ),
        "AA004",
        DiagnosticCategory::Logic,
        format!("Server '{}' is missing required 'protocol' field", name),
    ));
}

impl Default for AsyncApiChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for AsyncApiChecker {
    fn language(&self) -> Language {
        Language::Yaml
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        if !Self::is_asyncapi(&file.source) {
            return Vec::new();
        }

        let lines: Vec<&str> = file.source.lines().collect();
        let mut diagnostics = Vec::new();

        diagnostics.extend(Self::check_required_fields(&lines));
        diagnostics.extend(Self::check_channel_names(&lines));
        diagnostics.extend(Self::check_message_without_schema(&lines));
        diagnostics.extend(Self::check_server_protocol(&lines));
        diagnostics.extend(Self::check_description_markdown(&lines));

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec!["AA001", "AA002", "AA003", "AA004", "AA005"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint::Checker;

    fn check(source: &str) -> Vec<Diagnostic> {
        let file = ParsedFile::line_based(source.to_string(), Language::Yaml);
        AsyncApiChecker::new().check(&file, &LintConfig::default())
    }

    #[test]
    fn test_is_asyncapi_detection() {
        assert!(AsyncApiChecker::is_asyncapi("asyncapi: 2.6.0\ninfo:\n"));
        assert!(AsyncApiChecker::is_asyncapi("asyncapi: \"3.0.0\"\n"));
        assert!(!AsyncApiChecker::is_asyncapi("openapi: 3.0.0\n"));
        assert!(!AsyncApiChecker::is_asyncapi("apiVersion: v1\n"));
    }

    #[test]
    fn test_missing_required_fields() {
        let source = "asyncapi: 2.6.0\ninfo:\n  title: My API\n";
        let diags = check(source);
        let codes: Vec<&str> = diags.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&"AA001"), "expected AA001, got {:?}", codes);
    }

    #[test]
    fn test_valid_asyncapi_no_false_positives() {
        let source = "\
asyncapi: 2.6.0
info:
  title: My Event API
  version: 1.0.0
channels:
  /user/created:
    subscribe:
      message:
        payload:
          type: object
";
        let diags = check(source);
        let aa001: Vec<_> = diags.iter().filter(|d| d.code == "AA001").collect();
        let aa002: Vec<_> = diags.iter().filter(|d| d.code == "AA002").collect();
        let aa003: Vec<_> = diags.iter().filter(|d| d.code == "AA003").collect();
        assert!(aa001.is_empty(), "unexpected AA001: {:?}", aa001);
        assert!(aa002.is_empty(), "unexpected AA002: {:?}", aa002);
        assert!(aa003.is_empty(), "unexpected AA003: {:?}", aa003);
    }

    #[test]
    fn test_channel_name_missing_slash() {
        let source = "\
asyncapi: 2.6.0
info:
  title: T
  version: 1.0.0
channels:
  userCreated:
    subscribe:
      message:
        payload:
          type: object
";
        let diags = check(source);
        assert!(
            diags.iter().any(|d| d.code == "AA002"),
            "expected AA002, got {:?}",
            diags
        );
    }

    #[test]
    fn test_message_without_payload() {
        let source = "\
asyncapi: 2.6.0
info:
  title: T
  version: 1.0.0
channels:
  /events:
    subscribe:
      message:
        summary: An event
";
        let diags = check(source);
        assert!(
            diags.iter().any(|d| d.code == "AA003"),
            "expected AA003, got {:?}",
            diags
        );
    }

    #[test]
    fn test_server_missing_protocol() {
        let source = "\
asyncapi: 2.6.0
info:
  title: T
  version: 1.0.0
channels:
  /x:
    subscribe:
      message:
        payload:
          type: object
servers:
  production:
    url: mqtt://example.com
";
        let diags = check(source);
        assert!(
            diags.iter().any(|d| d.code == "AA004"),
            "expected AA004, got {:?}",
            diags
        );
    }

    #[test]
    fn test_not_asyncapi_returns_empty() {
        let source = "openapi: 3.0.0\ninfo:\n  title: T\n  version: 1.0.0\npaths: {}\n";
        let diags = check(source);
        assert!(diags.is_empty(), "expected no diags for non-asyncapi YAML");
    }
}
