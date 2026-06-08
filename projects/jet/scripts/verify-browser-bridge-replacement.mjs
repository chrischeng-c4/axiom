#!/usr/bin/env node
import fsSync from "node:fs";
import fs from "node:fs/promises";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { spawn, spawnSync } from "node:child_process";

const repoRoot = process.cwd();
const requireFromScript = createRequire(import.meta.url);

function parseArgs(argv) {
  const args = {
    jetBin: "target/release/jet",
    evidence: "/tmp/jet-basic-dom-gate/browser-bridge-replacement.json",
    timeoutMs: 15_000,
    baselineTimeoutMs: 120_000,
    baselineTools: ["playwright"],
    requireBaselines: true,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--jet-bin") args.jetBin = argv[++i];
    else if (arg === "--evidence") args.evidence = argv[++i];
    else if (arg === "--timeout-ms") args.timeoutMs = Number(argv[++i]);
    else if (arg === "--baseline-timeout-ms") args.baselineTimeoutMs = Number(argv[++i]);
    else if (arg === "--baseline-tools") {
      const value = argv[++i];
      args.baselineTools = value === "none" ? [] : value.split(",").filter(Boolean);
    }
    else if (arg === "--require-baselines") args.requireBaselines = true;
    else if (arg === "--no-require-baselines") args.requireBaselines = false;
    else throw new Error(`unknown argument: ${arg}`);
  }
  return args;
}

function rel(p) {
  return path.relative(repoRoot, p) || ".";
}

function shortLines(text, limit = 60) {
  const lines = text.trim().split(/\r?\n/).filter(Boolean);
  if (lines.length <= limit) return lines;
  return [...lines.slice(0, 20), "...", ...lines.slice(-20)];
}

