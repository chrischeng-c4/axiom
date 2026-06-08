// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#unit-test
// CODEGEN-BEGIN
//! Visual regression guard for real MUI on React DOM vs Jet WASM.
//!
//! This intentionally uses a committed fixture under examples/ instead of the
//! simplified parity oracle. The DOM side must load real MUI packages installed
//! by `jet install`, render non-blank content in Chromium, and give Jet WASM a
//! comparable external page surface.

mod common;

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::ffi::OsStr;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

use common::react_oracle::{
    screenshot_phash_diff_message, screenshot_phash_hamming_distance, screenshot_phashes_match,
    screenshot_summaries_match, screenshot_summary_from_png, screenshot_visual_probe_from_png,
};
use jet::browser::Page;
use jet::browser_cli;
use jet::wasm_build::manifest as wasm_manifest;

const VISUAL_TABLE_EXPECTED_ROWS: u64 = 100;
const VISUAL_TABLE_EXPECTED_CELLS: u64 = 10_000;
const VISUAL_TABLE_TITLE: &str = "MUI visual table fixture";
const VISUAL_TABLE_LAST_CELL: &str = "cell 9999";
const VISUAL_SCREENSHOT_MIN_FOREGROUND_COUNT: u64 = 1_000;
const VISUAL_SCREENSHOT_MIN_NON_WHITE: u64 = 1_000;
const WASM_COMPAT_LOWERING_MARKER: &str = "using Rust/WASM compatibility lowering";

const VISUAL_READY_EXPR: &str = r#"(() => {
  const summary = (() => {
    const domRoot = document.querySelector('#visual-root');
    if (domRoot) {
      const text = domRoot.innerText || '';
      return {
        visualRoot: true,
        source: 'dom',
        tableRows: domRoot.querySelectorAll('tbody tr').length,
        tableCells: domRoot.querySelectorAll('td').length,
        cellLabelTexts: domRoot.querySelectorAll('td').length,
        hasTitle: text.includes('MUI visual table fixture'),
        hasFirstCell: text.includes('cell 0'),
        hasLastCell: text.includes('cell 9999')
      };
    }
    const root = window.__jet_debug?.elementTree?.();
    const out = {
      visualRoot: false,
      source: window.__jet_debug ? 'jet-wasm' : 'none',
      tableRows: 0,
      tableCells: 0,
      cellLabelTexts: 0,
      hasZeroText: false,
      hasLastNumberText: false,
      hasTitle: false,
      hasFirstCell: false,
      hasLastCell: false
    };
    const walk = (node) => {
      if (!node || typeof node !== 'object') return;
      if (node.kind === 'intrinsic') {
        if (node.tag === 'main') out.visualRoot = true;
        if (node.tag === 'tr') out.tableRows += 1;
        if (node.tag === 'td') out.tableCells += 1;
      }
      if (node.kind === 'text') {
        const text = node.text == null ? '' : String(node.text);
        const trimmed = text.trim();
        if (text.includes('MUI visual table fixture')) out.hasTitle = true;
        if (trimmed === 'cell') out.cellLabelTexts += 1;
        if (trimmed === '0') out.hasZeroText = true;
        if (trimmed === '9999') out.hasLastNumberText = true;
        if (text.includes('cell 0')) out.hasFirstCell = true;
        if (text.includes('cell 9999')) out.hasLastCell = true;
      }
      for (const child of node.children || []) walk(child);
    };
    walk(root);
    out.hasFirstCell = out.hasFirstCell || (out.cellLabelTexts > 0 && out.hasZeroText);
    out.hasLastCell = out.hasLastCell || (out.cellLabelTexts > 0 && out.hasLastNumberText);
    return out;
  })();
  const baseReady = summary.visualRoot &&
    summary.tableRows === 100 &&
    summary.tableCells === 10000 &&
    summary.cellLabelTexts === 10000 &&
    summary.hasTitle &&
    summary.hasFirstCell &&
    summary.hasLastCell;
  if (!baseReady) return false;
  if (summary.source === 'jet-wasm') {
    const status = window.__jet_webgpu_status || {};
    return status.frames >= 1 &&
      status.bridgeMode === 'text' &&
      status.lastTextGlyphCount > 0 &&
      status.textAtlasMode === 'glyph-atlas' &&
      status.lastTextAtlasUploadCount >= 1 &&
      status.lastTextAtlasWidth > 1 &&
      status.lastTextAtlasHeight > 1 &&
      status.lastTextAtlasNonZeroAlphaCount > 0 &&
      status.lastUnsupportedCount === 0;
  }
  return true;
})()"#;

const VISUAL_TEXT_EXPR: &str = r#"(() => {
  const domRoot = document.querySelector('#visual-root');
  if (domRoot) {
    const text = domRoot.innerText || '';
    return [
      text.includes('MUI visual table fixture') ? 'MUI visual table fixture' : '',
      text.includes('cell 0') ? 'cell 0' : '',
      text.includes('cell 9999') ? 'cell 9999' : ''
    ].filter(Boolean).join(' ');
  }
  const root = window.__jet_debug?.elementTree?.();
  const tokens = [];
  let cellLabelTexts = 0;
  let hasZeroText = false;
  let hasLastNumberText = false;
  const walk = (node) => {
    if (!node || typeof node !== 'object') return;
    if (node.kind === 'text') {
      const text = node.text == null ? '' : String(node.text);
      const trimmed = text.trim();
      if (text.includes('MUI visual table fixture')) tokens.push('MUI visual table fixture');
      if (trimmed === 'cell') cellLabelTexts += 1;
      if (trimmed === '0') hasZeroText = true;
      if (trimmed === '9999') hasLastNumberText = true;
    }
    for (const child of node.children || []) walk(child);
  };
  walk(root);
  if (cellLabelTexts > 0 && hasZeroText) tokens.push('cell 0');
  if (cellLabelTexts > 0 && hasLastNumberText) tokens.push('cell 9999');
  return tokens.join(' ');
})()"#;

const WAIT_FOR_BROWSER_PAINT_EXPR: &str = r#"(() => new Promise((resolve) => {
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      void document.body?.getBoundingClientRect();
      resolve(true);
    });
  });
}))()"#;

const VISUAL_DIAGNOSTICS_EXPR: &str = r#"(() => {
  const summarizeDomShell = () => {
    const bodyText = document.body?.innerText || '';
    return {
      visualRoot: !!document.querySelector('#visual-root'),
      jetCanvas: !!document.querySelector('#jet-canvas'),
      largeTable: !!document.querySelector('#large-table'),
      tableElements: document.querySelectorAll('table').length,
      tbodyElements: document.querySelectorAll('tbody').length,
      trElements: document.querySelectorAll('tr').length,
      tdElements: document.querySelectorAll('td').length,
      bodyHasCell0: bodyText.includes('cell 0'),
      bodyHasLastCell: bodyText.includes('cell 9999'),
      bodyTextLength: bodyText.length
    };
  };
  const summarizeJetTree = () => {
    const root = window.__jet_debug?.elementTree?.();
    const out = {
      visualRoot: false,
      tableRows: 0,
      tableCells: 0,
      cellLabelTexts: 0,
      hasZeroText: false,
      hasLastNumberText: false,
      hasTitle: false,
      hasFirstCell: false,
      hasLastCell: false
    };
    const walk = (node) => {
      if (!node || typeof node !== 'object') return;
      if (node.kind === 'intrinsic') {
        if (node.tag === 'main') out.visualRoot = true;
        if (node.tag === 'tr') out.tableRows += 1;
        if (node.tag === 'td') out.tableCells += 1;
      }
      if (node.kind === 'text') {
        const text = node.text == null ? '' : String(node.text);
        const trimmed = text.trim();
        if (text.includes('MUI visual table fixture')) out.hasTitle = true;
        if (trimmed === 'cell') out.cellLabelTexts += 1;
        if (trimmed === '0') out.hasZeroText = true;
        if (trimmed === '9999') out.hasLastNumberText = true;
        if (text.includes('cell 0')) out.hasFirstCell = true;
        if (text.includes('cell 9999')) out.hasLastCell = true;
      }
      for (const child of node.children || []) walk(child);
    };
    walk(root);
    out.hasFirstCell = out.hasFirstCell || (out.cellLabelTexts > 0 && out.hasZeroText);
    out.hasLastCell = out.hasLastCell || (out.cellLabelTexts > 0 && out.hasLastNumberText);
    return out;
  };
  const domRoot = document.querySelector('#visual-root');
  const domText = domRoot?.innerText || '';
  return {
    visualRoot: !!domRoot,
    debugBridge: !!window.__jet_debug,
    domShell: summarizeDomShell(),
    webgpuStatus: window.__jet_webgpu_status
      ? JSON.parse(JSON.stringify(window.__jet_webgpu_status))
      : null,
    table: domRoot ? {
      source: 'dom',
      visualRoot: true,
      tableRows: domRoot.querySelectorAll('tbody tr').length,
      tableCells: domRoot.querySelectorAll('td').length,
      cellLabelTexts: domRoot.querySelectorAll('td').length,
      hasZeroText: domText.includes('cell 0'),
      hasLastNumberText: domText.includes('cell 9999'),
      hasTitle: domText.includes('MUI visual table fixture'),
      hasFirstCell: domText.includes('cell 0'),
      hasLastCell: domText.includes('cell 9999')
    } : {
      source: window.__jet_debug ? 'jet-wasm' : 'none',
      ...summarizeJetTree()
    },
    rootHtml: document.querySelector('#root')?.innerHTML?.slice(0, 800) ?? '',
    bodyHtml: document.body?.innerHTML?.slice(0, 800) ?? '',
    events: window.__jetVisualEvents ?? [],
    resourceCount: performance.getEntriesByType('resource').length,
    muiResources: performance.getEntriesByType('resource')
      .map((entry) => entry.name)
      .filter((name) => name.includes('@mui'))
      .slice(-40),
    failedResources: performance.getEntriesByType('resource')
      .filter((entry) => entry.responseStatus && entry.responseStatus >= 400)
      .map((entry) => ({ name: entry.name, status: entry.responseStatus }))
      .slice(-40)
  };
})()"#;

