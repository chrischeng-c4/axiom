// <HANDWRITE gap="missing-generator:logic:583d99f9" tracker="standardize-gap-projects-jet-src-stories-manager-rs" reason="Manager UI: render the manager HTML shell (sidebar tree from StoryIndex, toolbar, preview iframe) and the isolated per-story preview HTML entry (mounts only the selected story component, no app router/shell).">
//! HTML rendering for the `jet stories` native workbench (B2).
//!
//! Two pure functions, no I/O and no server state, so they are trivially
//! testable and the [`server`](super::server) module can call them per request:
//!
//! - [`render_manager_html`] — the manager shell: a sidebar tree built from the
//!   [`StoryIndex`] title hierarchy, a toolbar, and an `<iframe>` whose `src`
//!   points at the selected story's preview URL. Clicking a sidebar entry just
//!   navigates the iframe (a full preview reload — HMR is B2b/#176, out of
//!   scope here).
//! - [`render_preview_html`] — the *isolated* preview document for one story.
//!   It mounts ONLY that story's component/render into a single root `<div>`
//!   with no app router/shell around it, by dynamically importing the story
//!   module (served + transformed by the module route) and rendering the
//!   selected export.
//!
//! Both emit self-contained strings; escaping is intentionally minimal because
//! the inputs are developer-authored story ids / titles, but every dynamic
//! value that lands in HTML text is run through [`escape_html`] and every value
//! that lands in a JS string literal through [`escape_js`].

use std::collections::BTreeMap;

use super::controls::{Control, ControlKind};
use super::csf::CsfValue;
use super::{StoryEntry, StoryIndex};

