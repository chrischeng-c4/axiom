// SPEC-MANAGED: apps/agentic-workflow/tech-design/logic/aw-llm-offline-agent-orientation-command.md
// HANDWRITE-BEGIN aw-llm-orientation-surface
//! `aw llm` -- offline, binary-emitted agent orientation.
//!
//! The narrative complement to aw's machine-schema surface (the `aw.cli.v1`
//! envelope). It prints orientation topics an agent reads to understand how
//! to drive aw -- read-only, offline, deterministic, no model call. Per-verb
//! flag syntax is owned by clap `--help`; this surface never restates it.
//!
//! @spec apps/agentic-workflow/tech-design/logic/aw-llm-offline-agent-orientation-command.md
//! @spec apps/agentic-workflow/tech-design/surface/specs/aw-agent-prompt-contract.md

use crate::Result;
use clap::{Args, ValueEnum};
#[cfg(test)]
use clap::{Command, Subcommand};

/// Which agent-orientation topic to print.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LlmTopic {
    /// The loop model + topic map. Agents start here.
    Outline,
    /// The product: one agent-first project-iteration CLI and its ownership.
    Model,
    /// The goal: capability defines function; what to build + is it ready.
    Capability,
    /// The artifact: spec defines how; td code is what runs (caps-agnostic).
    Td,
    /// The verifier: ec defines what to test; ec green is the only gate.
    Ec,
    /// The loop state + engine: wi carries goal/verifier/iterations/
    /// last_result/next_action; `aw goal wi` iterates until ec green.
    Wi,
    /// The loop verb: the closed four-leaf root-type enum (wi / capability /
    /// backlog / adhoc) and which verifier each one names.
    Goal,
    /// The prompt projection: closed vocabulary, symbolic grammar, and the
    /// boundary that keeps the workflow engine authoritative.
    Prompt,
}

/// Output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LlmFormat {
    /// Human/agent-readable Markdown (default).
    Md,
    /// Machine-readable JSON topic or outline object.
    Json,
}

/// Print agent-facing orientation topics -- offline, no server, no model.
/// `outline` maps the topics; `model` defines the product boundary;
/// `capability` / `td` / `ec` are the three pillars; `wi` is how to operate
/// aw. Markdown by default; `--format
/// json` for a machine-readable form. For exact flags of any verb, run
/// `aw <verb> --help` -- this surface is orientation, not reference.
#[derive(Debug, Args, Clone)]
pub struct LlmArgs {
    /// Which topic to print.
    #[arg(long, value_enum, default_value_t = LlmTopic::Outline)]
    pub topic: LlmTopic,

    /// Output format.
    #[arg(long, value_enum, default_value_t = LlmFormat::Md)]
    pub format: LlmFormat,
}

const TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "model",
        summary:
            "the product: one agent-first CLI owns guidance, skeletons, strict phases, and codegen",
        body: MODEL_MD,
    },
    cli_std::llm::Topic {
        id: "capability",
        summary: "the goal: what to build and whether it is ready",
        body: CAPABILITY_MD,
    },
    cli_std::llm::Topic {
        id: "td",
        summary: "the artifact: how the implementation is authored and generated",
        body: TD_MD,
    },
    cli_std::llm::Topic {
        id: "ec",
        summary: "the verifier: what gets tested and what decides done",
        body: EC_MD,
    },
    cli_std::llm::Topic {
        id: "wi",
        summary: "the loop state and how to operate the aw goal envelope",
        body: WI_MD,
    },
    cli_std::llm::Topic {
        id: "goal",
        summary: "the loop verb: the four root types and their verifiers",
        body: GOAL_MD,
    },
    cli_std::llm::Topic {
        id: "prompt",
        summary: "the projection: typed lifecycle state rendered for an agent",
        body: PROMPT_MD,
    },
];

pub fn run(args: LlmArgs) -> Result<()> {
    let out = cli_std::llm::render(
        "aw",
        env!("AW_BUILD_VERSION"),
        TOPICS,
        topic_name(args.topic),
        cli_std_format(args.format),
    )?;
    println!("{out}");
    Ok(())
}