const DOM_SELECTION_TARGETS_EXPR: &str = r#"(() => {
  const cells = Array.from(document.querySelectorAll('#large-table td'));
  const start = cells.find((cell) => (cell.textContent || '').trim() === 'cell 0');
  const end = cells.find((cell) => (cell.textContent || '').trim() === 'cell 101');
  const center = (cell) => {
    const rect = cell.getBoundingClientRect();
    return {
      x: rect.left + rect.width / 2,
      y: rect.top + rect.height / 2,
      width: rect.width,
      height: rect.height
    };
  };
  if (!start) return { ok: false, reason: 'missing DOM cell 0', cellsFound: cells.length };
  if (!end) return { ok: false, reason: 'missing DOM cell 101', cellsFound: cells.length };
  return {
    ok: true,
    cellsFound: cells.length,
    start: center(start),
    end: center(end)
  };
})()"#;

const DOM_SELECTION_READ_EXPR: &str = r#"(() => {
  const expectedSelectedColor = 'rgb(232, 240, 254)';
  const installClipboardSpy = () => {
    if (Array.isArray(window.__jetClipboardWrites)) {
      return window.__jetClipboardWrites;
    }
    const writes = [];
    const clipboard = {
      writeText(text) {
        writes.push(String(text));
        return Promise.resolve();
      }
    };
    try {
      Object.defineProperty(navigator, 'clipboard', {
        value: clipboard,
        configurable: true
      });
    } catch (_) {
      navigator.clipboard = clipboard;
    }
    window.__jetClipboardWrites = writes;
    return writes;
  };
  const clipboardWrites = installClipboardSpy();
  const domRoot = document.querySelector('#visual-root');
  const domCells = Array.from(document.querySelectorAll('#large-table td'));
  const result = {
    ok: false,
    source: 'dom',
    input: 'cdp-mouse',
    reason: '',
    cellsFound: domCells.length,
    clicked: [],
    selectionFillOps: 0,
    selectionStrokeOps: 0,
    selectedValues: [],
    selectedTsv: '',
    copiedTsv: '',
    keyboardCopiedTsv: '',
    keyboardClipboardWriteCount: 0,
    directionChecks: [],
    clipboardWriteCount: 0,
    nativeSelectionText: '',
    liveSelection: null
  };
  if (!domRoot) {
    result.reason = 'missing visual root';
    return result;
  }
  if (domCells.length === 0) {
    result.reason = 'missing DOM cells';
    return result;
  }
  const finishDom = () => {
    const selected = domCells
      .map((cell) => ({
        cell,
        text: (cell.textContent || '').trim(),
        style: getComputedStyle(cell)
      }))
      .filter(({ cell, style }) =>
        cell.getAttribute('aria-selected') === 'true' ||
        style.backgroundColor === expectedSelectedColor
      );
    result.selectedValues = selected.map(({ text }) => text);
    result.selectionFillOps = selected.length;
    result.selectionStrokeOps = selected.filter(({ style }) => style.boxShadow !== 'none').length;
    result.selectedTsv = [
      result.selectedValues.slice(0, 2).join('\t'),
      result.selectedValues.slice(2, 4).join('\t')
    ].join('\n');
    result.nativeSelectionText = String(window.getSelection ? window.getSelection() : '');
    result.liveSelection = window.__muiVisualSelection || null;
    return new Promise((resolve) => setTimeout(() => {
      result.keyboardClipboardWriteCount = clipboardWrites.length;
      result.keyboardCopiedTsv = clipboardWrites[clipboardWrites.length - 1] || '';
      result.clipboardWriteCount = clipboardWrites.length;
      result.copiedTsv = result.keyboardCopiedTsv;
      result.ok = true;
      resolve(result);
    }, 100));
  };
  return new Promise((resolve) => setTimeout(() => resolve(finishDom()), 100));
})()"#;

const WASM_SELECTION_TARGETS_EXPR: &str = r#"(() => {
  const canvas = document.getElementById('jet-canvas');
  const debug = window.__jet_debug;
  if (!canvas) return { ok: false, reason: 'missing jet-canvas', cellsFound: 0 };
  if (!debug || typeof debug.layoutTree !== 'function') {
    return { ok: false, reason: 'missing debug layoutTree', cellsFound: 0 };
  }
  const layout = debug.layoutTree();
  if (!layout || !Array.isArray(layout.nodes)) {
    return { ok: false, reason: 'invalid layoutTree shape', cellsFound: 0 };
  }
  const contains = (rect, point) =>
    point.x >= rect.x &&
    point.x < rect.x + rect.w &&
    point.y >= rect.y &&
    point.y < rect.y + rect.h;
  const textCenterInside = (rect, textRect) => contains(rect, {
    x: textRect.x + textRect.w / 2,
    y: textRect.y + textRect.h / 2
  });
  const cells = [];
  for (let i = 0; i < layout.nodes.length; i += 1) {
    const node = layout.nodes[i];
    if (node?.kind?.kind !== 'intrinsic' || node.kind.tag !== 'td') continue;
    const value = layout.nodes
      .slice(i + 1)
      .filter((candidate) =>
        candidate?.kind?.kind === 'text' &&
        textCenterInside(node.rect, candidate.rect)
      )
      .map((candidate) => String(candidate.kind.text || ''))
      .join('')
      .trim();
    const match = /^cell (\d+)$/.exec(value);
    if (!match) continue;
    const index = Number(match[1]);
    cells.push({
      index,
      row: Math.floor(index / 100),
      col: index % 100,
      rect: node.rect,
      value
    });
  }
  const start = cells.find((cell) => cell.row === 0 && cell.col === 0);
  const end = cells.find((cell) => cell.row === 1 && cell.col === 1);
  const canvasRect = canvas.getBoundingClientRect();
  const center = (cell) => cell ? {
    x: canvasRect.left + cell.rect.x + cell.rect.w / 2,
    y: canvasRect.top + cell.rect.y + cell.rect.h / 2,
    width: cell.rect.w,
    height: cell.rect.h,
    value: cell.value,
    row: cell.row,
    col: cell.col
  } : null;
  if (!start) return { ok: false, reason: 'missing visible start cell', cellsFound: cells.length };
  if (!end) return { ok: false, reason: 'missing visible end cell', cellsFound: cells.length };
  return {
    ok: true,
    source: 'jet-wasm',
    input: 'jet-browser-cdp',
    cellsFound: cells.length,
    start: center(start),
    end: center(end)
  };
})()"#;

const WASM_INSTALL_CLIPBOARD_SPY_EXPR: &str = r#"(() => {
  const writes = [];
  const clipboard = {
    writeText(text) {
      writes.push(String(text));
      return Promise.resolve();
    }
  };
  try {
    Object.defineProperty(navigator, 'clipboard', {
      value: clipboard,
      configurable: true
    });
  } catch (_) {
    navigator.clipboard = clipboard;
  }
  window.__jetClipboardWrites = writes;
  document.getElementById('jet-canvas')?.focus?.();
  return true;
})()"#;

const WASM_SELECTION_READ_EXPR: &str = r#"(() => {
  const clone = (value) => JSON.parse(JSON.stringify(value || {}));
  const selectedFill = { r: 0x1a, g: 0x73, b: 0xe8, a: 0x33 };
  const debug = window.__jet_debug;
  const layout = debug?.layoutTree?.();
  const paintOps = debug?.paintOps?.() || [];
  const clipboardWrites = window.__jetClipboardWrites || [];
  const status = clone(window.__jet_webgpu_status);
  const result = {
    ok: false,
    source: 'jet-wasm',
    input: 'jet-browser-cdp',
    reason: '',
    statusBefore: null,
    statusAfter: status,
    cellsFound: 0,
    clicked: [],
    selectionFillOps: 0,
    selectionStrokeOps: 0,
    selectedValues: [],
    selectedTsv: '',
    copiedTsv: clipboardWrites[clipboardWrites.length - 1] || status.lastCopiedTsv || '',
    keyboardCopiedTsv: clipboardWrites[clipboardWrites.length - 1] || status.lastCopiedTsv || '',
    keyboardClipboardWriteCount: clipboardWrites.length,
    statusAfterKeyboard: status,
    directionChecks: [],
    clipboardWriteCount: clipboardWrites.length
  };
  if (!layout || !Array.isArray(layout.nodes)) {
    result.reason = 'invalid layoutTree shape';
    return result;
  }
  const contains = (rect, point) =>
    point.x >= rect.x &&
    point.x < rect.x + rect.w &&
    point.y >= rect.y &&
    point.y < rect.y + rect.h;
  const textCenterInside = (rect, textRect) => contains(rect, {
    x: textRect.x + textRect.w / 2,
    y: textRect.y + textRect.h / 2
  });
  const cells = [];
  for (let i = 0; i < layout.nodes.length; i += 1) {
    const node = layout.nodes[i];
    if (node?.kind?.kind !== 'intrinsic' || node.kind.tag !== 'td') continue;
    const value = layout.nodes
      .slice(i + 1)
      .filter((candidate) =>
        candidate?.kind?.kind === 'text' &&
        textCenterInside(node.rect, candidate.rect)
      )
      .map((candidate) => String(candidate.kind.text || ''))
      .join('')
      .trim();
    const match = /^cell (\d+)$/.exec(value);
    if (!match) continue;
    const index = Number(match[1]);
    cells.push({
      index,
      row: Math.floor(index / 100),
      col: index % 100,
      rect: node.rect,
      value
    });
  }
  result.cellsFound = cells.length;
  const isSelectionColor = (op) =>
    op?.color?.r === selectedFill.r &&
    op?.color?.g === selectedFill.g &&
    op?.color?.b === selectedFill.b &&
    op?.color?.a === selectedFill.a;
  const selectedRects = paintOps
    .filter((op) => op.op === 'fill_rect' && isSelectionColor(op))
    .map((op) => op.rect);
  const sameRect = (a, b) =>
    Math.abs(a.x - b.x) < 0.1 &&
    Math.abs(a.y - b.y) < 0.1 &&
    Math.abs(a.w - b.w) < 0.1 &&
    Math.abs(a.h - b.h) < 0.1;
  const selectedCells = cells
    .filter((cell) => selectedRects.some((rect) => sameRect(rect, cell.rect)))
    .sort((a, b) => a.row - b.row || a.col - b.col);
  result.selectedValues = selectedCells.map((cell) => cell.value);
  result.selectionFillOps = selectedCells.length;
  result.selectedTsv = [
    result.selectedValues.slice(0, 2).join('\t'),
    result.selectedValues.slice(2, 4).join('\t')
  ].join('\n');
  result.ok = true;
  return result;
})()"#;

