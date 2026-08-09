//! Goal / How / Acceptance / Never — the authored agent-instruction shape.
//!
//! One structure for every instruction addressed to an agent: a `type=change`
//! work item, a `SKILL.md`, a dispatch injection. The design constraint is
//! measured rather than assumed — the legacy six-section body degenerated into
//! title echoes precisely in the sections no code refused, so every section
//! defined here has a refusal below it. A section this module cannot refuse
//! does not belong in the template.
//!
//! See `CONTRIBUTING.md`, section "Authoring convention: every agent
//! instruction is Goal / How / Acceptance / Never".

use super::Issue;
use std::collections::BTreeSet;

/// The four canonical GHAN H2 headings, in canonical order.
pub const GHAN_SECTIONS: &[&str] = &["## Goal", "## How", "## Acceptance", "## Never"];

/// The legacy six-section change-body headings GHAN coexists with.
pub const LEGACY_SECTIONS: &[&str] = &[
    "## Problem",
    "## Capability Alignment",
    "## Requirements",
    "## Scope",
    "## Acceptance Criteria",
    "## Reference Context",
];

const HOW_PREMISES: &str = "### Verified premises";
const HOW_CHANGE_POINTS: &str = "### Change points";
const HOW_FROZEN: &str = "### Frozen decisions";
const ACCEPTANCE_NEGATIVE_CONTROL: &str = "### Negative control";
const NEVER_MUST_NOT_TOUCH: &str = "### Must not touch";
const NEVER_MUST_NOT_DO: &str = "### Must not do";

/// Hedges that turn a premise from an observation into an inference.
const HEDGE_WORDS: &[&str] = &[
    "should",
    "might",
    "probably",
    "seems",
    "appears",
    "likely",
    "presumably",
    "supposedly",
    "應該",
    "可能",
    "推測",
    "看起來",
    "似乎",
    "大概",
    "或許",
];

/// Phrases that make a negative control require the gate to go red.
const FAILURE_ASSERTIONS: &[&str] = &[
    "must fail",
    "must go red",
    "must be red",
    "must turn red",
    "必須紅",
    "必須失敗",
    "必须红",
];

/// Which authored vocabulary a work-item body is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WiBodyShape {
    /// No recognized change-body heading at all.
    Unstructured,
    /// Legacy six-section change body.
    Legacy,
    /// Goal / How / Acceptance / Never.
    Ghan,
    /// Both vocabularies present; refuse rather than guess which one governs.
    Mixed,
}

/// Classify a change-body by the heading vocabulary it uses.
pub fn body_shape(body: &str) -> WiBodyShape {
    let headings = h2_headings(body);
    let ghan = headings.iter().any(|h| heading_in(h, GHAN_SECTIONS));
    let legacy = headings.iter().any(|h| heading_in(h, LEGACY_SECTIONS));
    match (ghan, legacy) {
        (true, true) => WiBodyShape::Mixed,
        (true, false) => WiBodyShape::Ghan,
        (false, true) => WiBodyShape::Legacy,
        (false, false) => WiBodyShape::Unstructured,
    }
}

/// Validate one authored GHAN work item, including the boundedness check the
/// legacy path applies via `## Scope`.
pub fn validate_ghan_body(issue: &Issue) -> Vec<String> {
    let mut errors = validate_ghan_sections(&issue.body);
    if crate::issues::planner::looks_too_large_for_atomic_wi(issue) {
        errors.push(
            "too-large: non-epic work-item appears roadmap-sized; run `aw wi plan` or create `--type epic` first".to_string(),
        );
    }
    errors
}