function run(command, cwd, timeoutMs) {
  const started = process.hrtime.bigint();
  return new Promise((resolve) => {
    const stdout = [];
    const stderr = [];
    const child = spawn(command[0], command.slice(1), {
      cwd,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
    let timedOut = false;
    const timer = setTimeout(() => {
      timedOut = true;
      child.kill("SIGTERM");
    }, timeoutMs);
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.on("error", (error) => {
      clearTimeout(timer);
      const result = {
        command,
        cwd: rel(cwd),
        exit_code: 1,
        timed_out: timedOut,
        duration_ms: Number(process.hrtime.bigint() - started) / 1_000_000,
        stdout: [],
        stderr: [String(error.stack ?? error)],
      };
      Object.defineProperty(result, "stdoutText", { value: "" });
      resolve(result);
    });
    child.on("close", (code) => {
      clearTimeout(timer);
      const stdoutText = Buffer.concat(stdout).toString("utf8");
      const stderrText = Buffer.concat(stderr).toString("utf8");
      const result = {
        command,
        cwd: rel(cwd),
        exit_code: code ?? 1,
        timed_out: timedOut,
        duration_ms: Number(process.hrtime.bigint() - started) / 1_000_000,
        stdout: shortLines(stdoutText),
        stderr: shortLines(stderrText),
      };
      Object.defineProperty(result, "stdoutText", { value: stdoutText });
      resolve(result);
    });
  });
}

function parseJsonCommand(command) {
  const text = command.stdoutText ?? command.stdout.join("\n");
  if (!text.trim()) return null;
  return JSON.parse(text);
}

async function serveFixture(root) {
  const html = `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Jet BB replacement fixture</title>
    <style>
      body { font-family: system-ui, sans-serif; margin: 24px; }
      #fixture { display: grid; gap: 12px; width: 420px; }
      button, input { font: inherit; padding: 10px 12px; }
      label { display: flex; align-items: center; gap: 8px; }
      #scrollbox { height: 72px; overflow: auto; border: 1px solid #94a3b8; padding: 8px; }
      #scrollfill { height: 220px; background: linear-gradient(#ecfeff, #0891b2); }
      #drag-zone { height: 70px; border: 1px solid #64748b; position: relative; }
      #drag-handle { position: absolute; left: 12px; top: 16px; width: 72px; padding: 8px; background: #2563eb; color: white; cursor: grab; user-select: none; }
      #status { min-height: 24px; color: #0f766e; }
    </style>
  </head>
  <body>
    <main id="fixture">
      <h1>Jet BB replacement fixture</h1>
      <button id="count" type="button">Increment</button>
      <input id="field" aria-label="Fixture field" value="">
      <label><input id="agree" type="checkbox"> Agree</label>
      <div id="scrollbox"><div id="scrollfill">Scrollable target</div></div>
      <div id="drag-zone"><div id="drag-handle">Drag</div></div>
      <p id="status">count=0 value= checked=false scroll=0 drag=0</p>
    </main>
    <script>
      const events = [];
      const button = document.querySelector("#count");
      const field = document.querySelector("#field");
      const agree = document.querySelector("#agree");
      const scrollbox = document.querySelector("#scrollbox");
      const dragHandle = document.querySelector("#drag-handle");
      const status = document.querySelector("#status");
      let count = 0;
      let dragging = false;
      let dragStartX = 0;
      let dragDelta = 0;
      let dragReleased = false;
      function sync(event) {
        status.textContent = "count=" + count
          + " value=" + field.value
          + " checked=" + agree.checked
          + " scroll=" + scrollbox.scrollTop
          + " drag=" + Math.round(dragDelta);
        events.push(event + ":" + count + ":" + field.value + ":" + agree.checked + ":" + scrollbox.scrollTop + ":" + Math.round(dragDelta));
      }
      button.addEventListener("click", () => {
        count += 1;
        sync("click");
      });
      field.addEventListener("input", () => sync("input"));
      agree.addEventListener("change", () => sync("check"));
      scrollbox.addEventListener("scroll", () => sync("scroll"));
      dragHandle.addEventListener("mousedown", (event) => {
        dragging = true;
        dragReleased = false;
        dragStartX = event.clientX;
        dragDelta = 0;
        sync("drag-start");
      });
      document.addEventListener("mousemove", (event) => {
        if (!dragging) return;
        dragDelta = event.clientX - dragStartX;
        dragHandle.style.transform = "translateX(" + Math.max(0, dragDelta) + "px)";
        sync("drag-move");
      });
      document.addEventListener("mouseup", () => {
        if (!dragging) return;
        dragging = false;
        dragReleased = true;
        sync("drag-end");
      });
      window.__jetBbFixture = () => ({
        count,
        value: field.value,
        checked: agree.checked,
        scrollTop: scrollbox.scrollTop,
        dragDelta: Math.round(dragDelta),
        dragReleased,
        status: status.textContent,
        events: events.slice(),
        activeId: document.activeElement ? document.activeElement.id : "",
      });
    </script>
  </body>
</html>`;
  await fs.writeFile(path.join(root, "index.html"), html);

  const server = http.createServer(async (req, res) => {
    if (req.url === "/" || req.url === "/index.html") {
      res.writeHead(200, { "Content-Type": "text/html" });
      res.end(html);
    } else {
      res.writeHead(404, { "Content-Type": "text/plain" });
      res.end("not found");
    }
  });

  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      server.off("error", reject);
      resolve();
    });
  });
  const address = server.address();
  return {
    url: `http://127.0.0.1:${address.port}/`,
    close: () => new Promise((resolve) => server.close(resolve)),
  };
}

function snapshotExpression() {
  return `(() => {
    const button = document.querySelector("#count");
    const field = document.querySelector("#field");
    const agree = document.querySelector("#agree");
    const scrollbox = document.querySelector("#scrollbox");
    const dragHandle = document.querySelector("#drag-handle");
    const br = button.getBoundingClientRect();
    const fr = field.getBoundingClientRect();
    const ar = agree.getBoundingClientRect();
    const sr = scrollbox.getBoundingClientRect();
    const dr = dragHandle.getBoundingClientRect();
    return {
      title: document.title,
      text: document.body.innerText,
      buttonCenter: { x: br.left + br.width / 2, y: br.top + br.height / 2 },
      fieldCenter: { x: fr.left + fr.width / 2, y: fr.top + fr.height / 2 },
      checkboxCenter: { x: ar.left + ar.width / 2, y: ar.top + ar.height / 2 },
      scrollCenter: { x: sr.left + sr.width / 2, y: sr.top + sr.height / 2 },
      dragStart: { x: dr.left + dr.width / 2, y: dr.top + dr.height / 2 },
      dragEnd: { x: dr.left + dr.width / 2 + 96, y: dr.top + dr.height / 2 },
      fixture: window.__jetBbFixture()
    };
  })()`;
}

