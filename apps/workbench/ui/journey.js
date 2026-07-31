// HANDWRITE-BEGIN gap="missing-generator:logic:5aad99d2" tracker="pending-tracker" reason="Drive the native or deterministic test bridge production session without absorbing folder registry ownership."
// @spec apps/workbench/tech-design/logic/prove-the-workbench-folder-to-agent-to-artifact-production-journ.md#logic
(() => {
  "use strict";

  const elements = {
    agents: [...document.querySelectorAll('input[name="agent"]')],
    start: document.querySelector("#start-agent"),
    stop: document.querySelector("#stop-agent"),
    transcript: document.querySelector("#terminal-transcript"),
    form: document.querySelector("#terminal-form"),
    input: document.querySelector("#terminal-input"),
    send: document.querySelector("#send-terminal-input"),
    activeCwd: document.querySelector("#active-cwd"),
    cwdSource: document.querySelector("#cwd-source"),
    terminalBadge: document.querySelector("#terminal-badge"),
    contextTabs: [...document.querySelectorAll("[data-context-target]")],
    contextDocument: document.querySelector("#context-document"),
    contextProvenance: document.querySelector("#context-provenance"),
    sourceLinks: document.querySelector("#source-links"),
    status: document.querySelector("#journey-status"),
  };

  let selectedPath = null;
  let snapshot = null;
  let contextTarget = "workspace";
  let pollTimer = null;

  function command(name, args = {}) {
    const bridge = window.__WORKBENCH_TEST_BRIDGE__ ?? null;
    const invoke = bridge?.invoke?.bind(bridge) ?? window.__TAURI__?.core?.invoke;
    if (typeof invoke !== "function") {
      return Promise.reject(
        new Error("The native Workbench bridge is unavailable. Restart the desktop host."),
      );
    }
    return Promise.resolve(invoke(name, args));
  }

  function announce(message, kind = "info") {
    elements.status.textContent = message;
    elements.status.dataset.kind = kind;
  }

  function selectedAgent() {
    return elements.agents.find((agent) => agent.checked)?.value ?? "claude";
  }

  function setBusy(busy) {
    elements.start.disabled = busy || !selectedPath;
    elements.start.setAttribute("aria-busy", String(busy));
    elements.agents.forEach((agent) => {
      agent.disabled = busy;
    });
  }

  function renderSnapshot(next) {
    snapshot = next;
    elements.transcript.textContent = next?.transcript || "Session output will appear here.";
    elements.activeCwd.textContent = next?.activeCwd || selectedPath || "No active cwd";
    elements.cwdSource.textContent = next?.cwdSource || "Launch folder";
    const running = Boolean(next?.running);
    elements.terminalBadge.textContent = running
      ? `${next.agent} active`
      : next?.exitCode === null || next?.exitCode === undefined
        ? selectedPath
          ? "Ready to launch"
          : "Waiting for folder"
        : `Exited ${next.exitCode}`;
    elements.terminalBadge.classList.toggle("ready", Boolean(selectedPath));
    elements.terminalBadge.classList.toggle("active", running);
    elements.stop.disabled = !running;
    elements.input.disabled = !running;
    elements.send.disabled = !running;
    if (running) {
      elements.transcript.scrollTop = elements.transcript.scrollHeight;
    } else {
      stopPolling();
    }
  }

  function startPolling() {
    stopPolling();
    pollTimer = window.setInterval(async () => {
      try {
        renderSnapshot(await command("poll_journey_agent"));
      } catch (error) {
        stopPolling();
        announce(error instanceof Error ? error.message : String(error), "error");
      }
    }, 150);
  }

  function stopPolling() {
    if (pollTimer !== null) {
      window.clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  async function launch() {
    if (!selectedPath) {
      announce("Select a launch folder before starting an agent.", "error");
      return;
    }
    setBusy(true);
    try {
      const next = await command("launch_journey_agent", {
        agent: selectedAgent(),
        cwd: selectedPath,
      });
      renderSnapshot(next);
      announce(`${next.agent} launched in the selected canonical folder.`);
      startPolling();
      await renderContext(contextTarget);
      elements.input.focus();
    } catch (error) {
      renderSnapshot(null);
      announce(error instanceof Error ? error.message : String(error), "error");
    } finally {
      setBusy(false);
    }
  }

  async function terminate() {
    elements.stop.disabled = true;
    try {
      const next = await command("terminate_journey_agent");
      renderSnapshot(next);
      announce(`${next.agent} stopped cleanly.`);
    } catch (error) {
      announce(error instanceof Error ? error.message : String(error), "error");
    }
  }

  async function sendInput(event) {
    event.preventDefault();
    const input = elements.input.value;
    if (!input.trim()) return;
    elements.send.disabled = true;
    try {
      renderSnapshot(await command("send_journey_input", { input }));
      elements.input.value = "";
      elements.input.focus();
    } catch (error) {
      announce(error instanceof Error ? error.message : String(error), "error");
    } finally {
      elements.send.disabled = !snapshot?.running;
    }
  }

  function normalizeDocument(document) {
    return {
      rendererId: document.rendererId ?? document.renderer_id ?? "fallback",
      title: document.title ?? "Context preview",
      bodyHtml: document.bodyHtml ?? document.body_html ?? "",
      navigation: document.navigation ?? [],
      warnings: document.warnings ?? [],
      provenance: document.provenance ?? { root: selectedPath, sources: [] },
    };
  }

  async function renderContext(target) {
    if (!selectedPath) return;
    contextTarget = target;
    elements.contextTabs.forEach((tab) => {
      const active = tab.dataset.contextTarget === target;
      tab.setAttribute("aria-selected", String(active));
      tab.tabIndex = active ? 0 : -1;
    });
    elements.contextDocument.setAttribute("aria-busy", "true");
    try {
      const root = snapshot?.activeCwd || selectedPath;
      const requestedTarget = target === "workspace" ? null : target;
      const document = normalizeDocument(
        await command("render_journey_context", {
          root,
          target: requestedTarget,
        }),
      );
      elements.contextDocument.innerHTML = `<header><span>${document.rendererId}</span><h3>${escapeText(document.title)}</h3></header>${document.bodyHtml}`;
      elements.contextProvenance.textContent = `${document.rendererId} · canonical source root ${document.provenance.root}`;
      elements.sourceLinks.replaceChildren();
      document.navigation.forEach((navigation) => {
        const button = window.document.createElement("button");
        button.type = "button";
        button.textContent = navigation.label;
        button.dataset.path = navigation.path;
        button.addEventListener("click", () => {
          announce(`Source: ${navigation.path}`);
          window.dispatchEvent(
            new CustomEvent("workbench:source-navigate", {
              detail: { root, path: navigation.path },
            }),
          );
        });
        elements.sourceLinks.append(button);
      });
      if (document.warnings.length) {
        announce(document.warnings.join(" · "), "warning");
      }
    } catch (error) {
      elements.contextDocument.textContent =
        error instanceof Error ? error.message : String(error);
      elements.contextProvenance.textContent = "Context unavailable · source unchanged";
      elements.sourceLinks.replaceChildren();
      announce(elements.contextDocument.textContent, "error");
    } finally {
      elements.contextDocument.setAttribute("aria-busy", "false");
    }
  }

  function escapeText(value) {
    return String(value)
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#39;");
  }

  function focusContextTab(index) {
    const wrapped = (index + elements.contextTabs.length) % elements.contextTabs.length;
    elements.contextTabs[wrapped].focus();
  }

  async function reload() {
    await window.__WORKBENCH_SHELL__?.ready;
    selectedPath = window.__WORKBENCH_SHELL__?.snapshot()?.selectedPath ?? null;
    if (!selectedPath) {
      try {
        selectedPath = await command("selected_launch_path");
      } catch (_) {
        selectedPath = null;
      }
    }
    stopPolling();
    renderSnapshot(null);
    setBusy(false);
    elements.contextTabs.forEach((tab) => {
      tab.disabled = !selectedPath;
    });
    if (selectedPath) {
      await renderContext(contextTarget);
      announce("Folder, native agent, and read-only context are ready.");
    } else {
      elements.contextDocument.textContent = "Select a folder to inspect canonical context.";
      elements.contextProvenance.textContent = "No canonical source root selected";
      elements.sourceLinks.replaceChildren();
    }
    return { selectedPath, snapshot };
  }

  elements.start.addEventListener("click", launch);
  elements.stop.addEventListener("click", terminate);
  elements.form.addEventListener("submit", sendInput);
  elements.contextTabs.forEach((tab, index) => {
    tab.addEventListener("click", () => renderContext(tab.dataset.contextTarget));
    tab.addEventListener("keydown", (event) => {
      if (event.key === "ArrowRight" || event.key === "ArrowDown") {
        event.preventDefault();
        focusContextTab(index + 1);
      } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
        event.preventDefault();
        focusContextTab(index - 1);
      } else if (event.key === "Home") {
        event.preventDefault();
        focusContextTab(0);
      } else if (event.key === "End") {
        event.preventDefault();
        focusContextTab(elements.contextTabs.length - 1);
      } else if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        renderContext(tab.dataset.contextTarget);
      }
    });
  });
  window.addEventListener("workbench:launch-folder-selected", (event) => {
    selectedPath = event.detail.path;
    reload();
  });
  window.addEventListener("beforeunload", stopPolling);

  const ready = reload();
  window.__WORKBENCH_JOURNEY__ = {
    ready,
    reload,
    renderContext,
    snapshot: () => ({ selectedPath, snapshot, contextTarget }),
  };
})();
// HANDWRITE-END