fn cli_std_format(format: LlmFormat) -> cli_std::llm::Format {
    match format {
        LlmFormat::Md => cli_std::llm::Format::Md,
        LlmFormat::Json => cli_std::llm::Format::Json,
    }
}

/// The stable string name of a topic (matches the CLI value).
fn topic_name(topic: LlmTopic) -> &'static str {
    match topic {
        LlmTopic::Outline => "outline",
        LlmTopic::Model => "model",
        LlmTopic::Capability => "capability",
        LlmTopic::Td => "td",
        LlmTopic::Ec => "ec",
        LlmTopic::Wi => "wi",
        LlmTopic::Goal => "goal",
        LlmTopic::Prompt => "prompt",
    }
}

const MODEL_MD: &str = r#"# aw llm --topic model -- the product boundary

Agentic Workflow (`aw`) is an agent-first project-iteration CLI for coding agents. It owns next-action guidance, durable artifact skeletons, strict format and phase validation, and code generation.

## Public model

- Project owns the repository-side product scope and rollup root.
- Capability is the META-doc goal contract.
- WorkItem is one bounded iteration and its durable loop state.
- Artifact is an AW-produced skeleton plus declared fill slots.
- Gate evaluates a transition and records Evidence.
- Evidence permits Rollup through WorkItem, Capability, and Project roots.

## Product boundary

- The CLI is the product: stdout owns the unique next command or terminal/HITL marker.
- AW creates supported durable artifact skeletons before an agent fills them.
- Strict format and phase validation runs before durable state advances.
- TD codegen owns generated implementation; EC owns observable verification.
- A parallel collaboration application, general-purpose UI, or alternate workflow protocol is not part of AW.

For the current command surface, run `aw --help`.
"#;

/// The registered top-level verbs, sourced from the `Commands` enum itself so
/// the outline can never drift from the actual CLI. Sorted for determinism.
#[cfg(test)]
fn registered_verbs() -> Vec<String> {
    let cmd = crate::cli::Commands::augment_subcommands(Command::new("aw"));
    let mut verbs: Vec<String> = cmd
        .get_subcommands()
        .map(|c| c.get_name().to_string())
        .collect();
    verbs.sort();
    verbs.dedup();
    verbs
}

const CAPABILITY_MD: &str = r#"# aw llm --topic capability -- the WHAT pillar

A capability is the unit of "what should exist and be production-ready". The
product surface is declared as Markdown capability roots in the project's
CAPABILITIES.md (the default `cap_path`; capability headings + work-root
tables) with a human-readable summary in README; detailed proof lives in
validation inventories and external contracts.

## Mental model

- Capability roots are machine-readable CAPABILITIES.md headings, each with
  an `ID`, surfaces, EC dimensions, a promise, and a work-root table.
  README-resident capability structure is migration input only --
  `aw capability migrate` relocates it.
- Each work-root row is a gap to close and a claim to verify. Its slug is the
  `gap` / `claim` id that TD frontmatter references.
- Readiness is measured, not asserted: a capability is `verified` only when
  its claims have evidence (test gates, EC gates).

## The completion loop

`aw capability` runs report / next / check; driving a capability or
project-wide root to terminal is `aw goal capability` (#1899 -- the old
`aw capability run` clap path is retired and redirects to it):

- `report` -- readiness, production scope, and blockers for the project.
- `next`   -- the next bounded capability action to take.
- `aw goal capability [<cap-id>] --project <p>` -- drive that action (omit
  the id to run the whole project end to end).
- `check`  -- re-evaluate against the contract.

Aggregate readiness across all dimensions lives in `aw health`.

For exact flags, run `aw capability --help`.
"#;

const TD_MD: &str = r#"# aw llm --topic td -- executable design

