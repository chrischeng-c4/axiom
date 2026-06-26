//! `<tool> llm` — offline agent-facing self-documentation.
//!
//! Each CLI supplies its own `&[Topic]` (the single in-code source of truth for
//! its domain docs); this module renders the standard `outline` / topic / JSON
//! shapes so the command is uniform across the ecosystem.

/// One agent-facing documentation topic.
pub struct Topic {
    pub id: &'static str,
    pub summary: &'static str,
    pub body: &'static str,
}

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
    let mut s = format!(
        "# {project} — agent topic outline\n\n\
         Run `{project} llm --topic <topic>` for detail (add `--format json` for a machine-readable form).\n\n\
         ## Topics\n\n"
    );
    for t in topics {
        s.push_str(&format!("- `{}` — {}\n", t.id, t.summary));
    }
    s.push_str(&format!(
        "\n## Standard agent commands\n\n\
         - `{project} llm [--topic <t>] [--format md|json]` — this self-documentation (offline)\n\
         - `{project} upgrade [--version <tag>] [--check]` — self-update from GitHub releases\n\
         - `{project} issue search [query]` · `view <n>` · `create [--title <t>] [message...]` — search, read, and file diagnostics-rich issues\n"
    ));
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
}
