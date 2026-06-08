//! Dockerfile lint checker (line-based, no tree-sitter)

use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use crate::syntax::{Language, ParsedFile};

/// Dockerfile checker — operates on source lines, not tree-sitter AST
pub struct DockerfileChecker;

impl DockerfileChecker {
    pub fn new() -> Self {
        Self
    }

    /// Parse a line into (instruction, rest) ignoring comments and blank lines.
    /// Returns None for comments, blank lines, and continuation lines.
    fn parse_instruction(line: &str) -> Option<(&str, &str)> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let instruction = parts.next()?;
        let rest = parts.next().unwrap_or("").trim();
        Some((instruction, rest))
    }

    /// DK001: Missing FROM instruction
    fn check_missing_from(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let has_from = lines.iter().any(|line| {
            Self::parse_instruction(line)
                .map(|(inst, _)| inst.eq_ignore_ascii_case("FROM"))
                .unwrap_or(false)
        });

        if !has_from {
            vec![Diagnostic::error(
                Range::new(Position::new(0, 0), Position::new(0, 1)),
                "DK001",
                DiagnosticCategory::Logic,
                "Dockerfile missing FROM instruction",
            )]
        } else {
            Vec::new()
        }
    }

    /// DK002: Using `latest` tag or no tag in FROM
    fn check_latest_tag(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            if let Some((inst, rest)) = Self::parse_instruction(line) {
                if !inst.eq_ignore_ascii_case("FROM") {
                    continue;
                }
                // Extract the image reference (first token of rest, ignore AS alias)
                let image = rest.split_whitespace().next().unwrap_or("");
                if image.is_empty() || image == "scratch" {
                    continue;
                }
                // Check for :latest or missing tag
                let uses_latest = image.ends_with(":latest");
                let has_tag = image.contains(':') || image.contains('@');
                if uses_latest || !has_tag {
                    let msg = if uses_latest {
                        "FROM uses ':latest' tag — pin to a specific version"
                    } else {
                        "FROM has no tag — defaults to ':latest', pin a version"
                    };
                    diagnostics.push(Diagnostic::warning(
                        Range::new(
                            Position::new(line_num as u32, 0),
                            Position::new(line_num as u32, line.len() as u32),
                        ),
                        "DK002",
                        DiagnosticCategory::Style,
                        msg,
                    ));
                }
            }
        }

        diagnostics
    }

    /// DK003: Multiple CMD instructions
    fn check_multiple_cmd(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut cmd_positions: Vec<usize> = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            if let Some((inst, _)) = Self::parse_instruction(line) {
                if inst.eq_ignore_ascii_case("CMD") {
                    cmd_positions.push(line_num);
                }
            }
        }

        if cmd_positions.len() > 1 {
            // Flag all but the last (only the last CMD takes effect)
            for &pos in &cmd_positions[..cmd_positions.len() - 1] {
                diagnostics.push(Diagnostic::warning(
                    Range::new(
                        Position::new(pos as u32, 0),
                        Position::new(pos as u32, lines[pos].len() as u32),
                    ),
                    "DK003",
                    DiagnosticCategory::Logic,
                    "Multiple CMD instructions — only the last one takes effect",
                ));
            }
        }

        diagnostics
    }

    /// DK005: RUN apt-get install without --no-install-recommends
    fn check_apt_get_recommends(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            if let Some((inst, rest)) = Self::parse_instruction(line) {
                if !inst.eq_ignore_ascii_case("RUN") {
                    continue;
                }
                if rest.contains("apt-get install") && !rest.contains("--no-install-recommends") {
                    diagnostics.push(Diagnostic::warning(
                        Range::new(
                            Position::new(line_num as u32, 0),
                            Position::new(line_num as u32, line.len() as u32),
                        ),
                        "DK005",
                        DiagnosticCategory::Style,
                        "RUN apt-get install without --no-install-recommends increases image size",
                    ));
                }
            }
        }

        diagnostics
    }

    /// DK007: Using ADD when COPY suffices
    fn check_add_vs_copy(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            if let Some((inst, rest)) = Self::parse_instruction(line) {
                if !inst.eq_ignore_ascii_case("ADD") {
                    continue;
                }
                // ADD is fine for URLs and tar extraction
                let is_url = rest.contains("http://") || rest.contains("https://");
                let is_tar = rest.contains(".tar")
                    || rest.contains(".gz")
                    || rest.contains(".bz2")
                    || rest.contains(".xz");
                if !is_url && !is_tar {
                    diagnostics.push(Diagnostic::warning(
                        Range::new(
                            Position::new(line_num as u32, 0),
                            Position::new(line_num as u32, line.len() as u32),
                        ),
                        "DK007",
                        DiagnosticCategory::Style,
                        "Use COPY instead of ADD for simple file copies",
                    ));
                }
            }
        }

        diagnostics
    }

    /// DK008: Consecutive RUN instructions (layer bloat)
    fn check_unchained_run(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut prev_was_run = false;
        let mut prev_run_line: usize = 0;

        for (line_num, line) in lines.iter().enumerate() {
            if let Some((inst, _)) = Self::parse_instruction(line) {
                let is_run = inst.eq_ignore_ascii_case("RUN");
                if is_run && prev_was_run {
                    diagnostics.push(Diagnostic::new(
                        Range::new(
                            Position::new(line_num as u32, 0),
                            Position::new(line_num as u32, line.len() as u32),
                        ),
                        DiagnosticSeverity::Hint,
                        "DK008",
                        DiagnosticCategory::Style,
                        format!(
                            "Consecutive RUN instructions (lines {} and {}) — chain with && to reduce layers",
                            prev_run_line + 1,
                            line_num + 1,
                        ),
                    ));
                }
                // Only track non-continuation instruction lines
                prev_was_run = is_run;
                if is_run {
                    prev_run_line = line_num;
                }
            }
            // Blank/comment lines don't break the consecutive-run chain
        }

        diagnostics
    }

    /// DK004: COPY/ADD without --chown
    fn check_copy_without_chown(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            if let Some((inst, rest)) = Self::parse_instruction(line) {
                let is_copy = inst.eq_ignore_ascii_case("COPY");
                let is_add = inst.eq_ignore_ascii_case("ADD");
                if (is_copy || is_add) && !rest.contains("--chown") {
                    diagnostics.push(Diagnostic::warning(
                        Range::new(
                            Position::new(line_num as u32, 0),
                            Position::new(line_num as u32, line.len() as u32),
                        ),
                        "DK004",
                        DiagnosticCategory::Security,
                        format!(
                            "{} without --chown — files will be owned by root",
                            inst.to_uppercase(),
                        ),
                    ));
                }
            }
        }

        diagnostics
    }

    /// DK006: Missing HEALTHCHECK instruction
    fn check_missing_healthcheck(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let has_healthcheck = lines.iter().any(|line| {
            Self::parse_instruction(line)
                .map(|(inst, _)| inst.eq_ignore_ascii_case("HEALTHCHECK"))
                .unwrap_or(false)
        });

        // Only flag if this looks like a service (has EXPOSE or CMD)
        let has_expose_or_cmd = lines.iter().any(|line| {
            Self::parse_instruction(line)
                .map(|(inst, _)| {
                    inst.eq_ignore_ascii_case("EXPOSE") || inst.eq_ignore_ascii_case("CMD")
                })
                .unwrap_or(false)
        });

        if has_expose_or_cmd && !has_healthcheck {
            vec![Diagnostic::warning(
                Range::new(Position::new(0, 0), Position::new(0, 1)),
                "DK006",
                DiagnosticCategory::Logic,
                "Missing HEALTHCHECK instruction — consider adding one for container orchestration",
            )]
        } else {
            Vec::new()
        }
    }

    /// DK010: Suggest .dockerignore
    fn check_dockerignore_hint(&self, lines: &[&str]) -> Vec<Diagnostic> {
        // Check if any COPY/ADD uses a broad source like "." or "./"
        let has_broad_copy = lines.iter().any(|line| {
            Self::parse_instruction(line)
                .map(|(inst, rest)| {
                    let is_copy_or_add =
                        inst.eq_ignore_ascii_case("COPY") || inst.eq_ignore_ascii_case("ADD");
                    if !is_copy_or_add {
                        return false;
                    }
                    // Strip flags (--chown, --from, etc.)
                    let parts: Vec<&str> = rest
                        .split_whitespace()
                        .filter(|p| !p.starts_with("--"))
                        .collect();
                    // First non-flag token is the source
                    parts
                        .first()
                        .map(|src| *src == "." || *src == "./")
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        });

        if has_broad_copy {
            vec![Diagnostic::warning(
                Range::new(Position::new(0, 0), Position::new(0, 1)),
                "DK010",
                DiagnosticCategory::Style,
                "COPY/ADD uses '.' as source — ensure a .dockerignore file exists to exclude unnecessary files",
            )]
        } else {
            Vec::new()
        }
    }

    /// DK009: Hardcoded secrets in ENV/ARG
    fn check_hardcoded_secrets(&self, lines: &[&str]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        const SECRET_KEYWORDS: &[&str] = &[
            "PASSWORD",
            "SECRET",
            "TOKEN",
            "API_KEY",
            "APIKEY",
            "PRIVATE_KEY",
            "ACCESS_KEY",
            "CREDENTIAL",
        ];

        for (line_num, line) in lines.iter().enumerate() {
            if let Some((inst, rest)) = Self::parse_instruction(line) {
                if !inst.eq_ignore_ascii_case("ENV") && !inst.eq_ignore_ascii_case("ARG") {
                    continue;
                }
                let upper_rest = rest.to_uppercase();
                for keyword in SECRET_KEYWORDS {
                    if upper_rest.contains(keyword) {
                        // Check if there's an actual value assigned (not just a declaration)
                        let has_value = rest.contains('=')
                            && rest
                                .split('=')
                                .nth(1)
                                .map(|v| !v.trim().is_empty())
                                .unwrap_or(false);
                        if has_value {
                            diagnostics.push(Diagnostic::new(
                                Range::new(
                                    Position::new(line_num as u32, 0),
                                    Position::new(line_num as u32, line.len() as u32),
                                ),
                                DiagnosticSeverity::Error,
                                "DK009",
                                DiagnosticCategory::Security,
                                format!(
                                    "Hardcoded secret in {} — '{}' should use build args or secrets mount",
                                    inst, keyword,
                                ),
                            ));
                        }
                        break;
                    }
                }
            }
        }

        diagnostics
    }
}

impl Default for DockerfileChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for DockerfileChecker {
    fn language(&self) -> Language {
        Language::Dockerfile
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let lines: Vec<&str> = file.source.lines().collect();
        let mut diagnostics = Vec::new();

        diagnostics.extend(self.check_missing_from(&lines));
        diagnostics.extend(self.check_latest_tag(&lines));
        diagnostics.extend(self.check_multiple_cmd(&lines));
        diagnostics.extend(self.check_apt_get_recommends(&lines));
        diagnostics.extend(self.check_add_vs_copy(&lines));
        diagnostics.extend(self.check_unchained_run(&lines));
        diagnostics.extend(self.check_hardcoded_secrets(&lines));
        diagnostics.extend(self.check_copy_without_chown(&lines));
        diagnostics.extend(self.check_missing_healthcheck(&lines));
        diagnostics.extend(self.check_dockerignore_hint(&lines));

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "DK001", // Missing FROM
            "DK002", // Using latest tag
            "DK003", // Multiple CMD
            "DK005", // apt-get without --no-install-recommends
            "DK007", // ADD when COPY suffices
            "DK008", // Unchained RUN (layer bloat)
            "DK009", // Hardcoded secrets in ENV/ARG
            "DK004", // COPY/ADD without --chown
            "DK006", // Missing HEALTHCHECK
            "DK010", // Suggest .dockerignore
        ]
    }
}
