// <HANDWRITE gap="codegen:browser-semantic-surface" tracker="jet-bb-semantic-surface" reason="Ref-model snapshot JS, element-targeted action JS, and page-instrumentation init script have no deterministic generator primitive yet; feed back into Agentic Workflow until it can become CODEGEN.">
//! Semantic Browser Bridge verbs: snapshot/refs, element-targeted
//! actions, navigation, and page observability.
//!
//! This is the playwright-mcp/playwright-cli-shaped layer of `jet bb`:
//! `snapshot` walks the live DOM, assigns stable refs (`e1`, `e2`, …)
//! to interactable elements (kept on `window.__jet_bb_refs`), and the
//! action verbs (`click`, `fill`, `type`, `hover`, `select`,
//! `check`/`uncheck`) accept either one of those refs or a locator
//! selector (CSS, `text=…`, `role=…[name="…"]`). Navigation
//! (`goto`/`back`/`forward`/`reload`/`resize`/`wait`) and observability
//! (`console`/`requests`, fed by an init script installed at launch)
//! round out the surface.
//!
//! Every public verb returns a `serde_json::Value` so the CLI dispatch
//! and the MCP server share one implementation: the CLI prints the
//! value, MCP wraps it in a text content block.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::Path;
use std::time::{Duration, Instant};

use super::attach;
use crate::browser::{Actionability, Page};

/// Init script installed at `bb launch` time (before app code runs on
/// every new document). Buffers console output, uncaught errors, and
/// fetch/XHR activity into in-page rings so the re-dial-per-command
/// session model can read history without holding a CDP event stream.
pub const BB_OBSERVE_INIT_JS: &str = r#"(() => {
  if (window.__jet_bb_observe_installed) return;
  window.__jet_bb_observe_installed = true;
  const CAP = 1000;
  const consoleBuf = [];
  const requestsBuf = [];
  window.__jet_bb_console = consoleBuf;
  window.__jet_bb_requests = requestsBuf;
  const push = (buf, entry) => {
    buf.push(entry);
    if (buf.length > CAP) buf.splice(0, buf.length - CAP);
  };
  const fmt = (a) => {
    try {
      if (typeof a === 'string') return a;
      if (a instanceof Error) return a.stack || String(a);
      return JSON.stringify(a);
    } catch (_) {
      return String(a);
    }
  };
  for (const level of ['log', 'info', 'warn', 'error', 'debug']) {
    const orig = console[level];
    console[level] = function (...args) {
      push(consoleBuf, {
        level,
        text: args.map(fmt).join(' ').slice(0, 2000),
        ts: Date.now(),
      });
      return orig.apply(this, args);
    };
  }
  window.addEventListener('error', (e) => {
    push(consoleBuf, {
      level: 'error',
      text: String(e.message) + ' @ ' + (e.filename || '') + ':' + (e.lineno || 0),
      ts: Date.now(),
    });
  });
  window.addEventListener('unhandledrejection', (e) => {
    push(consoleBuf, {
      level: 'error',
      text: 'Unhandled rejection: ' + fmt(e.reason).slice(0, 2000),
      ts: Date.now(),
    });
  });
  const origFetch = window.fetch;
  if (origFetch) {
    window.fetch = function (input, init) {
      const url = typeof input === 'string' ? input : (input && input.url) || String(input);
      const method = ((init && init.method) || (input && input.method) || 'GET').toUpperCase();
      const started = Date.now();
      const entry = {
        api: 'fetch',
        method,
        url: String(url).slice(0, 500),
        status: null,
        ok: null,
        duration_ms: null,
        ts: started,
      };
      push(requestsBuf, entry);
      return origFetch.apply(this, arguments).then(
        (resp) => {
          entry.status = resp.status;
          entry.ok = resp.ok;
          entry.duration_ms = Date.now() - started;
          return resp;
        },
        (err) => {
          entry.status = -1;
          entry.ok = false;
          entry.error = String(err).slice(0, 200);
          entry.duration_ms = Date.now() - started;
          throw err;
        }
      );
    };
  }
  const proto = window.XMLHttpRequest && window.XMLHttpRequest.prototype;
  if (proto && proto.open && proto.send) {
    const origOpen = proto.open;
    const origSend = proto.send;
    proto.open = function (method, url) {
      this.__jet_bb_entry = {
        api: 'xhr',
        method: String(method).toUpperCase(),
        url: String(url).slice(0, 500),
      };
      return origOpen.apply(this, arguments);
    };
    proto.send = function () {
      const entry = this.__jet_bb_entry || { api: 'xhr', method: '?', url: '?' };
      entry.ts = Date.now();
      entry.status = null;
      entry.ok = null;
      entry.duration_ms = null;
      push(requestsBuf, entry);
      this.addEventListener('loadend', () => {
        entry.status = this.status;
        entry.ok = this.status >= 200 && this.status < 400;
        entry.duration_ms = Date.now() - entry.ts;
      });
      return origSend.apply(this, arguments);
    };
  }
})();"#;

