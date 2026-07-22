// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/interfaces/src/chain.md#source
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
        source: "run.rs:ec_draft_command (EC-first fresh WI admission)",
        sample: "aw ec draft 1500 --project agentic-workflow --wi 1500",
        note: "a fresh bounded WI creates its project-local EC skeleton before TD/codegen",
    },
    EmitSite {
        source: "ec.rs:EC-first WI transition",
        sample: "aw ec gen --project agentic-workflow --verify --wi 1500",
        note: "only a successful EC generation may unlock the owning WI's TD create act",
    },
    EmitSite {
        source: "run.rs:ec_verify_command (EC verdict transition)",
        sample: "aw ec verify --project agentic-workflow --wi 1500",
        note: "a completed TD/codegen candidate records its EC verdict before root dispatch",
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
        source: "project.rs:project_health_next_command (self-hosting capability verification)",
        sample: "aw capability check --project agentic-workflow --verify",
        note: "self-hosting health verifies capability work roots without re-entering a root runner",
    },
    EmitSite {
        source: "standardize.rs:takeover_audit_health_worker_command (unrecorded)",
        sample: "aw td audit-record --project agentic-workflow",
        note: "#1278: health takeover-audit axis routing pointer -- unrecorded preservation \
               audit routes to the relocated `aw td audit-record` remediation verb (`aw \
               standardize audit record` is retired)",
    },
    EmitSite {
        source: "standardize.rs:takeover_audit_health_worker_command (recorded)",
        sample: "aw health --project agentic-workflow takeover-audit --verbose",
        note: "#1278: health takeover-audit axis routing pointer -- already-recorded case \
               points back at the read-only health section instead of re-running record",
    },
    EmitSite {
        source: "standardize.rs:managed_health_worker_command (~:1298)",
        sample: "aw td create --from-source apps/agentic-workflow/src/cli/chain.rs --project agentic-workflow",
        note: "health managed-axis routing: unmarked file -> td create --from-source (#920, folded per #1273)",
    },
    EmitSite {
        source: "standardize.rs:semantic_health_worker_command (~:1313)",
        sample: "aw health --project agentic-workflow metrics --verbose",
        note: "health semantic-axis routing pointer (#920)",
    },
    EmitSite {
        source: "standardize.rs:traceability_health_worker_command (~:1323)",
        sample: "aw health --project agentic-workflow traceability --verbose",
        note: "health traceability-axis routing pointer (#920)",
    },
    EmitSite {
        source: "capability.rs:lifecycle_action_for_work_item (terminal check)",
        sample: "aw td code-check 915",
        note: "capability lifecycle driver's terminal-check command for a cb_filled/cb_reviewed WI",
    },
    EmitSite {
        source: "fillback.rs:run (explicit source-file dispatch envelope)",
        sample: "aw td gen-source --spec apps/agentic-workflow/tech-design/example.md --target apps/agentic-workflow/src/example.rs --dry-run",
        note: "lossless explicit source adoption emits the concrete per-file gen-source \
               verification command with both authoritative artifact paths",
    },
    EmitSite {
        source: "fillback.rs:run (directory completion envelope)",
        sample: "aw td check apps/agentic-workflow/tech-design/specs",
        note: "directory fillback keeps its existing human progress stream and ends with a \
               runnable TD artifact check",
    },
    EmitSite {
        source: "cb_fill.rs:td_code_check_command",
        sample: "aw td code-check 915",
        note: "fill loop's terminal-check follow-up command, always built with a slug",
    },
    EmitSite {
        source: "run.rs:wi_run_command",
        sample: "aw goal wi 915",
        note: "#1899: canonical `aw goal wi <id>` replacement for the retired \
               `aw wi run <id>` verb (itself #917's replacement for the earlier \
               `aw run --wi <id>` / `aw run --root wi:<id>` forms); shared by \
               loop_state_envelope, closed_wi_envelope's parent_inspection_command, \
               and project_ready_wi_envelope",
    },
    EmitSite {
        source: "run.rs:open_epic_envelope",
        sample: "aw wi atomize --project pgpool",
        note: "#1518: an open epic resolves project:/app:/lib: identity labels before \
               emitting its atomize handoff; unresolved labels block with remediation \
               instead of substituting a PROJECT placeholder",
    },
    EmitSite {
        source: "run.rs:capability_run_command",
        sample: "aw goal capability work-item-planning --project jet",
        note: "#1899: canonical `aw goal capability <capability-id> --project <project>` \
               replacement for the retired `aw capability run <capability-id> --project \
               <project>` verb (itself #917's replacement for the earlier `aw run --root \
               capability:<project>:<id>` forms); Agentic Workflow self-hosting is \
               rejected at admission",
    },
    EmitSite {
        source: "run.rs:project_capability_rollup_command",
        sample: "aw health --project agentic-workflow claims",
        note: "self-hosting rollup is a read-only health inspection; other projects use the \
               project-scoped `aw goal capability --project <project> --non-interactive \
               --max-ticks 1` rollup form (#1899)",
    },
    EmitSite {
        source: "cb.rs:bare_code_check_guidance_envelope",
        sample: "aw health --project agentic-workflow drift-marker --verbose",
        note: "#1276: bare, slug-less `aw td code-check`'s guidance envelope now points at the \
               `aw health` drift/marker axis instead of running the retired whole-tree audit \
               walker (the #844 livelock class) itself",
    },
    EmitSite {
        source: "goal.rs:print_check_outcome (blocked self-loop)",
        sample: "aw goal check a1b2c3d4",
        note: "#1897: `aw goal check`'s red-gate report loops back on itself with the \
               resolved goal id until every recorded gate is green or the goal's budget/\
               24h expiry ceiling is exhausted",
    },
    EmitSite {
        source: "run.rs:run_backlog_root (selected WI hand-off)",
        sample: "aw goal wi 915",
        note: "#1899 R7: `aw goal backlog --project <p>` selects the next unparked open WI \
               in priority order and hands it to the host via the same `aw goal wi <id>` \
               command `wi_run_command` builds elsewhere; a blocked/HITL candidate is \
               parked (never emitted) and the drain moves to the next one",
    },
    EmitSite {
        source: "issues.rs:capability_plan_review_command",
        sample: "aw wi plan-review --evidence-file /tmp/aw/capability-plan-review.json",
        note: "#2187: capability planning stops at a digest-bound independent review payload; \
               the emitted command consumes reviewer evidence before any bounded tracker WI is \
               published",
    },
];