TD is an executable Python project describing candidate design structure,
interfaces, components, and unit tests. It lowers through Python AST into
Python, Rust, or TypeScript CB targets. TD is **caps-agnostic**: it does not
redefine the capability; it has to satisfy the already-authored EC.

## Mental model

- The staged lifecycle is `EC -> TD -> EC[TD] -> CB -> EC[CB]`.
  `EC[TD]` requires behavior and security green before CB generation.
- `gen` lowers Python TD into code. Every in-scope region is either `CODEGEN`
  (emitted from the spec) or `HANDWRITE` (a named generator gap that codegen
  cannot yet cover). There is no skip state for source ownership.
- Regenerability invariant: delete the codebase, re-run codegen on the TDs,
  replay HANDWRITE blocks, and the tree is byte-equivalent.
- TD and CB use ordinary Python project structure rather than YAML/Mermaid as
  the canonical semantic representation.

## Where it lives

- TDs: `<project>/tech-design/`. Logic/behavior TDs under `logic/`.
- A SPEC-MANAGED source file carries `// @spec <td>` and CODEGEN / HANDWRITE
  markers tying it back to its TD.

For exact flags, run `aw td --help`.
"#;

const EC_MD: &str = r#"# aw llm --topic ec -- executable external contract

EC is what everything trusts. The loop terminates on ec; caps is "achieved"
iff ec is green; td chases ec green. So ec is the one artifact that decides
"done" -- and the one place judgment lives.

## The four dimensions

| dimension   | question                                  |
|-------------|-------------------------------------------|
| behavior    | does it do the right thing? (required)    |
| efficiency  | is it efficient enough for its target?   |
| security    | is it safe?                               |
| stability   | does it hold up under failure / time?     |

## Mental model

- EC is authored first as an ordinary executable Python project. Its code is
  the verifier; it does not need a semantic generation step.
- What to test is DERIVED FROM caps. That derivation is the single human +
  agent collaboration point (HITL) -- and the only place a review belongs,
  because a wrong ec yields a false green nothing downstream can catch.
- The approval path is `draft -> check -> review -> lock`. `needs_revision`
  routes back to bounded edits of the emitted Python inventory/source;
  `accepted` advances to locking and staged verification.
  Production-required EC needs digest-bound independent review evidence.
  `ec_review_backing` (either default, agent-first | agent | human, opt-in
  blocking human-only review) picks who may back it; same-agent self-review
  never counts, and a human audit can always reopen an agent-accepted EC.
  `ec_review_mode = "deferred"` queues a pending human review without
  blocking the runner (#1828/#1829/#1859).
- `EC[TD]` runs behavior and security against TD. After TD generation,
  stability and efficiency are added; `EC[CB]` runs all applicable dimensions
  against CB. Rust targets require efficiency by default.
- Wired per project via `aw.toml` `ec.<category>`; absent -> the
  default test gate. Non-capability scope (delivery, docs) has no behavior ec
  and rides a zero-EC / cold-build lane instead.
- `aw health --verify-ec` evaluates the dimensions required for production.

For exact flags, run `aw ec --help`.
"#;

const WI_MD: &str = r#"# aw llm --topic wi -- the loop state + how to operate the loop

A work-item IS the loop's durable state. You operate aw by reading one JSON
envelope (schema `aw.cli.v1`) and running the command it hands back, until the
loop converges on ec green.

## Terminology-first work-item types

Each closed-enum type is defined by its terminal state:

| type | terminal state |
|------|----------------|
| `epic` | all owned children are terminal |
| `change` | EC is green for the generated codebase and the lifecycle closes the change |
| `spike` | an ADR-style decision records spawned WI refs or explicit no-action; expiry converges to `gave_up` |
| `report` | typed `triage` either accepts and links a spawned change/epic, or closes as `duplicate`, `invalid`, or `by-design` |

Only `change` is executable backlog work. A `spike` is a timeboxed
investigation, never product-source implementation. A `report` enters the
project's intake queue until triage. Both converge by spawn-and-link rather
than changing type in place.

## The loop state (carried in the WI)

- `goal`        -- the capability gap this loop closes.
- `verifier`    -- the ec gate that decides done.
- `iterations`  -- the running log of act/verify passes.
- `last_result` -- none | green | red{dimension, why} | blocked{reason}.
- `next_action` -- the command to run next, derived from last_result.
- `status`      -- iterating | converged | blocked | failed.
- `tried`       -- failed approaches, so the loop does not repeat one.

## The decision (driven by ec, not review)

    ec green  -> converged   -> aw td code-check <wi>
    ec red    -> iterating   -> repair the owner named by the staged verifier
    blocked   -> HITL        -> surface hitl_question to a human

## The envelope

- `next.command` is the only command to run next; re-run the root after each
  child completes; stop when `completion.workflow_complete=true`.
- `completion.requires_hitl=true` -> stop and ask a human.

Drive it: `aw goal wi <id>` for one work item, or `aw goal capability
<capability-id> --project <project>` for a capability's work-root queue (see
the `goal` topic for the full root-type map); the linear authoring path is
`skeleton -> fill -> validate`; unresolved product decisions become HITL.
There is no WI review or arbitration phase. The implementation path is
`wi -> ec draft/check/review/lock -> td -> ec[td] -> cb -> ec[cb] -> code-check
-> parent rollup`; capability is the META-doc goal ledger and `aw health` is
read-only observation, not an authoring step.

