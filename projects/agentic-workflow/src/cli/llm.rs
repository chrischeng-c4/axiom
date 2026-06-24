// SPEC-MANAGED: projects/agentic-workflow/tech-design/logic/aw-llm-offline-agent-orientation-command.md
// HANDWRITE-BEGIN aw-llm-orientation-surface
//! `aw llm` -- offline, binary-emitted agent orientation.
//!
//! The narrative complement to aw's machine-schema surface (the `aw.cli.v1`
//! envelope). It prints orientation topics an agent reads to understand how
//! to drive aw -- read-only, offline, deterministic, no model call. Per-verb
//! flag syntax is owned by clap `--help`; this surface never restates it.
//!
//! @spec projects/agentic-workflow/tech-design/logic/aw-llm-offline-agent-orientation-command.md

use crate::Result;
use clap::{Args, Command, Subcommand, ValueEnum};

/// Which agent-orientation topic to print.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LlmTopic {
    /// The loop model + topic map. Agents start here.
    Outline,
    /// The goal: capability defines function; what to build + is it ready.
    Capability,
    /// The artifact: spec defines how; td code is what runs (caps-agnostic).
    Td,
    /// The verifier: ec defines what to test; ec green is the only gate.
    Ec,
    /// The loop state + engine: wi carries goal/verifier/iterations/
    /// last_result/next_action; aw run iterates until ec green.
    Wi,
}

/// Output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LlmFormat {
    /// Human/agent-readable Markdown (default).
    Md,
    /// Machine-readable `{ "topic", "markdown" }` object.
    Json,
}

/// Print agent-facing orientation topics -- offline, no server, no model.
/// `outline` maps the topics; `capability` / `td` / `ec` are the three
/// pillars; `loop` is how to operate aw. Markdown by default; `--format
/// json` for a machine-readable form. For exact flags of any verb, run
/// `aw <verb> --help` -- this surface is orientation, not reference.
#[derive(Debug, Args, Clone)]
pub struct LlmArgs {
    /// Which topic to print.
    #[arg(value_enum, default_value_t = LlmTopic::Outline)]
    pub topic: LlmTopic,

    /// Output format.
    #[arg(long, value_enum, default_value_t = LlmFormat::Md)]
    pub format: LlmFormat,
}

pub fn run(args: LlmArgs) -> Result<()> {
    let markdown = topic_markdown(args.topic);
    match args.format {
        LlmFormat::Md => println!("{markdown}"),
        LlmFormat::Json => {
            let value = serde_json::json!({
                "topic": topic_name(args.topic),
                "markdown": markdown,
            });
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
    }
    Ok(())
}

/// The stable string name of a topic (matches the CLI value).
fn topic_name(topic: LlmTopic) -> &'static str {
    match topic {
        LlmTopic::Outline => "outline",
        LlmTopic::Capability => "capability",
        LlmTopic::Td => "td",
        LlmTopic::Ec => "ec",
        LlmTopic::Wi => "wi",
    }
}

fn topic_markdown(topic: LlmTopic) -> String {
    match topic {
        LlmTopic::Outline => outline_md(),
        LlmTopic::Capability => capability_md(),
        LlmTopic::Td => td_md(),
        LlmTopic::Ec => ec_md(),
        LlmTopic::Wi => wi_md(),
    }
}

/// The registered top-level verbs, sourced from the `Commands` enum itself so
/// the outline can never drift from the actual CLI. Sorted for determinism.
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

fn outline_md() -> String {
    let verbs = registered_verbs().join(" ");
    format!(
        r#"# aw -- agent orientation

aw is a loop. You operate it by reading one JSON envelope (schema `aw.cli.v1`)
and running the single command it hands back, until the loop converges:

    read wi (state) -> do td (act) -> run ec (verify) -> write result to wi -> repeat

aw is mechanical: it never calls a model. The loop terminates on the VERIFIER
(ec), not on a review.

## The model: a loop over three artifacts

| layer       | what it is                            | role               |
|-------------|---------------------------------------|--------------------|
| `aw` (run)  | the loop engine                       | act->verify->decide|
| `aw wi`     | the loop STATE + iteration target     | persists the loop  |
| `aw caps`   | the goal (function definition)        | what "done" means  |
| `aw ec`     | the verifier (what to test); ec green | the only gate      |
| `aw td`     | the artifact (how + the running code) | caps-agnostic      |

One sentence: aw (loop) reads wi (state+target) -> does td (act) -> runs ec
(verify) -> writes the result back to wi -> repeats until ec is green = the
caps gap is closed. td may be any shape; only ec decides done. There is no
review -- you terminate on the verifier. (The one judgment point is deriving
ec from caps; see `aw llm ec`.)

## Topics -- read the smallest one you need

| topic             | read it when ...                                        |
|-------------------|---------------------------------------------------------|
| aw llm capability | you need the goal: what to build and whether it's ready |
| aw llm ec         | you are defining/guarding what gets tested (the gate)   |
| aw llm td         | you are authoring/generating the implementation         |
| aw llm wi         | you need to operate the loop: state, next_action, HITL  |

## Registered verbs

{verbs}

For exact flags/args of any verb, run `aw <verb> --help`. This surface is
orientation, not reference -- it never restates flag syntax.
"#
    )
}

