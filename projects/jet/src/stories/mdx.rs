//! Minimal MDX docs-page compiler for `jet stories`.
//!
//! Strategy decision for #996: use a Rust-side markdown + core JSX doc-block
//! subset for the first production slice. This keeps dev/static parity fully
//! inside Jet and fails unsupported MDX syntax with a named diagnostic instead
//! of silently rendering a blank page.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use walkdir::{DirEntry, WalkDir};

use super::manager::{DocsArgType, DocsPage, DocsStory};
use super::{StoryEntry, StoryIndex};

pub fn docs_pages(root: &Path, index: &StoryIndex, autodocs: &[DocsPage]) -> Vec<DocsPage> {
    let mut pages = Vec::new();
    for file in discover_mdx_files(root) {
        let rel = file.strip_prefix(root).unwrap_or(&file).to_path_buf();
        let source = match std::fs::read_to_string(&file) {
            Ok(source) => source,
            Err(_) => continue,
        };
        match compile_mdx_doc(root, &rel, &source, index, autodocs) {
            Ok(page) => pages.push(page),
            Err(_) => continue,
        }
    }
    pages.sort_by(|a, b| a.id.cmp(&b.id));
    pages
}

pub fn diagnostics(root: &Path, index: &StoryIndex) -> Vec<String> {
    let autodocs = Vec::new();
    let mut diagnostics = Vec::new();
    for file in discover_mdx_files(root) {
        let rel = file.strip_prefix(root).unwrap_or(&file).to_path_buf();
        match std::fs::read_to_string(&file) {
            Ok(source) => {
                if let Err(err) = compile_mdx_doc(root, &rel, &source, index, &autodocs) {
                    diagnostics.push(format!("{}: MDX compile error: {err}", rel.display()));
                }
            }
            Err(err) => diagnostics.push(format!("{}: failed to read MDX: {err}", rel.display())),
        }
    }
    diagnostics
}