Capability-to-WI planning has one semantic approval boundary because it can
publish tracker work: `aw wi plan` emits a digest-bound review payload and
`aw wi plan-review --evidence-file <path>` consumes it. The default/either
policy is independent-agent-first; `capability_plan_review_backing = "human"`
is the explicit blocking human-only opt-in. Accepted review publishes only
bounded, deduplicated claim WIs; `needs_revision` publishes nothing.

For exact flags, run `aw goal wi --help`, `aw goal capability --help`, or
`aw wi --help`.
"#;

const GOAL_MD: &str = r#"# aw llm --topic goal -- the loop verb (four root types, one mental model)

`aw goal` is aw's single loop verb (#1899). Every invocation names a root
and a verifier: lifecycle roots use the ec/terminal/rollup verifier chain,
the ad-hoc root uses gate commands. The root-type set is a closed
four-leaf enum -- never a fifth.

## The four leaves

| kind         | CLI form                                              | verifier |
|--------------|--------------------------------------------------------|----------|
| `wi`         | `aw goal wi <id>`                                       | lifecycle chain of that root (ec / terminal / rollup) |
| `capability` | `aw goal capability [<capability-id>] --project <p>`    | capability work-root closure / project promise rollup |
| `backlog`    | `aw goal backlog --project <p>`                         | zero open unparked WIs for the project |
| `adhoc`      | `aw goal set --gate "<cmd>" <intent>` -> `aw goal check` | every recorded gate command exits 0 |

## Mental model

- `wi` and `capability` are the re-homed lifecycle runners (formerly the
  now-retired `aw wi run` / `aw capability run` verbs): envelope semantics
  (`aw.cli.v1`, `invoke.command`, `agent_prompt`,
  `completion.workflow_complete`, `completion.requires_hitl`,
  `hitl_question`) carry over unchanged -- this was a re-homing, not a
  redesign.
- `backlog` is a tracker-driven drain of every open WI for a project, one
  WI per tick through the same engine `wi` uses; a WI that hits HITL or a
  hard blocker is parked (not surfaced) so the drain continues, and the
  terminal envelope reports the parked set for human follow-up.
- `adhoc` is for bounded work OUTSIDE the WI/TD/EC lifecycle (test-pass
  gates, migration sweeps): record one or more machine-runnable gate
  commands with `aw goal set`, then poll `aw goal check` until `done` or
  `gave_up`. Prose alone is never a gate.
- Never invent a fifth leaf. A retired top-level runner invocation
  (`aw wi run` / `aw capability run` / the older top-level `aw run`)
  returns an error envelope naming the exact `aw goal` replacement.