// ---------------------------------------------------------------------------
// epic #1270 R4+R9 (#1272): verb lifecycle registry + removal-precondition
// policy. Every registered `aw` CLI verb gets a declared lifecycle class so
// verb removal has a documented, enforced precondition instead of rotting in
// place (#1243 is the proof case: a migration verb with no stated retirement
// condition). See CONTRIBUTING.md "CLI verb lifecycle and removal gate" for
// the policy this table backs.
// ---------------------------------------------------------------------------

/// Lifecycle classification for one registered `aw` CLI verb.
///
///   - `Core`: the wi/td/ec/capability/health/conf lifecycle and loop
///     surface — the verbs the LINEAR loop (`aw wi` -> `aw td create` ->
///     `gen` -> `fill` -> `code-check`) and its sibling ec/capability/health
///     loops actually dispatch through.
///   - `Utility`: support tooling that is not itself a lifecycle-loop step —
///     the CLI-convention trio (`llm`/`upgrade`/`issue`), `guard`/`new`/
///     `report-issue`/`generator`, and the read-only/debug `td`
///     verbs (`ast`, `check`, `lock`, `promote`,
///     `audit-record` -- the former `standardize audit record`, rehomed by
///     #1278).
///   - `Migration`: scheduled for removal or fold-in once a stated condition
///     holds. Every `Migration` entry MUST carry a non-empty
///     [`VerbLifecycle::sunset_criterion`] naming that condition — this is
///     the conformance test's AC2 gate below.
// `Debug`/`Copy`/(in)equality are used by [`leaf_verb_paths_are_all_classified`];
// the type itself is otherwise only consumed by that test today — same
// dead-code shape as [`EmitSite`]/[`EMIT_REGISTRY`] above, which are
// `#[cfg(test)]`-only. This table is deliberately *not* test-gated (unlike
// `EMIT_REGISTRY`) so a later slice (e.g. an `aw health` axis) can read it
// at runtime without relocating it; `#[allow(dead_code)]` covers the gap
// until such a consumer exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum VerbLifecycleClass {
    Core,
    Utility,
    Migration,
}

/// One registered `aw` CLI verb's lifecycle classification. `path` is the
/// canonical dot-joined verb path down to the clap leaf command, using clap's
/// own kebab-case subcommand names (e.g. `"td.code-check"`,
/// `"wi.draft.init"`) — see [`leaf_verb_paths`] for how this is compared
/// against the real registered clap tree ([`TraceabilityCli`], the same tree
/// [`validate_aw_command_string`] validates against).
#[allow(dead_code)]
struct VerbLifecycle {
    path: &'static str,
    class: VerbLifecycleClass,
    /// `true` when this verb mutates tracked lifecycle/config state on disk
    /// (TD/EC/capability/WI artifacts, `aw.toml`'s generated project
    /// registry block, or a lifecycle-terminal transition) as opposed to
    /// only reading/reporting it. This is the classification #1417's
    /// stale-binary skew gate ([`crate::cli::drift::mutating_verb_gate`])
    /// reads: a binary strictly behind the checkout's source version may
    /// still run any `false` (read-only) verb with only the existing warn,
    /// but a `true` verb hard-refuses unless `AW_ALLOW_STALE_BINARY=1` is
    /// set — a stale binary must not write artifacts in a retired protocol
    /// shape. Orthogonal to [`VerbLifecycleClass`] (product-loop position);
    /// e.g. `td.promote`/`td.audit-record` are `Utility`-class but still
    /// mutate tracked state, so they are `true` here.
    mutates_lifecycle: bool,
    /// Non-empty only (and always) for `Migration` — the concrete condition
    /// under which this verb may be removed. Empty for `Core`/`Utility`.
    sunset_criterion: &'static str,
}

