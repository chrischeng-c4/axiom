// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/reference_context_service_preamble_source.md#source
// CODEGEN-BEGIN
//! Context service - Business logic for structured context artifact creation
//!
//! Each context type (spec, knowledge, codebase) has its own input struct
//! with type-specific validation and markdown rendering. The output is a
//! structured index (what was scanned, what was found, where it lives)
//! rather than a free-form copy of content.

use crate::models::context::{DocRef, FileRef, LensResult, PatternRef, SpecRef};
use crate::Result;
use chrono::Utc;
use std::path::Path;

// ---------------------------------------------------------------------------
// Input structs
// ---------------------------------------------------------------------------
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/reference_context_service.md#schema
// CODEGEN-BEGIN
/// Input for creating a codebase context artifact.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/reference_context_service.md#schema
#[derive(Debug, Clone)]
pub struct CreateCodebaseContextInput {
    /// Change identifier.
    pub change_id: String,
    /// Complexity tier.
    pub complexity: String,
    /// Iteration number.
    pub iteration: u32,
    /// Lens tools invoked.
    pub lens_tools_used: Vec<String>,
    /// Files scanned.
    pub files: Vec<FileRef>,
    /// Lens results.
    pub lens_results: Vec<LensResult>,
    /// Dependency graph entries.
    pub dependency_graph: Vec<String>,
}

/// Unified enum for dispatching to the correct creation path.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/reference_context_service.md#schema

pub enum CreateContextInput {
    Spec(CreateSpecContextInput),
    Knowledge(CreateKnowledgeContextInput),
    Codebase(CreateCodebaseContextInput),
    /// Gap analysis artifacts (free-form markdown).
    Gap {
        change_id: String,
        context_type: String,
        content: String,
    },
}

/// Input for creating a knowledge context artifact.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/reference_context_service.md#schema
#[derive(Debug, Clone)]
pub struct CreateKnowledgeContextInput {
    /// Change identifier.
    pub change_id: String,
    /// Complexity tier.
    pub complexity: String,
    /// Iteration number.
    pub iteration: u32,
    /// Scanned knowledge categories.
    pub scanned_categories: Vec<String>,
    /// Documents found.
    pub docs: Vec<DocRef>,
    /// Patterns identified.
    pub patterns: Vec<PatternRef>,
    /// Pitfalls.
    pub pitfalls: Vec<String>,
}