/// Route prefix for an isolated story preview document.
pub const PREVIEW_PREFIX: &str = "/__jet_stories_preview";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocsPage {
    pub id: String,
    pub title: String,
    pub description: String,
    pub primary_story_id: String,
    pub stories: Vec<DocsStory>,
    pub arg_types: Vec<DocsArgType>,
    pub content_html: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocsStory {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocsArgType {
    pub name: String,
    pub type_text: String,
    pub default_value: Option<String>,
    pub description: String,
}

/// How the renderers form the URLs they embed (iframe src, sidebar links, the
/// preview's module imports, and the HMR client).
///
/// - [`UrlMode::Dev`] (the default) emits **absolute dev-server routes** — e.g.
///   `/__jet_stories_preview/{id}` for the iframe + sidebar links and a
///   root-relative `/src/...` module URL the dev server transforms on demand —
///   plus the preview-frame HMR client. This is exactly the B2/B2b/B3 behavior
///   and is unchanged.
/// - [`UrlMode::Static`] emits **relative URLs** for the static export (B4): the
///   manager (at `index.html`) links the iframe + sidebar at `preview/{id}.html`,
///   and each preview (at `preview/{id}.html`) imports its module from
///   `../modules/...js`. No HMR client is injected (there is no server at serve
///   time), so the static site is hostable by any file server or `file://`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UrlMode {
    /// Absolute dev-server routes + HMR client (B2/B2b/B3 behavior).
    #[default]
    Dev,
    /// Relative URLs for a static, server-less export (B4).
    Static,
}

impl UrlMode {
    /// The iframe `src` / sidebar link for a story preview in this mode.
    fn preview_url(self, story_id: &str) -> String {
        match self {
            UrlMode::Dev => format!("{PREVIEW_PREFIX}/{story_id}"),
            // Relative to the manager document (`index.html`).
            UrlMode::Static => format!("preview/{story_id}.html"),
        }
    }

    /// The empty-state preview link (no stories / unknown selection).
    fn empty_preview_url(self) -> String {
        match self {
            UrlMode::Dev => format!("{PREVIEW_PREFIX}/"),
            UrlMode::Static => "preview/.html".to_string(),
        }
    }
}

/// Build the manager shell HTML: sidebar tree + toolbar + preview iframe.
///
/// `selected` is the id of the story whose preview the iframe loads first; when
/// `None` (or unknown) the first story in the index is used. With no stories at
/// all the iframe is pointed at an empty-state placeholder.
///
/// `controls` (B3) are the resolved controls for the initially-selected story —
/// one editable widget per component prop, seeded with the story's current arg
/// values. When empty (no props, or component source unavailable) the panel
/// shows a "no controls" placeholder. The server computes them via
/// [`super::controls::resolve_controls`] over the props the prop extractor reads
/// from the component file.
pub fn render_manager_html(
    index: &StoryIndex,
    selected: Option<&str>,
    controls: &[Control],
) -> String {
    render_manager_html_with_mode_and_docs(index, selected, controls, UrlMode::Dev, &[])
}

/// [`render_manager_html`] with an explicit [`UrlMode`]. The dev server calls the
/// [`UrlMode::Dev`] wrapper above (unchanged); the static exporter (B4) passes
/// [`UrlMode::Static`] so the iframe src + sidebar links are relative.
pub fn render_manager_html_with_mode(
    index: &StoryIndex,
    selected: Option<&str>,
    controls: &[Control],
    mode: UrlMode,
) -> String {
    render_manager_html_with_mode_and_docs(index, selected, controls, mode, &[])
}

pub fn render_manager_html_with_docs(
    index: &StoryIndex,
    selected: Option<&str>,
    controls: &[Control],
    docs_pages: &[DocsPage],
) -> String {
    render_manager_html_with_mode_and_docs(index, selected, controls, UrlMode::Dev, docs_pages)
}

pub fn render_manager_html_with_mode_and_docs(
    index: &StoryIndex,
    selected: Option<&str>,
    controls: &[Control],
    mode: UrlMode,
    docs_pages: &[DocsPage],
) -> String {
    // Resolve the initially-selected story: explicit id if it exists, else the
    // first discovered story (the index is already id-sorted).
    let selected_entry = selected
        .and_then(|id| index.stories.iter().find(|s| s.id == id))
        .or_else(|| index.stories.first());

    let initial_src = match selected_entry {
        Some(entry) => mode.preview_url(&entry.id),
        None => mode.empty_preview_url(),
    };
    let initial_id = selected_entry.map(|e| e.id.as_str()).unwrap_or("");

    let sidebar = render_sidebar(index, initial_id, mode, docs_pages);
    let diagnostics = render_diagnostics(index);
    let controls_panel = render_controls_panel(controls);
    let initial_args_json = controls_to_args_json(controls);
    let toolbar_config_json = toolbar_config_json(index);
    let docs_pages_html = render_docs_pages(docs_pages, mode);
    let brand = manager_brand_config(index);
    let story_count = index.stories.len();

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>jet stories</title>
<style>
  :root {{ --jet-accent: {brand_accent}; }}
  * {{ box-sizing: border-box; }}
  html, body {{ margin: 0; height: 100%; }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    display: grid; grid-template-columns: 260px 1fr; grid-template-rows: 44px 1fr 220px;
    grid-template-areas: "sidebar toolbar" "sidebar preview" "sidebar controls";
    height: 100vh; color: #1a1a1a;
  }}
  body.jet-dark {{ color: #e5e7eb; background: #111827; }}
  body.jet-dark #jet-sidebar, body.jet-dark #jet-toolbar, body.jet-dark #jet-controls,
  body.jet-dark #jet-preview-shell, body.jet-dark #jet-docs {{ background: #111827; border-color: #374151; color: #e5e7eb; }}
  body.jet-panel-hidden {{ grid-template-rows: 44px 1fr 0; }}
  body.jet-panel-hidden #jet-controls {{ display: none; }}
  body.jet-toolbar-hidden {{ grid-template-rows: 0 1fr 220px; }}
  body.jet-toolbar-hidden #jet-toolbar {{ display: none; }}
  body.jet-fullscreen {{ grid-template-columns: 0 1fr; grid-template-rows: 0 1fr 0; }}
  body.jet-fullscreen #jet-sidebar, body.jet-fullscreen #jet-toolbar, body.jet-fullscreen #jet-controls {{ display: none; }}
  #jet-sidebar {{
    grid-area: sidebar; border-right: 1px solid #e3e3e3; overflow-y: auto;
    background: #fafafa; padding: 8px 0;
  }}
  #jet-sidebar .jet-brand {{ font-weight: 600; padding: 8px 14px; font-size: 14px; }}
  #jet-search {{ width: calc(100% - 20px); margin: 0 10px 8px; padding: 5px 7px; border: 1px solid #ccc; border-radius: 4px; }}
  #jet-sidebar ul {{ list-style: none; margin: 0; padding: 0; }}
  #jet-sidebar .jet-group > span {{
    display: block; padding: 4px 14px; font-size: 12px; font-weight: 600;
    color: #555;
  }}
  #jet-sidebar a.jet-story {{
    display: block; padding: 4px 14px 4px 28px; font-size: 13px; color: #333;
    text-decoration: none; cursor: pointer;
  }}
  #jet-sidebar a.jet-story:hover, #jet-sidebar a.jet-docs-link:hover {{ background: #eef; }}
  #jet-sidebar a.jet-story.jet-active, #jet-sidebar a.jet-docs-link.jet-active {{ background: var(--jet-accent); color: #fff; }}
  #jet-sidebar mark {{ background: #fef08a; color: #111827; padding: 0; }}
  #jet-toolbar {{
    grid-area: toolbar; border-bottom: 1px solid #e3e3e3; display: flex;
    align-items: center; gap: 12px; padding: 0 14px; background: #fff; font-size: 13px;
  }}
  #jet-toolbar label {{ display: inline-flex; align-items: center; gap: 4px; color: #555; }}
  #jet-toolbar select, #jet-toolbar button {{
    font: inherit; border: 1px solid #d0d0d0; background: #fff; border-radius: 4px;
    min-height: 26px;
  }}
  #jet-toolbar select {{ padding: 2px 22px 2px 6px; }}
  #jet-toolbar button {{ min-width: 28px; padding: 2px 7px; cursor: pointer; }}
  #jet-toolbar button.jet-active-tool {{ background: var(--jet-accent); border-color: var(--jet-accent); color: #fff; }}
  #jet-preview-shell {{
    grid-area: preview; overflow: auto; display: flex; justify-content: center;
    align-items: flex-start; padding: 16px; background: #fff;
  }}
  body.jet-docs-mode #jet-preview-shell {{ display: none; }}
  #jet-preview {{
    border: 0; width: 100%; height: 100%; background: #fff; transform-origin: top center;
  }}
  #jet-docs {{ grid-area: preview; overflow: auto; padding: 18px 22px; background: #fff; display: none; }}
  body.jet-docs-mode #jet-docs {{ display: block; }}
  .jet-docs-page {{ display: none; max-width: 1120px; }}
  .jet-docs-page.jet-docs-active {{ display: block; }}
  .jet-docs-page h1 {{ margin: 0 0 8px; font-size: 24px; }}
  .jet-docs-description {{ margin: 0 0 16px; color: #555; max-width: 760px; }}
  .jet-docs-canvas-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 12px; }}
  .jet-docs-canvas iframe {{ width: 100%; height: 220px; border: 1px solid #e3e3e3; background: #fff; }}
  .jet-docs-canvas h3 {{ margin: 0 0 6px; font-size: 13px; color: #555; }}
  .jet-docs-argtypes {{ width: 100%; border-collapse: collapse; margin: 12px 0 18px; font-size: 13px; }}
  .jet-docs-argtypes th, .jet-docs-argtypes td {{ border: 1px solid #e3e3e3; padding: 6px 8px; text-align: left; vertical-align: top; }}
  .jet-docs-argtypes code {{ font: 12px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }}
  .jet-diag {{ color: #b00; font-size: 12px; padding: 6px 14px; }}
  #jet-controls {{
    grid-area: controls; border-top: 1px solid #e3e3e3; background: #fff;
    overflow-y: auto; padding: 8px 14px; font-size: 13px;
  }}
  #jet-controls h3 {{ margin: 0 0 8px; font-size: 12px; color: #555; font-weight: 600; }}
  #jet-controls h3 button {{ margin-left: 8px; font-size: 11px; }}
  #jet-controls table {{ border-collapse: collapse; width: 100%; }}
  #jet-controls td {{ padding: 4px 8px 4px 0; vertical-align: middle; }}
  #jet-controls td.jet-control-name {{ font-weight: 500; width: 140px; color: #333; }}
  #jet-controls input[type="text"], #jet-controls input[type="number"],
  #jet-controls select {{
    width: 100%; max-width: 260px; padding: 3px 6px; font-size: 13px;
    border: 1px solid #ccc; border-radius: 4px;
  }}
  #jet-controls .jet-no-controls {{ color: #999; }}
  .jet-action-row {{ display: flex; gap: 8px; align-items: baseline; padding: 3px 0; }}
  .jet-action-row code {{ color: #444; font-size: 12px; overflow-wrap: anywhere; }}
  .jet-interaction-row {{ display: flex; gap: 8px; align-items: baseline; padding: 3px 0; }}
  .jet-interaction-pass strong {{ color: #15803d; }}
  .jet-interaction-fail strong {{ color: #b91c1c; }}
  .jet-interaction-row code {{ color: #b91c1c; font-size: 12px; overflow-wrap: anywhere; }}
  .jet-a11y-summary {{ margin: 0 0 6px; color: #555; }}
  .jet-a11y-row {{ border-top: 1px solid #eee; padding: 6px 0; }}
  .jet-a11y-row strong {{ color: #b91c1c; }}
  .jet-a11y-row a {{ color: #4338ca; }}
  .jet-a11y-row code {{ display: block; color: #444; font-size: 12px; overflow-wrap: anywhere; }}
  #jet-source-code {{
    margin: 0; max-height: 220px; overflow: auto; background: #f8f8f8;
    border: 1px solid #e3e3e3; border-radius: 4px; padding: 8px;
  }}
  #jet-source-code code {{ color: #1f2937; font: 12px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; white-space: pre; }}
  #jet-shortcuts-overlay {{ position: fixed; inset: 0; background: rgba(0,0,0,.45); display: none; place-items: center; z-index: 2147483647; }}
  #jet-shortcuts-overlay.jet-open {{ display: grid; }}
  #jet-shortcuts-overlay .jet-shortcuts-card {{ background: #fff; color: #222; border-radius: 6px; padding: 16px; min-width: 280px; }}
</style>
</head>
<body>
<nav id="jet-sidebar" aria-label="Stories">
  <div class="jet-brand">{brand_title}</div>
  <input id="jet-search" type="search" placeholder="Search stories" aria-label="Search stories" />
  {sidebar}
  {diagnostics}
</nav>
<header id="jet-toolbar">
  <span id="jet-current-title">{initial_title}</span>
  <label>Viewport <select id="jet-viewport"></select></label>
  <label>Background <select id="jet-background"></select></label>
  <button id="jet-zoom-out" type="button" title="Zoom out">-</button>
  <button id="jet-zoom-reset" type="button" title="Reset zoom">100%</button>
  <button id="jet-zoom-in" type="button" title="Zoom in">+</button>
  <button id="jet-measure-toggle" type="button" title="Measure">Measure</button>
  <button id="jet-outline-toggle" type="button" title="Outline">Outline</button>
  <button id="jet-theme-toggle" type="button" title="Toggle theme">Theme</button>
  <button id="jet-shortcuts-button" type="button" title="Keyboard shortcuts">?</button>
  <span style="margin-left:auto;color:#777">{story_count} stories</span>
</header>
<div id="jet-preview-shell"><iframe id="jet-preview" name="jet-preview" src="{initial_src}"></iframe></div>
<section id="jet-docs" aria-label="Docs">{docs_pages}</section>
<section id="jet-controls" aria-label="Controls">
  <h3>Controls</h3>
  <div id="jet-controls-body">{controls_panel}</div>
  <h3>Actions <button id="jet-actions-clear" type="button">Clear</button></h3>
  <div id="jet-actions-log" aria-live="polite"></div>
  <h3>Interactions <button id="jet-interactions-clear" type="button">Clear</button></h3>
  <div id="jet-interactions-log" aria-live="polite"></div>
  <h3>A11y <button id="jet-a11y-run" type="button">Run</button></h3>
  <div id="jet-a11y-log" aria-live="polite"></div>
  <h3>Source <button id="jet-source-copy" type="button">Copy</button></h3>
  <pre id="jet-source-code"><code></code></pre>
</section>
<div id="jet-shortcuts-overlay" role="dialog" aria-modal="true" aria-label="Keyboard shortcuts">
  <div class="jet-shortcuts-card">
    <h2>Shortcuts</h2>
    <p><strong>/</strong> focus search</p>
    <p><strong>Arrow Up/Down</strong> choose visible story</p>
    <p><strong>Enter</strong> open focused story</p>
    <p><strong>A</strong> panel, <strong>T</strong> toolbar, <strong>F</strong> fullscreen</p>
  </div>
</div>
<script>
  // Full-reload navigation: clicking a story swaps the preview iframe src.
  // HMR is deliberately out of scope here (B2b / #176) — a reload is fine.
  const frame = document.getElementById('jet-preview');
  const previewShell = document.getElementById('jet-preview-shell');
  const docsShell = document.getElementById('jet-docs');
  const titleEl = document.getElementById('jet-current-title');
  const controlsBody = document.getElementById('jet-controls-body');
  const actionsLog = document.getElementById('jet-actions-log');
  const actionsClear = document.getElementById('jet-actions-clear');
  const interactionsLog = document.getElementById('jet-interactions-log');
  const interactionsClear = document.getElementById('jet-interactions-clear');
  const a11yLog = document.getElementById('jet-a11y-log');
  const a11yRun = document.getElementById('jet-a11y-run');
  const sourceCode = document.querySelector('#jet-source-code code');
  const sourceCopy = document.getElementById('jet-source-copy');
  const searchInput = document.getElementById('jet-search');
  const themeToggle = document.getElementById('jet-theme-toggle');
  const shortcutsButton = document.getElementById('jet-shortcuts-button');
  const shortcutsOverlay = document.getElementById('jet-shortcuts-overlay');
  const viewportSelect = document.getElementById('jet-viewport');
  const backgroundSelect = document.getElementById('jet-background');
  const zoomOutButton = document.getElementById('jet-zoom-out');
  const zoomResetButton = document.getElementById('jet-zoom-reset');
  const zoomInButton = document.getElementById('jet-zoom-in');
  const measureToggle = document.getElementById('jet-measure-toggle');
  const outlineToggle = document.getElementById('jet-outline-toggle');
  const jetToolbarConfigByStory = {toolbar_config_json};
  const jetDefaultToolbarConfig = jetToolbarConfigByStory.__default;
  const jetSourceByStory = {source_panel_json};
  let jetToolbarState = jetLoadToolbarState();
  const jetStoryArgs = new Map();
  let jetCurrentStoryId = "{initial_story_id}";
  if (jetCurrentStoryId) {{
    jetStoryArgs.set(jetCurrentStoryId, {initial_args_json});
    jetHydrateArgsFromUrl(jetCurrentStoryId);
  }}
  function jetEscapeHtml(value) {{
    return String(value == null ? '' : value).replace(/[&<>"]/g, (ch) => ({{'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}}[ch]));
  }}
  const jetTheme = localStorage.getItem('jet-stories-theme') || (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
  document.body.classList.toggle('jet-dark', jetTheme === 'dark');
  document.querySelectorAll('a.jet-story').forEach((a) => {{
    a.dataset.label = a.textContent || '';
    a.tabIndex = -1;
  }});

  function jetPersistShellState() {{
    localStorage.setItem('jet-stories-last-story', jetCurrentStoryId || '');
    localStorage.setItem('jet-stories-theme', document.body.classList.contains('jet-dark') ? 'dark' : 'light');
  }}

  function jetFilterStories(query) {{
    const q = query.trim().toLowerCase();
    document.querySelectorAll('a.jet-story').forEach((a) => {{
      const label = a.dataset.label || a.textContent || '';
      const haystack = (label + ' ' + (a.getAttribute('data-title') || '')).toLowerCase();
      const matched = !q || haystack.includes(q);
      a.closest('li').style.display = matched ? '' : 'none';
      a.innerHTML = q && matched ? jetEscapeHtml(label).replace(new RegExp('(' + q.replace(/[.*+?^${{}}()|[\\]\\\\]/g, '\\\\$&') + ')', 'ig'), '<mark>$1</mark>') : jetEscapeHtml(label);
    }});
    document.querySelectorAll('#jet-sidebar .jet-group').forEach((group) => {{
      const any = Array.from(group.querySelectorAll('a.jet-story')).some((a) => a.closest('li').style.display !== 'none');
      group.style.display = any ? '' : 'none';
    }});
  }}

  function jetVisibleStories() {{
    return Array.from(document.querySelectorAll('a.jet-story')).filter((a) => a.closest('li').style.display !== 'none');
  }}

  function jetMoveStoryFocus(delta) {{
    const items = jetVisibleStories();
    if (!items.length) return;
    const idx = Math.max(0, items.indexOf(document.activeElement));
    const next = items[(idx + delta + items.length) % items.length];
    next.focus();
  }}

  function jetToggleShortcuts() {{
    shortcutsOverlay.classList.toggle('jet-open');
  }}

  document.querySelectorAll('a.jet-story').forEach((a) => {{
    a.addEventListener('click', (ev) => {{
      ev.preventDefault();
      document.body.classList.remove('jet-docs-mode');
      const storyId = a.getAttribute('data-story-id');
      document.querySelectorAll('a.jet-story').forEach((x) => x.classList.remove('jet-active'));
      document.querySelectorAll('a.jet-docs-link').forEach((x) => x.classList.remove('jet-active'));
      a.classList.add('jet-active');
      jetCurrentStoryId = storyId;
      jetPersistShellState();
      jetA11yResult = null;
      jetRenderA11y();
      jetRenderSource();
      frame.setAttribute('src', a.getAttribute('data-preview'));
      titleEl.textContent = a.getAttribute('data-title');
      history.replaceState(null, '', '?story=' + encodeURIComponent(storyId));
      jetPopulateToolbar();
      jetApplyToolbar();
      jetLoadControls(storyId);
    }});
  }});
  document.querySelectorAll('a.jet-docs-link').forEach((a) => {{
    a.addEventListener('click', (ev) => {{
      ev.preventDefault();
      const docsId = a.getAttribute('data-docs-id');
      document.querySelectorAll('a.jet-story, a.jet-docs-link').forEach((x) => x.classList.remove('jet-active'));
      a.classList.add('jet-active');
      document.querySelectorAll('.jet-docs-page').forEach((page) => {{
        page.classList.toggle('jet-docs-active', page.getAttribute('data-docs-id') === docsId);
      }});
      document.body.classList.add('jet-docs-mode');
      titleEl.textContent = a.getAttribute('data-title') || 'Docs';
      history.replaceState(null, '', '?docs=' + encodeURIComponent(docsId || ''));
    }});
  }});
  searchInput.addEventListener('input', () => jetFilterStories(searchInput.value));
  searchInput.addEventListener('keydown', (ev) => {{
    if (ev.key === 'ArrowDown') {{ ev.preventDefault(); jetMoveStoryFocus(1); }}
    if (ev.key === 'ArrowUp') {{ ev.preventDefault(); jetMoveStoryFocus(-1); }}
    if (ev.key === 'Enter') {{
      const first = jetVisibleStories()[0];
      if (first) first.click();
    }}
  }});
  themeToggle.addEventListener('click', () => {{
    document.body.classList.toggle('jet-dark');
    jetPersistShellState();
  }});
  shortcutsButton.addEventListener('click', jetToggleShortcuts);
  shortcutsOverlay.addEventListener('click', (ev) => {{
    if (ev.target === shortcutsOverlay) jetToggleShortcuts();
  }});
  window.addEventListener('keydown', (ev) => {{
    const tag = (document.activeElement && document.activeElement.tagName || '').toLowerCase();
    const typing = tag === 'input' || tag === 'textarea' || tag === 'select';
    if (ev.key === '?' && !typing) {{ ev.preventDefault(); jetToggleShortcuts(); return; }}
    if ((ev.key === '/' || (ev.key.toLowerCase() === 'k' && (ev.metaKey || ev.ctrlKey))) && !typing) {{
      ev.preventDefault(); searchInput.focus(); searchInput.select(); return;
    }}
    if (typing) return;
    if (ev.key === 'ArrowDown') {{ ev.preventDefault(); jetMoveStoryFocus(1); return; }}
    if (ev.key === 'ArrowUp') {{ ev.preventDefault(); jetMoveStoryFocus(-1); return; }}
    if (ev.key === 'Enter' && document.activeElement && document.activeElement.matches('a.jet-story')) {{
      ev.preventDefault(); document.activeElement.click(); return;
    }}
    if (ev.key.toLowerCase() === 'a') document.body.classList.toggle('jet-panel-hidden');
    if (ev.key.toLowerCase() === 't') document.body.classList.toggle('jet-toolbar-hidden');
    if (ev.key.toLowerCase() === 'f') document.body.classList.toggle('jet-fullscreen');
  }});

  const jetActions = new Map();

  function jetRenderActions() {{
    if (!actionsLog) return;
    if (!jetActions.size) {{
      actionsLog.innerHTML = '<p class="jet-no-controls">No actions logged.</p>';
      return;
    }}
    actionsLog.innerHTML = Array.from(jetActions.values()).map((entry) =>
      '<div class="jet-action-row"><strong>' + entry.name + '</strong> ' +
      '<span>x' + entry.count + '</span> ' +
      '<code>' + jetEscapeHtml(entry.args) + '</code></div>'
    ).join('');
  }}

  window.addEventListener('message', (ev) => {{
    const data = ev && ev.data;
    if (!data || data.type !== 'jet-action') return;
    const key = data.name + '\\n' + data.args;
    const prev = jetActions.get(key);
    jetActions.set(key, {{
      name: data.name,
      args: data.args || '[]',
      count: prev ? prev.count + 1 : 1,
      ts: data.ts || Date.now(),
    }});
    jetRenderActions();
  }});
  actionsClear && actionsClear.addEventListener('click', () => {{
    jetActions.clear();
    jetRenderActions();
  }});
  const jetInteractions = [];

  function jetRenderInteractions() {{
    if (!interactionsLog) return;
    if (!jetInteractions.length) {{
      interactionsLog.innerHTML = '<p class="jet-no-controls">No interactions run.</p>';
      return;
    }}
    interactionsLog.innerHTML = jetInteractions.map((entry) =>
      '<div class="jet-interaction-row jet-interaction-' + entry.status + '">' +
      '<strong>' + entry.status + '</strong> ' +
      '<span>' + jetEscapeHtml(entry.name || '') + '</span>' +
      (entry.error ? '<code>' + jetEscapeHtml(entry.error) + '</code>' : '') +
      '</div>'
    ).join('');
  }}

  window.addEventListener('message', (ev) => {{
    const data = ev && ev.data;
    if (!data || data.type !== 'jet-interaction') return;
    jetInteractions.push({{
      name: data.name || 'play',
      status: data.status || 'unknown',
      error: data.error || '',
      ts: data.ts || Date.now(),
    }});
    jetRenderInteractions();
  }});
  interactionsClear && interactionsClear.addEventListener('click', () => {{
    jetInteractions.length = 0;
    jetRenderInteractions();
  }});
  let jetA11yResult = null;

  function jetRenderA11y() {{
    if (!a11yLog) return;
    if (!jetA11yResult) {{
      a11yLog.innerHTML = '<p class="jet-no-controls">No accessibility audit yet.</p>';
      return;
    }}
    if (jetA11yResult.status === 'disabled') {{
      a11yLog.innerHTML = '<p class="jet-a11y-summary">A11y audit disabled for this story.</p>';
      return;
    }}
    if (jetA11yResult.status === 'error') {{
      a11yLog.innerHTML = '<p class="jet-a11y-summary">A11y audit failed: ' + jetEscapeHtml(jetA11yResult.error) + '</p>';
      return;
    }}
    const violations = Array.isArray(jetA11yResult.violations) ? jetA11yResult.violations : [];
    if (!violations.length) {{
      a11yLog.innerHTML = '<p class="jet-a11y-summary">0 violations.</p>';
      return;
    }}
    a11yLog.innerHTML = '<p class="jet-a11y-summary">' + violations.length + ' violation' + (violations.length === 1 ? '' : 's') + '.</p>' +
      violations.map((violation) =>
        '<div class="jet-a11y-row">' +
        '<strong>' + jetEscapeHtml(violation.impact || 'minor') + '</strong> ' +
        '<span>' + jetEscapeHtml(violation.id || 'rule') + '</span> ' +
        (violation.helpUrl ? '<a href="' + jetEscapeHtml(violation.helpUrl) + '" target="_blank" rel="noreferrer">docs</a>' : '') +
        '<code>' + jetEscapeHtml(violation.description || violation.help || '') + '</code>' +
        '<code>' + jetEscapeHtml((violation.targets || []).join(', ')) + '</code>' +
        '</div>'
      ).join('');
  }}

  window.addEventListener('message', (ev) => {{
    const data = ev && ev.data;
    if (!data || data.type !== 'jet-a11y-result') return;
    if (data.storyId && jetCurrentStoryId && data.storyId !== jetCurrentStoryId) return;
    jetA11yResult = data;
    jetRenderA11y();
  }});
  a11yRun && a11yRun.addEventListener('click', () => {{
    const win = frame.contentWindow;
    if (win) win.postMessage({{ type: 'jet-a11y-run' }}, '*');
  }});

  function jetRenderSource() {{
    if (!sourceCode) return;
    const source = jetSourceByStory[jetCurrentStoryId] || '';
    sourceCode.textContent = source || 'No source available.';
  }}

  sourceCopy && sourceCopy.addEventListener('click', async () => {{
    const source = jetSourceByStory[jetCurrentStoryId] || '';
    if (!source || !navigator.clipboard) return;
    await navigator.clipboard.writeText(source);
  }});

  function jetLoadToolbarState() {{
    try {{
      return {{ viewport: 'responsive', background: 'transparent', zoom: 1, measure: false, outline: false, ...JSON.parse(sessionStorage.getItem('jet-stories-toolbar') || '{{}}') }};
    }} catch (_) {{
      return {{ viewport: 'responsive', background: 'transparent', zoom: 1, measure: false, outline: false }};
    }}
  }}

  function jetSaveToolbarState() {{
    sessionStorage.setItem('jet-stories-toolbar', JSON.stringify(jetToolbarState));
  }}

  function jetToolbarConfig() {{
    const story = jetToolbarConfigByStory[jetCurrentStoryId] || {{}};
    return {{
      viewports: {{ ...jetDefaultToolbarConfig.viewports, ...(story.viewports || {{}}) }},
      backgrounds: {{ ...jetDefaultToolbarConfig.backgrounds, ...(story.backgrounds || {{}}) }},
    }};
  }}

  function jetPopulateSelect(select, items, selected) {{
    select.innerHTML = '';
    Object.entries(items).forEach(([id, item]) => {{
      const option = document.createElement('option');
      option.value = id;
      option.textContent = item.name || id;
      option.selected = id === selected;
      select.appendChild(option);
    }});
  }}

  function jetPopulateToolbar() {{
    const config = jetToolbarConfig();
    if (!config.viewports[jetToolbarState.viewport]) jetToolbarState.viewport = 'responsive';
    if (!config.backgrounds[jetToolbarState.background]) jetToolbarState.background = 'transparent';
    jetPopulateSelect(viewportSelect, config.viewports, jetToolbarState.viewport);
    jetPopulateSelect(backgroundSelect, config.backgrounds, jetToolbarState.background);
  }}

  function jetApplyToolbar() {{
    const config = jetToolbarConfig();
    const viewport = config.viewports[jetToolbarState.viewport] || config.viewports.responsive;
    const background = config.backgrounds[jetToolbarState.background] || config.backgrounds.transparent;
    previewShell.style.background = background.value || '#fff';
    frame.style.width = viewport.width || '100%';
    frame.style.height = viewport.height || '100%';
    frame.style.transform = 'scale(' + jetToolbarState.zoom + ')';
    zoomResetButton.textContent = Math.round(jetToolbarState.zoom * 100) + '%';
    measureToggle.classList.toggle('jet-active-tool', Boolean(jetToolbarState.measure));
    outlineToggle.classList.toggle('jet-active-tool', Boolean(jetToolbarState.outline));
    jetPostCanvasTools();
    jetSaveToolbarState();
  }}

  function jetPostCanvasTools() {{
    const win = frame.contentWindow;
    if (!win) return;
    win.postMessage({{
      type: 'jet-canvas-tools',
      measure: Boolean(jetToolbarState.measure),
      outline: Boolean(jetToolbarState.outline),
    }}, '*');
  }}

  window.jetStoriesHighlight = (selectors) => {{
    const list = Array.isArray(selectors) ? selectors : [selectors];
    const win = frame.contentWindow;
    if (win) win.postMessage({{ type: 'jet-highlight', selectors: list.filter(Boolean) }}, '*');
  }};
  window.jetStoriesClearHighlight = () => {{
    const win = frame.contentWindow;
    if (win) win.postMessage({{ type: 'jet-highlight', selectors: [] }}, '*');
  }};

  viewportSelect.addEventListener('change', () => {{
    jetToolbarState.viewport = viewportSelect.value;
    jetApplyToolbar();
  }});
  backgroundSelect.addEventListener('change', () => {{
    jetToolbarState.background = backgroundSelect.value;
    jetApplyToolbar();
  }});
  zoomOutButton.addEventListener('click', () => {{
    jetToolbarState.zoom = Math.max(0.25, Math.round((jetToolbarState.zoom - 0.1) * 100) / 100);
    jetApplyToolbar();
  }});
  zoomResetButton.addEventListener('click', () => {{
    jetToolbarState.zoom = 1;
    jetApplyToolbar();
  }});
  zoomInButton.addEventListener('click', () => {{
    jetToolbarState.zoom = Math.min(3, Math.round((jetToolbarState.zoom + 0.1) * 100) / 100);
    jetApplyToolbar();
  }});
  measureToggle.addEventListener('click', () => {{
    jetToolbarState.measure = !jetToolbarState.measure;
    jetApplyToolbar();
  }});
  outlineToggle.addEventListener('click', () => {{
    jetToolbarState.outline = !jetToolbarState.outline;
    jetApplyToolbar();
  }});

  // ─── Controls panel (B3) ─────────────────────────────────────────────────
  // Live-edited args are keyed by story id. Switching stories re-fetches the
  // selected story's controls and never leaks args from another story.
  function jetCurrentArgs() {{
    if (!jetCurrentStoryId) return {{}};
    if (!jetStoryArgs.has(jetCurrentStoryId)) jetStoryArgs.set(jetCurrentStoryId, {{}});
    return jetStoryArgs.get(jetCurrentStoryId);
  }}

  function jetParseArgValue(text) {{
    try {{ return JSON.parse(text); }} catch (_) {{ return text; }}
  }}

  function jetHydrateArgsFromUrl(storyId) {{
    if (!storyId) return;
    const params = new URLSearchParams(location.search);
    const encoded = params.get('args');
    if (!encoded) return;
    const args = jetStoryArgs.get(storyId) || {{}};
    encoded.split(';').forEach((pair) => {{
      if (!pair) return;
      const idx = pair.indexOf(':');
      if (idx <= 0) return;
      const key = decodeURIComponent(pair.slice(0, idx));
      const value = decodeURIComponent(pair.slice(idx + 1));
      args[key] = jetParseArgValue(value);
    }});
    jetStoryArgs.set(storyId, args);
  }}

  function jetSyncArgsToUrl() {{
    const params = new URLSearchParams(location.search);
    if (jetCurrentStoryId) params.set('story', jetCurrentStoryId);
    const args = jetCurrentArgs();
    const pairs = Object.keys(args).sort().map((key) =>
      encodeURIComponent(key) + ':' + encodeURIComponent(JSON.stringify(args[key]))
    );
    if (pairs.length) params.set('args', pairs.join(';')); else params.delete('args');
    history.replaceState(null, '', '?' + params.toString());
  }}

  // Coerce a control's DOM value to the arg type the control declares.
  function jetControlValue(el) {{
    const kind = el.dataset.kind;
    if (kind === 'toggle') return el.checked;
    if (kind === 'number' || kind === 'range') {{
      const n = el.value.trim();
      if (n === '') return undefined;
      const f = Number(n);
      return Number.isNaN(f) ? el.value : f;
    }}
    if (kind === 'object') {{
      const text = el.value.trim();
      if (text === '') return undefined;
      return jetParseArgValue(text);
    }}
    if (kind === 'multi-select') {{
      return Array.from(el.selectedOptions).map((option) => option.value);
    }}
    if (kind === 'check') {{
      return Array.from(controlsBody.querySelectorAll('[data-control]'))
        .filter((candidate) => candidate.dataset.control === el.dataset.control && candidate.checked)
        .map((candidate) => candidate.value);
    }}
    if (kind === 'file') {{
      return Array.from(el.files || []).map((file) => file.name);
    }}
    return el.value;
  }}

  function jetMappedValue(el, value) {{
    if (!el.dataset.mapping || value === undefined) return value;
    let mapping = null;
    try {{ mapping = JSON.parse(el.dataset.mapping); }} catch (_) {{ return value; }}
    const mapOne = (item) => Object.prototype.hasOwnProperty.call(mapping, item) ? mapping[item] : item;
    return Array.isArray(value) ? value.map(mapOne) : mapOne(value);
  }}

  // Post the current args into the preview frame so it re-renders. The preview
  // client applies them through window.__jetStoriesRender (see render_preview_html).
  function jetPushArgs() {{
    const win = frame.contentWindow;
    if (!win) return;
    win.postMessage({{ type: 'jet-stories-args', args: jetCurrentArgs() }}, '*');
  }}

  function jetApplyArgsToControls(args) {{
    controlsBody.querySelectorAll('[data-control]').forEach((el) => {{
      const name = el.dataset.control;
      if (!Object.prototype.hasOwnProperty.call(args, name)) return;
      const value = args[name];
      if (el.dataset.kind === 'toggle') {{
        el.checked = Boolean(value);
      }} else if (el.dataset.kind === 'radio') {{
        el.checked = String(value) === el.value;
      }} else if (el.dataset.kind === 'check') {{
        const values = Array.isArray(value) ? value.map(String) : [String(value)];
        el.checked = values.includes(el.value);
      }} else if (el.dataset.kind === 'multi-select') {{
        const values = Array.isArray(value) ? value.map(String) : [String(value)];
        Array.from(el.options).forEach((option) => {{ option.selected = values.includes(option.value); }});
      }} else if (el.dataset.kind === 'object') {{
        el.value = value == null ? '' : JSON.stringify(value, null, 2);
      }} else if (el.dataset.kind === 'file') {{
        // Browser security prevents programmatic file selection.
      }} else {{
        el.value = value == null ? '' : String(value);
      }}
    }});
  }}

  function jetWireControls() {{
    controlsBody.querySelectorAll('[data-control]').forEach((el) => {{
      const name = el.dataset.control;
      const onEdit = () => {{
        const args = jetCurrentArgs();
        const v = jetMappedValue(el, jetControlValue(el));
        if (v === undefined) {{ delete args[name]; }} else {{ args[name] = v; }}
        jetSyncArgsToUrl();
        jetPushArgs();
      }};
      el.addEventListener('input', onEdit);
      el.addEventListener('change', onEdit);
    }});
  }}

  async function jetLoadControls(storyId) {{
    if (!storyId) return;
    try {{
      const response = await fetch('/__jet_stories_controls/' + encodeURIComponent(storyId));
      if (!response.ok) throw new Error('controls ' + response.status);
      const payload = await response.json();
      if (jetCurrentStoryId !== storyId) return;
      controlsBody.innerHTML = payload.html;
      if (!jetStoryArgs.has(storyId)) {{
        jetStoryArgs.set(storyId, payload.args || {{}});
      }}
      jetHydrateArgsFromUrl(storyId);
      jetApplyArgsToControls(jetCurrentArgs());
      jetWireControls();
      jetPushArgs();
    }} catch (err) {{
      controlsBody.innerHTML = '<p class="jet-no-controls">Controls unavailable.</p>';
      console.warn('[jet stories] failed to load controls', err);
    }}
  }}

  frame.addEventListener('load', () => {{
    jetPushArgs();
    jetPostCanvasTools();
  }});
  jetPopulateToolbar();
  jetApplyToolbar();
  jetRenderA11y();
  jetRenderSource();
  jetWireControls();
  const jetInitialParams = new URLSearchParams(location.search);
  const jetSavedStory = localStorage.getItem('jet-stories-last-story');
  if (!jetInitialParams.has('story') && !jetInitialParams.has('docs') && jetSavedStory) {{
    const savedLink = document.querySelector('a.jet-story[data-story-id="' + CSS.escape(jetSavedStory) + '"]');
    if (savedLink) savedLink.click();
  }}
</script>
</body>
</html>
"#,
        sidebar = sidebar,
        diagnostics = diagnostics,
        controls_panel = controls_panel,
        initial_args_json = initial_args_json,
        toolbar_config_json = toolbar_config_json,
        docs_pages = docs_pages_html,
        brand_title = escape_html(&brand.title),
        brand_accent = escape_html(&brand.accent),
        source_panel_json = source_panel_json(index),
        initial_src = escape_html(&initial_src),
        initial_story_id = escape_js(initial_id),
        initial_title = escape_html(
            selected_entry
                .map(story_display_title)
                .unwrap_or_else(|| "No stories".to_string())
                .as_str()
        ),
        story_count = story_count,
    )
}

/// Render the JSON payload consumed by the manager when a story selection needs
/// fresh Controls markup + seed args.
pub fn render_controls_payload_json(controls: &[Control]) -> String {
    format!(
        "{{\"html\":{},\"args\":{}}}",
        json_string(&render_controls_panel(controls)),
        controls_to_args_json(controls)
    )
}

fn source_panel_json(index: &StoryIndex) -> String {
    let mut out = String::from("{");
    for (idx, story) in index.stories.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&json_string(&story.id));
        out.push(':');
        out.push_str(&json_string(story.source.as_deref().unwrap_or("")));
    }
    out.push('}');
    out
}

struct ManagerBrandConfig {
    title: String,
    accent: String,
}

fn manager_brand_config(index: &StoryIndex) -> ManagerBrandConfig {
    let mut config = ManagerBrandConfig {
        title: "jet stories".to_string(),
        accent: "#4338ca".to_string(),
    };
    for meta in &index.metas {
        let Some(manager) = object_field(&meta.parameters, "manager") else {
            continue;
        };
        if let Some(title) = string_field(manager, "brandTitle") {
            config.title = title;
        }
        if let Some(accent) = string_field(manager, "accentColor") {
            config.accent = accent;
        }
        break;
    }
    config
}

fn toolbar_config_json(index: &StoryIndex) -> String {
    let mut out = String::from("{\"__default\":");
    out.push_str(&toolbar_config_entry_json(
        &default_viewports(),
        &default_backgrounds(),
    ));
    for story in &index.stories {
        let (viewports, backgrounds) = toolbar_config_for_story(story);
        if viewports.is_empty() && backgrounds.is_empty() {
            continue;
        }
        out.push(',');
        out.push_str(&json_string(&story.id));
        out.push(':');
        out.push_str(&toolbar_config_entry_json(&viewports, &backgrounds));
    }
    out.push('}');
    out
}

fn toolbar_config_entry_json(
    viewports: &BTreeMap<String, ToolbarViewport>,
    backgrounds: &BTreeMap<String, ToolbarBackground>,
) -> String {
    format!(
        "{{\"viewports\":{},\"backgrounds\":{}}}",
        toolbar_viewports_json(viewports),
        toolbar_backgrounds_json(backgrounds)
    )
}

#[derive(Debug, Clone)]
struct ToolbarViewport {
    name: String,
    width: String,
    height: String,
}

#[derive(Debug, Clone)]
struct ToolbarBackground {
    name: String,
    value: String,
}

fn default_viewports() -> BTreeMap<String, ToolbarViewport> {
    BTreeMap::from([
        (
            "responsive".to_string(),
            ToolbarViewport {
                name: "Responsive".to_string(),
                width: "100%".to_string(),
                height: "100%".to_string(),
            },
        ),
        (
            "mobile".to_string(),
            ToolbarViewport {
                name: "Mobile".to_string(),
                width: "360px".to_string(),
                height: "640px".to_string(),
            },
        ),
        (
            "tablet".to_string(),
            ToolbarViewport {
                name: "Tablet".to_string(),
                width: "768px".to_string(),
                height: "1024px".to_string(),
            },
        ),
        (
            "desktop".to_string(),
            ToolbarViewport {
                name: "Desktop".to_string(),
                width: "1280px".to_string(),
                height: "800px".to_string(),
            },
        ),
    ])
}

fn default_backgrounds() -> BTreeMap<String, ToolbarBackground> {
    BTreeMap::from([
        (
            "transparent".to_string(),
            ToolbarBackground {
                name: "Transparent".to_string(),
                value: "#ffffff".to_string(),
            },
        ),
        (
            "light".to_string(),
            ToolbarBackground {
                name: "Light".to_string(),
                value: "#ffffff".to_string(),
            },
        ),
        (
            "dark".to_string(),
            ToolbarBackground {
                name: "Dark".to_string(),
                value: "#333333".to_string(),
            },
        ),
    ])
}

fn toolbar_config_for_story(
    story: &StoryEntry,
) -> (
    BTreeMap<String, ToolbarViewport>,
    BTreeMap<String, ToolbarBackground>,
) {
    let mut viewports = BTreeMap::new();
    let mut backgrounds = BTreeMap::new();

    if let Some(viewport) = object_field(&story.parameters, "viewport") {
        if let Some(custom) = object_field(viewport, "viewports") {
            for (id, value) in custom {
                let CsfValue::Object(config) = value else {
                    continue;
                };
                let Some(styles) = object_field(config, "styles") else {
                    continue;
                };
                let width = string_field(styles, "width").unwrap_or_else(|| "100%".to_string());
                let height = string_field(styles, "height").unwrap_or_else(|| "100%".to_string());
                viewports.insert(
                    id.clone(),
                    ToolbarViewport {
                        name: string_field(config, "name").unwrap_or_else(|| id.clone()),
                        width,
                        height,
                    },
                );
            }
        }
    }

    if let Some(bg) = object_field(&story.parameters, "backgrounds") {
        if let Some(values) = object_field(bg, "values") {
            for (id, value) in values {
                let CsfValue::Object(config) = value else {
                    continue;
                };
                if let Some(color) = string_field(config, "value") {
                    backgrounds.insert(
                        id.clone(),
                        ToolbarBackground {
                            name: string_field(config, "name").unwrap_or_else(|| id.clone()),
                            value: color,
                        },
                    );
                }
            }
        }
        if let Some(CsfValue::Raw(raw)) = bg.get("values") {
            for (id, background) in parse_background_values_array(raw) {
                backgrounds.insert(id, background);
            }
        }
    }

    (viewports, backgrounds)
}

fn toolbar_viewports_json(viewports: &BTreeMap<String, ToolbarViewport>) -> String {
    let mut out = String::from("{");
    for (idx, (id, viewport)) in viewports.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&json_string(id));
        out.push_str(&format!(
            ":{{\"name\":{},\"width\":{},\"height\":{}}}",
            json_string(&viewport.name),
            json_string(&viewport.width),
            json_string(&viewport.height)
        ));
    }
    out.push('}');
    out
}

fn toolbar_backgrounds_json(backgrounds: &BTreeMap<String, ToolbarBackground>) -> String {
    let mut out = String::from("{");
    for (idx, (id, background)) in backgrounds.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&json_string(id));
        out.push_str(&format!(
            ":{{\"name\":{},\"value\":{}}}",
            json_string(&background.name),
            json_string(&background.value)
        ));
    }
    out.push('}');
    out
}

fn object_field<'a>(
    map: &'a BTreeMap<String, CsfValue>,
    key: &str,
) -> Option<&'a BTreeMap<String, CsfValue>> {
    match map.get(key) {
        Some(CsfValue::Object(object)) => Some(object),
        _ => None,
    }
}

fn string_field(map: &BTreeMap<String, CsfValue>, key: &str) -> Option<String> {
    match map.get(key) {
        Some(CsfValue::Str(value)) => Some(value.clone()),
        Some(CsfValue::Number(value)) => Some(value.clone()),
        Some(CsfValue::Raw(value)) => Some(value.trim_matches(['"', '\'']).to_string()),
        _ => None,
    }
}

fn parse_background_values_array(raw: &str) -> Vec<(String, ToolbarBackground)> {
    let mut out = Vec::new();
    for object in raw.split('{').skip(1) {
        let object = object.split('}').next().unwrap_or("");
        let Some(name) = raw_object_string_field(object, "name") else {
            continue;
        };
        let Some(value) = raw_object_string_field(object, "value") else {
            continue;
        };
        out.push((toolbar_id(&name), ToolbarBackground { name, value }));
    }
    out
}

fn toolbar_id(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

fn raw_object_string_field(object: &str, key: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let needle = format!("{key}: {quote}");
        if let Some(start) = object.find(&needle) {
            let rest = &object[start + needle.len()..];
            let end = rest.find(quote)?;
            return Some(rest[..end].to_string());
        }
        let needle = format!("{quote}{key}{quote}: {quote}");
        if let Some(start) = object.find(&needle) {
            let rest = &object[start + needle.len()..];
            let end = rest.find(quote)?;
            return Some(rest[..end].to_string());
        }
    }
    None
}

/// Render the Controls panel body (B3): one row per control with an editable
/// widget seeded with the story's current arg value. An empty list renders a
/// "no controls" placeholder so the panel is never blank without explanation.
fn render_controls_panel(controls: &[Control]) -> String {
    if controls.is_empty() {
        return "<p class=\"jet-no-controls\">No controls for this story.</p>".to_string();
    }
    let mut out = String::from("<table>");
    for control in controls {
        out.push_str("<tr><td class=\"jet-control-name\">");
        out.push_str(&escape_html(&control.name));
        out.push_str("</td><td>");
        out.push_str(&render_control_widget(control));
        out.push_str("</td></tr>");
    }
    out.push_str("</table>");
    out
}

/// Render a single control's input widget, seeded with its current value. Each
/// widget carries `data-control="<name>"` (the arg it edits) and `data-kind`
/// (so the manager script coerces the value to the right JS type).
fn render_control_widget(control: &Control) -> String {
    let name = escape_html(&control.name);
    let mapping_attr = control_mapping_attr(control);
    match &control.kind {
        ControlKind::Toggle => {
            let checked = matches!(control.current, Some(CsfValue::Bool(true)));
            format!(
                "<input type=\"checkbox\" data-control=\"{name}\" data-kind=\"toggle\"{mapping}{checked} />",
                name = name,
                mapping = mapping_attr,
                checked = if checked { " checked" } else { "" },
            )
        }
        ControlKind::Number => {
            let value = control
                .current
                .as_ref()
                .map(current_value_string)
                .unwrap_or_default();
            format!(
                "<input type=\"number\" data-control=\"{name}\" data-kind=\"number\"{mapping} value=\"{value}\" />",
                name = name,
                mapping = mapping_attr,
                value = escape_html(&value),
            )
        }
        ControlKind::Color => {
            let value = control
                .current
                .as_ref()
                .map(current_value_string)
                .unwrap_or_else(|| "#000000".to_string());
            format!(
                "<input type=\"color\" data-control=\"{name}\" data-kind=\"color\"{mapping} value=\"{value}\" />",
                name = name,
                mapping = mapping_attr,
                value = escape_html(&value),
            )
        }
        ControlKind::Date => {
            let value = control
                .current
                .as_ref()
                .map(current_value_string)
                .unwrap_or_default();
            format!(
                "<input type=\"date\" data-control=\"{name}\" data-kind=\"date\"{mapping} value=\"{value}\" />",
                name = name,
                mapping = mapping_attr,
                value = escape_html(&value),
            )
        }
        ControlKind::Range { min, max, step } => {
            let value = control
                .current
                .as_ref()
                .map(current_value_string)
                .unwrap_or_default();
            format!(
                "<input type=\"range\" data-control=\"{name}\" data-kind=\"range\"{mapping}{min}{max}{step} value=\"{value}\" />",
                name = name,
                mapping = mapping_attr,
                min = attr_if_some("min", min.as_deref()),
                max = attr_if_some("max", max.as_deref()),
                step = attr_if_some("step", step.as_deref()),
                value = escape_html(&value),
            )
        }
        ControlKind::Object => {
            let value = control
                .current
                .as_ref()
                .map(object_value_string)
                .unwrap_or_default();
            format!(
                "<textarea data-control=\"{name}\" data-kind=\"object\"{mapping}>{value}</textarea>",
                name = name,
                mapping = mapping_attr,
                value = escape_html(&value),
            )
        }
        ControlKind::Select { options } => {
            let opts = render_options(options, control, false);
            format!(
                "<select data-control=\"{name}\" data-kind=\"select\"{mapping}>{opts}</select>",
                name = name,
                mapping = mapping_attr,
                opts = opts,
            )
        }
        ControlKind::Radio { options, inline } => render_choice_inputs(
            "radio",
            "radio",
            options,
            control,
            *inline,
            false,
            &mapping_attr,
        ),
        ControlKind::Check { options, inline } => render_choice_inputs(
            "checkbox",
            "check",
            options,
            control,
            *inline,
            true,
            &mapping_attr,
        ),
        ControlKind::MultiSelect { options } => {
            let opts = render_options(options, control, true);
            format!(
                "<select multiple data-control=\"{name}\" data-kind=\"multi-select\"{mapping}>{opts}</select>",
                name = name,
                mapping = mapping_attr,
                opts = opts,
            )
        }
        ControlKind::File => {
            format!(
                "<input type=\"file\" data-control=\"{name}\" data-kind=\"file\"{mapping} />",
                name = name,
                mapping = mapping_attr,
            )
        }
        ControlKind::Text => {
            let value = control
                .current
                .as_ref()
                .map(current_value_string)
                .unwrap_or_default();
            format!(
                "<input type=\"text\" data-control=\"{name}\" data-kind=\"text\"{mapping} value=\"{value}\" />",
                name = name,
                mapping = mapping_attr,
                value = escape_html(&value),
            )
        }
    }
}

fn attr_if_some(name: &str, value: Option<&str>) -> String {
    value
        .map(|value| format!(" {name}=\"{}\"", escape_html(value)))
        .unwrap_or_default()
}

fn control_mapping_attr(control: &Control) -> String {
    if control.mapping.is_empty() {
        return String::new();
    }
    let json = args_to_json(&control.mapping);
    format!(" data-mapping=\"{}\"", escape_html(&json))
}

fn render_options(options: &[String], control: &Control, multi: bool) -> String {
    let selected_values = current_value_list(control.current.as_ref());
    let mut out = String::new();
    for opt in options {
        let selected = if multi {
            selected_values.iter().any(|value| value == opt)
        } else {
            selected_values
                .first()
                .map(|value| value == opt)
                .unwrap_or(false)
        };
        let label = control.labels.get(opt).unwrap_or(opt);
        out.push_str(&format!(
            "<option value=\"{v}\"{sel}>{label}</option>",
            v = escape_html(opt),
            sel = if selected { " selected" } else { "" },
            label = escape_html(label),
        ));
    }
    out
}

fn render_choice_inputs(
    input_type: &str,
    kind: &str,
    options: &[String],
    control: &Control,
    inline: bool,
    multi: bool,
    mapping_attr: &str,
) -> String {
    let selected_values = current_value_list(control.current.as_ref());
    let mut out = String::new();
    let class = if inline {
        "jet-choice jet-choice-inline"
    } else {
        "jet-choice"
    };
    for opt in options {
        let checked = if multi {
            selected_values.iter().any(|value| value == opt)
        } else {
            selected_values
                .first()
                .map(|value| value == opt)
                .unwrap_or(false)
        };
        let label = control.labels.get(opt).unwrap_or(opt);
        out.push_str(&format!(
            "<label class=\"{class}\"><input type=\"{input_type}\" name=\"jet-control-{name}\" data-control=\"{name}\" data-kind=\"{kind}\"{mapping} value=\"{value}\"{checked} /> {label}</label>",
            class = class,
            input_type = input_type,
            name = escape_html(&control.name),
            kind = kind,
            mapping = mapping_attr,
            value = escape_html(opt),
            checked = if checked { " checked" } else { "" },
            label = escape_html(label),
        ));
    }
    out
}

/// Render a [`CsfValue`] as a plain string for seeding an input's `value` or
/// matching a `<select>` option.
fn current_value_string(value: &CsfValue) -> String {
    match value {
        CsfValue::Str(s) => s.clone(),
        CsfValue::Bool(b) => b.to_string(),
        CsfValue::Number(n) => n.clone(),
        CsfValue::Null => String::new(),
        CsfValue::Object(_) => object_value_string(value),
        CsfValue::Raw(raw) => raw.clone(),
    }
}

fn object_value_string(value: &CsfValue) -> String {
    match value {
        CsfValue::Object(_) => value_to_json(value),
        CsfValue::Raw(raw) => raw.clone(),
        _ => current_value_string(value),
    }
}

fn current_value_list(value: Option<&CsfValue>) -> Vec<String> {
    match value {
        Some(CsfValue::Raw(raw)) if raw.trim().starts_with('[') => parse_raw_array_values(raw),
        Some(CsfValue::Str(s)) => vec![s.clone()],
        Some(CsfValue::Number(n)) => vec![n.clone()],
        Some(CsfValue::Bool(b)) => vec![b.to_string()],
        Some(CsfValue::Object(_)) | Some(CsfValue::Null) | None => Vec::new(),
        Some(CsfValue::Raw(raw)) => vec![raw.clone()],
    }
}

fn parse_raw_array_values(raw: &str) -> Vec<String> {
    let Some(inner) = raw
        .trim()
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
    else {
        return Vec::new();
    };
    inner
        .split(',')
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                None
            } else {
                Some(part.trim_matches(['"', '\'']).to_string())
            }
        })
        .collect()
}

/// Serialize the controls' current values into a JSON args object literal for
/// the manager's live `jetArgs` seed (mirrors [`args_to_json`]'s value rules).
fn controls_to_args_json(controls: &[Control]) -> String {
    let mut map: BTreeMap<String, super::csf::CsfValue> = BTreeMap::new();
    for control in controls {
        if let Some(value) = &control.current {
            map.insert(control.name.clone(), value.clone());
        }
    }
    args_to_json(&map)
}

/// The sidebar tree: stories grouped by their full title path so the sidebar
/// mirrors the hierarchy the user authored via `meta.title`.
fn render_sidebar(
    index: &StoryIndex,
    active_id: &str,
    mode: UrlMode,
    docs_pages: &[DocsPage],
) -> String {
    if index.stories.is_empty() {
        return "<p class=\"jet-diag\">No stories discovered.</p>".to_string();
    }

    let mut groups: BTreeMap<String, Vec<&StoryEntry>> = BTreeMap::new();
    for story in &index.stories {
        groups
            .entry(story.title_path.join(" / "))
            .or_default()
            .push(story);
    }

    let mut out = String::from("<ul>");
    for (title, stories) in &groups {
        out.push_str("<li class=\"jet-group\"><span>");
        out.push_str(&escape_html(title));
        out.push_str("</span><ul>");
        if let Some(docs) = docs_pages.iter().find(|page| page.title == *title) {
            out.push_str(&format!(
                "<li><a class=\"jet-docs-link\" href=\"#docs-{id}\" data-docs-id=\"{id}\" data-title=\"{title} — Docs\">Docs</a></li>",
                id = escape_html(&docs.id),
                title = escape_html(&docs.title),
            ));
        }
        for story in stories {
            let preview = mode.preview_url(&story.id);
            let active = if story.id == active_id {
                " jet-active"
            } else {
                ""
            };
            out.push_str(&format!(
                "<li><a class=\"jet-story{active}\" href=\"{preview}\" target=\"jet-preview\" \
                 data-preview=\"{preview}\" data-story-id=\"{id}\" data-title=\"{full_title}\">{name}</a></li>",
                active = active,
                preview = escape_html(&preview),
                id = escape_html(&story.id),
                full_title = escape_html(&story_display_title(story)),
                name = escape_html(&story.name),
            ));
        }
        out.push_str("</ul></li>");
    }
    out.push_str("</ul>");
    out
}

fn render_docs_pages(docs_pages: &[DocsPage], mode: UrlMode) -> String {
    let mut out = String::new();
    for page in docs_pages {
        out.push_str(&format!(
            "<article class=\"jet-docs-page\" data-docs-id=\"{id}\"><h1>{title}</h1>",
            id = escape_html(&page.id),
            title = escape_html(&page.title),
        ));
        if let Some(content_html) = &page.content_html {
            out.push_str(&render_docs_custom_html(content_html, mode));
            out.push_str("</article>");
            continue;
        }
        if !page.description.is_empty() {
            out.push_str(&format!(
                "<p class=\"jet-docs-description\">{}</p>",
                escape_html(&page.description)
            ));
        }
        if !page.primary_story_id.is_empty() {
            out.push_str("<h2>Primary</h2>");
            out.push_str(&format!(
                "<iframe title=\"{} primary story\" src=\"{}\"></iframe>",
                escape_html(&page.title),
                escape_html(&mode.preview_url(&page.primary_story_id)),
            ));
        }
        out.push_str("<h2>ArgTypes</h2>");
        out.push_str(&render_docs_argtypes(&page.arg_types));
        out.push_str("<h2>Stories</h2><div class=\"jet-docs-canvas-grid\">");
        for story in &page.stories {
            out.push_str(&format!(
                "<section class=\"jet-docs-canvas\"><h3>{name}</h3><iframe title=\"{title} {name}\" src=\"{src}\"></iframe></section>",
                name = escape_html(&story.name),
                title = escape_html(&page.title),
                src = escape_html(&mode.preview_url(&story.id)),
            ));
        }
        out.push_str("</div></article>");
    }
    out
}

fn render_docs_custom_html(content_html: &str, mode: UrlMode) -> String {
    let mut out = String::new();
    let mut rest = content_html;
    while let Some(start) = rest.find("{{jet-preview:") {
        out.push_str(&rest[..start]);
        let after = &rest[start + "{{jet-preview:".len()..];
        let Some(end) = after.find("}}") else {
            out.push_str(&rest[start..]);
            return out;
        };
        let story_id = &after[..end];
        out.push_str(&escape_html(&mode.preview_url(story_id)));
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

fn render_docs_argtypes(arg_types: &[DocsArgType]) -> String {
    if arg_types.is_empty() {
        return "<p class=\"jet-no-controls\">No props extracted.</p>".to_string();
    }
    let mut out = String::from(
        "<table class=\"jet-docs-argtypes\"><thead><tr><th>Name</th><th>Type</th><th>Default</th><th>Description</th></tr></thead><tbody>",
    );
    for arg in arg_types {
        out.push_str(&format!(
            "<tr><td>{name}</td><td><code>{type_text}</code></td><td>{default}</td><td>{description}</td></tr>",
            name = escape_html(&arg.name),
            type_text = escape_html(&arg.type_text),
            default = escape_html(arg.default_value.as_deref().unwrap_or("")),
            description = escape_html(&arg.description),
        ));
    }
    out.push_str("</tbody></table>");
    out
}

/// Render per-file diagnostics (parse errors etc.) so the user sees broken
/// story files instead of silently missing entries.
fn render_diagnostics(index: &StoryIndex) -> String {
    if index.diagnostics.is_empty() {
        return String::new();
    }
    let mut out = String::from("<div class=\"jet-diag\"><strong>Diagnostics</strong><ul>");
    for d in &index.diagnostics {
        out.push_str("<li>");
        out.push_str(&escape_html(d));
        out.push_str("</li>");
    }
    out.push_str("</ul></div>");
    out
}

/// `Components / Button — Primary` — used in the toolbar + sidebar tooltips.
fn story_display_title(story: &StoryEntry) -> String {
    if story.title_path.is_empty() {
        story.name.clone()
    } else {
        format!("{} — {}", story.title_path.join(" / "), story.name)
    }
}

/// Render the isolated preview document for one story.
///
/// `module_url` is the URL (served by the module route) of the story's source
/// file, transformed to JS. The document:
///   1. sets up an importmap so bare `react` / `react-dom/client` specifiers
///      resolve to esm.sh CDN modules (no local node_modules needed for the
///      React runtime itself — local relative imports still go through the
///      module route),
///   2. (dev mode, #196) loads the React Fast Refresh runtime from
///      `/@react-refresh` and installs the global `$RefreshReg$` / `$RefreshSig$`
///      hooks BEFORE the story module imports, so the transform-injected
///      component registration resolves and a hot update can refresh in place,
///   3. dynamically imports the story module,
///   4. picks the story's named export and renders it — honoring a custom
///      `render` function when the story declares one, otherwise mounting the
///      meta `component` (or the export value treated as a component),
///   5. mounts the result into a single `#jet-root` div with no surrounding app
///      shell, router, or providers.
pub fn render_preview_html(story: &StoryEntry, module_url: &str) -> String {
    render_preview_html_with_mode(story, module_url, UrlMode::Dev)
}

/// [`render_preview_html`] with an explicit [`UrlMode`].
///
/// [`UrlMode::Dev`] (the wrapper above) injects the preview-frame HMR client and
/// is unchanged. [`UrlMode::Static`] (B4) omits the HMR client entirely — there
/// is no dev server / WebSocket at serve time — so the emitted document is a
/// self-contained, server-less preview; `module_url` is expected to already be a
/// relative URL (e.g. `../modules/src/Button.stories.js`) pointing at an emitted
/// transformed module.
pub fn render_preview_html_with_mode(
    story: &StoryEntry,
    module_url: &str,
    mode: UrlMode,
) -> String {
    render_preview_html_with_project_preview(story, module_url, mode, None)
}

/// Render preview HTML with an optional project-level `.storybook/preview`
/// module URL. When present, the runtime imports that module and applies its
/// decorators, globals/globalTypes, parameters, and loaders before rendering.
pub fn render_preview_html_with_project_preview(
    story: &StoryEntry,
    module_url: &str,
    mode: UrlMode,
    project_preview_url: Option<&str>,
) -> String {
    let args_json = args_to_json(&story.args);
    // B2b/#176: the HMR client lives ONLY in the preview frame, so an edit
    // hot-updates the iframe while the manager shell stays put. The static
    // export has no server to talk to, so it ships no HMR client.
    let hmr_client = match mode {
        UrlMode::Dev => render_preview_hmr_client(),
        UrlMode::Static => String::new(),
    };

    // #196: in dev mode the preview-served `.tsx` modules carry the transform's
    // `$RefreshReg$` / `$RefreshSig$` instrumentation (it imports
    // `RefreshRuntime from '/@react-refresh'`), so the preview installs those
    // globals + loads the runtime BEFORE the story module and registers an
    // in-place refresh callback. The static export has no `/@react-refresh`
    // server route, so it ships none of this (and never refreshes — it's a
    // frozen snapshot).
    let (refresh_setup, refresh_register) = match mode {
        UrlMode::Dev => (
            format!(
                "  // ─── React Fast Refresh registry setup (#196) ────────────────────────────\n\
                 \x20 // Load the refresh runtime FIRST and install the global $RefreshReg$ /\n\
                 \x20 // $RefreshSig$ hooks the preview-served modules expect (the transform injects\n\
                 \x20 // `import RefreshRuntime from '/@react-refresh'` + registration into each\n\
                 \x20 // `.tsx` module). Setting these globals up before importing the story module\n\
                 \x20 // means the story's component types register their families with the runtime,\n\
                 \x20 // so a hot update can re-render the SAME root in place (preserving hook state)\n\
                 \x20 // rather than remounting a fresh root.\n\
                 \x20 import RefreshRuntime from \"{refresh_route}\";\n\
                 \x20 if (!window.$RefreshReg$) {{\n\
                 \x20   window.$RefreshReg$ = RefreshRuntime.register;\n\
                 \x20   window.$RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;\n\
                 \x20   window.__jetRefreshRuntime = RefreshRuntime;\n\
                 \x20 }}\n",
                refresh_route = super::server::REACT_REFRESH_ROUTE,
            ),
            // Register the in-place refresh callback the runtime drives on
            // `performReactRefresh()` (#196). It re-renders the EXISTING root
            // with the most-recently-imported module, so React reconciles the
            // live fiber tree and keeps component hook state instead of
            // unmount + remount.
            "  RefreshRuntime.onPerformReactRefresh(() => renderStory(lastModule));\n"
                .to_string(),
        ),
        UrlMode::Static => (String::new(), String::new()),
    };
    let project_preview_setup = match project_preview_url {
        Some(url) => format!(
            "  import * as ProjectPreview from \"{}\";\n",
            escape_js(url)
        ),
        None => "  const ProjectPreview = {};\n".to_string(),
    };

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<title>{title}</title>
<script type="importmap">
{{
  "imports": {{
    "react": "https://esm.sh/react@18",
    "react-dom": "https://esm.sh/react-dom@18",
    "react-dom/client": "https://esm.sh/react-dom@18/client",
    "react/jsx-runtime": "https://esm.sh/react@18/jsx-runtime",
    "axe-core": "https://esm.sh/axe-core@4.10.3",
    "@storybook/addon-actions": "data:text/javascript,export%20const%20action%3DglobalThis.__jetStoryActionShim%7C%7C((name)%3D%3E(...args)%3D%3Econsole.log(name%2Cargs))%3Bexport%20default%20%7B%20action%20%7D%3B",
    "@storybook/test": "data:text/javascript,export%20function%20fn(impl%3D()%3D%3Eundefined)%7Bconst%20mock%3D(...args)%3D%3E%7Bmock.mock.calls.push(args)%3Breturn%20impl(...args)%7D%3Bmock.mock%3D%7Bcalls%3A%5B%5D%7D%3Breturn%20mock%3B%7D%0Afunction%20textOf(n)%7Breturn%20(n%26%26n.textContent)%7C%7C%22%22%3B%7D%0Afunction%20byText(root%2Ctext)%7Bconst%20wanted%3DString(text)%3Bconst%20nodes%3D%5Broot%2C...root.querySelectorAll(%22*%22)%5D%3Bconst%20found%3Dnodes.find((n)%3D%3EtextOf(n).includes(wanted))%3Bif(!found)throw%20new%20Error(%22Unable%20to%20find%20text%3A%20%22%2Bwanted)%3Breturn%20found%3B%7D%0Aexport%20function%20within(root)%7Breturn%20%7BgetByText%3A(text)%3D%3EbyText(root%2Ctext)%2CqueryByText%3A(text)%3D%3E%7Btry%7Breturn%20byText(root%2Ctext)%7Dcatch(_)%7Breturn%20null%7D%7D%2CgetByRole%3A(role)%3D%3E%7Bconst%20found%3Droot.querySelector(%22%5Brole%3D%5C%22%22%2Brole%2B%22%5C%22%5D%22)%3Bif(!found)throw%20new%20Error(%22Unable%20to%20find%20role%3A%20%22%2Brole)%3Breturn%20found%7D%7D%3B%7D%0Aexport%20const%20userEvent%3D%7Bclick%3Aasync(el)%3D%3E%7Bel.click()%3B%7D%2Ctype%3Aasync(el%2Ctext)%3D%3E%7Bel.focus%26%26el.focus()%3Bel.value%3D(el.value%7C%7C%22%22)%2Btext%3Bel.dispatchEvent(new%20Event(%22input%22%2C%7Bbubbles%3Atrue%7D))%3Bel.dispatchEvent(new%20Event(%22change%22%2C%7Bbubbles%3Atrue%7D))%3B%7D%7D%3B%0Aexport%20function%20expect(actual)%7Bconst%20api%3D%7BtoBe%3A(expected)%3D%3E%7Bif(actual!%3D%3Dexpected)throw%20new%20Error(%22Expected%20%22%2Bactual%2B%22%20to%20be%20%22%2Bexpected)%3B%7D%2CtoEqual%3A(expected)%3D%3E%7Bif(JSON.stringify(actual)!%3D%3DJSON.stringify(expected))throw%20new%20Error(%22Expected%20%22%2BJSON.stringify(actual)%2B%22%20to%20equal%20%22%2BJSON.stringify(expected))%3B%7D%2CtoBeTruthy%3A()%3D%3E%7Bif(!actual)throw%20new%20Error(%22Expected%20value%20to%20be%20truthy%22)%3B%7D%2CtoHaveTextContent%3A(text)%3D%3E%7Bif(!textOf(actual).includes(String(text)))throw%20new%20Error(%22Expected%20text%20content%20to%20include%20%22%2Btext)%3B%7D%7D%3Bapi.not%3D%7BtoBe%3A(expected)%3D%3E%7Bif(actual%3D%3D%3Dexpected)throw%20new%20Error(%22Expected%20%22%2Bactual%2B%22%20not%20to%20be%20%22%2Bexpected)%3B%7D%2CtoBeTruthy%3A()%3D%3E%7Bif(actual)throw%20new%20Error(%22Expected%20value%20to%20be%20falsy%22)%3B%7D%7D%3Breturn%20api%3B%7D%0A"
  }}
}}
</script>
<style> html, body {{ margin: 0; font-family: -apple-system, "system-ui", "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"; line-height: 1.5714285714285714; }} #jet-root {{ box-sizing: border-box; min-height: 100vh; }} #jet-root.jet-layout-padded {{ padding: 16px; }} #jet-root.jet-layout-centered {{ min-height: 100vh; padding: 16px; display: grid; place-items: center; }} #jet-root.jet-layout-fullscreen {{ min-height: 100vh; padding: 0; }} </style>
</head>
<body>
<div id="jet-root" data-story-id="{story_id}"></div>
<script type="module">
{refresh_setup}  // Isolated mount: only this story renders here — no app router/shell.
{project_preview_setup}
  import * as Story from "{module_url}";
  import React from "react";
  import {{ createRoot }} from "react-dom/client";
  import axe from "axe-core";

  const exportName = "{export_name}";
  // `discoveredArgs` are a static fallback from CSF parsing. The transformed
  // story module's own args remain the initial source of truth for complex
  // values such as arrays, object literals, and imported constants.
  const discoveredArgs = {args_json};
  let liveArgs = null;
  let lastModule = Story;
  let playRunToken = 0;
  let a11yRunToken = 0;
  let lastA11yParameters = {{}};
  const rootEl = document.getElementById("jet-root");
  const root = createRoot(rootEl);
  window.__jetStoryTestStatus = {{
    storyId: "{story_id}",
    render: "pending",
    play: "pending",
    a11y: "pending",
    errors: [],
    interactions: [],
  }};
  window.__jetStoryActionShim = (name) => (...args) => postAction(name, args);
  // Exposed so the HMR client (loaded after this module) can re-render the
  // story in place with a freshly re-imported module — state-preserving for
  // react-refresh-compatible edits, isolated to this frame.
  window.__jetStoriesRender = renderStory;
  window.__jetStoriesHighlight = applyHighlight;
{refresh_register}
  let jetMeasureActive = false;
  let jetOutlineStyle = null;
  let jetMeasureOverlay = null;
  let jetHighlightLayer = null;

  window.addEventListener("message", (ev) => {{
    const data = ev && ev.data;
    if (!data || data.type !== "jet-canvas-tools") return;
    setMeasure(Boolean(data.measure));
    setOutline(Boolean(data.outline));
  }});

  window.addEventListener("message", (ev) => {{
    const data = ev && ev.data;
    if (!data || data.type !== "jet-highlight") return;
    applyHighlight(Array.isArray(data.selectors) ? data.selectors : []);
  }});

  window.addEventListener("message", (ev) => {{
    const data = ev && ev.data;
    if (!data || data.type !== "jet-a11y-run") return;
    runA11y(lastA11yParameters);
  }});

  function setOutline(active) {{
    if (active && !jetOutlineStyle) {{
      jetOutlineStyle = document.createElement("style");
      jetOutlineStyle.id = "jet-outline-style";
      jetOutlineStyle.textContent = '#jet-root * {{ outline: 1px solid rgba(67,56,202,.55) !important; outline-offset: -1px !important; }}';
      document.head.appendChild(jetOutlineStyle);
    }} else if (!active && jetOutlineStyle) {{
      jetOutlineStyle.remove();
      jetOutlineStyle = null;
    }}
  }}

  function setMeasure(active) {{
    if (active === jetMeasureActive) return;
    jetMeasureActive = active;
    if (active) {{
      rootEl.addEventListener("mousemove", updateMeasure, true);
      rootEl.addEventListener("mouseleave", clearMeasure, true);
    }} else {{
      rootEl.removeEventListener("mousemove", updateMeasure, true);
      rootEl.removeEventListener("mouseleave", clearMeasure, true);
      clearMeasure();
    }}
  }}

  function ensureMeasureOverlay() {{
    if (jetMeasureOverlay) return jetMeasureOverlay;
    jetMeasureOverlay = document.createElement("div");
    jetMeasureOverlay.id = 'jet-measure-overlay';
    jetMeasureOverlay.style.cssText = 'position:fixed;z-index:2147483647;pointer-events:none;border:2px solid #4338ca;background:rgba(67,56,202,.08);color:#fff;font:12px system-ui;';
    jetMeasureOverlay.innerHTML = '<span style="position:absolute;left:0;top:-20px;background:#4338ca;padding:2px 4px;border-radius:3px;white-space:nowrap"></span>';
    document.body.appendChild(jetMeasureOverlay);
    return jetMeasureOverlay;
  }}

  function updateMeasure(ev) {{
    const target = ev.target;
    if (!target || target === rootEl || target === jetMeasureOverlay) return;
    const rect = target.getBoundingClientRect();
    const overlay = ensureMeasureOverlay();
    overlay.style.left = rect.left + "px";
    overlay.style.top = rect.top + "px";
    overlay.style.width = rect.width + "px";
    overlay.style.height = rect.height + "px";
    overlay.firstElementChild.textContent = Math.round(rect.width) + " x " + Math.round(rect.height);
  }}

  function clearMeasure() {{
    if (jetMeasureOverlay) {{
      jetMeasureOverlay.remove();
      jetMeasureOverlay = null;
    }}
  }}

  function ensureHighlightLayer() {{
    if (jetHighlightLayer) return jetHighlightLayer;
    jetHighlightLayer = document.createElement("div");
    jetHighlightLayer.id = 'jet-highlight-layer';
    jetHighlightLayer.style.cssText = 'position:fixed;inset:0;z-index:2147483646;pointer-events:none;';
    document.body.appendChild(jetHighlightLayer);
    return jetHighlightLayer;
  }}

  function applyHighlight(selectors) {{
    if (jetHighlightLayer) jetHighlightLayer.replaceChildren();
    if (!selectors.length) return;
    const layer = ensureHighlightLayer();
    for (const selector of selectors) {{
      let nodes = [];
      try {{ nodes = Array.from(rootEl.querySelectorAll(selector)); }} catch (_) {{ nodes = []; }}
      for (const node of nodes) {{
        const rect = node.getBoundingClientRect();
        const box = document.createElement("div");
        box.className = 'jet-highlight-box';
        box.style.cssText = 'position:fixed;border:2px solid #f59e0b;background:rgba(245,158,11,.12);left:' + rect.left + 'px;top:' + rect.top + 'px;width:' + rect.width + 'px;height:' + rect.height + 'px;';
        layer.appendChild(box);
      }}
    }}
  }}

  function safeActionArgs(args) {{
    const seen = new WeakSet();
    const cap = (value, depth) => {{
      if (depth > 3) return "[depth capped]";
      if (value == null || typeof value !== "object") return value;
      if (seen.has(value)) return "[cycle]";
      seen.add(value);
      if (Array.isArray(value)) return value.slice(0, 20).map((item) => cap(item, depth + 1));
      const out = {{}};
      for (const [key, item] of Object.entries(value).slice(0, 20)) out[key] = cap(item, depth + 1);
      return out;
    }};
    try {{ return JSON.stringify(args.map((arg) => cap(arg, 0))); }} catch (_) {{ return "[unserializable]"; }}
  }}

  function postAction(name, args) {{
    parent.postMessage({{ type: "jet-action", name, args: safeActionArgs(args), ts: Date.now() }}, "*");
  }}

  function actionNames(meta, story, args) {{
    const names = new Set(Object.keys(args || {{}}).filter((key) => /^on[A-Z]/.test(key)));
    for (const argTypes of [meta && meta.argTypes, story && story.argTypes]) {{
      if (!argTypes) continue;
      for (const [name, config] of Object.entries(argTypes)) {{
        if (config && (config.action || /^on[A-Z]/.test(name))) names.add(name);
      }}
    }}
    return Array.from(names);
  }}

  function injectActionArgs(args, meta, story) {{
    const next = {{ ...args }};
    for (const name of actionNames(meta, story, next)) {{
      const original = next[name];
      next[name] = (...items) => {{
        postAction(name, items);
        if (typeof original === "function") return original(...items);
      }};
    }}
    return next;
  }}

  function postInteraction(status, name, error) {{
    window.__jetStoryTestStatus.interactions.push({{
      status,
      name,
      error: error ? (error.message || String(error)) : "",
      ts: Date.now(),
    }});
    parent.postMessage({{
      type: "jet-interaction",
      status,
      name,
      error: error ? (error.message || String(error)) : "",
      ts: Date.now(),
    }}, "*");
  }}

  function normalizeA11yParameters(parameters) {{
    const config = parameters && parameters.a11y && typeof parameters.a11y === "object"
      ? parameters.a11y
      : {{}};
    if (config.disable === true || config.disabled === true) return {{ disabled: true }};
    const options = config.options && typeof config.options === "object" ? {{ ...config.options }} : {{}};
    if (config.rules && !options.rules) options.rules = config.rules;
    return {{
      disabled: false,
      config: config.config && typeof config.config === "object" ? config.config : null,
      options,
    }};
  }}

  function serializeA11yViolation(violation) {{
    return {{
      id: violation.id || "",
      impact: violation.impact || "",
      help: violation.help || "",
      description: violation.description || "",
      helpUrl: violation.helpUrl || "",
      targets: (violation.nodes || []).flatMap((node) => node.target || []).slice(0, 12),
    }};
  }}

  async function runA11y(parameters) {{
    const token = ++a11yRunToken;
    const a11y = normalizeA11yParameters(parameters);
    if (a11y.disabled) {{
      window.__jetStoryTestStatus.a11y = "disabled";
      parent.postMessage({{ type: "jet-a11y-result", storyId: "{story_id}", status: "disabled", violations: [] }}, "*");
      return;
    }}
    window.__jetStoryTestStatus.a11y = "running";
    await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    if (token !== a11yRunToken) return;
    try {{
      if (a11y.config && axe && typeof axe.configure === "function") axe.configure(a11y.config);
      const result = await axe.run(rootEl, a11y.options || {{}});
      if (token !== a11yRunToken) return;
      window.__jetStoryTestStatus.a11y = "complete";
      parent.postMessage({{
        type: "jet-a11y-result",
        storyId: "{story_id}",
        status: "complete",
        violations: (result.violations || []).map(serializeA11yViolation),
        ts: Date.now(),
      }}, "*");
    }} catch (err) {{
      window.__jetStoryTestStatus.a11y = "error";
      window.__jetStoryTestStatus.errors.push("a11y: " + (err ? (err.message || String(err)) : "unknown axe error"));
      parent.postMessage({{
        type: "jet-a11y-result",
        storyId: "{story_id}",
        status: "error",
        error: err ? (err.message || String(err)) : "unknown axe error",
        violations: [],
        ts: Date.now(),
      }}, "*");
    }}
  }}

  async function runPlay(story, context) {{
    const token = ++playRunToken;
    if (!story || typeof story.play !== "function") {{
      window.__jetStoryTestStatus.play = "skipped";
      return;
    }}
    window.__jetStoryTestStatus.play = "running";
    await new Promise((resolve) => requestAnimationFrame(resolve));
    if (token !== playRunToken) return;
    const step = async (name, body) => {{
      postInteraction("start", name);
      try {{
        const result = await body();
        postInteraction("pass", name);
        return result;
      }} catch (err) {{
        postInteraction("fail", name, err);
        throw err;
      }}
    }};
    try {{
      await story.play({{ ...context, step }});
      if (token === playRunToken) window.__jetStoryTestStatus.play = "pass";
    }} catch (err) {{
      if (token === playRunToken) {{
        window.__jetStoryTestStatus.play = "fail";
        window.__jetStoryTestStatus.errors.push("play: " + (err ? (err.message || String(err)) : "unknown play error"));
      }}
      postInteraction("fail", "play", err);
    }}
  }}

  // B3: apply control edits from the manager. The manager posts the full args
  // object; we replace liveArgs and re-render the most recent module in place.
  window.addEventListener("message", (ev) => {{
    const data = ev && ev.data;
    if (!data || data.type !== "jet-stories-args" || !data.args) return;
    liveArgs = data.args;
    renderStory(lastModule);
  }});

  function normalizeList(value) {{
    if (!value) return [];
    return Array.isArray(value) ? value : [value];
  }}

  function mergeObjects(...values) {{
    return Object.assign({{}}, ...values.filter((value) => value && typeof value === "object"));
  }}

  function globalDefaults(...types) {{
    const out = {{}};
    for (const globalTypes of types) {{
      if (!globalTypes || typeof globalTypes !== "object") continue;
      for (const [name, config] of Object.entries(globalTypes)) {{
        if (config && Object.prototype.hasOwnProperty.call(config, "defaultValue")) {{
          out[name] = config.defaultValue;
        }}
      }}
    }}
    return out;
  }}

  function applyLayout(layout) {{
    const value = layout === "centered" || layout === "fullscreen" ? layout : "padded";
    rootEl.classList.remove("jet-layout-padded", "jet-layout-centered", "jet-layout-fullscreen");
    rootEl.classList.add("jet-layout-" + value);
  }}

  function pickComponent(mod) {{
    const story = mod[exportName];
    // A story may BE a component (function/class) or a CSF object with a
    // `render`/`component` field. Resolve to a renderable React element factory.
    if (typeof story === "function") return (props) => React.createElement(story, props);
    if (story && typeof story.render === "function") return (props, context) => story.render(props, context);
    if (story && story.component) return (props) => React.createElement(story.component, props);
    const meta = mod.default;
    if (meta && meta.component) return (props) => React.createElement(meta.component, props);
    return null;
  }}

  async function runLoaders(loaders, context) {{
    let loaded = {{}};
    let current = {{ ...context, loaded }};
    for (const loader of loaders) {{
      if (typeof loader !== "function") continue;
      const result = await loader(current);
      if (result && typeof result === "object") {{
        loaded = {{ ...loaded, ...result }};
        current = {{ ...current, loaded }};
      }}
    }}
    return loaded;
  }}

  function renderWithDecorators(factory, context, decorators) {{
    let storyFn = (nextContext = context) => factory(nextContext.args, nextContext);
    storyFn.args = context.args;
    for (const decorator of decorators) {{
      if (typeof decorator !== "function") continue;
      const previous = storyFn;
      storyFn = (nextContext = context) => decorator(previous, nextContext);
      storyFn.args = context.args;
    }}
    return storyFn(context);
  }}

  async function renderStory(mod) {{
    lastModule = mod;
    window.__jetStoryTestStatus.render = "running";
    window.__jetStoryTestStatus.play = "pending";
    window.__jetStoryTestStatus.errors = [];
    window.__jetStoryTestStatus.interactions = [];
    try {{
      const story = mod[exportName];
      const meta = mod.default || {{}};
      const factory = pickComponent(mod);
      if (!factory) {{
        window.__jetStoryTestStatus.render = "fail";
        window.__jetStoryTestStatus.play = "skipped";
        window.__jetStoryTestStatus.errors.push("render: missing component for export '" + exportName + "'");
        rootEl.textContent =
          "jet stories: could not resolve a component for export '" + exportName + "'";
      }} else {{
        const authoredArgs = (story && story.args) || {{}};
        const merged = liveArgs
          ? {{ ...authoredArgs, ...liveArgs }}
          : {{ ...discoveredArgs, ...(meta.args || {{}}), ...authoredArgs }};
        const effectiveArgs = injectActionArgs(merged, meta, story);
        const parameters = mergeObjects(
          ProjectPreview.parameters,
          meta.parameters,
          story && story.parameters,
        );
        const globals = mergeObjects(
          globalDefaults(ProjectPreview.globalTypes, meta.globalTypes),
          ProjectPreview.globals,
          meta.globals,
          story && story.globals,
        );
        const contextBase = {{
          id: "{story_id}",
          name: exportName,
          title: "{title_js}",
          args: effectiveArgs,
          argTypes: meta.argTypes || {{}},
          globals,
          parameters,
          canvasElement: rootEl,
          loaded: {{}},
        }};
        applyLayout(parameters.layout);
        lastA11yParameters = parameters;
        const loaders = [
          ...normalizeList(ProjectPreview.loaders),
          ...normalizeList(meta.loaders),
          ...normalizeList(story && story.loaders),
        ];
        const loaded = await runLoaders(loaders, contextBase);
        const context = {{ ...contextBase, loaded }};
        const decorators = [
          ...normalizeList(story && story.decorators),
          ...normalizeList(meta.decorators),
          ...normalizeList(ProjectPreview.decorators),
        ];
        root.render(renderWithDecorators(factory, context, decorators));
        window.__jetStoryTestStatus.render = "pass";
        runA11y(parameters);
        runPlay(story, context);
      }}
    }} catch (err) {{
      window.__jetStoryTestStatus.render = "fail";
      window.__jetStoryTestStatus.play = "skipped";
      window.__jetStoryTestStatus.errors.push("render: " + (err && err.message || err));
      rootEl.textContent = "jet stories render error: " + (err && err.message || err);
      console.error(err);
    }}
  }}

  renderStory(Story);
</script>
{hmr_client}
</body>
</html>
"#,
        title = escape_html(&story_display_title(story)),
        title_js = escape_js(&story_display_title(story)),
        story_id = escape_html(&story.id),
        refresh_setup = refresh_setup,
        refresh_register = refresh_register,
        project_preview_setup = project_preview_setup,
        module_url = escape_js(module_url),
        export_name = escape_js(&story.export_name),
        args_json = args_json,
        hmr_client = hmr_client,
    )
}

/// The HMR client `<script>` injected into the **preview frame only** (B2b/#176).
///
/// It connects to the stories HMR WebSocket ([`super::hmr::STORIES_HMR_ROUTE`])
/// and, per message:
///   - `update` → re-import the changed story module (cache-busted by the
///     server's timestamp) and re-render it in place via the
///     `window.__jetStoriesRender` hook the preview module exposes. This is the
///     state-preserving react-refresh path for compatible (component) edits.
///   - `reload` → `location.reload()` *inside this iframe only*, the safe
///     fallback for non-component edits.
///   - `connected` → no-op ack.
///
/// Crucially this script is NOT injected into the manager shell, so the manager
/// never reloads — only the iframe does. Reconnects with exponential backoff so
/// a server restart re-establishes live reload.
///
/// #196 (state-preserving refresh): on `update` the client re-imports the
/// freshly-transformed module — which re-runs the transform-injected
/// `$RefreshReg$(...)` registration, updating each component family's `current`
/// type in the runtime — then calls `RefreshRuntime.performReactRefresh()`. The
/// preview registered an in-place refresh callback (via
/// `RefreshRuntime.onPerformReactRefresh`) that re-renders the EXISTING
/// `createRoot()` root, so React reconciles the live fiber tree and keeps
/// component hook state instead of unmounting + remounting.
///
/// TODO(#196 follow-up): the shim's `performReactRefresh` resolves the new
/// component family by re-running the module's render rather than via React's
/// real `react-refresh/runtime` `injectIntoGlobalHook` family-swap. For the
/// isolated single-component preview a same-root re-render with stable component
/// identity (the family `current` type) preserves hook state in practice; a
/// future pass could wire the upstream `react-refresh/runtime` for parity with
/// multi-component trees / forced-remount on signature change.
fn render_preview_hmr_client() -> String {
    format!(
        r#"<script type="module">
  // ─── jet stories preview HMR client (B2b/#176 + #196) ────────────────────
  // Lives only in the preview frame; the manager shell never reloads.
  (function() {{
    const ROUTE = "{route}";
    let retryDelay = 500;
    const MAX_RETRY_DELAY = 10000;

    async function applyUpdate(msg) {{
      const render = window.__jetStoriesRender;
      if (typeof render !== "function") {{
        // Preview module not ready (or no render hook) — reload to be safe.
        location.reload();
        return;
      }}
      try {{
        // Cache-bust so the browser fetches the freshly transformed module. The
        // re-import re-runs the module's `$RefreshReg$(...)` registration, so the
        // runtime now holds the updated component family.
        const bust = (msg.path.indexOf("?") === -1 ? "?" : "&") + "t=" + msg.timestamp;
        const fresh = await import(msg.path + bust);
        // #196: state-preserving refresh. Hand the freshly-imported module to the
        // render hook (so `lastModule` tracks the new code) and then drive
        // `performReactRefresh()`, which re-renders the EXISTING root in place —
        // preserving hook state — instead of mounting a fresh root.
        const runtime = window.__jetRefreshRuntime;
        if (runtime && typeof runtime.performReactRefresh === "function") {{
          render(fresh);
          runtime.performReactRefresh();
          console.log("[jet stories] react-refresh applied (state preserved)", msg.path);
        }} else {{
          // No refresh runtime (shouldn't happen in dev) — plain re-render.
          render(fresh);
          console.log("[jet stories] hot updated", msg.path);
        }}
      }} catch (err) {{
        console.error("[jet stories] hot update failed, reloading preview:", err);
        location.reload();
      }}
    }}

    function connect() {{
      const protocol = location.protocol === "https:" ? "wss:" : "ws:";
      const ws = new WebSocket(protocol + "//" + location.host + ROUTE);

      ws.onopen = () => {{ retryDelay = 500; console.log("[jet stories] HMR connected"); }};

      ws.onmessage = (event) => {{
        let msg;
        try {{ msg = JSON.parse(event.data); }} catch (_) {{ return; }}
        switch (msg.type) {{
          case "connected":
            break;
          case "update":
            applyUpdate(msg);
            break;
          case "reload":
            // Reload THIS iframe only — the manager shell is untouched.
            console.log("[jet stories] preview reload:", msg.reason);
            location.reload();
            break;
          default:
            console.log("[jet stories] unknown HMR message", msg);
        }}
      }};

      ws.onclose = () => {{
        setTimeout(() => {{
          retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY);
          connect();
        }}, retryDelay);
      }};
      ws.onerror = () => {{ /* close handler reconnects */ }};
    }}

    connect();
  }})();
</script>
"#,
        route = super::hmr::STORIES_HMR_ROUTE,
    )
}

/// Empty-state preview document (no stories discovered).
pub fn render_empty_preview_html() -> String {
    "<!doctype html><html><head><meta charset=\"utf-8\"><title>jet stories</title></head>\
     <body style=\"font-family:sans-serif;padding:24px;color:#666\">\
     <h2>No stories discovered</h2>\
     <p>Add a <code>*.stories.tsx</code> file under your project root.</p>\
     </body></html>"
        .to_string()
}

/// Serialize a [`super::csf::CsfValue`] arg map into a compact JSON object
/// literal usable directly in a `<script>` block. Non-destructurable values
/// (`Raw`) are emitted as JSON strings so the runtime at least sees the source.
fn args_to_json(args: &BTreeMap<String, super::csf::CsfValue>) -> String {
    let mut out = String::from("{");
    let mut first = true;
    for (k, v) in args {
        if !first {
            out.push(',');
        }
        first = false;
        out.push_str(&json_string(k));
        out.push(':');
        out.push_str(&value_to_json(v));
    }
    out.push('}');
    out
}

fn value_to_json(v: &CsfValue) -> String {
    match v {
        CsfValue::Str(s) => json_string(s),
        CsfValue::Bool(b) => b.to_string(),
        CsfValue::Number(n) => {
            if n.parse::<f64>().is_ok() {
                n.clone()
            } else {
                json_string(n)
            }
        }
        CsfValue::Null => "null".to_string(),
        CsfValue::Object(map) => {
            let mut out = String::from("{");
            let mut first = true;
            for (k, val) in map {
                if !first {
                    out.push(',');
                }
                first = false;
                out.push_str(&json_string(k));
                out.push(':');
                out.push_str(&value_to_json(val));
            }
            out.push('}');
            out
        }
        // Raw source (identifiers, JSX, arrow fns) can't be safely evaluated
        // here; surface the source text as a string so it round-trips.
        CsfValue::Raw(s) => json_string(s),
    }
}

/// Minimal JSON string escaping for embedding in a `<script>` literal.
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // Avoid `</script>` breaking out of the script element.
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// HTML-escape a value destined for element text / attribute values.
fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// Escape a value destined for a JS double-quoted string literal in a module
/// `<script>` (used for the module URL + export name).
fn escape_js(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stories::StoryEntry;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn entry(id: &str, name: &str, title: &[&str]) -> StoryEntry {
        StoryEntry {
            id: id.to_string(),
            name: name.to_string(),
            export_name: name.to_string(),
            args: BTreeMap::new(),
            parameters: BTreeMap::new(),
            source: None,
            has_render: false,
            file: PathBuf::from("/x/Foo.stories.tsx"),
            title_path: title.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn manager_lists_each_story_and_points_iframe_at_first() {
        let mut index = StoryIndex::default();
        index.stories.push(entry(
            "components-button--primary",
            "Primary",
            &["Components", "Button"],
        ));
        index.stories.push(entry(
            "components-button--disabled",
            "Disabled",
            &["Components", "Button"],
        ));

        let html = render_manager_html(&index, None, &[]);
        assert!(html.contains("Primary"), "lists Primary");
        assert!(html.contains("Disabled"), "lists Disabled");
        assert!(
            html.contains("Components / Button"),
            "shows the group title"
        );
        // iframe defaults to the FIRST listed story.
        assert!(
            html.contains("src=\"/__jet_stories_preview/components-button--primary\"")
                || html.contains("src=\"/__jet_stories_preview/components-button--disabled\""),
            "iframe src points at a story: {html}"
        );
    }

    #[test]
    fn manager_honors_explicit_selection() {
        let mut index = StoryIndex::default();
        index.stories.push(entry(
            "components-button--primary",
            "Primary",
            &["Components", "Button"],
        ));
        index.stories.push(entry(
            "components-button--disabled",
            "Disabled",
            &["Components", "Button"],
        ));

        let html = render_manager_html(&index, Some("components-button--primary"), &[]);
        assert!(html.contains("src=\"/__jet_stories_preview/components-button--primary\""));
    }

    #[test]
    fn manager_toolbar_renders_viewport_background_zoom_and_custom_parameters() {
        let mut index = StoryIndex::default();
        let mut story = entry("x--full", "Full", &["X"]);
        story.source = Some("export const Full = { args: { label: 'Save' } };".to_string());
        let mut styles: BTreeMap<String, CsfValue> = BTreeMap::new();
        styles.insert("width".into(), CsfValue::Str("390px".into()));
        styles.insert("height".into(), CsfValue::Str("844px".into()));
        let mut phone: BTreeMap<String, CsfValue> = BTreeMap::new();
        phone.insert("name".into(), CsfValue::Str("Phone".into()));
        phone.insert("styles".into(), CsfValue::Object(styles));
        let mut viewports: BTreeMap<String, CsfValue> = BTreeMap::new();
        viewports.insert("phone".into(), CsfValue::Object(phone));
        let mut viewport: BTreeMap<String, CsfValue> = BTreeMap::new();
        viewport.insert("viewports".into(), CsfValue::Object(viewports));

        let mut brand: BTreeMap<String, CsfValue> = BTreeMap::new();
        brand.insert("name".into(), CsfValue::Str("Brand".into()));
        brand.insert("value".into(), CsfValue::Str("#ffcc00".into()));
        let mut values: BTreeMap<String, CsfValue> = BTreeMap::new();
        values.insert("brand".into(), CsfValue::Object(brand));
        let mut backgrounds: BTreeMap<String, CsfValue> = BTreeMap::new();
        backgrounds.insert("values".into(), CsfValue::Object(values));

        story.parameters = BTreeMap::new();
        story
            .parameters
            .insert("viewport".into(), CsfValue::Object(viewport));
        story
            .parameters
            .insert("backgrounds".into(), CsfValue::Object(backgrounds));
        index.stories.push(story);
        let mut manager_params = BTreeMap::new();
        manager_params.insert("brandTitle".into(), CsfValue::Str("Acme Workbench".into()));
        manager_params.insert("accentColor".into(), CsfValue::Str("#0f766e".into()));
        let mut meta_params = BTreeMap::new();
        meta_params.insert("manager".into(), CsfValue::Object(manager_params));
        index.metas.push(crate::stories::StoryMeta {
            file: PathBuf::from("/x/Foo.stories.tsx"),
            component: None,
            title: Some("X".into()),
            title_path: vec!["X".into()],
            args: BTreeMap::new(),
            arg_types: BTreeMap::new(),
            parameters: meta_params,
            tags: Vec::new(),
        });

        let html = render_manager_html(&index, None, &[]);
        assert!(html.contains("Acme Workbench"));
        assert!(html.contains(":root { --jet-accent: #0f766e; }"));
        assert!(html.contains("id=\"jet-search\""));
        assert!(html.contains("id=\"jet-theme-toggle\""));
        assert!(html.contains("id=\"jet-shortcuts-overlay\""));
        assert!(html.contains("localStorage.setItem('jet-stories-last-story'"));
        assert!(html.contains("localStorage.setItem('jet-stories-theme'"));
        assert!(html.contains("jetFilterStories"));
        assert!(html.contains("jetMoveStoryFocus"));
        assert!(html.contains("jet-panel-hidden"));
        assert!(html.contains("jet-fullscreen"));
        assert!(html.contains("id=\"jet-viewport\""));
        assert!(html.contains("id=\"jet-background\""));
        assert!(html.contains("id=\"jet-zoom-out\""));
        assert!(html.contains("id=\"jet-zoom-reset\""));
        assert!(html.contains("id=\"jet-zoom-in\""));
        assert!(html.contains("id=\"jet-measure-toggle\""));
        assert!(html.contains("id=\"jet-outline-toggle\""));
        assert!(html.contains("id=\"jet-actions-log\""));
        assert!(html.contains("id=\"jet-actions-clear\""));
        assert!(html.contains("id=\"jet-interactions-log\""));
        assert!(html.contains("id=\"jet-interactions-clear\""));
        assert!(html.contains("id=\"jet-a11y-log\""));
        assert!(html.contains("id=\"jet-a11y-run\""));
        assert!(html.contains("id=\"jet-source-copy\""));
        assert!(html.contains("id=\"jet-source-code\""));
        assert!(html.contains("sessionStorage.setItem('jet-stories-toolbar'"));
        assert!(html
            .contains("\"phone\":{\"name\":\"Phone\",\"width\":\"390px\",\"height\":\"844px\"}"));
        assert!(html.contains("\"brand\":{\"name\":\"Brand\",\"value\":\"#ffcc00\"}"));
        assert!(html.contains("frame.style.transform = 'scale('"));
        assert!(html.contains("type: 'jet-canvas-tools'"));
        assert!(html.contains("window.jetStoriesHighlight"));
        assert!(html.contains("type !== 'jet-action'"));
        assert!(html.contains("jetActions.set"));
        assert!(html.contains("type !== 'jet-interaction'"));
        assert!(html.contains("jetRenderInteractions"));
        assert!(html.contains("type !== 'jet-a11y-result'"));
        assert!(html.contains("jetRenderA11y"));
        assert!(html.contains("const jetSourceByStory = {"));
        assert!(html.contains("export const Full = { args: { label: 'Save' } };"));
        assert!(html.contains("navigator.clipboard.writeText"));
    }

    #[test]
    fn preview_references_module_and_export() {
        let e = entry(
            "components-button--primary",
            "Primary",
            &["Components", "Button"],
        );
        let html = render_preview_html(&e, "/src/components/Button.stories.tsx");
        assert!(html.contains("import * as Story from \"/src/components/Button.stories.tsx\""));
        assert!(html.contains("const exportName = \"Primary\""));
        assert!(
            html.contains("id=\"jet-root\""),
            "mounts into isolated root div"
        );
        // No app shell / router markers — just the single root.
        assert_eq!(html.matches("id=\"jet-root\"").count(), 1);
    }

    #[test]
    fn args_serialize_to_json() {
        let mut args = BTreeMap::new();
        args.insert(
            "label".to_string(),
            super::super::csf::CsfValue::Str("Hi".into()),
        );
        args.insert(
            "primary".to_string(),
            super::super::csf::CsfValue::Bool(true),
        );
        args.insert(
            "count".to_string(),
            super::super::csf::CsfValue::Number("3".into()),
        );
        let json = args_to_json(&args);
        assert!(json.contains("\"label\":\"Hi\""));
        assert!(json.contains("\"primary\":true"));
        assert!(json.contains("\"count\":3"));
    }

    #[test]
    fn empty_index_renders_empty_state() {
        let index = StoryIndex::default();
        let html = render_manager_html(&index, None, &[]);
        assert!(html.contains("No stories discovered"));
    }

    #[test]
    fn controls_panel_seeds_current_values_and_wires_render_hook() {
        use crate::stories::controls::{Control, ControlKind};
        use crate::stories::csf::CsfValue;

        let mut index = StoryIndex::default();
        index.stories.push(entry(
            "components-button--primary",
            "Primary",
            &["Components", "Button"],
        ));

        let controls = vec![
            Control {
                name: "primary".into(),
                kind: ControlKind::Toggle,
                current: Some(CsfValue::Bool(true)),
                labels: BTreeMap::new(),
                mapping: BTreeMap::new(),
            },
            Control {
                name: "label".into(),
                kind: ControlKind::Text,
                current: Some(CsfValue::Str("Click".into())),
                labels: BTreeMap::new(),
                mapping: BTreeMap::new(),
            },
            Control {
                name: "size".into(),
                kind: ControlKind::Select {
                    options: vec!["sm".into(), "lg".into()],
                },
                current: Some(CsfValue::Str("lg".into())),
                labels: BTreeMap::new(),
                mapping: BTreeMap::new(),
            },
        ];

        let html = render_manager_html(&index, None, &controls);
        // The panel renders a widget per control.
        assert!(
            html.contains("id=\"jet-controls\""),
            "controls panel present"
        );
        assert!(html.contains("data-control=\"primary\""), "toggle wired");
        assert!(html.contains("data-control=\"label\""), "text wired");
        assert!(html.contains("data-control=\"size\""), "select wired");
        // Current values seed the widgets.
        assert!(
            html.contains("data-kind=\"toggle\" checked"),
            "toggle seeded true"
        );
        assert!(
            html.contains("value=\"Click\""),
            "text seeded with current value"
        );
        assert!(
            html.contains("<option value=\"lg\" selected>"),
            "select seeds current option"
        );
        // The seed args object carries the current values.
        assert!(html.contains("\"label\":\"Click\""), "jetArgs seeded");
        assert!(
            html.contains("const jetStoryArgs = new Map()"),
            "args are keyed per story"
        );
        assert!(
            html.contains("fetch('/__jet_stories_controls/'"),
            "story switches fetch fresh controls"
        );
        assert!(
            html.contains("jetLoadControls(storyId)"),
            "sidebar selection refreshes controls"
        );
        // Editing posts new args to the preview render hook.
        assert!(
            html.contains("postMessage"),
            "controls post args to preview"
        );
        assert!(
            html.contains("jet-stories-args"),
            "uses the args-update message"
        );
    }

    #[test]
    fn controls_panel_renders_full_types_mapping_and_url_sync() {
        use crate::stories::controls::{Control, ControlKind};
        use crate::stories::csf::CsfValue;

        let mut index = StoryIndex::default();
        index.stories.push(entry("x--full", "Full", &["X"]));

        let mut labels = BTreeMap::new();
        labels.insert("sm".into(), "Small".into());
        let mut mapping = BTreeMap::new();
        mapping.insert("sm".into(), CsfValue::Number("1".into()));

        let controls = vec![
            Control {
                name: "color".into(),
                kind: ControlKind::Color,
                current: Some(CsfValue::Str("#ff0000".into())),
                labels: BTreeMap::new(),
                mapping: BTreeMap::new(),
            },
            Control {
                name: "range".into(),
                kind: ControlKind::Range {
                    min: Some("0".into()),
                    max: Some("10".into()),
                    step: Some("1".into()),
                },
                current: Some(CsfValue::Number("4".into())),
                labels: BTreeMap::new(),
                mapping: BTreeMap::new(),
            },
            Control {
                name: "object".into(),
                kind: ControlKind::Object,
                current: Some(CsfValue::Object(BTreeMap::new())),
                labels: BTreeMap::new(),
                mapping: BTreeMap::new(),
            },
            Control {
                name: "size".into(),
                kind: ControlKind::Radio {
                    options: vec!["sm".into()],
                    inline: true,
                },
                current: Some(CsfValue::Str("sm".into())),
                labels,
                mapping,
            },
            Control {
                name: "files".into(),
                kind: ControlKind::File,
                current: None,
                labels: BTreeMap::new(),
                mapping: BTreeMap::new(),
            },
        ];

        let html = render_manager_html(&index, None, &controls);
        assert!(html.contains("type=\"color\"") && html.contains("data-kind=\"color\""));
        assert!(
            html.contains("type=\"range\"")
                && html.contains("min=\"0\"")
                && html.contains("max=\"10\"")
                && html.contains("step=\"1\"")
        );
        assert!(html.contains("<textarea") && html.contains("data-kind=\"object\""));
        assert!(
            html.contains("data-kind=\"radio\"")
                && html.contains("jet-choice-inline")
                && html.contains("Small")
        );
        assert!(
            html.contains("data-mapping=\"{&quot;sm&quot;:1}\""),
            "mapping is embedded as JSON data: {html}"
        );
        assert!(html.contains("type=\"file\"") && html.contains("data-kind=\"file\""));
        assert!(html.contains("jetHydrateArgsFromUrl"));
        assert!(html.contains("params.set('args'"));
        assert!(html.contains("jetMappedValue"));
    }

    #[test]
    fn preview_applies_args_update_message() {
        let e = entry(
            "components-button--primary",
            "Primary",
            &["Components", "Button"],
        );
        let html = render_preview_html(&e, "/src/Button.stories.tsx");
        // The preview listens for the manager's args message and re-renders via
        // the exposed render hook.
        assert!(html.contains("window.__jetStoriesRender = renderStory"));
        assert!(
            html.contains("-apple-system") && html.contains("\"Segoe UI\""),
            "preview base CSS should match Storybook iframe's system font stack"
        );
        assert!(
            html.contains("line-height: 1.5714285714285714"),
            "preview base CSS should match Storybook/AntD default line height"
        );
        assert!(
            html.contains("jet-stories-args"),
            "listens for control updates"
        );
        assert!(
            html.contains("liveArgs = data.args"),
            "swaps live args on update"
        );
        assert!(
            html.contains("const discoveredArgs ="),
            "keeps parsed args as a fallback"
        );
        assert!(
            html.contains(": { ...discoveredArgs, ...(meta.args || {}), ...authoredArgs }"),
            "authored module args should override parsed fallback on initial render"
        );
        assert!(
            html.contains("globalDefaults(ProjectPreview.globalTypes, meta.globalTypes)"),
            "project/meta globalTypes default values feed globals"
        );
        assert!(
            html.contains("const loaders = [") && html.contains("await runLoaders"),
            "project/meta/story loaders are awaited before render"
        );
        assert!(
            html.contains("const decorators = [")
                && html.contains("...normalizeList(story && story.decorators)")
                && html.contains("...normalizeList(ProjectPreview.decorators)"),
            "story/meta/project decorators are composed"
        );
        assert!(
            html.contains("applyLayout(parameters.layout)"),
            "parameters.layout drives preview presentation"
        );
        assert!(
            html.contains("window.__jetStoriesHighlight = applyHighlight")
                && html.contains("type !== \"jet-canvas-tools\"")
                && html.contains("jet-measure-overlay")
                && html.contains("jet-outline-style")
                && html.contains("jet-highlight-layer"),
            "preview ships inactive canvas measure/outline/highlight runtime"
        );
        assert!(
            html.contains("@storybook/addon-actions")
                && html.contains("window.__jetStoryActionShim")
                && html.contains("function safeActionArgs")
                && html.contains("function injectActionArgs")
                && html.contains("type: \"jet-action\""),
            "preview ships action() shim and auto action logging"
        );
        assert!(
            html.contains("@storybook/test")
                && html.contains("function runPlay")
                && html.contains("story.play")
                && html.contains("postInteraction(\"start\"")
                && html.contains("postInteraction(\"pass\"")
                && html.contains("postInteraction(\"fail\""),
            "preview ships @storybook/test shim and play step timeline runtime"
        );
        assert!(
            html.contains("\"axe-core\": \"https://esm.sh/axe-core@4.10.3\"")
                && html.contains("import axe from \"axe-core\"")
                && html.contains("function runA11y")
                && html.contains("normalizeA11yParameters")
                && html.contains("parameters.a11y")
                && html.contains("type: \"jet-a11y-result\""),
            "preview ships pinned axe-core importmap and per-story a11y audit runtime"
        );
    }

    #[test]
    fn no_controls_renders_placeholder() {
        let mut index = StoryIndex::default();
        index.stories.push(entry("x--y", "Y", &["X"]));
        let html = render_manager_html(&index, None, &[]);
        assert!(html.contains("No controls for this story."));
    }

    #[test]
    fn dev_mode_is_the_default_and_emits_absolute_routes() {
        let mut index = StoryIndex::default();
        index.stories.push(entry(
            "components-button--primary",
            "Primary",
            &["Components", "Button"],
        ));

        // The default wrapper and the explicit Dev mode must be byte-identical
        // — no absolute→relative regression for the dev server.
        let default = render_manager_html(&index, None, &[]);
        let dev = render_manager_html_with_mode(&index, None, &[], UrlMode::Dev);
        assert_eq!(default, dev);
        assert!(default.contains("src=\"/__jet_stories_preview/components-button--primary\""));
        // The preview likewise defaults to Dev (absolute module URL + HMR client).
        let e = entry(
            "components-button--primary",
            "Primary",
            &["Components", "Button"],
        );
        let p_default = render_preview_html(&e, "/src/Button.stories.tsx");
        let p_dev = render_preview_html_with_mode(&e, "/src/Button.stories.tsx", UrlMode::Dev);
        assert_eq!(p_default, p_dev);
        assert!(
            p_default.contains("HMR connected"),
            "dev preview ships the HMR client"
        );
    }

    #[test]
    fn static_mode_emits_relative_preview_links() {
        let mut index = StoryIndex::default();
        index.stories.push(entry(
            "components-button--primary",
            "Primary",
            &["Components", "Button"],
        ));

        let html = render_manager_html_with_mode(&index, None, &[], UrlMode::Static);
        // iframe + sidebar link the relative preview file, never an absolute route.
        assert!(html.contains("src=\"preview/components-button--primary.html\""));
        assert!(html.contains("data-preview=\"preview/components-button--primary.html\""));
        assert!(
            !html.contains("/__jet_stories_preview"),
            "no dev routes in static mode"
        );
    }

    #[test]
    fn static_mode_preview_imports_relative_and_drops_hmr() {
        let e = entry(
            "components-button--primary",
            "Primary",
            &["Components", "Button"],
        );
        let html = render_preview_html_with_mode(
            &e,
            "../modules/src/components/Button.stories.js",
            UrlMode::Static,
        );
        assert!(
            html.contains("import * as Story from \"../modules/src/components/Button.stories.js\"")
        );
        // No HMR client / WebSocket wiring in the server-less static export.
        assert!(
            !html.contains("HMR connected"),
            "static preview omits the HMR client"
        );
        assert!(
            !html.contains("WebSocket"),
            "no WebSocket in static preview"
        );
        // Still an isolated single-root mount.
        assert_eq!(html.matches("id=\"jet-root\"").count(), 1);
    }
}
// </HANDWRITE>