// ── Targets: snapshot refs vs locator selectors ─────────────────────────────

/// An element target for an action verb: a snapshot ref (`e12` /
/// `ref=e12`) or a locator selector (CSS, `text=…`, `role=…`).
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Ref(String),
    Selector(String),
}

/// Parse a CLI/MCP target string. `ref=e12` is always a ref; a bare
/// `e12` (the exact shape `snapshot` mints) is treated as a ref too,
/// since no real-world CSS selector is a bare `e<digits>` tag.
pub fn parse_target(raw: &str) -> Result<Target> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("empty target — pass a snapshot ref (e.g. e12) or a selector");
    }
    if let Some(r) = trimmed.strip_prefix("ref=") {
        let r = r.trim();
        if !is_ref_shaped(r) {
            bail!("invalid ref {r:?} — snapshot refs look like e12");
        }
        return Ok(Target::Ref(r.to_string()));
    }
    if is_ref_shaped(trimmed) {
        return Ok(Target::Ref(trimmed.to_string()));
    }
    Ok(Target::Selector(trimmed.to_string()))
}

fn is_ref_shaped(s: &str) -> bool {
    let Some(digits) = s.strip_prefix('e') else {
        return false;
    };
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

fn json_str(s: &str) -> String {
    serde_json::to_string(s).expect("string serialization is infallible")
}

/// JS expression resolving a target to an element, with stale-ref
/// detection on the ref path. Selector targets resolve through the
/// Locator engine so `text=` / `role=` work everywhere a ref does.
async fn element_expr(page: &Page, target: &Target) -> Result<String> {
    match target {
        Target::Ref(r) => Ok(format!(
            "(() => {{ const el = (window.__jet_bb_refs || {{}})[{r}]; \
             if (!el) throw new Error('unknown ref ' + {r} + ' — run `jet bb snapshot` first'); \
             if (!el.isConnected) throw new Error('stale ref ' + {r} + ' (element left the DOM) — re-run `jet bb snapshot`'); \
             return el; }})()",
            r = json_str(r)
        )),
        Target::Selector(sel) => {
            let locator = page
                .locator(sel)
                .map_err(|e| anyhow::anyhow!("invalid selector {sel:?}: {e}"))?;
            locator
                .wait_for(Actionability::Actionable)
                .await
                .map_err(|e| anyhow::anyhow!("waiting for {sel:?}: {e}"))?;
            Ok(format!(
                "(() => {{ const el = {expr}; \
                 if (!el) throw new Error('no element matches selector ' + {sel}); \
                 return el; }})()",
                expr = locator.single_element_expr(),
                sel = json_str(sel)
            ))
        }
    }
}

/// Wrap an action body around a resolved element expression. The body
/// sees the element as `el` and must return a JSON-serializable value.
fn action_js(el_expr: &str, body: &str) -> String {
    format!(
        "(() => {{ const el = {el_expr}; \
         el.scrollIntoView({{ block: 'center', inline: 'center' }}); \
         {body} }})()"
    )
}

// ── Snapshot ─────────────────────────────────────────────────────────────────

/// One-sweep DOM walk: emit an indented, role-annotated line per
/// visible interactable/structural element, mint `e<N>` refs, and park
/// the ref → element map on `window.__jet_bb_refs` for action verbs.
/// Kept as a single Runtime.evaluate round-trip on purpose.
const SNAPSHOT_JS: &str = r#"(() => {
  const refs = {};
  let n = 0;
  const lines = [];
  const MAX_ELEMENTS = 600;
  let truncated = false;
  const SKIP_TAGS = new Set(['SCRIPT', 'STYLE', 'TEMPLATE', 'NOSCRIPT', 'META', 'LINK', 'HEAD']);
  const INTERACTIVE_TAGS = new Set(['a', 'button', 'input', 'select', 'textarea', 'summary', 'option', 'label']);
  const INTERACTIVE_ROLES = new Set(['button', 'link', 'checkbox', 'radio', 'combobox', 'textbox', 'searchbox', 'tab', 'menuitem', 'menuitemcheckbox', 'menuitemradio', 'option', 'slider', 'switch', 'spinbutton']);
  const TEXT_TAGS = new Set(['P', 'LI', 'TD', 'TH', 'DT', 'DD', 'BLOCKQUOTE', 'PRE', 'FIGCAPTION', 'LEGEND']);
  const cap = (s, len) => {
    s = (s || '').replace(/\s+/g, ' ').trim();
    return s.length > len ? s.slice(0, len) + '…' : s;
  };
  const implicitRole = (el) => {
    const tag = el.tagName.toLowerCase();
    switch (tag) {
      case 'a': return el.hasAttribute('href') ? 'link' : '';
      case 'button': return 'button';
      case 'select': return 'combobox';
      case 'textarea': return 'textbox';
      case 'img': return 'img';
      case 'nav': return 'navigation';
      case 'main': return 'main';
      case 'form': return 'form';
      case 'ul': case 'ol': return 'list';
      case 'table': return 'table';
      case 'input': {
        const t = (el.getAttribute('type') || 'text').toLowerCase();
        if (t === 'checkbox') return 'checkbox';
        if (t === 'radio') return 'radio';
        if (t === 'range') return 'slider';
        if (t === 'button' || t === 'submit' || t === 'reset') return 'button';
        if (t === 'hidden') return '';
        if (t === 'search') return 'searchbox';
        return 'textbox';
      }
      default:
        return /^H[1-6]$/.test(el.tagName) ? 'heading' : '';
    }
  };
  const accName = (el) => {
    const aria = el.getAttribute('aria-label');
    if (aria) return cap(aria, 80);
    const labelled = el.getAttribute('aria-labelledby');
    if (labelled) {
      const parts = labelled.split(/\s+/)
        .map((id) => { const t = document.getElementById(id); return t ? t.textContent : ''; })
        .join(' ');
      if (parts.trim()) return cap(parts, 80);
    }
    if (el.labels && el.labels.length) return cap(el.labels[0].textContent, 80);
    return cap(el.getAttribute('alt') || el.getAttribute('placeholder') || el.getAttribute('title') || el.textContent, 80);
  };
  const visible = (el) => {
    if (el.getAttribute('aria-hidden') === 'true') return false;
    const r = el.getBoundingClientRect();
    if (r.width <= 0 && r.height <= 0) return false;
    const s = getComputedStyle(el);
    return s.display !== 'none' && s.visibility !== 'hidden';
  };
  const walk = (el, depth) => {
    if (el.nodeType !== 1 || SKIP_TAGS.has(el.tagName)) return;
    if (n >= MAX_ELEMENTS) { truncated = true; return; }
    if (!visible(el)) return;
    const role = el.getAttribute('role') || implicitRole(el);
    const tag = el.tagName.toLowerCase();
    const interactive = INTERACTIVE_TAGS.has(tag) || INTERACTIVE_ROLES.has(role)
      || el.hasAttribute('onclick') || el.hasAttribute('contenteditable');
    let childDepth = depth;
    if (interactive || role) {
      n += 1;
      const ref = 'e' + n;
      refs[ref] = el;
      let line = '  '.repeat(depth) + '- ' + (role || tag);
      const name = accName(el);
      if (name) line += ' ' + JSON.stringify(name);
      if (el.disabled) line += ' [disabled]';
      if (el.checked) line += ' [checked]';
      if ((tag === 'input' || tag === 'textarea') && el.value) {
        line += ' [value=' + JSON.stringify(cap(String(el.value), 40)) + ']';
      }
      if (role === 'heading') line += ' [level=' + el.tagName[1] + ']';
      line += ' [ref=' + ref + ']';
      lines.push(line);
      childDepth = depth + 1;
    } else if (TEXT_TAGS.has(el.tagName)) {
      const text = cap(el.textContent, 120);
      if (text) lines.push('  '.repeat(depth) + '- text ' + JSON.stringify(text));
    }
    for (const c of el.children) walk(c, childDepth);
  };
  walk(document.body, 0);
  window.__jet_bb_refs = refs;
  return {
    url: location.href,
    title: document.title,
    ref_count: n,
    truncated,
    snapshot: lines.join('\n'),
  };
})()"#;

