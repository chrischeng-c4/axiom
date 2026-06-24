// HANDWRITE-BEGIN gap="missing-generator:logic:583d99f9" tracker="pending-tracker" reason="Manager UI: render the manager HTML shell (sidebar tree from StoryIndex, toolbar, preview iframe) and the isolated per-story preview HTML entry (mounts only the selected story component, no app router/shell)."
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
    render_manager_html_with_mode(index, selected, controls, UrlMode::Dev)
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

    let sidebar = render_sidebar(index, initial_id, mode);
    let diagnostics = render_diagnostics(index);
    let controls_panel = render_controls_panel(controls);
    let initial_args_json = controls_to_args_json(controls);
    let story_count = index.stories.len();

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>jet stories</title>
<style>
  * {{ box-sizing: border-box; }}
  html, body {{ margin: 0; height: 100%; }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    display: grid; grid-template-columns: 260px 1fr; grid-template-rows: 44px 1fr 220px;
    grid-template-areas: "sidebar toolbar" "sidebar preview" "sidebar controls";
    height: 100vh; color: #1a1a1a;
  }}
  #jet-sidebar {{
    grid-area: sidebar; border-right: 1px solid #e3e3e3; overflow-y: auto;
    background: #fafafa; padding: 8px 0;
  }}
  #jet-sidebar .jet-brand {{ font-weight: 600; padding: 8px 14px; font-size: 14px; }}
  #jet-sidebar ul {{ list-style: none; margin: 0; padding: 0; }}
  #jet-sidebar .jet-group > span {{
    display: block; padding: 4px 14px; font-size: 12px; font-weight: 600;
    color: #555;
  }}
  #jet-sidebar a.jet-story {{
    display: block; padding: 4px 14px 4px 28px; font-size: 13px; color: #333;
    text-decoration: none; cursor: pointer;
  }}
  #jet-sidebar a.jet-story:hover {{ background: #eef; }}
  #jet-sidebar a.jet-story.jet-active {{ background: #4338ca; color: #fff; }}
  #jet-toolbar {{
    grid-area: toolbar; border-bottom: 1px solid #e3e3e3; display: flex;
    align-items: center; gap: 12px; padding: 0 14px; background: #fff; font-size: 13px;
  }}
  #jet-preview {{ grid-area: preview; border: 0; width: 100%; height: 100%; }}
  .jet-diag {{ color: #b00; font-size: 12px; padding: 6px 14px; }}
  #jet-controls {{
    grid-area: controls; border-top: 1px solid #e3e3e3; background: #fff;
    overflow-y: auto; padding: 8px 14px; font-size: 13px;
  }}
  #jet-controls h3 {{ margin: 0 0 8px; font-size: 12px; color: #555; font-weight: 600; }}
  #jet-controls table {{ border-collapse: collapse; width: 100%; }}
  #jet-controls td {{ padding: 4px 8px 4px 0; vertical-align: middle; }}
  #jet-controls td.jet-control-name {{ font-weight: 500; width: 140px; color: #333; }}
  #jet-controls input[type="text"], #jet-controls input[type="number"],
  #jet-controls select {{
    width: 100%; max-width: 260px; padding: 3px 6px; font-size: 13px;
    border: 1px solid #ccc; border-radius: 4px;
  }}
  #jet-controls .jet-no-controls {{ color: #999; }}
</style>
</head>
<body>
<nav id="jet-sidebar" aria-label="Stories">
  <div class="jet-brand">jet stories</div>
  {sidebar}
  {diagnostics}
</nav>
<header id="jet-toolbar">
  <span id="jet-current-title">{initial_title}</span>
  <span style="margin-left:auto;color:#777">{story_count} stories</span>
</header>
<iframe id="jet-preview" name="jet-preview" src="{initial_src}"></iframe>
<section id="jet-controls" aria-label="Controls">
  <h3>Controls</h3>
  {controls_panel}
