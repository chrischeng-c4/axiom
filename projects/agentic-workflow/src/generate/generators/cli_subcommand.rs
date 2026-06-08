//! CLI subcommand codegen primitive.
//!
//! Emits a clap-derive `Args` struct + a dispatch match arm pair for one
//! `score`-style CLI subcommand described by a [`CliCommand`] descriptor.
//! Replaces legacy manual CLI blocks in
//! `projects/agentic-workflow/src/cli/{sdd.rs,td.rs,td_migrate.rs}` once the apply-side
//! dispatch is wired up.
//!
//! @spec projects/agentic-workflow/tech-design/core/generate/generators/cli-subcommand.md

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/cli-subcommand.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Specification of a single CLI argument field. Each CliArg produces one struct field in the generated clap Args struct with appropriate attribute annotations derived from the metadata fields. Satisfies R1, R3.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/cli-subcommand.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliArg {
    /// Argument name in snake_case. Used as the Rust field name and, for option and flag kinds, as the long name in kebab-case.
    pub name: String,
    /// Argument classification: positional, flag, or option. Determines which attribute variant is emitted for this field.
    pub kind: CliArgKind,
    /// When true, the field type becomes Vec<String> and the argument accepts one or more values. Applies to option and positional kinds. Ignored for flag kind.
    #[serde(default)]
    pub multiple: Option<bool>,
    /// Literal default value string emitted in the default_value attribute. When absent and required is true, the argument is mandatory. When absent and required is false, the field type is Option<String>.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    /// Single-character short name (without the leading dash). When present, the short attribute is emitted alongside the long form. Applies to option and flag kinds only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short: Option<String>,
    /// Help text emitted as a doc comment on the struct field. When absent, no doc comment is emitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    /// When true and default_value is absent, the field type is String or Vec<String> (non-optional). When false and default_value is absent, the field type is Option<String>.
    #[serde(default)]
    pub required: Option<bool>,
}

/// Classification of a CLI argument that controls attribute emission. positional — emitted as a plain positional argument with no named prefix. flag — emitted as a boolean presence flag with no value operand. option — emitted as a named value option accepting one or more values.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/cli-subcommand.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CliArgKind {
    Positional,
    Flag,
    Option,
}

/// Top-level CLI subcommand specification consumed by the cli_subcommand generator. The generator walks args to emit the clap Args struct and uses is_async and dispatch_fn to emit the match arm. Satisfies R1, R2, R3.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/cli-subcommand.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommand {
    /// Subcommand name in kebab-case (e.g. migrate-mermaid). Used as the subcommand string in the parent enum variant.
    pub name: String,
    /// Parent enum variant name in PascalCase (e.g. MigrateMermaid). When absent, derived from name by converting kebab-case to PascalCase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    /// Ordered list of argument specifications. An empty list produces a zero-field Args struct — valid for subcommands with no arguments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<CliArg>,
    /// When true, the generated dispatch arm appends .await to the handler call expression. When false, the call is synchronous. Satisfies R2, R3.
    #[serde(default)]
    pub is_async: Option<bool>,
    /// Qualified Rust path of the handler function used in the dispatch arm. When absent, derived as crate::<snake_name>::run from the subcommand name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dispatch_fn: Option<String>,
    /// Name of the generated clap Args struct in PascalCase. When absent, derived as <Variant>Args from the variant name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub struct_name: Option<String>,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/cli_subcommand_helpers.md#source
// CODEGEN-BEGIN

/// Result of running the CLI subcommand generator. Carries the rendered
/// clap-derive Args struct lines and the dispatch match-arm separately so
/// the apply-side dispatcher can splice each into the appropriate file.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/cli-subcommand.md#logic
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CliEmitted {
    /// Source lines for the clap `Args` struct (R1).
    pub args_struct_lines: Vec<String>,
    /// One-line dispatch match arm (R2).
    pub dispatch_arm: String,
    /// Resolved clap struct name (e.g. `CoverageArgs`).
    pub struct_name: String,
    /// Resolved parent enum variant name (e.g. `Coverage`).
    pub variant_name: String,
    /// Resolved dispatch handler function path (e.g. `run_coverage`).
    pub dispatch_fn: String,
}