/// Section-level GHAN rules. Split from [`validate_ghan_body`] so the rules can
/// be exercised against a body alone.
pub(crate) fn validate_ghan_sections(body: &str) -> Vec<String> {
    let headings = h2_headings(body);
    let mut errors = Vec::new();

    for required in GHAN_SECTIONS {
        if !headings.iter().any(|h| heading_eq(h, required)) {
            errors.push(format!("ghan: missing required {} section", required));
        }
    }
    for heading in &headings {
        if !heading_in(heading, GHAN_SECTIONS) {
            errors.push(format!(
                "ghan: unexpected H2 `{}`; a GHAN work item carries exactly: {}",
                heading.trim(),
                GHAN_SECTIONS.join(", ")
            ));
        }
    }
    // Per-section rules read section content; reporting them against a missing
    // or foreign section would bury the structural cause under noise.
    if !errors.is_empty() {
        return errors;
    }

    let goal = section_at(body, 2, "## Goal").unwrap_or_default();
    let how = section_at(body, 2, "## How").unwrap_or_default();
    let acceptance = section_at(body, 2, "## Acceptance").unwrap_or_default();
    let never = section_at(body, 2, "## Never").unwrap_or_default();

    errors.extend(validate_goal(&goal));
    errors.extend(validate_how(&how));
    errors.extend(validate_acceptance(&acceptance));
    errors.extend(validate_never(&never, &how));
    errors
}

// ---------------------------------------------------------------------------
// Goal
// ---------------------------------------------------------------------------

fn validate_goal(content: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let trimmed = content.trim();
    if trimmed.is_empty() {
        errors.push("ghan: ## Goal is empty".to_string());
        return errors;
    }
    if let Some(marker) = placeholder_marker(trimmed) {
        errors.push(format!(
            "ghan: ## Goal still carries the `{}` placeholder",
            marker
        ));
    }
    if trimmed.lines().any(is_list_item) {
        errors.push(
            "ghan: ## Goal must be one observable-difference sentence, not a list".to_string(),
        );
    }
    let paragraphs = trimmed
        .split("\n\n")
        .filter(|part| !part.trim().is_empty())
        .count();
    if paragraphs > 1 {
        errors.push(
            "ghan: ## Goal must be a single paragraph naming one observation point".to_string(),
        );
    }
    errors
}

// ---------------------------------------------------------------------------
// How
// ---------------------------------------------------------------------------

fn validate_how(content: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for required in [HOW_PREMISES, HOW_CHANGE_POINTS, HOW_FROZEN] {
        if section_at(content, 3, required).is_none() {
            errors.push(format!("ghan: ## How missing `{}` sub-section", required));
        }
    }
    if !errors.is_empty() {
        return errors;
    }

    let premises = section_at(content, 3, HOW_PREMISES).unwrap_or_default();
    let premise_items = list_items(&premises);
    if premise_items.is_empty() {
        errors.push(format!(
            "ghan: `{}` needs at least one observed premise",
            HOW_PREMISES
        ));
    }
    for item in &premise_items {
        if file_line_ref(item).is_none() {
            errors.push(format!(
                "ghan: premise carries no `file:line` evidence coordinate: '{}'",
                preview(item)
            ));
        }
        if let Some(hedge) = hedge_word(item) {
            errors.push(format!(
                "ghan: premise hedges with '{}'; a premise is an observation, not an inference: '{}'",
                hedge,
                preview(item)
            ));
        }
    }

    let change_points = section_at(content, 3, HOW_CHANGE_POINTS).unwrap_or_default();
    let change_items = list_items(&change_points);
    if change_items.is_empty() {
        errors.push(format!(
            "ghan: `{}` is empty; a change work item must name at least one write target (use `--type spike` for investigation)",
            HOW_CHANGE_POINTS
        ));
    }
    for item in &change_items {
        if path_ref(item).is_none() {
            errors.push(format!(
                "ghan: change point names no path: '{}'",
                preview(item)
            ));
        }
    }

    let frozen = section_at(content, 3, HOW_FROZEN).unwrap_or_default();
    if !frozen.lines().any(is_real_line) {
        errors.push(format!(
            "ghan: `{}` must record the decisions and exclusions already fixed (write `none` explicitly if there are none)",
            HOW_FROZEN
        ));
    }
    errors
}

// ---------------------------------------------------------------------------
// Acceptance
// ---------------------------------------------------------------------------

