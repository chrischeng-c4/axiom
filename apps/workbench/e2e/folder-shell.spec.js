// HANDWRITE-BEGIN gap="missing-generator:unit-test:7e629f86" tracker="pending-tracker" reason="Exercise the rendered shell through Jet at desktop and constrained widths and retain screenshot and interaction evidence."
// @spec apps/workbench/tech-design/logic/deliver-workbench-three-column-shell-and-registered-launch-folde.md#unit-test
const fs = require("node:fs");
const path = require("node:path");

const projectRoot = process.cwd();
const pageUrl = `file://${path.join(projectRoot, "apps/workbench/ui/index.html")}`;
const evidenceDir = path.join(
  projectRoot,
  "apps/workbench/evidence/folder-shell/2192",
);
const evidencePath = path.join(evidenceDir, "journey.json");

function initialEvidence() {
  return {
    schemaVersion: "workbench.folder-shell.evidence.v1",
    workItem: 2192,
    viewports: {},
    interactions: {},
    accessibility: {},
    selectedLaunchPath: null,
    noChildProcess: true,
  };
}

function updateEvidence(section, value) {
  fs.mkdirSync(evidenceDir, { recursive: true });
  let evidence = initialEvidence();
  if (fs.existsSync(evidencePath)) {
    evidence = JSON.parse(fs.readFileSync(evidencePath, "utf8"));
  }
  evidence[section] = value;
  fs.writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`);
}

async function installBridge(page, initialState, choices = []) {
  await page.evaluate(
    ({ seededState, seededChoices }) => {
      const clone = (value) => JSON.parse(JSON.stringify(value));
      const bridge = {
        calls: [],
        state: clone(seededState),
        choices: clone(seededChoices),
        async invoke(command, args = {}) {
          this.calls.push({ command, args: clone(args) });
          if (command === "load_shell_state") return clone(this.state);
          if (command === "choose_launch_folder") {
            const choice = this.choices.shift();
            if (choice === null || choice === undefined) return null;
            if (choice.error) throw new Error(choice.error);
            const existing = this.state.folders.find(
              (folder) => folder.path === choice.path,
            );
            const folder = existing ?? choice;
            if (!existing) this.state.folders.push(clone(folder));
            this.state.selectedId = folder.id;
            return clone(this.state);
          }
          if (command === "select_launch_folder") {
            const exists = this.state.folders.some(
              (folder) => folder.id === args.folderId,
            );
            if (!exists) throw new Error(`Unknown folder ${args.folderId}`);
            this.state.selectedId = args.folderId;
            return clone(this.state);
          }
          if (command === "selected_launch_path") {
            return (
              this.state.folders.find(
                (folder) => folder.id === this.state.selectedId,
              )?.path ?? null
            );
          }
          throw new Error(`Unexpected command ${command}`);
        },
      };
      window.__WORKBENCH_TEST_BRIDGE__ = bridge;
    },
    { seededState: initialState, seededChoices: choices },
  );
}

async function waitUntil(page, predicate, message, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await page.evaluate(predicate)) return;
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  throw new Error(message);
}

async function openShell(page, initialState, choices = []) {
  await page.goto(pageUrl);
  await waitUntil(
    page,
    () => Boolean(window.__WORKBENCH_SHELL__),
    "Workbench shell did not initialize",
  );
  await page.evaluate(() => window.__WORKBENCH_SHELL__.ready);
  await installBridge(page, initialState, choices);
  await page.evaluate(() => window.__WORKBENCH_SHELL__.reload());
}

describe("Workbench registered launch-folder shell", () => {
  test("adds folders and retains desktop primary-state evidence", async ({ page }) => {
    fs.mkdirSync(evidenceDir, { recursive: true });
    fs.writeFileSync(evidencePath, `${JSON.stringify(initialEvidence(), null, 2)}\n`);
    await page.setViewportSize({ width: 1440, height: 900 });
    await openShell(
      page,
      { folders: [], selectedId: null },
      [
        { id: "folder-axiom", name: "axiom", path: "/Users/demo/axiom" },
        {
          id: "folder-workbench",
          name: "app_workbench",
          path: "/Users/demo/axiom/app_workbench",
        },
      ],
    );

    expect(await page.locator("nav").count()).toBe(1);
    expect(await page.locator("main").count()).toBe(1);
    expect(await page.locator("aside").count()).toBe(1);
    expect(await page.locator("#folder-empty").isVisible()).toBe(true);
    await page.click("#add-folder");
    await page.click("#add-folder");

    expect(await page.locator(".folder-button").count()).toBe(2);
    expect(await page.locator("#terminal-ready").isVisible()).toBe(true);
    expect(await page.locator("#context-ready").isVisible()).toBe(true);
    expect(await page.locator("#active-path").innerText()).toBe(
      "/Users/demo/axiom/app_workbench",
    );
    const documentText = await page.locator("body").innerText();
    expect(documentText.includes("TODO")).toBe(false);
    expect(documentText.includes("Lorem ipsum")).toBe(false);

    const screenshot = path.join(evidenceDir, "desktop.png");
    await page.screenshot({ path: screenshot });
    const calls = await page.evaluate(() => window.__WORKBENCH_TEST_BRIDGE__.calls);
    updateEvidence("viewports", {
      desktop: { width: 1440, height: 900, artifact: "desktop.png" },
    });
    updateEvidence("interactions", {
      folderAdd: true,
      folderSelect: true,
      commandCalls: calls.map((call) => call.command),
    });
    updateEvidence("selectedLaunchPath", "/Users/demo/axiom/app_workbench");
  });

  test("supports keyboard navigation and a constrained compact rail", async ({ page }) => {
    await page.setViewportSize({ width: 860, height: 720 });
    await openShell(page, {
      folders: [
        { id: "folder-one", name: "axiom", path: "/Users/demo/axiom" },
        { id: "folder-two", name: "workbench", path: "/Users/demo/workbench" },
      ],
      selectedId: "folder-one",
    });

    await page.evaluate(() =>
      document.querySelector('[data-folder-id="folder-one"]')?.focus(),
    );
    await page.keyboard.press("ArrowDown");
    expect(
      await page.evaluate(() => document.activeElement?.dataset.folderId),
    ).toBe("folder-two");
    await page.keyboard.press("Enter");
    await waitUntil(
      page,
      () =>
        document.querySelector('[data-folder-id="folder-two"]')?.getAttribute("aria-current") ===
        "true",
      "Enter did not select folder-two",
    );

    await page.click("#collapse-folders");
    expect(await page.locator("#collapse-folders").getAttribute("aria-expanded")).toBe(
      "false",
    );
    expect(await page.locator("#launch-folders").getAttribute("data-collapsed")).toBe(
      "true",
    );
    await page.keyboard.press("Tab");
    expect(
      await page.evaluate(() => document.activeElement?.dataset.folderId),
    ).toBe("folder-two");
    const focusOrder = ["collapse-folders", "folder-two"];
    for (let step = 0; step < 16 && !focusOrder.includes("add-folder"); step += 1) {
      await page.keyboard.press("Tab");
      focusOrder.push(
        await page.evaluate(
          () =>
            document.activeElement?.id ||
            document.activeElement?.dataset.folderId ||
            document.activeElement?.tagName.toLowerCase(),
        ),
      );
    }
    expect(focusOrder.includes("add-folder")).toBe(true);

    await page.evaluate(() =>
      document.querySelector('[data-folder-id="folder-one"]')?.focus(),
    );
    await page.keyboard.type(" ");
    try {
      await waitUntil(
        page,
        () =>
          document.querySelector('[data-folder-id="folder-one"]')?.getAttribute(
            "aria-current",
          ) === "true",
        "Space did not select folder-one",
      );
    } catch (error) {
      const diagnostics = await page.evaluate(() => ({
        activeId: document.activeElement?.id,
        activeFolderId: document.activeElement?.dataset.folderId,
        ariaCurrent: document
          .querySelector('[data-folder-id="folder-one"]')
          ?.getAttribute("aria-current"),
        calls: window.__WORKBENCH_TEST_BRIDGE__.calls,
      }));
      throw new Error(`${error.message}: ${JSON.stringify(diagnostics)}`);
    }

    const screenshot = path.join(evidenceDir, "constrained.png");
    await page.screenshot({ path: screenshot });
    const prior = JSON.parse(fs.readFileSync(evidencePath, "utf8"));
    updateEvidence("viewports", {
      ...prior.viewports,
      constrained: { width: 860, height: 720, artifact: "constrained.png" },
    });
    updateEvidence("interactions", {
      ...prior.interactions,
      collapse: true,
      arrowNavigation: true,
      enterSelection: true,
      spaceSelection: true,
      focusOrder,
    });
  });

  test("keeps cancelled and failed registration states actionable", async ({ page }) => {
    await page.setViewportSize({ width: 1100, height: 760 });
    await openShell(page, { folders: [], selectedId: null }, [
      null,
      { error: "The selected path is not a directory" },
    ]);

    await page.click("#add-folder");
    expect((await page.locator("#shell-status").innerText()).includes("cancelled")).toBe(
      true,
    );
    await page.click("#add-folder");
    expect(
      (await page.locator("#shell-status").innerText()).includes("not a directory"),
    ).toBe(true);
    expect(await page.locator("#add-folder").isEnabled()).toBe(true);
    expect(await page.locator("#empty-add-folder").isEnabled()).toBe(true);

    const accessibility = await page.evaluate(() => ({
      landmarks:
        document.querySelectorAll("nav").length === 1 &&
        document.querySelectorAll("main").length === 1 &&
        document.querySelectorAll("aside").length === 1,
      headings: document.querySelectorAll("h1, h2, h3").length,
      labelledButtons: [...document.querySelectorAll("button")].every(
        (button) => Boolean(button.textContent.trim() || button.getAttribute("aria-label")),
      ),
      liveStatus: document.querySelector("#shell-status")?.getAttribute("role") === "status",
      minimumBodyFontPx: Number.parseFloat(getComputedStyle(document.body).fontSize),
    }));
    expect(accessibility.landmarks).toBe(true);
    expect(accessibility.headings >= 6).toBe(true);
    expect(accessibility.labelledButtons).toBe(true);
    expect(accessibility.liveStatus).toBe(true);
    expect(accessibility.minimumBodyFontPx >= 14).toBe(true);

    const calls = await page.evaluate(() => window.__WORKBENCH_TEST_BRIDGE__.calls);
    expect(calls.some((call) => call.command === "launch_agent")).toBe(false);
    updateEvidence("accessibility", accessibility);
    updateEvidence("functionalStates", {
      empty: true,
      cancelledPicker: true,
      invalidPath: true,
      actionsRemainEnabled: true,
    });
    updateEvidence("noChildProcess", true);
  });
});
// HANDWRITE-END