/// Convert a kebab-case (or already-snake) name into PascalCase.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/cli-subcommand.md#logic
fn kebab_to_pascal(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut next_upper = true;
    for ch in name.chars() {
        if ch == '-' || ch == '_' {
            next_upper = true;
            continue;
        }
        if next_upper {
            for u in ch.to_uppercase() {
                out.push(u);
            }
            next_upper = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// Convert a kebab-case name into snake_case.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/cli-subcommand.md#logic
fn kebab_to_snake(name: &str) -> String {
    name.replace('-', "_")
}

/// Resolve the field's Rust type given kind / multiple / required / default_value.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/cli-subcommand.md#logic
fn resolve_field_type(arg: &CliArg) -> &'static str {
    let multiple = arg.multiple.unwrap_or(false);
    let required = arg.required.unwrap_or(true);
    let has_default = arg.default_value.is_some();
    match arg.kind {
        CliArgKind::Flag => "bool",
        CliArgKind::Positional | CliArgKind::Option => {
            if multiple {
                "Vec<String>"
            } else if required || has_default {
                "String"
            } else {
                "Option<String>"
            }
        }
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/cli-subcommand.md#logic
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/cli-subcommand.md#logic
pub fn emit_cli_subcommand(cmd: &CliCommand) -> CliEmitted {
    // 1. Resolve variant / struct / dispatch names.
    let variant = cmd
        .variant
        .clone()
        .unwrap_or_else(|| kebab_to_pascal(&cmd.name));
    let struct_name = cmd
        .struct_name
        .clone()
        .unwrap_or_else(|| format!("{}Args", variant));
    let snake = kebab_to_snake(&cmd.name);
    let dispatch_fn = cmd
        .dispatch_fn
        .clone()
        .unwrap_or_else(|| format!("crate::{}::run", snake));

    // 2. Emit the Args struct.
    let mut lines: Vec<String> = Vec::new();
    lines.push("#[derive(Debug, Args)]".to_string());
    lines.push(format!("pub struct {} {{", struct_name));

    for arg in &cmd.args {
        // Optional help → doc comment.
        if let Some(help) = &arg.help {
            lines.push(format!("    /// {}", help));
        }

        let multiple = arg.multiple.unwrap_or(false);
        let ty = resolve_field_type(arg);

        match arg.kind {
            CliArgKind::Positional => {
                // Build #[arg(...)] attribute, only emit if it has anything.
                let mut parts: Vec<String> = Vec::new();
                if let Some(dv) = &arg.default_value {
                    parts.push(format!("default_value = {:?}", dv));
                }
                if multiple {
                    parts.push("value_delimiter = ','".to_string());
                }
                if !parts.is_empty() {
                    lines.push(format!("    #[arg({})]", parts.join(", ")));
                }
                lines.push(format!("    pub {}: {},", arg.name, ty));
            }
            CliArgKind::Flag => {
                let mut parts: Vec<String> = vec!["long".to_string()];
                if let Some(s) = &arg.short {
                    parts.push(format!("short = '{}'", s));
                }
                lines.push(format!("    #[arg({})]", parts.join(", ")));
                lines.push(format!("    pub {}: bool,", arg.name));
            }
            CliArgKind::Option => {
                let mut parts: Vec<String> = vec!["long".to_string()];
                if let Some(s) = &arg.short {
                    parts.push(format!("short = '{}'", s));
                }
                if let Some(dv) = &arg.default_value {
                    parts.push(format!("default_value = {:?}", dv));
                }
                if multiple {
                    parts.push("value_delimiter = ','".to_string());
                }
                lines.push(format!("    #[arg({})]", parts.join(", ")));
                lines.push(format!("    pub {}: {},", arg.name, ty));
            }
        }
    }

    lines.push("}".to_string());

    // 3. Emit dispatch arm.
    let await_suffix = if cmd.is_async.unwrap_or(false) {
        ".await"
    } else {
        ""
    };
    let dispatch_arm = format!("{}(a) => {}(a){},", variant, dispatch_fn, await_suffix);

    CliEmitted {
        args_struct_lines: lines,
        dispatch_arm,
        struct_name,
        variant_name: variant,
        dispatch_fn,
    }
}
// CODEGEN-END

#[cfg(test)]
mod tests {
    use super::*;

    fn arg(name: &str, kind: CliArgKind) -> CliArg {
        CliArg {
            name: name.into(),
            kind,
            multiple: None,
            default_value: None,
            short: None,
            help: None,
            required: None,
        }
    }

    /// R7 — empty args list emits a zero-field Args struct.
    #[test]
    fn empty_args() {
        let cmd = CliCommand {
            name: "audit".into(),
            variant: None,
            args: vec![],
            is_async: None,
            dispatch_fn: None,
            struct_name: None,
        };
        let out = emit_cli_subcommand(&cmd);
        assert_eq!(
            out.args_struct_lines,
            vec![
                "#[derive(Debug, Args)]".to_string(),
                "pub struct AuditArgs {".to_string(),
                "}".to_string(),
            ]
        );
    }

    /// R7 — positional-only with a required string field.
    #[test]
    fn positional_only() {
        let cmd = CliCommand {
            name: "ast".into(),
            variant: None,
            args: vec![arg("path", CliArgKind::Positional)],
            is_async: None,
            dispatch_fn: None,
            struct_name: None,
        };
        let out = emit_cli_subcommand(&cmd);
        // Required positional with no default has no #[arg(...)] line.
        assert_eq!(
            out.args_struct_lines,
            vec![
                "#[derive(Debug, Args)]".to_string(),
                "pub struct AstArgs {".to_string(),
                "    pub path: String,".to_string(),
                "}".to_string(),
            ]
        );
    }

    /// R7 — flag-only with short.
    #[test]
    fn flag_only() {
        let cmd = CliCommand {
            name: "scan".into(),
            variant: None,
            args: vec![CliArg {
                name: "all".into(),
                kind: CliArgKind::Flag,
                multiple: None,
                default_value: None,
                short: Some("a".into()),
                help: None,
                required: None,
            }],
            is_async: None,
            dispatch_fn: None,
            struct_name: None,
        };
        let out = emit_cli_subcommand(&cmd);
        assert_eq!(
            out.args_struct_lines,
            vec![
                "#[derive(Debug, Args)]".to_string(),
                "pub struct ScanArgs {".to_string(),
                "    #[arg(long, short = 'a')]".to_string(),
                "    pub all: bool,".to_string(),
                "}".to_string(),
            ]
        );
    }

    /// R7 — option-only with default.
    #[test]
    fn option_only() {
        let cmd = CliCommand {
            name: "build".into(),
            variant: None,
            args: vec![CliArg {
                name: "format".into(),
                kind: CliArgKind::Option,
                multiple: None,
                default_value: Some("text".into()),
                short: None,
                help: None,
                required: None,
            }],
            is_async: None,
            dispatch_fn: None,
            struct_name: None,
        };
        let out = emit_cli_subcommand(&cmd);
        assert_eq!(
            out.args_struct_lines,
            vec![
                "#[derive(Debug, Args)]".to_string(),
                "pub struct BuildArgs {".to_string(),
                "    #[arg(long, default_value = \"text\")]".to_string(),
                "    pub format: String,".to_string(),
                "}".to_string(),
            ]
        );
    }

    /// R7 — mixed args: positional + flag + option with help.
    #[test]
    fn mixed_args() {
        let cmd = CliCommand {
            name: "report".into(),
            variant: None,
            args: vec![
                CliArg {
                    name: "path".into(),
                    kind: CliArgKind::Positional,
                    multiple: None,
                    default_value: None,
                    short: None,
                    help: Some("Target path.".into()),
                    required: None,
                },
                CliArg {
                    name: "verbose".into(),
                    kind: CliArgKind::Flag,
                    multiple: None,
                    default_value: None,
                    short: Some("v".into()),
                    help: None,
                    required: None,
                },
                CliArg {
                    name: "format".into(),
                    kind: CliArgKind::Option,
                    multiple: None,
                    default_value: Some("json".into()),
                    short: None,
                    help: None,
                    required: None,
                },
            ],
            is_async: None,
            dispatch_fn: None,
            struct_name: None,
        };
        let out = emit_cli_subcommand(&cmd);
        assert_eq!(
            out.args_struct_lines,
            vec![
                "#[derive(Debug, Args)]".to_string(),
                "pub struct ReportArgs {".to_string(),
                "    /// Target path.".to_string(),
                "    pub path: String,".to_string(),
                "    #[arg(long, short = 'v')]".to_string(),
                "    pub verbose: bool,".to_string(),
                "    #[arg(long, default_value = \"json\")]".to_string(),
                "    pub format: String,".to_string(),
                "}".to_string(),
            ]
        );
    }

    /// R2 — async command emits dispatch arm with .await.
    #[test]
    fn async_command_dispatch_arm() {
        let cmd = CliCommand {
            name: "merge".into(),
            variant: None,
            args: vec![],
            is_async: Some(true),
            dispatch_fn: Some("run_merge".into()),
            struct_name: None,
        };
        let out = emit_cli_subcommand(&cmd);
        assert_eq!(out.dispatch_arm, "Merge(a) => run_merge(a).await,");
    }

    /// R2 — sync command emits dispatch arm without .await.
    #[test]
    fn sync_command_dispatch_arm() {
        let cmd = CliCommand {
            name: "audit".into(),
            variant: None,
            args: vec![],
            is_async: Some(false),
            dispatch_fn: Some("run_audit".into()),
            struct_name: None,
        };
        let out = emit_cli_subcommand(&cmd);
        assert_eq!(out.dispatch_arm, "Audit(a) => run_audit(a),");
    }

    /// R2 — defaulted dispatch_fn falls back to `crate::<snake>::run`.
    #[test]
    fn default_dispatch_fn() {
        let cmd = CliCommand {
            name: "migrate-mermaid".into(),
            variant: None,
            args: vec![],
            is_async: Some(true),
            dispatch_fn: None,
            struct_name: None,
        };
        let out = emit_cli_subcommand(&cmd);
        assert_eq!(
            out.dispatch_arm,
            "MigrateMermaid(a) => crate::migrate_mermaid::run(a).await,"
        );
        assert_eq!(out.variant_name, "MigrateMermaid");
        assert_eq!(out.struct_name, "MigrateMermaidArgs");
    }

    /// R7 — multiple option emits Vec<String> + value_delimiter.
    #[test]
    fn multiple_option() {
        let cmd = CliCommand {
            name: "tag".into(),
            variant: None,
            args: vec![CliArg {
                name: "labels".into(),
                kind: CliArgKind::Option,
                multiple: Some(true),
                default_value: None,
                short: Some("l".into()),
                help: None,
                required: None,
            }],
            is_async: None,
            dispatch_fn: None,
            struct_name: None,
        };
        let out = emit_cli_subcommand(&cmd);
        assert_eq!(
            out.args_struct_lines,
            vec![
                "#[derive(Debug, Args)]".to_string(),
                "pub struct TagArgs {".to_string(),
                "    #[arg(long, short = 'l', value_delimiter = ',')]".to_string(),
                "    pub labels: Vec<String>,".to_string(),
                "}".to_string(),
            ]
        );
    }

    /// R1 + R3 — snapshot of a representative fixture covering positional,
    /// flag, option, help, default, short, and async.
    #[test]
    fn snapshot_fixture() {
        let cmd = CliCommand {
            name: "coverage".into(),
            variant: None,
            args: vec![
                CliArg {
                    name: "workspace_root".into(),
                    kind: CliArgKind::Option,
                    multiple: None,
                    default_value: Some(".".into()),
                    short: Some("w".into()),
                    help: Some("Root directory to scan recursively.".into()),
                    required: None,
                },
                CliArg {
                    name: "json".into(),
                    kind: CliArgKind::Flag,
                    multiple: None,
                    default_value: None,
                    short: None,
                    help: Some("Emit JSON.".into()),
                    required: None,
                },
            ],
            is_async: Some(false),
            dispatch_fn: Some("run_coverage".into()),
            struct_name: None,
        };
        let out = emit_cli_subcommand(&cmd);
        let rendered = out.args_struct_lines.join("\n");
        assert_eq!(
            rendered,
            "#[derive(Debug, Args)]\n\
             pub struct CoverageArgs {\n    \
             /// Root directory to scan recursively.\n    \
             #[arg(long, short = 'w', default_value = \".\")]\n    \
             pub workspace_root: String,\n    \
             /// Emit JSON.\n    \
             #[arg(long)]\n    \
             pub json: bool,\n\
             }"
        );
        assert_eq!(out.dispatch_arm, "Coverage(a) => run_coverage(a),");
    }
}
