// HANDWRITE-BEGIN gap="missing-generator:logic:277082b0" tracker="pending-tracker" reason="Detect configured AW TD, EC, capability, and WI Markdown; extract typed sections, commands, assertions, relationships, and source navigation without mutation."
use std::{collections::BTreeSet, fs};

use super::{
    escape_html, markdown::safe_markdown_html, ContextDocument, ContextDocumentKind,
    ContextNavigation, ContextProvenance, ContextRenderer, ContextRequest, ContextTarget,
    RendererError, RendererSupport,
};

const MAX_AW_ARTIFACT_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AwArtifactKind {
    TechDesign,
    ExternalContract,
    Capability,
    WorkItem,
}

impl AwArtifactKind {
    fn label(self) -> &'static str {
        match self {
            Self::TechDesign => "Tech design",
            Self::ExternalContract => "External contract",
            Self::Capability => "Capability contract",
            Self::WorkItem => "Work item",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedSection {
    title: String,
    line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AwArtifactModel {
    kind: AwArtifactKind,
    frontmatter: Vec<(String, String)>,
    sections: Vec<ParsedSection>,
    mermaid_blocks: Vec<String>,
    commands: Vec<String>,
    assertions: Vec<String>,
    relationships: Vec<String>,
}

#[derive(Debug, Default)]
pub struct AwTypedRenderer;

impl AwTypedRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl ContextRenderer for AwTypedRenderer {
    fn id(&self) -> &'static str {
        "aw-typed"
    }

    fn priority(&self) -> i32 {
        300
    }

    fn supports(&self, request: &ContextRequest) -> RendererSupport {
        if !request.root().join("aw.toml").is_file() || !markdown_target(request.target()) {
            return RendererSupport::Unsupported;
        }

        let Ok(source_path) = request.resolve_target() else {
            return RendererSupport::Unsupported;
        };
        let Ok(bytes) = fs::read(source_path) else {
            return RendererSupport::Unsupported;
        };
        if bytes.len() > MAX_AW_ARTIFACT_BYTES {
            return RendererSupport::Unsupported;
        }
        let Ok(source) = std::str::from_utf8(&bytes) else {
            return RendererSupport::Unsupported;
        };

        if detect_kind(source).is_some() {
            RendererSupport::Supported
        } else {
            RendererSupport::Unsupported
        }
    }

    fn render(&self, request: &ContextRequest) -> Result<ContextDocument, RendererError> {
        let source_path = request.resolve_target()?;
        let bytes = fs::read(&source_path).map_err(|error| {
            RendererError::new(format!(
                "could not read AW artifact {}: {error}",
                source_path.display()
            ))
        })?;
        if bytes.len() > MAX_AW_ARTIFACT_BYTES {
            return Err(RendererError::new(format!(
                "AW artifact exceeds the {MAX_AW_ARTIFACT_BYTES}-byte limit"
            )));
        }
        let source = std::str::from_utf8(&bytes)
            .map_err(|_| RendererError::new("AW artifact is not valid UTF-8"))?;
        let model = parse_artifact(source)
            .ok_or_else(|| RendererError::new("AW artifact structure is not recognized"))?;
        let relative_source = source_path
            .strip_prefix(request.root())
            .unwrap_or(&source_path)
            .to_path_buf();

        let navigation = model
            .sections
            .iter()
            .map(|section| ContextNavigation {
                label: format!("{} · line {}", section.title, section.line),
                path: relative_source.clone(),
            })
            .collect();
        let file_name = source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("AW artifact");

        Ok(ContextDocument {
            renderer_id: self.id().to_owned(),
            kind: ContextDocumentKind::AwTyped,
            title: format!("{}: {file_name}", model.kind.label()),
            body_html: render_model(&model, source),
            navigation,
            warnings: Vec::new(),
            provenance: ContextProvenance {
                root: request.root().to_path_buf(),
                sources: vec![source_path],
            },
        })
    }
}

fn markdown_target(target: &ContextTarget) -> bool {
    let ContextTarget::File(path) = target else {
        return false;
    };
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some(extension)
            if extension.eq_ignore_ascii_case("md")
                || extension.eq_ignore_ascii_case("markdown")
    )
}

fn parse_artifact(source: &str) -> Option<AwArtifactModel> {
    let kind = detect_kind(source)?;
    let frontmatter = parse_frontmatter(source);
    let sections = parse_sections(source);
    let (mermaid_blocks, commands) = parse_fenced_blocks(source);
    let assertions = parse_assertions(source, kind);
    let relationships = parse_relationships(source);
    Some(AwArtifactModel {
        kind,
        frontmatter,
        sections,
        mermaid_blocks,
        commands,
        assertions,
        relationships,
    })
}

fn detect_kind(source: &str) -> Option<AwArtifactKind> {
    let lower = source.to_ascii_lowercase();
    if lower.contains("\nfill_sections:") || lower.contains("\nkind: tech-design") {
        return Some(AwArtifactKind::TechDesign);
    }
    if lower.contains("kind: external-contract")
        || (source.contains("## Assertions") && source.contains("## Verifier"))
    {
        return Some(AwArtifactKind::ExternalContract);
    }
    if source.contains("## Capabilities") && source.contains("### Capability Index") {
        return Some(AwArtifactKind::Capability);
    }
    if [
        "## Problem",
        "## Capability Alignment",
        "## Scope",
        "## Acceptance Criteria",
        "## Reference Context",
    ]
    .iter()
    .all(|heading| source.contains(heading))
    {
        return Some(AwArtifactKind::WorkItem);
    }
    None
}

fn parse_frontmatter(source: &str) -> Vec<(String, String)> {
    let mut lines = source.lines().skip_while(|line| *line != "---");
    if lines.next() != Some("---") {
        return Vec::new();
    }
    lines
        .take_while(|line| *line != "---")
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            Some((key.trim().to_owned(), value.trim().to_owned()))
        })
        .collect()
}

