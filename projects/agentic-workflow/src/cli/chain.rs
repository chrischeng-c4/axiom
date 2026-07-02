// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/chain.md#source
// CODEGEN-BEGIN
//! Validate an emitted `aw ...` next-command string against the real CLI
//! surface, and normalize a small set of legacy/stale persisted commands.
//!
//! aw's envelopes (`invoke.command` / `next.command`, and the loop-state
//! block's `next_action`) emit next-command strings as raw text templates
//! that nothing previously validated end-to-end:
//!
//!   - Bare `aw td code-check` passes clap (`CbCheckArgs.target` is
//!     `Option<String>`) but silently runs a whole-tree audit instead of the
//!     terminal check for the emitting WI (#844).
//!   - A persisted legacy string like `aw td merge` dispatches verbatim even
//!     though `aw td merge` was removed from the LINEAR lifecycle (#845).
//!
//! [`validate_aw_command_string`] closes the first gap: it re-parses a
//! command string through the *real* clap tree ([`super::standardize::TraceabilityCli`],
//! the same full `Commands` tree the binary itself dispatches through) and
//! then consults a small chain-policy table
//! ([`CHAIN_REQUIRED_POSITIONALS`]) for positionals that are clap-optional
//! but semantically required for correct dispatch. [`EMIT_REGISTRY`] is a
//! one-entry-per-emit-site catalogue of every place in the codebase that
//! builds one of these command strings; a unit test below walks every entry
//! through the validator, which is this slice's red-before/green-after
//! regression proof for #844/#845 (see epic #914 slice A / issue #915).
//!
//! [`normalize_legacy_next_action`] closes the second gap for the one
//! read-path this slice owns (`aw run`'s loop-state dispatch, see
//! `run.rs::loop_state_envelope`): it recognizes a small table of known
//! stale/legacy persisted strings and rewrites them to their LINEAR
//! equivalent, or returns `None` when the string cannot be repaired — the
//! caller must then surface a blocked/HITL envelope instead of dispatching
//! it verbatim.
//!
//! Scope note (#915 slice A only): this module validates and normalizes
//! *string* commands. It does not introduce a `NextCommand` newtype across
//! every emit site (slice C) and does not implement tier 1b/2 richer chain
//! policy or a health axis (slice G) — see epic #914 for the full sequence.

use clap::CommandFactory;

use super::run;
use super::standardize::TraceabilityCli;

/// Why [`validate_aw_command_string`] rejected a command string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainBlockerKind {
    /// The command string was empty (or whitespace-only) after trimming.
    EmptyCommand,
    /// The first whitespace-separated token was not `aw`.
    NotAwCommand,
    /// The real `aw` clap tree rejected the tokens (unknown verb at any
    /// depth, missing a clap-required argument, malformed flag, ...).
    ClapRejected,
    /// The command parsed cleanly under clap but is missing a positional
    /// that the chain-policy table ([`CHAIN_REQUIRED_POSITIONALS`]) marks as
    /// required for correct dispatch even though clap itself allows it to be
    /// absent.
    MissingChainRequiredPositional,
}

impl ChainBlockerKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ChainBlockerKind::EmptyCommand => "empty_command",
            ChainBlockerKind::NotAwCommand => "not_aw_command",
            ChainBlockerKind::ClapRejected => "clap_rejected",
            ChainBlockerKind::MissingChainRequiredPositional => "missing_chain_required_positional",
        }
    }
}

/// A chain-validation failure for one emitted command string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainBlocker {
    pub kind: ChainBlockerKind,
    pub command: String,
    pub reason: String,
}

impl ChainBlocker {
    fn new(kind: ChainBlockerKind, command: &str, reason: impl Into<String>) -> Self {
        Self {
            kind,
            command: command.to_string(),
            reason: reason.into(),
        }
    }
}

impl std::fmt::Display for ChainBlocker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "chain-invalid command `{}` ({}): {}",
            self.command,
            self.kind.as_str(),
            self.reason
        )
    }
}