fn compile_mdx_doc(
    root: &Path,
    rel: &Path,
    source: &str,
    index: &StoryIndex,
    autodocs: &[DocsPage],
) -> Result<DocsPage> {
    let title = mdx_title(source).unwrap_or_else(|| fallback_title(root, rel));
    let title_path = title_path_from_mdx_title(&title);
    let title_display = title_path.join(" / ");
    let mut html = String::new();
    let mut paragraph = Vec::new();
    let mut primary_story_id = String::new();
    let mut stories = Vec::new();
    let autodoc = autodocs.iter().find(|page| page.title == title_display);

    for (line_no, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush_paragraph(&mut html, &mut paragraph);
            continue;
        }
        if trimmed.starts_with("import ") || trimmed.starts_with("export ") {
            continue;
        }
        if trimmed.starts_with("<Meta") {
            ensure_self_closing_doc_block(rel, line_no, trimmed)?;
            continue;
        }
        if trimmed.starts_with("<Canvas") || trimmed.starts_with("<Story") {
            flush_paragraph(&mut html, &mut paragraph);
            ensure_self_closing_doc_block(rel, line_no, trimmed)?;
            let story = resolve_story_ref(trimmed, &title_path, index).ok_or_else(|| {
                anyhow!(
                    "line {}: doc block references an unknown story",
                    line_no + 1
                )
            })?;
            if primary_story_id.is_empty() {
                primary_story_id = story.id.clone();
            }
            if !stories.iter().any(|s: &DocsStory| s.id == story.id) {
                stories.push(DocsStory {
                    id: story.id.clone(),
                    name: story.name.clone(),
                });
            }
            let label = if trimmed.starts_with("<Canvas") {
                "Canvas"
            } else {
                "Story"
            };
            html.push_str(&format!(
                "<section class=\"jet-docs-canvas\"><h3>{}</h3><iframe title=\"{} {}\" src=\"{{{{jet-preview:{}}}}}\"></iframe></section>",
                escape_html(label),
                escape_html(&title_display),
                escape_html(&story.name),
                escape_html(&story.id),
            ));
            continue;
        }
        if trimmed.starts_with("<ArgTypes") {
            flush_paragraph(&mut html, &mut paragraph);
            ensure_self_closing_doc_block(rel, line_no, trimmed)?;
            html.push_str(&render_argtypes_table(
                autodoc.map(|page| page.arg_types.as_slice()).unwrap_or(&[]),
            ));
            continue;
        }
        if trimmed.starts_with("<Source") {
            flush_paragraph(&mut html, &mut paragraph);
            ensure_self_closing_doc_block(rel, line_no, trimmed)?;
            let story = resolve_story_ref(trimmed, &title_path, index).ok_or_else(|| {
                anyhow!("line {}: Source references an unknown story", line_no + 1)
            })?;
            html.push_str("<pre id=\"jet-source-code\"><code>");
            html.push_str(&escape_html(story.source.as_deref().unwrap_or("")));
            html.push_str("</code></pre>");
            continue;
        }
        if let Some(tag) = unsupported_jsx_tag(trimmed) {
            return Err(anyhow!(
                "line {}: unsupported MDX JSX tag <{}>; supported doc blocks are Meta, Canvas, Story, ArgTypes, and Source",
                line_no + 1,
                tag
            ));
        }
        if let Some((level, text)) = markdown_heading(trimmed) {
            flush_paragraph(&mut html, &mut paragraph);
            html.push_str(&format!(
                "<h{level}>{}</h{level}>",
                escape_html(text.trim())
            ));
            continue;
        }
        paragraph.push(trimmed.to_string());
    }
    flush_paragraph(&mut html, &mut paragraph);

    if primary_story_id.is_empty() {
        if let Some(story) = first_story_for_title(index, &title_path) {
            primary_story_id = story.id.clone();
        }
    }
    if stories.is_empty() {
        stories.extend(
            index
                .stories
                .iter()
                .filter(|story| story.title_path == title_path)
                .map(|story| DocsStory {
                    id: story.id.clone(),
                    name: story.name.clone(),
                }),
        );
    }

    Ok(DocsPage {
        id: format!("mdx-{}", slug_for_docs_id(&title_display)),
        title: title_display,
        description: String::new(),
        primary_story_id,
        stories,
        arg_types: autodoc
            .map(|page| page.arg_types.clone())
            .unwrap_or_default(),
        content_html: Some(html),
    })
}

fn discover_mdx_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !ignored_entry(entry))
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) == Some("mdx") {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    files
}

fn ignored_entry(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    matches!(
        name.as_ref(),
        ".git" | "node_modules" | "dist" | "dist-stories" | "target"
    )
}

fn mdx_title(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<Meta") {
            if let Some(title) = attr_value(trimmed, "title") {
                return Some(title);
            }
        }
    }
    source
        .lines()
        .find_map(|line| markdown_heading(line.trim()).map(|(_, text)| text.trim().to_string()))
}

fn fallback_title(root: &Path, rel: &Path) -> String {
    root.join(rel)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Docs")
        .replace(['-', '_'], " ")
}

fn title_path_from_mdx_title(title: &str) -> Vec<String> {
    title
        .split('/')
        .flat_map(|part| part.split(" / "))
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn resolve_story_ref<'a>(
    block: &str,
    title_path: &[String],
    index: &'a StoryIndex,
) -> Option<&'a StoryEntry> {
    if let Some(id) = attr_value(block, "id") {
        return index.stories.iter().find(|story| story.id == id);
    }
    if let Some(of) = attr_expr(block, "of") {
        let name = of
            .rsplit('.')
            .next()
            .map(|s| s.trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '_'))
            .unwrap_or("");
        if !name.is_empty() {
            return index
                .stories
                .iter()
                .find(|story| story.title_path == title_path && story.name == name);
        }
    }
    first_story_for_title(index, title_path)
}

fn first_story_for_title<'a>(
    index: &'a StoryIndex,
    title_path: &[String],
) -> Option<&'a StoryEntry> {
    index
        .stories
        .iter()
        .find(|story| story.title_path == title_path)
}

