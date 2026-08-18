// HANDWRITE-BEGIN gap="missing-generator:logic:c858562b" tracker="pending-tracker" reason="Render bounded UTF-8 Markdown to safe HTML with explicit source navigation."
use std::fs;

use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};

use super::{
    ContextDocument, ContextDocumentKind, ContextNavigation, ContextProvenance, ContextRenderer,
    ContextRequest, ContextTarget, RendererError, RendererSupport,
};

const MAX_MARKDOWN_BYTES: usize = 1024 * 1024;

#[derive(Debug, Default)]
pub struct MarkdownRenderer;

impl MarkdownRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl ContextRenderer for MarkdownRenderer {
    fn id(&self) -> &'static str {
        "markdown"
    }

    fn priority(&self) -> i32 {
        200
    }

    fn supports(&self, request: &ContextRequest) -> RendererSupport {
        let ContextTarget::File(path) = request.target() else {
            return RendererSupport::Unsupported;
        };
        match path.extension().and_then(|extension| extension.to_str()) {
            Some(extension) if extension.eq_ignore_ascii_case("md") => RendererSupport::Supported,
            Some(extension) if extension.eq_ignore_ascii_case("markdown") => {
                RendererSupport::Supported
            }
            _ => RendererSupport::Unsupported,
        }
    }

    fn render(&self, request: &ContextRequest) -> Result<ContextDocument, RendererError> {
        let source = request.resolve_target()?;
        let bytes = fs::read(&source).map_err(|error| {
            RendererError::new(format!("could not read {}: {error}", source.display()))
        })?;
        if bytes.len() > MAX_MARKDOWN_BYTES {
            return Err(RendererError::new(format!(
                "Markdown source exceeds the {MAX_MARKDOWN_BYTES}-byte limit"
            )));
        }
        let markdown = String::from_utf8(bytes)
            .map_err(|_| RendererError::new("Markdown source is not valid UTF-8"))?;
        let body_html = safe_markdown_html(&markdown);
        let relative_source = source
            .strip_prefix(request.root())
            .unwrap_or(&source)
            .to_path_buf();

        Ok(ContextDocument {
            renderer_id: self.id().to_owned(),
            kind: ContextDocumentKind::Markdown,
            title: source
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Markdown")
                .to_owned(),
            body_html,
            navigation: vec![ContextNavigation {
                label: relative_source.display().to_string(),
                path: relative_source,
            }],
            warnings: Vec::new(),
            provenance: ContextProvenance {
                root: request.root().to_path_buf(),
                sources: vec![source],
            },
        })
    }
}

pub(crate) fn safe_markdown_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let events = Parser::new_ext(markdown, options).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: safe_destination(dest_url),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Image {
            link_type,
            dest_url: safe_destination(dest_url),
            title,
            id,
        }),
        event => event,
    });

    let mut rendered = String::new();
    html::push_html(&mut rendered, events);
    rendered
}

fn safe_destination(destination: CowStr<'_>) -> CowStr<'_> {
    if destination_is_safe(&destination) {
        destination
    } else {
        CowStr::Borrowed("#")
    }
}

fn destination_is_safe(destination: &str) -> bool {
    let destination = destination.trim();
    let lower = destination.to_ascii_lowercase();
    if destination.is_empty() || destination.starts_with('#') {
        return true;
    }
    if lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("mailto:")
    {
        return true;
    }
    if destination.starts_with("//") {
        return false;
    }

    let colon = lower.find(':');
    let path_delimiter = lower.find(['/', '#', '?']);
    !matches!((colon, path_delimiter), (Some(colon), Some(delimiter)) if colon < delimiter)
        && !matches!((colon, path_delimiter), (Some(_), None))
}
// HANDWRITE-END