/// Capture a ref-annotated semantic snapshot of the live DOM. Refs stay
/// valid until the next snapshot or navigation.
pub async fn snapshot(root_dir: &Path) -> Result<Value> {
    let page = attach(root_dir).await?;
    let v = page
        .evaluate(SNAPSHOT_JS)
        .await
        .context("capturing semantic snapshot")?;
    if !v.is_object() {
        bail!("snapshot returned non-object value: {v}");
    }
    Ok(v)
}

// ── Element actions ──────────────────────────────────────────────────────────

/// Click the target element (`el.click()` — same MVP tradeoff as the
/// Locator engine). `dblclick` issues two clicks plus a `dblclick`
/// MouseEvent for handlers bound to the dedicated event.
pub async fn click(root_dir: &Path, target: &Target, dblclick: bool) -> Result<Value> {
    let page = attach(root_dir).await?;
    let el = element_expr(&page, target).await?;
    let body = if dblclick {
        "el.click(); el.click(); \
         el.dispatchEvent(new MouseEvent('dblclick', { bubbles: true })); return true;"
    } else {
        "el.click(); return true;"
    };
    page.evaluate(&action_js(&el, body))
        .await
        .with_context(|| format!("clicking {target:?}"))?;
    Ok(json!({ "ok": true, "action": if dblclick { "dblclick" } else { "click" } }))
}

