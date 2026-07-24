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
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

// SPEC-MANAGED: apps/agentic-workflow/tech-design/core/generate/fillback/code.md#schema
// CODEGEN-BEGIN
/// Code import strategy with AST-based analysis.
/// @spec apps/agentic-workflow/tech-design/core/generate/fillback/code.md#schema
pub struct CodeStrategy {
    /// Strategy configuration.
    config: CodeStrategyConfig,
}

/// Configuration for the code analysis strategy.
/// @spec apps/agentic-workflow/tech-design/core/generate/fillback/code.md#schema
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

// SPEC-MANAGED: apps/agentic-workflow/tech-design/core/generate/fillback/code.md#source
// CODEGEN-BEGIN
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExplicitSourceFileOutcome {
    pub(crate) spec_path: Option<PathBuf>,
    pub(crate) target_path: PathBuf,
    pub(crate) refreshed_existing: bool,
    pub(crate) partition_count: usize,
    pub(crate) item_count: usize,
    pub(crate) requires_hitl: bool,
    pub(crate) message: String,
}

impl ExplicitSourceFileOutcome {
    fn hitl(target_path: PathBuf, message: impl Into<String>) -> Self {
        Self {
            spec_path: None,
            target_path,
            refreshed_existing: false,
            partition_count: 0,
            item_count: 0,
            requires_hitl: true,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceUnitPersistOutcome {
    Written,
    ConcurrentDrift,
}

fn persist_source_unit_candidate(
    spec_path: &Path,
    candidate: &str,
    existing_snapshot: Option<&str>,
) -> std::io::Result<SourceUnitPersistOutcome> {
    if let Some(snapshot) = existing_snapshot {
        let current = fs::read_to_string(spec_path)?;
        if current != snapshot {
            return Ok(SourceUnitPersistOutcome::ConcurrentDrift);
        }
        fs::write(spec_path, candidate)?;
        return Ok(SourceUnitPersistOutcome::Written);
    }

    let mut file = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(spec_path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Ok(SourceUnitPersistOutcome::ConcurrentDrift);
        }
        Err(error) => return Err(error),
    };
    file.write_all(candidate.as_bytes())?;
    Ok(SourceUnitPersistOutcome::Written)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourcePartitionBoundary {
    Ast,
    OversizedAstFallback,
    ParseFallback,
}

impl SourcePartitionBoundary {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ast => "ast",
            Self::OversizedAstFallback => "oversized-ast-fallback",
            Self::ParseFallback => "parse-fallback",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourcePartition {
    content: String,
    boundary: SourcePartitionBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceUnitFormat {
    language: SupportedLanguage,
    section_type: &'static str,
    fence_lang: &'static str,
}

impl SourceUnitFormat {
    fn for_language(language: SupportedLanguage) -> Self {
        match language {
            SupportedLanguage::Rust => Self {
                language,
                section_type: "rust-source-unit",
                fence_lang: "rust",
            },
            SupportedLanguage::Python => Self {
                language,
                section_type: "text-source-unit",
                fence_lang: "bash",
            },
            SupportedLanguage::JavaScript => Self {
                language,
                section_type: "text-source-unit",
                fence_lang: "bash",
            },
            SupportedLanguage::TypeScript => Self {
                language,
                section_type: "text-source-unit",
                fence_lang: "bash",
            },
            SupportedLanguage::Go => Self {
                language,
                section_type: "text-source-unit",
                fence_lang: "bash",
            },
        }
    }

    fn accepts_owner_section(self, section: &str) -> bool {
        section == "source" || section == self.section_type
    }

    fn source_lang(self) -> &'static str {
        match self.language {
            SupportedLanguage::Rust => "rust",
            SupportedLanguage::Python => "python",
            SupportedLanguage::JavaScript => "javascript",
            SupportedLanguage::TypeScript => "typescript",
            SupportedLanguage::Go => "go",
        }
    }
}

/// @spec apps/agentic-workflow/tech-design/core/generate/fillback/code.md#source
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

/// @spec apps/agentic-workflow/tech-design/core/generate/fillback/code.md#source
impl CodeStrategy {
    pub fn new() -> Self {
        Self {
            config: CodeStrategyConfig::default(),
        }
    }

    pub fn with_config(config: CodeStrategyConfig) -> Self {
        Self { config }
    }

    /// Scan a source directory or one explicitly selected file for analysis.
    ///
    /// The directory path keeps the bounded 100 KB/file discovery ceiling.
    /// An explicit file is the user's already-bounded selection: analyze only
    /// that file, never its siblings, and do not apply the directory scanner's
    /// size ceiling (#1506).
    fn scan_files(&self, source: &Path) -> Result<Vec<(String, String)>> {
        if source.is_file() {
            let ext = source.extension().and_then(|s| s.to_str()).unwrap_or("");
            if SupportedLanguage::from_extension(ext).is_none() {
                anyhow::bail!("Unsupported source file: {}", source.display());
            }
            let content = fs::read_to_string(source)
                .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", source.display()))?;
            let name = source
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("source")
                .to_string();
            return Ok(vec![(name, content)]);
        }

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
            let full_path = if source.is_file() {
                source.to_path_buf()
            } else {
                source.join(&rel_path)
            };

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

    /// Adopt one supported source file as a lossless source-unit TD.
    ///
    /// Existing CODEGEN ownership is refreshed only when the target resolves
    /// to exactly one project-local owner and that owner already declares a
    /// compatible whole-file source unit. Ambiguous or partial ownership is a
    /// no-mutation HITL result. New files receive a deterministic per-source
    /// spec under the caller's fillback output directory.
    pub(crate) fn import_explicit_source_file(
        &self,
        source: &Path,
        project_root: &Path,
        output_dir: &Path,
    ) -> Result<ExplicitSourceFileOutcome> {
        let canonical_root = project_root.canonicalize().map_err(|e| {
            anyhow::anyhow!(
                "failed to canonicalize project root {}: {e}",
                project_root.display()
            )
        })?;
        let canonical_source = source.canonicalize().map_err(|e| {
            anyhow::anyhow!("failed to canonicalize source {}: {e}", source.display())
        })?;
        let Ok(target_rel_path) = canonical_source.strip_prefix(&canonical_root) else {
            return Ok(ExplicitSourceFileOutcome::hitl(
                source.to_path_buf(),
                format!(
                    "explicit source is outside the repository root: {}",
                    source.display()
                ),
            ));
        };
        let target_rel = normalize_spec_path(target_rel_path);
        let target_path = PathBuf::from(&target_rel);
        if target_rel.chars().any(char::is_whitespace) {
            return Ok(ExplicitSourceFileOutcome::hitl(
                target_path,
                "explicit source paths containing whitespace cannot yet be emitted as a chain-validated next command",
            ));
        }
        let extension = canonical_source
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();
        let Some(language) = SupportedLanguage::from_extension(extension) else {
            return Ok(ExplicitSourceFileOutcome::hitl(
                target_path,
                format!("unsupported explicit source extension: {extension}"),
            ));
        };
        let source_format = SourceUnitFormat::for_language(language);
        let source_content = fs::read_to_string(&canonical_source)
            .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", source.display()))?;

        let (partitions, item_count) = if language == SupportedLanguage::Rust {
            let parsed_unit = crate::generate::rust_source_unit::parse(&source_content).ok();
            if parsed_unit
                .as_ref()
                .is_some_and(|unit| unit.emit() != source_content)
            {
                return Ok(ExplicitSourceFileOutcome::hitl(
                    target_path,
                    "rust-source-unit parse/emit was not byte-identical; refusing to mutate TD ownership",
                ));
            }
            let item_count = parsed_unit.as_ref().map_or(0, |unit| unit.items().count());
            let partitions = if source_content.len()
                > crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES
                || parsed_unit.is_none()
                || source_needs_partition_encoding(&source_content)
            {
                Some(partition_rust_source(&source_content, parsed_unit.as_ref()))
            } else {
                None
            };
            (partitions, item_count)
        } else {
            let mut analyzer = AstAnalyzer::new()?;
            let ast_boundaries = analyzer
                .top_level_byte_boundaries(&canonical_source, &source_content)
                .ok();
            let item_count = ast_boundaries.as_ref().map_or(0, Vec::len);
            // Non-Rust source units always use the partition manifest, even
            // when small, so the real parser language is explicit and can be
            // checked against the target extension while the TD's canonical
            // text-source-unit fence language remains `bash`.
            let partitions = Some(partition_text_source(
                &source_content,
                ast_boundaries.as_deref(),
            ));
            (partitions, item_count)
        };
        let partition_count = partitions.as_ref().map_or(1, Vec::len);

        let Some(td_root) = tech_design_root_from_output(output_dir) else {
            return Ok(ExplicitSourceFileOutcome::hitl(
                target_path,
                format!(
                    "fillback output {} is not inside a tech-design root",
                    output_dir.display()
                ),
            ));
        };

        let blocks = match crate::generate::apply::parse_source_codegen_blocks(
            &canonical_source,
            &source_content,
        ) {
            Ok(blocks) => blocks,
            Err(error) => {
                return Ok(ExplicitSourceFileOutcome::hitl(
                    target_path,
                    format!("source ownership markers are ambiguous: {error}"),
                ));
            }
        };
        let marker_owner_section = blocks
            .first()
            .and_then(|block| spec_ref_section(&block.spec_ref))
            .map(str::to_string);
        let marker_refs: BTreeSet<String> = blocks
            .iter()
            .filter_map(|block| spec_ref_path(&block.spec_ref))
            .collect();
        if !blocks.is_empty() {
            if marker_refs.len() != 1 {
                return Ok(ExplicitSourceFileOutcome::hitl(
                    target_path,
                    "source has CODEGEN markers without exactly one resolvable TD owner; whole-file fillback requires HITL",
                ));
            }
            let source_lines = source_content.lines().collect::<Vec<_>>();
            let block_start = blocks[0].begin_line.saturating_sub(1);
            let prefix = source_lines.get(..block_start).unwrap_or_default();
            let suffix = source_lines
                .get(blocks[0].end_line.saturating_add(1)..)
                .unwrap_or_default();
            let allowed_prefix = prefix.iter().all(|line| line.trim().is_empty())
                || (prefix
                    .first()
                    .is_some_and(|line| crate::generate::apply::is_unix_shebang(line))
                    && prefix[1..].iter().all(|line| line.trim().is_empty()));
            let is_one_full_file_block = blocks.len() == 1
                && blocks[0].begin_line > 0
                && allowed_prefix
                && suffix.iter().all(|line| line.trim().is_empty())
                && spec_ref_section(&blocks[0].spec_ref)
                    .is_some_and(|section| source_format.accepts_owner_section(section));
            if !is_one_full_file_block {
                return Ok(ExplicitSourceFileOutcome::hitl(
                    target_path,
                    "source has partial, multiple, or non-source CODEGEN ownership; whole-file fillback requires HITL",
                ));
            }
        }

        let mut owners = collect_codegen_owner_specs(&td_root, &canonical_root, &target_rel)?;
        owners.extend(marker_refs);
        if owners.len() > 1 {
            return Ok(ExplicitSourceFileOutcome::hitl(
                target_path,
                format!(
                    "multiple CODEGEN TD owners claim `{target_rel}`: {}",
                    owners.into_iter().collect::<Vec<_>>().join(", ")
                ),
            ));
        }

        let (spec_path, candidate, refreshed_existing, existing_snapshot) = if let Some(owner_ref) =
            owners.first()
        {
            let owner_path = canonical_root.join(owner_ref);
            if !path_is_within(&owner_path, &td_root) || !owner_path.is_file() {
                return Ok(ExplicitSourceFileOutcome::hitl(
                    target_path,
                    format!(
                        "CODEGEN owner `{owner_ref}` is missing or outside {}",
                        td_root.display()
                    ),
                ));
            }
            let existing = fs::read_to_string(&owner_path)
                .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", owner_path.display()))?;
            if let Err(error) = crate::generate::apply::validate_exact_source_spec_contract(
                &existing,
                &canonical_root,
                &canonical_source,
            ) {
                return Ok(ExplicitSourceFileOutcome::hitl(
                    target_path,
                    format!(
                        "CODEGEN owner `{owner_ref}` is not safe to refresh as one exact source unit: {error}"
                    ),
                ));
            }
            let change_entries = crate::generate::apply::extract_change_entries(&existing);
            let generated_source_entries = change_entries
                .iter()
                .filter(|entry| {
                    matches!(
                        entry.section_id.as_deref(),
                        Some("source" | "rust-source-unit" | "text-source-unit")
                    ) && entry.impl_mode == crate::generate::apply::ImplMode::Codegen
                })
                .collect::<Vec<_>>();
            let target_codegen_entries: Vec<_> = change_entries
                .iter()
                .filter(|entry| {
                    normalize_spec_path(Path::new(&entry.path)) == target_rel
                        && entry.impl_mode == crate::generate::apply::ImplMode::Codegen
                })
                .collect();
            if generated_source_entries.len() != 1
                || target_codegen_entries.len() != 1
                || !std::ptr::eq(generated_source_entries[0], target_codegen_entries[0])
            {
                return Ok(ExplicitSourceFileOutcome::hitl(
                    target_path,
                    format!(
                        "CODEGEN owner `{owner_ref}` must contain exactly one authoritative generated source entry and it must target `{target_rel}`",
                    ),
                ));
            }
            let owner_section = target_codegen_entries[0].section_id.as_deref();
            if !owner_section.is_some_and(|section| source_format.accepts_owner_section(section)) {
                return Ok(ExplicitSourceFileOutcome::hitl(
                    target_path,
                    format!(
                        "CODEGEN owner `{owner_ref}` does not contain a compatible {} change entry",
                        source_format.section_type,
                    ),
                ));
            }
            if marker_owner_section
                .as_deref()
                .is_some_and(|marker_section| Some(marker_section) != owner_section)
            {
                return Ok(ExplicitSourceFileOutcome::hitl(
                    target_path,
                    format!(
                        "source CODEGEN marker fragment `{}` does not match owner Changes section `{}`",
                        marker_owner_section.as_deref().unwrap_or_default(),
                        owner_section.unwrap_or_default(),
                    ),
                ));
            }
            let Some(updated) = replace_source_unit_section(
                &existing,
                &source_content,
                partitions.as_deref(),
                source_format,
            ) else {
                return Ok(ExplicitSourceFileOutcome::hitl(
                    target_path,
                    format!("CODEGEN owner `{owner_ref}` has no replaceable `## Source` section"),
                ));
            };
            (owner_path, updated, true, Some(existing))
        } else {
            let mut spec_path = output_dir.join(&target_rel);
            spec_path.set_extension("md");
            if spec_path.exists() {
                return Ok(ExplicitSourceFileOutcome::hitl(
                    target_path,
                    format!(
                        "unclaimed fillback artifact already exists at {}; refusing to overwrite it",
                        spec_path.display()
                    ),
                ));
            }
            (
                spec_path,
                render_source_unit_spec(
                    &target_rel,
                    &source_content,
                    partitions.as_deref(),
                    source_format,
                ),
                false,
                None,
            )
        };

        crate::td_ast::parse_td_str(&candidate).map_err(|e| {
            anyhow::anyhow!(
                "generated {} TD for {} is invalid: {}",
                source_format.section_type,
                source.display(),
                e.message
            )
        })?;
        let spec_rel = spec_path
            .strip_prefix(&canonical_root)
            .map(normalize_spec_path)
            .unwrap_or_else(|_| normalize_spec_path(&spec_path));
        let regenerated = crate::generate::apply::try_generate_source_section_code(
            &candidate,
            &spec_rel,
            Some(&target_rel),
            &canonical_root,
        )
        .map_err(anyhow::Error::msg)?;
        if regenerated != source_content {
            return Ok(ExplicitSourceFileOutcome::hitl(
                target_path,
                "normal source-unit generation would not reproduce the selected source; refusing to mutate TD ownership",
            ));
        }

        if let Some(parent) = spec_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if persist_source_unit_candidate(&spec_path, &candidate, existing_snapshot.as_deref())?
            == SourceUnitPersistOutcome::ConcurrentDrift
        {
            return Ok(ExplicitSourceFileOutcome::hitl(
                target_path,
                format!(
                    "source-unit TD `{}` changed or was created after preflight; refusing concurrent overwrite",
                    spec_path.display()
                ),
            ));
        }

        Ok(ExplicitSourceFileOutcome {
            spec_path: Some(spec_path),
            target_path,
            refreshed_existing,
            partition_count,
            item_count,
            requires_hitl: false,
            message: format!("lossless {} artifact written", source_format.section_type),
        })
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
    /// @spec apps/agentic-workflow/tech-design/surface/specs/score-recovery-verbs-non-interactive.md#logic
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
    /// @spec apps/agentic-workflow/tech-design/surface/specs/score-recovery-verbs-non-interactive.md#logic
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
        // files. Project root is derived explicitly from output_dir's
        // `tech-design` path component (#1313) rather than a fixed
        // parent-arithmetic depth, since callers use both the
        // single-level `<root>/tech-design` layout
        // (`workspace::tech_design_path`) and the `<root>/tech-design/specs`
        // layout (`aw td create --from-source`, #1273). When the module
        // path doesn't sit under project root (e.g. an absolute path
        // passed via --path that points outside), fall back to the
        // legacy flat name.
        let project_root_for_mirror: Option<PathBuf> =
            project_root_from_tech_design_output(output_dir);
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

/// Resolve the project root that `generate_specs` should mirror source-tree
/// paths against, from a `tech-design` output directory (#1313).
///
/// Walks `output_dir`'s ancestry looking for the `tech-design` path
/// component and returns its parent — this handles both known output
/// layouts without hardcoding a parent-arithmetic depth:
/// - `<root>/tech-design` (`workspace::tech_design_path`, one level below
///   root): the ancestor search finds `tech-design` at `output_dir` itself.
/// - `<root>/tech-design/specs` (`aw td create --from-source`, #1273, two
///   levels below root): the search finds `tech-design` one level up.
///
/// Returns `None` when no `tech-design` component is present in the path
/// (e.g. a caller-supplied `--output-dir` outside the tech-design tree),
/// matching the legacy flat-name fallback in `generate_specs`.
fn project_root_from_tech_design_output(output_dir: &Path) -> Option<PathBuf> {
    output_dir
        .ancestors()
        .find(|p| p.file_name().is_some_and(|n| n == "tech-design"))
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
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

/// @spec apps/agentic-workflow/tech-design/core/generate/fillback/code.md#source
impl Default for CodeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
/// @spec apps/agentic-workflow/tech-design/core/generate/fillback/code.md#source
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
            || (source.is_file()
                && source
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .and_then(SupportedLanguage::from_extension)
                    .is_some())
    }

    fn name(&self) -> &'static str {
        "code"
    }
}

fn normalize_spec_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

fn spec_ref_path(spec_ref: &str) -> Option<String> {
    let (path, _) = spec_ref.trim().split_once('#')?;
    let path = path.trim();
    (!path.is_empty()).then(|| normalize_spec_path(Path::new(path)))
}

fn spec_ref_section(spec_ref: &str) -> Option<&str> {
    spec_ref
        .trim()
        .split_once('#')
        .map(|(_, section)| section.trim())
}

fn tech_design_root_from_output(output_dir: &Path) -> Option<PathBuf> {
    output_dir
        .ancestors()
        .find(|path| path.file_name().is_some_and(|name| name == "tech-design"))
        .map(Path::to_path_buf)
}

fn path_is_within(candidate: &Path, root: &Path) -> bool {
    let Ok(root) = root.canonicalize() else {
        return false;
    };
    candidate
        .canonicalize()
        .is_ok_and(|candidate| candidate.starts_with(root))
}

fn collect_codegen_owner_specs(
    td_root: &Path,
    project_root: &Path,
    target_rel: &str,
) -> Result<BTreeSet<String>> {
    let mut owners = BTreeSet::new();
    if !td_root.exists() {
        return Ok(owners);
    }
    for entry in WalkBuilder::new(td_root).standard_filters(true).build() {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        if crate::generate::apply::extract_change_entries(&content)
            .into_iter()
            .any(|change| {
                normalize_spec_path(Path::new(&change.path)) == target_rel
                    && change.impl_mode == crate::generate::apply::ImplMode::Codegen
            })
        {
            let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            let spec_rel = canonical_path
                .strip_prefix(project_root)
                .map(normalize_spec_path)
                .unwrap_or_else(|_| normalize_spec_path(&canonical_path));
            owners.insert(spec_rel);
        }
    }
    Ok(owners)
}

fn replace_source_unit_section(
    existing: &str,
    source: &str,
    partitions: Option<&[SourcePartition]>,
    source_format: SourceUnitFormat,
) -> Option<String> {
    let lines: Vec<&str> = existing.lines().collect();
    let start = lines.iter().position(|line| line.trim() == "## Source")?;
    let end = next_h2_outside_markdown_fences(&lines, start + 1);

    let mut out = lines[..start].join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(&render_source_unit_section(
        source,
        partitions,
        source_format,
    ));
    if end < lines.len() {
        out.push_str(&lines[end..].join("\n"));
        out.push('\n');
    } else if existing.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

fn next_h2_outside_markdown_fences(lines: &[&str], start: usize) -> usize {
    let mut open: Option<(u8, usize)> = None;
    for (idx, line) in lines.iter().enumerate().skip(start) {
        if let Some((open_char, open_len)) = open {
            if markdown_fence_marker(line).is_some_and(|(close_char, close_len, suffix)| {
                close_char == open_char && close_len >= open_len && suffix.trim().is_empty()
            }) {
                open = None;
            }
            continue;
        }
        if let Some((fence_char, fence_len, _)) = markdown_fence_marker(line) {
            open = Some((fence_char, fence_len));
            continue;
        }
        if line.starts_with("## ") {
            return idx;
        }
    }
    lines.len()
}

fn markdown_fence_marker(line: &str) -> Option<(u8, usize, &str)> {
    let leading = line.len() - line.trim_start_matches(' ').len();
    if leading > 3 {
        return None;
    }
    let trimmed = &line[leading..];
    let first = *trimmed.as_bytes().first()?;
    if !matches!(first, b'`' | b'~') {
        return None;
    }
    let count = trimmed
        .as_bytes()
        .iter()
        .take_while(|byte| **byte == first)
        .count();
    (count >= 3).then_some((first, count, &trimmed[count..]))
}

fn render_source_unit_spec(
    target_rel: &str,
    source: &str,
    partitions: Option<&[SourcePartition]>,
    source_format: SourceUnitFormat,
) -> String {
    let slug = target_rel
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let escaped_target = target_rel.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        "---\nid: {slug}\nsummary: Lossless {} coverage for `{target_rel}`.\nfill_sections: [{}, changes]\n---\n\n# Fillback {target_rel}\n\n{}## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: \"{escaped_target}\"\n    action: modify\n    section: {}\n    impl_mode: codegen\n    description: |\n      Lossless {} ownership created from explicit file fillback.\n```\n",
        source_format.section_type,
        source_format.section_type,
        render_source_unit_section(source, partitions, source_format),
        source_format.section_type,
        source_format.section_type,
    )
}

fn render_source_unit_section(
    source: &str,
    partitions: Option<&[SourcePartition]>,
    source_format: SourceUnitFormat,
) -> String {
    let Some(partitions) = partitions else {
        return render_source_fence_section(
            "## Source",
            source_format.section_type,
            source_format.fence_lang,
            source,
            None,
        );
    };

    let source_digest = crate::generate::apply::partition_sha256(source.as_bytes());
    let manifest = format!(
        "// AW source partition manifest v1: {} ordered {} chunks, max {} decoded / {} encoded bytes, digest {}\n",
        partitions.len(),
        source_format.source_lang(),
        crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES,
        crate::generate::apply::RUST_SOURCE_PARTITION_MAX_PAYLOAD_BYTES,
        source_digest,
    );
    let mut section = format!(
        "## Source\n<!-- type: {} lang: {} -->\n<!-- aw-source-partitions: version=1 count={} max_bytes={} max_payload_bytes={} encoding=base64 source_lang={} digest={} -->\n\n{}",
        source_format.section_type,
        source_format.fence_lang,
        partitions.len(),
        crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES,
        crate::generate::apply::RUST_SOURCE_PARTITION_MAX_PAYLOAD_BYTES,
        source_format.source_lang(),
        source_digest,
        render_dynamic_fence(source_format.fence_lang, &manifest),
    );
    section.push('\n');

    for (idx, partition) in partitions.iter().enumerate() {
        let index = idx + 1;
        let terminal_newline = partition.content.ends_with('\n');
        let digest = crate::generate::apply::partition_sha256(partition.content.as_bytes());
        let payload =
            crate::generate::apply::encode_source_partition_payload(partition.content.as_bytes());
        debug_assert!(
            payload.len() <= crate::generate::apply::RUST_SOURCE_PARTITION_MAX_PAYLOAD_BYTES
        );
        section.push_str(&format!(
            "### Source Partition {index:04}\n<!-- aw-source-partition: index={index} count={} bytes={} payload_bytes={} encoding=base64 digest={} boundary={} terminal_newline={} -->\n\n```text\n{}\n```\n\n",
            partitions.len(),
            partition.content.len(),
            payload.len(),
            digest,
            partition.boundary.as_str(),
            terminal_newline,
            payload,
        ));
    }
    section
}

fn render_source_fence_section(
    heading: &str,
    section_type: &str,
    fence_lang: &str,
    source: &str,
    extra_annotation: Option<&str>,
) -> String {
    let mut section = format!("{heading}\n<!-- type: {section_type} lang: {fence_lang} -->\n");
    if let Some(annotation) = extra_annotation {
        section.push_str(annotation);
        section.push('\n');
    }
    section.push('\n');
    section.push_str(&render_dynamic_fence(fence_lang, source));
    section.push_str("\n\n");
    section
}

fn render_dynamic_fence(lang: &str, source: &str) -> String {
    let max_ticks = source
        .lines()
        .map(|line| {
            line.as_bytes()
                .iter()
                .fold((0usize, 0usize), |(best, run), byte| {
                    if *byte == b'`' {
                        (best.max(run + 1), run + 1)
                    } else {
                        (best, 0)
                    }
                })
                .0
        })
        .max()
        .unwrap_or(0);
    let fence = "`".repeat((max_ticks + 1).max(3));
    let mut section = format!("{fence}{lang}\n{source}");
    if !source.ends_with('\n') {
        section.push('\n');
    }
    section.push_str(&fence);
    section
}

fn source_needs_partition_encoding(source: &str) -> bool {
    source.contains('\r')
        || !source.ends_with('\n')
        || source.contains("// AW source partition manifest v1:")
        || source.contains("<!-- aw-source-partition")
        || source.contains("### Source Partition ")
        || source.lines().any(|line| {
            let trimmed = line.trim_start();
            line.len() - trimmed.len() <= 3
                && (trimmed.starts_with("```") || trimmed.starts_with("~~~"))
        })
}

fn partition_rust_source(
    source: &str,
    parsed: Option<&crate::generate::rust_source_unit::RustSourceUnit>,
) -> Vec<SourcePartition> {
    let Some(unit) = parsed else {
        return split_bounded_source(source, SourcePartitionBoundary::ParseFallback);
    };

    let mut partitions = Vec::new();
    let mut current = String::new();
    for segment in &unit.segments {
        let segment_source = match segment {
            crate::generate::rust_source_unit::Segment::Item(item) => item.text.as_str(),
            crate::generate::rust_source_unit::Segment::Trivia { text } => text.as_str(),
        };
        if segment_source.len() > crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES {
            if !current.is_empty() {
                partitions.push(SourcePartition {
                    content: std::mem::take(&mut current),
                    boundary: SourcePartitionBoundary::Ast,
                });
            }
            partitions.extend(split_bounded_source(
                segment_source,
                SourcePartitionBoundary::OversizedAstFallback,
            ));
            continue;
        }
        if !current.is_empty()
            && current.len() + segment_source.len()
                > crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES
        {
            partitions.push(SourcePartition {
                content: std::mem::take(&mut current),
                boundary: SourcePartitionBoundary::Ast,
            });
        }
        current.push_str(segment_source);
    }
    if !current.is_empty() {
        partitions.push(SourcePartition {
            content: current,
            boundary: SourcePartitionBoundary::Ast,
        });
    }
    if partitions.is_empty() {
        partitions.push(SourcePartition {
            content: String::new(),
            boundary: SourcePartitionBoundary::Ast,
        });
    }
    debug_assert_eq!(
        partitions
            .iter()
            .map(|partition| partition.content.as_str())
            .collect::<String>(),
        source
    );
    debug_assert!(partitions.iter().all(|partition| partition.content.len()
        <= crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES));
    partitions
}

fn partition_text_source(source: &str, ast_boundaries: Option<&[usize]>) -> Vec<SourcePartition> {
    let Some(ast_boundaries) = ast_boundaries else {
        return split_bounded_source(source, SourcePartitionBoundary::ParseFallback);
    };

    let mut partitions = Vec::new();
    let mut current = String::new();
    let mut start = 0usize;
    for end in ast_boundaries
        .iter()
        .copied()
        .chain(std::iter::once(source.len()))
    {
        if end <= start || end > source.len() || !source.is_char_boundary(end) {
            continue;
        }
        let segment = &source[start..end];
        if segment.len() > crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES {
            if !current.is_empty() {
                partitions.push(SourcePartition {
                    content: std::mem::take(&mut current),
                    boundary: SourcePartitionBoundary::Ast,
                });
            }
            partitions.extend(split_bounded_source(
                segment,
                SourcePartitionBoundary::OversizedAstFallback,
            ));
        } else {
            if !current.is_empty()
                && current.len() + segment.len()
                    > crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES
            {
                partitions.push(SourcePartition {
                    content: std::mem::take(&mut current),
                    boundary: SourcePartitionBoundary::Ast,
                });
            }
            current.push_str(segment);
        }
        start = end;
    }
    if start < source.len() {
        let tail = &source[start..];
        if tail.len() > crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES {
            if !current.is_empty() {
                partitions.push(SourcePartition {
                    content: std::mem::take(&mut current),
                    boundary: SourcePartitionBoundary::Ast,
                });
            }
            partitions.extend(split_bounded_source(
                tail,
                SourcePartitionBoundary::OversizedAstFallback,
            ));
        } else {
            current.push_str(tail);
        }
    }
    if !current.is_empty() {
        partitions.push(SourcePartition {
            content: current,
            boundary: SourcePartitionBoundary::Ast,
        });
    }
    if partitions.is_empty() {
        partitions.push(SourcePartition {
            content: String::new(),
            boundary: SourcePartitionBoundary::Ast,
        });
    }
    debug_assert_eq!(
        partitions
            .iter()
            .map(|partition| partition.content.as_str())
            .collect::<String>(),
        source
    );
    debug_assert!(partitions.iter().all(|partition| partition.content.len()
        <= crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES));
    partitions
}

fn split_bounded_source(source: &str, boundary: SourcePartitionBoundary) -> Vec<SourcePartition> {
    if source.is_empty() {
        return vec![SourcePartition {
            content: String::new(),
            boundary,
        }];
    }
    let mut partitions = Vec::new();
    let mut remaining = source;
    while remaining.len() > crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES {
        let mut limit =
            crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES.min(remaining.len());
        while !remaining.is_char_boundary(limit) {
            limit -= 1;
        }
        let split_at = remaining[..limit]
            .rfind('\n')
            .map(|idx| idx + 1)
            .filter(|idx| *idx > 0)
            .unwrap_or(limit);
        partitions.push(SourcePartition {
            content: remaining[..split_at].to_string(),
            boundary,
        });
        remaining = &remaining[split_at..];
    }
    if !remaining.is_empty() {
        partitions.push(SourcePartition {
            content: remaining.to_string(),
            boundary,
        });
    }
    partitions
}

/// Recompute the one canonical partition sequence for a decoded source unit.
/// The strict decoder uses this to prove that declared `ast`/fallback labels
/// and chunk endpoints came from AW's deterministic partitioner rather than a
/// digest-consistent but arbitrary mid-token split (#1506).
pub(crate) fn canonical_source_partition_plan(
    source: &str,
    source_lang: &str,
) -> Result<Vec<(String, String)>> {
    let language = match source_lang {
        "rust" => SupportedLanguage::Rust,
        "python" => SupportedLanguage::Python,
        "javascript" => SupportedLanguage::JavaScript,
        "typescript" => SupportedLanguage::TypeScript,
        "go" => SupportedLanguage::Go,
        other => anyhow::bail!("unsupported source partition language `{other}`"),
    };
    let partitions = if language == SupportedLanguage::Rust {
        let parsed = crate::generate::rust_source_unit::parse(source).ok();
        partition_rust_source(source, parsed.as_ref())
    } else {
        let extension = match language {
            SupportedLanguage::Python => "py",
            SupportedLanguage::JavaScript => "js",
            SupportedLanguage::TypeScript => "ts",
            SupportedLanguage::Go => "go",
            SupportedLanguage::Rust => unreachable!(),
        };
        let synthetic_path = PathBuf::from(format!("source.{extension}"));
        let mut analyzer = AstAnalyzer::new()?;
        let boundaries = analyzer
            .top_level_byte_boundaries(&synthetic_path, source)
            .ok();
        partition_text_source(source, boundaries.as_deref())
    };
    Ok(partitions
        .into_iter()
        .map(|partition| (partition.content, partition.boundary.as_str().to_string()))
        .collect())
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
    fn test_scan_explicit_large_file_ignores_siblings_and_directory_size_limit() {
        let temp_dir = TempDir::new().unwrap();
        let selected = temp_dir.path().join("selected.rs");
        let sibling = temp_dir.path().join("sibling.rs");
        let mut source = String::new();
        let payload = "x".repeat(2_048);
        for idx in 0..64 {
            source.push_str(&format!(
                "fn selected_{idx}() -> &'static str {{ \"{payload}\" }}\n"
            ));
        }
        assert!(source.len() > 100_000);
        fs::write(&selected, &source).unwrap();
        fs::write(&sibling, "pub fn sibling_must_not_be_scanned() {}\n").unwrap();

        let strategy = CodeStrategy::new();
        let files = strategy.scan_files(&selected).unwrap();

        assert_eq!(files, vec![("selected.rs".to_string(), source)]);
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
        // Build the configured tech-design root under project_root so the
        // .parent().parent() recovery used by generate_specs reaches
        // project_root.
        let output_dir = crate::shared::workspace::tech_design_path(project_root);
        fs::create_dir_all(&output_dir).unwrap();

        // Two files with the same basename in different subdirs.
        let ui_dir = project_root.join("apps/demo/src/ui");
        let panel_dir = ui_dir.join("panel");
        fs::create_dir_all(&ui_dir).unwrap();
        fs::create_dir_all(&panel_dir).unwrap();
        fs::write(
            ui_dir.join("mod.rs"),
            "pub fn ui_root() {}\npub struct UiRoot;\n",
        )
        .unwrap();
        fs::write(
            panel_dir.join("mod.rs"),
            "pub fn panel_mod() {}\npub struct PanelMod;\n",
        )
        .unwrap();

        let strategy = CodeStrategy::new();
        let (context, _) = strategy
            .analyze_codebase(&project_root.join("apps"))
            .unwrap();
        let graph = DependencyGraph::from_analysis(&context);
        let clarifications = HashMap::new();
        let created = strategy
            .generate_specs(&context, &graph, &output_dir, &clarifications)
            .unwrap();

        // Both subdirs must be present, mirrored under output_dir.
        let ui_spec = output_dir.join("apps/demo/src/ui/mod.md");
        let panel_spec = output_dir.join("apps/demo/src/ui/panel/mod.md");
        assert!(
            ui_spec.exists(),
            "ui/mod.md should exist; created={:?}",
            created
        );
        assert!(
            panel_spec.exists(),
            "ui/panel/mod.md should exist; created={:?}",
            created
        );
        // The two files must be distinct (no flatten-overwrite).
        let a = fs::read_to_string(&ui_spec).unwrap();
        let b = fs::read_to_string(&panel_spec).unwrap();
        assert_ne!(a, b, "mirrored specs must contain distinct content");
        assert!(a.contains("UiRoot") || a.contains("ui_root"));
        assert!(b.contains("PanelMod") || b.contains("panel_mod"));
    }

    /// #1313: `project_root_from_tech_design_output` must recover the
    /// correct project root for both known `tech-design` output layouts —
    /// the single-level `<root>/tech-design` layout
    /// (`workspace::tech_design_path`) and the two-level
    /// `<root>/tech-design/specs` layout (`aw td create --from-source`,
    /// #1273) — without a fixed parent-arithmetic depth.
    #[test]
    fn test_project_root_from_tech_design_output_both_layouts() {
        let direct = Path::new("/repo/apps/agentic-workflow/tech-design");
        assert_eq!(
            project_root_from_tech_design_output(direct),
            Some(PathBuf::from("/repo/apps/agentic-workflow"))
        );

        let nested = Path::new("/repo/apps/jet/tech-design/specs");
        assert_eq!(
            project_root_from_tech_design_output(nested),
            Some(PathBuf::from("/repo/apps/jet"))
        );

        let no_tech_design = Path::new("/tmp/some/other/output");
        assert_eq!(project_root_from_tech_design_output(no_tech_design), None);
    }

    fn large_complete_source(extension: &str) -> String {
        match extension {
            "py" => {
                let mut source = "#!/usr/bin/env python3\n".to_string();
                for idx in 0..2_500 {
                    source.push_str(&format!(
                        "def item_{idx}(value: int) -> int:\n    return value + {idx}\n\n"
                    ));
                }
                source
            }
            "js" => (0..2_500)
                .map(|idx| {
                    format!("export function item_{idx}(value) {{ return value + {idx}; }}\n")
                })
                .collect(),
            "ts" => (0..2_500)
                .map(|idx| {
                    format!(
                        "export function item_{idx}(value: number): number {{ return value + {idx}; }}\n"
                    )
                })
                .collect(),
            "go" => {
                let mut source = "package direct\n\n".to_string();
                for idx in 0..2_500 {
                    source.push_str(&format!(
                        "func Item{idx}(value int) int {{ return value + {idx} }}\n"
                    ));
                }
                source
            }
            other => panic!("unsupported fixture extension {other}"),
        }
    }

    #[test]
    fn explicit_large_python_javascript_typescript_and_go_files_are_lossless() {
        let cases = [
            ("py", "python", "# SPEC-MANAGED:"),
            ("js", "javascript", "// SPEC-MANAGED:"),
            ("ts", "typescript", "// SPEC-MANAGED:"),
            ("go", "go", "// SPEC-MANAGED:"),
        ];
        for (extension, source_lang, marker_prefix) in cases {
            let temp_dir = TempDir::new().unwrap();
            let root = temp_dir.path();
            let source_path = root.join(format!("apps/demo/src/direct.{extension}"));
            let output_dir = root.join("apps/demo/tech-design/specs");
            fs::create_dir_all(source_path.parent().unwrap()).unwrap();
            fs::create_dir_all(&output_dir).unwrap();
            let source = large_complete_source(extension);
            assert!(source.len() > 100_000, "{extension}");
            fs::write(&source_path, &source).unwrap();

            let first = CodeStrategy::new()
                .import_explicit_source_file(&source_path, root, &output_dir)
                .unwrap();
            assert!(!first.requires_hitl, "{extension}: {}", first.message);
            assert!(first.partition_count > 1, "{extension}");
            assert!(first.item_count > 2_000, "{extension}");
            let spec_path = first.spec_path.unwrap();
            let first_spec = fs::read_to_string(&spec_path).unwrap();
            assert!(first_spec.contains("<!-- type: text-source-unit lang: bash -->"));
            let decoded = crate::generate::apply::decode_partitioned_source(&first_spec)
                .unwrap()
                .unwrap();
            assert_eq!(decoded.source_lang, source_lang, "{extension}");
            assert_eq!(decoded.source, source, "{extension}");

            let generated = crate::generate::apply::try_generate_source_section_code(
                &first_spec,
                &normalize_spec_path(spec_path.strip_prefix(root).unwrap()),
                Some(&normalize_spec_path(
                    source_path.strip_prefix(root).unwrap(),
                )),
                root,
            )
            .unwrap();
            assert_eq!(generated, source, "{extension}: selected payload");

            let plan = canonical_source_partition_plan(&source, source_lang).unwrap();
            assert_eq!(plan.len(), first.partition_count, "{extension}");
            assert!(plan.iter().all(|(chunk, boundary)| {
                chunk.len() <= crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES
                    && crate::generate::apply::encode_source_partition_payload(chunk.as_bytes())
                        .len()
                        <= crate::generate::apply::RUST_SOURCE_PARTITION_MAX_PAYLOAD_BYTES
                    && boundary == "ast"
            }));
            assert_eq!(
                plan.iter()
                    .map(|(chunk, _)| chunk.as_str())
                    .collect::<String>(),
                source,
                "{extension}: bounded plan reassembles exactly"
            );

            let second = CodeStrategy::new()
                .import_explicit_source_file(&source_path, root, &output_dir)
                .unwrap();
            assert!(!second.requires_hitl, "{extension}: {}", second.message);
            assert_eq!(second.partition_count, first.partition_count, "{extension}");
            assert_eq!(
                fs::read_to_string(&spec_path).unwrap(),
                first_spec,
                "{extension}"
            );

            let report = crate::generate::apply::run_apply_exact_source_target(
                &spec_path,
                root,
                false,
                &source_path,
            )
            .unwrap();
            assert_eq!(report.files.iter().filter(|file| file.processed).count(), 1);
            let managed = fs::read_to_string(&source_path).unwrap();
            let blocks =
                crate::generate::apply::parse_source_codegen_blocks(&source_path, &managed)
                    .unwrap();
            assert_eq!(blocks.len(), 1, "{extension}");
            let expected_body = if extension == "py" {
                source
                    .strip_prefix("#!/usr/bin/env python3\n")
                    .expect("Python fixture shebang")
            } else {
                source.as_str()
            };
            assert_eq!(
                blocks[0].content,
                expected_body.trim_end_matches(['\r', '\n']),
                "{extension}"
            );
            assert!(blocks[0].spec_ref.ends_with("#text-source-unit"));
            if extension == "py" {
                assert!(managed.starts_with("#!/usr/bin/env python3\n# SPEC-MANAGED:"));
            } else {
                assert!(managed.starts_with(marker_prefix), "{extension}");
            }
        }
    }

    #[test]
    fn explicit_parse_incomplete_python_uses_deterministic_bounded_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let source_path = root.join("apps/demo/src/incomplete.py");
        let output_dir = root.join("apps/demo/tech-design/specs");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::create_dir_all(&output_dir).unwrap();
        let mut source = "def incomplete(\n".to_string();
        for idx in 0..4_000 {
            source.push_str(&format!(
                "# fallback {idx:04} with deterministic unicode-free payload and fake markers\n"
            ));
        }
        assert!(source.len() > 100_000);
        let mut analyzer = AstAnalyzer::new().unwrap();
        assert!(analyzer
            .top_level_byte_boundaries(&source_path, &source)
            .is_err());
        fs::write(&source_path, &source).unwrap();

        let first = CodeStrategy::new()
            .import_explicit_source_file(&source_path, root, &output_dir)
            .unwrap();
        assert!(!first.requires_hitl, "{}", first.message);
        assert_eq!(first.item_count, 0);
        assert!(first.partition_count > 1);
        let spec_path = first.spec_path.unwrap();
        let first_spec = fs::read_to_string(&spec_path).unwrap();
        let decoded = crate::generate::apply::decode_partitioned_source(&first_spec)
            .unwrap()
            .unwrap();
        assert_eq!(decoded.source_lang, "python");
        assert_eq!(decoded.source, source);
        let plan = canonical_source_partition_plan(&source, "python").unwrap();
        assert!(plan.iter().all(|(chunk, boundary)| {
            boundary == "parse-fallback"
                && chunk.len() <= crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES
                && crate::generate::apply::encode_source_partition_payload(chunk.as_bytes()).len()
                    <= crate::generate::apply::RUST_SOURCE_PARTITION_MAX_PAYLOAD_BYTES
        }));
        let second = CodeStrategy::new()
            .import_explicit_source_file(&source_path, root, &output_dir)
            .unwrap();
        assert_eq!(second.partition_count, first.partition_count);
        assert_eq!(fs::read_to_string(&spec_path).unwrap(), first_spec);

        crate::generate::apply::run_apply_exact_source_target(
            &spec_path,
            root,
            false,
            &source_path,
        )
        .unwrap();
        let managed = fs::read_to_string(&source_path).unwrap();
        assert!(managed.starts_with("# SPEC-MANAGED:"));
        assert!(managed.contains(&source));
        assert!(managed.ends_with("# CODEGEN-END\n"));
        let blocks = crate::generate::apply::parse_source_codegen_blocks(&source_path, &managed)
            .expect("AW's canonical wrapper remains recognizable around incomplete source");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].content, source.trim_end());
        let dry = crate::generate::apply::run_apply_exact_source_target(
            &spec_path,
            root,
            true,
            &source_path,
        )
        .unwrap();
        assert!(!dry.wrote_files);
        assert_eq!(fs::read_to_string(&source_path).unwrap(), managed);
        let repeated = crate::generate::apply::run_apply_exact_source_target(
            &spec_path,
            root,
            false,
            &source_path,
        )
        .unwrap();
        assert!(!repeated.wrote_files);
        assert_eq!(fs::read_to_string(&source_path).unwrap(), managed);
    }

    #[test]
    fn javascript_template_literal_marker_fixture_is_not_ownership() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let source_path = root.join("apps/demo/src/fixture.mjs");
        let output_dir = root.join("apps/demo/tech-design/specs");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::create_dir_all(&output_dir).unwrap();
        let source = "export const fixture = `\n// SPEC-MANAGED: fake.md#source\n// CODEGEN-BEGIN\nnot real ownership\n// CODEGEN-END\n`;\n";
        fs::write(&source_path, source).unwrap();
        assert!(
            crate::generate::apply::parse_source_codegen_blocks(&source_path, source)
                .unwrap()
                .is_empty()
        );
        let outcome = CodeStrategy::new()
            .import_explicit_source_file(&source_path, root, &output_dir)
            .unwrap();
        assert!(!outcome.requires_hitl, "{}", outcome.message);
        let spec = fs::read_to_string(outcome.spec_path.unwrap()).unwrap();
        assert_eq!(
            crate::generate::apply::decode_partitioned_source(&spec)
                .unwrap()
                .unwrap()
                .source,
            source
        );
    }

    #[test]
    fn canonical_typescript_owner_can_contain_fake_template_markers_and_refresh() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let source_path = root.join("apps/demo/src/fixture.ts");
        let spec_path = root.join("apps/demo/tech-design/semantic/source/fixture.md");
        let output_dir = root.join("apps/demo/tech-design/specs");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::create_dir_all(spec_path.parent().unwrap()).unwrap();
        fs::create_dir_all(&output_dir).unwrap();
        let spec_ref = "apps/demo/tech-design/semantic/source/fixture.md#text-source-unit";
        let source = format!(
            "// SPEC-MANAGED: {spec_ref}\n// CODEGEN-BEGIN\nconst fixture = `\n// SPEC-MANAGED: fake.md#text-source-unit\n// CODEGEN-BEGIN\n// CODEGEN-END\n`;\nexport const selected = 1;\n// CODEGEN-END\n"
        );
        fs::write(&source_path, &source).unwrap();
        let blocks =
            crate::generate::apply::parse_source_codegen_blocks(&source_path, &source).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].spec_ref, spec_ref);

        let stale_source = "export const stale = 0;\n";
        let mut analyzer = AstAnalyzer::new().unwrap();
        let stale_boundaries = analyzer
            .top_level_byte_boundaries(Path::new("fixture.ts"), stale_source)
            .unwrap();
        let stale_partitions = partition_text_source(stale_source, Some(&stale_boundaries));
        let format = SourceUnitFormat::for_language(SupportedLanguage::TypeScript);
        let stale_spec = render_source_unit_spec(
            "apps/demo/src/fixture.ts",
            stale_source,
            Some(&stale_partitions),
            format,
        );
        fs::write(&spec_path, stale_spec).unwrap();

        let outcome = CodeStrategy::new()
            .import_explicit_source_file(&source_path, root, &output_dir)
            .unwrap();
        assert!(!outcome.requires_hitl, "{}", outcome.message);
        assert!(outcome.refreshed_existing);
        let refreshed = fs::read_to_string(&spec_path).unwrap();
        let decoded = crate::generate::apply::decode_partitioned_source(&refreshed)
            .unwrap()
            .unwrap();
        assert_eq!(decoded.source_lang, "typescript");
        assert_eq!(decoded.source, source);
        assert_eq!(fs::read_to_string(&source_path).unwrap(), source);
    }

    #[test]
    fn explicit_large_rust_file_refreshes_one_owner_and_round_trips_losslessly() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let source_path = root.join("apps/demo/src/large.rs");
        let spec_path = root.join("apps/demo/tech-design/semantic/source/large.md");
        let output_dir = root.join("apps/demo/tech-design/specs");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::create_dir_all(spec_path.parent().unwrap()).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        let spec_ref = "apps/demo/tech-design/semantic/source/large.md#rust-source-unit";
        let mut original = format!("// SPEC-MANAGED: {spec_ref}\n// CODEGEN-BEGIN\n");
        original.push_str("fn large_source_unit() {\n");
        for idx in 0..7_500 {
            original.push_str(&format!(
                "    let value_{idx}: usize = {idx}; // realistic large function body\n"
            ));
        }
        original.push_str("}\n// CODEGEN-END\n");
        assert!(original.len() > 315_000);
        fs::write(&source_path, &original).unwrap();

        let stale = format!(
            "---\nid: large\ncapability_refs:\n  - id: preserved-capability\n    role: primary\n    gap: preserved-gap\n    claim: preserved-gap\n    coverage: full\nfill_sections: [overview, rust-source-unit, changes]\n---\n\n# Large\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPreserve this overview.\n\n## Source\n<!-- type: rust-source-unit lang: rust -->\n\n```rust\n// SPEC-MANAGED: {spec_ref}\n// CODEGEN-BEGIN\nfn stale() {{}}\n// CODEGEN-END\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/demo/src/large.rs\n    action: modify\n    section: rust-source-unit\n    impl_mode: codegen\n```\n"
        );
        fs::write(&spec_path, &stale).unwrap();

        let outcome = CodeStrategy::new()
            .import_explicit_source_file(&source_path, root, &output_dir)
            .unwrap();
        assert!(!outcome.requires_hitl, "{}", outcome.message);
        assert!(outcome.refreshed_existing);
        assert_eq!(
            outcome.spec_path.as_deref(),
            Some(spec_path.canonicalize().unwrap().as_path())
        );
        assert_eq!(outcome.item_count, 1);
        assert!(outcome.partition_count > 1);

        let refreshed = fs::read_to_string(&spec_path).unwrap();
        assert!(refreshed.contains("preserved-capability"));
        assert!(refreshed.contains("Preserve this overview."));
        assert_eq!(
            refreshed.matches("### Source Partition ").count(),
            outcome.partition_count
        );
        assert_eq!(
            crate::generate::apply::decode_partitioned_source(&refreshed)
                .unwrap()
                .as_ref()
                .map(|decoded| decoded.source.as_str()),
            Some(original.as_str())
        );
        let annotations = crate::models::section::parse_all_section_annotations(&refreshed);
        assert!(annotations.iter().all(|(_, meta)| {
            meta.section_type != crate::models::spec_rules::SectionType::TextSourceUnit
        }));
        assert!(
            crate::validate::rules::section_format::check_section_format(
                &spec_path,
                &refreshed,
                crate::validate::rules::section_format::DEFAULT_LOOKAHEAD,
            )
            .is_empty()
        );

        let parsed = crate::generate::rust_source_unit::parse(&original).unwrap();
        let partitions = partition_rust_source(&original, Some(&parsed));
        assert_eq!(
            partitions
                .iter()
                .map(|partition| partition.content.as_str())
                .collect::<String>(),
            original
        );
        assert!(partitions.iter().all(|partition| {
            partition.content.len() <= crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES
                && crate::generate::apply::encode_source_partition_payload(
                    partition.content.as_bytes(),
                )
                .len()
                    <= crate::generate::apply::RUST_SOURCE_PARTITION_MAX_PAYLOAD_BYTES
        }));
        assert!(partitions.iter().any(|partition| {
            partition.boundary == SourcePartitionBoundary::OversizedAstFallback
        }));

        let second_outcome = CodeStrategy::new()
            .import_explicit_source_file(&source_path, root, &output_dir)
            .unwrap();
        assert_eq!(second_outcome.partition_count, outcome.partition_count);
        assert_eq!(fs::read_to_string(&spec_path).unwrap(), refreshed);

        fs::write(&source_path, "fn corrupted() {}\n").unwrap();
        crate::generate::apply::run_apply_scoped_targets(
            &spec_path,
            root,
            false,
            std::slice::from_ref(&source_path),
        )
        .unwrap();
        assert_eq!(fs::read_to_string(&source_path).unwrap(), original);
    }

    #[test]
    fn explicit_unowned_files_keep_lossless_payload_when_normal_apply_adds_ownership() {
        let cases = [
            (
                "normal",
                "pub fn normal() -> &'static str { \"ok\" }\n".to_string(),
                false,
            ),
            (
                "no-final-newline",
                "pub fn no_final_newline() -> &'static str { \"é\" }".to_string(),
                true,
            ),
            (
                "crlf",
                "pub fn crlf() {\r\n    let _ = \"é\";\r\n}\r\n".to_string(),
                true,
            ),
            (
                "markdown-hazard",
                "const DOC: &str = r#\"\n## Fake\n<!-- type: text-source-unit lang: rust -->\n```rust\nfn fake() {}\n```\n\"#;\n"
                    .to_string(),
                true,
            ),
        ];

        for (name, source, expects_partitions) in cases {
            let temp_dir = TempDir::new().unwrap();
            let root = temp_dir.path();
            let source_path = root.join(format!("apps/demo/src/{name}.rs"));
            let output_dir = root.join("apps/demo/tech-design/specs");
            fs::create_dir_all(source_path.parent().unwrap()).unwrap();
            fs::create_dir_all(&output_dir).unwrap();
            fs::write(&source_path, &source).unwrap();

            let outcome = CodeStrategy::new()
                .import_explicit_source_file(&source_path, root, &output_dir)
                .unwrap();
            assert!(!outcome.requires_hitl, "{name}: {}", outcome.message);
            let spec_path = outcome.spec_path.unwrap();
            let spec = fs::read_to_string(&spec_path).unwrap();
            let decoded = crate::generate::apply::decode_partitioned_source(&spec).unwrap();
            assert_eq!(decoded.is_some(), expects_partitions, "{name}");
            if let Some(decoded) = decoded {
                assert_eq!(decoded.source, source, "{name}");
                assert_eq!(decoded.source_lang, "rust", "{name}");
            }
            let generated_payload = crate::generate::apply::try_generate_source_section_code(
                &spec,
                &normalize_spec_path(spec_path.strip_prefix(root).unwrap()),
                Some(&normalize_spec_path(
                    source_path.strip_prefix(root).unwrap(),
                )),
                root,
            )
            .unwrap();
            assert_eq!(generated_payload, source, "{name}: generator payload");

            fs::write(&source_path, "pub fn corrupted() {}\n").unwrap();
            crate::generate::apply::run_apply_scoped_targets(
                &spec_path,
                root,
                false,
                std::slice::from_ref(&source_path),
            )
            .unwrap();
            let managed = fs::read_to_string(&source_path).unwrap();
            let blocks = crate::generate::marker::parse_codegen_blocks(&managed);
            assert_eq!(blocks.len(), 1, "{name}: one whole-file CODEGEN owner");
            assert!(blocks[0].spec_ref.ends_with("#rust-source-unit"), "{name}");
            assert_eq!(
                blocks[0].content,
                source.lines().collect::<Vec<_>>().join("\n"),
                "{name}: managed wrapper must preserve the selected payload after the marker parser's documented newline normalization"
            );
            assert!(managed.starts_with("// SPEC-MANAGED: "), "{name}");
            assert!(managed.ends_with("// CODEGEN-END\n"), "{name}");
        }
    }

    #[test]
    fn explicit_parse_incomplete_large_file_uses_deterministic_bounded_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let source_path = root.join("apps/demo/src/incomplete.rs");
        let output_dir = root.join("apps/demo/tech-design/specs");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        let mut source = "pub fn incomplete( {\r\n".to_string();
        for idx in 0..4_000 {
            source.push_str(&format!(
                "// é fallback line {idx:04} with ``` and ## fake heading\r\n"
            ));
        }
        source.push_str("// deterministic final line without newline");
        assert!(source.len() > crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES);
        assert!(crate::generate::rust_source_unit::parse(&source).is_err());
        fs::write(&source_path, &source).unwrap();

        let first = CodeStrategy::new()
            .import_explicit_source_file(&source_path, root, &output_dir)
            .unwrap();
        assert!(!first.requires_hitl, "{}", first.message);
        assert_eq!(first.item_count, 0);
        assert!(first.partition_count > 1);
        let spec_path = first.spec_path.unwrap();
        let first_spec = fs::read_to_string(&spec_path).unwrap();
        assert_eq!(
            crate::generate::apply::decode_partitioned_source(&first_spec)
                .unwrap()
                .as_ref()
                .map(|decoded| decoded.source.as_str()),
            Some(source.as_str())
        );
        assert_eq!(
            crate::generate::apply::try_generate_source_section_code(
                &first_spec,
                &normalize_spec_path(spec_path.strip_prefix(root).unwrap()),
                Some(&normalize_spec_path(source_path.strip_prefix(root).unwrap())),
                root,
            )
            .unwrap(),
            source,
            "parse-incomplete fallback must remain lossless through the normal source generator core"
        );
        let partitions = partition_rust_source(&source, None);
        assert!(partitions.iter().all(|partition| {
            partition.boundary == SourcePartitionBoundary::ParseFallback
                && partition.content.len()
                    <= crate::generate::apply::RUST_SOURCE_PARTITION_MAX_BYTES
                && crate::generate::apply::encode_source_partition_payload(
                    partition.content.as_bytes(),
                )
                .len()
                    <= crate::generate::apply::RUST_SOURCE_PARTITION_MAX_PAYLOAD_BYTES
        }));

        let second = CodeStrategy::new()
            .import_explicit_source_file(&source_path, root, &output_dir)
            .unwrap();
        assert_eq!(second.partition_count, first.partition_count);
        assert_eq!(fs::read_to_string(&spec_path).unwrap(), first_spec);

        fs::write(&source_path, "pub fn corrupted() {}\n").unwrap();
        crate::generate::apply::run_apply_scoped_targets(
            &spec_path,
            root,
            false,
            std::slice::from_ref(&source_path),
        )
        .unwrap();
        let managed = fs::read_to_string(&source_path).unwrap();
        let blocks = crate::generate::marker::parse_codegen_blocks(&managed);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].spec_ref.ends_with("#rust-source-unit"));
        assert_eq!(
            blocks[0].content,
            source.lines().collect::<Vec<_>>().join("\n"),
            "managed gen-source must preserve the fallback payload inside its required ownership envelope"
        );
    }

    #[test]
    fn explicit_partial_codegen_owner_returns_hitl_without_mutation() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let source_path = root.join("apps/demo/src/partial.rs");
        let spec_path = root.join("apps/demo/tech-design/semantic/source/partial.md");
        let output_dir = root.join("apps/demo/tech-design/specs");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::create_dir_all(spec_path.parent().unwrap()).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        let spec_ref = "apps/demo/tech-design/semantic/source/partial.md#rust-source-unit";
        let source = format!(
            "fn hand_written() {{}}\n// SPEC-MANAGED: {spec_ref}\n// CODEGEN-BEGIN\nfn generated() {{}}\n// CODEGEN-END\n"
        );
        fs::write(&source_path, source).unwrap();
        let existing = format!(
            "---\nid: partial\nfill_sections: [rust-source-unit, changes]\n---\n\n## Source\n<!-- type: rust-source-unit lang: rust -->\n\n```rust\nfn stale() {{}}\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/demo/src/partial.rs\n    action: modify\n    section: rust-source-unit\n    impl_mode: codegen\n```\n"
        );
        fs::write(&spec_path, &existing).unwrap();

        let outcome = CodeStrategy::new()
            .import_explicit_source_file(&source_path, root, &output_dir)
            .unwrap();

        assert!(outcome.requires_hitl);
        assert!(outcome.message.contains("partial"));
        assert_eq!(fs::read_to_string(&spec_path).unwrap(), existing);
    }

    #[test]
    fn explicit_rust_marker_ambiguity_returns_hitl_without_td_mutation() {
        let cases = [
            (
                "unmatched",
                "// CODEGEN-BEGIN\npub fn selected() {}\n",
            ),
            (
                "parse-incomplete",
                "pub fn outside() {}\n// SPEC-MANAGED: apps/demo/tech-design/specs/selected.md#rust-source-unit\n// CODEGEN-BEGIN\npub fn selected( {\n// CODEGEN-END\n",
            ),
        ];
        for (name, source) in cases {
            let temp_dir = TempDir::new().unwrap();
            let root = temp_dir.path();
            let source_path = root.join("apps/demo/src/selected.rs");
            let output_dir = root.join("apps/demo/tech-design/specs");
            let expected_spec = output_dir.join("apps/demo/src/selected.md");
            fs::create_dir_all(source_path.parent().unwrap()).unwrap();
            fs::create_dir_all(&output_dir).unwrap();
            fs::write(&source_path, source).unwrap();

            let outcome = CodeStrategy::new()
                .import_explicit_source_file(&source_path, root, &output_dir)
                .unwrap();
            assert!(outcome.requires_hitl, "{name}: {}", outcome.message);
            assert!(
                outcome.message.contains("ambiguous"),
                "{name}: {}",
                outcome.message
            );
            assert!(!expected_spec.exists(), "{name}");
            assert_eq!(fs::read_to_string(&source_path).unwrap(), source, "{name}");
        }
    }

    #[test]
    fn existing_owner_invalid_exact_contract_is_hitl_before_refresh() {
        let base = "---\nid: owner\nfill_sections: [rust-source-unit, changes]\n---\n\n## Source\n<!-- type: rust-source-unit lang: rust -->\n\n```rust\npub fn stale() {}\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/demo/src/owned.rs\n    action: modify\n    section: rust-source-unit\n    impl_mode: codegen\n```\n";
        let cases = [
            (
                "create-action",
                base.replace("action: modify", "action: create"),
            ),
            (
                "replaces",
                base.replace(
                    "    impl_mode: codegen",
                    "    replaces: [owned]\n    impl_mode: codegen",
                ),
            ),
            (
                "duplicate-annotation",
                base.replace(
                    "<!-- type: rust-source-unit lang: rust -->",
                    "<!-- type: rust-source-unit lang: rust -->\n<!-- type: rust-source-unit lang: rust -->",
                ),
            ),
            (
                "second-source-entry",
                base.replace(
                    "    impl_mode: codegen\n```\n",
                    "    impl_mode: codegen\n  - path: apps/demo/src/owned.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n```\n",
                ),
            ),
        ];

        for (name, existing) in cases {
            let temp_dir = TempDir::new().unwrap();
            let root = temp_dir.path();
            let source_path = root.join("apps/demo/src/owned.rs");
            let spec_path = root.join("apps/demo/tech-design/semantic/source/owned.md");
            let output_dir = root.join("apps/demo/tech-design/specs");
            fs::create_dir_all(source_path.parent().unwrap()).unwrap();
            fs::create_dir_all(spec_path.parent().unwrap()).unwrap();
            fs::create_dir_all(&output_dir).unwrap();
            let source = "pub fn owned() {}\n";
            fs::write(&source_path, source).unwrap();
            fs::write(&spec_path, &existing).unwrap();

            let outcome = CodeStrategy::new()
                .import_explicit_source_file(&source_path, root, &output_dir)
                .unwrap();
            assert!(outcome.requires_hitl, "{name}: {}", outcome.message);
            assert_eq!(fs::read_to_string(&spec_path).unwrap(), existing, "{name}");
            assert_eq!(fs::read_to_string(&source_path).unwrap(), source, "{name}");
        }
    }

    #[test]
    fn source_unit_candidate_persistence_detects_snapshot_drift_and_no_clobber() {
        let temp_dir = TempDir::new().unwrap();
        let existing = temp_dir.path().join("existing.md");
        fs::write(&existing, "original\n").unwrap();
        let snapshot = fs::read_to_string(&existing).unwrap();
        fs::write(&existing, "concurrent\n").unwrap();
        assert_eq!(
            persist_source_unit_candidate(&existing, "candidate\n", Some(&snapshot)).unwrap(),
            SourceUnitPersistOutcome::ConcurrentDrift
        );
        assert_eq!(fs::read_to_string(&existing).unwrap(), "concurrent\n");

        let raced_new = temp_dir.path().join("new.md");
        fs::write(&raced_new, "other-agent\n").unwrap();
        assert_eq!(
            persist_source_unit_candidate(&raced_new, "candidate\n", None).unwrap(),
            SourceUnitPersistOutcome::ConcurrentDrift
        );
        assert_eq!(fs::read_to_string(&raced_new).unwrap(), "other-agent\n");

        let clean_new = temp_dir.path().join("clean.md");
        assert_eq!(
            persist_source_unit_candidate(&clean_new, "candidate\n", None).unwrap(),
            SourceUnitPersistOutcome::Written
        );
        assert_eq!(fs::read_to_string(clean_new).unwrap(), "candidate\n");
    }

    #[test]
    fn invalid_existing_owner_is_fence_aware_and_fails_closed() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let source_path = root.join("apps/demo/src/raw_doc.rs");
        let spec_path = root.join("apps/demo/tech-design/semantic/source/raw-doc.md");
        let output_dir = root.join("apps/demo/tech-design/specs");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::create_dir_all(spec_path.parent().unwrap()).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        let spec_ref = "apps/demo/tech-design/semantic/source/raw-doc.md#rust-source-unit";
        let source = format!(
            "// SPEC-MANAGED: {spec_ref}\n// CODEGEN-BEGIN\nconst DOC: &str = r#\"\n## Fake\n```rust\nfn fake() {{}}\n```\n\"#;\n// CODEGEN-END\n"
        );
        fs::write(&source_path, &source).unwrap();
        let existing = format!(
            "---\nid: raw-doc\ncapability_refs:\n  - id: keep-me\n    role: primary\n    gap: keep-gap\n    claim: keep-gap\n    coverage: full\nfill_sections: [rust-source-unit, changes]\n---\n\n# Raw Doc\n\n## Source\n<!-- type: rust-source-unit lang: rust -->\n\n````rust\nconst OLD: &str = r#\"\n## Fake\n\"#;\n````\n\n### Source Partition 9999\n<!-- aw-source-partition: index=9999 count=9999 bytes=1 payload_bytes=4 encoding=base64 digest=sha256:bad boundary=ast terminal_newline=false -->\n\n```text\neA==\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/demo/src/raw_doc.rs\n    action: modify\n    section: rust-source-unit\n    impl_mode: codegen\n```\n"
        );
        fs::write(&spec_path, &existing).unwrap();

        let outcome = CodeStrategy::new()
            .import_explicit_source_file(&source_path, root, &output_dir)
            .unwrap();
        assert!(outcome.requires_hitl, "{}", outcome.message);
        assert!(
            outcome
                .message
                .contains("partition controls require exactly one canonical source manifest"),
            "{}",
            outcome.message
        );
        assert_eq!(fs::read_to_string(&spec_path).unwrap(), existing);
        assert_eq!(fs::read_to_string(&source_path).unwrap(), source);
    }

    #[test]
    fn test_can_handle() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("test.rs");
        fs::write(&file, "fn main() {}").unwrap();

        let strategy = CodeStrategy::new();
        assert!(strategy.can_handle(temp_dir.path()));
        assert!(strategy.can_handle(&file));
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
