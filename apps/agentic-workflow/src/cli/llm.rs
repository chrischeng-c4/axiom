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

const TD_MD: &str = r#"# aw llm --topic td -- the artifact (how + the running code)

Spec defines how; `td` code is the artifact that runs. Implementation is
generated from the tech-design, and the tree is regenerable. td is
**caps-agnostic**: it does not reference the capability -- it only has to pass
ec. Passing ec verify == achieving caps, so td chases ec green, in any shape.

## Mental model

- The lifecycle is LINEAR: `create -> gen -> fill -> code-check`. There is no
  review/revise step -- the gate is ec, not a reviewer. Logic and unit-test
  sections are Mermaid Plus blocks (YAML IR + a rendered diagram).
- `gen` turns the TD into code. Every in-scope region is either `CODEGEN`
  (emitted from the spec) or `HANDWRITE` (a named generator gap that codegen
  cannot yet cover). There is no skip state for source ownership.
- Regenerability invariant: delete the codebase, re-run codegen on the TDs,
  replay HANDWRITE blocks, and the tree is byte-equivalent.
- Code beauty is irrelevant; ec green is the only truth.

## Where it lives

- TDs: `<project>/tech-design/`. Logic/behavior TDs under `logic/`.
- A SPEC-MANAGED source file carries `// @spec <td>` and CODEGEN / HANDWRITE
  markers tying it back to its TD.

For exact flags, run `aw td --help`.
"#;

const EC_MD: &str = r#"# aw llm --topic ec -- the verifier (the only gate)

EC is what everything trusts. The loop terminates on ec; caps is "achieved"
iff ec is green; td chases ec green. So ec is the one artifact that decides
"done" -- and the one place judgment lives.

## The four dimensions

| dimension   | question                                  |
|-------------|-------------------------------------------|
| behavior    | does it do the right thing? (required)    |
| benchmark   | is it fast enough?                        |
| security    | is it safe?                               |
| stability   | does it hold up under failure / time?     |

## Mental model

- ec is two things: the `doc` defines WHAT to test; the `code` is the verifier
  that runs and yields green / red.
- What to test is DERIVED FROM caps. That derivation is the single human +
  agent collaboration point (HITL) -- and the only place a review belongs,
  because a wrong ec yields a false green nothing downstream can catch.
- The approval path is `draft -> fill -> check -> review`. `needs_revision`
  routes back to bounded `fill`; `accepted` advances to `gen -> verify`.
  Production-required EC needs digest-bound independent review evidence.
  `ec_review_backing` (either default, agent-first | agent | human, opt-in
  blocking human-only review) picks who may back it; same-agent self-review
  never counts, and a human audit can always reopen an agent-accepted EC.
  `ec_review_mode = "deferred"` queues a pending human review without
  blocking the runner (#1828/#1829/#1859).
- ec green is the only code-check gate. Code style / fmt are not gates.
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
    ec red    -> iterating   -> aw td gen <wi> (adapt; never re-run the same fail)
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
`wi -> ec skeleton/fill/review/gen -> td/codegen -> ec verify -> code-check
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
}
// HANDWRITE-END aw-llm-orientation-surface
