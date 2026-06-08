//! Code Analysis Strategy
//!
//! Analyzes source code files using AST parsing (tree-sitter) and generates
//! high-level technical specifications. Includes interactive clarification
//! and incremental update support.

use crate::fillback::ast::{
    AnalysisContext, AstAnalyzer, ModuleInfo, ParseError, SupportedLanguage, Symbol, SymbolKind,
};
use crate::fillback::graph::{DependencyGraph, GraphStats};
use crate::fillback::strategy::ImportStrategy;
use crate::models::validation::{DocumentType, ValidationRules};
use crate::validator::{SemanticValidator, SpecFormatValidator};
use crate::Result;
use async_trait::async_trait;
use colored::Colorize;
use dialoguer::{Confirm, Input, MultiSelect};
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/fillback/code.md#schema
// CODEGEN-BEGIN
/// Code import strategy with AST-based analysis.
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/code.md#schema
pub struct CodeStrategy {
    /// Strategy configuration.
    config: CodeStrategyConfig,
}

/// Configuration for the code analysis strategy.
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/code.md#schema
#[derive(Debug, Clone)]
pub struct CodeStrategyConfig {
    /// Path to analyze (defaults to current directory).
    pub path: Option<String>,
    /// Specific module to analyze (optional filter).
    pub module: Option<String>,
    /// Force overwrite without confirmation.
    pub force: bool,
    /// Output directory for specs.
    pub output_dir: Option<String>,
    /// Quick mode: skip LLM enrichment and use AST-only analysis.
    pub quick: bool,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/fillback/code.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/code.md#source
impl Default for CodeStrategyConfig {
    fn default() -> Self {
        Self {
            path: None,
            module: None,
            force: false,
            output_dir: None,
            quick: false,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/code.md#source
impl CodeStrategy {
    pub fn new() -> Self {
        Self {
            config: CodeStrategyConfig::default(),
        }
    }

    pub fn with_config(config: CodeStrategyConfig) -> Self {
        Self { config }
    }

    /// Scan source directory and collect files for analysis
    fn scan_files(&self, source: &Path) -> Result<Vec<(String, String)>> {
        let mut files = Vec::new();
        let max_files = 500; // Higher limit since we're using AST
        let max_file_size = 100_000; // 100KB limit per file

        let walker = WalkBuilder::new(source).standard_filters(true).build();

        let mut skipped_count = 0;

        for entry in walker {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            // Check file size
            if let Ok(metadata) = fs::metadata(path) {
                if metadata.len() > max_file_size as u64 {
                    skipped_count += 1;
                    continue;
                }
            }

            // Check if we support this language
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if SupportedLanguage::from_extension(ext).is_some() {
                    if let Ok(content) = fs::read_to_string(path) {
                        let relative_path = path
                            .strip_prefix(source)
                            .unwrap_or(path)
                            .to_string_lossy()
                            .to_string();
                        files.push((relative_path, content));

                        if files.len() >= max_files {
                            println!(
                                "{}",
                                format!(
                                    "  Reached file limit ({}). Some files were skipped.",
                                    max_files
                                )
                                .yellow()
                            );
                            break;
                        }
                    }
                }
            }
        }

        if skipped_count > 0 {
            println!(
                "{}",
                format!("  Skipped {} files (too large)", skipped_count).bright_black()
            );
        }

        Ok(files)
    }

    /// Analyze codebase using AST parser
    pub fn analyze_codebase(&self, source: &Path) -> Result<(AnalysisContext, Vec<ParseError>)> {
        let mut analyzer = AstAnalyzer::new()?;
        let mut context = AnalysisContext::new();
        let mut parse_errors = Vec::new();

        let files = self.scan_files(source)?;

        if files.is_empty() {
            anyhow::bail!("No supported source files found in: {}", source.display());
        }

        println!(
            "{}",
            format!("  Analyzing {} files with tree-sitter...", files.len()).bright_black()
        );

        for (rel_path, content) in files {
            let full_path = source.join(&rel_path);

            match analyzer.parse_file(&full_path, &content) {
                Ok(module) => {
                    // Update language counts
                    let lang_name = module.language.display_name().to_string();
                    *context.language_counts.entry(lang_name).or_insert(0) += 1;

                    // Filter by module name if specified
                    if let Some(ref filter) = self.config.module {
                        if !module.name.contains(filter) {
                            continue;
                        }
                    }

                    context.modules.push(module);
                }
                Err(err) => {
                    context.skipped_files.push(rel_path.clone());
                    parse_errors.push(err);
                }
            }
        }

        if context.modules.is_empty() {
            if let Some(ref filter) = self.config.module {
                anyhow::bail!("No modules matching '{}' found", filter);
            } else {
                anyhow::bail!("Failed to parse any source files");
            }
        }

        Ok((context, parse_errors))
    }

    /// Display analysis summary
    pub fn display_summary(&self, context: &AnalysisContext, graph: &DependencyGraph) {
        let stats = GraphStats::from_graph(graph);

        println!();
        println!("{}", "Analysis Summary".cyan().bold());
        println!("{}", "----------------".bright_black());
        println!(
            "  Modules analyzed: {}",
            context.modules.len().to_string().green()
        );
        println!(
            "  Total symbols:    {}",
            context.total_symbols().to_string().green()
        );
        println!(
            "  External deps:    {}",
            stats.external_dependencies.to_string().yellow()
        );

        // Language breakdown
        if !context.language_counts.is_empty() {
            println!();
            println!("  {}", "Languages:".bright_black());
            for (lang, count) in &context.language_counts {
                println!("    {}: {} files", lang, count);
            }
        }

        // Most connected modules
        if !stats.most_connected_modules.is_empty() {
            println!();
            println!("  {}", "Most connected modules:".bright_black());
            for (name, count) in stats.most_connected_modules.iter().take(3) {
                println!("    {}: {} dependencies", name, count);
            }
        }

        // Skipped files
        if !context.skipped_files.is_empty() {
            println!();
            println!(
                "{}",
                format!(
                    "  Skipped {} files (parse errors)",
                    context.skipped_files.len()
                )
                .yellow()
            );
        }

        println!();
    }

    /// Display the dependency graph in compact form
    pub fn display_dependency_graph(&self, graph: &DependencyGraph) {
        println!("{}", "Dependency Graph (Mermaid)".cyan().bold());
        println!("{}", "-------------------------".bright_black());
        println!("{}", graph.to_mermaid_compact());
        println!();
    }

    /// Interactive clarification phase - asks questions to refine understanding
    ///
    /// In non-interactive contexts (no TTY on stdin, or `SCORE_NON_INTERACTIVE=1`
    /// in the environment) the function returns immediately with an empty
    /// answer set; safe defaults are applied downstream. This is what makes
    /// `aw cb claim --non-interactive` and agent-dispatch contexts viable.
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-recovery-verbs-non-interactive.md#logic
    pub fn run_clarification(&self, context: &AnalysisContext) -> Result<HashMap<String, String>> {
        use std::io::IsTerminal;
        if !std::io::stdin().is_terminal() || std::env::var("SCORE_NON_INTERACTIVE").is_ok() {
            return Ok(HashMap::new());
        }

        let mut answers = HashMap::new();

        println!("{}", "Clarification Questions".cyan().bold());
        println!("{}", "-----------------------".bright_black());
        println!(
            "{}",
            "Please answer a few questions to improve specification quality:".bright_black()
        );
        println!();

        // Question 1: Main entry point
        let modules: Vec<&str> = context.modules.iter().map(|m| m.name.as_str()).collect();
        if !modules.is_empty() {
            let main_candidates: Vec<&str> = modules
                .iter()
                .filter(|m| {
                    m.contains("main")
                        || m.contains("lib")
                        || m.contains("app")
                        || m.contains("index")
                })
                .copied()
                .collect();

            if !main_candidates.is_empty() {
                println!("Which module is the main entry point?");
                let selection = MultiSelect::new().items(&main_candidates).interact_opt()?;

                if let Some(indices) = selection {
                    let selected: Vec<String> = indices
                        .iter()
                        .map(|&i| main_candidates[i].to_string())
                        .collect();
                    answers.insert("entry_points".to_string(), selected.join(", "));
                }
            }
        }

        // Question 2: Public API modules
        let public_modules: Vec<&ModuleInfo> = context
            .modules
            .iter()
            .filter(|m| m.symbols.iter().any(|s| s.is_public))
            .collect();

        if !public_modules.is_empty() {
            println!();
            println!(
                "Found {} modules with public symbols. Which are part of the public API?",
                public_modules.len()
            );

            let module_names: Vec<&str> = public_modules.iter().map(|m| m.name.as_str()).collect();
            let selection = MultiSelect::new().items(&module_names).interact_opt()?;

            if let Some(indices) = selection {
                let selected: Vec<String> = indices
                    .iter()
                    .map(|&i| module_names[i].to_string())
                    .collect();
                answers.insert("public_api_modules".to_string(), selected.join(", "));
            }
        }

        // Question 3: Project description
        println!();
        let description: String = Input::new()
            .with_prompt("Brief project description (optional)")
            .allow_empty(true)
            .interact_text()?;

        if !description.is_empty() {
            answers.insert("project_description".to_string(), description);
        }

        // Question 4: Architecture style
        println!();
        let arch_styles = vec![
            "Monolithic",
            "Microservices",
            "Layered/Clean Architecture",
            "Event-Driven",
            "CLI Tool",
            "Library/SDK",
            "Other",
        ];

        println!("What architecture style best describes this project?");
        let selection = dialoguer::Select::new()
            .items(&arch_styles)
            .default(0)
            .interact_opt()?;

        if let Some(idx) = selection {
            answers.insert(
                "architecture_style".to_string(),
                arch_styles[idx].to_string(),
            );
        }

        Ok(answers)
    }

    /// Check for existing specs and handle incremental updates
    pub fn check_existing_specs(&self, output_dir: &Path) -> Result<Vec<String>> {
        let mut existing_files = Vec::new();

        if output_dir.exists() {
            for entry in fs::read_dir(output_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        existing_files.push(name.to_string());
                    }
                }
            }
        }

        Ok(existing_files)
    }