const WASM_SCROLL_READ_EXPR: &str = r#"(() => {
  const clone = (value) => JSON.parse(JSON.stringify(value || {}));
  const debug = window.__jet_debug;
  const layout = debug?.layoutTree?.();
  const status = clone(window.__jet_webgpu_status);
  const result = {
    ok: false,
    source: 'jet-wasm',
    input: 'jet-browser-wheel',
    reason: '',
    status,
    cellsFound: 0,
    visibleValues: [],
    targetValuesVisible: false
  };
  if (!layout || !Array.isArray(layout.nodes)) {
    result.reason = 'invalid layoutTree shape after wheel';
    return result;
  }
  const contains = (rect, point) =>
    point.x >= rect.x &&
    point.x < rect.x + rect.w &&
    point.y >= rect.y &&
    point.y < rect.y + rect.h;
  const textCenterInside = (rect, textRect) => contains(rect, {
    x: textRect.x + textRect.w / 2,
    y: textRect.y + textRect.h / 2
  });
  const cells = [];
  for (let i = 0; i < layout.nodes.length; i += 1) {
    const node = layout.nodes[i];
    if (node?.kind?.kind !== 'intrinsic' || node.kind.tag !== 'td') continue;
    const value = layout.nodes
      .slice(i + 1)
      .filter((candidate) =>
        candidate?.kind?.kind === 'text' &&
        textCenterInside(node.rect, candidate.rect)
      )
      .map((candidate) => String(candidate.kind.text || ''))
      .join('')
      .trim();
    if (!/^cell \d+$/.test(value)) continue;
    cells.push({ value, rect: node.rect });
  }
  result.cellsFound = cells.length;
  result.visibleValues = cells.map((cell) => cell.value).slice(0, 40);
  const values = new Set(cells.map((cell) => cell.value));
  result.targetValuesVisible = values.has('cell 2000') && values.has('cell 2001');
  result.ok = result.targetValuesVisible && status.scrollTop >= 660;
  if (!result.ok) {
    result.reason = 'expected row 20 cells after wheel scroll';
  }
  return result;
})()"#;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("projects/")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

async fn free_port() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind free port");
    listener.local_addr().expect("local addr").port()
}

fn missing_mui_install_deps(fixture: &Path) -> Vec<&'static str> {
    let required = [
        "node_modules/react/package.json",
        "node_modules/react-dom/package.json",
        "node_modules/@mui/material/package.json",
    ];
    required
        .iter()
        .filter(|rel| !fixture.join(rel).exists())
        .copied()
        .collect()
}

fn require_mui_install(fixture: &Path) {
    let missing = missing_mui_install_deps(fixture);
    assert!(
        missing.is_empty(),
        "examples/mui-visual-demo dependencies are missing: {missing:?}. Run `cd examples/mui-visual-demo && jet install` before this visual regression gate."
    );
}

