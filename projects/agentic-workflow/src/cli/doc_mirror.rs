// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/doc_mirror.md#source
// CODEGEN-BEGIN
//! Shared whitelist definition for the root `CLAUDE.md` / `AGENTS.md` mirror
//! contract (issue #984, init-projector slice 1/3).
//!
//! `AGENTS.md` is `CLAUDE.md` plus a small, fixed set of Codex-only
//! insertions: a title-line swap, the `## Codex Operational Rules` section
//! (outside the `aw:start`/`aw:end` managed block), and a slash-command
//! translation paragraph (inside the managed block). This module is the ONE
//! definition of that whitelist, consumed by both `aw init`'s AGENTS.md
//! projection (`crate::cli::init`) and `root_doc_mirror_test`, so the
//! projector and the checker can never disagree (issue #984 AC3).
//!
//! Only [`CODEX_TRANSLATE_PREFIX`], [`CODEX_TRANSLATE_PARAGRAPH`],
//! [`CODEX_TRANSLATE_ANCHOR`], and [`agents_block_from_claude_block`] are
//! actually consumed by `aw init`'s projection: the title-line and
//! `## Codex Operational Rules` constants describe content that sits
//! OUTSIDE the `aw:start` block, so init's block-replace never touches it —
//! those two are relevant only to the mirror test's outside-block
//! normalization.

/// CLAUDE.md's title line.
pub const CLAUDE_TITLE: &str = "# CLAUDE.md - Implementation Essentials";

/// AGENTS.md's title line — the only whitelisted title-line divergence.
pub const AGENTS_TITLE: &str = "# AGENTS.md - Implementation Essentials";

/// Heading of the Codex-only operational-rules section. Lives OUTSIDE the
/// `aw:start`/`aw:end` block, so `aw init`'s block-replace never touches it.
pub const CODEX_RULES_HEADING: &str = "## Codex Operational Rules";

/// First line of the Codex-only slash-command translation paragraph. Lives
/// INSIDE the `aw:start` block, so `aw init` inserts it when projecting
/// AGENTS.md.
pub const CODEX_TRANSLATE_PREFIX: &str = "Codex should translate Claude slash-command references";

/// Full text of the slash-command translation paragraph (no leading/trailing
/// blank line), inserted immediately before [`CODEX_TRANSLATE_ANCHOR`].
pub const CODEX_TRANSLATE_PARAGRAPH: &str = "Codex should translate Claude slash-command references such as `/aw:td` or\n`/aw:wi` to the equivalent `aw ...` CLI command unless the user\nexplicitly asks for Claude-specific behavior.";

/// The line the translate paragraph is inserted directly before, inside the
/// `aw:start` block (immediately after the self-AW carve-out paragraph, per
/// the current CLAUDE.md/AGENTS.md content).
pub const CODEX_TRANSLATE_ANCHOR: &str = "### Workflow CLI";

/// Project the Codex-only slash-command translation paragraph into a
/// CLAUDE.md `aw:start` block's content, producing the AGENTS.md block.
///
/// `aw init` calls this on the template-derived CLAUDE.md section to build
/// the section it upserts into AGENTS.md, so the two docs' managed blocks
/// can never drift apart by hand-editing.
///
/// # Panics
///
/// Panics if `block` does not contain [`CODEX_TRANSLATE_ANCHOR`] — the
/// `aw:start` block content is compiled from
/// `templates/cli/mainthread/CLAUDE.md`, so a missing anchor is a template
/// authoring defect, not a runtime input failure.
pub fn agents_block_from_claude_block(block: &str) -> String {
    let anchor_at = block
        .find(CODEX_TRANSLATE_ANCHOR)
        .expect("aw:start block must contain the `### Workflow CLI` heading");
    format!(
        "{}{}\n\n{}",
        &block[..anchor_at],
        CODEX_TRANSLATE_PARAGRAPH,
        &block[anchor_at..]
    )
}

// ---------------------------------------------------------------------------
// CLI tables (issue #985, init-projector slice 2/3)
// ---------------------------------------------------------------------------

/// Top-level `Commands` verbs rendered into the generated "Workflow CLI"
/// table inside `aw:start` — the delivery-lifecycle nouns (work-item,
/// tech-design, external-contract, capability, health, and existing-project
/// takeover). Everything else agent-facing lands in [`SUPPORT_TABLE_VERBS`].
/// A static allowlist, not a clap `is_hide_set()` filter: no top-level verb
/// is actually clap-hidden today, so hide-based membership would leave most
/// verbs (`new`, `generator`, `sync`, `report-issue`, ...) unclassified by
/// either table instead of giving every relevant verb one home.
pub const WORKFLOW_TABLE_VERBS: &[&str] =
    &["wi", "capability", "td", "ec", "health", "standardize"];

/// Top-level `Commands` verbs rendered into the generated "Support CLI"
/// table: the CLI-convention trio (`llm`/`upgrade`/`issue` — see "CLI
/// Convention: every CLI ships `llm`, `upgrade`, `issue`") plus the
/// remaining agent-support verbs.
pub const SUPPORT_TABLE_VERBS: &[&str] =
    &["init", "chat", "guard", "llm", "upgrade", "issue", "view"];