</section>
<script>
  // Full-reload navigation: clicking a story swaps the preview iframe src.
  // HMR is deliberately out of scope here (B2b / #176) — a reload is fine.
  const frame = document.getElementById('jet-preview');
  const titleEl = document.getElementById('jet-current-title');
  document.querySelectorAll('a.jet-story').forEach((a) => {{
    a.addEventListener('click', (ev) => {{
      ev.preventDefault();
      document.querySelectorAll('a.jet-story').forEach((x) => x.classList.remove('jet-active'));
      a.classList.add('jet-active');
      frame.setAttribute('src', a.getAttribute('data-preview'));
      titleEl.textContent = a.getAttribute('data-title');
      history.replaceState(null, '', '?story=' + encodeURIComponent(a.getAttribute('data-story-id')));
    }});
  }});

  // ─── Controls panel (B3) ─────────────────────────────────────────────────
  // The live args object, seeded with the selected story's current values.
  // Editing any control mutates this object and posts the full args set into
  // the preview iframe, which re-renders via window.__jetStoriesRender.
  const jetArgs = {initial_args_json};

  // Coerce a control's DOM value to the arg type the control declares.
  function jetControlValue(el) {{
    if (el.dataset.kind === 'toggle') return el.checked;
    if (el.dataset.kind === 'number') {{
      const n = el.value.trim();
      if (n === '') return undefined;
      const f = Number(n);
      return Number.isNaN(f) ? el.value : f;
    }}
    return el.value;
  }}

  // Post the current args into the preview frame so it re-renders. The preview
  // client applies them through window.__jetStoriesRender (see render_preview_html).
  function jetPushArgs() {{
    const win = frame.contentWindow;
    if (!win) return;
    win.postMessage({{ type: 'jet-stories-args', args: jetArgs }}, '*');
  }}

  document.querySelectorAll('[data-control]').forEach((el) => {{
    const name = el.dataset.control;
    el.addEventListener('input', () => {{
      const v = jetControlValue(el);
      if (v === undefined) {{ delete jetArgs[name]; }} else {{ jetArgs[name] = v; }}
      jetPushArgs();
    }});
    el.addEventListener('change', () => {{
      const v = jetControlValue(el);
      if (v === undefined) {{ delete jetArgs[name]; }} else {{ jetArgs[name] = v; }}
      jetPushArgs();
    }});
  }});
</script>
</body>
</html>
"#,
        sidebar = sidebar,
        diagnostics = diagnostics,
        controls_panel = controls_panel,
        initial_args_json = initial_args_json,
        initial_src = escape_html(&initial_src),
        initial_title = escape_html(
            selected_entry
                .map(story_display_title)
                .unwrap_or_else(|| "No stories".to_string())
                .as_str()
        ),
        story_count = story_count,
    )
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
    match &control.kind {
        ControlKind::Toggle => {
            let checked = matches!(control.current, Some(CsfValue::Bool(true)));
            format!(
                "<input type=\"checkbox\" data-control=\"{name}\" data-kind=\"toggle\"{checked} />",
                name = name,
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
                "<input type=\"number\" data-control=\"{name}\" data-kind=\"number\" value=\"{value}\" />",
                name = name,
                value = escape_html(&value),
            )
        }
        ControlKind::Select { options } => {
            let current = control.current.as_ref().map(current_value_string);
            let mut opts = String::new();
            for opt in options {
                let selected = current.as_deref() == Some(opt.as_str());
                opts.push_str(&format!(
                    "<option value=\"{v}\"{sel}>{label}</option>",
                    v = escape_html(opt),
                    sel = if selected { " selected" } else { "" },
                    label = escape_html(opt),
                ));
            }
            format!(
                "<select data-control=\"{name}\" data-kind=\"select\">{opts}</select>",
                name = name,
                opts = opts,
            )
        }
        ControlKind::Text => {
            let value = control
                .current
                .as_ref()
                .map(current_value_string)
                .unwrap_or_default();
            format!(
                "<input type=\"text\" data-control=\"{name}\" data-kind=\"text\" value=\"{value}\" />",
                name = name,
                value = escape_html(&value),
            )
        }
    }
}