async fn wait_for_http(url: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;
    for _ in 0..300 {
        if client.get(url).send().await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err(anyhow!("server did not become ready at {url}"))
}

fn run_jet_command<I, S>(fixture: &Path, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let exe = env!("CARGO_BIN_EXE_jet");
    Command::new(exe)
        .args(args)
        .current_dir(fixture)
        .output()
        .context("run jet command")
}

fn require_success(output: Output, context: &str) -> Result<Output> {
    if output.status.success() {
        return Ok(output);
    }
    Err(anyhow!(
        "{context} failed\nstatus={}\nstdout={}\nstderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn assert_no_wasm_compat_lowering_log(text: &str, context: &str) {
    assert!(
        !text.contains(WASM_COMPAT_LOWERING_MARKER),
        "{context} must not use Rust/WASM compatibility lowering; that would make the visual gate a fake success.\nlog={}",
        truncate_for_failure(text)
    );
}

fn assert_wasm_manifest_uses_strict_lowering(
    fixture: &Path,
    expected_mode: &str,
    context: &str,
) -> Result<()> {
    let manifest: Value = serde_json::from_str(
        &std::fs::read_to_string(fixture.join("dist/jet-target.json"))
            .with_context(|| format!("read {context} jet-target.json"))?,
    )
    .with_context(|| format!("parse {context} jet-target.json"))?;
    assert_eq!(
        manifest.pointer("/build/mode").and_then(Value::as_str),
        Some(expected_mode),
        "{context} must record build.mode={expected_mode:?} in jet-target.json.\nmanifest={}",
        json_for_failure(&manifest)
    );
    assert_eq!(
        manifest
            .pointer("/build/tsx_lowering")
            .and_then(Value::as_str),
        Some(wasm_manifest::TSX_LOWERING_STRICT),
        "{context} must record strict TSX lowering in jet-target.json; compatibility lowering would make the visual gate a fake success.\nmanifest={}",
        json_for_failure(&manifest)
    );
    Ok(())
}

fn assert_wasm_demo_uses_strict_lowering(fixture: &Path) -> Result<()> {
    let output = require_success(
        run_jet_command(fixture, ["build", "--wasm"])?,
        "strict-lowering preflight `jet build --wasm` for examples/mui-visual-demo",
    )?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_no_wasm_compat_lowering_log(&stdout, "`jet build --wasm` stdout");
    assert_no_wasm_compat_lowering_log(&stderr, "`jet build --wasm` stderr");
    assert_wasm_manifest_uses_strict_lowering(fixture, "release", "`jet build --wasm` preflight")?;
    Ok(())
}

fn spawn_jet_dev(fixture: &Path, port: u16, wasm: bool) -> Result<Child> {
    let exe = env!("CARGO_BIN_EXE_jet");
    let port = port.to_string();
    let mut command = Command::new(exe);
    if wasm {
        command.args(["dev", "--wasm", "--debug", "-p", port.as_str()]);
    } else {
        command.args(["dev", "-p", port.as_str()]);
    }
    command
        .current_dir(fixture)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| {
            if wasm {
                "spawn jet dev --wasm --debug for MUI fixture"
            } else {
                "spawn jet dev for React DOM MUI fixture"
            }
            .to_string()
        })
}

fn run_jet_install(fixture: &Path) -> Result<()> {
    require_success(
        run_jet_command(fixture, ["install", "--frozen-lockfile"])?,
        "jet install --frozen-lockfile for examples/mui-visual-demo",
    )?;
    Ok(())
}

fn read_child_stderr(child: &mut Child) -> String {
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ = pipe.read_to_string(&mut stderr);
    }
    stderr
}

fn wait_child_exit(child: &mut Child, context: &str) -> Result<String> {
    for _ in 0..120 {
        if let Some(status) = child.try_wait()? {
            let stderr = read_child_stderr(child);
            if status.success() {
                return Ok(stderr);
            }
            return Err(anyhow!(
                "{context} exited unsuccessfully: status={status}\nstderr={}",
                truncate_for_failure(&stderr)
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(anyhow!("{context} did not exit after Jet shutdown request"))
}

fn shutdown_jet_dev(fixture: &Path, port: u16, child: &mut Child) -> Result<String> {
    let port = port.to_string();
    require_success(
        run_jet_command(fixture, ["dev", "shutdown", "-p", port.as_str()])?,
        "jet dev shutdown",
    )?;
    wait_child_exit(child, "jet dev")
}

fn truncate_for_failure(text: &str) -> String {
    const MAX: usize = 12_000;
    if text.len() <= MAX {
        return text.to_string();
    }
    format!("{}... <truncated {} bytes>", &text[..MAX], text.len() - MAX)
}

fn json_for_failure(value: &Value) -> String {
    truncate_for_failure(&serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()))
}

struct VisualSnapshot {
    body_text: String,
    diagnostics: Value,
    browser_capture: Value,
    png: Vec<u8>,
    screenshot_summary: Value,
    screenshot_visual_probe: Value,
    browser_stderr: String,
}

struct SelectionInteractionSnapshot {
    diagnostics: Value,
    png: Vec<u8>,
    screenshot_summary: Value,
    screenshot_visual_probe: Value,
    browser_stderr: String,
}

struct VisualRunSnapshots {
    visual: VisualSnapshot,
    selection: SelectionInteractionSnapshot,
}

async fn wait_for_visual_ready(page: &Page, stem: &str) -> Result<()> {
    for _ in 0..300 {
        let ready = page
            .evaluate(VISUAL_READY_EXPR)
            .await
            .unwrap_or(Value::Bool(false));
        if ready.as_bool().unwrap_or(false) {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let body_text = page
        .evaluate(VISUAL_TEXT_EXPR)
        .await
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default();
    let diagnostics = page
        .evaluate(VISUAL_DIAGNOSTICS_EXPR)
        .await
        .unwrap_or(Value::Null);
    Err(anyhow!(
        "{stem} page did not reach visual-ready state before snapshot\nbody={body_text:?}\ndiag={}",
        serde_json::to_string_pretty(&diagnostics).unwrap_or_else(|_| diagnostics.to_string())
    ))
}

fn selection_target_coord(targets: &Value, name: &str, axis: &str) -> Result<f64> {
    targets
        .get(name)
        .and_then(|target| target.get(axis))
        .and_then(Value::as_f64)
        .with_context(|| format!("DOM selection target {name}.{axis} missing in {targets}"))
}

async fn drive_dom_selection_with_jet_browser(
    page: &Page,
    root_dir: &Path,
    reverse: bool,
) -> Result<()> {
    let targets = page
        .evaluate(DOM_SELECTION_TARGETS_EXPR)
        .await
        .context("resolve DOM selection target coordinates")?;
    if !targets.get("ok").and_then(Value::as_bool).unwrap_or(false) {
        return Err(anyhow!(
            "DOM selection targets were not available: {}",
            serde_json::to_string_pretty(&targets).unwrap_or_else(|_| targets.to_string())
        ));
    }

    let start_x = selection_target_coord(&targets, "start", "x")?;
    let start_y = selection_target_coord(&targets, "start", "y")?;
    let end_x = selection_target_coord(&targets, "end", "x")?;
    let end_y = selection_target_coord(&targets, "end", "y")?;
    let (from_x, from_y, to_x, to_y) = if reverse {
        (end_x, end_y, start_x, start_y)
    } else {
        (start_x, start_y, end_x, end_y)
    };

    browser_cli::drag(root_dir, from_x, from_y, to_x, to_y, 8)
        .await
        .context("drive React DOM Jet browser drag selection")?;
    tokio::time::sleep(Duration::from_millis(150)).await;
    Ok(())
}

async fn drive_wasm_selection_with_jet_browser(page: &Page, root_dir: &Path) -> Result<Value> {
    let targets = page
        .evaluate(WASM_SELECTION_TARGETS_EXPR)
        .await
        .context("resolve WASM selection target coordinates")?;
    if !targets.get("ok").and_then(Value::as_bool).unwrap_or(false) {
        return Err(anyhow!(
            "WASM selection targets were not available: {}",
            serde_json::to_string_pretty(&targets).unwrap_or_else(|_| targets.to_string())
        ));
    }

    let start_x = selection_target_coord(&targets, "start", "x")?;
    let start_y = selection_target_coord(&targets, "start", "y")?;
    let end_x = selection_target_coord(&targets, "end", "x")?;
    let end_y = selection_target_coord(&targets, "end", "y")?;

    page.evaluate(WASM_INSTALL_CLIPBOARD_SPY_EXPR)
        .await
        .context("install clipboard spy before Jet browser selection input")?;
    browser_cli::drag(root_dir, start_x, start_y, end_x, end_y, 8)
        .await
        .context("drive Jet browser forward drag selection")?;
    tokio::time::sleep(Duration::from_millis(150)).await;
    let forward = page
        .evaluate(WASM_SELECTION_READ_EXPR)
        .await
        .context("read WASM forward selection after Jet browser drag")?;

    browser_cli::drag(root_dir, end_x, end_y, start_x, start_y, 8)
        .await
        .context("drive Jet browser reverse drag selection")?;
    tokio::time::sleep(Duration::from_millis(150)).await;
    let reverse_before_copy = page
        .evaluate(WASM_SELECTION_READ_EXPR)
        .await
        .context("read WASM reverse selection after Jet browser drag")?;

    browser_cli::key(root_dir, "c", 2)
        .await
        .context("drive Jet browser Ctrl+C TSV copy")?;
    tokio::time::sleep(Duration::from_millis(150)).await;
    let reverse_after_copy = page
        .evaluate(WASM_SELECTION_READ_EXPR)
        .await
        .context("read WASM selection after Jet browser Ctrl+C")?;

    Ok(with_selection_direction_checks(
        reverse_after_copy,
        &forward,
        &reverse_before_copy,
    ))
}

async fn drive_wasm_scroll_with_jet_browser(page: &Page, root_dir: &Path) -> Result<Value> {
    browser_cli::wheel(root_dir, 100.0, 190.0, 0.0, 660.0)
        .await
        .context("drive Jet browser wheel scroll into large table")?;
    tokio::time::sleep(Duration::from_millis(250)).await;
    page.evaluate(WASM_SCROLL_READ_EXPR)
        .await
        .context("read WASM large-table state after Jet browser wheel")
}

fn selection_direction_check(direction: &str, diagnostics: &Value) -> Value {
    json!({
        "direction": direction,
        "source": diagnostics.get("source").cloned().unwrap_or(Value::Null),
        "selectionFillOps": diagnostics.get("selectionFillOps").cloned().unwrap_or(Value::Null),
        "selectionStrokeOps": diagnostics.get("selectionStrokeOps").cloned().unwrap_or(Value::Null),
        "selectedValues": diagnostics.get("selectedValues").cloned().unwrap_or(Value::Null),
        "selectedTsv": diagnostics.get("selectedTsv").cloned().unwrap_or(Value::Null),
        "keyboardCopiedTsv": diagnostics.get("keyboardCopiedTsv").cloned().unwrap_or(Value::Null),
        "clipboardWriteCount": diagnostics.get("clipboardWriteCount").cloned().unwrap_or(Value::Null),
    })
}

fn with_selection_direction_checks(
    mut diagnostics: Value,
    forward: &Value,
    reverse: &Value,
) -> Value {
    if let Value::Object(map) = &mut diagnostics {
        map.insert(
            "directionChecks".to_string(),
            Value::Array(vec![
                selection_direction_check("forward", forward),
                selection_direction_check("reverse", reverse),
            ]),
        );
    }
    diagnostics
}

async fn page_and_selection_snapshots(
    url: &str,
    artifact_dir: &Path,
    stem: &str,
    jet_browser_root: &Path,
) -> Result<VisualRunSnapshots> {
    let browser = browser_cli::prepare_session(jet_browser_root, url)
        .await
        .context("prepare Jet browser session for visual regression")?;
    let page = browser_cli::attach(jet_browser_root)
        .await
        .context("reattach Jet browser session for visual regression")?;

    wait_for_visual_ready(&page, stem).await?;

    let (body_text, visual_diagnostics) = read_stable_visual_snapshot(&page, stem).await?;
    let (visual_screenshot, visual_screenshot_summary, visual_screenshot_visual_probe) =
        capture_nonblank_visual_screenshot(&page, stem)
            .await
            .with_context(|| format!("capture nonblank {stem} visual screenshot"))?;
    std::fs::write(artifact_dir.join(format!("{stem}.png")), &visual_screenshot)
        .with_context(|| format!("write {stem} visual screenshot"))?;

    let mut selection_diagnostics = if stem == "react-dom" {
        drive_dom_selection_with_jet_browser(&page, jet_browser_root, false)
            .await
            .context("drive React DOM forward selection with Jet browser")?;
        let forward = page
            .evaluate(DOM_SELECTION_READ_EXPR)
            .await
            .context("read React DOM forward selection after Jet browser drag")?;
        drive_dom_selection_with_jet_browser(&page, jet_browser_root, true)
            .await
            .context("drive React DOM reverse selection with Jet browser")?;
        let reverse_before_copy = page
            .evaluate(DOM_SELECTION_READ_EXPR)
            .await
            .context("read React DOM reverse selection after Jet browser drag")?;
        browser_cli::key(jet_browser_root, "c", 2)
            .await
            .context("drive React DOM Ctrl+C with Jet browser")?;
        tokio::time::sleep(Duration::from_millis(150)).await;
        let reverse_after_copy = page
            .evaluate(DOM_SELECTION_READ_EXPR)
            .await
            .context("read React DOM selection after Jet browser key")?;
        with_selection_direction_checks(reverse_after_copy, &forward, &reverse_before_copy)
    } else {
        drive_wasm_selection_with_jet_browser(&page, jet_browser_root)
            .await
            .context("run Jet browser CDP selection interaction probe")?
    };
    let selection_screenshot = page
        .screenshot()
        .await
        .context("capture selection screenshot")?;
    let source = selection_diagnostics
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    std::fs::write(
        artifact_dir.join(format!("{source}-selection.png")),
        &selection_screenshot,
    )
    .context("write selection screenshot")?;
    let selection_screenshot_summary = screenshot_summary_from_png(&selection_screenshot);
    let selection_screenshot_visual_probe = screenshot_visual_probe_from_png(&selection_screenshot);
    let browser_capture = if stem == "jet-wasm" {
        browser_cli::observation_bundle(jet_browser_root, &[])
            .await
            .context("capture Jet WASM observation bundle after selection interaction")?
    } else {
        browser_cli::dom_observation_bundle(jet_browser_root, "#visual-root")
            .await
            .context(
                "capture React DOM Jet browser observation bundle after selection interaction",
            )?
    };
    if stem == "jet-wasm" {
        let scroll_check = drive_wasm_scroll_with_jet_browser(&page, jet_browser_root)
            .await
            .context("run Jet browser wheel scroll probe")?;
        if let Value::Object(map) = &mut selection_diagnostics {
            map.insert("scrollCheck".to_string(), scroll_check);
        }
    }
    let _ = browser.close().await;
    browser_cli::session::clear(jet_browser_root);

    let visual = VisualSnapshot {
        body_text,
        diagnostics: visual_diagnostics,
        browser_capture,
        png: visual_screenshot,
        screenshot_summary: visual_screenshot_summary,
        screenshot_visual_probe: visual_screenshot_visual_probe,
        browser_stderr: String::new(),
    };

    let selection = SelectionInteractionSnapshot {
        diagnostics: selection_diagnostics,
        png: selection_screenshot,
        screenshot_summary: selection_screenshot_summary,
        screenshot_visual_probe: selection_screenshot_visual_probe,
        browser_stderr: String::new(),
    };

    Ok(VisualRunSnapshots { visual, selection })
}

async fn read_stable_visual_snapshot(page: &Page, stem: &str) -> Result<(String, Value)> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let body_text = page
            .evaluate(VISUAL_TEXT_EXPR)
            .await?
            .as_str()
            .unwrap_or_default()
            .to_string();
        let diagnostics = page.evaluate(VISUAL_DIAGNOSTICS_EXPR).await?;
        if visual_table_text_matches(&body_text) && visual_table_diagnostics_match(&diagnostics) {
            return Ok((body_text, diagnostics));
        }
        if Instant::now() >= deadline {
            tracing::warn!(
                target: "jet::mui_visual_regression",
                stem,
                body = %body_text,
                diagnostics = %diagnostics,
                "visual snapshot did not stabilize before capture"
            );
            return Ok((body_text, diagnostics));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn capture_nonblank_visual_screenshot(
    page: &Page,
    stem: &str,
) -> Result<(Vec<u8>, Value, Value)> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        page.bring_to_front()
            .await
            .with_context(|| format!("bring {stem} target to front before visual screenshot"))?;
        page.session()
            .send(
                "Input.dispatchMouseEvent",
                json!({
                    "type": "mouseMoved",
                    "x": 1.0,
                    "y": 1.0,
                    "buttons": 0,
                }),
            )
            .await
            .with_context(|| format!("kick {stem} compositor before visual screenshot"))?;
        let _ = page.evaluate(WAIT_FOR_BROWSER_PAINT_EXPR).await;
        let screenshot = page
            .screenshot()
            .await
            .with_context(|| format!("capture {stem} visual screenshot"))?;
        let summary = screenshot_summary_from_png(&screenshot);
        let probe = screenshot_visual_probe_from_png(&screenshot);
        let foreground_count = probe
            .get("foregroundCount")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let non_white = probe.get("nonWhite").and_then(Value::as_u64).unwrap_or(0);
        if foreground_count >= VISUAL_SCREENSHOT_MIN_FOREGROUND_COUNT
            && non_white >= VISUAL_SCREENSHOT_MIN_NON_WHITE
        {
            return Ok((screenshot, summary, probe));
        }

        if Instant::now() >= deadline {
            let diagnostics = page
                .evaluate(VISUAL_DIAGNOSTICS_EXPR)
                .await
                .unwrap_or(Value::Null);
            let browser_state = page
                .evaluate(
                    r#"(() => ({
                      readyState: document.readyState,
                      visibilityState: document.visibilityState,
                      hidden: document.hidden,
                      hasFocus: document.hasFocus(),
                      activeElement: document.activeElement
                        ? {
                            tag: document.activeElement.tagName,
                            id: document.activeElement.id || '',
                          }
                        : null,
                      viewport: {
                        innerWidth: window.innerWidth,
                        innerHeight: window.innerHeight,
                        devicePixelRatio: window.devicePixelRatio,
                      },
                      bodyRect: (() => {
                        const rect = document.body?.getBoundingClientRect?.();
                        return rect
                          ? { x: rect.x, y: rect.y, w: rect.width, h: rect.height }
                          : null;
                      })(),
                    }))()"#,
                )
                .await
                .unwrap_or(Value::Null);
            let layout_metrics = page
                .session()
                .send("Page.getLayoutMetrics", json!({}))
                .await
                .unwrap_or(Value::Null);
            let last_failure = json!({
                "summary": summary,
                "probe": probe,
                "diagnostics": diagnostics,
                "browser_state": browser_state,
                "layout_metrics": layout_metrics,
            });
            return Err(anyhow!(
                "{stem} visual screenshot stayed blank or near-blank before phash comparison.\nlast_capture={}",
                json_for_failure(&last_failure)
            ));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn visual_table_text_matches(text: &str) -> bool {
    text.contains(VISUAL_TABLE_TITLE)
        && text.contains("cell 0")
        && text.contains(VISUAL_TABLE_LAST_CELL)
}

fn visual_table_diagnostics_match(diagnostics: &Value) -> bool {
    let Some(table) = diagnostics.get("table") else {
        return false;
    };
    table
        .get("visualRoot")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        && table.get("tableRows").and_then(Value::as_u64).unwrap_or(0) == VISUAL_TABLE_EXPECTED_ROWS
        && table.get("tableCells").and_then(Value::as_u64).unwrap_or(0)
            == VISUAL_TABLE_EXPECTED_CELLS
        && table
            .get("cellLabelTexts")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == VISUAL_TABLE_EXPECTED_CELLS
        && table
            .get("hasTitle")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        && table
            .get("hasFirstCell")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        && table
            .get("hasLastCell")
            .and_then(Value::as_bool)
            .unwrap_or(false)
}

fn assert_wasm_dom_shell_has_no_table_fixture(diagnostics: &Value) {
    let dom_shell = diagnostics
        .get("domShell")
        .expect("visual diagnostics must include domShell");
    assert!(
        dom_shell
            .get("jetCanvas")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        "Jet WASM page must expose only the canvas shell as the real DOM render target: {dom_shell:?}"
    );
    assert!(
        !dom_shell
            .get("visualRoot")
            .and_then(Value::as_bool)
            .unwrap_or(true)
            && !dom_shell
                .get("largeTable")
                .and_then(Value::as_bool)
                .unwrap_or(true)
            && dom_shell
                .get("tableElements")
                .and_then(Value::as_u64)
                .unwrap_or(u64::MAX)
                == 0
            && dom_shell
                .get("trElements")
                .and_then(Value::as_u64)
                .unwrap_or(u64::MAX)
                == 0
            && dom_shell
                .get("tdElements")
                .and_then(Value::as_u64)
                .unwrap_or(u64::MAX)
                == 0
            && !dom_shell
                .get("bodyHasCell0")
                .and_then(Value::as_bool)
                .unwrap_or(true)
            && !dom_shell
                .get("bodyHasLastCell")
                .and_then(Value::as_bool)
                .unwrap_or(true),
        "Jet WASM page must not pass by rendering the 100x100 table as real DOM: {dom_shell:?}"
    );
}

fn selection_direction_checks_match(diagnostics: &Value, expected_tsv: &str) -> bool {
    let Some(checks) = diagnostics.get("directionChecks").and_then(Value::as_array) else {
        return false;
    };
    let direction_matches = |direction: &str| {
        checks.iter().any(|check| {
            check.get("direction").and_then(Value::as_str) == Some(direction)
                && check
                    .get("selectionFillOps")
                    .and_then(Value::as_u64)
                    .is_some_and(|count| count == 4)
                && check.get("selectedTsv").and_then(Value::as_str) == Some(expected_tsv)
        })
    };
    direction_matches("forward") && direction_matches("reverse")
}

fn assert_wasm_clipboard_status_fulfilled(status: &Value, context: &str) {
    assert_eq!(
        status.get("clipboardWriteState").and_then(Value::as_str),
        Some("fulfilled"),
        "{context} must prove the clipboard Promise fulfilled: {status:?}"
    );
    assert!(
        status
            .get("clipboardWriteSuccessCount")
            .and_then(Value::as_u64)
            .is_some_and(|count| count >= 1),
        "{context} must prove at least one successful clipboard write: {status:?}"
    );
    assert_eq!(
        status
            .get("clipboardWriteErrorCount")
            .and_then(Value::as_u64),
        Some(0),
        "{context} must not report clipboard write errors: {status:?}"
    );
    assert_eq!(
        status.get("clipboardWriteError").and_then(Value::as_str),
        Some(""),
        "{context} must keep clipboardWriteError empty after success: {status:?}"
    );
}

fn assert_shared_selection_interaction(snapshot: &SelectionInteractionSnapshot) {
    let diagnostics = &snapshot.diagnostics;
    let status = diagnostics.get("statusAfter").unwrap_or(&Value::Null);
    let expected_tsv = "cell 0\tcell 1\ncell 100\tcell 101";
    let ok = diagnostics
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let source = diagnostics.get("source").and_then(Value::as_str);
    let cells_found = diagnostics
        .get("cellsFound")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let selection_fill_ops = diagnostics
        .get("selectionFillOps")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let selection_stroke_ops = diagnostics
        .get("selectionStrokeOps")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let clipboard_write_count = diagnostics
        .get("clipboardWriteCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let keyboard_clipboard_write_count = diagnostics
        .get("keyboardClipboardWriteCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let status_after_keyboard = diagnostics
        .get("statusAfterKeyboard")
        .unwrap_or(&Value::Null);
    let glyph_count = status
        .get("lastTextGlyphCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let unsupported_count = status
        .get("lastUnsupportedCount")
        .and_then(Value::as_u64)
        .unwrap_or(u64::MAX);
    let selection_count = status
        .get("lastSelectionCellCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let wasm_status_ok = match source {
        Some("jet-wasm") => {
            status
                .get("bridgeMode")
                .and_then(Value::as_str)
                .is_some_and(|mode| mode == "text")
                && glyph_count > 0
                && status
                    .get("textAtlasMode")
                    .and_then(Value::as_str)
                    .is_some_and(|mode| mode == "glyph-atlas")
                && status
                    .get("lastTextAtlasUploadCount")
                    .and_then(Value::as_u64)
                    .is_some_and(|count| count >= 1)
                && status
                    .get("lastTextAtlasWidth")
                    .and_then(Value::as_u64)
                    .is_some_and(|width| width > 1)
                && status
                    .get("lastTextAtlasHeight")
                    .and_then(Value::as_u64)
                    .is_some_and(|height| height > 1)
                && status
                    .get("lastTextAtlasNonZeroAlphaCount")
                    .and_then(Value::as_u64)
                    .is_some_and(|count| count > 0)
                && unsupported_count == 0
        }
        Some("dom") => true,
        _ => false,
    };
    let keyboard_copy_ok = match source {
        Some("jet-wasm") => {
            assert_wasm_clipboard_status_fulfilled(
                status_after_keyboard,
                "Jet WASM shared selection probe",
            );
            status_after_keyboard
                .get("copyCount")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                >= 1
                && status_after_keyboard
                    .get("lastCopiedTsv")
                    .and_then(Value::as_str)
                    .is_some_and(|tsv| tsv == expected_tsv)
                && status_after_keyboard
                    .get("lastSelectionCellCount")
                    .and_then(Value::as_u64)
                    .is_some_and(|count| count == 4)
                && diagnostics
                    .get("keyboardCopiedTsv")
                    .and_then(Value::as_str)
                    .is_some_and(|tsv| tsv == expected_tsv)
                && keyboard_clipboard_write_count >= 1
        }
        Some("dom") => {
            diagnostics
                .get("keyboardCopiedTsv")
                .and_then(Value::as_str)
                .is_some_and(|tsv| tsv == expected_tsv)
                && keyboard_clipboard_write_count >= 1
        }
        _ => false,
    };

    assert!(
        ok
            && cells_found >= 4
            && wasm_status_ok
            && keyboard_copy_ok
            && selection_fill_ops == 4
            && (selection_stroke_ops == 0 || selection_stroke_ops == 4)
            && diagnostics
                .get("selectedTsv")
                .and_then(Value::as_str)
                .is_some_and(|tsv| tsv == expected_tsv)
            && selection_direction_checks_match(diagnostics, expected_tsv)
            && match source {
                Some("jet-wasm") | Some("dom") => {
                    clipboard_write_count >= 1
                        && diagnostics
                            .get("copiedTsv")
                            .and_then(Value::as_str)
                            .is_some_and(|tsv| tsv == expected_tsv)
                }
                _ => false,
            },
        "Shared DOM/WASM selection interaction did not meet the gate.\ndiag={}\nscreenshot={}\nvisual_probe={}\npng_bytes={}\nbrowser_stderr={}\nselection_count_status={selection_count}",
        serde_json::to_string_pretty(diagnostics).unwrap_or_else(|_| diagnostics.to_string()),
        serde_json::to_string_pretty(&snapshot.screenshot_summary)
            .unwrap_or_else(|_| snapshot.screenshot_summary.to_string()),
        serde_json::to_string_pretty(&snapshot.screenshot_visual_probe)
            .unwrap_or_else(|_| snapshot.screenshot_visual_probe.to_string()),
        snapshot.png.len(),
        truncate_for_failure(&snapshot.browser_stderr)
    );
}

fn assert_wasm_scroll_interaction(snapshot: &SelectionInteractionSnapshot) {
    let scroll_check = snapshot
        .diagnostics
        .get("scrollCheck")
        .expect("Jet WASM selection diagnostics must include scrollCheck");
    assert!(
        scroll_check
            .get("ok")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            && scroll_check
                .get("targetValuesVisible")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            && scroll_check
                .get("status")
                .and_then(|status| status.get("scrollTop"))
                .and_then(Value::as_f64)
                .is_some_and(|scroll_top| scroll_top >= 660.0),
        "Jet WASM large table must wheel-scroll far rows into the visible layout tree: {}",
        serde_json::to_string_pretty(scroll_check).unwrap_or_else(|_| scroll_check.to_string())
    );
}

fn count_jet_intrinsic_tag(node: &Value, tag: &str) -> u64 {
    let self_count = (node.get("kind").and_then(Value::as_str) == Some("intrinsic")
        && node.get("tag").and_then(Value::as_str) == Some(tag)) as u64;
    self_count
        + node
            .get("children")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|child| count_jet_intrinsic_tag(child, tag))
            .sum::<u64>()
}

fn count_jet_text(node: &Value, expected: &str) -> u64 {
    let self_count = (node.get("kind").and_then(Value::as_str) == Some("text")
        && node
            .get("text")
            .and_then(Value::as_str)
            .is_some_and(|text| text.trim() == expected)) as u64;
    self_count
        + node
            .get("children")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|child| count_jet_text(child, expected))
            .sum::<u64>()
}

fn assert_webgpu_timing_status_observable(status: &Value) {
    let timing_enabled = status
        .get("gpuTimingEnabled")
        .and_then(Value::as_bool)
        .expect("Jet browser capture must include gpuTimingEnabled boolean");
    let sample_ready = status
        .get("gpuTimingSampleReady")
        .and_then(Value::as_bool)
        .expect("Jet browser capture must include gpuTimingSampleReady boolean");
    let last_ms = status
        .get("lastFrameGpuMs")
        .expect("Jet browser capture must include lastFrameGpuMs");
    if !timing_enabled {
        assert!(
            !sample_ready,
            "GPU timing cannot be sample-ready when TIMESTAMP_QUERY is disabled: {status:?}"
        );
    }
    if sample_ready {
        let ms = last_ms
            .as_f64()
            .expect("lastFrameGpuMs must be numeric when gpuTimingSampleReady=true");
        assert!(
            ms.is_finite() && ms >= 0.0,
            "lastFrameGpuMs must be a finite non-negative duration: {status:?}"
        );
    } else {
        assert!(
            last_ms.is_null(),
            "lastFrameGpuMs must be null until a GPU timing sample is ready: {status:?}"
        );
    }
}

fn assert_wasm_browser_capture_matches_table_and_selection(capture: &Value) {
    let expected_tsv = "cell 0\tcell 1\ncell 100\tcell 101";
    assert_eq!(
        capture.get("schema_version").and_then(Value::as_str),
        Some("jet.browser.observation.v1"),
        "Jet browser capture must use the wasm observation bundle schema: {capture}"
    );
    let build_artifact = capture
        .get("build_artifact")
        .expect("Jet browser capture must include build_artifact");
    assert_eq!(
        build_artifact.get("present").and_then(Value::as_bool),
        Some(true),
        "Jet browser capture must include dist/jet-target.json build evidence: {capture}"
    );
    let target_manifest = build_artifact
        .get("manifest")
        .expect("Jet browser capture must include build_artifact.manifest");
    assert_eq!(
        target_manifest
            .pointer("/build/mode")
            .and_then(Value::as_str),
        Some("dev"),
        "Jet browser capture must prove the dev-profile WASM build mode: {target_manifest:?}"
    );
    assert_eq!(
        target_manifest
            .pointer("/build/tsx_lowering")
            .and_then(Value::as_str),
        Some(wasm_manifest::TSX_LOWERING_STRICT),
        "Jet browser capture must prove strict TSX lowering: {target_manifest:?}"
    );
    let screenshot_visual_probe = capture
        .get("screenshot_visual_probe")
        .expect("Jet browser capture must include screenshot_visual_probe");
    assert!(
        screenshot_visual_probe.get("error").is_none(),
        "Jet browser capture screenshot visual probe must not fail: {screenshot_visual_probe:?}"
    );
    assert_eq!(
        screenshot_visual_probe
            .get("schema_version")
            .and_then(Value::as_str),
        Some("jet.browser.screenshot_visual_probe.v1"),
        "Jet browser capture screenshot probe schema must be stable: {screenshot_visual_probe:?}"
    );
    assert!(
        screenshot_visual_probe
            .get("hash")
            .and_then(Value::as_str)
            .is_some_and(|hash| hash.len() == 16 && hash.chars().all(|ch| ch.is_ascii_hexdigit()))
            && screenshot_visual_probe
                .get("hashOnes")
                .and_then(Value::as_u64)
                .is_some_and(|ones| ones <= 64),
        "Jet browser capture screenshot probe must include a comparable 64-bit visual hash: {screenshot_visual_probe:?}"
    );
    assert!(
        screenshot_visual_probe
            .get("pngByteLen")
            .and_then(Value::as_u64)
            .is_some_and(|count| count > 1_000)
            && screenshot_visual_probe
                .get("nonTransparent")
                .and_then(Value::as_u64)
                .is_some_and(|count| count > 0)
            && screenshot_visual_probe
                .get("nonWhite")
                .and_then(Value::as_u64)
                .is_some_and(|count| count > 0)
            && screenshot_visual_probe
                .get("nonBlack")
                .and_then(Value::as_u64)
                .is_some_and(|count| count > 0)
            && screenshot_visual_probe
                .get("uniqueBuckets")
                .and_then(Value::as_u64)
                .is_some_and(|count| count >= 2),
        "Jet browser capture must include nonblank screenshot pixel evidence: {screenshot_visual_probe:?}"
    );
    let canvas_visual_probe = capture
        .get("runtime")
        .and_then(|runtime| runtime.get("canvas_visual_probe"))
        .expect("Jet browser capture must include runtime.canvas_visual_probe");
    assert!(
        canvas_visual_probe.get("error").is_none(),
        "Jet browser capture canvas visual probe must not fail: {canvas_visual_probe:?}"
    );
    let element_tree = capture
        .get("element_tree")
        .expect("Jet browser capture must include element_tree");
    assert_eq!(
        count_jet_intrinsic_tag(element_tree, "tr"),
        VISUAL_TABLE_EXPECTED_ROWS,
        "Jet browser capture must prove the 100-row table structure: {capture}"
    );
    assert_eq!(
        count_jet_intrinsic_tag(element_tree, "td"),
        VISUAL_TABLE_EXPECTED_CELLS,
        "Jet browser capture must prove the 10,000-cell table structure: {capture}"
    );
    assert_eq!(
        count_jet_text(element_tree, "cell"),
        VISUAL_TABLE_EXPECTED_CELLS,
        "Jet browser capture must prove every cell label reached the WASM element tree: {capture}"
    );
    assert!(
        count_jet_text(element_tree, "0") >= 1 && count_jet_text(element_tree, "9999") >= 1,
        "Jet browser capture must include first and last table values: {capture}"
    );

    let status = capture
        .get("runtime")
        .and_then(|runtime| runtime.get("webgpu_status"))
        .expect("Jet browser capture must include runtime.webgpu_status after selection");
    assert_eq!(
        status.get("bridgeMode").and_then(Value::as_str),
        Some("text"),
        "Jet browser capture must prove WebGPU text bridge mode: {status:?}"
    );
    assert_eq!(
        status.get("textAtlasMode").and_then(Value::as_str),
        Some("glyph-atlas"),
        "Jet browser capture must prove real glyph atlas usage: {status:?}"
    );
    assert!(
        status
            .get("lastTextGlyphCount")
            .and_then(Value::as_u64)
            .is_some_and(|count| count > 0),
        "Jet browser capture must include non-empty glyph count: {status:?}"
    );
    assert_webgpu_timing_status_observable(status);
    assert_eq!(
        status.get("selectionRange").and_then(Value::as_str),
        Some("0,0:1,1"),
        "Jet browser capture must prove the selected cell range: {status:?}"
    );
    assert_eq!(
        status.get("lastSelectionCellCount").and_then(Value::as_u64),
        Some(4),
        "Jet browser capture must prove four selected cells: {status:?}"
    );
    assert_eq!(
        status.get("lastCopiedTsv").and_then(Value::as_str),
        Some(expected_tsv),
        "Jet browser capture must prove TSV copy content: {status:?}"
    );
    assert!(
        status
            .get("copyCount")
            .and_then(Value::as_u64)
            .is_some_and(|count| count >= 1),
        "Jet browser capture must prove the copy shortcut ran: {status:?}"
    );
    assert_wasm_clipboard_status_fulfilled(status, "Jet browser capture");
}

fn count_selection_blue_pixels(bytes: &[u8]) -> u64 {
    let image = image::load_from_memory(bytes)
        .unwrap_or_else(|err| panic!("decode selection screenshot PNG: {err}"))
        .to_rgba8();
    image
        .pixels()
        .filter(|pixel| {
            let [r, g, b, a] = pixel.0;
            if a == 0 {
                return false;
            }
            let dark_selection_stroke = b >= 170 && (70..=190).contains(&g) && r <= 150;
            let light_selection_fill =
                b >= 240 && g >= 215 && (180..=245).contains(&r) && b > r + 6;
            dark_selection_stroke || light_selection_fill
        })
        .count() as u64
}

fn assert_selection_screenshot_has_visible_delta(
    label: &str,
    before_png: &[u8],
    after_png: &[u8],
    before_summary: &Value,
    after_summary: &Value,
    before_probe: &Value,
    after_probe: &Value,
) {
    let before_blue = count_selection_blue_pixels(before_png);
    let after_blue = count_selection_blue_pixels(after_png);
    assert!(
        after_blue >= 1_000 && after_blue > before_blue + 500,
        "{label} selection screenshot must visibly add blue selected-cell pixels.\nblue_before={before_blue}\nblue_after={after_blue}\nbefore_summary={}\nafter_summary={}\nbefore_probe={}\nafter_probe={}",
        serde_json::to_string_pretty(before_summary).unwrap_or_else(|_| before_summary.to_string()),
        serde_json::to_string_pretty(after_summary).unwrap_or_else(|_| after_summary.to_string()),
        serde_json::to_string_pretty(before_probe).unwrap_or_else(|_| before_probe.to_string()),
        serde_json::to_string_pretty(after_probe).unwrap_or_else(|_| after_probe.to_string()),
    );
}

fn write_visual_mismatch_artifacts(
    dom_png: &[u8],
    wasm_png: &[u8],
    dom_selection_png: &[u8],
    wasm_selection_png: &[u8],
    diagnostics: &Value,
) -> Result<PathBuf> {
    let artifact_dir = std::env::temp_dir().join("jet-mui-visual-regression");
    std::fs::create_dir_all(&artifact_dir)?;
    std::fs::write(artifact_dir.join("react-dom.png"), dom_png)?;
    std::fs::write(artifact_dir.join("jet-wasm.png"), wasm_png)?;
    std::fs::write(
        artifact_dir.join("react-dom-selection.png"),
        dom_selection_png,
    )?;
    std::fs::write(
        artifact_dir.join("jet-wasm-selection.png"),
        wasm_selection_png,
    )?;
    std::fs::write(
        artifact_dir.join("diagnostics.json"),
        serde_json::to_vec_pretty(diagnostics)?,
    )?;
    Ok(artifact_dir)
}

fn write_latest_visual_diagnostics(artifact_dir: &Path, diagnostics: &Value) {
    let path = artifact_dir.join("latest-diagnostics.json");
    let bytes =
        serde_json::to_vec_pretty(diagnostics).expect("serialize latest visual diagnostics");
    std::fs::write(&path, bytes).expect("write latest visual diagnostics");
}

fn write_surface_visual_diagnostics(artifact_dir: &Path, surface: &str, run: &VisualRunSnapshots) {
    write_latest_visual_diagnostics(
        artifact_dir,
        &json!({
            "stage": "surface-complete",
            "surface": surface,
            "visual": {
                "body": &run.visual.body_text,
                "diagnostics": &run.visual.diagnostics,
                "browser_capture": &run.visual.browser_capture,
                "screenshot": &run.visual.screenshot_summary,
                "visual_probe": &run.visual.screenshot_visual_probe,
            },
            "selection": {
                "diagnostics": &run.selection.diagnostics,
                "screenshot": &run.selection.screenshot_summary,
                "visual_probe": &run.selection.screenshot_visual_probe,
            }
        }),
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mui_visual_fixture_renders_on_react_dom_and_jet_wasm() {
    common::require_full_wasm_e2e_env();

    let fixture = workspace_root().join("examples/mui-visual-demo");
    let artifact_dir = std::env::temp_dir().join("jet-mui-visual-regression");
    std::fs::create_dir_all(&artifact_dir).expect("create visual artifact dir");

    if !missing_mui_install_deps(&fixture).is_empty() {
        run_jet_install(&fixture).expect("jet install for MUI fixture");
    }
    require_mui_install(&fixture);
    assert_wasm_demo_uses_strict_lowering(&fixture)
        .expect("MUI visual WASM fixture must not use compatibility lowering");

    let dom_port = free_port().await;
    let dom_url = format!("http://127.0.0.1:{dom_port}/");
    let mut dom_server = spawn_jet_dev(&fixture, dom_port, false).expect("spawn jet dev");
    wait_for_http(&dom_url).await.expect("React DOM dev server");

    let dom_run = page_and_selection_snapshots(&dom_url, &artifact_dir, "react-dom", &fixture)
        .await
        .expect("React DOM page and shared selection snapshots");
    write_surface_visual_diagnostics(&artifact_dir, "react-dom", &dom_run);
    assert_shared_selection_interaction(&dom_run.selection);
    let dom_selection = dom_run.selection;
    let dom_server_stderr =
        shutdown_jet_dev(&fixture, dom_port, &mut dom_server).expect("jet dev shutdown for DOM");
    let dom_server_stderr = truncate_for_failure(&dom_server_stderr);
    let dom_snapshot = dom_run.visual;
    let dom_text = dom_snapshot.body_text;
    let dom_diag = dom_snapshot.diagnostics;
    let dom_capture = dom_snapshot.browser_capture;
    let dom_png = dom_snapshot.png;
    let dom_screenshot = dom_snapshot.screenshot_summary;
    let dom_visual_probe = dom_snapshot.screenshot_visual_probe;
    let dom_selection_diag = dom_selection.diagnostics;
    let dom_selection_png = dom_selection.png;
    let dom_selection_screenshot = dom_selection.screenshot_summary;
    let dom_selection_visual_probe = dom_selection.screenshot_visual_probe;
    let dom_browser_stderr = truncate_for_failure(&dom_snapshot.browser_stderr);

    assert!(
        visual_table_text_matches(&dom_text) && visual_table_diagnostics_match(&dom_diag),
        "React DOM MUI table fixture rendered blank or incomplete.\nbody={dom_text:?}\ndiag={}\nbrowser_capture={}\njet_dev_stderr={dom_server_stderr}\njet_browser_stderr={dom_browser_stderr}",
        json_for_failure(&dom_diag),
        json_for_failure(&dom_capture)
    );

    let wasm_port = free_port().await;
    let wasm_url = format!("http://127.0.0.1:{wasm_port}/");
    let mut wasm_server = spawn_jet_dev(&fixture, wasm_port, true)
        .expect("spawn jet dev --wasm --debug for MUI visual fixture");
    wait_for_http(&wasm_url).await.expect("Jet WASM dev server");
    assert_wasm_manifest_uses_strict_lowering(&fixture, "dev", "`jet dev --wasm --debug`")
        .expect("MUI visual WASM dev fixture must record strict lowering");

    let wasm_run = page_and_selection_snapshots(&wasm_url, &artifact_dir, "jet-wasm", &fixture)
        .await
        .expect("Jet WASM page and shared Jet browser selection snapshots");
    write_surface_visual_diagnostics(&artifact_dir, "jet-wasm", &wasm_run);
    assert_shared_selection_interaction(&wasm_run.selection);
    assert_wasm_scroll_interaction(&wasm_run.selection);
    assert_wasm_browser_capture_matches_table_and_selection(&wasm_run.visual.browser_capture);
    let wasm_selection = wasm_run.selection;
    let wasm_server_stderr =
        shutdown_jet_dev(&fixture, wasm_port, &mut wasm_server).expect("jet dev shutdown for WASM");
    assert_no_wasm_compat_lowering_log(&wasm_server_stderr, "`jet dev --wasm --debug` stderr");
    let wasm_server_stderr = truncate_for_failure(&wasm_server_stderr);
    let wasm_snapshot = wasm_run.visual;
    let wasm_text = wasm_snapshot.body_text;
    let wasm_diag = wasm_snapshot.diagnostics;
    let wasm_capture = wasm_snapshot.browser_capture;
    let wasm_png = wasm_snapshot.png;
    let wasm_screenshot = wasm_snapshot.screenshot_summary;
    let wasm_visual_probe = wasm_snapshot.screenshot_visual_probe;
    let wasm_selection_diag = wasm_selection.diagnostics;
    let wasm_selection_png = wasm_selection.png;
    let wasm_selection_screenshot = wasm_selection.screenshot_summary;
    let wasm_selection_visual_probe = wasm_selection.screenshot_visual_probe;
    let wasm_browser_stderr = truncate_for_failure(&wasm_snapshot.browser_stderr);

    write_latest_visual_diagnostics(
        &artifact_dir,
        &json!({
            "react_dom": {
                "body": &dom_text,
                "diagnostics": &dom_diag,
                "browser_capture": &dom_capture,
                "screenshot": &dom_screenshot,
                "visual_probe": &dom_visual_probe,
                "selection_diagnostics": &dom_selection_diag,
                "selection_screenshot": &dom_selection_screenshot,
                "selection_visual_probe": &dom_selection_visual_probe,
                "jet_browser_stderr": &dom_browser_stderr,
                "jet_dev_stderr": &dom_server_stderr,
            },
            "jet_wasm": {
                "body": &wasm_text,
                "diagnostics": &wasm_diag,
                "browser_capture": &wasm_capture,
                "screenshot": &wasm_screenshot,
                "visual_probe": &wasm_visual_probe,
                "selection_diagnostics": &wasm_selection_diag,
                "selection_screenshot": &wasm_selection_screenshot,
                "selection_visual_probe": &wasm_selection_visual_probe,
                "jet_browser_stderr": &wasm_browser_stderr,
                "jet_dev_stderr": &wasm_server_stderr,
            }
        }),
    );

    assert!(
        visual_table_text_matches(&wasm_text) && visual_table_diagnostics_match(&wasm_diag),
        "Jet WASM MUI table fixture rendered blank or incomplete.\nbody={wasm_text:?}\ndiag={}\nbrowser_capture={}\njet_dev_stderr={wasm_server_stderr}\njet_browser_stderr={wasm_browser_stderr}",
        json_for_failure(&wasm_diag),
        json_for_failure(&wasm_capture)
    );
    assert_wasm_dom_shell_has_no_table_fixture(&wasm_diag);
    assert_selection_screenshot_has_visible_delta(
        "React DOM",
        &dom_png,
        &dom_selection_png,
        &dom_screenshot,
        &dom_selection_screenshot,
        &dom_visual_probe,
        &dom_selection_visual_probe,
    );
    assert_selection_screenshot_has_visible_delta(
        "Jet WASM",
        &wasm_png,
        &wasm_selection_png,
        &wasm_screenshot,
        &wasm_selection_screenshot,
        &wasm_visual_probe,
        &wasm_selection_visual_probe,
    );

    let screenshot_summary_match = screenshot_summaries_match(&dom_screenshot, &wasm_screenshot);
    let phash_distance = screenshot_phash_hamming_distance(&dom_visual_probe, &wasm_visual_probe);
    let phash_match = screenshot_phashes_match(&dom_visual_probe, &wasm_visual_probe);
    if !phash_match {
        let failed_gates = vec!["foreground-mask-phash"];
        let diagnostics = json!({
            "comparison": {
                "failed_gates": failed_gates.clone(),
                "phash_distance": phash_distance,
                "phash_hamming_tolerance": common::react_oracle::SCREENSHOT_PHASH_HAMMING_TOLERANCE,
                "screenshot_summary_match": screenshot_summary_match,
                "phash_match": phash_match,
            },
            "react_dom": {
                "body": dom_text,
                "diagnostics": dom_diag,
                "browser_capture": dom_capture,
                "screenshot": dom_screenshot,
                "visual_probe": dom_visual_probe,
                "jet_browser_stderr": dom_browser_stderr,
                "jet_dev_stderr": dom_server_stderr,
            },
            "jet_wasm": {
                "body": wasm_text,
                "diagnostics": wasm_diag,
                "browser_capture": wasm_capture,
                "screenshot": wasm_screenshot,
                "visual_probe": wasm_visual_probe,
                "jet_browser_stderr": wasm_browser_stderr,
                "jet_dev_stderr": wasm_server_stderr,
            }
        });
        let artifact_dir = write_visual_mismatch_artifacts(
            &dom_png,
            &wasm_png,
            &dom_selection_png,
            &wasm_selection_png,
            &diagnostics,
        )
        .ok();
        panic!(
            "{}\nfailed_gates={:?}\npHash distance={:?}; tolerance={}\nartifacts={}",
            screenshot_phash_diff_message(
                "mui-visual-demo",
                "initial-foreground-mask-phash",
                &diagnostics["react_dom"]["visual_probe"],
                &diagnostics["jet_wasm"]["visual_probe"],
                phash_distance,
            ),
            failed_gates,
            phash_distance,
            common::react_oracle::SCREENSHOT_PHASH_HAMMING_TOLERANCE,
            artifact_dir
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<failed to write /tmp artifacts>".to_string())
        );
    }

    let selection_summary_match =
        screenshot_summaries_match(&dom_selection_screenshot, &wasm_selection_screenshot);
    let selection_phash_distance = screenshot_phash_hamming_distance(
        &dom_selection_visual_probe,
        &wasm_selection_visual_probe,
    );
    let selection_phash_match =
        screenshot_phashes_match(&dom_selection_visual_probe, &wasm_selection_visual_probe);
    if !selection_phash_match {
        let failed_gates = vec!["selection-foreground-mask-phash"];
        let diagnostics = json!({
            "comparison": {
                "failed_gates": failed_gates.clone(),
                "phash_distance": selection_phash_distance,
                "phash_hamming_tolerance": common::react_oracle::SCREENSHOT_PHASH_HAMMING_TOLERANCE,
                "screenshot_summary_match": selection_summary_match,
                "phash_match": selection_phash_match,
            },
            "react_dom": {
                "body": dom_text,
                "diagnostics": dom_diag,
                "browser_capture": dom_capture,
                "screenshot": dom_screenshot,
                "visual_probe": dom_visual_probe,
                "selection_screenshot": dom_selection_screenshot,
                "selection_visual_probe": dom_selection_visual_probe,
                "jet_browser_stderr": dom_browser_stderr,
                "jet_dev_stderr": dom_server_stderr,
            },
            "jet_wasm": {
                "body": wasm_text,
                "diagnostics": wasm_diag,
                "browser_capture": wasm_capture,
                "screenshot": wasm_screenshot,
                "visual_probe": wasm_visual_probe,
                "selection_screenshot": wasm_selection_screenshot,
                "selection_visual_probe": wasm_selection_visual_probe,
                "jet_browser_stderr": wasm_browser_stderr,
                "jet_dev_stderr": wasm_server_stderr,
            }
        });
        let artifact_dir = write_visual_mismatch_artifacts(
            &dom_png,
            &wasm_png,
            &dom_selection_png,
            &wasm_selection_png,
            &diagnostics,
        )
        .ok();
        panic!(
            "{}\nfailed_gates={:?}\npHash distance={:?}; tolerance={}\nartifacts={}",
            screenshot_phash_diff_message(
                "mui-visual-demo",
                "selection-foreground-mask-phash",
                &diagnostics["react_dom"]["selection_visual_probe"],
                &diagnostics["jet_wasm"]["selection_visual_probe"],
                selection_phash_distance,
            ),
            failed_gates,
            selection_phash_distance,
            common::react_oracle::SCREENSHOT_PHASH_HAMMING_TOLERANCE,
            artifact_dir
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<failed to write /tmp artifacts>".to_string())
        );
    }
}
// CODEGEN-END