fn parse_sections(source: &str) -> Vec<ParsedSection> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let hashes = line
                .chars()
                .take_while(|character| *character == '#')
                .count();
            if hashes == 0 || hashes > 6 || !line[hashes..].starts_with(' ') {
                return None;
            }
            Some(ParsedSection {
                title: line[hashes..].trim().to_owned(),
                line: index + 1,
            })
        })
        .collect()
}

fn parse_fenced_blocks(source: &str) -> (Vec<String>, Vec<String>) {
    let mut mermaid = Vec::new();
    let mut commands = Vec::new();
    let mut active: Option<(String, Vec<String>)> = None;

    for line in source.lines() {
        if let Some(language) = line.trim().strip_prefix("```") {
            if let Some((language, content)) = active.take() {
                let content = content.join("\n");
                if language == "mermaid" {
                    mermaid.push(content);
                } else if matches!(language.as_str(), "bash" | "sh" | "shell" | "console") {
                    commands.extend(
                        content
                            .lines()
                            .map(str::trim)
                            .filter(|line| !line.is_empty() && !line.starts_with('#'))
                            .map(str::to_owned),
                    );
                }
            } else {
                active = Some((language.trim().to_ascii_lowercase(), Vec::new()));
            }
        } else if let Some((_, content)) = &mut active {
            content.push(line.to_owned());
        }
    }

    for token in backtick_tokens(source) {
        if ["aw ", "cargo ", "git "]
            .iter()
            .any(|prefix| token.starts_with(prefix))
        {
            commands.push(token);
        }
    }
    commands.sort();
    commands.dedup();
    (mermaid, commands)
}

fn parse_assertions(source: &str, kind: AwArtifactKind) -> Vec<String> {
    if kind != AwArtifactKind::ExternalContract {
        return Vec::new();
    }
    let mut assertions = BTreeSet::new();
    let mut in_assertions = false;
    for line in source.lines() {
        if line.starts_with("## ") {
            in_assertions = line.trim() == "## Assertions";
            continue;
        }
        if !in_assertions {
            continue;
        }
        let trimmed = line.trim_start_matches([' ', '-']);
        if let Some(id) = trimmed.strip_prefix("id:") {
            assertions.insert(id.trim().to_owned());
        }
    }
    assertions.into_iter().collect()
}

fn parse_relationships(source: &str) -> Vec<String> {
    let mut relationships = BTreeSet::new();
    for token in backtick_tokens(source) {
        if token.ends_with(".md") || issue_reference(&token) {
            relationships.insert(token);
        }
    }
    for word in source.split_whitespace() {
        let word = word
            .trim_matches(|character: char| !character.is_ascii_alphanumeric() && character != '#');
        if issue_reference(word) {
            relationships.insert(word.to_owned());
        }
    }
    relationships.into_iter().collect()
}

fn backtick_tokens(source: &str) -> Vec<String> {
    source
        .split('`')
        .enumerate()
        .filter_map(|(index, token)| (index % 2 == 1).then(|| token.trim().to_owned()))
        .filter(|token| !token.is_empty())
        .collect()
}

fn issue_reference(value: &str) -> bool {
    value.strip_prefix('#').is_some_and(|number| {
        !number.is_empty() && number.chars().all(|character| character.is_ascii_digit())
    })
}

fn render_model(model: &AwArtifactModel, source: &str) -> String {
    let mut html = format!(
        "<article class=\"aw-typed-context\"><header><p>AW artifact type</p><h2>{}</h2></header>",
        escape_html(model.kind.label())
    );
    render_pairs(&mut html, "Frontmatter", &model.frontmatter);
    render_list(
        &mut html,
        "Sections",
        model
            .sections
            .iter()
            .map(|section| format!("{} · line {}", section.title, section.line)),
    );
    render_list(&mut html, "Commands", model.commands.iter().cloned());
    render_list(&mut html, "Assertions", model.assertions.iter().cloned());
    render_list(
        &mut html,
        "Relationships",
        model.relationships.iter().cloned(),
    );
    if !model.mermaid_blocks.is_empty() {
        html.push_str("<section><h3>Mermaid</h3>");
        for block in &model.mermaid_blocks {
            html.push_str("<pre data-language=\"mermaid\">");
            html.push_str(&escape_html(block));
            html.push_str("</pre>");
        }
        html.push_str("</section>");
    }
    html.push_str("<section><h3>Source</h3>");
    html.push_str(&safe_markdown_html(source));
    html.push_str("</section></article>");
    html
}

fn render_pairs(html: &mut String, heading: &str, items: &[(String, String)]) {
    if items.is_empty() {
        return;
    }
    html.push_str(&format!("<section><h3>{}</h3><dl>", escape_html(heading)));
    for (key, value) in items {
        html.push_str(&format!(
            "<dt>{}</dt><dd>{}</dd>",
            escape_html(key),
            escape_html(value)
        ));
    }
    html.push_str("</dl></section>");
}

fn render_list(html: &mut String, heading: &str, items: impl IntoIterator<Item = String>) {
    let items: Vec<String> = items.into_iter().collect();
    if items.is_empty() {
        return;
    }
    html.push_str(&format!("<section><h3>{}</h3><ul>", escape_html(heading)));
    for item in items {
        html.push_str(&format!("<li>{}</li>", escape_html(&item)));
    }
    html.push_str("</ul></section>");
}
// HANDWRITE-END