For exact flags, run `aw goal --help`, `aw goal wi --help`, `aw goal
capability --help`, or `aw goal backlog --help`.
"#;

const PROMPT_MD: &str = r#"# aw llm --topic prompt -- aw.prompt.v1

`aw.prompt.v1` is the typed projection of lifecycle state into a concise agent
instruction. It is descriptive only: the AW workflow engine is the sole owner
of state, transition selection, mutation, and completion. Never evaluate
the prompt as Python and never invent a command absent from `next.command` or
`invoke.command`.

## Closed vocabulary

- truth: `unknown` (not yet validly verified), `red` (valid verifier rejected),
  `green` (valid verifier accepted)
- terminal level: `stage terminal`, `change closed`, `root complete`
- owner: `EC`, `TD`, or `CB`; invalid oracle/evidence belongs to EC, while a
  valid red gate belongs to the target it rejected
- blocker: `decision`, `approval`, `environment`, `red_gate`,
  `missing_evidence`

`action == done` can mean a child stage ended.
Only `completion.workflow_complete == true` means root complete.

## Closed ASCII grammar

| operator | meaning |
|---|---|
| `A -> B` | workflow selects B after A |
| `A --gate-> V == green` | verifier V must be green before transition |
| `x := value` | bind a projection-local name |
| `==`, `!=` | equality predicates |
| `in`, `notin` | finite membership predicates |

The canonical operator set is exactly `->`, `--gate->`, `:=`, `==`, `!=`,
`in`, and `notin`.

## Python Spec pipeline

```text
EC := unknown
EC -> TD
TD --gate-> EC[TD].behavior == green
TD --gate-> EC[TD].security == green
EC[TD] -> CB
CB --gate-> EC[CB].behavior == green
CB --gate-> EC[CB].security == green
CB --gate-> EC[CB].stability == green
CB --gate-> EC[CB].efficiency in {green, not-applicable}
completion.workflow_complete == true
```