/// Marker pair around the generated Workflow CLI table, nested inside the
/// `aw:start`/`aw:end` managed block.
pub const CLI_TABLE_WORKFLOW_START: &str = "<!-- aw:cli-table:workflow:start -->";
pub const CLI_TABLE_WORKFLOW_END: &str = "<!-- aw:cli-table:workflow:end -->";

/// Marker pair around the generated Support CLI table, nested inside the
/// `aw:start`/`aw:end` managed block.
pub const CLI_TABLE_SUPPORT_START: &str = "<!-- aw:cli-table:support:start -->";
pub const CLI_TABLE_SUPPORT_END: &str = "<!-- aw:cli-table:support:end -->";

/// Marker pair around the repo-root README's generated Projects table.
/// Standalone (not nested inside `aw:start`) since it lives in README.md,
/// not CLAUDE.md/AGENTS.md.
pub const PROJECTS_TABLE_START: &str = "<!-- aw:projects-table:start -->";
pub const PROJECTS_TABLE_END: &str = "<!-- aw:projects-table:end -->";

/// The full top-level `aw` clap command tree, built via
/// `Subcommand::augment_subcommands` — the same pattern already used by
/// `llm::registered_verbs` and `standardize::TraceabilityCli` — so the CLI
/// tables can never drift from `aw --help`.
fn top_level_command() -> clap::Command {
    <crate::cli::Commands as clap::Subcommand>::augment_subcommands(clap::Command::new("aw"))
}

/// Render a lean `| Verb | About |` Markdown table for `verbs` against
/// `root`'s subcommands, sourcing each row's one-liner from that
/// subcommand's clap `about` string (issue #985 design decision: `about` is
/// already agent-facing prose kept current by `--help`, so reusing it means
/// the table can never drift from `--help` the way a hand-duplicated cell
/// could — improve the `about` string on the command definition itself if a
/// row ever reads too thin).
///
/// Split from [`render_verb_table`] so tests can exercise it against a
/// synthetic `clap::Command` tree without depending on the real `aw` CLI
/// surface.
///
/// # Panics
///
/// Panics if `verbs` names a subcommand that does not exist on `root`, or
/// one that has no `about` string — both are allowlist/command authoring
/// defects, not runtime input failures.
fn render_verb_table_from(root: &clap::Command, verbs: &[&str]) -> String {
    let mut out = String::from("| Verb | About |\n|------|-------|\n");
    for verb in verbs {
        let sub = root
            .get_subcommands()
            .find(|c| c.get_name() == *verb)
            .unwrap_or_else(|| panic!("`aw {verb}` must exist on the top-level clap tree"));
        let about = sub
            .get_about()
            .unwrap_or_else(|| panic!("`aw {verb}` must have a clap `about` string"));
        out.push_str(&format!("| `aw {verb}` | {about} |\n"));
    }
    out
}

/// Render the generated Workflow/Support CLI table for `verbs` against the
/// real top-level `aw` clap tree.
pub fn render_verb_table(verbs: &[&str]) -> String {
    render_verb_table_from(&top_level_command(), verbs)
}

/// Splice `inner` between `start_marker` and `end_marker` inside `text`,
/// keeping both markers in place. Shared by the CLI-table and
/// Projects-table projections (issue #985) so every fine-grained generated
/// block upserts identically.
///
/// # Panics
///
/// Panics if `text` does not contain `start_marker` followed by
/// `end_marker` — a template/document authoring defect, not a runtime input
/// failure (mirrors [`agents_block_from_claude_block`]'s panic contract).
fn replace_between_markers(
    text: &str,
    start_marker: &str,
    end_marker: &str,
    inner: &str,
) -> String {
    let start_at = text
        .find(start_marker)
        .unwrap_or_else(|| panic!("document must contain `{start_marker}`"));
    let after_start = start_at + start_marker.len();
    let end_at = text[after_start..]
        .find(end_marker)
        .map(|off| after_start + off)
        .unwrap_or_else(|| panic!("document must contain `{end_marker}` after `{start_marker}`"));
    format!(
        "{}{}\n{}\n{}",
        &text[..start_at],
        start_marker,
        inner.trim_end(),
        &text[end_at..],
    )
}

/// Regenerate both fine-grained CLI tables inside a CLAUDE.md/AGENTS.md
/// `aw:start` section (issue #985). Runs BEFORE `aw init`'s existing
/// whole-block diff/upsert/staleness machinery ever sees the section text,
/// so table drift shows up as ordinary managed-section drift with zero new
/// detection code (`aw init --check` already covers it for free).
///
/// # Panics
///
/// Panics if `section` is missing either marker pair (template authoring
/// defect) or the real `aw` clap tree is missing an allowlisted verb.
pub fn render_cli_tables(section: &str) -> String {
    let with_workflow = replace_between_markers(
        section,
        CLI_TABLE_WORKFLOW_START,
        CLI_TABLE_WORKFLOW_END,
        &render_verb_table(WORKFLOW_TABLE_VERBS),
    );
    replace_between_markers(
        &with_workflow,
        CLI_TABLE_SUPPORT_START,
        CLI_TABLE_SUPPORT_END,
        &render_verb_table(SUPPORT_TABLE_VERBS),
    )
}

