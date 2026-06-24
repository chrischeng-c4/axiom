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
    /// Axiom + one-crown-two-axes model + topic map. Agents start here.
    Outline,
    /// The capability pillar: product surface, completion loop, readiness.
    Capability,
    /// The inward axis: spec-is-truth, td -> gen, regenerable source.
    Td,
    /// The outward axis: external contract, generated gates, opt-in.
    Ec,
    /// How to operate aw: the envelope contract, wi -> td -> merge, HITL.
    Loop,
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
        LlmTopic::Loop => "loop",
    }
}

fn topic_markdown(topic: LlmTopic) -> String {
    match topic {
        LlmTopic::Outline => outline_md(),
        LlmTopic::Capability => capability_md(),
        LlmTopic::Td => td_md(),
        LlmTopic::Ec => ec_md(),
        LlmTopic::Loop => loop_md(),
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

aw drives spec-driven development. One axiom underlies every verb:

    declare in a spec  ->  generate the artifact  ->  measure readiness

aw is mechanical: it never calls a model. Each command emits a JSON envelope
(schema `aw.cli.v1`) carrying `next.command` and `agent_prompt`; YOU run the
model and re-run the root command until `completion.workflow_complete=true`.

## The model: one crown, two axes

                 capability        WHAT should exist + is it production-ready
                /          \
        td + gen            ec + gen
        inward              outward
        "built right?"      "safe for consumers?"

- capability -- the product surface. Work = completing capabilities to ready.
- td + gen   -- spec is the source of truth; implementation is generated from
                the tech-design and is regenerable (no hand-drift).
- ec + gen   -- the external contract across correctness / benchmark /
                security / stability; its gates are generated and run as
                production gates.

`gen` is not a fourth pillar -- it is the shared verb of td and ec. You
declare, you generate; you never hand-author the artifact.

## Topics -- read the smallest one you need

| topic             | read it when ...                                        |
|-------------------|---------------------------------------------------------|
| aw llm capability | you need to know what to build and whether it's ready   |
| aw llm td         | you are designing / generating implementation from a TD |
| aw llm ec         | you are guarding the external contract and its gates    |
| aw llm loop       | you need to operate aw: envelope, wi -> td -> merge, HITL|

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
    r#"# aw llm td -- the inward axis (built right?)

Spec is the source of truth; code is a derived artifact. Implementation is
generated from the tech-design, and the tree is regenerable.

## Mental model

- A TD is authored in phases (applicability -> contract) and reviewed before
  it generates code. Logic and unit-test sections are Mermaid Plus blocks
  (YAML IR + a rendered diagram).
- `gen` turns the TD into code. Every in-scope region is either `CODEGEN`
  (emitted from the spec) or `HANDWRITE` (a named generator gap that codegen
  cannot yet cover). There is no skip state for source ownership.
- Regenerability invariant: delete the codebase, re-run codegen on the TDs,
  replay HANDWRITE blocks, and the tree is byte-equivalent.
- When a gap-blocker lands, `HANDWRITE` -> `CODEGEN` and the invariant
  tightens.

## Where it lives

- TDs: `<project>/tech-design/`. Logic/behavior TDs under `logic/`.
- A SPEC-MANAGED source file carries `// @spec <td>` and CODEGEN / HANDWRITE
  markers tying it back to its TD.

For exact flags, run `aw td --help`.
"#
    .to_string()
}

fn ec_md() -> String {
    r#"# aw llm ec -- the outward axis (safe for consumers?)

The External Contract is what keeps aw-managed projects from breaking the
people who depend on them: "don't break things for consumers". aw is the EC
gatekeeper across four dimensions.

## The four dimensions

| dimension   | question                                  |
|-------------|-------------------------------------------|
| behavior    | does it still do the right thing?         |
| benchmark   | is it still fast enough?                  |
| security    | is it still safe?                         |
| stability   | does it hold up under failure / time?     |

## Mental model

- EC artifacts -- tests, tool configs, gates -- are generated (`gen`), not
  hand-authored, and run as production gates.
- A dimension is wired per project via `.aw/config.toml` `ec.<category>`; when
  a category is absent it falls back to the default cargo-test gate.
- `aw health --verify-ec` evaluates the dimensions required for production.
  A dimension marked `required_for_production=false` is reported but not
  gated.

For exact flags, run `aw ec --help`.
"#
    .to_string()
}

fn loop_md() -> String {
    r#"# aw llm loop -- how to operate aw

aw is mechanical. You operate it by reading one JSON envelope and running the
single command it hands back, in a loop, until it says you are done.

## The envelope contract (schema `aw.cli.v1`)

- `next.command`                 -- the only command to run next.
- `agent_prompt`                 -- guidance for you, the calling agent.
- `completion.workflow_complete` -- stop only when this is `true`.
- `completion.requires_hitl`     -- a human decision is required; surface it.

Re-run the same root command after each child command completes. `action=done`
can mean only the current child root is complete -- inspect the parent.

## The forward loop

    aw wi        -- bound a work-item (Scope / Acceptance Criteria / refs)
      -> aw td   -- author + review the tech-design, then gen code
        -> merge -- aw td merge lands the change

Drive the whole thing with the root runner:

    aw run --wi <id>        (or --project / --capability)

follow `invoke.command` until `workflow_complete=true` or `requires_hitl=true`.

## HITL

When `requires_hitl=true`, stop and put the `hitl_question` to a human; do not
guess past a blocked decision.

For exact flags, run `aw run --help` or `aw td --help`.
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
            LlmTopic::Loop,
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
            LlmTopic::Loop,
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
