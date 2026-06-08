// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_preamble_source.md#source
// CODEGEN-BEGIN
//! Issue section parser for structured issue format.
//!
//! Parses markdown bodies from temp-backed issue working-copy files
//! to extract structured sections that can skip early SDD phases.
//!
//! Required sections: `## Problem`, `## Requirements`, `## Scope`
//! Optional sections: `## Acceptance Criteria`, `## Key Decisions`, `## Reference Context`
//!
//! Write-time validation lives here as well: see [`validate_structured_issue`]
//! and [`ValidationError`]. Validation is tiered by the issue's `state`
//! (draft vs non-draft) per `projects/agentic-workflow/logic/structured-issue.md`.

use crate::issues::IssueState;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser.md#schema
// CODEGEN-BEGIN
use serde::Serialize;

/// A single acceptance criterion.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser.md#schema
#[derive(Debug, Clone)]
pub struct AcceptanceCriterion {
    /// Criterion ID.
    pub id: String,
    /// Criterion text.
    pub text: String,
}

/// A key decision.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser.md#schema
#[derive(Debug, Clone)]
pub struct Decision {
    /// Decision ID.
    pub id: String,
    /// Decision text.
    pub text: String,
}

/// Issue quality result.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct IssueQualityResult {
    /// Whether the issue passed quality checks.
    pub passed: bool,
    /// Errors found during quality check.
    pub errors: Vec<String>,
}

/// Reference context from the issue body.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser.md#schema
#[derive(Debug, Clone, Default)]
pub struct IssueReferenceContext {
    /// Spec references.
    pub specs: Vec<SpecReference>,
    /// Spec plan entries.
    pub spec_plan: Vec<SpecPlanEntry>,
    /// Raw text of the section.
    pub raw: String,
}

/// Scope extracted from the issue body.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser.md#schema
#[derive(Debug, Clone, Default)]
pub struct IssueScope {
    /// In-scope text.
    pub in_scope: String,
    /// Out-of-scope text.
    pub out_of_scope: String,
}

/// A single requirement.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser.md#schema
#[derive(Debug, Clone)]
pub struct Requirement {
    /// Requirement ID.
    pub id: String,
    /// Requirement text.
    pub text: String,
    /// Optional priority.
    pub priority: Option<String>,
}

/// A spec plan entry.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser.md#schema
#[derive(Debug, Clone)]
pub struct SpecPlanEntry {
    /// Spec identifier.
    pub spec_id: String,
    /// Planned action.
    pub action: String,
    /// Main spec reference.
    pub main_spec_ref: String,
    /// Sections to fill.
    pub sections: Vec<String>,
}

/// A spec reference from the reference context.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser.md#schema
#[derive(Debug, Clone)]
pub struct SpecReference {
    /// Spec path.
    pub path: String,
    /// Relevance label.
    pub relevance: String,
    /// Key requirements text.
    pub key_requirements: String,
}

/// A fully parsed structured issue.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser.md#schema
#[derive(Debug, Clone)]
pub struct StructuredIssue {
    /// Problem statement.
    pub problem: String,
    /// Requirements.
    pub requirements: Vec<Requirement>,
    /// Acceptance criteria.
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    /// Scope.
    pub scope: IssueScope,
    /// Key decisions.
    pub key_decisions: Vec<Decision>,
    /// Optional reference context.
    pub reference_context: Option<IssueReferenceContext>,
}

/// A validation failure for an issue body.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    /// Human-readable error message.
    pub error: String,
    /// Stable error code.
    pub code: String,
    /// Missing sections or invalid items.
    pub missing: Vec<String>,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
// CODEGEN-BEGIN
// =============================================================================
// Section-format guardrail (R3h, R6, R8)
// =============================================================================

/// Run [`SectionFormatRule`] against an issue body and return all findings.
///
/// Wired at the `aw wi fill-section --apply` gate so malformed
/// section bodies are rejected before being merged into the worktree
/// (per R6, R8 of the section-format-rule TD spec — hard reject, no
/// warning-only fallback).
///
/// `path_label` is the file label that surfaces in finding output;
/// callers should pass the issue file path or a synthetic label like
/// `"<slug>.md"` so the operator can locate the offending content.
///
/// @spec projects/agentic-workflow/tech-design/core/validate/section-format-rule.md#requirements
pub fn check_issue_body_section_format(
    path_label: &std::path::Path,
    body: &str,
) -> Vec<crate::validate::Finding> {
    crate::validate::rules::section_format::check_section_format(
        path_label,
        body,
        crate::validate::rules::section_format::DEFAULT_LOOKAHEAD,
    )
}

// =============================================================================
// Detection
// =============================================================================

/// Check whether an issue body contains the required structured sections.
///
/// A structured issue must have all three required headers:
/// - `## Problem`
/// - `## Requirements`
/// - `## Scope`
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
pub fn is_structured_issue(body: &str) -> bool {
    let required = ["## Problem", "## Requirements", "## Scope"];
    required.iter().all(|h| body.contains(h))
}

// =============================================================================
// Write-time Validation
// =============================================================================