// ---------------------------------------------------------------------------
// Skill-tree projection (issue #986, init-projector slice 3/3)
// ---------------------------------------------------------------------------

/// Literal-substring rewrites applied when projecting a templates-authored
/// `aw-*` `SKILL.md` body (the `.claude/skills/` install source) into the
/// sibling `.agents/skills/` tree. Established by diffing all 16 `aw-*`
/// skills across `templates/cli/mainthread/skills/`, `.claude/skills/`, and
/// `.agents/skills/` (issue #986): every real content difference between the
/// `.claude` and `.agents` copies reduces to exactly these two self-
/// referencing literal swaps — a skill's own script-invocation path
/// (`.claude/skills/...` → `.agents/skills/...`, needed by
/// aw-build-debug/aw-build-release/aw-mamba-test-coverage) and a doc
/// cross-reference (`CLAUDE.md` → `AGENTS.md`, needed by aw-cb-fill/aw-wi).
/// Companion `scripts/*.sh` files need no transform (verified: zero
/// `.claude`/`CLAUDE` literal references in any of the 4 scripts), so only
/// `SKILL.md` bodies are run through this whitelist.
const SKILL_TREE_LITERAL_SWAPS: &[(&str, &str)] = &[
    (".claude/skills/", ".agents/skills/"),
    ("CLAUDE.md", "AGENTS.md"),
];

/// Project a `.claude/skills/<name>/SKILL.md` body into the body `aw init`
/// installs at `.agents/skills/<name>/SKILL.md`, applying
/// [`SKILL_TREE_LITERAL_SWAPS`] in order. Consumed by both `aw init`'s
/// `.agents/skills` installer (`crate::cli::init::install_agents_skills`) and
/// its staleness check, so the two can never disagree (issue #986 AC3, same
/// shared-whitelist pattern as [`agents_block_from_claude_block`]).
pub fn agents_skill_body_from_claude_skill_body(body: &str) -> String {
    let mut out = body.to_string();
    for (from, to) in SKILL_TREE_LITERAL_SWAPS {
        out = out.replace(from, to);
    }
    out
}

// ---------------------------------------------------------------------------
// Repo-root Projects table (issue #985, init-projector slice 2/3)
// ---------------------------------------------------------------------------

/// One row of the repo-root README's generated Projects table: a top-level
/// `projects/<name>` entry sourced from `.aw/config.toml`'s `[[projects]]`
/// registry.
struct ConfigProjectRow {
    name: String,
    path: String,
}

/// The TOML value of a `key = "value"` line, given the text right after
/// `key`. Returns `None` when `after_key` doesn't start with `=` — including
/// when `after_key` actually came from a different key that merely shares
/// `key` as a textual prefix (e.g. matching `"name"` against a
/// `named_thing = ...` line leaves `"d_thing = ..."`, which does not start
/// with `=`).
fn strip_toml_string_value(after_key: &str) -> Option<String> {
    let val = after_key.trim().strip_prefix('=')?.trim();
    let val = val.trim_matches('"').trim_matches('\'');
    (!val.is_empty()).then(|| val.to_string())
}

/// True if `path` is a direct `projects/<single-segment>` entry — the root
/// Projects table lists top-level projects only, not nested library crates
/// (`projects/mamba/mambalibs/...`) or `crates/`/`libs/` entries.
fn is_top_level_project_path(path: &str) -> bool {
    match path.strip_prefix("projects/") {
        Some(rest) => !rest.is_empty() && !rest.contains('/'),
        None => false,
    }
}

/// Scan `.aw/config.toml`'s text for every top-level `[[projects]]` entry —
/// both the hand-written registry and the auto-generated `# BEGIN/END AW
/// SYNC` block, since the Projects table's source of truth is "registered
/// in config.toml", not which section registered it. Nested array tables
/// such as `[[projects.workspaces]]` are skipped: only a literal
/// `[[projects]]` header line opens a top-level entry. When a name is
/// registered more than once (a hand-written entry and its `AW SYNC`
/// duplicate), the first occurrence in file order wins, so the hand-written
/// section always takes priority.
fn parse_top_level_project_rows(config_text: &str) -> Vec<ConfigProjectRow> {
    let mut rows = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut in_projects_table = false;
    let mut current_name: Option<String> = None;

    for line in config_text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_projects_table = trimmed == "[[projects]]";
            current_name = None;
            continue;
        }
        if !in_projects_table {
            continue;
        }
        if let Some(name) = trimmed
            .strip_prefix("name")
            .and_then(strip_toml_string_value)
        {
            current_name = Some(name);
        } else if let Some(path) = trimmed
            .strip_prefix("path")
            .and_then(strip_toml_string_value)
        {
            if let Some(name) = current_name.clone() {
                if is_top_level_project_path(&path) && seen.insert(name.clone()) {
                    rows.push(ConfigProjectRow { name, path });
                }
            }
        }
    }
    rows
}