fn validate_acceptance(content: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let rows = table_rows(content);
    if rows.is_empty() {
        errors.push("ghan: ## Acceptance needs a gate table with at least one row".to_string());
    }
    for row in &rows {
        if row.len() < 5 {
            errors.push(format!(
                "ghan: gate row needs 5 columns (#, command, current, target, why it cannot hold by accident): '{}'",
                preview(&row.join(" | "))
            ));
            continue;
        }
        let command = row[1].trim();
        let current = row[2].trim();
        let target = row[3].trim();
        let why = row[4].trim();
        if !command.contains('`') {
            errors.push(format!(
                "ghan: gate command must be a verbatim backticked command: '{}'",
                preview(command)
            ));
        }
        if current.eq_ignore_ascii_case(target) {
            errors.push(format!(
                "ghan: gate row states the same current and target observation ('{}'); it cannot discriminate",
                preview(current)
            ));
        }
        if !is_real_line(why) {
            errors.push(format!(
                "ghan: gate row must say why it cannot hold by accident: '{}'",
                preview(command)
            ));
        }
    }

    match section_at(content, 3, ACCEPTANCE_NEGATIVE_CONTROL) {
        None => errors.push(format!(
            "ghan: ## Acceptance missing `{}`; a gate nobody has seen fail proves nothing",
            ACCEPTANCE_NEGATIVE_CONTROL
        )),
        Some(control) => {
            if !asserts_failure(&control) {
                errors.push(format!(
                    "ghan: `{}` must require the gate to go red under the mutation",
                    ACCEPTANCE_NEGATIVE_CONTROL
                ));
            }
            if sha256_token(&control).is_none() {
                errors.push(format!(
                    "ghan: `{}` must name the sha256 the mutated file restores to",
                    ACCEPTANCE_NEGATIVE_CONTROL
                ));
            }
        }
    }
    errors
}

// ---------------------------------------------------------------------------
// Never
// ---------------------------------------------------------------------------

fn validate_never(content: &str, how: &str) -> Vec<String> {
    let mut errors = Vec::new();
    match content.lines().map(str::trim).find(|line| !line.is_empty()) {
        None => {
            errors.push("ghan: ## Never is empty".to_string());
            return errors;
        }
        Some(line) if is_list_item(line) || line.starts_with('#') => errors.push(
            "ghan: ## Never must open with a line fixing the addressee before any list".to_string(),
        ),
        Some(_) => {}
    }

    let mut missing_list = false;
    for required in [NEVER_MUST_NOT_TOUCH, NEVER_MUST_NOT_DO] {
        match section_at(content, 3, required) {
            None => {
                errors.push(format!("ghan: ## Never missing `{}` list", required));
                missing_list = true;
            }
            Some(list) if list_items(&list).is_empty() => errors.push(format!(
                "ghan: `{}` has no entries; a limit nobody can name is not a limit",
                required
            )),
            Some(_) => {}
        }
    }
    if missing_list {
        return errors;
    }

    let change_paths: BTreeSet<String> = list_items(&section_at(how, 3, HOW_CHANGE_POINTS).unwrap_or_default())
        .iter()
        .filter_map(|item| path_ref(item))
        .map(normalize_path)
        .collect();
    let must_not_touch = section_at(content, 3, NEVER_MUST_NOT_TOUCH).unwrap_or_default();
    for item in list_items(&must_not_touch) {
        let Some(path) = path_ref(&item) else {
            continue;
        };
        if change_paths.contains(&normalize_path(path)) {
            errors.push(format!(
                "ghan: `{}` is listed as both a change point and must-not-touch",
                path
            ));
        }
    }
    errors
}

// ---------------------------------------------------------------------------
// Markdown helpers
// ---------------------------------------------------------------------------

