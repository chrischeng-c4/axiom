---
id: libs-cli-std-src-llm-rs
summary: Lossless rust-source-unit coverage for `libs/cli-std/src/llm.rs`.
capability_refs:
  - id: standard-agent-cli-commands
    role: primary
    claim: standard-agent-cli-commands-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Cli Std library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/cli-std/src/llm.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/cli-std/src/llm.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Topic` | libs/cli-std/src/llm.rs | struct | pub | 21 | pub struct Topic { |
| `TopicSection` | libs/cli-std/src/llm.rs | enum | pub | 34 | pub enum TopicSection { |
| `SectionedTopic` | libs/cli-std/src/llm.rs | struct | pub | 49 | pub struct SectionedTopic { |
| `RenderedSection` | libs/cli-std/src/llm.rs | struct | pub | 58 | pub struct RenderedSection { |
| `RenderableTopic` | libs/cli-std/src/llm.rs | trait | pub | 66 | pub trait RenderableTopic { |
| `assert_topics_render` | libs/cli-std/src/llm.rs | function | pub | 132 | pub fn assert_topics_render<T: RenderableTopic>(topics: &[T]) { |
| `Format` | libs/cli-std/src/llm.rs | enum | pub | 167 | pub enum Format { |
| `parse` | libs/cli-std/src/llm.rs | function | pub | 177 | pub fn parse(s: &str) -> Self { |
| `render` | libs/cli-std/src/llm.rs | function | pub | 189 | pub fn render( |
| `render_sectioned` | libs/cli-std/src/llm.rs | function | pub | 257 | pub fn render_sectioned( |

Phase 1 of #2494 (call-time-generated topic sections): `TopicSection`,
`SectionedTopic`, `RenderedSection`, `RenderableTopic`, `render_sectioned`,
and `assert_topics_render` are additive — `Topic` and `render` keep their
exact prior signature and behavior.


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `<tool> llm` — offline agent-facing self-documentation.
//!
//! Each CLI supplies its own `&[Topic]` (the single in-code source of truth for
//! its domain docs); this module renders the standard `outline` / topic / JSON
//! shapes so the command is uniform across the ecosystem.
//!
//! Phase 1 of #2494 adds [`SectionedTopic`]: a topic whose body is composed of
//! [`TopicSection`]s, some of which are computed at *call time* rather than
//! frozen into a `&'static str` at compile time. Facts that drift out from
//! under a hand-written body (command inventories, config surfaces, feature
//! flags, ...) belong in a `TopicSection::Generated` section so `<tool> llm`
//! always reports what's true right now. `Topic` + `render` are unchanged —
//! every existing CLI keeps compiling and behaving exactly as before.

use std::collections::BTreeSet;

/// One agent-facing documentation topic.
pub struct Topic {
    pub id: &'static str,
    pub summary: &'static str,
    pub body: &'static str,
}

/// One piece of a [`SectionedTopic`]'s rendered body.
///
/// `Prose` is a static string baked into the binary at compile time, exactly
/// like [`Topic::body`]. `Generated` sections call `render` **at render
/// time** so the emitted content always reflects live facts instead of a
/// snapshot frozen when the `&'static str` was written — the freshness gap
/// tracked by #2494.
pub enum TopicSection {
    /// Static Markdown, rendered verbatim.
    Prose(&'static str),
    /// Computed when the topic is rendered. `id` should be unique within the
    /// owning topic (and, for [`assert_topics_render`], across the whole
    /// topic list) so JSON consumers and conformance checks can key on it.
    Generated {
        id: &'static str,
        render: fn() -> String,
    },
}

/// A documentation topic composed of ordered [`TopicSection`]s instead of one
/// static body. Render it with [`render_sectioned`], which produces the same
/// outline/detail/JSON shapes as [`render`] does for [`Topic`].
pub struct SectionedTopic {
    pub id: &'static str,
    pub summary: &'static str,
    pub sections: &'static [TopicSection],
}

/// One [`TopicSection`] already resolved to owned content — the internal
/// representation both [`Topic`] and [`SectionedTopic`] feed into for JSON
/// rendering and for [`assert_topics_render`].
pub struct RenderedSection {
    pub id: String,
    pub kind: &'static str,
    pub content: String,
}

/// Implemented by both [`Topic`] and [`SectionedTopic`] so
/// [`assert_topics_render`] can check either registry through one call.
pub trait RenderableTopic {
    fn topic_id(&self) -> &'static str;
    fn render_sections(&self) -> Vec<RenderedSection>;
}

impl RenderableTopic for Topic {
    fn topic_id(&self) -> &'static str {
        self.id
    }