/// Extract the first full sentence of a project README's `## Brief` section
/// as the Projects table's one-liner source (issue #985 design decision:
/// the project's own `## Brief`, not a duplicated description field in
/// config.toml, so the two can never drift). Soft-wrapped lines are joined
/// before sentence-splitting so a paragraph that wraps mid-sentence in the
/// source Markdown never truncates mid-word.
fn first_brief_sentence(readme_text: &str) -> Option<String> {
    let heading = "## Brief";
    let after_heading = &readme_text[readme_text.find(heading)? + heading.len()..];
    let paragraph = after_heading
        .lines()
        .skip_while(|l| l.trim().is_empty())
        .take_while(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let joined = paragraph.split_whitespace().collect::<Vec<_>>().join(" ");
    let cleaned = joined.replace("**", "");
    if cleaned.is_empty() {
        return None;
    }
    Some(match cleaned.find(". ") {
        Some(idx) => cleaned[..=idx].to_string(),
        None => cleaned,
    })
}

/// Render the repo-root README's generated Projects table from
/// `<project_root>/.aw/config.toml`'s top-level `[[projects]]` registry, one
/// row per entry in scan order, with each row's one-liner sourced from that
/// project's own `README.md` `## Brief` first sentence (see
/// [`first_brief_sentence`]).
///
/// # Errors
///
/// Returns an error if `.aw/config.toml` cannot be read, or if any resolved
/// project's `README.md` cannot be read or has no `## Brief` sentence — both
/// indicate a genuinely broken registry entry, not optional input.
pub fn render_projects_table(project_root: &std::path::Path) -> anyhow::Result<String> {
    let config_text = std::fs::read_to_string(project_root.join(".aw/config.toml"))?;
    let rows = parse_top_level_project_rows(&config_text);

    let mut out = String::from("| Project | What it is |\n|---------|------------|\n");
    for row in rows {
        let readme_path = project_root.join(&row.path).join("README.md");
        let readme_text = std::fs::read_to_string(&readme_path).map_err(|e| {
            anyhow::anyhow!(
                "Projects table row `{}`: reading {}: {e}",
                row.name,
                readme_path.display()
            )
        })?;
        let brief = first_brief_sentence(&readme_text).ok_or_else(|| {
            anyhow::anyhow!(
                "Projects table row `{}`: {} has no `## Brief` sentence",
                row.name,
                readme_path.display()
            )
        })?;
        out.push_str(&format!(
            "| [{}]({}/README.md) | {} |\n",
            row.name, row.path, brief
        ));
    }
    Ok(out)
}

/// Regenerate `doc_text`'s Projects table between
/// [`PROJECTS_TABLE_START`]/[`PROJECTS_TABLE_END`] from
/// `<project_root>/.aw/config.toml` (issue #985). Callers gate on the
/// markers already being present in `doc_text` — the table is opt-in per
/// document (see `aw init`'s README projection, which only touches
/// README.md when it already carries the markers).
pub fn upsert_projects_table(
    project_root: &std::path::Path,
    doc_text: &str,
) -> anyhow::Result<String> {
    let table = render_projects_table(project_root)?;
    Ok(replace_between_markers(
        doc_text,
        PROJECTS_TABLE_START,
        PROJECTS_TABLE_END,
        &table,
    ))
}

// ---------------------------------------------------------------------------
// Trait registry (issue #1077 slice 1/3, extended by #1078 slice 2/3: new
// anchored traits, the migrated general traits, and the `service` umbrella)
// ---------------------------------------------------------------------------

/// One `[capability.profile].traits` id's data: the baseline capabilities it
/// derives, the CONTRIBUTING.md "Service archetype" heading it enforces, and
/// an agent-facing one-liner. Replaces the hand-maintained match family in
/// `crate::cli::capability` for traits that already have a settled
/// CONTRIBUTING.md doc home; [`render_trait_table`] projects this table's
/// rows into CONTRIBUTING.md's generated `<!-- aw:trait-table:start/end -->`
/// block so the archetype's doc side stops being prose-only.
///
/// `contributing_anchor` stores the exact heading line (including the `#`
/// prefix, at whatever level the section actually is — most are `### `
/// archetype subsections, but `cli_std`/`chainable_output` anchor to `## `
/// top-level CLI-convention sections) rather than a precomputed slug, so
/// both `render_trait_table` (which derives the link's href from it) and
/// `root_trait_coverage_test` (which resolves it against a live heading scan
/// of CONTRIBUTING.md) share one robust, literal-match representation that
/// can never fall out of sync with a hand-edited heading the way a
/// duplicated slug string could.
///
/// `contributing_anchor` is `None` for traits with no settled CONTRIBUTING.md
/// doc anchor (issue #1078, archetype-as-traits slice 2/3: `cli_facing`,
/// `competitive_replacement`, `long_running`, `network_exposed`,
/// `agent_facing`, `stateful_storage` — carried over from
/// `capability::other_known_trait_baseline_caps`, now folded into this one
/// registry with no anchor rather than a second lookup table). Anchored
/// traits render a working `Enforces` link and participate in
/// `root_trait_coverage_test`'s archetype-H3 coverage direction; anchor-less
/// traits render `—` in that column and are exempt from the H3-coverage
/// direction (there is no heading for them to cover), but still participate
/// in the "every `Some` anchor resolves to a real heading" direction.
// `pub`, not `pub(crate)`: the bidirectional coverage test
// (`root_trait_coverage_test`, issue #1077 AC3) lives in `tests/` and
// compiles against this crate as an external dependent, so it can only see
// `pub` items — matching every other shared-projector constant in this
// module (`TRAIT_TABLE_START`, `PROJECTS_TABLE_START`, ...).
pub struct TraitDef {
    pub id: &'static str,
    pub baseline_caps: &'static [&'static str],
    pub contributing_anchor: Option<&'static str>,
    pub about: &'static str,
}