/// H2 heading lines in document order, verbatim and fence-aware so a fenced
/// command block inside `## Acceptance` cannot masquerade as a heading.
fn h2_headings(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut fence: Option<usize> = None;
    for raw in body.lines() {
        if let Some(len) = fence_len(raw) {
            fence = match fence {
                Some(open) if len >= open => None,
                Some(open) => Some(open),
                None => Some(len),
            };
            continue;
        }
        if fence.is_some() {
            continue;
        }
        let line = raw.trim_end();
        if line.starts_with("## ") && !line.starts_with("### ") {
            out.push(line.to_string());
        }
    }
    out
}

/// Content under an exact heading line at `level` (2 or 3), up to the next
/// heading at the same or a shallower level.
fn section_at(text: &str, level: usize, heading: &str) -> Option<String> {
    let prefix = format!("{} ", "#".repeat(level));
    let deeper = format!("{} ", "#".repeat(level + 1));
    let mut collecting = false;
    let mut found = false;
    let mut out: Vec<&str> = Vec::new();
    let mut fence: Option<usize> = None;

    for raw in text.lines() {
        if let Some(len) = fence_len(raw) {
            fence = match fence {
                Some(open) if len >= open => None,
                Some(open) => Some(open),
                None => Some(len),
            };
            if collecting {
                out.push(raw);
            }
            continue;
        }
        let line = raw.trim_end();
        let is_heading = fence.is_none()
            && line.starts_with('#')
            && (line.starts_with(&prefix) || shallower_heading(line, level));
        if is_heading && !line.starts_with(&deeper) {
            if collecting {
                break;
            }
            if heading_eq(line, heading) {
                collecting = true;
                found = true;
            }
            continue;
        }
        if collecting {
            out.push(raw);
        }
    }
    found.then(|| out.join("\n"))
}

/// Is `line` an ATX heading strictly shallower than `level`?
fn shallower_heading(line: &str, level: usize) -> bool {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    hashes > 0 && hashes < level && line[hashes..].starts_with(' ')
}

/// Opening/closing fence width, or `None` when the line is not a fence.
fn fence_len(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let ticks = trimmed.chars().take_while(|c| *c == '`').count();
    if ticks >= 3 {
        return Some(ticks);
    }
    let tildes = trimmed.chars().take_while(|c| *c == '~').count();
    (tildes >= 3).then_some(tildes)
}

fn heading_eq(line: &str, heading: &str) -> bool {
    line.trim().eq_ignore_ascii_case(heading.trim())
}

fn heading_in(line: &str, set: &[&str]) -> bool {
    set.iter().any(|candidate| heading_eq(line, candidate))
}

fn is_list_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ")
}

/// Bullet text with the marker stripped.
fn list_items(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| is_list_item(line))
        .map(|line| {
            line.trim_start()
                .trim_start_matches(['-', '*', '+'])
                .trim()
                .to_string()
        })
        .filter(|item| !item.is_empty())
        .collect()
}

fn placeholder_marker(text: &str) -> Option<&'static str> {
    ["(fill)", "(replace-this)"]
        .into_iter()
        .find(|marker| text.contains(marker))
}

/// Non-empty, non-placeholder content.
fn is_real_line(line: &str) -> bool {
    let trimmed = line
        .trim()
        .trim_start_matches(['-', '*', '+', '#'])
        .trim()
        .to_ascii_lowercase();
    if trimmed.is_empty() {
        return false;
    }
    !matches!(
        trimmed.as_str(),
        "(fill)" | "(replace-this)" | "tbd" | "todo" | "maybe" | "unclear" | "uncertain"
    )
}

/// A `path/to/file.rs:123` evidence coordinate.
fn file_line_ref(text: &str) -> Option<&str> {
    text.split(is_token_break).find(|token| {
        let Some((path, line)) = token.rsplit_once(':') else {
            return false;
        };
        let digits = line.trim_end_matches(|c: char| !c.is_ascii_digit());
        !digits.is_empty()
            && digits.chars().all(|c| c.is_ascii_digit())
            && path.contains('.')
            && path.chars().any(|c| c.is_ascii_alphanumeric())
    })
}

