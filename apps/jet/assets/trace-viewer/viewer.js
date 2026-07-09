// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-assets-trace-viewer.md#logic
// CODEGEN-BEGIN
/**
 * jet trace viewer — vanilla JS, no framework dependencies.
 *
 * Fetches /trace.json (TraceManifest), renders step timeline on the left,
 * and detail tabs (DOM snapshot, screenshot, network, console) on the right.
 *
 * @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R7
 * @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R8
 * @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R9
 */

(function () {
  "use strict";

  // ── State ──────────────────────────────────────────────────────────────────

  /** @type {{ manifest: import('./trace').TraceManifest | null, selectedStepId: number | null }} */
  const state = {
    manifest: null,
    selectedStepId: null,
    activeTab: "snapshot",
  };

  // ── DOM refs ───────────────────────────────────────────────────────────────

  const $ = (id) => document.getElementById(id);

  // ── Init ───────────────────────────────────────────────────────────────────

  document.addEventListener("DOMContentLoaded", () => {
    loadTrace();
  });

  async function loadTrace() {
    const loading = $("loading");
    const app = $("app");

    try {
      const resp = await fetch("/trace.json");
      if (!resp.ok) {
        throw new Error(`/trace.json responded ${resp.status}`);
      }
      state.manifest = await resp.json();
      if (loading) loading.remove();
      app.style.display = "flex";
      renderAll();
    } catch (err) {
      if (loading) loading.textContent = `Failed to load trace: ${err.message}`;
      console.error(err);
    }
  }

  // ── Render ─────────────────────────────────────────────────────────────────

  function renderAll() {
    renderHeader();
    renderTimeline();
    renderTabs();
    // Auto-select first action step if any.
    const firstAction = state.manifest.events.find(
      (e) => e.kind === "action_step"
    );
    if (firstAction) {
      selectStep(firstAction.step_id);
    } else {
      updateDetailPanel();
    }
  }

  function renderHeader() {
    const m = state.manifest;
    const outcomeEl = $("outcome");
    if (outcomeEl) {
      outcomeEl.textContent = m.outcome;
      outcomeEl.className = "outcome " + m.outcome;
    }
    const titleEl = $("test-title");
    if (titleEl) titleEl.textContent = m.test_title;

    const metaEl = $("test-meta");
    if (metaEl) {
      const durationMs = m.finished_at - m.started_at;
      metaEl.textContent = `${m.spec_file}  •  ${durationMs}ms`;
    }
  }

  function renderTimeline() {
    const list = $("step-list");
    if (!list) return;
    list.innerHTML = "";

    const actionSteps = state.manifest.events.filter(
      (e) => e.kind === "action_step"
    );

    if (actionSteps.length === 0) {
      list.innerHTML =
        '<li class="empty-state">No action steps recorded.</li>';
      return;
    }

    for (const step of actionSteps) {
      const li = document.createElement("li");
      li.dataset.stepId = step.step_id;
      if (step.error) li.classList.add("step-error");

      const icon = actionIcon(step.action);
      const durationMs = step.ts_end - step.ts_start;

      li.innerHTML = `
        <span class="step-icon">${icon}</span>
        <span class="step-info">
          <div class="step-action">${escHtml(step.action)}</div>
          <div class="step-selector">${escHtml(step.selector || step.url || "")}</div>
        </span>
        <span class="step-duration">${durationMs}ms</span>
      `;

      li.addEventListener("click", () => selectStep(step.step_id));
      list.appendChild(li);
    }
  }

  function renderTabs() {
    const tabs = $("tabs");
    if (!tabs) return;

    const tabDefs = [
      { id: "snapshot", label: "DOM Snapshot" },
      { id: "screenshot", label: "Screenshot" },
      { id: "network", label: "Network" },
      { id: "console", label: "Console" },
    ];

    tabs.innerHTML = "";
    for (const tab of tabDefs) {
      const btn = document.createElement("button");
      btn.className = "tab-btn" + (state.activeTab === tab.id ? " active" : "");
      btn.textContent = tab.label;
      btn.addEventListener("click", () => {
        state.activeTab = tab.id;
        renderTabs();
        updateDetailPanel();
      });
      tabs.appendChild(btn);
    }
  }

  // ── Step selection ─────────────────────────────────────────────────────────

  function selectStep(stepId) {
    state.selectedStepId = stepId;

    // Highlight in timeline.
    const list = $("step-list");
    if (list) {
      for (const li of list.querySelectorAll("li[data-step-id]")) {
        li.classList.toggle(
          "selected",
          parseInt(li.dataset.stepId, 10) === stepId
        );
      }
    }

    updateDetailPanel();
  }

  // ── Detail panel ──────────────────────────────────────────────────────────

  function updateDetailPanel() {
    const content = $("tab-content");
    if (!content) return;
    content.innerHTML = "";

    const step = getSelectedStep();

    switch (state.activeTab) {
      case "snapshot":
        renderSnapshotTab(content, step);
        break;
      case "screenshot":
        renderScreenshotTab(content, step);
        break;
      case "network":
        renderNetworkTab(content, step);
        break;
      case "console":
        renderConsoleTab(content, step);
        break;
    }
  }

  function getSelectedStep() {
    if (state.selectedStepId === null) return null;
    return state.manifest.events.find(
      (e) => e.kind === "action_step" && e.step_id === state.selectedStepId
    ) || null;
  }

  // ── Snapshot tab ───────────────────────────────────────────────────────────

  function renderSnapshotTab(container, step) {
    if (!step || !step.dom_snapshot_ref) {
      const assetPath = step && step.dom_snapshot_ref
        ? state.manifest.assets[step.dom_snapshot_ref]
        : null;

      container.innerHTML =
        '<div class="empty-state">No DOM snapshot for this step.</div>';
      return;
    }

    const assetPath = state.manifest.assets[step.dom_snapshot_ref];
    if (!assetPath) {
      container.innerHTML =
        '<div class="empty-state">DOM snapshot asset not found.</div>';
      return;
    }

    const iframe = document.createElement("iframe");
    iframe.id = "snapshot-pane";
    iframe.src = `/assets/${encodeURIComponent(step.dom_snapshot_ref)}`;
    iframe.setAttribute("sandbox", "allow-same-origin allow-scripts");
    container.appendChild(iframe);
  }

  // ── Screenshot tab ────────────────────────────────────────────────────────

  function renderScreenshotTab(container, step) {
    const screenshotRef = step && step.screenshot_ref ? step.screenshot_ref : null;

    if (!screenshotRef) {
      container.innerHTML =
        '<div class="empty-state">No screenshot for this step.</div>';
      return;
    }

    const div = document.createElement("div");
    div.id = "screenshot-pane";
    const img = document.createElement("img");
    img.src = `/assets/${encodeURIComponent(screenshotRef)}`;
    img.alt = "Step screenshot";
    div.appendChild(img);
    container.appendChild(div);
  }

  // ── Network tab ───────────────────────────────────────────────────────────

  function renderNetworkTab(container, step) {
    const allNetwork = state.manifest.events.filter(
      (e) => e.kind === "network"
    );

    // Filter to events within the selected step's time window.
    let events = allNetwork;
    if (step) {
      events = allNetwork.filter(
        (e) => e.ts_start >= step.ts_start && e.ts_start <= step.ts_end
      );
    }

    const pane = document.createElement("div");
    pane.id = "network-pane";

    if (events.length === 0) {
      pane.innerHTML =
        '<div class="empty-state">No network requests in this step window.</div>';
      container.appendChild(pane);
      return;
    }

    const table = document.createElement("table");
    table.className = "network-table";
    table.innerHTML = `
      <thead>
        <tr>
          <th>Method</th>
          <th>URL</th>
          <th>Status</th>
          <th>Duration</th>
        </tr>
      </thead>
    `;

    const tbody = document.createElement("tbody");
    for (const ev of events) {
      const tr = document.createElement("tr");
      const duration =
        ev.ts_end !== undefined && ev.ts_end !== null
          ? `${ev.ts_end - ev.ts_start}ms`
          : "…";
      const statusClass = ev.status
        ? `status-${Math.floor(ev.status / 100)}xx`
        : "";
      tr.innerHTML = `
        <td>${escHtml(ev.method)}</td>
        <td title="${escHtml(ev.url)}">${escHtml(truncate(ev.url, 60))}</td>
        <td class="${statusClass}">${ev.status != null ? ev.status : "—"}</td>
        <td>${duration}</td>
      `;
      tbody.appendChild(tr);
    }
    table.appendChild(tbody);
    pane.appendChild(table);
    container.appendChild(pane);
  }

  // ── Console tab ───────────────────────────────────────────────────────────

  function renderConsoleTab(container, step) {
    const allConsole = state.manifest.events.filter(
      (e) => e.kind === "console"
    );

    let events = allConsole;
    if (step) {
      events = allConsole.filter(
        (e) => e.ts >= step.ts_start && e.ts <= step.ts_end
      );
    }

    const pane = document.createElement("div");
    pane.id = "console-pane";

    if (events.length === 0) {
      pane.innerHTML =
        '<div class="empty-state">No console messages in this step window.</div>';
      container.appendChild(pane);
      return;
    }

    for (const ev of events) {
      const entry = document.createElement("div");
      entry.className = "console-entry";
      entry.innerHTML = `
        <span class="console-time">+${ev.ts}ms</span>
        <span class="console-level ${ev.level}">${escHtml(ev.level)}</span>
        <span class="console-text">${escHtml(ev.text)}</span>
      `;
      pane.appendChild(entry);
    }
    container.appendChild(pane);
  }

  // ── Utilities ──────────────────────────────────────────────────────────────

  function actionIcon(action) {
    const icons = {
      click: "👆",
      fill: "⌨️",
      goto: "🔗",
      evaluate: "⚡",
      screenshot: "📸",
      wait_for: "⏳",
      hover: "🎯",
      check: "✅",
      uncheck: "☐",
      type_text: "⌨️",
    };
    return icons[action] || "▶";
  }

  function escHtml(str) {
    if (!str) return "";
    return String(str)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function truncate(str, max) {
    if (!str) return "";
    return str.length > max ? str.slice(0, max) + "…" : str;
  }
})();
// CODEGEN-END