/// The full known-trait registry (archetype-anchored and general), in
/// CONTRIBUTING.md trait-table row order.
pub const TRAITS: &[TraitDef] = &[
    TraitDef {
        id: "http2_api",
        baseline_caps: &["http2-api-list"],
        contributing_anchor: Some("### Transport — h2c + OpenAPI on one port"),
        about: "Project owes a public HTTP/2 (h2c) + OpenAPI transport baseline, not full OpenAPI completeness.",
    },
    TraitDef {
        id: "kubernetes_native",
        baseline_caps: &["kubernetes-native-deployment"],
        contributing_anchor: Some("### Deploy artifacts — image, cluster API, operator, instance"),
        about: "Project owes a Kubernetes-native deployment baseline: image, cluster API, operator, instance.",
    },
    TraitDef {
        id: "primary_replicas",
        baseline_caps: &["primary-replicas"],
        contributing_anchor: Some("### HA — `raft-core`, sharded and strongly consistent"),
        about: "Project owes a primary/replica HA topology baseline; select only for projects that actually support that topology.",
    },
    TraitDef {
        id: "standard_endpoints",
        baseline_caps: &["standard-operational-endpoints"],
        contributing_anchor: Some(
            "### Standard endpoints — one operational surface, one contract three ways",
        ),
        about: "Project owes the standard /healthz, /readyz, /metrics, /openapi.json, /docs operational surface on one port.",
    },
    TraitDef {
        id: "ec_gated",
        baseline_caps: &["ec-gates-configured"],
        contributing_anchor: Some(
            "### EC gates — `vat`-driven, evidence under `external-contracts/`",
        ),
        about: "Project owes vat-driven EC gate files (vat.toml/meter*.toml/guard*.toml) with evidence under external-contracts/.",
    },
    TraitDef {
        id: "cli_std",
        baseline_caps: &["cli-standard-surface"],
        contributing_anchor: Some("## CLI convention: every CLI ships `llm`, `upgrade`, `issue`"),
        about: "Project owes the mandatory llm/upgrade/issue CLI surface every tool in the ecosystem ships.",
    },
    TraitDef {
        id: "chainable_output",
        baseline_caps: &["chainable-output-conformance"],
        contributing_anchor: Some("## CLI convention: stdout tells the agent the next step"),
        about: "Project owes chainable stdout: every structured/terminal output carries a runnable next command or an explicit terminal marker.",
    },
    TraitDef {
        id: "cli_facing",
        baseline_caps: &["cli-interface"],
        contributing_anchor: None,
        about: "Project is primarily driven through a CLI surface; no settled CONTRIBUTING.md doc home yet.",
    },
    TraitDef {
        id: "competitive_replacement",
        baseline_caps: &["competitor-feature-parity", "competitor-performance"],
        contributing_anchor: None,
        about: "Project aims to replace or match an existing competitor tool; no settled CONTRIBUTING.md doc home yet.",
    },
    TraitDef {
        id: "long_running",
        baseline_caps: &["long-running-stability"],
        contributing_anchor: None,
        about: "Project runs as a long-lived process; no settled CONTRIBUTING.md doc home yet.",
    },
    TraitDef {
        id: "network_exposed",
        baseline_caps: &["security-hardening"],
        contributing_anchor: None,
        about: "Project exposes a network-reachable surface; no settled CONTRIBUTING.md doc home yet.",
    },
    TraitDef {
        id: "agent_facing",
        baseline_caps: &[],
        contributing_anchor: None,
        about: "Project is primarily driven by agents rather than humans; prompt-only, no enforced baseline capability yet.",
    },
    TraitDef {
        id: "stateful_storage",
        baseline_caps: &[],
        contributing_anchor: None,
        about: "Project owns durable stateful storage; prompt-only, no enforced baseline capability yet.",
    },
];

/// One `[capability.profile].traits` umbrella id's expansion: declaring it
/// is equivalent to declaring every id in `members` (issue #1078,
/// archetype-as-traits slice 2/3). Not a `TraitDef` itself — an umbrella
/// derives no baseline caps of its own, only via its members — so it gets
/// its own small table rather than a fake anchor-less `TraitDef` row.
pub struct TraitExpansion {
    pub id: &'static str,
    pub members: &'static [&'static str],
    pub about: &'static str,
}