/// Replace the target input/textarea value (native setter + `input` +
/// `change` events, same recipe as `Locator::fill`).
pub async fn fill(root_dir: &Path, target: &Target, text: &str) -> Result<Value> {
    let page = attach(root_dir).await?;
    let el = element_expr(&page, target).await?;
    let body = format!(
        "el.focus(); \
         const proto = el.tagName === 'TEXTAREA' ? window.HTMLTextAreaElement.prototype : window.HTMLInputElement.prototype; \
         const setter = Object.getOwnPropertyDescriptor(proto, 'value')?.set; \
         if (setter) {{ setter.call(el, {val}); }} else {{ el.value = {val}; }} \
         el.dispatchEvent(new Event('input', {{ bubbles: true }})); \
         el.dispatchEvent(new Event('change', {{ bubbles: true }})); \
         return true;",
        val = json_str(text)
    );
    page.evaluate(&action_js(&el, &body))
        .await
        .with_context(|| format!("filling {target:?}"))?;
    Ok(json!({ "ok": true, "action": "fill" }))
}

/// Focus the target and type `text` through CDP `Input.insertText`,
/// which exercises the page's real key/input pipeline (append — use
/// `fill` to replace).
pub async fn type_text(root_dir: &Path, target: &Target, text: &str) -> Result<Value> {
    let page = attach(root_dir).await?;
    let el = element_expr(&page, target).await?;
    page.evaluate(&action_js(&el, "el.focus(); return true;"))
        .await
        .with_context(|| format!("focusing {target:?}"))?;
    page.session()
        .send("Input.insertText", json!({ "text": text }))
        .await
        .context("inserting text via CDP")?;
    Ok(json!({ "ok": true, "action": "type", "chars": text.chars().count() }))
}

/// Hover the target (mouseenter + mousemove at the element center,
/// same recipe as `Locator::hover`).
pub async fn hover(root_dir: &Path, target: &Target) -> Result<Value> {
    let page = attach(root_dir).await?;
    let el = element_expr(&page, target).await?;
    let body = "const r = el.getBoundingClientRect(); \
         const opts = { bubbles: true, clientX: r.left + r.width / 2, clientY: r.top + r.height / 2 }; \
         el.dispatchEvent(new MouseEvent('mouseenter', opts)); \
         el.dispatchEvent(new MouseEvent('mousemove', opts)); return true;";
    page.evaluate(&action_js(&el, body))
        .await
        .with_context(|| format!("hovering {target:?}"))?;
    Ok(json!({ "ok": true, "action": "hover" }))
}