/// One entry in the chain-policy table: a subcommand path (as clap kebab-case
/// names, e.g. `["td", "code-check"]`) plus the id of a positional/optional
/// argument that must be present on that leaf for the command to be safe to
/// dispatch, even though clap alone permits it to be absent.
struct ChainRequiredPositional {
    path: &'static [&'static str],
    arg_id: &'static str,
    note: &'static str,
}

/// #844: `aw td code-check`'s `target` (`CbCheckArgs.target: Option<String>`
/// in `cb.rs`) is clap-optional — clap happily parses a bare
/// `aw td code-check` — but a bare invocation silently runs a whole-tree
/// audit instead of the terminal check for the WI that emitted the command.
/// Every EMIT_REGISTRY producer must substitute a real target/slug here.
const CHAIN_REQUIRED_POSITIONALS: &[ChainRequiredPositional] = &[ChainRequiredPositional {
    path: &["td", "code-check"],
    arg_id: "target",
    note: "clap-optional (CbCheckArgs.target: Option<String>) but chain-required: a bare \
           `aw td code-check` runs a whole-tree audit instead of the terminal check for the \
           emitting WI (#844)",
}];

/// Validate one emitted `aw ...` next-command string.
///
/// Two passes:
///   1. Re-parse the tokens through the real clap tree
///      ([`TraceabilityCli`], the same `Commands` tree the `aw` binary
///      itself dispatches through) via `try_get_matches_from` — this
///      rejects unknown verbs/subcommands at any depth and malformed flags
///      without executing anything.
///   2. Consult [`CHAIN_REQUIRED_POSITIONALS`] for positionals that are
///      clap-optional but semantically required for correct dispatch.
///
/// Token splitting is plain `str::split_whitespace` (a documented
/// limitation, not a real shell/shlex split): no EMIT_REGISTRY template
/// today quotes a substituted value, so a naive split round-trips correctly
/// for the current surface. A real shlex split is tier 2 (#914 slice G) if
/// that stops being true.
pub fn validate_aw_command_string(cmd: &str) -> Result<(), ChainBlocker> {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return Err(ChainBlocker::new(
            ChainBlockerKind::EmptyCommand,
            cmd,
            "command string is empty",
        ));
    }
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.first() != Some(&"aw") {
        return Err(ChainBlocker::new(
            ChainBlockerKind::NotAwCommand,
            cmd,
            "command does not start with `aw`",
        ));
    }
    let matches = TraceabilityCli::command()
        .try_get_matches_from(tokens.iter().copied())
        .map_err(|err| {
            let reason = err
                .to_string()
                .lines()
                .next()
                .unwrap_or("clap rejected the command")
                .to_string();
            ChainBlocker::new(ChainBlockerKind::ClapRejected, cmd, reason)
        })?;
    check_chain_required_positionals(cmd, &matches)?;
    Ok(())
}