/// Input for creating a spec context artifact.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/reference_context_service.md#schema
#[derive(Debug, Clone)]
pub struct CreateSpecContextInput {
    /// Change identifier.
    pub change_id: String,
    /// Complexity tier.
    pub complexity: String,
    /// Iteration number.
    pub iteration: u32,
    /// Scanned spec groups.
    pub scanned_groups: Vec<String>,
    /// Specs found.
    pub specs: Vec<SpecRef>,
    /// Dependencies.
    pub dependencies: Vec<String>,
    /// Identified gaps.
    pub gaps: Vec<String>,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/reference_context_service_runtime_source.md#source
// CODEGEN-BEGIN
// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Create a context artifact file with structured input
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/reference_context_service_runtime_source.md#source
pub fn create_context(input: CreateContextInput, project_root: &Path) -> Result<String> {
    match input {
        CreateContextInput::Spec(i) => create_spec_context(i, project_root),
        CreateContextInput::Knowledge(i) => create_knowledge_context(i, project_root),
        CreateContextInput::Codebase(i) => create_codebase_context(i, project_root),
        CreateContextInput::Gap {
            change_id,
            context_type,
            content,
        } => create_gap_context(&change_id, &context_type, &content, project_root),
    }
}

// ---------------------------------------------------------------------------
// Spec context
// ---------------------------------------------------------------------------

fn validate_spec_context(input: &CreateSpecContextInput) -> Result<()> {
    validate_change_id(&input.change_id)?;
    validate_complexity(&input.complexity)?;
    validate_iteration(input.iteration)?;
    if input.scanned_groups.is_empty() {
        anyhow::bail!("scanned_groups must be non-empty (completeness proof)");
    }
    validate_non_empty_strings(&input.scanned_groups, "scanned_groups")?;
    if input.specs.is_empty() {
        anyhow::bail!("specs must be non-empty (at least one spec reference required)");
    }
    for (i, spec) in input.specs.iter().enumerate() {
        if spec.id.is_empty() {
            anyhow::bail!("specs[{}]: must have a non-empty id", i);
        }
        reject_nul(&spec.id, &format!("specs[{}].id", i))?;
        reject_nul(&spec.group, &format!("specs[{}].group", i))?;
        reject_nul(&spec.reason, &format!("specs[{}].reason", i))?;
        if !["high", "medium", "low"].contains(&spec.relevance.as_str()) {
            anyhow::bail!(
                "specs[{}] '{}': invalid relevance '{}' (must be high/medium/low)",
                i,
                spec.id,
                spec.relevance
            );
        }
        for (j, s) in spec.key_sections.iter().enumerate() {
            reject_empty_or_nul(s, &format!("specs[{}].key_sections[{}]", i, j))?;
        }
    }
    validate_non_empty_strings(&input.dependencies, "dependencies")?;
    validate_non_empty_strings(&input.gaps, "gaps")?;
    Ok(())
}

fn render_spec_context(input: &CreateSpecContextInput) -> String {
    let now = Utc::now();
    let mut out = String::new();

    // YAML frontmatter
    out.push_str("---\n");
    out.push_str(&format!("change_id: {}\n", yaml_safe(&input.change_id)));
    out.push_str("type: spec_context\n");
    out.push_str(&format!("created_at: {}\n", now.to_rfc3339()));
    out.push_str(&format!("updated_at: {}\n", now.to_rfc3339()));
    out.push_str(&format!("iteration: {}\n", input.iteration));
    out.push_str(&format!("complexity: {}\n", yaml_safe(&input.complexity)));
    out.push_str("stage: spec\n");
    out.push_str("scanned_groups:\n");
    for g in &input.scanned_groups {
        out.push_str(&format!("  - {}\n", yaml_safe(g)));
    }
    out.push_str("---\n\n");

    // Body
    out.push_str("# Spec Context\n\n");

    out.push_str("## Relevant Specs\n\n");
    for spec in &input.specs {
        out.push_str(&format!("- **{}**", spec.id));
        if !spec.group.is_empty() {
            out.push_str(&format!(" (group: {})", spec.group));
        }
        out.push('\n');
        out.push_str(&format!("  - relevance: {}\n", spec.relevance));
        if !spec.reason.is_empty() {
            out.push_str(&format!("  - reason: {}\n", spec.reason));
        }
        if !spec.key_sections.is_empty() {
            out.push_str(&format!(
                "  - key sections: {}\n",
                spec.key_sections.join(", ")
            ));
        }
    }

    if !input.dependencies.is_empty() {
        out.push_str("\n## Dependencies\n\n");
        for dep in &input.dependencies {
            out.push_str(&format!("- {}\n", dep));
        }
    }

    if !input.gaps.is_empty() {
        out.push_str("\n## Gaps\n\n");
        for gap in &input.gaps {
            out.push_str(&format!("- {}\n", gap));
        }
    }

    out
}

fn create_spec_context(input: CreateSpecContextInput, project_root: &Path) -> Result<String> {
    validate_spec_context(&input)?;
    let content = render_spec_context(&input);
    write_context_file(&input.change_id, "spec_context.md", &content, project_root)
}

// ---------------------------------------------------------------------------
// Knowledge context
// ---------------------------------------------------------------------------

fn validate_knowledge_context(input: &CreateKnowledgeContextInput) -> Result<()> {
    validate_change_id(&input.change_id)?;
    validate_complexity(&input.complexity)?;
    validate_iteration(input.iteration)?;
    if input.scanned_categories.is_empty() {
        anyhow::bail!("scanned_categories must be non-empty (completeness proof)");
    }
    validate_non_empty_strings(&input.scanned_categories, "scanned_categories")?;
    if input.docs.is_empty() {
        anyhow::bail!("docs must be non-empty (at least one document reference required)");
    }
    for (i, doc) in input.docs.iter().enumerate() {
        if doc.path.is_empty() {
            anyhow::bail!("docs[{}]: must have a non-empty path", i);
        }
        validate_safe_path(&doc.path, &format!("docs[{}].path", i))?;
        if doc.summary.is_empty() {
            anyhow::bail!("docs[{}]: must have a non-empty summary", i);
        }
        reject_nul(&doc.summary, &format!("docs[{}].summary", i))?;
        for (j, s) in doc.relevant_sections.iter().enumerate() {
            reject_empty_or_nul(s, &format!("docs[{}].relevant_sections[{}]", i, j))?;
        }
    }
    for (i, p) in input.patterns.iter().enumerate() {
        reject_nul(&p.name, &format!("patterns[{}].name", i))?;
        reject_nul(&p.description, &format!("patterns[{}].description", i))?;
        if p.source.is_empty() {
            anyhow::bail!("patterns[{}]: must have a non-empty source", i);
        }
        validate_safe_path(&p.source, &format!("patterns[{}].source", i))?;
    }
    validate_non_empty_strings(&input.pitfalls, "pitfalls")?;
    Ok(())
}

fn render_knowledge_context(input: &CreateKnowledgeContextInput) -> String {
    let now = Utc::now();
    let mut out = String::new();

    // YAML frontmatter
    out.push_str("---\n");
    out.push_str(&format!("change_id: {}\n", yaml_safe(&input.change_id)));
    out.push_str("type: knowledge_context\n");
    out.push_str(&format!("created_at: {}\n", now.to_rfc3339()));
    out.push_str(&format!("updated_at: {}\n", now.to_rfc3339()));
    out.push_str(&format!("iteration: {}\n", input.iteration));
    out.push_str(&format!("complexity: {}\n", yaml_safe(&input.complexity)));
    out.push_str("stage: knowledge\n");
    out.push_str("scanned_categories:\n");
    for c in &input.scanned_categories {
        out.push_str(&format!("  - {}\n", yaml_safe(c)));
    }
    out.push_str("---\n\n");

    // Body
    out.push_str("# Knowledge Context\n\n");

    out.push_str("## Relevant Documents\n\n");
    for doc in &input.docs {
        out.push_str(&format!("- **{}**\n", doc.path));
        out.push_str(&format!("  - summary: {}\n", doc.summary));
        if !doc.relevant_sections.is_empty() {
            out.push_str(&format!(
                "  - relevant sections: {}\n",
                doc.relevant_sections.join(", ")
            ));
        }
    }

    if !input.patterns.is_empty() {
        out.push_str("\n## Patterns\n\n");
        for p in &input.patterns {
            out.push_str(&format!("- **{}** (source: {})\n", p.name, p.source));
            out.push_str(&format!("  - {}\n", p.description));
        }
    }

    if !input.pitfalls.is_empty() {
        out.push_str("\n## Pitfalls\n\n");
        for pitfall in &input.pitfalls {
            out.push_str(&format!("- {}\n", pitfall));
        }
    }

    out
}

fn create_knowledge_context(
    input: CreateKnowledgeContextInput,
    project_root: &Path,
) -> Result<String> {
    validate_knowledge_context(&input)?;
    let content = render_knowledge_context(&input);
    write_context_file(
        &input.change_id,
        "knowledge_context.md",
        &content,
        project_root,
    )
}

// ---------------------------------------------------------------------------
// Codebase context
// ---------------------------------------------------------------------------

fn validate_codebase_context(input: &CreateCodebaseContextInput) -> Result<()> {
    validate_change_id(&input.change_id)?;
    validate_complexity(&input.complexity)?;
    validate_iteration(input.iteration)?;
    if input.lens_tools_used.is_empty() {
        anyhow::bail!("lens_tools_used must be non-empty (at least one Lens tool required)");
    }
    validate_non_empty_strings(&input.lens_tools_used, "lens_tools_used")?;
    if input.files.is_empty() {
        anyhow::bail!("files must be non-empty (at least one file reference required)");
    }
    for (i, f) in input.files.iter().enumerate() {
        if f.path.is_empty() {
            anyhow::bail!("files[{}]: must have a non-empty path", i);
        }
        validate_safe_path(&f.path, &format!("files[{}].path", i))?;
        reject_nul(&f.role, &format!("files[{}].role", i))?;
        for (j, s) in f.symbols.iter().enumerate() {
            reject_empty_or_nul(s, &format!("files[{}].symbols[{}]", i, j))?;
        }
    }
    for (i, r) in input.lens_results.iter().enumerate() {
        reject_nul(&r.tool, &format!("lens_results[{}].tool", i))?;
        reject_nul(&r.query, &format!("lens_results[{}].query", i))?;
        reject_nul(&r.summary, &format!("lens_results[{}].summary", i))?;
    }
    validate_non_empty_strings(&input.dependency_graph, "dependency_graph")?;
    Ok(())
}

fn render_codebase_context(input: &CreateCodebaseContextInput) -> String {
    let now = Utc::now();
    let mut out = String::new();

    // YAML frontmatter
    out.push_str("---\n");
    out.push_str(&format!("change_id: {}\n", yaml_safe(&input.change_id)));
    out.push_str("type: codebase_context\n");
    out.push_str(&format!("created_at: {}\n", now.to_rfc3339()));
    out.push_str(&format!("updated_at: {}\n", now.to_rfc3339()));
    out.push_str(&format!("iteration: {}\n", input.iteration));
    out.push_str(&format!("complexity: {}\n", yaml_safe(&input.complexity)));
    out.push_str("stage: codebase\n");
    out.push_str("lens_tools_used:\n");
    for t in &input.lens_tools_used {
        out.push_str(&format!("  - {}\n", yaml_safe(t)));
    }
    out.push_str("---\n\n");

    // Body
    out.push_str("# Codebase Context\n\n");

    out.push_str("## Analyzed Files\n\n");
    for f in &input.files {
        out.push_str(&format!("- **{}**", f.path));
        if !f.role.is_empty() {
            out.push_str(&format!(" — {}", f.role));
        }
        out.push('\n');
        if !f.symbols.is_empty() {
            out.push_str(&format!(
                "  - symbols: {}\n",
                f.symbols
                    .iter()
                    .map(|s| format!("`{}`", s))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    if !input.lens_results.is_empty() {
        out.push_str("\n## Lens Results\n\n");
        for r in &input.lens_results {
            out.push_str(&format!("- **{}** (query: `{}`)\n", r.tool, r.query));
            out.push_str(&format!("  - {}\n", r.summary));
        }
    }

    if !input.dependency_graph.is_empty() {
        out.push_str("\n## Dependency Graph\n\n");
        for dep in &input.dependency_graph {
            out.push_str(&format!("- {}\n", dep));
        }
    }

    out
}

fn create_codebase_context(
    input: CreateCodebaseContextInput,
    project_root: &Path,
) -> Result<String> {
    validate_codebase_context(&input)?;
    let content = render_codebase_context(&input);
    write_context_file(
        &input.change_id,
        "codebase_context.md",
        &content,
        project_root,
    )
}

// ---------------------------------------------------------------------------
// Gap context (free-form markdown)
// ---------------------------------------------------------------------------

fn create_gap_context(
    change_id: &str,
    context_type: &str,
    content: &str,
    project_root: &Path,
) -> Result<String> {
    validate_change_id(change_id)?;
    let filename = format!("{}.md", context_type);
    let now = Utc::now();

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("change_id: {}\n", yaml_safe(change_id)));
    out.push_str(&format!("type: {}\n", yaml_safe(context_type)));
    out.push_str(&format!("created_at: {}\n", now.to_rfc3339()));
    out.push_str(&format!("updated_at: {}\n", now.to_rfc3339()));
    out.push_str("---\n\n");
    out.push_str(content);

    write_context_file(change_id, &filename, &out, project_root)
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn validate_change_id(change_id: &str) -> Result<()> {
    if change_id.is_empty() {
        anyhow::bail!("Invalid change_id: must not be empty");
    }
    if change_id.len() > 64 {
        anyhow::bail!("Invalid change_id: must be at most 64 characters");
    }
    if change_id == "." || change_id == ".." {
        anyhow::bail!("Invalid change_id: '.' and '..' are not allowed");
    }
    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!("Invalid change_id: must be lowercase alphanumeric with hyphens only");
    }
    Ok(())
}

fn validate_safe_path(path: &str, field_name: &str) -> Result<()> {
    if path.contains('\0') {
        anyhow::bail!("{}: null bytes are not allowed", field_name);
    }
    if path.starts_with('/') || path.starts_with('\\') {
        anyhow::bail!(
            "{}: absolute paths are not allowed (got '{}')",
            field_name,
            path
        );
    }
    // Reject Windows drive-letter paths like C:\foo or C:/foo
    let bytes = path.as_bytes();
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
    {
        anyhow::bail!(
            "{}: absolute paths are not allowed (got '{}')",
            field_name,
            path
        );
    }
    if path.contains("..") {
        anyhow::bail!(
            "{}: path traversal ('..') is not allowed (got '{}')",
            field_name,
            path
        );
    }
    Ok(())
}

fn validate_non_empty_strings(items: &[String], field_name: &str) -> Result<()> {
    for (i, s) in items.iter().enumerate() {
        if s.is_empty() {
            anyhow::bail!("{}[{}]: must be a non-empty string", field_name, i);
        }
        if s.contains('\0') {
            anyhow::bail!("{}[{}]: null bytes are not allowed", field_name, i);
        }
    }
    Ok(())
}

fn reject_nul(value: &str, field_name: &str) -> Result<()> {
    if value.contains('\0') {
        anyhow::bail!("{}: null bytes are not allowed", field_name);
    }
    Ok(())
}

fn reject_empty_or_nul(value: &str, field_name: &str) -> Result<()> {
    if value.is_empty() {
        anyhow::bail!("{}: must be a non-empty string", field_name);
    }
    if value.contains('\0') {
        anyhow::bail!("{}: null bytes are not allowed", field_name);
    }
    Ok(())
}

/// Quote a YAML value if it contains special characters
fn yaml_safe(s: &str) -> String {
    if s.contains(':')
        || s.contains('#')
        || s.contains('{')
        || s.contains('}')
        || s.contains('[')
        || s.contains(']')
        || s.contains('\'')
        || s.contains('"')
        || s.contains('&')
        || s.contains('*')
        || s.contains('!')
        || s.contains('\n')
        || s.contains('\r')
        || s.starts_with('-')
        || s.starts_with("---")
    {
        format!(
            "\"{}\"",
            s.replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
        )
    } else {
        s.to_string()
    }
}

const VALID_COMPLEXITIES: &[&str] = &["low", "medium", "high", "critical"];

fn validate_complexity(complexity: &str) -> Result<()> {
    if !VALID_COMPLEXITIES.contains(&complexity) {
        anyhow::bail!(
            "Invalid complexity '{}': must be one of low/medium/high/critical",
            complexity
        );
    }
    Ok(())
}

fn validate_iteration(iteration: u32) -> Result<()> {
    if iteration < 1 {
        anyhow::bail!("Invalid iteration {}: must be >= 1", iteration);
    }
    Ok(())
}

fn write_context_file(
    change_id: &str,
    filename: &str,
    content: &str,
    project_root: &Path,
) -> Result<String> {
    let change_dir = project_root.join(".aw/changes").join(change_id);
    if !change_dir.exists() {
        std::fs::create_dir_all(&change_dir)?;
    }
    let file_path = change_dir.join(filename);
    std::fs::write(&file_path, content)?;
    Ok(format!(
        "Context artifact written: .aw/changes/{}/{}",
        change_id, filename
    ))
}

#[cfg(test)]
#[path = "reference_context_service_tests.rs"]
mod tests;
// CODEGEN-END