/// Select an option of a `<select>` by value or label and dispatch
/// `input` + `change`.
pub async fn select(root_dir: &Path, target: &Target, option: &str) -> Result<Value> {
    let page = attach(root_dir).await?;
    let el = element_expr(&page, target).await?;
    let body = format!(
        "if (el.tagName !== 'SELECT') throw new Error('select target is <' + el.tagName.toLowerCase() + '>, not <select>'); \
         const want = {val}; \
         const opt = [...el.options].find((o) => o.value === want || o.label === want || o.textContent.trim() === want); \
         if (!opt) throw new Error('no option ' + JSON.stringify(want) + ' — available: ' + [...el.options].map((o) => o.value || o.label).join(', ')); \
         el.value = opt.value; \
         el.dispatchEvent(new Event('input', {{ bubbles: true }})); \
         el.dispatchEvent(new Event('change', {{ bubbles: true }})); \
         return opt.value;",
        val = json_str(option)
    );
    let selected = page
        .evaluate(&action_js(&el, &body))
        .await
        .with_context(|| format!("selecting {option:?} on {target:?}"))?;
    Ok(json!({ "ok": true, "action": "select", "value": selected }))
}

/// Check (or uncheck) a checkbox idempotently — clicks only when the
/// state differs, matching `Locator::check`/`uncheck`.
pub async fn set_checked(root_dir: &Path, target: &Target, checked: bool) -> Result<Value> {
    let page = attach(root_dir).await?;
    let el = element_expr(&page, target).await?;
    let body = format!(
        "if (el.checked !== {checked}) el.click(); return el.checked;",
        checked = if checked { "true" } else { "false" }
    );
    let now = page
        .evaluate(&action_js(&el, &body))
        .await
        .with_context(|| format!("setting checked={checked} on {target:?}"))?;
    Ok(json!({
        "ok": true,
        "action": if checked { "check" } else { "uncheck" },
        "checked": now,
    }))
}

// ── Navigation ───────────────────────────────────────────────────────────────

/// Re-arm the observability hooks around a navigation. CDP drops
/// `addScriptToEvaluateOnNewDocument` registrations when the session
/// that added them disconnects — and every `jet bb` command is its own
/// short-lived session — so each navigating verb must re-register the
/// init script on its live session *before* the navigation (the new
/// document then gets hooks at birth, capturing boot-time logs).
/// Page-initiated navigations (link clicks between commands) still
/// lose hooks until the next verb runs; `console`/`requests` heal
/// that by re-installing lazily at read time.
async fn arm_observe_hooks(page: &Page) -> Result<()> {
    // New-document scripts only fire with the Page domain enabled on
    // the registering session (goto gets this from `Page.goto`;
    // reload/history navigation must do it themselves).
    page.session()
        .send("Page.enable", json!({}))
        .await
        .context("enabling Page domain for init-script registration")?;
    page.add_init_script(BB_OBSERVE_INIT_JS)
        .await
        .context("re-registering bb observability init script")?;
    Ok(())
}

/// Navigate the attached session and wait for the load event.
pub async fn goto(root_dir: &Path, url: &str) -> Result<Value> {
    let page = attach(root_dir).await?;
    arm_observe_hooks(&page).await?;
    page.goto(url)
        .await
        .with_context(|| format!("navigating to {url}"))?;
    let landed = page.url().await.unwrap_or_else(|_| url.to_string());
    Ok(json!({ "ok": true, "action": "goto", "url": landed }))
}

/// Step through session history. `delta` is -1 for back, +1 for forward.
pub async fn history_step(root_dir: &Path, delta: i64) -> Result<Value> {
    let page = attach(root_dir).await?;
    let hist = page
        .session()
        .send("Page.getNavigationHistory", json!({}))
        .await
        .context("reading navigation history")?;
    let current = hist["currentIndex"]
        .as_i64()
        .context("navigation history missing currentIndex")?;
    let entries = hist["entries"]
        .as_array()
        .context("navigation history missing entries")?;
    let target = current + delta;
    let direction = if delta < 0 { "back" } else { "forward" };
    if target < 0 || target as usize >= entries.len() {
        bail!("no {direction} history entry (at {current} of {})", entries.len());
    }
    let entry = &entries[target as usize];
    let entry_id = entry["id"].as_i64().context("history entry missing id")?;
    arm_observe_hooks(&page).await?;
    page.session()
        .send("Page.navigateToHistoryEntry", json!({ "entryId": entry_id }))
        .await
        .context("navigating to history entry")?;
    wait_for_ready(&page, 10_000).await?;
    Ok(json!({
        "ok": true,
        "action": direction,
        "url": entry["url"].as_str().unwrap_or(""),
    }))
}