/// Render a [`CsfValue`] as a plain string for seeding an input's `value` or
/// matching a `<select>` option.
fn current_value_string(value: &CsfValue) -> String {
    match value {
        CsfValue::Str(s) => s.clone(),
        CsfValue::Bool(b) => b.to_string(),
        CsfValue::Number(n) => n.clone(),
        CsfValue::Null => String::new(),
        CsfValue::Object(_) | CsfValue::Raw(_) => String::new(),
    }
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
fn render_sidebar(index: &StoryIndex, active_id: &str, mode: UrlMode) -> String {
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
///   2. dynamically imports the story module,
///   3. picks the story's named export and renders it — honoring a custom
///      `render` function when the story declares one, otherwise mounting the
///      meta `component` (or the export value treated as a component),
///   4. mounts the result into a single `#jet-root` div with no surrounding app
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
    let args_json = args_to_json(&story.args);
    // B2b/#176: the HMR client lives ONLY in the preview frame, so an edit
    // hot-updates the iframe while the manager shell stays put. The static
    // export has no server to talk to, so it ships no HMR client.
    let hmr_client = match mode {
        UrlMode::Dev => render_preview_hmr_client(),
        UrlMode::Static => String::new(),
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
    "react/jsx-runtime": "https://esm.sh/react@18/jsx-runtime"
  }}
}}
</script>
<style> html, body {{ margin: 0; }} #jet-root {{ padding: 16px; }} </style>
</head>
<body>
<div id="jet-root" data-story-id="{story_id}"></div>
<script type="module">
  // Isolated mount: only this story renders here — no app router/shell.
  import * as Story from "{module_url}";
  import React from "react";
  import {{ createRoot }} from "react-dom/client";

  const exportName = "{export_name}";
  // `liveArgs` start as the story's authored args and are replaced wholesale
  // when the manager's Controls panel posts a `jet-stories-args` message (B3).
  let liveArgs = {args_json};
  let lastModule = Story;
  const root = createRoot(document.getElementById("jet-root"));
  // Exposed so the HMR client (loaded after this module) can re-render the
  // story in place with a freshly re-imported module — state-preserving for
  // react-refresh-compatible edits, isolated to this frame.
  window.__jetStoriesRender = renderStory;

  // B3: apply control edits from the manager. The manager posts the full args
  // object; we replace liveArgs and re-render the most recent module in place.
  window.addEventListener("message", (ev) => {{
    const data = ev && ev.data;
    if (!data || data.type !== "jet-stories-args" || !data.args) return;
    liveArgs = data.args;
    renderStory(lastModule);
  }});

  function pickComponent(mod) {{
    const story = mod[exportName];
    // A story may BE a component (function/class) or a CSF object with a
    // `render`/`component` field. Resolve to a renderable React element factory.
    if (typeof story === "function") return (props) => React.createElement(story, props);
    if (story && typeof story.render === "function") return (props) => story.render(props);
    if (story && story.component) return (props) => React.createElement(story.component, props);
    const meta = mod.default;
    if (meta && meta.component) return (props) => React.createElement(meta.component, props);
    return null;
  }}

  function renderStory(mod) {{
    lastModule = mod;
    try {{
      const factory = pickComponent(mod);
      if (!factory) {{
        document.getElementById("jet-root").textContent =
          "jet stories: could not resolve a component for export '" + exportName + "'";
      }} else {{
        const merged = {{ ...(mod[exportName] && mod[exportName].args), ...liveArgs }};
        root.render(factory(merged));
      }}
    }} catch (err) {{
      document.getElementById("jet-root").textContent = "jet stories render error: " + (err && err.message || err);
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
        story_id = escape_html(&story.id),
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
/// TODO(#176 follow-up): wire the dev server's full react-refresh runtime
/// (`/@react-refresh` + `$RefreshReg$` instrumentation, see
/// [`crate::dev_server::react_refresh`]) so hook state survives a component edit
/// without a re-mount. Today an `update` re-imports + re-renders the story
/// (fast, frame-local, no manager reload) but `createRoot().render` still
/// re-mounts the component subtree, so component-local hook state is reset.
fn render_preview_hmr_client() -> String {
    format!(
        r#"<script type="module">
  // ─── jet stories preview HMR client (B2b/#176) ───────────────────────────
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
        // Cache-bust so the browser fetches the freshly transformed module.
        const bust = (msg.path.indexOf("?") === -1 ? "?" : "&") + "t=" + msg.timestamp;
        const fresh = await import(msg.path + bust);
        render(fresh);
        console.log("[jet stories] hot updated", msg.path);
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
    use super::csf::CsfValue;

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
            has_render: false,
            file: PathBuf::from("/x/Foo.stories.tsx"),
            title_path: title.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn manager_lists_each_story_and_points_iframe_at_first() {
        let mut index = StoryIndex::default();
        index
            .stories
            .push(entry("components-button--primary", "Primary", &["Components", "Button"]));
        index
            .stories
            .push(entry("components-button--disabled", "Disabled", &["Components", "Button"]));

        let html = render_manager_html(&index, None, &[]);
        assert!(html.contains("Primary"), "lists Primary");
        assert!(html.contains("Disabled"), "lists Disabled");
        assert!(html.contains("Components / Button"), "shows the group title");
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
        index
            .stories
            .push(entry("components-button--primary", "Primary", &["Components", "Button"]));
        index
            .stories
            .push(entry("components-button--disabled", "Disabled", &["Components", "Button"]));

        let html = render_manager_html(&index, Some("components-button--primary"), &[]);
        assert!(html.contains("src=\"/__jet_stories_preview/components-button--primary\""));
    }

    #[test]
    fn preview_references_module_and_export() {
        let e = entry("components-button--primary", "Primary", &["Components", "Button"]);
        let html = render_preview_html(&e, "/src/components/Button.stories.tsx");
        assert!(html.contains("import * as Story from \"/src/components/Button.stories.tsx\""));
        assert!(html.contains("const exportName = \"Primary\""));
        assert!(html.contains("id=\"jet-root\""), "mounts into isolated root div");
        // No app shell / router markers — just the single root.
        assert_eq!(html.matches("id=\"jet-root\"").count(), 1);
    }

    #[test]
    fn args_serialize_to_json() {
        let mut args = BTreeMap::new();
        args.insert("label".to_string(), super::super::csf::CsfValue::Str("Hi".into()));
        args.insert("primary".to_string(), super::super::csf::CsfValue::Bool(true));
        args.insert("count".to_string(), super::super::csf::CsfValue::Number("3".into()));
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
        index
            .stories
            .push(entry("components-button--primary", "Primary", &["Components", "Button"]));

        let controls = vec![
            Control {
                name: "primary".into(),
                kind: ControlKind::Toggle,
                current: Some(CsfValue::Bool(true)),
            },
            Control {
                name: "label".into(),
                kind: ControlKind::Text,
                current: Some(CsfValue::Str("Click".into())),
            },
            Control {
                name: "size".into(),
                kind: ControlKind::Select {
                    options: vec!["sm".into(), "lg".into()],
                },
                current: Some(CsfValue::Str("lg".into())),
            },
        ];

        let html = render_manager_html(&index, None, &controls);
        // The panel renders a widget per control.
        assert!(html.contains("id=\"jet-controls\""), "controls panel present");
        assert!(html.contains("data-control=\"primary\""), "toggle wired");
        assert!(html.contains("data-control=\"label\""), "text wired");
        assert!(html.contains("data-control=\"size\""), "select wired");
        // Current values seed the widgets.
        assert!(html.contains("data-kind=\"toggle\" checked"), "toggle seeded true");
        assert!(html.contains("value=\"Click\""), "text seeded with current value");
        assert!(html.contains("<option value=\"lg\" selected>"), "select seeds current option");
        // The seed args object carries the current values.
        assert!(html.contains("\"label\":\"Click\""), "jetArgs seeded");
        // Editing posts new args to the preview render hook.
        assert!(html.contains("postMessage"), "controls post args to preview");
        assert!(html.contains("jet-stories-args"), "uses the args-update message");
    }

    #[test]
    fn preview_applies_args_update_message() {
        let e = entry("components-button--primary", "Primary", &["Components", "Button"]);
        let html = render_preview_html(&e, "/src/Button.stories.tsx");
        // The preview listens for the manager's args message and re-renders via
        // the exposed render hook.
        assert!(html.contains("window.__jetStoriesRender = renderStory"));
        assert!(html.contains("jet-stories-args"), "listens for control updates");
        assert!(html.contains("liveArgs = data.args"), "swaps live args on update");
    }

    #[test]
    fn no_controls_renders_placeholder() {
        let mut index = StoryIndex::default();
        index
            .stories
            .push(entry("x--y", "Y", &["X"]));
        let html = render_manager_html(&index, None, &[]);
        assert!(html.contains("No controls for this story."));
    }

    #[test]
    fn dev_mode_is_the_default_and_emits_absolute_routes() {
        let mut index = StoryIndex::default();
        index
            .stories
            .push(entry("components-button--primary", "Primary", &["Components", "Button"]));

        // The default wrapper and the explicit Dev mode must be byte-identical
        // — no absolute→relative regression for the dev server.
        let default = render_manager_html(&index, None, &[]);
        let dev = render_manager_html_with_mode(&index, None, &[], UrlMode::Dev);
        assert_eq!(default, dev);
        assert!(default.contains("src=\"/__jet_stories_preview/components-button--primary\""));
        // The preview likewise defaults to Dev (absolute module URL + HMR client).
        let e = entry("components-button--primary", "Primary", &["Components", "Button"]);
        let p_default = render_preview_html(&e, "/src/Button.stories.tsx");
        let p_dev = render_preview_html_with_mode(&e, "/src/Button.stories.tsx", UrlMode::Dev);
        assert_eq!(p_default, p_dev);
        assert!(p_default.contains("HMR connected"), "dev preview ships the HMR client");
    }

    #[test]
    fn static_mode_emits_relative_preview_links() {
        let mut index = StoryIndex::default();
        index
            .stories
            .push(entry("components-button--primary", "Primary", &["Components", "Button"]));

        let html = render_manager_html_with_mode(&index, None, &[], UrlMode::Static);
        // iframe + sidebar link the relative preview file, never an absolute route.
        assert!(html.contains("src=\"preview/components-button--primary.html\""));
        assert!(html.contains("data-preview=\"preview/components-button--primary.html\""));
        assert!(!html.contains("/__jet_stories_preview"), "no dev routes in static mode");
    }

    #[test]
    fn static_mode_preview_imports_relative_and_drops_hmr() {
        let e = entry("components-button--primary", "Primary", &["Components", "Button"]);
        let html = render_preview_html_with_mode(
            &e,
            "../modules/src/components/Button.stories.js",
            UrlMode::Static,
        );
        assert!(html.contains("import * as Story from \"../modules/src/components/Button.stories.js\""));
        // No HMR client / WebSocket wiring in the server-less static export.
        assert!(!html.contains("HMR connected"), "static preview omits the HMR client");
        assert!(!html.contains("WebSocket"), "no WebSocket in static preview");
        // Still an isolated single-root mount.
        assert_eq!(html.matches("id=\"jet-root\"").count(), 1);
    }
}
// HANDWRITE-END