    fn render_sections(&self) -> Vec<RenderedSection> {
        vec![RenderedSection {
            id: "body".to_string(),
            kind: "prose",
            content: self.body.to_string(),
        }]
    }
}

impl RenderableTopic for SectionedTopic {
    fn topic_id(&self) -> &'static str {
        self.id
    }

    fn render_sections(&self) -> Vec<RenderedSection> {
        resolve_sections(self.sections)
    }
}

fn resolve_sections(sections: &[TopicSection]) -> Vec<RenderedSection> {
    sections
        .iter()
        .enumerate()
        .map(|(i, section)| match section {
            TopicSection::Prose(text) => RenderedSection {
                id: format!("prose-{i}"),
                kind: "prose",
                content: (*text).to_string(),
            },
            TopicSection::Generated { id, render } => RenderedSection {
                id: (*id).to_string(),
                kind: "generated",
                content: render(),
            },
        })
        .collect()
}

/// Join resolved sections into one Markdown body with clear section
/// separation (a blank line between each section's content).
fn join_sections(sections: &[RenderedSection]) -> String {
    sections
        .iter()
        .map(|s| s.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Conformance helper for consumers' tests: every topic in `topics` must
/// render non-empty output, and every `Generated` section must render
/// non-empty content with an id that's unique across the whole slice.
/// Panics naming the offending topic (and section, for `Generated`
/// failures) so a broken generator fails a specific, named assertion rather
/// than a vague "something is empty somewhere".
///
/// Works for both `&[Topic]` and `&[SectionedTopic]` via [`RenderableTopic`].
pub fn assert_topics_render<T: RenderableTopic>(topics: &[T]) {
    let mut seen_generated_ids = BTreeSet::new();
    for topic in topics {
        let sections = topic.render_sections();
        let body = join_sections(&sections);
        assert!(
            !body.trim().is_empty(),
            "llm topic `{}` rendered empty output",
            topic.topic_id()
        );
        for section in &sections {
            if section.kind != "generated" {
                continue;
            }
            assert!(
                !section.content.trim().is_empty(),
                "llm topic `{}` generated section `{}` rendered empty output",
                topic.topic_id(),
                section.id
            );
            assert!(
                seen_generated_ids.insert(section.id.clone()),
                "llm topic `{}` generated section id `{}` is not unique",
                topic.topic_id(),
                section.id
            );
        }
    }
}

/// Typed, task-navigation protocol for tools that need stronger machine
/// semantics than a static topic body.
pub mod v2;

/// Output format for `llm`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    Md,
    Json,
}

impl Format {
    /// Parse `md`/`json` (case-insensitive; anything else → `Md`).
    pub fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("json") {
            Format::Json
        } else {
            Format::Md
        }
    }
}

/// Render `<tool> llm --topic <topic> --format <fmt>`. `topic == "outline"` (the
/// default) prints the topic map + the standard-command footer.
pub fn render(
    project: &str,
    version: &str,
    topics: &[Topic],
    topic: &str,
    format: Format,
) -> anyhow::Result<String> {
    if topic == "outline" {
        return Ok(match format {
            Format::Md => outline_md(project, topics),
            Format::Json => {
                let ts: Vec<_> = topics
                    .iter()
                    .map(|t| serde_json::json!({ "id": t.id, "summary": t.summary }))
                    .collect();
                serde_json::to_string_pretty(&serde_json::json!({
                    "project": project, "version": version, "topics": ts,
                }))?
            }
        });
    }
    let Some(t) = topics.iter().find(|t| t.id == topic) else {
        let ids: Vec<&str> = topics.iter().map(|t| t.id).collect();
        anyhow::bail!(
            "unknown llm topic '{topic}'. Try: outline, {}",
            ids.join(", ")
        );
    };
    Ok(match format {
        Format::Md => t.body.to_string(),
        Format::Json => serde_json::to_string_pretty(&serde_json::json!({
            "project": project, "topic": t.id, "summary": t.summary, "body": t.body,
        }))?,
    })
}