    /// Prompt for confirmation before overwriting existing specs.
    ///
    /// In non-interactive contexts (no TTY or `SCORE_NON_INTERACTIVE=1`)
    /// returns `Ok(true)` so the pipeline runs to completion.
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-recovery-verbs-non-interactive.md#logic
    pub fn confirm_overwrite(&self, existing_files: &[String]) -> Result<bool> {
        use std::io::IsTerminal;
        if self.config.force {
            return Ok(true);
        }

        if existing_files.is_empty() {
            return Ok(true);
        }

        if !std::io::stdin().is_terminal() || std::env::var("SCORE_NON_INTERACTIVE").is_ok() {
            return Ok(true);
        }

        println!();
        println!("{}", "Existing Specifications Found".yellow().bold());
        println!("{}", "-----------------------------".bright_black());
        for file in existing_files {
            println!("  - {}", file);
        }
        println!();

        let confirm = Confirm::new()
            .with_prompt("Overwrite existing specifications?")
            .default(false)
            .interact()?;

        Ok(confirm)
    }

    /// Generate specification files based on analysis
    pub fn generate_specs(
        &self,
        context: &AnalysisContext,
        graph: &DependencyGraph,
        output_dir: &Path,
        clarifications: &HashMap<String, String>,
    ) -> Result<Vec<String>> {
        fs::create_dir_all(output_dir)?;

        let mut created_files = Vec::new();

        // Generate dependency graph file
        let graph_content = graph.to_markdown("Analyzed Project");
        let graph_path = output_dir.join("_dependency-graph.md");
        fs::write(&graph_path, graph_content)?;
        created_files.push("_dependency-graph.md".to_string());

        // Generate overview spec
        let overview_content = self.generate_overview_spec(context, graph, clarifications);
        let overview_path = output_dir.join("_overview.md");
        fs::write(&overview_path, overview_content)?;
        created_files.push("_overview.md".to_string());

        // Bug fix (cb_claim_path_inference): mirror the source-tree
        // directory structure under output_dir so two files with the
        // same basename (e.g. `mod.rs` in different subdirs) produce
        // distinct spec files. Previously every module landed at
        // `output_dir/<basename>.md`, which silently overwrote
        // collisions and polluted the tech_design root with sibling
        // files. Project root is derived from output_dir as
        // `<root>/.aw/tech-design`, so .parent().parent() recovers
        // it. When the module path doesn't sit under project root
        // (e.g. an absolute path passed via --path that points
        // outside), fall back to the legacy flat name.
        let project_root_for_mirror: Option<PathBuf> = output_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf());
        for module in &context.modules {
            if module.symbols.is_empty() {
                continue;
            }

            let spec_content = self.generate_module_spec(module);
            let spec_rel: PathBuf = (|| -> Option<PathBuf> {
                let root = project_root_for_mirror.as_ref()?;
                let module_path = PathBuf::from(&module.path);
                let canon_module = module_path.canonicalize().unwrap_or(module_path);
                let canon_root = root.canonicalize().unwrap_or_else(|_| root.clone());
                let rel = canon_module.strip_prefix(&canon_root).ok()?;
                let mut buf = rel.to_path_buf();
                buf.set_extension("md");
                Some(buf)
            })()
            .unwrap_or_else(|| PathBuf::from(format!("{}.md", module.name)));

            let spec_path = output_dir.join(&spec_rel);
            if let Some(parent) = spec_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&spec_path, spec_content)?;
            created_files.push(spec_rel.to_string_lossy().to_string());
        }