fn capability_md() -> String {
    r#"# aw llm capability -- the WHAT pillar

A capability is the unit of "what should exist and be production-ready". The
product surface is declared as Markdown capability roots in the project
README (capability headings + work-root tables); detailed proof lives in
validation inventories and external contracts.

## Mental model

- Capability roots are machine-readable README headings, each with an `ID`,
  surfaces, EC dimensions, a promise, and a work-root table.
- Each work-root row is a gap to close and a claim to verify. Its slug is the
  `gap` / `claim` id that TD frontmatter references.
- Readiness is measured, not asserted: a capability is `verified` only when
  its claims have evidence (test gates, EC gates).

## The completion loop

`aw capability` runs report / next / run / check:

- `report` -- readiness, production scope, and blockers for the project.
- `next`   -- the next bounded capability action to take.
- `run`    -- drive that action.
- `check`  -- re-evaluate against the contract.

Aggregate readiness across all dimensions lives in `aw health`.

For exact flags, run `aw capability --help`.
"#
    .to_string()
}

fn td_md() -> String {
    r#"# aw llm td -- the artifact (how + the running code)

Spec defines how; `td` code is the artifact that runs. Implementation is
generated from the tech-design, and the tree is regenerable. td is
**caps-agnostic**: it does not reference the capability -- it only has to pass
ec. Passing ec verify == achieving caps, so td chases ec green, in any shape.

## Mental model

- The lifecycle is LINEAR: `create -> gen -> merge`. There is no review/revise
  step -- the gate is ec, not a reviewer. Logic and unit-test sections are
  Mermaid Plus blocks (YAML IR + a rendered diagram).
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
"#
    .to_string()
}

fn ec_md() -> String {
    r#"# aw llm ec -- the verifier (the only gate)

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
- ec green is the only merge gate. Code style / fmt are not gates.
- Wired per project via `.aw/config.toml` `ec.<category>`; absent -> the
  default test gate. Non-capability scope (delivery, docs) has no behavior ec
  and rides a zero-EC / cold-build lane instead.
- `aw health --verify-ec` evaluates the dimensions required for production.

For exact flags, run `aw ec --help`.
"#
    .to_string()
}

fn wi_md() -> String {
    r#"# aw llm wi -- the loop state + how to operate the loop

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

    ec green  -> converged   -> aw td merge
    ec red    -> iterating   -> aw td gen      (adapt; never re-run the same fail)
    blocked   -> HITL        -> surface hitl_question to a human

## The envelope

- `next.command` is the only command to run next; re-run the root after each
  child completes; stop when `completion.workflow_complete=true`.
- `completion.requires_hitl=true` -> stop and ask a human.

Drive it: `aw run --wi <id>` (or `--project` / `--capability`); the linear
forward path is `wi -> td -> merge`.

For exact flags, run `aw run --help` or `aw wi --help`.
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // @spec aw-llm-offline-agent-orientation-command.md R1
    #[test]
    fn llm_outline_lists_registered_verbs() {
        let verbs = registered_verbs();
        assert!(
            verbs.iter().any(|v| v == "td") && verbs.iter().any(|v| v == "ec"),
            "registered verbs should include td and ec, got {verbs:?}"
        );
        let outline = outline_md();
        for verb in &verbs {
            assert!(
                outline.contains(verb.as_str()),
                "outline must list registered verb `{verb}` so it cannot drift"
            );
        }
    }

    // @spec aw-llm-offline-agent-orientation-command.md R2
    #[test]
    fn llm_every_topic_emits_markdown() {
        for topic in [
            LlmTopic::Outline,
            LlmTopic::Capability,
            LlmTopic::Td,
            LlmTopic::Ec,
            LlmTopic::Wi,
        ] {
            let md = topic_markdown(topic);
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
    fn llm_format_json_wraps_markdown() {
        let markdown = topic_markdown(LlmTopic::Outline);
        let value = serde_json::json!({
            "topic": topic_name(LlmTopic::Outline),
            "markdown": markdown,
        });
        assert_eq!(value["topic"], "outline");
        assert!(value["markdown"]
            .as_str()
            .unwrap()
            .contains("agent orientation"));
    }

    // @spec aw-llm-offline-agent-orientation-command.md R4
    #[test]
    fn llm_topics_are_deterministic() {
        for topic in [
            LlmTopic::Outline,
            LlmTopic::Capability,
            LlmTopic::Td,
            LlmTopic::Ec,
            LlmTopic::Wi,
        ] {
            assert_eq!(
                topic_markdown(topic),
                topic_markdown(topic),
                "{} topic must be pure and deterministic",
                topic_name(topic)
            );
        }
    }
}
// HANDWRITE-END aw-llm-orientation-surface
