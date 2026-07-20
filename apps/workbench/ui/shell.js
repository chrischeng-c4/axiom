// HANDWRITE-BEGIN gap="missing-generator:logic:447ec524" tracker="pending-tracker" reason="Drive Tauri or deterministic test bridge commands, folder selection, collapse, keyboard navigation, and actionable errors."
// @spec apps/workbench/tech-design/logic/deliver-workbench-three-column-shell-and-registered-launch-folde.md#logic
(() => {
  "use strict";

  const elements = {
    shell: document.querySelector("#workbench-shell"),
    nav: document.querySelector("#launch-folders"),
    collapse: document.querySelector("#collapse-folders"),
    add: document.querySelector("#add-folder"),
    emptyAdd: document.querySelector("#empty-add-folder"),
    folderEmpty: document.querySelector("#folder-empty"),
    folderList: document.querySelector("#folder-list"),
    folderTemplate: document.querySelector("#folder-item-template"),
    activePath: document.querySelector("#active-path"),
    terminalBadge: document.querySelector("#terminal-badge"),
    terminalEmpty: document.querySelector("#terminal-empty"),
    terminalReady: document.querySelector("#terminal-ready"),
    terminalPath: document.querySelector("#terminal-path"),
    contextEmpty: document.querySelector("#context-empty"),
    contextReady: document.querySelector("#context-ready"),
    contextName: document.querySelector("#context-folder-name"),
    status: document.querySelector("#shell-status"),
  };

  let state = { folders: [], selectedId: null };
  let collapsed = false;
  let statusTimer = null;

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

  function selectedFolder() {
    return state.folders.find((folder) => folder.id === state.selectedId) ?? null;
  }

  function folderInitial(folder) {
    return folder.name.trim().slice(0, 2) || "·";
  }

  function announce(message, kind = "info") {
    window.clearTimeout(statusTimer);
    elements.status.textContent = message;
    elements.status.classList.toggle("error", kind === "error");
    elements.status.classList.add("visible");
    statusTimer = window.setTimeout(() => {
      elements.status.classList.remove("visible");
    }, kind === "error" ? 8000 : 4200);
  }

  function clearAnnouncement() {
    window.clearTimeout(statusTimer);
    elements.status.textContent = "";
    elements.status.classList.remove("visible", "error");
  }

  function folderButtons() {
    return [...elements.folderList.querySelectorAll(".folder-button")];
  }

  function focusFolderAt(index) {
    const buttons = folderButtons();
    if (!buttons.length) return;
    const wrapped = (index + buttons.length) % buttons.length;
    buttons.forEach((button, position) => {
      button.tabIndex = position === wrapped ? 0 : -1;
    });
    buttons[wrapped].focus();
  }

  async function selectFolder(folderId, source = "pointer") {
    try {
      state = await command("select_launch_folder", { folderId });
      const path = await command("selected_launch_path");
      render();
      announce(
        source === "keyboard"
          ? `Selected ${selectedFolder()?.name ?? "launch folder"} from the keyboard.`
          : `Selected ${selectedFolder()?.name ?? "launch folder"}.`,
      );
      window.dispatchEvent(
        new CustomEvent("workbench:launch-folder-selected", {
          detail: { folderId, path },
        }),
      );
    } catch (error) {
      announce(error instanceof Error ? error.message : String(error), "error");
    }
  }

  function renderFolders() {
    const fragment = document.createDocumentFragment();
    state.folders.forEach((folder, index) => {
      const item = elements.folderTemplate.content.cloneNode(true);
      const button = item.querySelector(".folder-button");
      const selected = folder.id === state.selectedId;
      button.dataset.folderId = folder.id;
      button.setAttribute("aria-current", String(selected));
      button.setAttribute("aria-label", `${folder.name}, ${folder.path}`);
      button.title = folder.path;
      button.tabIndex = selected || (!state.selectedId && index === 0) ? 0 : -1;
      item.querySelector(".folder-monogram").textContent = folderInitial(folder);
      item.querySelector(".folder-name").textContent = folder.name;
      item.querySelector(".folder-path").textContent = folder.path;

      button.addEventListener("click", () => selectFolder(folder.id));
      button.addEventListener("keydown", (event) => {
        const buttons = folderButtons();
        const current = buttons.indexOf(event.currentTarget);
        if (event.key === "ArrowDown") {
          event.preventDefault();
          focusFolderAt(current + 1);
        } else if (event.key === "ArrowUp") {
          event.preventDefault();
          focusFolderAt(current - 1);
        } else if (event.key === "Home") {
          event.preventDefault();
          focusFolderAt(0);
        } else if (event.key === "End") {
          event.preventDefault();
          focusFolderAt(buttons.length - 1);
        } else if (
          event.key === "Enter" ||
          event.key === " " ||
          event.key === "Space"
        ) {
          event.preventDefault();
          selectFolder(folder.id, "keyboard");
        }
      });
      fragment.append(item);
    });
    elements.folderList.replaceChildren(fragment);
    elements.folderEmpty.hidden = state.folders.length > 0;
  }

  function renderSelection() {
    const folder = selectedFolder();
    const ready = Boolean(folder);
    elements.activePath.textContent = folder?.path ?? "None selected";
    elements.activePath.title = folder?.path ?? "No launch folder selected";
    elements.terminalEmpty.hidden = ready;
    elements.terminalReady.hidden = !ready;
    elements.contextEmpty.hidden = ready;
    elements.contextReady.hidden = !ready;
    elements.terminalBadge.textContent = ready ? "Folder ready" : "Waiting for folder";
    elements.terminalBadge.classList.toggle("ready", ready);
    if (folder) {
      elements.terminalPath.textContent = folder.path;
      elements.contextName.textContent = folder.name;
    }
  }

  function render() {
    renderFolders();
    renderSelection();
    elements.nav.dataset.collapsed = String(collapsed);
    elements.shell.dataset.navCollapsed = String(collapsed);
    elements.collapse.setAttribute("aria-expanded", String(!collapsed));
    elements.collapse.setAttribute(
      "aria-label",
      collapsed ? "Expand launch folders" : "Collapse launch folders",
    );
    elements.collapse.title = collapsed
      ? "Expand launch folders"
      : "Collapse launch folders";
  }

  async function addFolder() {
    elements.add.disabled = true;
    elements.emptyAdd.disabled = true;
    try {
      const next = await command("choose_launch_folder");
      if (next === null) {
        announce("Folder selection cancelled. Your current selection was kept.");
        return;
      }
      state = next;
      render();
      announce(`Added ${selectedFolder()?.name ?? "launch folder"}.`);
    } catch (error) {
      announce(error instanceof Error ? error.message : String(error), "error");
    } finally {
      elements.add.disabled = false;
      elements.emptyAdd.disabled = false;
    }
  }

  function toggleCollapsed() {
    collapsed = !collapsed;
    render();
    announce(collapsed ? "Launch folders collapsed." : "Launch folders expanded.");
  }

  async function reload() {
    state = await command("load_shell_state");
    render();
    clearAnnouncement();
    return state;
  }

  async function initialize() {
    elements.add.addEventListener("click", addFolder);
    elements.emptyAdd.addEventListener("click", addFolder);
    elements.collapse.addEventListener("click", toggleCollapsed);
    try {
      await reload();
    } catch (error) {
      render();
      announce(error instanceof Error ? error.message : String(error), "error");
    }
    window.dispatchEvent(new CustomEvent("workbench:shell-ready"));
    return state;
  }

  const ready = initialize();
  window.__WORKBENCH_SHELL__ = {
    ready,
    reload,
    snapshot: () => ({
      state: structuredClone(state),
      collapsed,
      selectedPath: selectedFolder()?.path ?? null,
    }),
  };
})();
// HANDWRITE-END
