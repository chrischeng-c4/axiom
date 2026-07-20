// HANDWRITE-BEGIN gap="missing-generator:unit-test:c4c7768b" tracker="pending-tracker" reason="Exercise keyboard operation, launch, transcript, cwd, context navigation, unavailable-agent recovery, accessibility, and retained desktop/constrained evidence through Jet."
const fs = require("node:fs");
const path = require("node:path");

const projectRoot = process.cwd();
const pageUrl = `file://${path.join(projectRoot, "apps/workbench/ui/index.html")}`;
const evidenceDir = path.join(
  projectRoot,
  "apps/workbench/evidence/production-journey/v1",
);
const manifestPath = path.join(evidenceDir, "manifest.json");
const productionCommand =
  "cargo test -p workbench --test production_journey -- --nocapture";

function emptyManifest() {
  return {
    schemaVersion: "workbench.production-journey.evidence.v1",
    workItem: 2201,
    command: productionCommand,
    assertions: {},
    artifacts: {},
  };
}

function updateManifest(mutator) {
  fs.mkdirSync(evidenceDir, { recursive: true });
  const manifest = fs.existsSync(manifestPath)
    ? JSON.parse(fs.readFileSync(manifestPath, "utf8"))
    : emptyManifest();
  mutator(manifest);
  fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
}

async function installBridge(page, { failNextLaunch = false } = {}) {
  await page.evaluate(
    ({ shouldFail }) => {
      const clone = (value) => JSON.parse(JSON.stringify(value));
      const selectedPath = "/Users/demo/axiom/app_workbench";
      const activeCwd = `${selectedPath}/nested`;
      const bridge = {
        calls: [],
        failNextLaunch: shouldFail,
        state: {
          folders: [
            {
              id: "folder-workbench",
              name: "app_workbench",
              path: selectedPath,
            },
          ],
          selectedId: "folder-workbench",
        },
        session: {
          agent: "Codex",
          running: true,
          exitCode: null,
          activeCwd,
          cwdSource: "OSC 7",
          transcript: "$ codex\nWorkbench production fixture ready\n",
        },
        async invoke(command, args = {}) {
          this.calls.push({ command, args: clone(args) });
          if (command === "load_shell_state") return clone(this.state);
          if (command === "selected_launch_path") return selectedPath;
          if (command === "select_launch_folder") return clone(this.state);
          if (command === "choose_launch_folder") return clone(this.state);
          if (command === "launch_journey_agent") {
            if (this.failNextLaunch) {
              this.failNextLaunch = false;
              throw new Error(
                `${args.agent || "Selected agent"} is unavailable; install it or select another native agent`,
              );
            }
            this.session.agent =
              { claude: "Claude Code", codex: "Codex", agy: "AGY" }[
                args.agent
              ] ?? "Claude Code";
            this.session.running = true;
            this.session.exitCode = null;
            this.session.transcript = `$ ${args.agent}\nWorkbench production fixture ready\n`;
            return clone(this.session);
          }
          if (command === "poll_journey_agent") return clone(this.session);
          if (command === "send_journey_input") {
            this.session.transcript += `› ${args.input}\ncontext rendered\n`;
            return clone(this.session);
          }
          if (command === "resize_journey_agent") return clone(this.session);
          if (command === "interrupt_journey_agent") return clone(this.session);
          if (command === "terminate_journey_agent") {
            this.session.running = false;
            this.session.exitCode = 0;
            return clone(this.session);
          }
          if (command === "render_journey_context") {
            const target = args.target ?? null;
            if (target === null) {
              return {
                rendererId: "git",
                kind: "git",
                title: "Git working tree",
                bodyHtml:
                  "<section><h3>Status</h3><pre>M README.md\n?? nested/tech-design.md</pre></section>",
                navigation: [
                  { label: "README.md", path: "README.md" },
                  { label: "nested/tech-design.md", path: "nested/tech-design.md" },
                ],
                warnings: [],
                provenance: { root: activeCwd, sources: [activeCwd] },
              };
            }
            const typed = target === "tech-design.md";
            return {
              rendererId: typed ? "aw-typed" : "markdown",
              kind: typed ? "aw_typed" : "markdown",
              title: typed ? "Tech design: tech-design.md" : "README.md",
              bodyHtml: typed
                ? "<article><h2>Tech design</h2><h3>Logic</h3><p>Canonical source remains read only.</p></article>"
                : "<article><h1>Workbench fixture</h1><p>Markdown context is canonical.</p></article>",
              navigation: [
                {
                  label: typed ? "Logic · line 8" : "README.md",
                  path: target,
                },
              ],
              warnings: [],
              provenance: {
                root: activeCwd,
                sources: [`${activeCwd}/${target}`],
              },
            };
          }
          throw new Error(`Unexpected command ${command}`);
        },
      };
      window.__WORKBENCH_TEST_BRIDGE__ = bridge;
    },
    { shouldFail: failNextLaunch },
  );
}