        Ok(created_files)
    }

    /// Generate overview specification in TD format.
    ///
    /// Emits YAML frontmatter + `## Overview` section with
    /// `<!-- type: overview lang: markdown -->` annotation so the file is a
    /// valid TD spec that `aw td validate` accepts. Architecture /
    /// dependency prose lives inside Overview as plain Markdown.
    fn generate_overview_spec(
        &self,
        context: &AnalysisContext,
        graph: &DependencyGraph,
        clarifications: &HashMap<String, String>,
    ) -> String {
        let stats = GraphStats::from_graph(graph);
        let mut c = String::new();

        // Frontmatter — the literal `---` open/close is what split_frontmatter
        // keys off; `id` is required by validators downstream.
        c.push_str("---\n");
        c.push_str("id: overview\n");
        c.push_str("fill_sections: [overview]\n");
        c.push_str("---\n\n");

        c.push_str("## Overview\n");
        c.push_str("<!-- type: overview lang: markdown -->\n\n");

        if let Some(desc) = clarifications.get("project_description") {
            c.push_str(&format!("{}\n\n", desc));
        } else {
            c.push_str(
                "_Auto-generated from codebase analysis by `score fillback`. \
                Hand-written symbols only — `## Changes` lives on a per-module \
                spec; see sibling files._\n\n",
            );
        }

        if let Some(style) = clarifications.get("architecture_style") {
            c.push_str(&format!("**Architecture:** {}\n\n", style));
        }

        c.push_str("### Module Structure\n\n");
        c.push_str("| Module | Symbols | Public | Language |\n");
        c.push_str("|--------|---------|--------|----------|\n");
        for module in &context.modules {
            let pubn = module.symbols.iter().filter(|s| s.is_public).count();
            c.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                module.name,
                module.symbols.len(),
                pubn,
                module.language.display_name()
            ));
        }

        if let Some(entry_points) = clarifications.get("entry_points") {
            c.push_str("\n### Entry Points\n\n");
            for entry in entry_points.split(", ") {
                c.push_str(&format!("- `{}`\n", entry));
            }
        }

        if let Some(public_api) = clarifications.get("public_api_modules") {
            c.push_str("\n### Public API Modules\n\n");
            for module in public_api.split(", ") {
                c.push_str(&format!("- `{}`\n", module));
            }
        }

        c.push_str("\n### Dependencies\n\n");
        c.push_str(&format!(
            "- internal modules: {}\n- external deps: {}\n- avg deps/module: {:.1}\n\n",
            stats.internal_modules, stats.external_dependencies, stats.avg_dependencies_per_module
        ));

        let external_deps = graph.external_dependencies();
        if !external_deps.is_empty() {
            c.push_str("**External:**\n\n");
            for dep in external_deps {
                c.push_str(&format!("- `{}`\n", dep.name));
            }
            c.push('\n');
        }

        if !context.language_counts.is_empty() {
            c.push_str("### Language Breakdown\n\n");
            for (lang, count) in &context.language_counts {
                c.push_str(&format!("- {}: {} file(s)\n", lang, count));
            }
        }

        c
    }

    /// Run validators on generated specs
    fn validate_specs(&self, output_dir: &Path, created_files: &[String]) -> Result<Vec<String>> {
        let format_rules = ValidationRules::for_document_type(DocumentType::Spec);
        let format_validator = SpecFormatValidator::new(format_rules);
        let semantic_rules = ValidationRules::for_document_type(DocumentType::Spec);
        let semantic_validator = SemanticValidator::new(semantic_rules);

        let mut issues = Vec::new();

        for file_name in created_files {
            if file_name.starts_with('_') {
                continue;
            }

            let spec_path = output_dir.join(file_name);

            let format_result = format_validator.validate(&spec_path);
            for error in &format_result.errors {
                issues.push(format!("{}: [format] {}", file_name, error.message));
            }

            let semantic_result = semantic_validator.validate(&spec_path);
            for error in &semantic_result.errors {
                issues.push(format!("{}: [semantic] {}", file_name, error.message));
            }
        }

        Ok(issues)
    }

    /// Generate module-specific specification
    /// Generate module-specific TD spec.
    ///
    /// Produces a spec that `aw td validate` accepts:
    /// - YAML frontmatter with `id` + `fill_sections`
    /// - `## Overview` — Markdown prose describing the module + its symbols
    /// - `## Changes` — YAML block listing the module file with
    ///   `impl_mode: hand-written` so `aw td gen-code` skips it
    ///   (Rule 2-2). Future fillback slices will emit Schema / Logic
    ///   sections when AST coverage deepens enough to produce valid
    ///   Mermaid Plus content.
    fn generate_module_spec(&self, module: &ModuleInfo) -> String {
        let mut c = String::new();

        let slug = Self::slugify(&module.name);
        let has_schemas = module
            .symbols
            .iter()
            .any(|s| s.is_public && matches!(s.kind, SymbolKind::Struct | SymbolKind::Enum));
        let logic_fns: Vec<&Symbol> = module
            .symbols
            .iter()
            .filter(|s| s.is_public && s.logic.is_some())
            .collect();
        let has_logic = !logic_fns.is_empty();

        c.push_str("---\n");
        c.push_str(&format!("id: {}\n", slug));
        let sections = match (has_schemas, has_logic) {
            (true, true) => "fill_sections: [overview, schema, logic, changes]\n",
            (true, false) => "fill_sections: [overview, schema, changes]\n",
            (false, true) => "fill_sections: [overview, logic, changes]\n",
            (false, false) => "fill_sections: [overview, changes]\n",
        };
        c.push_str(sections);
        c.push_str("---\n\n");

        c.push_str("## Overview\n");
        c.push_str("<!-- type: overview lang: markdown -->\n\n");
        c.push_str(&format!(
            "Module `{}` ({}) — {} symbol(s) ({} public). Spec auto-generated \
             by `score fillback`; `impl_mode: hand-written` until extended with \
             proper Schema / Logic sections.\n\n",
            module.name,
            module.language.display_name(),
            module.symbols.len(),
            module.symbols.iter().filter(|s| s.is_public).count(),
        ));

        if !module.symbols.is_empty() {
            c.push_str("### Symbols\n\n");
            c.push_str("| Name | Kind | Visibility | Line |\n");
            c.push_str("|------|------|------------|------|\n");
            for s in &module.symbols {
                let vis = if s.is_public { "pub" } else { "priv" };
                c.push_str(&format!(
                    "| `{}` | {} | {} | {} |\n",
                    s.name, s.kind, vis, s.line,
                ));
            }
            c.push('\n');
        }

        // Public function signatures as prose — until the Interface section
        // generator supports this explicitly, keeping sigs inside Overview
        // under a Markdown code fence is the most honest representation.
        let funcs: Vec<_> = module
            .symbols
            .iter()
            .filter(|s| s.signature.is_some() && s.is_public)
            .collect();
        if !funcs.is_empty() {
            c.push_str("### Public Signatures\n\n```rust\n");
            for f in funcs {
                if let Some(doc) = &f.doc {
                    for line in doc.lines() {
                        c.push_str(&format!("/// {}\n", line));
                    }
                }
                if let Some(sig) = &f.signature {
                    c.push_str(&format!("{}\n\n", sig));
                }
            }
            c.push_str("```\n\n");
        }

        if !module.imports.is_empty() {
            c.push_str("### Imports\n\n");
            for imp in &module.imports {
                let kind = if imp.is_external {
                    "external"
                } else {
                    "internal"
                };
                c.push_str(&format!("- `{}` ({})\n", imp.path, kind));
            }
            c.push('\n');
        }

        // Schema section — emit one YAML schema entry per pub struct/enum
        // with captured fields/variants. Rule 2-2 (whole spec hand-written)
        // means these schemas won't drive codegen yet; they exist so TD
        // validation has something real to lint and future slices can flip
        // impl_mode: codegen without re-deriving the shape.
        if has_schemas {
            c.push_str("## Schema\n");
            c.push_str("<!-- type: schema lang: yaml -->\n\n");
            c.push_str("```yaml\n");
            c.push_str("schemas:\n");
            for sym in module.symbols.iter().filter(|s| s.is_public) {
                match sym.kind {
                    SymbolKind::Struct => emit_struct_schema(&mut c, sym),
                    SymbolKind::Enum => emit_enum_schema(&mut c, sym),
                    _ => {}
                }
            }
            c.push_str("```\n\n");
        }

        // Logic section — one Mermaid Plus LogicContent per pub fn whose
        // body has top-level if/match. Shallow extraction: condition text
        // as decision label, return Ok/Err mapped to terminal nodes,
        // bare calls mapped to process nodes. Nested control flow is not
        // re-recursed (one decision node per top-level if).
        if has_logic {
            for (i, sym) in logic_fns.iter().enumerate() {
                let logic = sym.logic.as_ref().expect("filtered above");
                c.push_str(&format!("## Logic: {}\n", sym.name));
                c.push_str("<!-- type: logic lang: mermaid -->\n\n");
                c.push_str("```yaml\n");
                emit_logic_content(&mut c, logic);
                c.push_str("```\n\n");
                // Separator between multiple logic sections.
                if i + 1 < logic_fns.len() {
                    // no-op visual; keeps preceding newlines
                }
            }
        }

        // Changes section — critical for TD compliance. impl_mode: hand-written
        // marks this entry as out of codegen's path so gen-code is a no-op.
        // YAML literal-block `description: |` survives the inner newlines.
        c.push_str("## Changes\n");
        c.push_str("<!-- type: changes lang: yaml -->\n\n");
        c.push_str("```yaml\n");
        c.push_str("changes:\n");
        c.push_str(&format!("  - path: {}\n", module.path));
        c.push_str("    action: modify\n");
        c.push_str("    impl_mode: hand-written\n");
        c.push_str(
            "    description: |\n\
             \x20     Pre-existing module captured by `score fillback`.\n\
             \x20     Governance: hand-written until extended with\n\
             \x20     Schema / Logic / Interface sections.\n",
        );
        c.push_str("```\n");

        c
    }

    /// Produce a filesystem-safe slug from a module name. Rust-allowed
    /// identifiers (`snake_case`) pass through; anything else gets
    /// lowercased with non-alphanumerics replaced by `-`.
    fn slugify(name: &str) -> String {
        let mut out = String::with_capacity(name.len());
        let mut prev_dash = false;
        for ch in name.chars() {
            if ch.is_ascii_alphanumeric() {
                out.push(ch.to_ascii_lowercase());
                prev_dash = false;
            } else if !prev_dash && !out.is_empty() {
                out.push('-');
                prev_dash = true;
            }
        }
        while out.ends_with('-') {
            out.pop();
        }
        if out.is_empty() {
            "mod".to_string()
        } else {
            out
        }
    }

    /// Print summary of skipped files with errors
    pub fn print_parse_errors(&self, errors: &[ParseError]) {
        if errors.is_empty() {
            return;
        }

        println!();
        println!("{}", "Parse Errors".yellow().bold());
        println!("{}", "------------".bright_black());
        for error in errors.iter().take(10) {
            println!("  {}: {}", error.path.bright_black(), error.reason);
        }
        if errors.len() > 10 {
            println!("  ... and {} more", errors.len() - 10);
        }
        println!();
    }
}