function okCommand(command) {
  return command.exit_code === 0 && !command.timed_out;
}

function writeJsonSync(file, value) {
  fsSync.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}

function isExecutable(file) {
  try {
    fsSync.accessSync(file, fsSync.constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function findChromiumInCache(cacheRoot) {
  const binarySubpath = process.platform === "darwin"
    ? "chrome-mac/Chromium.app/Contents/MacOS/Chromium"
    : process.platform === "linux"
      ? "chrome-linux/chrome"
      : "chrome-win/chrome.exe";
  let entries = [];
  try {
    entries = fsSync.readdirSync(cacheRoot, { withFileTypes: true });
  } catch {
    return null;
  }
  return entries
    .filter((entry) => entry.isDirectory() && /^chromium-\d+$/.test(entry.name))
    .map((entry) => ({
      revision: Number(entry.name.slice("chromium-".length)),
      bin: path.join(cacheRoot, entry.name, binarySubpath),
    }))
    .sort((a, b) => b.revision - a.revision)
    .find((entry) => isExecutable(entry.bin))
    ?.bin ?? null;
}

function findChromiumExecutable() {
  for (const envName of ["JET_BROWSER_EXECUTABLE", "CHROME_PATH", "CHROME", "CHROMIUM_PATH"]) {
    const value = process.env[envName];
    if (value && isExecutable(value)) return value;
  }
  const home = os.homedir();
  const cached = findChromiumInCache(path.join(home, ".jet", "browsers"));
  if (cached) return cached;
  const systemCandidates = process.platform === "darwin"
    ? [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
      ]
    : process.platform === "linux"
      ? ["google-chrome", "google-chrome-stable", "chromium-browser", "chromium"]
      : [
          "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
          "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
        ];
  for (const candidate of systemCandidates) {
    if (path.isAbsolute(candidate) && isExecutable(candidate)) return candidate;
    if (process.platform === "linux") {
      const found = spawnSync("which", [candidate], { encoding: "utf8" });
      if (found.status === 0) {
        const resolved = found.stdout.trim();
        if (resolved && isExecutable(resolved)) return resolved;
      }
    }
  }
  return null;
}

function resolvePackageRoot(packageName, searchRoots) {
  try {
    return path.dirname(requireFromScript.resolve(`${packageName}/package.json`, {
      paths: searchRoots,
    }));
  } catch {
    return null;
  }
}

async function provisionPlaywrightWithJet(args, tmpRoot) {
  const root = path.join(tmpRoot, "playwright-baseline-tool");
  await fs.mkdir(root, { recursive: true });
  await fs.writeFile(
    path.join(root, "package.json"),
    `${JSON.stringify(
      {
        name: "jet-bb-playwright-baseline-tool",
        version: "0.0.0",
        private: true,
        dependencies: {
          playwright: "^1.58.0",
        },
      },
      null,
      2,
    )}\n`,
  );
  const install = await run(
    [path.resolve(repoRoot, args.jetBin), "install"],
    root,
    args.baselineTimeoutMs,
  );
  const packageRoot = resolvePackageRoot("playwright", [root]);
  return {
    strategy: "jet_install_playwright_baseline_tool",
    result: install.exit_code === 0 && packageRoot ? "green" : "red",
    root: rel(root),
    package_root: packageRoot ? rel(packageRoot) : null,
    install,
    absolute_package_root: packageRoot,
  };
}

async function resolvePlaywright(args, tmpRoot) {
  const envRoot = process.env.JET_PLAYWRIGHT_PACKAGE_ROOT;
  if (envRoot && fsSync.existsSync(path.join(envRoot, "package.json"))) {
    const localRequire = createRequire(path.join(envRoot, "package.json"));
    return {
      playwright: localRequire("playwright"),
      setup: {
        strategy: "JET_PLAYWRIGHT_PACKAGE_ROOT",
        result: "green",
        package_root: rel(envRoot),
      },
    };
  }
  const setup = await provisionPlaywrightWithJet(args, tmpRoot);
  if (setup.result !== "green" || !setup.absolute_package_root) {
    return { playwright: null, setup };
  }
  const localRequire = createRequire(path.join(setup.absolute_package_root, "package.json"));
  return {
    playwright: localRequire("playwright"),
    setup: {
      ...setup,
      absolute_package_root: undefined,
    },
  };
}

function fixtureStateOk(snapshot) {
  return snapshot?.fixture?.count === 1 &&
    snapshot?.fixture?.value === "hi" &&
    snapshot?.fixture?.checked === true &&
    (snapshot?.fixture?.scrollTop ?? 0) > 0 &&
    (snapshot?.fixture?.dragDelta ?? 0) >= 80 &&
    snapshot?.fixture?.dragReleased === true;
}

async function runPlaywrightBaseline(args, tmpRoot, serverUrl) {
  const started = process.hrtime.bigint();
  const baseline = {
    tool: "playwright",
    result: "pending",
    command_policy: "baseline_only_not_jet_executor",
    setup: null,
    browser_executable: null,
    duration_ms: null,
    checks: [],
    failures: [],
    snapshots: {},
    screenshot: null,
  };
  let browser = null;
  try {
    const resolved = await resolvePlaywright(args, tmpRoot);
    baseline.setup = resolved.setup;
    if (!resolved.playwright) {
      baseline.failures.push("playwright_package_unavailable");
      baseline.result = "red";
      return baseline;
    }
    const executablePath = findChromiumExecutable();
    baseline.browser_executable = executablePath ? rel(executablePath) : null;
    const launchOptions = {
      headless: true,
    };
    if (executablePath) launchOptions.executablePath = executablePath;
    browser = await resolved.playwright.chromium.launch(launchOptions);
    const page = await browser.newPage();
    await page.goto(serverUrl, { waitUntil: "domcontentloaded" });
    baseline.snapshots.initial = await page.evaluate(snapshotExpression());

    const { buttonCenter, fieldCenter, checkboxCenter, scrollCenter, dragStart, dragEnd } =
      baseline.snapshots.initial;
    await page.mouse.click(buttonCenter.x, buttonCenter.y);
    await page.mouse.click(fieldCenter.x, fieldCenter.y);
    await page.keyboard.type("hi");
    await page.mouse.click(checkboxCenter.x, checkboxCenter.y);
    await page.mouse.move(scrollCenter.x, scrollCenter.y);
    await page.mouse.wheel(0, 160);
    await page.mouse.move(dragStart.x, dragStart.y);
    await page.mouse.down();
    await page.mouse.move(dragEnd.x, dragEnd.y, { steps: 8 });
    await page.mouse.up();

    baseline.snapshots.afterActions = await page.evaluate(snapshotExpression());
    const screenshotPath = path.join(tmpRoot, "playwright-baseline.png");
    await page.screenshot({ path: screenshotPath });
    const stat = await fs.stat(screenshotPath);
    baseline.screenshot = { path: rel(screenshotPath), bytes: stat.size };
    baseline.checks = [
      { name: "playwright_observes_fixture", ok: baseline.snapshots.initial?.fixture?.status === "count=0 value= checked=false scroll=0 drag=0" },
      { name: "playwright_gesture_semantics_match_contract", ok: fixtureStateOk(baseline.snapshots.afterActions) },
      { name: "playwright_screenshot_available", ok: stat.size > 1000 },
    ];
    baseline.failures.push(...baseline.checks.filter((check) => !check.ok).map((check) => check.name));
    baseline.result = baseline.failures.length === 0 ? "green" : "red";
  } catch (error) {
    baseline.failures.push(String(error.stack ?? error));
    baseline.result = "red";
  } finally {
    if (browser) {
      try {
        await browser.close();
      } catch {
        // Baseline cleanup failure is not part of Jet BB executor semantics.
      }
    }
    baseline.duration_ms = Number(process.hrtime.bigint() - started) / 1_000_000;
  }
  return baseline;
}

const externalBrowserAutomationBins = new Set([
  "playwright",
  "cypress",
  "npx",
  "npm",
  "pnpm",
  "yarn",
  "bun",
  "corepack",
]);

function externalBrowserAutomationCommands(commands) {
  return commands
    .filter((command) => externalBrowserAutomationBins.has(path.basename(command.command?.[0] || "")))
    .map((command) => command.command.join(" "));
}

async function clickAt(commands, jet, sessionRoot, timeoutMs, point) {
  commands.push(await run([jet, "bb", "mouse", "--button", "left", "--buttons", "1", "--click-count", "1", "mousePressed", String(point.x), String(point.y)], sessionRoot, timeoutMs));
  commands.push(await run([jet, "bb", "mouse", "--button", "left", "--buttons", "0", "--click-count", "1", "mouseReleased", String(point.x), String(point.y)], sessionRoot, timeoutMs));
}

async function typeText(commands, jet, sessionRoot, timeoutMs, text) {
  for (const char of text) {
    commands.push(await run([jet, "bb", "key", char], sessionRoot, timeoutMs));
  }
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const tmpRoot = await fs.mkdtemp(path.join(os.tmpdir(), "jet-bb-replacement-"));
  const webRoot = path.join(tmpRoot, "web");
  const sessionRoot = path.join(tmpRoot, "session");
  await fs.mkdir(webRoot, { recursive: true });
  await fs.mkdir(sessionRoot, { recursive: true });

  const commands = [];
  let server;
  let snapshots = {};
  let domCapture = null;
  let screenshot = null;
  let baselineBenchmarks = [];
  let result = "green";
  const failures = [];

  try {
    server = await serveFixture(webRoot);
    const jet = path.resolve(repoRoot, args.jetBin);
    commands.push(await run([jet, "bb", "launch", server.url], sessionRoot, args.timeoutMs));
    if (!okCommand(commands.at(-1))) failures.push("launch_failed");

    commands.push(await run([jet, "bb", "eval", snapshotExpression()], sessionRoot, args.timeoutMs));
    if (okCommand(commands.at(-1))) snapshots.initial = parseJsonCommand(commands.at(-1));
    else failures.push("initial_eval_failed");

    const button = snapshots.initial?.buttonCenter;
    if (button) {
      await clickAt(commands, jet, sessionRoot, args.timeoutMs, button);
    } else {
      failures.push("button_coordinates_missing");
    }

    const field = snapshots.initial?.fieldCenter;
    if (field) {
      await clickAt(commands, jet, sessionRoot, args.timeoutMs, field);
      await typeText(commands, jet, sessionRoot, args.timeoutMs, "hi");
    } else {
      failures.push("field_coordinates_missing");
    }

    const checkbox = snapshots.initial?.checkboxCenter;
    if (checkbox) {
      await clickAt(commands, jet, sessionRoot, args.timeoutMs, checkbox);
    } else {
      failures.push("checkbox_coordinates_missing");
    }

    const scroll = snapshots.initial?.scrollCenter;
    if (scroll) {
      commands.push(await run([jet, "bb", "wheel", String(scroll.x), String(scroll.y), "--delta-y", "160"], sessionRoot, args.timeoutMs));
    } else {
      failures.push("scroll_coordinates_missing");
    }

    const dragStart = snapshots.initial?.dragStart;
    const dragEnd = snapshots.initial?.dragEnd;
    if (dragStart && dragEnd) {
      commands.push(await run([jet, "bb", "drag", String(dragStart.x), String(dragStart.y), String(dragEnd.x), String(dragEnd.y), "--steps", "8"], sessionRoot, args.timeoutMs));
    } else {
      failures.push("drag_coordinates_missing");
    }

    commands.push(await run([jet, "bb", "eval", snapshotExpression()], sessionRoot, args.timeoutMs));
    if (okCommand(commands.at(-1))) snapshots.afterActions = parseJsonCommand(commands.at(-1));
    else failures.push("after_actions_eval_failed");

    commands.push(await run([jet, "bb", "capture", "--surface", "dom", "--root-selector", "#fixture", "--pretty"], sessionRoot, args.timeoutMs));
    if (okCommand(commands.at(-1))) domCapture = parseJsonCommand(commands.at(-1));
    else failures.push("dom_capture_failed");

    const screenshotPath = path.join(tmpRoot, "bb-screenshot.png");
    commands.push(await run([jet, "bb", "screenshot", "--out", screenshotPath], sessionRoot, args.timeoutMs));
    if (okCommand(commands.at(-1))) {
      const stat = await fs.stat(screenshotPath);
      screenshot = { path: rel(screenshotPath), bytes: stat.size };
    } else {
      failures.push("screenshot_failed");
    }

    for (const tool of args.baselineTools) {
      if (tool === "playwright") {
        baselineBenchmarks.push(await runPlaywrightBaseline(args, tmpRoot, server.url));
      } else {
        baselineBenchmarks.push({
          tool,
          result: "red",
          command_policy: "baseline_only_not_jet_executor",
          hard_failures: [`unsupported browser baseline tool: ${tool}`],
        });
      }
    }
  } catch (error) {
    result = "red";
    failures.push(String(error.stack ?? error));
  } finally {
    if (sessionRoot) {
      commands.push(await run([path.resolve(repoRoot, args.jetBin), "bb", "shutdown"], sessionRoot, 5_000));
    }
    if (server) await server.close();
  }

  const forbiddenExternalCommands = externalBrowserAutomationCommands(commands);
  const checks = [
    { name: "jet_bb_launch_exit_zero", ok: okCommand(commands[0] ?? {}) },
    { name: "all_jet_bb_commands_exit_zero", ok: commands.every(okCommand) },
    { name: "no_playwright_cypress_npm_executor_commands", ok: forbiddenExternalCommands.length === 0 },
    { name: "initial_eval_observes_fixture", ok: snapshots.initial?.fixture?.status === "count=0 value= checked=false scroll=0 drag=0" },
    { name: "mouse_click_updates_state", ok: snapshots.afterActions?.fixture?.count === 1 },
    { name: "keyboard_input_updates_state", ok: snapshots.afterActions?.fixture?.value === "hi" },
    { name: "checkbox_click_updates_state", ok: snapshots.afterActions?.fixture?.checked === true },
    { name: "wheel_scroll_updates_state", ok: (snapshots.afterActions?.fixture?.scrollTop ?? 0) > 0 },
    { name: "drag_updates_state", ok: (snapshots.afterActions?.fixture?.dragDelta ?? 0) >= 80 && snapshots.afterActions?.fixture?.dragReleased === true },
    { name: "dom_capture_available", ok: Boolean(domCapture) },
    { name: "screenshot_available", ok: (screenshot?.bytes ?? 0) > 1000 },
    {
      name: "required_baseline_benchmarks_green",
      ok: !args.requireBaselines ||
        (baselineBenchmarks.length === args.baselineTools.length &&
          baselineBenchmarks.every((baseline) => baseline.result === "green")),
    },
  ];

  for (const check of checks) {
    if (!check.ok) failures.push(check.name);
  }
  if (failures.length > 0) result = "red";

  const evidence = {
    contract_id: "basic.browser-bridge.replacement",
    result,
    phase: 2,
    prerequisite_contracts: [
      "basic.install.replacement",
    ],
    generated_at: new Date().toISOString(),
    note: "Jet BB is the executor. External browser-automation tools are comparison targets for action semantics, not runtime dependencies for this gate.",
    fixture: {
      root: rel(webRoot),
      url: server?.url ?? null,
    },
    browser_action_contract: {
      actions: [
        "launch",
        "eval DOM state",
        "click button",
        "focus input",
        "type text",
        "check checkbox",
        "wheel scroll",
        "drag handle",
        "capture DOM",
        "screenshot",
      ],
      expected_state: {
        count: 1,
        value: "hi",
        checked: true,
        min_scroll_top: 1,
        min_drag_delta: 80,
        dragReleased: true,
      },
    },
    executor_contract: {
      executor: "jet bb",
      forbidden_browser_automation_executors: [...externalBrowserAutomationBins],
      forbidden_commands: forbiddenExternalCommands,
    },
    baseline_benchmark_contract: {
      tools: args.baselineTools,
      required_for_exit_zero: args.requireBaselines,
      command_policy: "baseline_only_not_jet_executor",
    },
    checks,
    failures,
    snapshots,
    baseline_benchmarks: baselineBenchmarks,
    dom_capture_summary: domCapture
      ? {
          schema_version: domCapture.schema_version ?? null,
          surface: domCapture.surface ?? null,
          root_selector: domCapture.root_selector ?? "#fixture",
        }
      : null,
    screenshot,
    commands,
  };

  await fs.mkdir(path.dirname(args.evidence), { recursive: true });
  await fs.writeFile(args.evidence, `${JSON.stringify(evidence, null, 2)}\n`);
  console.log(JSON.stringify(evidence, null, 2));
  process.exit(result === "green" ? 0 : 1);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