/// A path-like token, with or without a line suffix.
fn path_ref(text: &str) -> Option<&str> {
    text.split(is_token_break).find(|token| {
        let bare = token.trim_end_matches([':', '.', ',']);
        let bare = bare.rsplit_once(':').map_or(bare, |(head, tail)| {
            if tail.chars().all(|c| c.is_ascii_digit()) {
                head
            } else {
                bare
            }
        });
        if bare.is_empty() {
            return false;
        }
        let has_known_ext = bare.rsplit_once('.').is_some_and(|(stem, ext)| {
            !stem.is_empty()
                && matches!(
                    ext,
                    "rs" | "py" | "md" | "toml" | "json" | "yaml" | "yml" | "sh" | "ts" | "tsx"
                        | "js" | "jsx" | "sql" | "proto"
                )
        });
        has_known_ext || (bare.contains('/') && bare.contains('.'))
    })
}

fn is_token_break(c: char) -> bool {
    c.is_whitespace() || matches!(c, '`' | ',' | '(' | ')' | '[' | ']' | '"' | '\'' | ';')
}

fn normalize_path(token: &str) -> String {
    let bare = token.trim_matches(|c: char| is_token_break(c) || c == '.');
    match bare.rsplit_once(':') {
        Some((head, tail)) if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) => {
            head.to_string()
        }
        _ => bare.to_string(),
    }
}

fn hedge_word(text: &str) -> Option<&'static str> {
    let lower = text.to_ascii_lowercase();
    HEDGE_WORDS.iter().copied().find(|hedge| {
        if hedge.is_ascii() {
            lower
                .split(|c: char| !c.is_ascii_alphanumeric())
                .any(|word| word == *hedge)
        } else {
            text.contains(hedge)
        }
    })
}

fn asserts_failure(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    FAILURE_ASSERTIONS
        .iter()
        .any(|phrase| lower.contains(phrase) || text.contains(*phrase))
}

fn sha256_token(text: &str) -> Option<&str> {
    text.split(is_token_break).find(|token| {
        token.len() == 64 && token.chars().all(|c| c.is_ascii_hexdigit())
    })
}

/// Data rows of the first markdown table, header and separator removed.
fn table_rows(content: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut seen_separator = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            if seen_separator && !rows.is_empty() {
                break;
            }
            continue;
        }
        let cells: Vec<String> = trimmed
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().to_string())
            .collect();
        if cells
            .iter()
            .all(|cell| !cell.is_empty() && cell.chars().all(|c| c == '-' || c == ':'))
        {
            seen_separator = true;
            continue;
        }
        if seen_separator {
            rows.push(cells);
        }
    }
    rows
}

fn preview(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= 60 {
        return flat;
    }
    flat.chars().take(60).collect()
}

/// A minimal well-formed GHAN body, shared with the CLI routing tests so the
/// rules and the wiring that reaches them are exercised against one artifact.
#[cfg(test)]
pub(crate) const SAMPLE_GHAN_BODY: &str = r#"## Goal

Running `aw wi validate` on a GHAN body reports section errors instead of `body must contain structured work-item sections`.

## How

### Verified premises

- `apps/agentic-workflow/src/cli/issues.rs:2176` pushes the unstructured error and early-returns.
- `apps/agentic-workflow/src/services/issue_parser.rs:245` hard-requires the legacy problem and requirements headings.

### Change points

- `apps/agentic-workflow/src/cli/issues.rs` — route by body shape.
- `apps/agentic-workflow/src/issues/ghan.rs` — the validator itself.

### Frozen decisions

- The legacy six-section shape stays valid; this is coexistence, not replacement.

## Acceptance

| # | Command | Current | Target | Why it cannot hold by accident |
|---|---|---|---|---|
| 1 | `cargo test -p agentic-workflow --lib -- --test-threads=1` | 3755 passed / 0 failed | 3770 passed / 0 failed | the new cases assert refusal strings that do not exist before the change |

### Negative control

Delete the shape branch in `validate_publishable_issue_body`. Re-run the gate; the new cases must fail.
Restore the file byte-for-byte to sha256 `59d66dea106b9bd7c8c319d9096f1e5fe1c82957faa4837a8fa8c7cd6528a32b`.