/// Emit the LogicContent as Mermaid Plus YAML (frontmatter shape — no
/// fenced mermaid rendering follows; that's a visual aid callers can add
/// by hand after generation). The YAML passes
/// `serde_yaml::from_value::<LogicContent>` by construction: the content
/// we emit is the same shape we'd deserialise from.
fn emit_logic_content(
    c: &mut String,
    logic: &crate::generate::diagrams::content::logic::LogicContent,
) {
    c.push_str(&format!("id: {}\n", logic.id));
    c.push_str(&format!("entry: {}\n", logic.entry));
    if let Some(title) = &logic.title {
        c.push_str(&format!("title: {}\n", title));
    }
    c.push_str("nodes:\n");
    let mut ids: Vec<&str> = logic.nodes.keys().map(|s| s.as_str()).collect();
    ids.sort();
    for id in ids {
        let n = &logic.nodes[id];
        let kind = match n.kind {
            crate::generate::diagrams::content::logic::FlowNodeKind::Start => "start",
            crate::generate::diagrams::content::logic::FlowNodeKind::Process => "process",
            crate::generate::diagrams::content::logic::FlowNodeKind::Decision => "decision",
            crate::generate::diagrams::content::logic::FlowNodeKind::Terminal => "terminal",
        };
        match &n.label {
            Some(label) => c.push_str(&format!(
                "  {}: {{ kind: {}, label: \"{}\" }}\n",
                id,
                kind,
                label.replace('"', "\\\""),
            )),
            None => c.push_str(&format!("  {}: {{ kind: {} }}\n", id, kind)),
        }
    }
    c.push_str("edges:\n");
    for e in &logic.edges {
        match &e.label {
            Some(label) => c.push_str(&format!(
                "  - {{ from: {}, to: {}, label: \"{}\" }}\n",
                e.from,
                e.to,
                label.replace('"', "\\\""),
            )),
            None => c.push_str(&format!("  - {{ from: {}, to: {} }}\n", e.from, e.to)),
        }
    }
}