EC and TD are ordinary executable Python projects. EC is authored first. TD
lowers through Python AST to Python, Rust, or TypeScript. CB lives under
`src/*`, is grouped by domain, and includes unit tests. TD-stage verification
requires behavior and security; CB-stage verification requires all applicable
dimensions, with efficiency required by default for Rust targets.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    // @spec aw-llm-offline-agent-orientation-command.md R1
    #[test]
    fn llm_outline_uses_cli_std_and_standard_commands() {
        let verbs = registered_verbs();
        assert!(
            ["llm", "upgrade", "issue"]
                .iter()
                .all(|want| verbs.iter().any(|verb| verb == want)),
            "registered verbs should include standard CLI commands, got {verbs:?}"
        );
        let outline = cli_std::llm::render(
            "aw",
            env!("AW_BUILD_VERSION"),
            TOPICS,
            "outline",
            cli_std::llm::Format::Md,
        )
        .unwrap();

        assert!(outline.contains("aw upgrade"));
        assert!(outline.contains("aw issue"));
        assert!(outline.contains("`model`"));
        assert!(outline.contains("`capability`"));
    }

    // @spec aw-llm-offline-agent-orientation-command.md R2
    #[test]
    fn llm_every_topic_emits_markdown() {
        for topic in [
            LlmTopic::Outline,
            LlmTopic::Model,
            LlmTopic::Capability,
            LlmTopic::Td,
            LlmTopic::Ec,
            LlmTopic::Wi,
            LlmTopic::Goal,
            LlmTopic::Prompt,
        ] {
            let md = cli_std::llm::render(
                "aw",
                env!("AW_BUILD_VERSION"),
                TOPICS,
                topic_name(topic),
                cli_std::llm::Format::Md,
            )
            .unwrap();
            assert!(
                md.trim_start().starts_with("# aw"),
                "{} topic must emit an orientation heading",
                topic_name(topic)
            );
            assert!(
                md.len() > 200,
                "{} topic must emit non-empty orientation content",
                topic_name(topic)
            );
        }
    }

    // @spec aw-llm-offline-agent-orientation-command.md R3
    #[test]
    fn llm_format_json_uses_cli_std_shape() {
        let outline = cli_std::llm::render(
            "aw",
            env!("AW_BUILD_VERSION"),
            TOPICS,
            "outline",
            cli_std::llm::Format::Json,
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&outline).unwrap();

        assert_eq!(value["project"], "aw");
        assert!(value["topics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|topic| topic["id"] == "capability"));
        assert!(value["topics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|topic| topic["id"] == "model"));

        let topic = cli_std::llm::render(
            "aw",
            env!("AW_BUILD_VERSION"),
            TOPICS,
            "wi",
            cli_std::llm::Format::Json,
        )
        .unwrap();
        let topic_value: serde_json::Value = serde_json::from_str(&topic).unwrap();
        assert_eq!(topic_value["topic"], "wi");
        assert!(topic_value["body"].as_str().unwrap().contains("loop state"));
    }

    // @spec aw-llm-offline-agent-orientation-command.md R4
    #[test]
    fn llm_topics_are_deterministic() {
        for topic in [
            LlmTopic::Outline,
            LlmTopic::Model,
            LlmTopic::Capability,
            LlmTopic::Td,
            LlmTopic::Ec,
            LlmTopic::Wi,
            LlmTopic::Goal,
            LlmTopic::Prompt,
        ] {
            assert_eq!(
                cli_std::llm::render(
                    "aw",
                    env!("AW_BUILD_VERSION"),
                    TOPICS,
                    topic_name(topic),
                    cli_std::llm::Format::Md,
                )
                .unwrap(),
                cli_std::llm::render(
                    "aw",
                    env!("AW_BUILD_VERSION"),
                    TOPICS,
                    topic_name(topic),
                    cli_std::llm::Format::Md,
                )
                .unwrap(),
                "{} topic must be pure and deterministic",
                topic_name(topic)
            );
        }
    }

    #[test]
    fn prompt_topic_public_renderer_pins_closed_language() {
        let md = cli_std::llm::render(
            "aw",
            env!("AW_BUILD_VERSION"),
            TOPICS,
            "prompt",
            cli_std::llm::Format::Md,
        )
        .unwrap();
        let json = cli_std::llm::render(
            "aw",
            env!("AW_BUILD_VERSION"),
            TOPICS,
            "prompt",
            cli_std::llm::Format::Json,
        )
        .unwrap();
        let body = serde_json::from_str::<serde_json::Value>(&json).unwrap()["body"]
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!(body, md);

        let vocabulary = md
            .split_once("## Closed vocabulary\n\n")
            .unwrap()
            .1
            .split_once("\n\n## Closed ASCII grammar")
            .unwrap()
            .0;
        assert_eq!(
            vocabulary,
            r#"- truth: `unknown` (not yet validly verified), `red` (valid verifier rejected),
  `green` (valid verifier accepted)
- terminal level: `stage terminal`, `change closed`, `root complete`
- owner: `EC`, `TD`, or `CB`; invalid oracle/evidence belongs to EC, while a
  valid red gate belongs to the target it rejected
- blocker: `decision`, `approval`, `environment`, `red_gate`,
  `missing_evidence`

`action == done` can mean a child stage ended.
Only `completion.workflow_complete == true` means root complete."#
        );

        let grammar = md
            .split_once("## Closed ASCII grammar\n\n")
            .unwrap()
            .1
            .split_once("\n\n## Python Spec pipeline")
            .unwrap()
            .0;
        assert_eq!(
            grammar,
            "| operator | meaning |\n\
|---|---|\n\
| `A -> B` | workflow selects B after A |\n\
| `A --gate-> V == green` | verifier V must be green before transition |\n\
| `x := value` | bind a projection-local name |\n\
| `==`, `!=` | equality predicates |\n\
| `in`, `notin` | finite membership predicates |\n\n\
The canonical operator set is exactly `->`, `--gate->`, `:=`, `==`, `!=`,\n\
`in`, and `notin`."
        );
        assert!(md.contains(
            "the AW workflow engine is the sole owner\nof state, transition selection, mutation, and completion"
        ));

        let expected_pipeline = "```text\n\
EC := unknown\n\
EC -> TD\n\
TD --gate-> EC[TD].behavior == green\n\
TD --gate-> EC[TD].security == green\n\
EC[TD] -> CB\n\
CB --gate-> EC[CB].behavior == green\n\
CB --gate-> EC[CB].security == green\n\
CB --gate-> EC[CB].stability == green\n\
CB --gate-> EC[CB].efficiency in {green, not-applicable}\n\
completion.workflow_complete == true\n\
```";
        assert!(md.contains(expected_pipeline));
        for lookalike in ['→', '⇒', '⟶', '∈', '≠', '≔'] {
            assert!(
                !md.contains(lookalike),
                "public prompt contains non-canonical operator `{lookalike}`"
            );
        }
    }

    /// #1496: binary orientation and canonical active contracts teach one
    /// agent-first CLI product. Stable legacy machine ids and paths remain for
    /// traceability, but removed product semantics may not return in prose.
    #[test]
    fn agent_first_product_contracts_reject_removed_architecture() {
        let active_contracts = [
            ("aw llm model", MODEL_MD),
            ("README", include_str!("../../README.md")),
            ("CAPABILITIES", include_str!("../../CAPABILITIES.md")),
            (
                "project iteration model TD",
                include_str!("../../tech-design/surface/specs/aw-core-client-model.md"),
            ),
            (
                "CLI product boundary TD",
                include_str!("../../tech-design/surface/specs/aw-client-boundaries.md"),
            ),
        ];

        for (name, contract) in active_contracts {
            let normalized = contract.to_ascii_lowercase();
            // README is human-first under the META-doc audience partition
            // (#1816): it must not advertise removed semantics, but the
            // canonical responsibility phrasing lives in CAPABILITIES.md and
            // the TD contracts, not in human prose.
            if name != "README" {
                for required in [
                    "agent-first project-iteration cli",
                    "next-action guidance",
                    "durable artifact skeletons",
                    "strict format",
                    "code generation",
                ] {
                    assert!(
                        normalized.contains(required),
                        "{name} must contain canonical product responsibility `{required}`",
                    );
                }
            }
            for removed in [
                "cue",
                "multi-client",
                "future client",
                "client-independent",
                "repo view desktop app",
            ] {
                assert!(
                    !normalized.contains(removed),
                    "{name} still advertises removed product semantics `{removed}`",
                );
            }
        }
    }

    // @spec aw-agent-prompt-contract.md
    #[test]
    fn prompt_topic_defines_closed_language() {
        for term in [
            "aw.prompt.v1",
            "unknown",
            "red",
            "green",
            "stage terminal",
            "change closed",
            "root complete",
            "decision",
            "approval",
            "environment",
            "red_gate",
            "missing_evidence",
            "EC -> TD",
            "EC[TD]",
            "EC[CB]",
            "completion.workflow_complete == true",
            "sole owner",
        ] {
            assert!(
                PROMPT_MD.contains(term),
                "prompt topic must define canonical term `{term}`"
            );
        }
        for operator in ["->", "--gate->", ":=", "==", "!=", "in", "notin"] {
            assert!(
                PROMPT_MD.contains(&format!("`{operator}`")),
                "prompt topic must define canonical operator `{operator}`"
            );
        }
        for stale in [
            "Mermaid Plus",
            "YAML IR",
            "ec skeleton/fill/review/gen",
            "draft -> fill",
            "ec draft/fill",
        ] {
            assert!(
                ![MODEL_MD, TD_MD, EC_MD, WI_MD, GOAL_MD, PROMPT_MD]
                    .iter()
                    .any(|topic| topic.contains(stale)),
                "orientation must not teach stale lifecycle wording `{stale}`"
            );
        }
    }
}
// HANDWRITE-END aw-llm-orientation-surface