/// The umbrella-trait expansion registry. `service` is the full
/// service-archetype adopter: every trait with a settled CONTRIBUTING.md
/// "Service archetype" or "CLI convention" doc home, excluding
/// `primary_replicas` (topology-dependent, stays opt-in per the archetype's
/// own HA row -- not every service needs sharded HA).
pub const TRAIT_EXPANSIONS: &[TraitExpansion] = &[TraitExpansion {
    id: "service",
    members: &[
        "http2_api",
        "kubernetes_native",
        "standard_endpoints",
        "ec_gated",
        "cli_std",
        "chainable_output",
    ],
    about: "Umbrella for a full service-archetype adopter; expands to the transport, deploy, operational, EC-gate, and CLI baseline traits, deduped against any of its members also declared directly.",
}];

/// Expand `traits` into the trait-id set actually used for baseline-cap
/// derivation (issue #1078 AC1): every umbrella id in [`TRAIT_EXPANSIONS`] is
/// replaced by its `members`, and everything else passes through unchanged.
/// A `BTreeSet` so an umbrella declared alongside one of its own members
/// (`["service", "http2_api"]`) collapses to one entry — "umbrella+member
/// double-declaration doesn't duplicate".
pub fn expand_capability_profile_traits(traits: &[String]) -> std::collections::BTreeSet<String> {
    let mut expanded = std::collections::BTreeSet::new();
    for trait_id in traits {
        match TRAIT_EXPANSIONS.iter().find(|exp| exp.id == trait_id) {
            Some(expansion) => {
                for member in expansion.members {
                    expanded.insert((*member).to_string());
                }
            }
            None => {
                expanded.insert(trait_id.clone());
            }
        }
    }
    expanded
}

/// Marker pair around the generated CONTRIBUTING.md trait table, placed in
/// the "Service archetype" section right after its intro paragraph/table and
/// before the first H3.
pub const TRAIT_TABLE_START: &str = "<!-- aw:trait-table:start -->";
pub const TRAIT_TABLE_END: &str = "<!-- aw:trait-table:end -->";

/// A best-effort Markdown heading slug for same-document anchor links:
/// lower-case, ASCII-alphanumeric runs kept, every other character (including
/// the CONTRIBUTING.md headings' em dashes, backticks, and commas) collapsed
/// to a single `-`, with leading/trailing `-` trimmed. Not guaranteed to
/// reproduce every renderer's exact slug algorithm (GitHub's, for one, keeps
/// internal hyphens as single hyphens too, so the two happen to agree here),
/// but deterministic and shared by [`render_trait_table`] alone — the
/// bidirectional coverage test resolves anchors via the literal heading text
/// in [`TraitDef::contributing_anchor`], not this slug.
fn slugify_heading(heading_text: &str) -> String {
    let mut slug = String::new();
    let mut prev_was_sep = true;
    for ch in heading_text.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_was_sep = false;
        } else if !prev_was_sep {
            slug.push('-');
            prev_was_sep = true;
        }
    }
    slug.trim_end_matches('-').to_string()
}

/// Render the generated `| Trait | Derives | Enforces | About |` table from
/// [`TRAITS`] plus [`TRAIT_EXPANSIONS`] (issue #1077, extended #1078).
/// `Enforces` is a same-document Markdown link to the trait's
/// `contributing_anchor` heading when it has one, or `—` for general traits
/// with no settled CONTRIBUTING.md doc home. Umbrella rows (from
/// `TRAIT_EXPANSIONS`) render `Derives` as `expands: <member ids>` and
/// `Enforces` as `—` — the umbrella has no anchor of its own, only via its
/// members' own rows.
pub fn render_trait_table() -> String {
    let mut out = String::from("| Trait | Derives | Enforces | About |\n|---|---|---|---|\n");
    for def in TRAITS {
        let derives = def
            .baseline_caps
            .iter()
            .map(|cap| format!("`{cap}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let enforces = match def.contributing_anchor {
            Some(anchor) => {
                let heading_text = anchor.trim_start_matches('#').trim_start();
                let slug = slugify_heading(heading_text);
                format!("[{heading_text}](#{slug})")
            }
            None => "—".to_string(),
        };
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            def.id, derives, enforces, def.about
        ));
    }
    for expansion in TRAIT_EXPANSIONS {
        let derives = format!(
            "expands: {}",
            expansion
                .members
                .iter()
                .map(|member| format!("`{member}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        out.push_str(&format!(
            "| `{}` | {} | — | {} |\n",
            expansion.id, derives, expansion.about
        ));
    }
    out
}