/// Emit a single struct schema entry (`schemas:` list item) under a Mermaid
/// Plus-less YAML block. `title` and `rust_type` both take the Rust struct
/// name — they are the same for single-file hand-written TD, divergence
/// starts when a spec aliases / renames.
fn emit_struct_schema(c: &mut String, sym: &Symbol) {
    c.push_str(&format!("  - title: {}\n", sym.name));
    c.push_str("    type: object\n");
    c.push_str(&format!("    rust_type: {}\n", sym.name));
    let required: Vec<&str> = sym
        .fields
        .iter()
        .filter(|f| !f.rust_type.trim_start().starts_with("Option<"))
        .map(|f| f.name.as_str())
        .collect();
    if !required.is_empty() {
        c.push_str(&format!("    required: [{}]\n", required.join(", ")));
    }
    if !sym.fields.is_empty() {
        c.push_str("    properties:\n");
        for f in &sym.fields {
            c.push_str(&format!("      {}:\n", f.name));
            // Inline-safe: types may contain `<>`, `:`, etc. — quote to
            // guarantee YAML parsability.
            c.push_str(&format!("        rust_type: \"{}\"\n", f.rust_type));
        }
    }
}

/// Emit a single enum schema entry. Variant payloads aren't represented —
/// fillback treats enums as string-enums for now; tuple/struct variants
/// become bare `enum: [name]` entries. Extending to payload types is a
/// follow-up when `EnumContent` supports it.
fn emit_enum_schema(c: &mut String, sym: &Symbol) {
    c.push_str(&format!("  - title: {}\n", sym.name));
    c.push_str("    type: enum\n");
    c.push_str(&format!("    rust_type: {}\n", sym.name));
    if !sym.variants.is_empty() {
        c.push_str("    enum:\n");
        for v in &sym.variants {
            c.push_str(&format!("      - {}\n", v));
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/code.md#source
impl Default for CodeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/code.md#source
impl ImportStrategy for CodeStrategy {
    async fn execute(&self, source: &Path, _change_id: &str) -> Result<()> {
        println!();
        println!(
            "{}",
            format!("Scanning codebase at: {}", source.display()).cyan()
        );

        // Step 1: Analyze codebase with AST
        let (context, parse_errors) = self.analyze_codebase(source)?;

        // Step 2: Build dependency graph
        let graph = DependencyGraph::from_analysis(&context);

        // Step 3: Display analysis summary
        self.display_summary(&context, &graph);

        // Step 4: Display dependency graph
        self.display_dependency_graph(&graph);

        // Step 5: Print any parse errors
        self.print_parse_errors(&parse_errors);

        // Step 6: Run interactive clarification
        let clarifications = self.run_clarification(&context)?;

        // Step 7: Determine output directory
        let output_dir = if let Some(ref dir) = self.config.output_dir {
            std::path::PathBuf::from(dir)
        } else {
            crate::shared::workspace::tech_design_path(&std::env::current_dir()?)
        };

        // Step 8: Check for existing specs
        let existing_specs = self.check_existing_specs(&output_dir)?;

        // Step 9: Confirm overwrite if needed
        if !self.confirm_overwrite(&existing_specs)? {
            println!("{}", "Cancelled by user.".yellow());
            return Ok(());
        }

        // Step 10: Generate specification files
        println!();
        println!("{}", "Generating specifications...".cyan());
        let created_files = self.generate_specs(&context, &graph, &output_dir, &clarifications)?;

        // Step 11: Validate specs (fillback is now AST-only; LLM enrichment
        // via subprocess was removed when Score switched to client-dispatched
        // executor model. For LLM enrichment, invoke `score fillback` from
        // within a Claude Code session — the mainthread/subagent can read
        // AST-generated specs and enrich them using the normal edit tools.)
        if !self.config.quick {
            println!();
            println!("{}", "Validation".cyan().bold());
            println!("{}", "----------".bright_black());
            let issues = self.validate_specs(&output_dir, &created_files)?;
            if issues.is_empty() {
                println!("  {}", "✓ All specs pass validation".green());
            } else {
                println!(
                    "  {}",
                    format!("⚠ {} validation issues found:", issues.len()).yellow()
                );
                for issue in issues.iter().take(10) {
                    println!("    - {}", issue);
                }
                if issues.len() > 10 {
                    println!("    ... and {} more", issues.len() - 10);
                }
            }
        }

        // Step 13: Summary
        println!();
        println!("{}", "Generated Files".green().bold());
        println!("{}", "---------------".bright_black());
        for file in &created_files {
            println!("  {}", output_dir.join(file).display());
        }

        let mode = if self.config.quick {
            " (quick/AST-only)"
        } else {
            " (LLM-enriched)"
        };
        println!();
        println!(
            "{}",
            format!(
                "Generated {} specification files in {}{}",
                created_files.len(),
                output_dir.display(),
                mode,
            )
            .green()
            .bold()
        );

        Ok(())
    }

    fn can_handle(&self, source: &Path) -> bool {
        source.is_dir()
    }

    fn name(&self) -> &'static str {
        "code"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project(dir: &Path) {
        let src_dir = dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create main.rs
        fs::write(
            src_dir.join("main.rs"),
            r#"
use std::path::Path;

/// Main entry point
pub fn main() {
    println!("Hello, world!");
}

fn helper() -> i32 {
    42
}
"#,
        )
        .unwrap();

        // Create lib.rs
        fs::write(
            src_dir.join("lib.rs"),
            r#"
pub mod utils;

pub struct Config {
    pub name: String,
}

pub fn init() -> Config {
    Config { name: "test".to_string() }
}
"#,
        )
        .unwrap();

        // Create utils.rs
        fs::write(
            src_dir.join("utils.rs"),
            r#"
use std::collections::HashMap;

pub fn format_string(s: &str) -> String {
    s.to_uppercase()
}

enum InternalEnum {
    A,
    B,
}
"#,
        )
        .unwrap();
    }

    #[test]
    fn test_scan_files() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let strategy = CodeStrategy::new();
        let files = strategy.scan_files(&temp_dir.path().join("src")).unwrap();

        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|(path, _)| path.contains("main.rs")));
        assert!(files.iter().any(|(path, _)| path.contains("lib.rs")));
        assert!(files.iter().any(|(path, _)| path.contains("utils.rs")));
    }

    #[test]
    fn test_analyze_codebase() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let strategy = CodeStrategy::new();
        let (context, errors) = strategy
            .analyze_codebase(&temp_dir.path().join("src"))
            .unwrap();

        assert_eq!(context.modules.len(), 3);
        assert!(errors.is_empty() || errors.len() < context.modules.len());

        // Check symbols were extracted
        let total_symbols: usize = context.modules.iter().map(|m| m.symbols.len()).sum();
        assert!(total_symbols > 0);

        // Check language counts
        assert!(context.language_counts.contains_key("Rust"));
    }

    #[test]
    fn test_analyze_with_module_filter() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let strategy = CodeStrategy::with_config(CodeStrategyConfig {
            module: Some("main".to_string()),
            ..Default::default()
        });

        let (context, _) = strategy
            .analyze_codebase(&temp_dir.path().join("src"))
            .unwrap();

        assert_eq!(context.modules.len(), 1);
        assert_eq!(context.modules[0].name, "main");
    }

    #[test]
    fn test_check_existing_specs() {
        let temp_dir = TempDir::new().unwrap();
        let specs_dir = temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        // Create some existing spec files
        fs::write(specs_dir.join("overview.md"), "# Overview").unwrap();
        fs::write(specs_dir.join("module_a.md"), "# Module A").unwrap();

        let strategy = CodeStrategy::new();
        let existing = strategy.check_existing_specs(&specs_dir).unwrap();

        assert_eq!(existing.len(), 2);
        assert!(existing.contains(&"overview.md".to_string()));
        assert!(existing.contains(&"module_a.md".to_string()));
    }

    #[test]
    fn test_generate_module_spec() {
        use crate::fillback::ast::{Import, Symbol, SymbolKind};

        let module = ModuleInfo {
            name: "test_module".to_string(),
            path: "src/test_module.rs".to_string(),
            language: SupportedLanguage::Rust,
            symbols: vec![
                Symbol {
                    name: "public_fn".to_string(),
                    kind: SymbolKind::Function,
                    signature: Some("public_fn(x: i32) -> String".to_string()),
                    doc: Some("A public function".to_string()),
                    line: 5,
                    is_public: true,
                    ..Default::default()
                },
                Symbol {
                    name: "TestStruct".to_string(),
                    kind: SymbolKind::Struct,
                    signature: None,
                    doc: None,
                    line: 10,
                    is_public: true,
                    ..Default::default()
                },
            ],
            imports: vec![Import {
                path: "std::collections".to_string(),
                items: vec![],
                is_external: true,
            }],
        };

        let strategy = CodeStrategy::new();
        let spec = strategy.generate_module_spec(&module);

        // TD-format frontmatter + section annotations.
        assert!(
            spec.starts_with("---\n"),
            "TD spec must open with YAML frontmatter"
        );
        assert!(
            spec.contains("id: test-module"),
            "id slug should reflect module name"
        );
        assert!(spec.contains("## Overview"));
        assert!(spec.contains("<!-- type: overview lang: markdown -->"));
        assert!(spec.contains("## Changes"));
        assert!(spec.contains("<!-- type: changes lang: yaml -->"));
        assert!(spec.contains("impl_mode: hand-written"));
        // Symbol content is preserved in prose.
        assert!(spec.contains("public_fn"));
        assert!(spec.contains("TestStruct"));
        assert!(spec.contains("std::collections"));
    }

    #[test]
    fn test_generate_specs_creates_files() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        let strategy = CodeStrategy::new();
        let (context, _) = strategy
            .analyze_codebase(&temp_dir.path().join("src"))
            .unwrap();
        let graph = DependencyGraph::from_analysis(&context);

        let output_dir = temp_dir.path().join("specs");
        let clarifications = HashMap::new();

        let created = strategy
            .generate_specs(&context, &graph, &output_dir, &clarifications)
            .unwrap();

        assert!(!created.is_empty());
        assert!(output_dir.join("_dependency-graph.md").exists());
        assert!(output_dir.join("_overview.md").exists());

        // At least one module spec should exist
        assert!(created.iter().any(|f| !f.starts_with('_')));
    }

    /// Regression for cb_claim_path_inference_bug: two source files
    /// sharing a basename (e.g. `mod.rs` in different subdirs) must
    /// produce two distinct spec files mirroring the source-tree layout
    /// under `output_dir`. Previously both landed at
    /// `output_dir/mod.md` and silently overwrote each other.
    #[test]
    fn test_generate_specs_mirrors_source_dir_structure() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        // Build .aw/tech-design under project_root so the
        // .parent().parent() recovery used by generate_specs reaches
        // project_root.
        let output_dir = project_root.join(".aw").join("tech-design");
        fs::create_dir_all(&output_dir).unwrap();

        // Two files with the same basename in different subdirs.
        let ui_dir = project_root.join("projects/agentic-workflow/src/ui");
        let viewer_dir = project_root.join("projects/agentic-workflow/src/ui/viewer");
        fs::create_dir_all(&ui_dir).unwrap();
        fs::create_dir_all(&viewer_dir).unwrap();
        fs::write(
            ui_dir.join("mod.rs"),
            "pub fn ui_root() {}\npub struct UiRoot;\n",
        )
        .unwrap();
        fs::write(
            viewer_dir.join("mod.rs"),
            "pub fn viewer_mod() {}\npub struct ViewerMod;\n",
        )
        .unwrap();

        let strategy = CodeStrategy::new();
        let (context, _) = strategy
            .analyze_codebase(&project_root.join("projects"))
            .unwrap();
        let graph = DependencyGraph::from_analysis(&context);
        let clarifications = HashMap::new();
        let created = strategy
            .generate_specs(&context, &graph, &output_dir, &clarifications)
            .unwrap();

        // Both subdirs must be present, mirrored under output_dir.
        let ui_spec = output_dir.join("projects/agentic-workflow/src/ui/mod.md");
        let viewer_spec = output_dir.join("projects/agentic-workflow/src/ui/viewer/mod.md");
        assert!(
            ui_spec.exists(),
            "ui/mod.md should exist; created={:?}",
            created
        );
        assert!(
            viewer_spec.exists(),
            "ui/viewer/mod.md should exist; created={:?}",
            created
        );
        // The two files must be distinct (no flatten-overwrite).
        let a = fs::read_to_string(&ui_spec).unwrap();
        let b = fs::read_to_string(&viewer_spec).unwrap();
        assert_ne!(a, b, "mirrored specs must contain distinct content");
        assert!(a.contains("UiRoot") || a.contains("ui_root"));
        assert!(b.contains("ViewerMod") || b.contains("viewer_mod"));
    }

    #[test]
    fn test_can_handle() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("test.rs");
        fs::write(&file, "fn main() {}").unwrap();

        let strategy = CodeStrategy::new();
        assert!(strategy.can_handle(temp_dir.path()));
        assert!(!strategy.can_handle(&file));
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let strategy = CodeStrategy::new();

        let result = strategy.analyze_codebase(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_force_overwrite() {
        let strategy = CodeStrategy::with_config(CodeStrategyConfig {
            force: true,
            ..Default::default()
        });

        let existing = vec!["file1.md".to_string(), "file2.md".to_string()];
        assert!(strategy.confirm_overwrite(&existing).unwrap());
    }
}
// CODEGEN-END