/// Reload the current document and wait for it to become ready again.
pub async fn reload(root_dir: &Path) -> Result<Value> {
    let page = attach(root_dir).await?;
    arm_observe_hooks(&page).await?;
    page.session()
        .send("Page.reload", json!({}))
        .await
        .context("reloading page")?;
    wait_for_ready(&page, 10_000).await?;
    let url = page.url().await.unwrap_or_default();
    Ok(json!({ "ok": true, "action": "reload", "url": url }))
}

/// Resize the viewport via CDP device-metrics override.
pub async fn resize(root_dir: &Path, width: u64, height: u64) -> Result<Value> {
    let page = attach(root_dir).await?;
    page.session()
        .send(
            "Emulation.setDeviceMetricsOverride",
            json!({
                "width": width,
                "height": height,
                "deviceScaleFactor": 0,
                "mobile": false,
            }),
        )
        .await
        .context("overriding device metrics")?;
    Ok(json!({ "ok": true, "action": "resize", "width": width, "height": height }))
}

/// Poll `document.readyState` until `complete`, tolerating the
/// transient evaluate failures a mid-navigation context produces.
async fn wait_for_ready(page: &Page, timeout_ms: u64) -> Result<()> {
    let start = Instant::now();
    loop {
        if let Ok(v) = page.evaluate("document.readyState").await {
            if v.as_str() == Some("complete") {
                return Ok(());
            }
        }
        if start.elapsed() > Duration::from_millis(timeout_ms) {
            bail!("timed out after {timeout_ms}ms waiting for document.readyState=complete");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Wait for a selector to attach, a text to appear, or a fixed delay.
/// Exactly one of `selector`/`text`/`ms` must be given.
pub async fn wait(
    root_dir: &Path,
    selector: Option<&str>,
    text: Option<&str>,
    ms: Option<u64>,
    timeout_ms: u64,
) -> Result<Value> {
    let given = [selector.is_some(), text.is_some(), ms.is_some()]
        .iter()
        .filter(|b| **b)
        .count();
    if given != 1 {
        bail!("pass exactly one of --selector, --text, or --ms");
    }
    if let Some(ms) = ms {
        tokio::time::sleep(Duration::from_millis(ms)).await;
        return Ok(json!({ "ok": true, "action": "wait", "slept_ms": ms }));
    }
    let page = attach(root_dir).await?;
    if let Some(sel) = selector {
        let locator = page
            .locator(sel)
            .map_err(|e| anyhow::anyhow!("invalid selector {sel:?}: {e}"))?
            .with_options(crate::browser::LocatorOptions {
                timeout_ms,
                ..Default::default()
            });
        locator
            .wait_for(Actionability::Attached)
            .await
            .map_err(|e| anyhow::anyhow!("waiting for {sel:?}: {e}"))?;
        return Ok(json!({ "ok": true, "action": "wait", "selector": sel }));
    }
    let needle = text.expect("text is the only remaining arm");
    let js = format!(
        "(document.body.innerText || '').includes({})",
        json_str(needle)
    );
    let start = Instant::now();
    loop {
        if page.evaluate(&js).await?.as_bool() == Some(true) {
            return Ok(json!({ "ok": true, "action": "wait", "text": needle }));
        }
        if start.elapsed() > Duration::from_millis(timeout_ms) {
            bail!("timed out after {timeout_ms}ms waiting for text {needle:?}");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

// ── Observability ────────────────────────────────────────────────────────────

/// Read (and optionally drain) one of the init-script observation
/// rings. When the current document is missing the hooks (it was
/// created by a page-initiated navigation while no `jet bb` command
/// was attached), install them now and say so — history from before
/// this call is genuinely gone in that case.
async fn read_observe_buffer(
    root_dir: &Path,
    buffer: &str,
    level: Option<&str>,
    limit: usize,
    clear: bool,
) -> Result<(Value, bool)> {
    let page = attach(root_dir).await?;
    let filter = match level {
        Some(l) => format!(".filter((e) => e.level === {})", json_str(l)),
        None => String::new(),
    };
    let js = format!(
        "(() => {{ const buf = window.{buffer}; \
         if (!buf) return null; \
         const out = buf{filter}.slice(-{limit}); \
         {clear_stmt} \
         return out; }})()",
        clear_stmt = if clear { "buf.length = 0;" } else { "" },
    );
    let v = page.evaluate(&js).await.context("reading observation buffer")?;
    if !v.is_null() {
        return Ok((v, false));
    }
    page.evaluate(BB_OBSERVE_INIT_JS)
        .await
        .context("installing observability hooks on the live document")?;
    arm_observe_hooks(&page).await?;
    Ok((json!([]), true))
}

/// Caveat attached to reads that had to heal missing hooks.
const HOOKS_HEALED_NOTE: &str = "observability hooks were missing on this document \
     (page-initiated navigation) — installed now; earlier history is unavailable";

fn observe_result(entries: Value, healed: bool) -> Value {
    if healed {
        json!({ "ok": true, "entries": entries, "note": HOOKS_HEALED_NOTE })
    } else {
        json!({ "ok": true, "entries": entries })
    }
}

/// Console messages, page errors, and unhandled rejections captured
/// since launch (or the last `--clear`).
pub async fn console(
    root_dir: &Path,
    level: Option<&str>,
    limit: usize,
    clear: bool,
) -> Result<Value> {
    let (entries, healed) =
        read_observe_buffer(root_dir, "__jet_bb_console", level, limit.max(1), clear).await?;
    Ok(observe_result(entries, healed))
}

/// fetch/XHR activity captured since launch (or the last `--clear`).
pub async fn requests(root_dir: &Path, limit: usize, clear: bool) -> Result<Value> {
    let (entries, healed) =
        read_observe_buffer(root_dir, "__jet_bb_requests", None, limit.max(1), clear).await?;
    Ok(observe_result(entries, healed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_parsing_distinguishes_refs_from_selectors() {
        assert_eq!(parse_target("e12").unwrap(), Target::Ref("e12".into()));
        assert_eq!(parse_target("ref=e3").unwrap(), Target::Ref("e3".into()));
        assert_eq!(
            parse_target("#save-btn").unwrap(),
            Target::Selector("#save-btn".into())
        );
        assert_eq!(
            parse_target("text=Submit").unwrap(),
            Target::Selector("text=Submit".into())
        );
        assert_eq!(
            parse_target("role=button[name=\"Save\"]").unwrap(),
            Target::Selector("role=button[name=\"Save\"]".into())
        );
        // `em` and `e12x` are element/CSS shapes, not refs.
        assert_eq!(parse_target("em").unwrap(), Target::Selector("em".into()));
        assert_eq!(
            parse_target("e12x").unwrap(),
            Target::Selector("e12x".into())
        );
        assert!(parse_target("").is_err());
        assert!(parse_target("ref=button").is_err());
    }

    #[test]
    fn ref_json_encoding_survives_quotes_and_unicode() {
        // Ref/selector strings are spliced into JS via json_str — pin
        // that the encoding is real JSON, not naive quoting.
        assert_eq!(json_str("e7"), "\"e7\"");
        assert_eq!(json_str("a\"b"), "\"a\\\"b\"");
        assert_eq!(json_str("text=按鈕"), "\"text=按鈕\"");
    }

    #[test]
    fn snapshot_js_mints_refs_and_persists_the_map() {
        assert!(SNAPSHOT_JS.contains("window.__jet_bb_refs = refs"));
        assert!(SNAPSHOT_JS.contains("'e' + n"));
        assert!(SNAPSHOT_JS.contains("ref_count"));
        // Visibility gating must be present — invisible elements get
        // refs in no snapshot model worth the tokens.
        assert!(SNAPSHOT_JS.contains("visibility"));
    }

    #[test]
    fn observe_init_js_installs_both_buffers_idempotently() {
        assert!(BB_OBSERVE_INIT_JS.contains("__jet_bb_observe_installed"));
        assert!(BB_OBSERVE_INIT_JS.contains("window.__jet_bb_console"));
        assert!(BB_OBSERVE_INIT_JS.contains("window.__jet_bb_requests"));
        assert!(BB_OBSERVE_INIT_JS.contains("unhandledrejection"));
        // Rings must be capped — unbounded buffers leak in long sessions.
        assert!(BB_OBSERVE_INIT_JS.contains("CAP = 1000"));
    }

    #[test]
    fn action_js_scrolls_target_into_view() {
        let js = action_js("document.body", "return true;");
        assert!(js.starts_with("(() => {"));
        assert!(js.contains("scrollIntoView"));
        assert!(js.contains("document.body"));
    }
}
// </HANDWRITE>