/// A validation failure for an issue body.
///
/// Serializes to JSON for stderr output by the CLI:
/// `{"error": "...", "code": "VALIDATION_ERROR", "missing": [...]}`.
///
/// See `projects/agentic-workflow/logic/structured-issue.md` (R6) for the wire format.
// REQ: structured-issue#R6
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
impl ValidationError {
    /// Construct a new validation error with the standard `VALIDATION_ERROR` code.
    pub fn new(error: impl Into<String>, missing: Vec<String>) -> Self {
        Self {
            error: error.into(),
            code: "VALIDATION_ERROR".to_string(),
            missing,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
impl std::error::Error for ValidationError {}

/// Validate an issue body against the tiered structured-issue rules.
///
/// Tiered validation per `projects/agentic-workflow/logic/structured-issue.md` R3-R5:
///
/// | State | Required headers           | Content non-empty | R-id format    | Spec path check |
/// |-------|----------------------------|-------------------|----------------|-----------------|
/// | draft | `## Problem`, `## Requirements` | no             | no             | no              |
/// | open  | `## Problem`, `## Requirements` | yes (warn-soft) | yes (`^R\d+:`) | warn-only       |
///
/// `## Reference Context` is optional. If present in non-draft state, spec
/// path validation is *advisory* (warn, not hard error) and is left to the
/// caller — this function only enforces required headers and R-id format.
///
/// # Errors
///
/// Returns [`ValidationError`] when:
/// - Required section headers are missing (any state)
/// - In non-draft state, any list item in `## Requirements` does not match
///   the `^R\d+:` pattern
// REQ: structured-issue#R3
// REQ: structured-issue#R4
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
pub fn validate_structured_issue(body: &str, state: IssueState) -> Result<(), ValidationError> {
    // ---- Check required section headers (always required) ----
    // REQ: structured-issue#R3
    let required = ["## Problem", "## Requirements"];
    let missing: Vec<String> = required
        .iter()
        .filter(|h| !body.contains(*h))
        .map(|h| (*h).to_string())
        .collect();

    if !missing.is_empty() {
        let error = if missing.len() == 1 {
            format!("missing required section: {}", missing[0])
        } else {
            format!("missing required sections: {}", missing.join(", "))
        };
        return Err(ValidationError::new(error, missing));
    }

    // ---- Draft state stops here: headers present, content can be empty ----
    if state == IssueState::Draft {
        return Ok(());
    }

    // ---- Non-draft (open/closed): enforce R-id format on Requirements items ----
    // REQ: structured-issue#R4
    let sections = split_sections(body);
    if let Some(req_section) = sections.get("Requirements") {
        let mut bad_items: Vec<String> = Vec::new();
        for line in req_section.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("- ") && !trimmed.starts_with("* ") {
                continue;
            }
            let content = trimmed.trim_start_matches("- ").trim_start_matches("* ");
            if !is_valid_rid_item(content) {
                bad_items.push(content.to_string());
            }
        }

        if !bad_items.is_empty() {
            // Match the example error from the spec for the first bad item.
            let error = format!(
                "requirement item does not match R-id format (^R\\d+:): '{}'",
                bad_items[0]
            );
            return Err(ValidationError::new(error, Vec::new()));
        }
    }

    Ok(())
}

/// Check whether a Requirements list item matches the `^R\d+:` pattern.
///
/// Accepted forms (after the leading `- ` is stripped):
/// - `R1: text`
/// - `**R1**: text`
/// - `**R1** (priority): text`
fn is_valid_rid_item(content: &str) -> bool {
    // Bold form: **R1**: ... or **R1** (high): ...
    if let Some(rest) = content.strip_prefix("**") {
        if let Some(end) = rest.find("**") {
            let id_str = &rest[..end];
            if id_starts_with_rid(id_str) {
                return true;
            }
        }
        return false;
    }

    // Plain form: R1: text
    id_prefix_matches_rid(content)
}

/// Check if a string is exactly `R` followed by one or more digits.
fn id_starts_with_rid(id: &str) -> bool {
    if let Some(num) = id.strip_prefix('R') {
        !num.is_empty() && num.chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

// ─── CRR Quality Validation (R3, R4) ────────────────────────────────────────

/// Validate issue quality for the CRR draft→open gate.
///
/// Checks:
/// 1. Required sections present (Problem, Requirements, Scope)
/// 2. R-id format in Requirements
/// 3. Out of Scope sub-heading present and non-empty
/// 4. Spec Plan present in Reference Context
/// 5. No TBD/TODO/maybe in requirement text
// REQ: R4 — score-review checks issue quality
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
pub fn validate_issue_quality(body: &str) -> IssueQualityResult {
    let mut errors = Vec::new();

    // 1. Required headers
    for header in &["## Problem", "## Requirements", "## Scope"] {
        if !body.contains(header) {
            errors.push(format!("missing required section: {}", header));
        }
    }

    let sections = split_sections(body);

    // 2. R-id format + 5. ambiguity check in Requirements
    if let Some(req_section) = sections.get("Requirements") {
        let mut has_items = false;
        for line in req_section.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("- ") && !trimmed.starts_with("* ") {
                continue;
            }
            has_items = true;
            let content = trimmed.trim_start_matches("- ").trim_start_matches("* ");
            if !is_valid_rid_item(content) {
                let preview: String = content.chars().take(60).collect();
                errors.push(format!("requirement missing R-id format: '{}'", preview));
            }
            let lower = content.to_ascii_lowercase();
            for ambiguous in &["tbd", "todo", "maybe", "unclear", "uncertain"] {
                if lower.contains(ambiguous) {
                    let preview: String = content.chars().take(60).collect();
                    errors.push(format!(
                        "ambiguous requirement contains '{}': '{}'",
                        ambiguous, preview
                    ));
                }
            }
        }
        if !has_items {
            errors.push("## Requirements section is empty".to_string());
        }
    }

    // 3. Out of Scope sub-heading
    if let Some(scope_section) = sections.get("Scope") {
        let scope_lower = scope_section.to_ascii_lowercase();
        let has_out_of_scope =
            scope_lower.contains("### out of scope") || scope_lower.contains("### out-of-scope");
        if !has_out_of_scope {
            errors.push("## Scope missing '### Out of Scope' sub-section".to_string());
        }
    }

    // 4. Spec Plan in Reference Context
    if let Some(ref_ctx) = sections.get("Reference Context") {
        if !ref_ctx.contains("### Spec Plan") {
            errors.push("## Reference Context missing '### Spec Plan' table".to_string());
        }
    } else {
        errors.push("missing ## Reference Context section with Spec Plan".to_string());
    }

    IssueQualityResult {
        passed: errors.is_empty(),
        errors,
    }
}

/// Check if a string starts with `R\d+:`.
fn id_prefix_matches_rid(content: &str) -> bool {
    if let Some(rest) = content.strip_prefix('R') {
        let colon = match rest.find(':') {
            Some(c) => c,
            None => return false,
        };
        let num = &rest[..colon];
        !num.is_empty() && num.chars().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}

// =============================================================================
// Parsing
// =============================================================================

/// Parse a structured issue body into a `StructuredIssue`.
///
/// Returns `None` if the body does not contain all required sections.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
pub fn parse_structured_issue(body: &str) -> Option<StructuredIssue> {
    if !is_structured_issue(body) {
        return None;
    }

    let sections = split_sections(body);

    let problem = sections.get("Problem")?.trim().to_string();
    let requirements_text = sections.get("Requirements")?;
    let scope_text = sections.get("Scope")?;

    let requirements = parse_requirements(requirements_text);
    let scope = parse_scope(scope_text);

    let acceptance_criteria = sections
        .get("Acceptance Criteria")
        .map(|t| parse_acceptance_criteria(t))
        .unwrap_or_default();

    let key_decisions = sections
        .get("Key Decisions")
        .map(|t| parse_decisions(t))
        .unwrap_or_default();

    let reference_context = sections
        .get("Reference Context")
        .map(|t| parse_reference_context(t));

    Some(StructuredIssue {
        problem,
        requirements,
        acceptance_criteria,
        scope,
        key_decisions,
        reference_context,
    })
}

/// Split markdown body into sections by `## Header` markers.
///
/// Returns a map of header name -> section body text.
fn split_sections(body: &str) -> std::collections::HashMap<String, String> {
    let mut sections = std::collections::HashMap::new();
    let mut current_header: Option<String> = None;
    let mut current_body = String::new();

    for line in body.lines() {
        if let Some(header) = line.strip_prefix("## ") {
            // Save previous section
            if let Some(prev_header) = current_header.take() {
                sections.insert(prev_header, current_body.clone());
            }
            current_header = Some(header.trim().to_string());
            current_body.clear();
        } else if current_header.is_some() {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }

    // Save last section
    if let Some(header) = current_header {
        sections.insert(header, current_body);
    }

    sections
}

/// Parse requirements from section text.
///
/// Matches patterns like:
/// - `- **R1**: description`
/// - `- R1: description`
/// - `- **R1** (high): description`
fn parse_requirements(text: &str) -> Vec<Requirement> {
    let mut reqs = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("- ") && !trimmed.starts_with("* ") {
            continue;
        }
        let content = trimmed.trim_start_matches("- ").trim_start_matches("* ");

        // Try pattern: **R{n}**: text or **R{n}** (priority): text
        if let Some(req) = try_parse_id_item(content, "R") {
            reqs.push(Requirement {
                id: req.0,
                text: req.2,
                priority: req.1,
            });
        }
    }

    reqs
}

/// Parse acceptance criteria from section text.
///
/// Matches patterns like:
/// - `- **AC1**: description`
/// - `- AC1: description`
fn parse_acceptance_criteria(text: &str) -> Vec<AcceptanceCriterion> {
    let mut criteria = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("- ") && !trimmed.starts_with("* ") {
            continue;
        }
        let content = trimmed.trim_start_matches("- ").trim_start_matches("* ");

        if let Some(item) = try_parse_id_item(content, "AC") {
            criteria.push(AcceptanceCriterion {
                id: item.0,
                text: item.2,
            });
        }
    }

    criteria
}

/// Parse key decisions from section text.
///
/// Matches patterns like:
/// - `- **D1**: description`
/// - `- D1: description`
fn parse_decisions(text: &str) -> Vec<Decision> {
    let mut decisions = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("- ") && !trimmed.starts_with("* ") {
            continue;
        }
        let content = trimmed.trim_start_matches("- ").trim_start_matches("* ");

        if let Some(item) = try_parse_id_item(content, "D") {
            decisions.push(Decision {
                id: item.0,
                text: item.2,
            });
        }
    }

    decisions
}

/// Parse scope section into in-scope and out-of-scope parts.
///
/// Looks for `### In Scope` and `### Out of Scope` sub-headers.
/// Falls back to treating the whole section as in_scope if no sub-headers found.
fn parse_scope(text: &str) -> IssueScope {
    let mut in_scope = String::new();
    let mut out_of_scope = String::new();
    let mut current_sub: Option<&str> = None;

    for line in text.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("### in scope") || lower.starts_with("### in-scope") {
            current_sub = Some("in");
            continue;
        }
        if lower.starts_with("### out of scope")
            || lower.starts_with("### out-of-scope")
            || lower.starts_with("### not in scope")
        {
            current_sub = Some("out");
            continue;
        }
        // Skip other ### sub-headers
        if trimmed.starts_with("### ") {
            current_sub = None;
            continue;
        }

        match current_sub {
            Some("in") => {
                in_scope.push_str(line);
                in_scope.push('\n');
            }
            Some("out") => {
                out_of_scope.push_str(line);
                out_of_scope.push('\n');
            }
            _ => {
                // No sub-header yet — treat as in_scope by default
                in_scope.push_str(line);
                in_scope.push('\n');
            }
        }
    }

    IssueScope {
        in_scope: in_scope.trim().to_string(),
        out_of_scope: out_of_scope.trim().to_string(),
    }
}

/// Parse reference context section.
///
/// This is a best-effort parse — the section may contain free-form text,
/// spec tables, or structured sub-sections.
fn parse_reference_context(text: &str) -> IssueReferenceContext {
    IssueReferenceContext {
        specs: Vec::new(),
        spec_plan: Vec::new(),
        raw: text.trim().to_string(),
    }
}

/// Try to parse an ID-prefixed list item.
///
/// Supports patterns:
/// - `**{prefix}{n}**: text`
/// - `**{prefix}{n}** (priority): text`
/// - `{prefix}{n}: text`
///
/// Returns (id, optional_priority, text) on success.
fn try_parse_id_item(content: &str, prefix: &str) -> Option<(String, Option<String>, String)> {
    // Bold patterns: **R1**: text  or  **R1** (high): text
    if content.starts_with("**") {
        let after_first_stars = &content[2..];
        let second_stars = after_first_stars.find("**")?;
        let id_str = &after_first_stars[..second_stars];

        if !id_str.starts_with(prefix) {
            return None;
        }
        let num_part = &id_str[prefix.len()..];
        if num_part.is_empty() || !num_part.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }

        let after = after_first_stars[second_stars + 2..].trim();

        // Pattern: **R1**: text (bold with immediate colon)
        if let Some(text) = after.strip_prefix(':') {
            return Some((id_str.to_string(), None, text.trim().to_string()));
        }

        // Pattern: **R1** (priority): text
        if let Some(paren_start) = after.strip_prefix('(') {
            if let Some(paren_end) = paren_start.find(')') {
                let priority = paren_start[..paren_end].trim().to_string();
                let text = paren_start[paren_end + 1..]
                    .trim_start_matches(':')
                    .trim()
                    .to_string();
                return Some((id_str.to_string(), Some(priority), text));
            }
        }

        return None;
    }