fn attr_value(block: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = block.find(&needle)? + needle.len();
    let rest = &block[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn attr_expr(block: &str, name: &str) -> Option<String> {
    let needle = format!("{name}={{");
    let start = block.find(&needle)? + needle.len();
    let rest = &block[start..];
    let end = rest.find('}')?;
    Some(rest[..end].to_string())
}

fn ensure_self_closing_doc_block(rel: &Path, line_no: usize, line: &str) -> Result<()> {
    if line.contains("/>") {
        return Ok(());
    }
    Err(anyhow!(
        "{} line {}: multi-line MDX doc blocks are not supported in this Jet slice; use a self-closing block",
        rel.display(),
        line_no + 1
    ))
}

fn unsupported_jsx_tag(line: &str) -> Option<String> {
    let line = line.strip_prefix('<')?;
    let first = line.chars().next()?;
    if !first.is_ascii_uppercase() {
        return None;
    }
    let tag: String = line
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '.')
        .collect();
    if matches!(
        tag.as_str(),
        "Meta" | "Canvas" | "Story" | "ArgTypes" | "Source"
    ) {
        None
    } else {
        Some(tag)
    }
}

fn markdown_heading(line: &str) -> Option<(usize, &str)> {
    let hashes = line.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = line.get(hashes..)?;
    if !rest.starts_with(' ') {
        return None;
    }
    Some((hashes.min(3), rest))
}

fn flush_paragraph(html: &mut String, paragraph: &mut Vec<String>) {
    if paragraph.is_empty() {
        return;
    }
    html.push_str("<p>");
    html.push_str(&escape_html(&paragraph.join(" ")));
    html.push_str("</p>");
    paragraph.clear();
}

fn render_argtypes_table(arg_types: &[DocsArgType]) -> String {
    if arg_types.is_empty() {
        return "<p class=\"jet-no-controls\">No props extracted.</p>".to_string();
    }
    let mut out = String::from(
        "<table class=\"jet-docs-argtypes\"><thead><tr><th>Name</th><th>Type</th><th>Default</th><th>Description</th></tr></thead><tbody>",
    );
    for arg in arg_types {
        out.push_str(&format!(
            "<tr><td>{}</td><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
            escape_html(&arg.name),
            escape_html(&arg.type_text),
            escape_html(arg.default_value.as_deref().unwrap_or("")),
            escape_html(&arg.description),
        ));
    }
    out.push_str("</tbody></table>");
    out
}

fn slug_for_docs_id(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stories::StoryIndex;

    #[test]
    fn compiles_core_doc_blocks() {
        let mut index = StoryIndex::default();
        index.stories.push(StoryEntry {
            id: "components-button--primary".to_string(),
            name: "Primary".to_string(),
            export_name: "Primary".to_string(),
            args: Default::default(),
            parameters: Default::default(),
            source: Some("export const Primary = {};".to_string()),
            has_render: false,
            file: PathBuf::from("src/Button.stories.tsx"),
            title_path: vec!["Components".to_string(), "Button".to_string()],
        });
        let page = compile_mdx_doc(
            Path::new("."),
            Path::new("src/Button.mdx"),
            r#"<Meta title="Components/Button" />

# Button
Use buttons for actions.
<Canvas of={ButtonStories.Primary} />
<Story id="components-button--primary" />
<ArgTypes of={ButtonStories} />
<Source of={ButtonStories.Primary} />
"#,
            &index,
            &[],
        )
        .unwrap();

        let html = page.content_html.unwrap();
        assert_eq!(page.title, "Components / Button");
        assert!(html.contains("{{jet-preview:components-button--primary}}"));
        assert!(html.contains("export const Primary"));
    }

    #[test]
    fn unsupported_jsx_tags_fail_loudly() {
        let err = compile_mdx_doc(
            Path::new("."),
            Path::new("src/Broken.mdx"),
            "<Meta title=\"Components/Button\" />\n<CustomThing />",
            &StoryIndex::default(),
            &[],
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("unsupported MDX JSX tag <CustomThing>"));
    }
}