fn check_chain_required_positionals(
    cmd: &str,
    matches: &clap::ArgMatches,
) -> Result<(), ChainBlocker> {
    for req in CHAIN_REQUIRED_POSITIONALS {
        if let Some(sub) = descend_subcommand(matches, req.path) {
            if sub.get_one::<String>(req.arg_id).is_none() {
                return Err(ChainBlocker::new(
                    ChainBlockerKind::MissingChainRequiredPositional,
                    cmd,
                    format!(
                        "`aw {}` requires `{}`: {}",
                        req.path.join(" "),
                        req.arg_id,
                        req.note
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Walk `matches` down a subcommand path, returning the leaf `ArgMatches` if
/// the path matches exactly, or `None` if the command took a different
/// subcommand path.
fn descend_subcommand<'a>(
    matches: &'a clap::ArgMatches,
    path: &[&str],
) -> Option<&'a clap::ArgMatches> {
    let mut current = matches;
    for segment in path {
        let (name, sub) = current.subcommand()?;
        if name != *segment {
            return None;
        }
        current = sub;
    }
    Some(current)
}

/// One place in the codebase that builds an `aw ...` next-command string for
/// an emitted envelope or persisted loop-state field. `sample` is a
/// representative instantiation of that site's template (with placeholders
/// filled with realistic values) that [`validate_aw_command_string`] must
/// accept — this is the emit-site regression proof for #844/#845.
///
/// `#[cfg(test)]`-only for now: today's sole consumer is
/// `emit_registry_entries_are_all_chain_valid` below. A later slice (#914
/// health axis) may promote this to a runtime-readable registry.
#[cfg(test)]
struct EmitSite {
    /// `<file>:<function>` locating the producer, for triage.
    source: &'static str,
    /// A representative instantiation of the emitted template.
    sample: &'static str,
    #[allow(dead_code)]
    note: &'static str,
}

/// #915 AC1/AC3: one entry per `aw ...` command-string emit site. A unit
/// test below (`emit_registry_entries_are_all_chain_valid`) walks every
/// sample through [`validate_aw_command_string`] and fails on any blocker —
/// this was red (bare `aw td code-check` from the loop-state site) before
/// this slice's fix and is green after.
#[cfg(test)]
const EMIT_REGISTRY: &[EmitSite] = &[
    EmitSite {
        source: "run.rs:resolve_run_root",
        sample: "aw run --root wi:915",
        note: "root resolution re-dispatch when --wi is given without --root",
    },
    EmitSite {
        source: "run.rs:resolve_explicit_root",
        sample: "aw run --project agentic-workflow",
        note: "root resolution re-dispatch when --project is given",
    },
    EmitSite {
        source: "run.rs:capability_root_command",
        sample: "aw run --project agentic-workflow --root capability:work-item-planning",
        note: "root resolution re-dispatch for a --capability root",
    },
    EmitSite {
        source: "run.rs:project_envelope (capability check gate)",
        sample: "aw capability check --project agentic-workflow --verify",
        note: "project-root envelope's capability verification gate command",
    },
    EmitSite {
        source: "run.rs:project_envelope (wi prioritize gate)",
        sample: "aw wi prioritize --project agentic-workflow",
        note: "project-root envelope's work-item queue gate command",
    },
    EmitSite {
        source: "run.rs:loop_state_envelope (converged)",
        sample: "aw td code-check 915",
        note: "loop engine's terminal act, sourced from LoopState.next_action \
               (see loop_state.rs:decide_next_action) — this is the #844 site",
    },
    EmitSite {
        source: "project.rs:project_health_next_command (td lock)",
        sample: "aw td lock --project agentic-workflow",
        note: "project health's next-remediation command for a stale TD lock",
    },
    EmitSite {
        source: "project.rs:project_health_next_command (capability run)",
        sample: "aw capability run --project agentic-workflow --non-interactive --max-ticks 1",
        note: "project health's next-remediation command for capability readiness",
    },
    EmitSite {
        source: "standardize.rs:standardize (audit record)",
        sample: "aw standardize audit record --project agentic-workflow",
        note: "standardize workflow's audit-record follow-up command",
    },
    EmitSite {
        source: "standardize.rs:standardize (traceability report/next)",
        sample: "aw standardize traceability report --project agentic-workflow",
        note: "standardize traceability layer's report command",
    },
    EmitSite {
        source: "capability.rs:lifecycle_action_for_work_item (terminal check)",
        sample: "aw td code-check 915",
        note: "capability lifecycle driver's terminal-check command for a cb_filled/cb_reviewed WI",
    },
    EmitSite {
        source: "td.rs:run_claim (dispatch envelope)",
        sample: "aw td gen 915",
        note: "structured Invoke{command: \"aw td gen\", args: {slug}} reconstructed as the flat \
               command an agent runs from invoke.command + invoke.args.slug",
    },
    EmitSite {
        source: "cb_fill.rs:td_code_check_command",
        sample: "aw td code-check 915",
        note: "fill loop's terminal-check follow-up command, always built with a slug",
    },
    EmitSite {
        source: "run.rs:wi_run_command",
        sample: "aw wi run 915",
        note: "#917: canonical `aw wi run <id>` replacement for the deprecated \
               `aw run --wi <id>` / `aw run --root wi:<id>` forms; shared by \
               loop_state_envelope, closed_wi_envelope's parent_inspection_command, \
               and project_ready_wi_envelope",
    },
    EmitSite {
        source: "run.rs:capability_run_command",
        sample: "aw capability run work-item-planning --project agentic-workflow",
        note: "#917: canonical `aw capability run <capability-id> --project <project>` \
               replacement for the deprecated `aw run --root capability:<project>:<id>` forms",
    },
    EmitSite {
        source: "run.rs:project_capability_rollup_command",
        sample: "aw capability run --project agentic-workflow --non-interactive --max-ticks 1",
        note: "#917: canonical replacement for the deprecated bare `aw run --project <project>` \
               form; subsumes the project root via the existing project-scoped capability loop",
    },
];

/// One known stale/legacy persisted `next_action` string and how to repair
/// it for the current LINEAR lifecycle (`aw wi` -> `aw td create` -> `gen`
/// -> `fill` -> `code-check`, terminal).
struct LegacyNextActionRule {
    /// Matches when the trimmed raw command equals this string exactly.
    exact: &'static str,
    /// Replacement template; `{slug}` is substituted with the caller's slug.
    replacement_template: &'static str,
    #[allow(dead_code)]
    note: &'static str,
}

/// #845: `aw td merge` was removed from the LINEAR lifecycle; `aw td
/// code-check` is now the terminal step. #844: a persisted bare
/// `aw td code-check` (predating chain-required-positional enforcement)
/// needs the WI slug carried into it so dispatch targets that WI instead of
/// auditing the whole tree.
const LEGACY_NEXT_ACTION_RULES: &[LegacyNextActionRule] = &[
    LegacyNextActionRule {
        exact: "aw td merge",
        replacement_template: "aw td code-check {slug}",
        note: "#845: `aw td merge` was removed; `code-check` is now the terminal step",
    },
    LegacyNextActionRule {
        exact: "aw td code-check",
        replacement_template: "aw td code-check {slug}",
        note: "#844: bare code-check audits the whole tree instead of this WI",
    },
];

/// #917: repair a persisted `next_action` that used one of the now-deprecated
/// `aw run` root-selection flag shapes, rewriting it to the equivalent
/// `aw wi run <id>` / `aw capability run <capability-id> --project <project>`
/// form. Reuses the same command-string builders `aw wi run` and
/// `aw capability run` themselves call ([`run::wi_run_command`],
/// [`run::capability_run_command`], [`run::project_capability_rollup_command`])
/// so there is exactly one place that knows what those verbs look like.
///
/// Token parsing is the same plain `split_whitespace` scheme as
/// [`validate_aw_command_string`] (not a shell/shlex split).
///
/// Returns nested `Option`s so the caller can distinguish "not a legacy
/// root-selection form at all" (outer `None`, e.g. a bare `aw run` or
/// `aw run --human` with no root-selecting flag — the caller should fall
/// back to its own chain-validity pass-through) from "this *is* a legacy
/// root-selection form, and it either rewrites (inner `Some`) or is
/// definitively unrepairable (inner `None`, e.g. a bare `--root
/// capability:<id>` / `--capability <id>` with no explicit `--project` to
/// carry — no project inference is available from a static string)". In the
/// unrepairable case the caller must not fall through to a pass-through
/// check: the input is a known-deprecated form, not a valid one.
fn normalize_legacy_aw_run_command(cmd: &str) -> Option<Option<String>> {
    let rest = cmd.strip_prefix("aw run")?.trim();
    if rest.is_empty() {
        return None;
    }
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    let mut wi: Option<&str> = None;
    let mut project: Option<&str> = None;
    let mut capability: Option<&str> = None;
    let mut root: Option<&str> = None;
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            "--wi" => {
                wi = tokens.get(i + 1).copied();
                i += 2;
            }
            "--project" => {
                project = tokens.get(i + 1).copied();
                i += 2;
            }
            "--capability" => {
                capability = tokens.get(i + 1).copied();
                i += 2;
            }
            "--root" => {
                root = tokens.get(i + 1).copied();
                i += 2;
            }
            _ => i += 1,
        }
    }

    if wi.is_none() && root.is_none() && capability.is_none() && project.is_none() {
        // No recognized root-selection flag at all (e.g. `aw run --human`):
        // not this rewriter's concern.
        return None;
    }

    if let Some(id) = wi {
        return Some(Some(run::wi_run_command(id)));
    }
    if let Some(raw_root) = root {
        if let Some(id) = raw_root.strip_prefix("wi:") {
            return Some(Some(run::wi_run_command(id)));
        }
        if let Some(cap_id) = raw_root.strip_prefix("capability:") {
            return Some(project.map(|p| run::capability_run_command(p, cap_id)));
        }
        return Some(None);
    }
    if let Some(cap_id) = capability {
        return Some(project.map(|p| run::capability_run_command(p, cap_id)));
    }
    if let Some(project) = project {
        return Some(Some(run::project_capability_rollup_command(project)));
    }
    None
}

/// Normalize a persisted `next_action` string for dispatch.
///
/// - If `cmd` is a legacy `aw run ...` root-selection form (has a
///   recognized `--wi`/`--root`/`--capability`/`--project` flag),
///   [`normalize_legacy_aw_run_command`] is authoritative: it either
///   rewrites `cmd` to the current `aw wi run` / `aw capability run` verb,
///   or — if the form cannot be repaired (e.g. a capability root with no
///   explicit project) — this function returns `None` without falling back
///   to the plain chain-validity pass-through below (an `aw run` root form
///   is deprecated by definition here even when clap still accepts it).
/// - If `cmd` is already chain-valid, it is returned unchanged.
/// - If `cmd` exactly matches a [`LEGACY_NEXT_ACTION_RULES`] entry, the
///   repaired command (with `slug` substituted in) is returned — but only if
///   the repaired command is itself chain-valid (so an empty `slug` does not
///   silently produce another bare, chain-invalid command).
/// - Otherwise `None`: the caller must not dispatch `cmd` verbatim and
///   should surface a blocked/HITL envelope instead.
pub fn normalize_legacy_next_action(cmd: &str, slug: &str) -> Option<String> {
    let trimmed = cmd.trim();
    if let Some(outcome) = normalize_legacy_aw_run_command(trimmed) {
        return outcome.filter(|candidate| validate_aw_command_string(candidate).is_ok());
    }
    if validate_aw_command_string(trimmed).is_ok() {
        return Some(trimmed.to_string());
    }
    for rule in LEGACY_NEXT_ACTION_RULES {
        if trimmed == rule.exact {
            let candidate = rule.replacement_template.replace("{slug}", slug);
            return validate_aw_command_string(&candidate)
                .ok()
                .map(|_| candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // #915 AC1/AC3: this is the red-before/green-after regression proof for
    // #844/#845 — every known emit site's sample must be chain-valid.
    #[test]
    fn emit_registry_entries_are_all_chain_valid() {
        for site in EMIT_REGISTRY {
            if let Err(blocker) = validate_aw_command_string(site.sample) {
                panic!(
                    "emit site `{}` produced a chain-invalid sample `{}`: {}",
                    site.source, site.sample, blocker
                );
            }
        }
    }

    // #844: the exact bug — bare `aw td code-check` passes clap but is
    // chain-invalid (missing the chain-required `target`).
    #[test]
    fn bare_code_check_is_chain_invalid() {
        let err = validate_aw_command_string("aw td code-check").unwrap_err();
        assert_eq!(err.kind, ChainBlockerKind::MissingChainRequiredPositional);
    }

    #[test]
    fn code_check_with_target_is_chain_valid() {
        assert!(validate_aw_command_string("aw td code-check 915").is_ok());
    }

    #[test]
    fn unknown_verb_is_rejected_at_any_depth() {
        let top = validate_aw_command_string("aw bogus-verb").unwrap_err();
        assert_eq!(top.kind, ChainBlockerKind::ClapRejected);

        let nested = validate_aw_command_string("aw td bogus-subcommand").unwrap_err();
        assert_eq!(nested.kind, ChainBlockerKind::ClapRejected);
    }

    #[test]
    fn non_aw_command_is_rejected() {
        let err = validate_aw_command_string("ls -la").unwrap_err();
        assert_eq!(err.kind, ChainBlockerKind::NotAwCommand);
    }

    #[test]
    fn empty_command_is_rejected() {
        assert_eq!(
            validate_aw_command_string("").unwrap_err().kind,
            ChainBlockerKind::EmptyCommand
        );
        assert_eq!(
            validate_aw_command_string("   ").unwrap_err().kind,
            ChainBlockerKind::EmptyCommand
        );
    }

    // #845: `aw td merge` was removed; a persisted legacy string must
    // normalize to the current terminal step, not dispatch verbatim.
    #[test]
    fn legacy_td_merge_normalizes_to_code_check() {
        assert_eq!(
            normalize_legacy_next_action("aw td merge", "915"),
            Some("aw td code-check 915".to_string())
        );
    }

    // #844: a persisted bare code-check must pick up the caller's slug.
    #[test]
    fn legacy_bare_code_check_normalizes_to_slugged_form() {
        assert_eq!(
            normalize_legacy_next_action("aw td code-check", "915"),
            Some("aw td code-check 915".to_string())
        );
    }

    #[test]
    fn already_valid_command_passes_through_unchanged() {
        assert_eq!(
            normalize_legacy_next_action("aw td gen 915", "915"),
            Some("aw td gen 915".to_string())
        );
    }

    // Without a slug to carry, a legacy rule cannot produce a chain-valid
    // repair — the caller must not dispatch it and must surface HITL/blocked
    // instead of silently falling back to the bare, chain-invalid form.
    #[test]
    fn legacy_rule_without_a_usable_slug_normalizes_to_none() {
        assert_eq!(normalize_legacy_next_action("aw td merge", ""), None);
    }

    #[test]
    fn unparseable_next_action_normalizes_to_none() {
        assert_eq!(normalize_legacy_next_action("aw bogus-verb", "915"), None);
    }

    // #917: persisted `aw run --wi <id>` / `aw run --root wi:<id>` strings
    // repair to the new `aw wi run <id>` verb.
    #[test]
    fn legacy_aw_run_wi_flag_normalizes_to_wi_run() {
        assert_eq!(
            normalize_legacy_next_action("aw run --wi 915", "irrelevant"),
            Some("aw wi run 915".to_string())
        );
    }

    #[test]
    fn legacy_aw_run_root_wi_normalizes_to_wi_run() {
        assert_eq!(
            normalize_legacy_next_action("aw run --root wi:915", "irrelevant"),
            Some("aw wi run 915".to_string())
        );
    }

    // #917: persisted `aw run --project <p> --root capability:<id>` (or the
    // deprecated `--capability <id> --project <p>` form) repairs to the new
    // `aw capability run <id> --project <p>` verb.
    #[test]
    fn legacy_aw_run_root_capability_normalizes_to_capability_run() {
        assert_eq!(
            normalize_legacy_next_action(
                "aw run --project agentic-workflow --root capability:work-item-planning",
                "irrelevant",
            ),
            Some("aw capability run work-item-planning --project agentic-workflow".to_string())
        );
    }

    #[test]
    fn legacy_aw_run_capability_flag_normalizes_to_capability_run() {
        assert_eq!(
            normalize_legacy_next_action(
                "aw run --capability work-item-planning --project agentic-workflow",
                "irrelevant",
            ),
            Some("aw capability run work-item-planning --project agentic-workflow".to_string())
        );
    }

    // #917: a bare persisted `aw run --project <p>` repairs to the
    // project-scoped capability rollup command.
    #[test]
    fn legacy_aw_run_project_only_normalizes_to_capability_rollup() {
        assert_eq!(
            normalize_legacy_next_action("aw run --project agentic-workflow", "irrelevant"),
            Some(
                "aw capability run --project agentic-workflow --non-interactive --max-ticks 1"
                    .to_string()
            )
        );
    }

    // A `--root capability:<id>` form with no explicit `--project` cannot be
    // repaired from a static string (no project inference available here);
    // the caller must surface blocked/HITL instead of dispatching verbatim.
    #[test]
    fn legacy_aw_run_root_capability_without_project_normalizes_to_none() {
        assert_eq!(
            normalize_legacy_next_action(
                "aw run --root capability:work-item-planning",
                "irrelevant"
            ),
            None
        );
    }
}
// CODEGEN-END