    // Plain pattern: R1: text (no bold)
    if content.starts_with(prefix) {
        let after_prefix = &content[prefix.len()..];
        let colon_pos = after_prefix.find(':')?;
        let num_part = &after_prefix[..colon_pos];
        if num_part.is_empty() || !num_part.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let id = format!("{}{}", prefix, num_part);
        let text = after_prefix[colon_pos + 1..].trim().to_string();
        return Some((id, None, text));
    }

    None
}

// =============================================================================
// Issue Slug Extraction
// =============================================================================

/// Extract an issue slug reference from a change description.
///
/// Looks for patterns like:
/// - `issue:some-slug`
/// - `issue: some-slug`
/// - `#1234` (numeric issue reference — not a slug, returns None)
///
/// Returns the slug if found.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
pub fn extract_issue_slug(description: &str) -> Option<String> {
    // Pattern: issue:{slug} or issue: {slug}
    for word in description.split_whitespace() {
        if let Some(slug) = word.strip_prefix("issue:") {
            let slug = slug.trim();
            if !slug.is_empty() {
                return Some(slug.to_string());
            }
        }
    }
    None
}

/// Resolve an issue slug to its local filename stem, trying multiple strategies.
///
/// Tries in order:
/// 1. Each entry in `issues` (if provided) — as either `#<num>` (look up by `github_id`
///    frontmatter field) or as a bare slug (filename stem).
/// 2. `issue:<slug>` literal in `description` (legacy path).
/// 3. `#<num>` in `description` — look up by `github_id`.
///
/// Returns the first strategy that resolves to an existing temp issue working copy.
///
/// This is the correct entry point for any code that wants "find the local issue file
/// for this user input". `extract_issue_slug` only handles strategy 2 and is kept for
/// legacy call sites that explicitly expect that behavior.
///
/// # Bug fix: `bug-init-change-try-structured-issue-skip-silently-sho`
///
/// Before this helper existed, `init_change::try_structured_issue_skip` called
/// `extract_issue_slug(description)` directly, which only recognized the `issue:<slug>`
/// literal. User inputs like `"Py3.12 conformance #756"` or `--issue "#756"` silently
/// skipped the structured-issue path and left the change in a broken state.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
pub fn resolve_issue_slug(
    project_root: &std::path::Path,
    description: &str,
    issues: Option<&[String]>,
) -> Option<String> {
    // Strategy 1: explicit issues list
    if let Some(refs) = issues {
        for reference in refs {
            if let Some(slug) = resolve_issue_ref(project_root, reference) {
                return Some(slug);
            }
        }
    }

    // Strategy 2: issue:<slug> literal in description (legacy)
    if let Some(slug) = extract_issue_slug(description) {
        // Verify the file actually exists before returning
        if load_issue_body(project_root, &slug).is_some() {
            return Some(slug);
        }
    }

    // Strategy 3: #<num> in description
    for word in description.split_whitespace() {
        let cleaned = word.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '#');
        if let Some(num_str) = cleaned.strip_prefix('#') {
            if num_str.parse::<u64>().is_ok() {
                if let Some(slug) = find_slug_by_github_id(project_root, num_str) {
                    return Some(slug);
                }
            }
        }
    }

    None
}