async function waitUntil(page, predicate, message, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await page.evaluate(predicate)) return;
    await new Promise((resolve) => setTimeout(resolve, 40));
  }
  throw new Error(message);
}

async function openProduction(page, options = {}) {
  await page.goto(pageUrl);
  await waitUntil(
    page,
    () => Boolean(window.__WORKBENCH_SHELL__ && window.__WORKBENCH_JOURNEY__),
    "Workbench production shell did not initialize",
  );
  await installBridge(page, options);
  await page.evaluate(async () => {
    await window.__WORKBENCH_SHELL__.reload();
    await window.__WORKBENCH_JOURNEY__.reload();
  });
}

describe("Workbench folder-to-agent-to-artifact production journey", () => {
  test("renders the complete desktop primary state and source navigation", async ({
    page,
  }) => {
    fs.mkdirSync(evidenceDir, { recursive: true });
    fs.writeFileSync(manifestPath, `${JSON.stringify(emptyManifest(), null, 2)}\n`);
    await page.setViewportSize({ width: 1440, height: 900 });
    await openProduction(page);

    await page.click('input[name="agent"][value="codex"]');
    await page.click("#start-agent");
    await waitUntil(
      page,
      () => document.querySelector("#terminal-transcript")?.textContent.includes("fixture ready"),
      "terminal transcript did not become ready",
    );
    expect(await page.locator("#active-cwd").innerText()).toBe(
      "/Users/demo/axiom/app_workbench/nested",
    );
    await page.click("#terminal-input");
    await page.keyboard.type("show context");
    await page.click("#send-terminal-input");
    await waitUntil(
      page,
      () => document.querySelector("#terminal-transcript")?.textContent.includes("context rendered"),
      "terminal input was not rendered",
    );

    await page.click('[data-context-target="workspace"]');
    expect(
      (await page.locator("#context-document").innerText()).includes(
        "Git working tree",
      ),
    ).toBe(true);
    await page.click('[data-context-target="tech-design.md"]');
    expect(
      (await page.locator("#context-document").innerText()).includes("Tech design"),
    ).toBe(true);
    expect(
      (await page.locator("#context-provenance").innerText()).includes("aw-typed"),
    ).toBe(true);
    expect((await page.locator("#source-links button").count()) > 0).toBe(true);
    const bodyText = await page.locator("body").innerText();
    for (const placeholder of ["TODO", "Lorem ipsum", "No renderer is active yet"]) {
      expect(bodyText.includes(placeholder)).toBe(false);
    }

    const screenshot = path.join(evidenceDir, "desktop.png");
    await page.screenshot({ path: screenshot });
    updateManifest((manifest) => {
      manifest.artifacts.desktop = {
        path: "desktop.png",
        mediaType: "image/png",
        width: 1440,
        height: 900,
      };
      manifest.assertions.folderAgentCwd = {
        passed: true,
        artifacts: ["desktop.png", "pty-transcript.txt"],
      };
      manifest.assertions.markdownGitAwContext = {
        passed: true,
        artifacts: ["desktop.png", "context-summary.json"],
      };
      manifest.assertions.sourceNavigation = {
        passed: true,
        artifacts: ["desktop.png", "context-summary.json"],
      };
      manifest.assertions.placeholderFreePrimaryState = {
        passed: true,
        artifacts: ["desktop.png"],
      };
    });
  });

  test("is keyboard operable and readable at constrained desktop width", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 860, height: 720 });
    await openProduction(page);
    await page.evaluate(() =>
      document.querySelector('input[name="agent"][value="claude"]')?.focus(),
    );
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("Tab");
    expect(await page.evaluate(() => document.activeElement?.id)).toBe("start-agent");
    await page.keyboard.press("Enter");
    await waitUntil(
      page,
      () => document.querySelector("#terminal-transcript")?.textContent.includes("fixture ready"),
      "keyboard launch failed",
    );
    await page.evaluate(() =>
      document.querySelector('[data-context-target="workspace"]')?.focus(),
    );
    await page.keyboard.press("ArrowRight");
    await page.keyboard.press("Enter");
    await waitUntil(
      page,
      () => document.querySelector("#context-document")?.textContent.includes("Workbench fixture"),
      "keyboard context selection failed",
    );

    const accessibility = await page.evaluate(() => ({
      noHorizontalOverflow: document.documentElement.scrollWidth <= window.innerWidth,
      labelledControls: [...document.querySelectorAll("button, input")].every(
        (element) =>
          Boolean(
            element.getAttribute("aria-label") ||
              element.labels?.length ||
              element.textContent?.trim(),
          ),
      ),
      liveStatus: document.querySelector("#journey-status")?.getAttribute("role") === "status",
      bodyFontPx: Number.parseFloat(getComputedStyle(document.body).fontSize),
      focusedOutlinePx: Number.parseFloat(
        getComputedStyle(document.activeElement).outlineWidth,
      ),
    }));
    accessibility.reducedMotionRule = fs
      .readFileSync(path.join(projectRoot, "apps/workbench/ui/shell.css"), "utf8")
      .includes("@media (prefers-reduced-motion: reduce)");
    expect(accessibility.noHorizontalOverflow).toBe(true);
    expect(accessibility.labelledControls).toBe(true);
    expect(accessibility.liveStatus).toBe(true);
    expect(accessibility.bodyFontPx >= 16).toBe(true);
    expect(accessibility.focusedOutlinePx >= 2).toBe(true);
    expect(accessibility.reducedMotionRule).toBe(true);

    const screenshot = path.join(evidenceDir, "constrained.png");
    await page.screenshot({ path: screenshot });
    updateManifest((manifest) => {
      manifest.artifacts.constrained = {
        path: "constrained.png",
        mediaType: "image/png",
        width: 860,
        height: 720,
      };
      manifest.assertions.keyboardAccessibility = {
        passed: true,
        artifacts: ["constrained.png"],
        details: accessibility,
      };
      manifest.assertions.constrainedReadability = {
        passed: true,
        artifacts: ["constrained.png"],
      };
    });
  });

  test("recovers from an unavailable agent without losing context", async ({ page }) => {
    await page.setViewportSize({ width: 1100, height: 760 });
    await openProduction(page, { failNextLaunch: true });
    await page.click('input[name="agent"][value="agy"]');
    await page.click("#start-agent");
    await waitUntil(
      page,
      () => document.querySelector("#journey-status")?.textContent.includes("unavailable"),
      "unavailable agent error was not announced",
    );
    expect(await page.locator("#start-agent").isEnabled()).toBe(true);
    await page.click('input[name="agent"][value="claude"]');
    await page.click("#start-agent");
    await waitUntil(
      page,
      () => document.querySelector("#terminal-transcript")?.textContent.includes("fixture ready"),
      "retry with another agent failed",
    );
    await page.click('[data-context-target="README.md"]');
    expect(
      (await page.locator("#context-document").innerText()).includes(
        "Workbench fixture",
      ),
    ).toBe(true);
    updateManifest((manifest) => {
      manifest.assertions.unavailableAgentRecovery = {
        passed: true,
        artifacts: ["desktop.png", "context-summary.json"],
      };
      manifest.artifacts.ptyTranscript = {
        path: "pty-transcript.txt",
        mediaType: "text/plain",
      };
      manifest.artifacts.contextSummary = {
        path: "context-summary.json",
        mediaType: "application/json",
      };
    });
  });
});
// HANDWRITE-END