fn outline_md(project: &str, topics: &[Topic]) -> String {
    let mut s = outline_header(project);
    for t in topics {
        s.push_str(&format!("- `{}` — {}\n", t.id, t.summary));
    }
    s.push_str(&standard_commands_footer(project));
    s
}

fn outline_header(project: &str) -> String {
    format!(
        "# {project} — agent topic outline\n\n\
         Run `{project} llm --topic <topic>` for detail (add `--format json` for a machine-readable form).\n\n\
         ## Topics\n\n"
    )
}

fn standard_commands_footer(project: &str) -> String {
    format!(
        "\n## Standard agent commands\n\n\
         - `{project} llm [--topic <t>] [--format md|json]` — this self-documentation (offline)\n\
         - `{project} upgrade [--version <tag>] [--check]` — self-update from GitHub releases\n\
         - `{project} issue search [query]` · `view <n>` · `create [--title <t>] [message...]` · `comment <n> [message...]` — search, read, file, and comment on diagnostics-rich issues; comment ensures the issue is open\n"
    )
}

/// Render `<tool> llm --topic <topic> --format <fmt>` for a [`SectionedTopic`]
/// registry. Mirrors [`render`]'s outline/detail/Markdown shapes; JSON detail
/// additionally reports each resolved section as `{id, kind, content}` (see
/// [`RenderedSection`]) so agents can tell static prose apart from
/// call-time-generated facts (#2494). `Generated` sections call their `render`
/// fn here, at call time — never at topic-registration time.
pub fn render_sectioned(
    project: &str,
    version: &str,
    topics: &[SectionedTopic],
    topic: &str,
    format: Format,
) -> anyhow::Result<String> {
    if topic == "outline" {
        return Ok(match format {
            Format::Md => outline_md_sectioned(project, topics),
            Format::Json => {
                let ts: Vec<_> = topics
                    .iter()
                    .map(|t| serde_json::json!({ "id": t.id, "summary": t.summary }))
                    .collect();
                serde_json::to_string_pretty(&serde_json::json!({
                    "project": project, "version": version, "topics": ts,
                }))?
            }
        });
    }
    let Some(t) = topics.iter().find(|t| t.id == topic) else {
        let ids: Vec<&str> = topics.iter().map(|t| t.id).collect();
        anyhow::bail!(
            "unknown llm topic '{topic}'. Try: outline, {}",
            ids.join(", ")
        );
    };
    let sections = resolve_sections(t.sections);
    let body = join_sections(&sections);
    Ok(match format {
        Format::Md => body,
        Format::Json => serde_json::to_string_pretty(&serde_json::json!({
            "project": project,
            "topic": t.id,
            "summary": t.summary,
            "body": body,
            "sections": sections
                .iter()
                .map(|s| serde_json::json!({ "id": s.id, "kind": s.kind, "content": s.content }))
                .collect::<Vec<_>>(),
        }))?,
    })
}