/// Resolve a single reference string (either `#<num>`, UUID prefix, or a bare slug) to a local slug.
fn resolve_issue_ref(project_root: &std::path::Path, reference: &str) -> Option<String> {
    let trimmed = reference.trim();
    // Try `#<num>` form
    if let Some(num_str) = trimmed.strip_prefix('#') {
        if num_str.parse::<u64>().is_ok() {
            return find_slug_by_github_id(project_root, num_str);
        }
    }
    // Try as bare slug (filename stem) first — cheapest check.
    if load_issue_body(project_root, trimmed).is_some() {
        return Some(trimmed.to_string());
    }
    // Try UUID prefix — see issue-centric-workflow.md §change_id Resolution.
    // Ambiguous matches return Err; treat as "not resolved" at this layer
    // (init_change's explicit resolver surfaces the ambiguity error).
    if looks_like_uuid_prefix(trimmed) {
        if let Ok(Some(slug)) = find_slug_by_uuid_prefix(project_root, trimmed) {
            return Some(slug);
        }
    }
    None
}

/// Search the temp issue working copy for an issue file whose YAML
/// frontmatter `id:` field starts with `prefix` (hex UUID).
///
/// Returns `Ok(Some(slug))` on unique match, `Ok(None)` on no match,
/// `Err` if the prefix matches multiple issues (ambiguous).
///
/// Callers use this to let users pass short UUID prefixes (e.g. `45ac7e9e`)
/// instead of long slug filenames. Requires at least 4 hex chars to reduce
/// accidental slug/uuid confusion.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
pub fn find_slug_by_uuid_prefix(
    project_root: &std::path::Path,
    prefix: &str,
) -> anyhow::Result<Option<String>> {
    let issues_dir = crate::shared::workspace::issues_path(project_root);
    let prefix_lower = prefix.to_ascii_lowercase();
    let mut matches: Vec<String> = Vec::new();

    for subdir in &["open", "closed"] {
        let dir = issues_dir.join(subdir);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let frontmatter_end = content
                .trim_start()
                .find("\n---")
                .map(|n| n + 4)
                .unwrap_or(content.len().min(2048));
            let frontmatter = &content[..frontmatter_end.min(content.len())];
            for line in frontmatter.lines() {
                let trimmed = line.trim();
                let Some(rest) = trimmed.strip_prefix("id:") else {
                    continue;
                };
                let uuid = rest
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_ascii_lowercase();
                if uuid.starts_with(&prefix_lower) {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        matches.push(stem.to_string());
                    }
                }
                break;
            }
        }
    }

    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(matches.remove(0))),
        _ => anyhow::bail!(
            "UUID prefix '{}' is ambiguous — matches {} issues: {}",
            prefix,
            matches.len(),
            matches.join(", ")
        ),
    }
}

/// Returns `true` if `s` looks like a short UUID prefix (4–36 hex chars,
/// lowercase hex + optional hyphens). Used to decide whether to attempt
/// UUID-prefix resolution before treating the input as a bare slug.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
pub fn looks_like_uuid_prefix(s: &str) -> bool {
    let n = s.len();
    if !(4..=36).contains(&n) {
        return false;
    }
    s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// Search the temp issue working copy for an issue file whose YAML
/// frontmatter contains `github_id: <num>`, returning its filename stem (slug).
fn find_slug_by_github_id(project_root: &std::path::Path, num_str: &str) -> Option<String> {
    let issues_dir = crate::shared::workspace::issues_path(project_root);
    let target_line = format!("github_id: {}", num_str);

    for subdir in &["open", "closed"] {
        let dir = issues_dir.join(subdir);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            // Only match inside frontmatter (first `---...---` block).
            // Simple substring check is sufficient — frontmatter is the first thing in the file.
            let frontmatter_end = content
                .trim_start()
                .find("\n---")
                .map(|n| n + 4)
                .unwrap_or(content.len().min(2048));
            let frontmatter = &content[..frontmatter_end.min(content.len())];
            if frontmatter.lines().any(|l| l.trim() == target_line) {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    return Some(stem.to_string());
                }
            }
        }
    }
    None
}