/// The full registered-verb lifecycle registry (epic #1270 R4+R9). One entry
/// per leaf `aw` CLI verb — see [`leaf_verb_paths_are_all_classified`] for
/// the conformance test that keeps this in sync with the real clap tree.
#[allow(dead_code)]
const VERB_LIFECYCLE_REGISTRY: &[VerbLifecycle] = &[
    // -- top-level utility --------------------------------------------------
    VerbLifecycle {
        path: "new",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "llm",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "upgrade",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "report-issue",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    // -- top-level core -------------------------------------------------
    VerbLifecycle {
        path: "health",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    // -- generator (support: takeover-readiness gap request surface) --------
    VerbLifecycle {
        path: "generator.check",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "generator.request",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    // -- guard (support) ------------------------------------------------
    VerbLifecycle {
        path: "guard.on",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "guard.off",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "guard.pretool",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    // -- conf (core: aw.toml project registry the loop reads from) ------
    VerbLifecycle {
        path: "conf.check",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "conf.sync",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    // -- meta (core: repository/project iteration control plane) -------
    VerbLifecycle {
        path: "meta.init",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "meta.sync",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "meta.check",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    // -- wi (core loop: work-item inventory, linear authoring, run) -----
    VerbLifecycle {
        path: "wi.draft.init",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.draft.fill",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.draft.validate",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.list",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.show",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.run",
        // #1899 R3: retired -- `aw goal wi <id>` is the unified re-home.
        // The clap leaf still parses (structured `emit_retired_verb_redirect`
        // envelope instead of a bare clap usage error) but never re-enters
        // the run engine.
        class: VerbLifecycleClass::Migration,
        mutates_lifecycle: false,
        sunset_criterion: "retires (clap leaf removed) once every EMIT_REGISTRY/loop-state/\
                            HITL-resumption caller and every persisted `next_action` has been \
                            observed on `aw goal wi` for a full deploy cycle with no remaining \
                            `aw wi run` invocations in telemetry",
    },
    VerbLifecycle {
        path: "wi.create",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.update",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.close",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.find",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.plan",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.plan-review",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.epicize",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.atomize",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.prioritize",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.enrich",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.validate",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "wi.fill-section",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    // -- issue (support: CLI-convention trio's issue verb) ---------------
    VerbLifecycle {
        path: "issue.search",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "issue.view",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "issue.create",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "issue.comment",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    // -- goal (support: ad-hoc verifiable-condition loop, #1897) --------
    // Goal state is workspace-scoped ephemeral `/tmp/aw` JSON, explicitly
    // outside the WI/TD/EC tracked-lifecycle surface `mutates_lifecycle`
    // documents above, so every `goal.*` leaf is `false` here regardless
    // of whether it writes its own state file.
    VerbLifecycle {
        path: "goal.set",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "goal.check",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "goal.show",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "goal.list",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "goal.clear",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    // -- goal (lifecycle root types, #1899: they ARE the loop now) ------
    // Unlike the ad-hoc goal.* leaves above, `goal.wi`/`goal.capability`/
    // `goal.backlog` drive the tracked WI/capability lifecycle exactly as
    // the retired `aw wi run`/`aw capability run` verbs did -- Core,
    // mutating.
    VerbLifecycle {
        path: "goal.wi",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "goal.capability",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "goal.backlog",
        // #1899 R7: tracker-driven drain root type -- drives the tracked WI
        // lifecycle one open WI at a time via the shared `goal.wi` engine.
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    // -- td (core LINEAR lifecycle + read-only/debug support verbs) -----
    VerbLifecycle {
        path: "td.create",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    // `td.validate` retired by #1277 (epic #1270 R3): its slug-mode
    // lifecycle-mutation half was inlined into `td.create`'s whole-file
    // apply path; its read-only rule check was already shared with
    // `td.check`, which remains the sole authoritative read-only TD
    // checker (read-only/debug support, like `td.ast`/`td.lock`).
    VerbLifecycle {
        path: "td.check",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "td.ast",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "td.migrate-mermaid",
        class: VerbLifecycleClass::Migration,
        mutates_lifecycle: false,
        sunset_criterion: "retires once `aw td migrate-mermaid <project-td-root> --check` \
                            reports `legacy_block_count: 0` for every configured project's \
                            tech-design root (no remaining frontmatter-less legacy mermaid \
                            blocks for the migrator's detector)",
    },
    VerbLifecycle {
        path: "td.lock",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "td.claim",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "td.gen",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "td.gen-source",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "td.code-check",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "td.fill",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "td.promote",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    // #1278 (epic #1270 R7): `aw standardize audit record` rehomed here,
    // mirroring the `td.promote` fold-in above.
    VerbLifecycle {
        path: "td.audit-record",
        class: VerbLifecycleClass::Utility,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    // -- ec (core: external-contract lifecycle) --------------------------
    VerbLifecycle {
        path: "ec.draft",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "ec.fill",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "ec.gen",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "ec.check",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "ec.lock",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "ec.review",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "ec.record",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "ec.verify",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "ec.doc.gen",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "ec.doc.check",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "ec.doc.preview",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    // #1278 (epic #1270 R7): `aw standardize` namespace (`standardize.audit.check`
    // / `standardize.audit.record`) fully retired -- reporting folded into
    // the `aw health` takeover-audit axis, `audit record` rehomed as
    // `td.audit-record` above.
    // -- capability (core: product capability completion loop) ----------
    VerbLifecycle {
        path: "capability.report",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "capability.next",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "capability.draft",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "capability.apply-draft",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "capability.run",
        // #1899 R3: retired -- `aw goal capability [<cap-id>] --project <p>`
        // is the unified re-home. The clap leaf still parses (structured
        // `emit_retired_verb_redirect` envelope instead of a bare clap
        // usage error) but never re-enters the run engine.
        class: VerbLifecycleClass::Migration,
        mutates_lifecycle: false,
        sunset_criterion: "retires (clap leaf removed) once every EMIT_REGISTRY/loop-state/\
                            HITL-resumption caller and every persisted `next_action` has been \
                            observed on `aw goal capability` for a full deploy cycle with no \
                            remaining `aw capability run` invocations in telemetry",
    },
    VerbLifecycle {
        path: "capability.migrate",
        class: VerbLifecycleClass::Migration,
        mutates_lifecycle: true,
        sunset_criterion: "retires once `aw capability sweep --skip-issue-inventory` reports \
                            zero projects grouped under `next_action_kind: \
                            \"format_migration_required\"` across all configured `cap_path`s \
                            (no remaining YAML `## Capability:` sections or legacy capability \
                            tables left to convert)",
    },
    VerbLifecycle {
        path: "capability.check",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "capability.init",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "capability.sweep",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: false,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "capability.set-type",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "capability.set-status",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "capability.set-surface",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "capability.set-ec-dimension",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
    VerbLifecycle {
        path: "capability.set-wi-ref",
        class: VerbLifecycleClass::Core,
        mutates_lifecycle: true,
        sunset_criterion: "",
    },
];

/// Recursively collect every leaf verb path (dot-joined, down to clap
/// commands with no further subcommands) in the real registered `aw` CLI
/// tree, skipping clap's auto-added `help` pseudo-subcommand. Walks the same
/// full `Commands` tree [`validate_aw_command_string`] validates against
/// ([`TraceabilityCli`]) — the source of truth
/// [`leaf_verb_paths_are_all_classified`] compares [`VERB_LIFECYCLE_REGISTRY`]
/// against.
#[allow(dead_code)]
fn leaf_verb_paths() -> Vec<String> {
    fn walk(cmd: &clap::Command, prefix: &str, out: &mut Vec<String>) {
        let subcommands: Vec<&clap::Command> = cmd
            .get_subcommands()
            .filter(|sub| sub.get_name() != "help")
            .collect();
        if subcommands.is_empty() {
            if !prefix.is_empty() {
                out.push(prefix.to_string());
            }
            return;
        }
        for sub in subcommands {
            let path = if prefix.is_empty() {
                sub.get_name().to_string()
            } else {
                format!("{prefix}.{}", sub.get_name())
            };
            walk(sub, &path, out);
        }
    }

    let root = TraceabilityCli::command();
    let mut out = Vec::new();
    walk(&root, "", &mut out);
    out
}

/// #1417: look up whether a leaf verb path (dot-joined, e.g. `"td.fill"`,
/// `"wi.run"` — the same shape [`leaf_verb_paths`] produces) mutates tracked
/// lifecycle/config state, per [`VERB_LIFECYCLE_REGISTRY`]. `None` when
/// `path` has no registry entry — the stale-binary gate
/// ([`crate::cli::drift::enforce_mutating_verb_gate`]) treats an unresolved
/// verb as "do not refuse" (fail open) rather than blocking an unrecognized
/// invocation.
pub(crate) fn verb_mutates_lifecycle(path: &str) -> Option<bool> {
    VERB_LIFECYCLE_REGISTRY
        .iter()
        .find(|entry| entry.path == path)
        .map(|entry| entry.mutates_lifecycle)
}

/// #1417: resolve the canonical dot-joined leaf verb path the current
/// process was actually invoked with (same shape as
/// [`VERB_LIFECYCLE_REGISTRY`]'s `path` field), by re-parsing `args`
/// (expected to be `std::env::args().collect()`, program name in `args[0]`,
/// which clap ignores for subcommand matching) through the real clap tree
/// ([`TraceabilityCli`] — the same tree [`validate_aw_command_string`]
/// validates against and [`leaf_verb_paths`] walks). Returns `None` when
/// `args` doesn't resolve to a leaf subcommand at all (e.g. `aw --version`,
/// `aw --help`, or an invocation clap itself would reject) — callers must
/// treat that as "verb unknown", not as evidence the invocation is safe.
pub(crate) fn resolve_invoked_verb_path(args: &[String]) -> Option<String> {
    let matches = TraceabilityCli::command().try_get_matches_from(args).ok()?;
    let mut parts = Vec::new();
    let mut current = &matches;
    while let Some((name, sub)) = current.subcommand() {
        parts.push(name.to_string());
        current = sub;
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("."))
    }
}

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

// ---------------------------------------------------------------------------
// #921 tier 1b: validate a cross-CLI `ec.*` binding's resolved command
// against the vat.toml runner registry it targets. The 6 real production
// bindings (`aw.toml` `ec.<category> = { ..., command = "cd <dir> &&
// ../../target/debug/vat run <runner-id>" }`) are dispatched by
// `aw ec check` / `aw health --verify-ec` today with nobody validating the
// runner id statically — a typo only ever surfaces as a runtime `Failed`
// deep inside `--verify-ec`. This closes that gap for the common two-hop
// shape (`vat run <runner-id>`); guard's `--meter-command` flag value is a
// further, nested third hop this deliberately does not recurse into
// (documented as executed-only, covered by `--verify-ec`).
// ---------------------------------------------------------------------------

/// One `[[runners]]` entry in a `vat.toml` file. Deserialized locally (no aw
/// -> vat build dependency) — this only needs the two fields tier 1b cares
/// about.
#[derive(Debug, Clone, serde::Deserialize)]
struct VatRunnerEntry {
    id: String,
    #[serde(default)]
    cmd: Vec<String>,
}

/// Top-level shape of a `vat.toml` file, restricted to `[[runners]]`.
#[derive(Debug, Clone, Default, serde::Deserialize)]
struct VatRunnersFile {
    #[serde(default)]
    runners: Vec<VatRunnerEntry>,
}

/// The `cd <dir> && <bin> run <runner-id>` shape every real `ec.*` binding
/// uses today. `dir`/`binary` are exactly as written in the command string
/// (relative paths, not yet joined to anything).
struct VatRunnerInvocation<'a> {
    dir: &'a str,
    binary: &'a str,
    runner_id: &'a str,
}

/// Parse a resolved `ec.*` binding command string for the `cd <dir> &&
/// <bin> run <runner-id>` shape. Returns `None` for anything else — this is
/// deliberately narrow (not a general shell parser); a command that isn't
/// shaped like this has nothing for tier 1b to validate.
fn parse_vat_runner_invocation(command: &str) -> Option<VatRunnerInvocation<'_>> {
    let (cd_part, run_part) = command.split_once("&&")?;
    let dir = cd_part.trim().strip_prefix("cd ")?.trim();
    if dir.is_empty() {
        return None;
    }
    let run_tokens: Vec<&str> = run_part.trim().split_whitespace().collect();
    if run_tokens.len() < 3 || run_tokens[1] != "run" {
        return None;
    }
    let binary = run_tokens[0];
    let is_vat_binary = std::path::Path::new(binary)
        .file_name()
        .and_then(|name| name.to_str())
        == Some("vat");
    if !is_vat_binary {
        return None;
    }
    Some(VatRunnerInvocation {
        dir,
        binary,
        runner_id: run_tokens[2],
    })
}

/// #921 AC1: validate one `ec.<category>` binding's resolved command against
/// the vat.toml runner registry it targets. `project_root` is the repo root
/// (`dir` in the command is relative to it). Returns `(blockers, warnings)`:
/// a blocker names the vat.toml path and the missing runner id (AC1); a
/// path-shaped `cmd[0]` (e.g. `../../target/debug/meter`) missing on disk for
/// an otherwise-valid runner is warn-only ("buildable, not built" —
/// `--verify-ec` would catch an actually-broken binary at run time anyway). A
/// bare `cmd[0]` command name (e.g. `sh`, `cargo`) is resolved via `PATH`, not
/// relative to `dir`, so it is never checked for existence. A command that
/// isn't shaped like `cd <dir> && vat run <runner-id>` returns `(vec![],
/// vec![])` — nothing for tier 1b to validate there.
pub(crate) fn check_ec_vat_runner_binding(
    project_root: &std::path::Path,
    category: &str,
    command: &str,
) -> (Vec<String>, Vec<String>) {
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let Some(invocation) = parse_vat_runner_invocation(command) else {
        return (blockers, warnings);
    };
    let dir = project_root.join(invocation.dir);
    let vat_toml_path = dir.join("vat.toml");
    let vat_toml_display = format!("{}/vat.toml", invocation.dir);
    let content = match std::fs::read_to_string(&vat_toml_path) {
        Ok(content) => content,
        Err(_) => {
            blockers.push(format!(
                "ec binding `{category}`: {vat_toml_display} not found (referenced by `cd {} \
                 && {} run {}`)",
                invocation.dir, invocation.binary, invocation.runner_id
            ));
            return (blockers, warnings);
        }
    };
    let runners_file: VatRunnersFile = match toml::from_str(&content) {
        Ok(parsed) => parsed,
        Err(err) => {
            blockers.push(format!(
                "ec binding `{category}`: {vat_toml_display} failed to parse as vat.toml: {err}"
            ));
            return (blockers, warnings);
        }
    };
    match runners_file
        .runners
        .iter()
        .find(|runner| runner.id == invocation.runner_id)
    {
        None => {
            blockers.push(format!(
                "ec binding `{category}`: {vat_toml_display} has no runner id `{}`",
                invocation.runner_id
            ));
        }
        Some(runner) => {
            if let Some(bin) = runner.cmd.first() {
                // Only meaningful for a path-shaped `cmd[0]` (e.g.
                // `../../target/debug/meter`) — a bare command name like `sh`
                // or `cargo` is resolved via `PATH` at run time, not relative
                // to `dir`, so there is nothing on disk here to check.
                if bin.contains('/') && !dir.join(bin).exists() {
                    warnings.push(format!(
                        "ec binding `{category}`: runner `{}` in {vat_toml_display} points to \
                         `{bin}`, which is not built yet (buildable, not built)",
                        invocation.runner_id
                    ));
                }
            }
        }
    }
    (blockers, warnings)
}

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

/// #1899 R4: repair a persisted `next_action` that used the now-retired
/// `aw wi run <id>` / `aw capability run [<capability-id>] --project
/// <project>` verb shapes, rewriting it to the equivalent `aw goal wi <id>`
/// / `aw goal capability [<capability-id>] --project <project>` form. An
/// in-flight workflow started under the old runner verbs must still
/// complete post-flip: `aw wi run`/`aw capability run` still parse (clap
/// leaf kept for the [`emit_retired_verb_redirect`] envelope), so without
/// this rewrite [`normalize_legacy_next_action`]'s plain chain-validity
/// pass-through below would hand the old form straight back and dispatch it
/// into the retired-verb bail instead of the resumed tick. Reuses the same
/// command-string builders [`run::wi_run_command`] /
/// [`run::capability_run_command`] / [`run::project_capability_rollup_command`]
/// (already goal-namespace producers, #1899 R1/R3) so there is exactly one
/// place that knows what those verbs look like.
///
/// Token parsing is the same plain `split_whitespace` scheme as
/// [`validate_aw_command_string`] (not a shell/shlex split). Returns `None`
/// when `cmd` isn't one of these two retired verb shapes at all -- the
/// caller should fall back to its own chain-validity pass-through.
fn normalize_legacy_wi_capability_run_command(cmd: &str) -> Option<String> {
    if let Some(rest) = cmd.strip_prefix("aw wi run") {
        let id = rest.trim().split_whitespace().next()?;
        return Some(run::wi_run_command(id));
    }
    let rest = cmd.strip_prefix("aw capability run")?.trim();
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    // The capability id, if present, is always the leading positional
    // (mirrors `capability_run_command`'s own `<capability-id> --project
    // <project>` shape) -- never treat a later bare token (e.g. the `1` in
    // `--max-ticks 1`) as a capability id.
    let capability_id = tokens.first().filter(|t| !t.starts_with("--")).copied();
    let flags_start = if capability_id.is_some() { 1 } else { 0 };
    let mut project: Option<&str> = None;
    let mut i = flags_start;
    while i < tokens.len() {
        match tokens[i] {
            "--project" => {
                project = tokens.get(i + 1).copied();
                i += 2;
            }
            _ => i += 1,
        }
    }
    match (capability_id, project) {
        (Some(cap_id), Some(p)) => Some(run::capability_run_command(p, cap_id)),
        (None, Some(p)) => Some(run::project_capability_rollup_command(p)),
        _ => None,
    }
}

/// Normalize a persisted `next_action` string for dispatch.
///
/// - If `cmd` is a legacy `aw run ...` root-selection form (has a
///   recognized `--wi`/`--root`/`--capability`/`--project` flag),
///   [`normalize_legacy_aw_run_command`] is authoritative: it either
///   rewrites `cmd` to the current `aw goal wi` / `aw goal capability` verb,
///   or — if the form cannot be repaired (e.g. a capability root with no
///   explicit project) — this function returns `None` without falling back
///   to the plain chain-validity pass-through below (an `aw run` root form
///   is deprecated by definition here even when clap still accepts it).
/// - If `cmd` is the retired `aw wi run <id>` / `aw capability run
///   [<capability-id>] --project <project>` form (#1899 R4),
///   [`normalize_legacy_wi_capability_run_command`] rewrites it to the
///   equivalent `aw goal wi` / `aw goal capability` form so an in-flight
///   pre-flip workflow still completes post-flip, rather than falling
///   through to the pass-through below and dispatching straight into the
///   retired-verb redirect bail.
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
    if let Some(rewritten) = normalize_legacy_wi_capability_run_command(trimmed) {
        return validate_aw_command_string(&rewritten)
            .ok()
            .map(|_| rewritten);
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

    // #1272 AC1/AC2 (epic #1270 R4+R9): every leaf verb in the real
    // registered clap tree must carry a [`VERB_LIFECYCLE_REGISTRY`] entry
    // (AC1), every registry entry must reference a verb that still exists
    // (no dangling classification), and every `Migration`-class entry must
    // carry a non-empty `sunset_criterion` (AC2).
    #[test]
    fn leaf_verb_paths_are_all_classified() {
        let registered: std::collections::BTreeSet<String> =
            leaf_verb_paths().into_iter().collect();
        assert!(
            !registered.is_empty(),
            "leaf_verb_paths() found no registered verbs"
        );

        let classified: std::collections::BTreeSet<&'static str> = VERB_LIFECYCLE_REGISTRY
            .iter()
            .map(|entry| entry.path)
            .collect();

        let unclassified: Vec<&String> = registered
            .iter()
            .filter(|path| !classified.contains(path.as_str()))
            .collect();
        assert!(
            unclassified.is_empty(),
            "registered verb(s) with no VERB_LIFECYCLE_REGISTRY entry: {unclassified:?}"
        );

        let dangling: Vec<&'static str> = VERB_LIFECYCLE_REGISTRY
            .iter()
            .map(|entry| entry.path)
            .filter(|path| !registered.contains(*path))
            .collect();
        assert!(
            dangling.is_empty(),
            "VERB_LIFECYCLE_REGISTRY entries referencing verbs that no longer exist: {dangling:?}"
        );

        let missing_sunset: Vec<&'static str> = VERB_LIFECYCLE_REGISTRY
            .iter()
            .filter(|entry| {
                entry.class == VerbLifecycleClass::Migration && entry.sunset_criterion.is_empty()
            })
            .map(|entry| entry.path)
            .collect();
        assert!(
            missing_sunset.is_empty(),
            "migration-class verb(s) missing a sunset_criterion: {missing_sunset:?}"
        );

        let non_migration_with_sunset: Vec<&'static str> = VERB_LIFECYCLE_REGISTRY
            .iter()
            .filter(|entry| {
                entry.class != VerbLifecycleClass::Migration && !entry.sunset_criterion.is_empty()
            })
            .map(|entry| entry.path)
            .collect();
        assert!(
            non_migration_with_sunset.is_empty(),
            "non-migration verb(s) carrying a sunset_criterion (only Migration entries should): \
             {non_migration_with_sunset:?}"
        );
    }

    // #1417: spot-check the `mutates_lifecycle` classification for a
    // representative sample of each named category from the issue's design
    // (td/wi/capability/ec/conf/top-level mutating verbs vs. read-only
    // verbs) — the full registry is total by construction (the struct
    // literal requires the field), this only guards the *values*.
    #[test]
    fn mutates_lifecycle_classification_matches_1417_design() {
        let mutating = [
            "new",
            "conf.sync",
            "goal.wi",
            "goal.capability",
            "goal.backlog",
            "wi.create",
            "wi.update",
            "wi.close",
            "wi.fill-section",
            "td.create",
            "td.gen",
            "td.gen-source",
            "td.fill",
            "td.claim",
            "td.promote",
            "td.audit-record",
            "td.code-check",
            "ec.gen",
            "ec.lock",
            "capability.apply-draft",
            "capability.init",
            "capability.migrate",
            "capability.set-type",
            "capability.set-status",
            "capability.set-surface",
            "capability.set-ec-dimension",
            "capability.set-wi-ref",
        ];
        for path in mutating {
            assert_eq!(
                verb_mutates_lifecycle(path),
                Some(true),
                "{path} must be classified mutates_lifecycle: true"
            );
        }

        let read_only = [
            "wi.list",
            "wi.show",
            "wi.find",
            // #1899 R3: both retired -- `mutates_lifecycle: false` now (see
            // VERB_LIFECYCLE_REGISTRY comments above), since these leaves
            // only ever emit an `emit_retired_verb_redirect` envelope and
            // never re-enter the run engine.
            "wi.run",
            "capability.run",
            "health",
            "llm",
            "upgrade",
            "td.check",
            "td.lock",
            "td.ast",
            "ec.check",
            "ec.verify",
            "capability.report",
            "capability.next",
            "capability.check",
            "conf.check",
        ];
        for path in read_only {
            assert_eq!(
                verb_mutates_lifecycle(path),
                Some(false),
                "{path} must be classified mutates_lifecycle: false"
            );
        }
    }

    #[test]
    fn verb_mutates_lifecycle_none_for_unknown_path() {
        assert_eq!(verb_mutates_lifecycle("not.a.real.verb"), None);
    }

    // #1417: `resolve_invoked_verb_path` must recover the same dot-joined
    // leaf path `VERB_LIFECYCLE_REGISTRY` and `leaf_verb_paths()` use, from
    // a raw `std::env::args()`-shaped slice (program name in slot 0).
    #[test]
    fn resolve_invoked_verb_path_recovers_leaf_path() {
        let args: Vec<String> = ["aw", "td", "fill", "some-slug"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(
            resolve_invoked_verb_path(&args),
            Some("td.fill".to_string())
        );
    }

    #[test]
    fn resolve_invoked_verb_path_recovers_nested_leaf_path() {
        // `ec doc check` has no required args, unlike `wi draft init`
        // (`--title`/`--type`) — pick a no-required-arg 3-level leaf so this
        // test only exercises path resolution, not argument validity.
        let args: Vec<String> = ["aw", "ec", "doc", "check"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(
            resolve_invoked_verb_path(&args),
            Some("ec.doc.check".to_string())
        );
    }

    #[test]
    fn resolve_invoked_verb_path_none_for_unrecognized_invocation() {
        let args: Vec<String> = ["aw", "not-a-real-verb"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(resolve_invoked_verb_path(&args), None);
    }

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

    // #1276 AC2/AC3c: no cataloged emit site's sample is the bare, slug-less
    // `aw td code-check` form (the #844 whole-tree-audit livelock class);
    // `emit_registry_entries_are_all_chain_valid` above already proves every
    // sample passes `validate_aw_command_string`, which independently rejects
    // a slug-less code-check (`bare_code_check_is_chain_invalid`) -- this
    // test pins the intent directly so a future emit site can't reintroduce
    // the exact bare string even if some other chain-required-positional
    // covers it incidentally.
    #[test]
    fn no_emit_site_produces_slugless_code_check() {
        for site in EMIT_REGISTRY {
            assert_ne!(
                site.sample, "aw td code-check",
                "emit site `{}` produces the bare, slug-less `aw td code-check` form",
                site.source
            );
        }
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

    // #917/#1899: persisted `aw run --wi <id>` / `aw run --root wi:<id>`
    // strings repair to the current `aw goal wi <id>` verb.
    #[test]
    fn legacy_aw_run_wi_flag_normalizes_to_wi_run() {
        assert_eq!(
            normalize_legacy_next_action("aw run --wi 915", "irrelevant"),
            Some("aw goal wi 915".to_string())
        );
    }

    #[test]
    fn legacy_aw_run_root_wi_normalizes_to_wi_run() {
        assert_eq!(
            normalize_legacy_next_action("aw run --root wi:915", "irrelevant"),
            Some("aw goal wi 915".to_string())
        );
    }

    // #917/#1899: persisted `aw run --project <p> --root capability:<id>`
    // (or the deprecated `--capability <id> --project <p>` form) repairs to
    // the current `aw goal capability <id> --project <p>` verb.
    #[test]
    fn legacy_aw_run_root_capability_normalizes_to_capability_run() {
        assert_eq!(
            normalize_legacy_next_action(
                "aw run --project agentic-workflow --root capability:work-item-planning",
                "irrelevant",
            ),
            Some("aw goal capability work-item-planning --project agentic-workflow".to_string())
        );
    }

    #[test]
    fn legacy_aw_run_capability_flag_normalizes_to_capability_run() {
        assert_eq!(
            normalize_legacy_next_action(
                "aw run --capability work-item-planning --project agentic-workflow",
                "irrelevant",
            ),
            Some("aw goal capability work-item-planning --project agentic-workflow".to_string())
        );
    }

    // #917: a bare persisted `aw run --project <p>` repairs to the
    // project-scoped capability rollup command.
    #[test]
    fn legacy_aw_run_project_only_normalizes_to_capability_rollup() {
        assert_eq!(
            normalize_legacy_next_action("aw run --project agentic-workflow", "irrelevant"),
            Some("aw health --project agentic-workflow claims".to_string())
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

    // #1899 R4: a persisted `next_action` carrying the retired `aw wi run
    // <id>` verb still repairs to the current `aw goal wi <id>` form, so an
    // in-flight pre-flip workflow completes post-flip instead of dispatching
    // straight into the retired-verb redirect bail.
    #[test]
    fn legacy_wi_run_next_action_normalizes_to_goal_wi() {
        assert_eq!(
            normalize_legacy_next_action("aw wi run 915", "irrelevant"),
            Some("aw goal wi 915".to_string())
        );
    }

    // #1899 R4: a persisted `next_action` carrying the retired `aw
    // capability run <id> --project <p>` verb still repairs to the current
    // `aw goal capability <id> --project <p>` form.
    #[test]
    fn legacy_capability_run_next_action_normalizes_to_goal_capability() {
        assert_eq!(
            normalize_legacy_next_action(
                "aw capability run work-item-planning --project jet",
                "irrelevant",
            ),
            Some("aw goal capability work-item-planning --project jet".to_string())
        );
    }

    // #1899 R4: the bare project-wide `aw capability run --project <p>
    // --non-interactive --max-ticks 1` form (no capability id) still
    // repairs to the current `aw goal capability --project <p>
    // --non-interactive --max-ticks 1` rollup form.
    #[test]
    fn legacy_capability_run_project_only_next_action_normalizes_to_goal_capability_rollup() {
        assert_eq!(
            normalize_legacy_next_action(
                "aw capability run --project jet --non-interactive --max-ticks 1",
                "irrelevant",
            ),
            Some("aw goal capability --project jet --non-interactive --max-ticks 1".to_string())
        );
    }

    // #921 AC1: a misspelled runner id in an otherwise-valid vat.toml is a
    // blocker naming the vat.toml path and the bad id.
    #[test]
    fn vat_runner_binding_blocks_on_misspelled_runner_id() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join("projects/demo")).unwrap();
        std::fs::write(
            root.join("projects/demo/vat.toml"),
            "[[runners]]\nid = \"meter-perf\"\ncmd = [\"../../target/debug/meter\", \"test\"]\ntimeout_s = 600\n",
        )
        .unwrap();

        let (blockers, warnings) = check_ec_vat_runner_binding(
            root,
            "efficiency",
            "cd projects/demo && ../../target/debug/vat run meter-perf-typo",
        );
        assert!(warnings.is_empty());
        assert_eq!(blockers.len(), 1);
        assert!(blockers[0].contains("projects/demo/vat.toml"));
        assert!(blockers[0].contains("meter-perf-typo"));
    }

    // #921 AC1: a good runner id whose cmd[0] binary is present on disk is
    // clean — no blocker, no warning.
    #[test]
    fn vat_runner_binding_clean_when_binary_present() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join("projects/demo/bin")).unwrap();
        std::fs::write(root.join("projects/demo/bin/meter"), "").unwrap();
        std::fs::write(
            root.join("projects/demo/vat.toml"),
            "[[runners]]\nid = \"meter-perf\"\ncmd = [\"bin/meter\", \"test\"]\ntimeout_s = 600\n",
        )
        .unwrap();

        let (blockers, warnings) = check_ec_vat_runner_binding(
            root,
            "efficiency",
            "cd projects/demo && ../../target/debug/vat run meter-perf",
        );
        assert!(blockers.is_empty());
        assert!(warnings.is_empty());
    }

    // #921: a good runner id whose cmd[0] binary is missing on disk is
    // warn-only ("buildable, not built") — it must not block `clean`.
    #[test]
    fn vat_runner_binding_warns_when_binary_missing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join("projects/demo")).unwrap();
        std::fs::write(
            root.join("projects/demo/vat.toml"),
            "[[runners]]\nid = \"meter-perf\"\ncmd = [\"../../target/debug/meter\", \"test\"]\ntimeout_s = 600\n",
        )
        .unwrap();

        let (blockers, warnings) = check_ec_vat_runner_binding(
            root,
            "efficiency",
            "cd projects/demo && ../../target/debug/vat run meter-perf",
        );
        assert!(blockers.is_empty());
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("meter-perf"));
        assert!(warnings[0].contains("not built yet"));
    }

    // A bare `cmd[0]` command name (no path separator) is resolved via `PATH`
    // at run time, not relative to `dir` — never on-disk-checked, so it never
    // produces a warning even though it doesn't exist as a file next to the
    // runner's workdir. Regression coverage for a real false positive found
    // smoke-testing against apps/lumen/vat.toml (`cmd = ["sh", "-c", ...]`).
    #[test]
    fn vat_runner_binding_skips_bare_command_name() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join("projects/demo")).unwrap();
        std::fs::write(
            root.join("projects/demo/vat.toml"),
            "[[runners]]\nid = \"rig-resilience\"\ncmd = [\"sh\", \"-c\", \"echo hi\"]\ntimeout_s = 600\n",
        )
        .unwrap();

        let (blockers, warnings) = check_ec_vat_runner_binding(
            root,
            "stability",
            "cd projects/demo && ../../target/debug/vat run rig-resilience",
        );
        assert!(blockers.is_empty());
        assert!(warnings.is_empty());
    }

    // A command that isn't shaped like `cd <dir> && vat run <runner-id>` is
    // out of tier 1b's narrow scope — never a blocker or a warning.
    #[test]
    fn non_vat_runner_shaped_command_is_not_applicable() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        let (blockers, warnings) =
            check_ec_vat_runner_binding(root, "behavior", "cargo test -p demo");
        assert!(blockers.is_empty());
        assert!(warnings.is_empty());
    }

    // A missing vat.toml file (dangling `cd <dir>`) is also a blocker — a
    // binding pointing at a directory with no vat.toml at all is a static
    // mistake tier 1b should catch, not just a misspelled id within one.
    #[test]
    fn vat_runner_binding_blocks_on_missing_vat_toml() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join("projects/demo")).unwrap();

        let (blockers, warnings) = check_ec_vat_runner_binding(
            root,
            "efficiency",
            "cd projects/demo && ../../target/debug/vat run meter-perf",
        );
        assert!(warnings.is_empty());
        assert_eq!(blockers.len(), 1);
        assert!(blockers[0].contains("projects/demo/vat.toml"));
    }
}
// CODEGEN-END