fn outline_md_sectioned(project: &str, topics: &[SectionedTopic]) -> String {
    let mut s = outline_header(project);
    for t in topics {
        s.push_str(&format!("- `{}` — {}\n", t.id, t.summary));
    }
    s.push_str(&standard_commands_footer(project));
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: &[Topic] = &[Topic {
        id: "workflow",
        summary: "how it works",
        body: "# the body",
    }];

    #[test]
    fn outline_lists_topics_and_standard_commands() {
        let o = render("lumen", "0.4.3", T, "outline", Format::Md).unwrap();
        assert!(o.contains("`workflow`"));
        assert!(o.contains("lumen upgrade"));
        assert!(o.contains("lumen issue search"));
        assert!(o.contains("comment <n>"));
        assert!(!o.contains("report-issue"));
    }

    #[test]
    fn topic_body_and_unknown() {
        assert_eq!(
            render("lumen", "0.4.3", T, "workflow", Format::Md).unwrap(),
            "# the body"
        );
        assert!(render("lumen", "0.4.3", T, "nope", Format::Md).is_err());
    }

    #[test]
    fn json_outline_shape() {
        let j = render("lumen", "0.4.3", T, "outline", Format::Json).unwrap();
        assert!(j.contains("\"project\"") && j.contains("\"topics\""));
        assert_eq!(Format::parse("JSON"), Format::Json);
        assert_eq!(Format::parse("md"), Format::Md);
    }

    // --- SectionedTopic / TopicSection (#2494 Phase 1) ---

    fn fixed_fact() -> String {
        "the sky is blue".to_string()
    }

    const SECTIONED: &[SectionedTopic] = &[SectionedTopic {
        id: "workflow",
        summary: "how it works",
        sections: &[
            TopicSection::Prose("# intro prose"),
            TopicSection::Generated {
                id: "fact",
                render: fixed_fact,
            },
        ],
    }];

    #[test]
    fn static_topic_unchanged_behavior() {
        // The existing Topic + render() signatures and outputs are untouched
        // by adding SectionedTopic — same exact-match assertions as before.
        assert_eq!(
            render("lumen", "0.4.3", T, "workflow", Format::Md).unwrap(),
            "# the body"
        );
        let o = render("lumen", "0.4.3", T, "outline", Format::Md).unwrap();
        assert!(o.contains("`workflow`") && o.contains("lumen upgrade"));
        // assert_topics_render also accepts a plain `&[Topic]` registry.
        assert_topics_render(T);
    }

    #[test]
    fn sectioned_topic_renders_prose_then_generated_in_order() {
        let body = render_sectioned("lumen", "0.4.3", SECTIONED, "workflow", Format::Md).unwrap();
        let prose_at = body.find("# intro prose").expect("prose section present");
        let fact_at = body
            .find("the sky is blue")
            .expect("generated section present");
        assert!(
            prose_at < fact_at,
            "prose must render before the generated section: {body:?}"
        );
    }

    #[test]
    fn conformance_helper_catches_empty_generated_section() {
        fn empty() -> String {
            String::new()
        }
        const BROKEN: &[SectionedTopic] = &[SectionedTopic {
            id: "broken",
            summary: "has a dead generator",
            sections: &[TopicSection::Generated {
                id: "dead",
                render: empty,
            }],
        }];
        let result = std::panic::catch_unwind(|| assert_topics_render(BROKEN));
        assert!(
            result.is_err(),
            "assert_topics_render must panic on an empty generated section"
        );
    }

    #[test]
    fn conformance_helper_passes_healthy_sectioned_topics() {
        assert_topics_render(SECTIONED);
    }

    #[test]
    fn sectioned_json_format_shape() {
        let j = render_sectioned("lumen", "0.4.3", SECTIONED, "workflow", Format::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&j).unwrap();
        assert_eq!(parsed["project"], "lumen");
        assert_eq!(parsed["topic"], "workflow");
        assert_eq!(parsed["summary"], "how it works");
        assert!(parsed["body"].as_str().unwrap().contains("the sky is blue"));
        let sections = parsed["sections"].as_array().unwrap();
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0]["kind"], "prose");
        assert_eq!(sections[1]["kind"], "generated");
        assert_eq!(sections[1]["id"], "fact");
        assert_eq!(sections[1]["content"], "the sky is blue");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/cli-std/src/llm.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/cli-std/src/llm.rs` captured during libs codegen standardization.
      Refs #2494 (Phase 1): added TopicSection/SectionedTopic/RenderedSection/RenderableTopic,
      render_sectioned, and assert_topics_render for call-time-generated topic sections. Topic
      and render are unchanged.
```