/// Load an issue title from the temp issue working copy by slug.
///
/// Reads the YAML frontmatter `title:` field. Returns `None` if the issue
/// file doesn't exist or has no title.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
pub fn load_issue_title(project_root: &std::path::Path, slug: &str) -> Option<String> {
    let issues_dir = crate::shared::workspace::issues_path(project_root);
    let filename = format!("{}.md", slug);

    for subdir in &["open", "closed"] {
        let path = issues_dir.join(subdir).join(&filename);
        if let Ok(content) = std::fs::read_to_string(&path) {
            let trimmed = content.trim_start();
            if !trimmed.starts_with("---") {
                continue;
            }
            let after_first = &trimmed[3..];
            let end = after_first.find("\n---").unwrap_or(after_first.len());
            let frontmatter = &after_first[..end];
            for line in frontmatter.lines() {
                if let Some(title) = line.strip_prefix("title:") {
                    let t = title.trim().trim_matches('"').trim_matches('\'');
                    if !t.is_empty() {
                        return Some(t.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Load an issue body from the temp issue working copy by slug.
///
/// Searches both `open/` and `closed/` subdirectories.
/// Returns (body, slug) on success.
pub fn load_issue_body(project_root: &std::path::Path, slug: &str) -> Option<String> {
    let issues_dir = crate::shared::workspace::issues_path(project_root);
    let filename = format!("{}.md", slug);

    for subdir in &["open", "closed"] {
        let path = issues_dir.join(subdir).join(&filename);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Strip YAML frontmatter
                let body = strip_frontmatter(&content);
                return Some(body);
            }
        }
    }
    None
}

/// Strip YAML frontmatter (--- ... ---) from a markdown string.
fn strip_frontmatter(content: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return content.to_string();
    }
    // Find the closing ---
    let after_first = &trimmed[3..];
    if let Some(end) = after_first.find("\n---") {
        let body_start = end + 4; // skip "\n---"
        after_first[body_start..]
            .trim_start_matches('\n')
            .to_string()
    } else {
        content.to_string()
    }
}

// =============================================================================
// Artifact Generation
// =============================================================================

/// Generate `requirements.md` content from a structured issue.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
pub fn generate_requirements_md(
    change_id: &str,
    group_id: &str,
    structured: &StructuredIssue,
) -> String {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut content = format!(
        "---\nchange: {}\ngroup: {}\ndate: {}\nsource: structured-issue\n---\n\n# Requirements\n\n",
        change_id, group_id, today
    );

    content.push_str("## Problem\n\n");
    content.push_str(&structured.problem);
    content.push_str("\n\n## Requirements\n\n");

    for req in &structured.requirements {
        if let Some(ref priority) = req.priority {
            content.push_str(&format!("- **{}** ({}): {}\n", req.id, priority, req.text));
        } else {
            content.push_str(&format!("- **{}**: {}\n", req.id, req.text));
        }
    }

    if !structured.acceptance_criteria.is_empty() {
        content.push_str("\n## Acceptance Criteria\n\n");
        for ac in &structured.acceptance_criteria {
            content.push_str(&format!("- **{}**: {}\n", ac.id, ac.text));
        }
    }

    content
}

/// Generate `pre_clarifications.md` content from key decisions.
///
/// Key decisions from the structured issue are treated as pre-answered
/// clarification questions.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
pub fn generate_pre_clarifications_md(
    change_id: &str,
    group_id: &str,
    decisions: &[Decision],
) -> String {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let status = if decisions.is_empty() {
        "skipped"
    } else {
        "answered"
    };

    let mut content = format!(
        "---\nchange: {}\ngroup: {}\ndate: {}\nstatus: {}\nsource: structured-issue\n---\n\n# Pre-Clarifications\n\n",
        change_id, group_id, today, status
    );

    if decisions.is_empty() {
        content.push_str(
            "No pre-clarification questions needed. Key decisions were provided in the structured issue.\n",
        );
    } else {
        for (i, d) in decisions.iter().enumerate() {
            content.push_str(&format!("### Q{}: Decision {}\n", i + 1, d.id));
            content.push_str(&format!(
                "- **Question**: What is the decision for {}?\n",
                d.id
            ));
            content.push_str(&format!("- **Answer**: {}\n\n", d.text));
        }
    }

    content
}

/// Generate `post_clarifications.md` content from scope and acceptance criteria.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
pub fn generate_post_clarifications_md(
    change_id: &str,
    group_id: &str,
    scope: &IssueScope,
    acceptance_criteria: &[AcceptanceCriterion],
) -> String {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut content = format!(
        "---\nchange: {}\ngroup: {}\ndate: {}\nstatus: skipped\nsource: structured-issue\n---\n\n# Post-Clarifications\n\n",
        change_id, group_id, today
    );

    content.push_str("## Scope Summary\n\n");
    content.push_str("### Problem\n");
    content.push_str("-> See requirements.md\n\n");

    content.push_str("### Success Criteria\n");
    if acceptance_criteria.is_empty() {
        content.push_str("-> See requirements.md\n\n");
    } else {
        for ac in acceptance_criteria {
            content.push_str(&format!("- **{}**: {}\n", ac.id, ac.text));
        }
        content.push('\n');
    }

    content.push_str("### Boundary\n");
    if !scope.in_scope.is_empty() {
        content.push_str(&format!("- **In scope**: {}\n", scope.in_scope));
    } else {
        content.push_str("- **In scope**: See requirements.md\n");
    }
    if !scope.out_of_scope.is_empty() {
        content.push_str(&format!("- **Out of scope**: {}\n", scope.out_of_scope));
    }
    content.push('\n');

    content
}

/// Generate `reference_context.md` content from issue reference context.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/issue_parser_runtime_source.md#source
pub fn generate_reference_context_md(
    change_id: &str,
    group_id: &str,
    ref_ctx: &IssueReferenceContext,
) -> String {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut content = format!(
        "---\nchange: {}\ngroup: {}\ndate: {}\nsource: structured-issue\n---\n\n# Reference Context\n\n",
        change_id, group_id, today
    );

    if !ref_ctx.raw.is_empty() {
        content.push_str(&ref_ctx.raw);
        content.push('\n');
    } else {
        content.push_str("Reference context was not provided in the structured issue.\n");
    }

    content
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const STRUCTURED_ISSUE_BODY: &str = r#"
Part of #1159 - formal notation for non-code artifact kinds.

## Problem

SDD phases 2-3 and 7 re-derive information that a well-structured issue should already contain.
This adds ~6 mainthread round-trips per change.

## Requirements

- **R1**: Parse markdown body by splitting on `## ` headers
- **R2** (high): Extract R1/R2/AC1/D1 patterns from list items
- **R3**: Backward compat — unstructured issues fall back to current flow

## Acceptance Criteria

- **AC1**: Structured issues skip to PostClarificationsCreated
- **AC2**: Unstructured issues proceed through normal flow

## Scope

### In Scope
- Issue section parser
- init_change update for phase skip

### Out of Scope
- `aw wi enrich` CLI command
- SKILL.md update

## Key Decisions

- **D1**: No new state machine states — skip by advancing phase directly
- **D2**: Reference Context is agent-filled via `aw wi enrich`, not manual

## Reference Context

See `projects/agentic-workflow/tech-design/core/` for state machine and workflow specs.
"#;

    // REQ: REQ-001 - is_structured_issue detection
    #[test]
    fn test_is_structured_issue_positive() {
        assert!(is_structured_issue(STRUCTURED_ISSUE_BODY));
    }

    // REQ: REQ-001 - is_structured_issue detection (negative)
    #[test]
    fn test_is_structured_issue_negative_missing_section() {
        let body = "## Problem\nSome problem\n\n## Requirements\n- R1: do thing\n";
        assert!(!is_structured_issue(body));
    }

    #[test]
    fn test_is_structured_issue_empty() {
        assert!(!is_structured_issue(""));
    }

    #[test]
    fn test_is_structured_issue_plain_text() {
        assert!(!is_structured_issue(
            "Just a regular issue body without headers."
        ));
    }

    // REQ: REQ-002 - parse_structured_issue full parse
    #[test]
    fn test_parse_structured_issue_full() {
        let result = parse_structured_issue(STRUCTURED_ISSUE_BODY).unwrap();

        // Problem
        assert!(result.problem.contains("SDD phases 2-3 and 7"));

        // Requirements
        assert_eq!(result.requirements.len(), 3);
        assert_eq!(result.requirements[0].id, "R1");
        assert!(result.requirements[0].text.contains("Parse markdown body"));
        assert!(result.requirements[0].priority.is_none());
        assert_eq!(result.requirements[1].id, "R2");
        assert_eq!(result.requirements[1].priority.as_deref(), Some("high"));
        assert_eq!(result.requirements[2].id, "R3");

        // Acceptance Criteria
        assert_eq!(result.acceptance_criteria.len(), 2);
        assert_eq!(result.acceptance_criteria[0].id, "AC1");
        assert!(result.acceptance_criteria[0]
            .text
            .contains("PostClarificationsCreated"));
        assert_eq!(result.acceptance_criteria[1].id, "AC2");

        // Scope
        assert!(result.scope.in_scope.contains("Issue section parser"));
        assert!(result.scope.out_of_scope.contains("aw wi enrich"));

        // Key Decisions
        assert_eq!(result.key_decisions.len(), 2);
        assert_eq!(result.key_decisions[0].id, "D1");
        assert!(result.key_decisions[0]
            .text
            .contains("No new state machine"));
        assert_eq!(result.key_decisions[1].id, "D2");

        // Reference Context
        assert!(result.reference_context.is_some());
        let ref_ctx = result.reference_context.unwrap();
        assert!(ref_ctx
            .raw
            .contains("projects/agentic-workflow/tech-design/core/"));
    }

    // REQ: REQ-002 - parse returns None for unstructured
    #[test]
    fn test_parse_structured_issue_returns_none_for_unstructured() {
        let body = "Just a plain issue without structure.";
        assert!(parse_structured_issue(body).is_none());
    }

    // REQ: REQ-003 - parse with only required sections
    #[test]
    fn test_parse_structured_issue_minimal() {
        let body = r#"
## Problem

Something is broken.

## Requirements

- **R1**: Fix the thing

## Scope

Fix only the thing, nothing else.
"#;
        let result = parse_structured_issue(body).unwrap();
        assert!(result.problem.contains("Something is broken"));
        assert_eq!(result.requirements.len(), 1);
        assert!(result.acceptance_criteria.is_empty());
        assert!(result.key_decisions.is_empty());
        assert!(result.reference_context.is_none());
        // Scope without sub-headers falls back to in_scope
        assert!(result.scope.in_scope.contains("Fix only the thing"));
    }

    // REQ: REQ-004 - requirement parsing patterns
    #[test]
    fn test_parse_requirements_plain_id() {
        let text = "- R1: do something\n- R2: do another\n";
        let reqs = parse_requirements(text);
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[0].id, "R1");
        assert_eq!(reqs[0].text, "do something");
        assert_eq!(reqs[1].id, "R2");
    }

    #[test]
    fn test_parse_requirements_bold_id() {
        let text = "- **R1**: bold requirement\n";
        let reqs = parse_requirements(text);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].id, "R1");
        assert_eq!(reqs[0].text, "bold requirement");
    }

    #[test]
    fn test_parse_requirements_with_priority() {
        let text = "- **R1** (high): important thing\n- **R2** (low): minor thing\n";
        let reqs = parse_requirements(text);
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[0].id, "R1");
        assert_eq!(reqs[0].priority.as_deref(), Some("high"));
        assert_eq!(reqs[0].text, "important thing");
        assert_eq!(reqs[1].priority.as_deref(), Some("low"));
    }

    #[test]
    fn test_parse_requirements_ignores_non_list_lines() {
        let text = "Some intro text\n- **R1**: actual requirement\nMore text\n";
        let reqs = parse_requirements(text);
        assert_eq!(reqs.len(), 1);
    }

    // REQ: REQ-005 - acceptance criteria parsing
    #[test]
    fn test_parse_acceptance_criteria() {
        let text = "- **AC1**: first criterion\n- **AC2**: second criterion\n";
        let acs = parse_acceptance_criteria(text);
        assert_eq!(acs.len(), 2);
        assert_eq!(acs[0].id, "AC1");
        assert_eq!(acs[1].id, "AC2");
    }

    // REQ: REQ-006 - decision parsing
    #[test]
    fn test_parse_decisions() {
        let text = "- **D1**: Use REST\n- **D2**: No new states\n";
        let decs = parse_decisions(text);
        assert_eq!(decs.len(), 2);
        assert_eq!(decs[0].id, "D1");
        assert_eq!(decs[0].text, "Use REST");
    }

    // REQ: REQ-007 - scope parsing with sub-headers
    #[test]
    fn test_parse_scope_with_subheaders() {
        let text = r#"
### In Scope
- Feature A
- Feature B

### Out of Scope
- Feature C
"#;
        let scope = parse_scope(text);
        assert!(scope.in_scope.contains("Feature A"));
        assert!(scope.in_scope.contains("Feature B"));
        assert!(scope.out_of_scope.contains("Feature C"));
    }

    // REQ: REQ-008 - extract_issue_slug
    #[test]
    fn test_extract_issue_slug() {
        assert_eq!(
            extract_issue_slug("Implement issue:sdd-structured-issue"),
            Some("sdd-structured-issue".to_string())
        );
        assert_eq!(extract_issue_slug("No issue ref here"), None);
        assert_eq!(extract_issue_slug("issue:"), None);
    }

    // REQ: REQ-009 - strip_frontmatter
    #[test]
    fn test_strip_frontmatter() {
        let content = "---\ntype: enhancement\ntitle: Test\n---\n\n## Problem\n\nBody here.\n";
        let body = strip_frontmatter(content);
        assert!(body.starts_with("## Problem"));
        assert!(body.contains("Body here."));
        assert!(!body.contains("type: enhancement"));
    }

    #[test]
    fn test_strip_frontmatter_no_frontmatter() {
        let content = "Just plain text.";
        assert_eq!(strip_frontmatter(content), content);
    }

    // REQ: REQ-010 - artifact generation
    #[test]
    fn test_generate_requirements_md() {
        let structured = StructuredIssue {
            problem: "Things are broken.".to_string(),
            requirements: vec![
                Requirement {
                    id: "R1".to_string(),
                    text: "Fix the parser".to_string(),
                    priority: None,
                },
                Requirement {
                    id: "R2".to_string(),
                    text: "Add tests".to_string(),
                    priority: Some("high".to_string()),
                },
            ],
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "AC1".to_string(),
                text: "Parser works".to_string(),
            }],
            scope: IssueScope::default(),
            key_decisions: vec![],
            reference_context: None,
        };

        let md = generate_requirements_md("test-change", "my-group", &structured);
        assert!(md.contains("change: test-change"));
        assert!(md.contains("group: my-group"));
        assert!(md.contains("source: structured-issue"));
        assert!(md.contains("Things are broken."));
        assert!(md.contains("**R1**: Fix the parser"));
        assert!(md.contains("**R2** (high): Add tests"));
        assert!(md.contains("**AC1**: Parser works"));
    }

    #[test]
    fn test_generate_pre_clarifications_md_with_decisions() {
        let decisions = vec![Decision {
            id: "D1".to_string(),
            text: "Use REST".to_string(),
        }];
        let md = generate_pre_clarifications_md("test-change", "my-group", &decisions);
        assert!(md.contains("status: answered"));
        assert!(md.contains("Decision D1"));
        assert!(md.contains("Use REST"));
    }

    #[test]
    fn test_generate_pre_clarifications_md_empty() {
        let md = generate_pre_clarifications_md("test-change", "my-group", &[]);
        assert!(md.contains("status: skipped"));
        assert!(md.contains("No pre-clarification"));
    }

    #[test]
    fn test_generate_post_clarifications_md() {
        let scope = IssueScope {
            in_scope: "Feature A, Feature B".to_string(),
            out_of_scope: "Feature C".to_string(),
        };
        let criteria = vec![AcceptanceCriterion {
            id: "AC1".to_string(),
            text: "Feature works".to_string(),
        }];
        let md = generate_post_clarifications_md("test-change", "my-group", &scope, &criteria);
        assert!(md.contains("status: skipped"));
        assert!(md.contains("source: structured-issue"));
        assert!(md.contains("Feature A, Feature B"));
        assert!(md.contains("Feature C"));
        assert!(md.contains("**AC1**: Feature works"));
    }

    #[test]
    fn test_generate_reference_context_md() {
        let ref_ctx = IssueReferenceContext {
            specs: vec![],
            spec_plan: vec![],
            raw: "See `projects/agentic-workflow/tech-design/core/` for details.".to_string(),
        };
        let md = generate_reference_context_md("test-change", "my-group", &ref_ctx);
        assert!(md.contains("source: structured-issue"));
        assert!(md.contains("projects/agentic-workflow/tech-design/core/"));
    }

    #[test]
    fn test_generate_reference_context_md_empty() {
        let ref_ctx = IssueReferenceContext::default();
        let md = generate_reference_context_md("test-change", "my-group", &ref_ctx);
        assert!(md.contains("not provided"));
    }

    // REQ: REQ-011 - split_sections
    #[test]
    fn test_split_sections() {
        let body = "## Problem\nSome problem\n\n## Requirements\n- R1: thing\n\n## Scope\nAll.\n";
        let sections = split_sections(body);
        assert!(sections.contains_key("Problem"));
        assert!(sections.contains_key("Requirements"));
        assert!(sections.contains_key("Scope"));
        assert_eq!(sections.len(), 3);
    }

    #[test]
    fn test_split_sections_preserves_body() {
        let body = "## Problem\nLine 1\nLine 2\n\n## Scope\nScope content\n";
        let sections = split_sections(body);
        let problem = sections.get("Problem").unwrap();
        assert!(problem.contains("Line 1"));
        assert!(problem.contains("Line 2"));
    }

    // =====================================================================
    // Write-time validation tests
    // =====================================================================

    // REQ: structured-issue#R3 — draft state requires only headers
    #[test]
    fn test_validate_draft_passes_with_empty_required_sections() {
        let body = "## Problem\n\n## Requirements\n";
        assert!(validate_structured_issue(body, IssueState::Draft).is_ok());
    }

    // REQ: structured-issue#R3 — draft state still rejects missing Problem
    #[test]
    fn test_validate_draft_rejects_missing_problem() {
        let body = "## Requirements\n- R1: do thing\n";
        let err = validate_structured_issue(body, IssueState::Draft).unwrap_err();
        assert_eq!(err.code, "VALIDATION_ERROR");
        assert!(err.error.contains("## Problem"));
        assert_eq!(err.missing, vec!["## Problem".to_string()]);
    }

    // REQ: structured-issue#R3 — draft state rejects missing Requirements
    #[test]
    fn test_validate_draft_rejects_missing_requirements() {
        let body = "## Problem\nSomething.\n";
        let err = validate_structured_issue(body, IssueState::Draft).unwrap_err();
        assert!(err.error.contains("## Requirements"));
        assert_eq!(err.missing, vec!["## Requirements".to_string()]);
    }

    // REQ: structured-issue#R3 — both missing reported together
    #[test]
    fn test_validate_reports_multiple_missing() {
        let body = "Just plain text, no headers.";
        let err = validate_structured_issue(body, IssueState::Draft).unwrap_err();
        assert_eq!(err.missing.len(), 2);
        assert!(err.missing.contains(&"## Problem".to_string()));
        assert!(err.missing.contains(&"## Requirements".to_string()));
        assert!(err.error.contains("missing required sections"));
    }

    // REQ: structured-issue#R4 — open state enforces R-id format
    #[test]
    fn test_validate_open_rejects_bad_rid() {
        let body = "## Problem\nBroken thing.\n\n## Requirements\n- Add feature X\n";
        let err = validate_structured_issue(body, IssueState::Open).unwrap_err();
        assert_eq!(err.code, "VALIDATION_ERROR");
        assert!(err.error.contains("R-id format"));
        assert!(err.error.contains("Add feature X"));
    }

    // REQ: structured-issue#R4 — valid plain R-id passes in open state
    #[test]
    fn test_validate_open_accepts_plain_rid() {
        let body = "## Problem\nP\n\n## Requirements\n- R1: do something\n- R2: do another\n";
        assert!(validate_structured_issue(body, IssueState::Open).is_ok());
    }

    // REQ: structured-issue#R4 — bold R-id is accepted
    #[test]
    fn test_validate_open_accepts_bold_rid() {
        let body =
            "## Problem\nP\n\n## Requirements\n- **R1**: do thing\n- **R2** (high): do other\n";
        assert!(validate_structured_issue(body, IssueState::Open).is_ok());
    }

    // REQ: structured-issue#R4 — draft skips R-id format check
    #[test]
    fn test_validate_draft_skips_rid_check() {
        let body = "## Problem\nP\n\n## Requirements\n- Add feature X\n";
        // Bad R-id, but draft skips the format check.
        assert!(validate_structured_issue(body, IssueState::Draft).is_ok());
    }

    // REQ: structured-issue#R6 — error JSON shape
    #[test]
    fn test_validation_error_json_shape() {
        let err = ValidationError::new("missing", vec!["## Problem".to_string()]);
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "VALIDATION_ERROR");
        assert_eq!(json["error"], "missing");
        assert_eq!(json["missing"][0], "## Problem");
    }

    // REQ: structured-issue#R4 — closed state behaves like open
    #[test]
    fn test_validate_closed_enforces_rid_like_open() {
        let body = "## Problem\nP\n\n## Requirements\n- bare item\n";
        assert!(validate_structured_issue(body, IssueState::Closed).is_err());
    }

    // is_valid_rid_item helper
    #[test]
    fn test_is_valid_rid_item_variants() {
        assert!(is_valid_rid_item("R1: text"));
        assert!(is_valid_rid_item("R42: text"));
        assert!(is_valid_rid_item("**R1**: text"));
        assert!(is_valid_rid_item("**R7** (high): text"));
        assert!(!is_valid_rid_item("Add feature X"));
        assert!(!is_valid_rid_item("R: missing num"));
        assert!(!is_valid_rid_item("RX: not digit"));
        assert!(!is_valid_rid_item("**RX**: not digit"));
    }

    // REQ: bug-init-change-try-structured-issue-skip-silently-sho
    #[test]
    fn test_resolve_issue_slug_github_number_in_description() {
        let tmp = tempfile::TempDir::new().unwrap();
        let issues_open = crate::shared::workspace::issues_path(tmp.path()).join("open");
        std::fs::create_dir_all(&issues_open).unwrap();
        let slug = "enhancement-py3-12-conformance-generator-iterator-protocol";
        std::fs::write(
            issues_open.join(format!("{}.md", slug)),
            "---\n\
             type: enhancement\n\
             github_id: 756\n\
             labels:\n\
             - crate:mamba\n\
             ---\n\
             \n\
             ## Problem\n\nBody.\n\
             ## Requirements\n\n- R1\n\
             ## Scope\n\n### In Scope\n- x\n",
        )
        .unwrap();

        // Strategy 3: #<num> in description resolves via github_id lookup
        let resolved = resolve_issue_slug(
            tmp.path(),
            "Py3.12 conformance: Generator & iterator protocol #756",
            None,
        );
        assert_eq!(resolved, Some(slug.to_string()));
    }

    #[test]
    fn test_resolve_issue_slug_explicit_issues_array() {
        let tmp = tempfile::TempDir::new().unwrap();
        let issues_open = crate::shared::workspace::issues_path(tmp.path()).join("open");
        std::fs::create_dir_all(&issues_open).unwrap();
        let slug = "bug-sample";
        std::fs::write(
            issues_open.join(format!("{}.md", slug)),
            "---\ntype: bug\ngithub_id: 42\n---\n\n## Problem\n\n## Requirements\n\n## Scope\n",
        )
        .unwrap();

        // Strategy 1: `#<num>` in the explicit issues list
        let resolved = resolve_issue_slug(tmp.path(), "any description", Some(&["#42".into()]));
        assert_eq!(resolved, Some(slug.to_string()));

        // Strategy 1 variant: bare slug in the explicit issues list
        let resolved = resolve_issue_slug(tmp.path(), "any description", Some(&[slug.into()]));
        assert_eq!(resolved, Some(slug.to_string()));
    }

    #[test]
    fn test_resolve_issue_slug_legacy_literal() {
        let tmp = tempfile::TempDir::new().unwrap();
        let issues_open = crate::shared::workspace::issues_path(tmp.path()).join("open");
        std::fs::create_dir_all(&issues_open).unwrap();
        let slug = "legacy-slug-form";
        std::fs::write(
            issues_open.join(format!("{}.md", slug)),
            "---\ntype: bug\n---\n\n## Problem\n\n## Requirements\n\n## Scope\n",
        )
        .unwrap();

        // Strategy 2: issue:<slug> literal still works
        let resolved = resolve_issue_slug(tmp.path(), &format!("Implement issue:{}", slug), None);
        assert_eq!(resolved, Some(slug.to_string()));
    }

    #[test]
    fn test_resolve_issue_slug_no_match() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(crate::shared::workspace::issues_path(tmp.path()).join("open"))
            .unwrap();

        // #999 doesn't exist locally → None (not a panic, not a false positive)
        assert_eq!(resolve_issue_slug(tmp.path(), "Fix #999 bug", None), None);
        // No issue ref at all → None
        assert_eq!(
            resolve_issue_slug(tmp.path(), "just some words", None),
            None
        );
    }

    // REQ: issue-centric-workflow#U1 — UUID prefix expands to unique slug
    #[test]
    fn test_find_slug_by_uuid_prefix_unique() {
        let tmp = tempfile::TempDir::new().unwrap();
        let issues_open = crate::shared::workspace::issues_path(tmp.path()).join("open");
        std::fs::create_dir_all(&issues_open).unwrap();
        std::fs::write(
            issues_open.join("enhancement-foo.md"),
            "---\ntype: enhancement\nid: 45ac7e9e-7e1f-4bca-93a7-29cf08016db7\n---\nbody\n",
        )
        .unwrap();
        std::fs::write(
            issues_open.join("bug-bar.md"),
            "---\ntype: bug\nid: deadbeef-1111-2222-3333-444455556666\n---\nbody\n",
        )
        .unwrap();

        assert_eq!(
            find_slug_by_uuid_prefix(tmp.path(), "45ac7e9e").unwrap(),
            Some("enhancement-foo".to_string())
        );
        // Uppercase prefix still matches (case-insensitive)
        assert_eq!(
            find_slug_by_uuid_prefix(tmp.path(), "DEADBEEF").unwrap(),
            Some("bug-bar".to_string())
        );
        // Unknown prefix → None, not error
        assert_eq!(
            find_slug_by_uuid_prefix(tmp.path(), "cafebabe").unwrap(),
            None
        );
    }

    // REQ: issue-centric-workflow#U2 — ambiguous prefix errors with candidate list
    #[test]
    fn test_find_slug_by_uuid_prefix_ambiguous() {
        let tmp = tempfile::TempDir::new().unwrap();
        let issues_open = crate::shared::workspace::issues_path(tmp.path()).join("open");
        std::fs::create_dir_all(&issues_open).unwrap();
        std::fs::write(
            issues_open.join("one.md"),
            "---\nid: abcd1111-0000-0000-0000-000000000000\n---\n",
        )
        .unwrap();
        std::fs::write(
            issues_open.join("two.md"),
            "---\nid: abcd2222-0000-0000-0000-000000000000\n---\n",
        )
        .unwrap();

        let err = find_slug_by_uuid_prefix(tmp.path(), "abcd").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("ambiguous"), "expected ambiguous error: {msg}");
        assert!(msg.contains("one"));
        assert!(msg.contains("two"));
    }

    #[test]
    fn test_looks_like_uuid_prefix() {
        assert!(looks_like_uuid_prefix("45ac7e9e"));
        assert!(looks_like_uuid_prefix("45ac7e9e-7e1f"));
        assert!(looks_like_uuid_prefix(
            "45ac7e9e-7e1f-4bca-93a7-29cf08016db7"
        ));
        // Too short
        assert!(!looks_like_uuid_prefix("abc"));
        // Not hex
        assert!(!looks_like_uuid_prefix("not-a-uuid-xyz"));
        // Slugs with hex-ish chunks still reject due to non-hex chars
        assert!(!looks_like_uuid_prefix("enhancement-foo"));
    }

    // REQ: issue-centric-workflow#U1 — resolve_issue_ref picks up UUID prefix via --issue
    #[test]
    fn test_resolve_issue_ref_by_uuid_prefix() {
        let tmp = tempfile::TempDir::new().unwrap();
        let issues_open = crate::shared::workspace::issues_path(tmp.path()).join("open");
        std::fs::create_dir_all(&issues_open).unwrap();
        std::fs::write(
            issues_open.join("my-slug.md"),
            "---\nid: 45ac7e9e-1111-2222-3333-444455556666\n---\n",
        )
        .unwrap();

        assert_eq!(
            resolve_issue_ref(tmp.path(), "45ac7e9e"),
            Some("my-slug".to_string())
        );
    }
}
// CODEGEN-END