/// Regenerate the repo-root CONTRIBUTING.md's trait table between
/// [`TRAIT_TABLE_START`]/[`TRAIT_TABLE_END`] (issue #1077). Callers gate on
/// the markers already being present in `doc_text`, mirroring
/// [`upsert_projects_table`]'s opt-in-per-document contract.
pub fn upsert_trait_table(doc_text: &str) -> String {
    replace_between_markers(
        doc_text,
        TRAIT_TABLE_START,
        TRAIT_TABLE_END,
        &render_trait_table(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agents_block_from_claude_block_inserts_paragraph_before_anchor() {
        let block = "<!-- aw:start -->\nfoo\n\n### Workflow CLI\nbar\n<!-- aw:end -->";
        let out = agents_block_from_claude_block(block);

        let para_pos = out
            .find(CODEX_TRANSLATE_PREFIX)
            .expect("translate paragraph must be present");
        let anchor_pos = out
            .find(CODEX_TRANSLATE_ANCHOR)
            .expect("anchor must still be present");
        assert!(
            para_pos < anchor_pos,
            "translate paragraph must precede the Workflow CLI anchor"
        );
        assert!(out.starts_with("<!-- aw:start -->\nfoo\n\n"));
        assert!(out.ends_with("bar\n<!-- aw:end -->"));
    }

    #[test]
    #[should_panic(expected = "Workflow CLI")]
    fn agents_block_from_claude_block_panics_without_anchor() {
        let _ =
            agents_block_from_claude_block("<!-- aw:start -->\nno anchor here\n<!-- aw:end -->");
    }

    // -- CLI tables (issue #985) --------------------------------------------

    #[test]
    fn render_verb_table_from_synthetic_tree_uses_about_strings() {
        let root = clap::Command::new("aw")
            .subcommand(clap::Command::new("foo").about("Foo thing."))
            .subcommand(clap::Command::new("bar").about("Bar thing."));
        let table = render_verb_table_from(&root, &["foo", "bar"]);
        assert!(table.starts_with("| Verb | About |\n|------|-------|\n"));
        assert!(table.contains("| `aw foo` | Foo thing. |"));
        assert!(table.contains("| `aw bar` | Bar thing. |"));
    }

    #[test]
    #[should_panic(expected = "must exist on the top-level clap tree")]
    fn render_verb_table_from_panics_on_missing_verb() {
        let root = clap::Command::new("aw");
        let _ = render_verb_table_from(&root, &["missing"]);
    }

    #[test]
    fn render_cli_tables_splices_both_marker_pairs_and_drops_stale_rows() {
        let section = format!(
            "intro\n\n### Workflow CLI\n\n{}\nold workflow row\n{}\n\n### Support CLI\n\n{}\nold support row\n{}\n",
            CLI_TABLE_WORKFLOW_START, CLI_TABLE_WORKFLOW_END, CLI_TABLE_SUPPORT_START,
            CLI_TABLE_SUPPORT_END
        );
        let rendered = render_cli_tables(&section);
        assert!(!rendered.contains("old workflow row"));
        assert!(!rendered.contains("old support row"));
        assert!(rendered.contains("| `aw wi` |"));
        assert!(rendered.contains("| `aw init` |"));
        assert!(rendered.contains(CLI_TABLE_WORKFLOW_START));
        assert!(rendered.contains(CLI_TABLE_SUPPORT_END));
    }

    // -- Skill-tree projection (issue #986) ---------------------------------

    #[test]
    fn agents_skill_body_swaps_self_referencing_script_path() {
        let claude_body = "Run:\n\n```bash\n.claude/skills/aw-build-debug/scripts/build.sh\n```\n";
        let agents_body = agents_skill_body_from_claude_skill_body(claude_body);
        assert!(agents_body.contains(".agents/skills/aw-build-debug/scripts/build.sh"));
        assert!(!agents_body.contains(".claude/skills/"));
    }

    #[test]
    fn agents_skill_body_swaps_claude_md_doc_reference() {
        let claude_body = "See `CLAUDE.md § AW envelope (mainthread protocol)`.";
        let agents_body = agents_skill_body_from_claude_skill_body(claude_body);
        assert_eq!(
            agents_body,
            "See `AGENTS.md § AW envelope (mainthread protocol)`."
        );
    }

    #[test]
    fn agents_skill_body_is_identity_without_whitelisted_literals() {
        let claude_body = "# /aw:health\n\nNo tree-specific literals here.\n";
        assert_eq!(
            agents_skill_body_from_claude_skill_body(claude_body),
            claude_body
        );
    }

    // -- Repo-root Projects table (issue #985) ------------------------------

    #[test]
    fn first_brief_sentence_joins_wrapped_lines_and_stops_at_first_period() {
        let readme = "# demo\n\n## Brief\n\nThis is a wrapped\nsentence that spans two lines. It has more.\n\n## Next\n";
        assert_eq!(
            first_brief_sentence(readme).as_deref(),
            Some("This is a wrapped sentence that spans two lines.")
        );
    }

    #[test]
    fn first_brief_sentence_returns_none_without_brief_heading() {
        assert_eq!(first_brief_sentence("# demo\n\nno brief here\n"), None);
    }

    #[test]
    fn parse_top_level_project_rows_dedupes_and_skips_nested_tables() {
        let config = r#"
[[projects]]
name = "aw"
path = "projects/aw"

[[projects.workspaces]]
name = "aw-workspace"
paths = ["projects/aw/**"]

[[projects]]
name = "nested-lib"
path = "projects/mamba/mambalibs/pgkit"

[[projects]]
name = "aw"
path = "projects/aw-duplicate"
"#;
        let rows = parse_top_level_project_rows(config);
        assert_eq!(
            rows.len(),
            1,
            "nested table + non-top-level path must be skipped"
        );
        assert_eq!(rows[0].name, "aw");
        assert_eq!(
            rows[0].path, "projects/aw",
            "first occurrence in file order must win over the duplicate"
        );
    }

    #[test]
    fn render_projects_table_reads_config_and_project_readmes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".aw")).unwrap();
        std::fs::write(
            root.join(".aw/config.toml"),
            "[[projects]]\nname = \"demo\"\npath = \"projects/demo\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("projects/demo")).unwrap();
        std::fs::write(
            root.join("projects/demo/README.md"),
            "# demo\n\n## Brief\n\nDemo project one-liner here.\n",
        )
        .unwrap();

        let table = render_projects_table(root).unwrap();
        assert!(table.contains("[demo](projects/demo/README.md)"));
        assert!(table.contains("Demo project one-liner here."));
    }

    #[test]
    fn upsert_projects_table_replaces_only_the_marked_region() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".aw")).unwrap();
        std::fs::write(
            root.join(".aw/config.toml"),
            "[[projects]]\nname = \"demo\"\npath = \"projects/demo\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("projects/demo")).unwrap();
        std::fs::write(
            root.join("projects/demo/README.md"),
            "# demo\n\n## Brief\n\nDemo one-liner.\n",
        )
        .unwrap();

        let doc = format!(
            "# axiom\n\n## Projects\n\n{}\nstale table\n{}\n\n## Install\nkeep me\n",
            PROJECTS_TABLE_START, PROJECTS_TABLE_END
        );
        let updated = upsert_projects_table(root, &doc).unwrap();
        assert!(!updated.contains("stale table"));
        assert!(updated.contains("[demo](projects/demo/README.md)"));
        assert!(updated.contains("## Install\nkeep me"));
    }

    // -- Trait registry (issue #1077, extended #1078) ------------------------

    #[test]
    fn slugify_heading_matches_hand_computed_contributing_anchors() {
        assert_eq!(
            slugify_heading("Transport — h2c + OpenAPI on one port"),
            "transport-h2c-openapi-on-one-port"
        );
        assert_eq!(
            slugify_heading("Deploy artifacts — image, cluster API, operator, instance"),
            "deploy-artifacts-image-cluster-api-operator-instance"
        );
        assert_eq!(
            slugify_heading("HA — `raft-core`, sharded and strongly consistent"),
            "ha-raft-core-sharded-and-strongly-consistent"
        );
    }

    #[test]
    fn render_trait_table_has_one_row_per_trait_def_with_working_links() {
        let table = render_trait_table();
        assert!(table.starts_with("| Trait | Derives | Enforces | About |\n|---|---|---|---|\n"));
        assert_eq!(
            table.lines().count(),
            2 + TRAITS.len() + TRAIT_EXPANSIONS.len()
        );
        for def in TRAITS {
            assert!(
                table.contains(&format!("| `{}` |", def.id)),
                "missing row for trait `{}`",
                def.id
            );
            for cap in def.baseline_caps {
                assert!(table.contains(&format!("`{cap}`")));
            }
            assert!(table.contains(def.about));
            match def.contributing_anchor {
                Some(anchor) => {
                    let heading_text = anchor.trim_start_matches('#').trim_start();
                    let slug = slugify_heading(heading_text);
                    assert!(
                        table.contains(&format!("[{heading_text}](#{slug})")),
                        "trait `{}` must render a working Enforces link",
                        def.id
                    );
                }
                None => {
                    assert!(
                        table.contains(&format!("| `{}` | {} | — |", def.id, {
                            def.baseline_caps
                                .iter()
                                .map(|cap| format!("`{cap}`"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        })),
                        "anchor-less trait `{}` must render `—` in Enforces",
                        def.id
                    );
                }
            }
        }
        for expansion in TRAIT_EXPANSIONS {
            assert!(table.contains(&format!("| `{}` |", expansion.id)));
            assert!(table.contains(&format!(
                "expands: {}",
                expansion
                    .members
                    .iter()
                    .map(|member| format!("`{member}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
            assert!(table.contains(expansion.about));
        }
    }

    #[test]
    fn upsert_trait_table_replaces_only_the_marked_region() {
        let doc = format!(
            "## Service archetype\n\nintro\n\n{}\nstale row\n{}\n\n### Next H3\n",
            TRAIT_TABLE_START, TRAIT_TABLE_END
        );
        let updated = upsert_trait_table(&doc);
        assert!(!updated.contains("stale row"));
        assert!(updated.contains("| `http2_api` |"));
        assert!(updated.contains("### Next H3"));
    }

    // -- Umbrella trait expansion (issue #1078) ------------------------------

    #[test]
    fn expand_capability_profile_traits_expands_umbrella_and_dedups_members() {
        let traits = vec!["service".to_string(), "http2_api".to_string()];
        let expanded = expand_capability_profile_traits(&traits);
        assert_eq!(
            expanded,
            [
                "chainable_output",
                "cli_std",
                "ec_gated",
                "http2_api",
                "kubernetes_native",
                "standard_endpoints",
            ]
            .into_iter()
            .map(str::to_string)
            .collect()
        );
    }

    #[test]
    fn expand_capability_profile_traits_passes_through_non_umbrella_ids() {
        let traits = vec!["primary_replicas".to_string(), "cli_facing".to_string()];
        let expanded = expand_capability_profile_traits(&traits);
        assert_eq!(
            expanded,
            ["cli_facing", "primary_replicas"]
                .into_iter()
                .map(str::to_string)
                .collect()
        );
    }
}
// CODEGEN-END