## Never

The addressee of these limits is the agent executing this work item, not the dispatcher.

### Must not touch

- `apps/agentic-workflow/external-contracts/src/wi_contract_fixture.py`

### Must not do

- Do not relax an existing assertion to make the gate green.
- Do not narrow the test selector so it matches nothing.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD_BODY: &str = SAMPLE_GHAN_BODY;

    fn body_without(section: &str) -> String {
        let mut out = String::new();
        let mut skipping = false;
        for line in GOOD_BODY.lines() {
            if line.starts_with("## ") {
                skipping = line.trim() == section;
            }
            if !skipping {
                out.push_str(line);
                out.push('\n');
            }
        }
        out
    }

    #[test]
    fn good_ghan_body_passes_every_section_rule() {
        assert_eq!(validate_ghan_sections(GOOD_BODY), Vec::<String>::new());
    }

    #[test]
    fn shape_detection_separates_ghan_from_legacy_and_mixed() {
        assert_eq!(body_shape(GOOD_BODY), WiBodyShape::Ghan);
        assert_eq!(
            body_shape("## Problem\n\nx\n\n## Requirements\n\n- R1: y\n"),
            WiBodyShape::Legacy
        );
        assert_eq!(
            body_shape("## Goal\n\nx\n\n## Problem\n\ny\n"),
            WiBodyShape::Mixed
        );
        assert_eq!(body_shape("just prose\n"), WiBodyShape::Unstructured);
    }

    #[test]
    fn acceptance_criteria_heading_is_not_read_as_acceptance() {
        // `## Acceptance Criteria` starts with `## Acceptance`; substring
        // matching would classify every legacy body as GHAN.
        assert_eq!(
            body_shape("## Problem\n\nx\n\n## Acceptance Criteria\n\n- AC1: y\n"),
            WiBodyShape::Legacy
        );
    }

    #[test]
    fn each_missing_section_is_named() {
        for section in GHAN_SECTIONS {
            let errors = validate_ghan_sections(&body_without(section));
            assert!(
                errors
                    .iter()
                    .any(|e| e == &format!("ghan: missing required {} section", section)),
                "removing {section} did not produce its own error: {errors:?}"
            );
        }
    }

    #[test]
    fn goal_rejects_a_list_and_a_second_paragraph() {
        let listed = GOOD_BODY.replace(
            "Running `aw wi validate` on a GHAN body reports section errors instead of `body must contain structured work-item sections`.",
            "- one goal\n- another goal",
        );
        assert!(validate_ghan_sections(&listed)
            .iter()
            .any(|e| e.contains("not a list")));

        let two_paragraphs = GOOD_BODY.replace(
            "Running `aw wi validate` on a GHAN body reports section errors instead of `body must contain structured work-item sections`.",
            "First observation point moves.\n\nSecond observation point also moves.",
        );
        assert!(validate_ghan_sections(&two_paragraphs)
            .iter()
            .any(|e| e.contains("single paragraph")));
    }

    #[test]
    fn premise_without_file_line_is_refused() {
        let body = GOOD_BODY.replace(
            "- `apps/agentic-workflow/src/cli/issues.rs:2176` pushes the unstructured error and early-returns.",
            "- the validator rejects unstructured bodies.",
        );
        assert!(validate_ghan_sections(&body)
            .iter()
            .any(|e| e.contains("no `file:line` evidence coordinate")));
    }

    #[test]
    fn hedged_premise_is_refused() {
        let body = GOOD_BODY.replace(
            "- `apps/agentic-workflow/src/cli/issues.rs:2176` pushes the unstructured error and early-returns.",
            "- `apps/agentic-workflow/src/cli/issues.rs:2176` should push the unstructured error.",
        );
        assert!(validate_ghan_sections(&body)
            .iter()
            .any(|e| e.contains("hedges with 'should'")));
    }

    #[test]
    fn empty_change_point_list_routes_to_spike() {
        let body = GOOD_BODY
            .replace(
                "- `apps/agentic-workflow/src/cli/issues.rs` — route by body shape.\n",
                "",
            )
            .replace(
                "- `apps/agentic-workflow/src/issues/ghan.rs` — the validator itself.\n",
                "",
            );
        assert!(validate_ghan_sections(&body)
            .iter()
            .any(|e| e.contains("--type spike")));
    }

    #[test]
    fn gate_with_identical_current_and_target_cannot_discriminate() {
        let body = GOOD_BODY.replace(
            "| 3755 passed / 0 failed | 3770 passed / 0 failed |",
            "| 3755 passed / 0 failed | 3755 passed / 0 failed |",
        );
        assert!(validate_ghan_sections(&body)
            .iter()
            .any(|e| e.contains("cannot discriminate")));
    }

    #[test]
    fn missing_negative_control_is_refused() {
        let body = GOOD_BODY.replace("### Negative control", "### Notes");
        assert!(validate_ghan_sections(&body)
            .iter()
            .any(|e| e.contains("a gate nobody has seen fail proves nothing")));
    }

    #[test]
    fn negative_control_needs_a_restore_digest_and_a_red_assertion() {
        let no_digest = GOOD_BODY.replace(
            "59d66dea106b9bd7c8c319d9096f1e5fe1c82957faa4837a8fa8c7cd6528a32b",
            "the original content",
        );
        assert!(validate_ghan_sections(&no_digest)
            .iter()
            .any(|e| e.contains("must name the sha256")));

        let no_red = GOOD_BODY.replace("the new cases must fail.", "check the result.");
        assert!(validate_ghan_sections(&no_red)
            .iter()
            .any(|e| e.contains("must require the gate to go red")));
    }

    #[test]
    fn never_needs_an_addressee_line_before_its_lists() {
        let body = GOOD_BODY.replace(
            "The addressee of these limits is the agent executing this work item, not the dispatcher.",
            "- no addressee, straight to a list",
        );
        assert!(validate_ghan_sections(&body)
            .iter()
            .any(|e| e.contains("fixing the addressee")));
    }

    #[test]
    fn a_change_point_cannot_also_be_must_not_touch() {
        let body = GOOD_BODY.replace(
            "- `apps/agentic-workflow/external-contracts/src/wi_contract_fixture.py`",
            "- `apps/agentic-workflow/src/issues/ghan.rs`",
        );
        assert!(validate_ghan_sections(&body)
            .iter()
            .any(|e| e.contains("both a change point and must-not-touch")));
    }

    #[test]
    fn foreign_h2_is_refused_so_the_four_sections_stay_the_contract() {
        let body = format!("{GOOD_BODY}\n## Notes\n\nanything\n");
        assert!(validate_ghan_sections(&body)
            .iter()
            .any(|e| e.contains("unexpected H2 `## Notes`")));
    }

    #[test]
    fn fenced_headings_do_not_split_sections() {
        let body = GOOD_BODY.replace(
            "### Negative control\n",
            "### Negative control\n\n```sh\n## not a heading\n```\n",
        );
        assert_eq!(body_shape(&body), WiBodyShape::Ghan);
        assert_eq!(validate_ghan_sections(&body), Vec::<String>::new());
    }

    #[test]
    fn table_rows_skips_header_and_separator() {
        let rows = table_rows("| a | b |\n|---|---|\n| 1 | 2 |\n");
        assert_eq!(rows, vec![vec!["1".to_string(), "2".to_string()]]);
    }

    #[test]
    fn path_and_file_line_helpers_reject_non_coordinates() {
        assert!(file_line_ref("see `src/cli/issues.rs:2176` there").is_some());
        assert!(file_line_ref("see issue #3358").is_none());
        assert!(path_ref("touch `src/issues/ghan.rs`").is_some());
        assert!(path_ref("touch the validator").is_none());
        assert_eq!(normalize_path("`src/cli/issues.rs:2176`"), "src/cli/issues.rs");
    }
}
