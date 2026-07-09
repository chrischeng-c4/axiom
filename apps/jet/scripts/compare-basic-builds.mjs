#!/usr/bin/env node
// <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-scripts-compare-basic-builds-mjs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
import { brotliCompressSync, gzipSync } from "node:zlib";
import { createHash } from "node:crypto";
import { cp, mkdir, mkdtemp, readFile, readdir, rm, stat, symlink, writeFile } from "node:fs/promises";
import { constants as fsConstants } from "node:fs";
import { access } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";
import { createServer } from "node:http";

const __filename = fileURLToPath(import.meta.url);
const repoRoot = path.resolve(path.dirname(__filename), "../../..");
const defaultFixture = path.join(repoRoot, "examples/react-bench");

const args = new Map();
for (let i = 2; i < process.argv.length; i += 1) {
  const arg = process.argv[i];
  if (!arg.startsWith("--")) {
    throw new Error(`unexpected positional argument: ${arg}`);
  }
  const next = process.argv[i + 1];
  if (!next || next.startsWith("--")) {
    args.set(arg, "true");
  } else {
    args.set(arg, next);
    i += 1;
  }
}

const selfTestMode = args.has("--self-test");
const durationRatioMax = 1.25;
const gzipRatioMax = 1.05;
const defaultJetBin = await firstExecutablePath(["target/release/jet", "target/debug/jet"]);
const jetBin = path.resolve(repoRoot, args.get("--jet-bin") ?? process.env.JET_BIN ?? defaultJetBin);
const outputRoot =
  args.has("--out-dir")
    ? path.resolve(repoRoot, args.get("--out-dir"))
    : await mkdtemp(path.join(tmpdir(), "jet-basic-build-compare-"));
const evidencePath =
  args.has("--evidence")
    ? path.resolve(repoRoot, args.get("--evidence"))
    : path.join(outputRoot, "basic-build-compare.json");
const runtimeSmokeMode = normalizeRuntimeSmokeMode(args.get("--runtime-smoke") ?? "off");
const commandTimeoutMs = Number(args.get("--command-timeout-ms") ?? 120_000);
const runtimeToolTimeoutMs = Number(args.get("--runtime-timeout-ms") ?? 30_000);
const buildSamples = parsePositiveInteger(args.get("--build-samples") ?? "1", "--build-samples");
const fixtureSource = path.resolve(repoRoot, args.get("--fixture") ?? defaultFixture);
const fixtureName = args.get("--fixture-name") ?? path.basename(fixtureSource);
const dependencyRoot = path.resolve(repoRoot, args.get("--dependency-root") ?? fixtureSource);
const toolRoot = path.resolve(repoRoot, args.get("--tool-root") ?? dependencyRoot);
const toolNames = parseTools(args.get("--tools") ?? "jet,vite,webpack");
const nodeBin = selfTestMode ? null : await findNodeBin(toolRoot, toolNames);
const runtimeCase = args.get("--runtime-case") ?? "react-bench";
const semanticStrings = parseCsv(
  args.get("--semantic-strings") ?? "React Bench,Counter,Todos,Add todo",
);
const requireCss = args.has("--require-css");
const requiredPublicFiles = parseCsv(args.get("--require-public") ?? "");

function normalizeRuntimeSmokeMode(value) {
  if (["off", "optional", "required"].includes(value)) {
    return value;
  }
  throw new Error(`--runtime-smoke must be one of off, optional, or required; got ${value}`);
}

function parseCsv(raw) {
  return raw
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

async function findNodeBin(start, requiredTools) {
  const starts = [path.resolve(start)];
  for (const startDir of starts) {
    let current = startDir;
    while (true) {
      const candidate = path.join(current, "node_modules/.bin");
      if (await hasRequiredNodeTools(candidate, requiredTools)) {
        return candidate;
      }

      if (current === repoRoot || current === path.dirname(current)) {
        break;
      }
      current = path.dirname(current);
    }
  }
  throw new Error(`could not find node_modules/.bin with ${requiredTools.join(",")} from ${start}`);
}

async function hasRequiredNodeTools(candidate, requiredTools) {
  const nodeTools = requiredTools.filter((tool) => tool !== "jet");
  for (const tool of nodeTools) {
    try {
      await access(path.join(candidate, tool), fsConstants.X_OK);
    } catch {
      return false;
    }
  }
  return true;
}

async function materializeFixtureRoot(source, dependencyRoot, outputRoot, fixtureName) {
  const workRoot = path.join(outputRoot, ".fixture-work", fixtureName);
  await rm(workRoot, { recursive: true, force: true });
  await mkdir(path.dirname(workRoot), { recursive: true });
  await cp(source, workRoot, {
    recursive: true,
    filter: (src) => !src.split(path.sep).includes("node_modules") && !src.split(path.sep).includes("dist"),
  });
  await sanitizeAwClaimWrappers(workRoot);

  const dependencyNodeModules = path.join(dependencyRoot, "node_modules");
  const fixtureNodeModules = path.join(workRoot, "node_modules");
  if (await pathIsDirectory(dependencyNodeModules)) {
    await symlink(dependencyNodeModules, fixtureNodeModules, "dir");
  }
  return workRoot;
}

async function sanitizeAwClaimWrappers(root) {
  for (const file of await walkFiles(root)) {
    if (!isAwWrappedTextCandidate(file)) continue;
    const text = await readFile(file, "utf8");
    const stripped = stripAwClaimWrapperLines(text);
    if (stripped !== text) await writeFile(file, stripped);
  }
}

function isAwWrappedTextCandidate(file) {
  return new Set([
    ".cjs",
    ".css",
    ".html",
    ".js",
    ".json",
    ".jsx",
    ".mjs",
    ".ts",
    ".tsx",
  ]).has(path.extname(file));
}

function stripAwClaimWrapperLines(text) {
  if (!text.includes("// <HANDWRITE") && !text.includes("// </HANDWRITE>")) return text;
  return text
    .split(/\n/)
    .filter((line) => {
      const trimmed = line.trimStart();
      return !trimmed.startsWith("// <HANDWRITE ") && trimmed !== "// </HANDWRITE>";
    })
    .join("\n");
}

async function pathIsDirectory(candidate) {
  try {
    return (await stat(candidate)).isDirectory();
  } catch {
    return false;
  }
}

function parseTools(raw) {
  const parsed = parseCsv(raw);
  const supported = new Set(["jet", "vite", "webpack"]);
  const seen = new Set();
  const tools = [];
  for (const tool of parsed) {
    if (!supported.has(tool)) {
      throw new Error(`--tools contains unsupported tool ${tool}`);
    }
    if (!seen.has(tool)) {
      tools.push(tool);
      seen.add(tool);
    }
  }
  if (!seen.has("jet")) {
    throw new Error("--tools must include jet");
  }
  if (tools.length < 2) {
    throw new Error("--tools must include jet and at least one baseline tool");
  }
  return tools;
}

function parsePositiveInteger(raw, label) {
  const parsed = Number(raw);
  if (!Number.isInteger(parsed) || parsed < 1) {
    throw new Error(`${label} must be a positive integer; got ${raw}`);
  }
  return parsed;
}

function progress(message) {
  if (!args.has("--quiet")) {
    console.error(`[basic-build-compare] ${message}`);
  }
}

async function ensureExecutable(label, executable) {
  try {
    await access(executable, fsConstants.X_OK);
  } catch (err) {
    throw new Error(`${label} executable is not available at ${executable}: ${err.message}`);
  }
}

async function firstExecutablePath(candidates) {
  for (const candidate of candidates) {
    try {
      await access(path.resolve(repoRoot, candidate), fsConstants.X_OK);
      return candidate;
    } catch {
      // Try the next build profile.
    }
  }
  return candidates[0];
}

async function runCommand(run) {
  const started = process.hrtime.bigint();
  const stdout = [];
  const stderr = [];
  let timedOut = false;
  const env = { ...process.env, CI: "1" };
  if (nodeBin) {
    env.PATH = `${nodeBin}${path.delimiter}${env.PATH ?? ""}`;
  }
  const child = spawn(run.command[0], run.command.slice(1), {
    cwd: run.cwd,
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const timer = setTimeout(() => {
    timedOut = true;
    child.kill("SIGTERM");
    setTimeout(() => child.kill("SIGKILL"), 2_000).unref?.();
  }, run.timeout_ms ?? commandTimeoutMs);
  child.stdout.on("data", (chunk) => stdout.push(chunk));
  child.stderr.on("data", (chunk) => stderr.push(chunk));
  const exitCode = await new Promise((resolve, reject) => {
    child.on("error", reject);
    child.on("close", resolve);
  });
  clearTimeout(timer);
  const ended = process.hrtime.bigint();
  return {
    ...run,
    exit_code: exitCode,
    timed_out: timedOut,
    duration_ms: Number(ended - started) / 1_000_000,
    stdout: Buffer.concat(stdout).toString("utf8"),
    stderr: Buffer.concat(stderr).toString("utf8"),
  };
}

async function fileExists(file) {
  try {
    const info = await stat(file);
    return info.isFile();
  } catch {
    return false;
  }
}

async function startStaticServer(root, artifacts) {
  const normalizedRoot = path.resolve(root);
  const fallbackHtml = generatedHtmlForArtifacts(artifacts);
  const server = createServer(async (req, res) => {
    try {
      const url = new URL(req.url ?? "/", "http://127.0.0.1");
      let pathname = decodeURIComponent(url.pathname);
      if (pathname === "/") {
        pathname = "/index.html";
      }

      if (pathname === "/index.html" && !(await fileExists(path.join(normalizedRoot, "index.html")))) {
        if (!fallbackHtml) {
          res.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
          res.end("missing index.html and JS fallback");
          return;
        }
        res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
        res.end(fallbackHtml);
        return;
      }

      const absolute = path.resolve(normalizedRoot, `.${pathname}`);
      if (absolute !== normalizedRoot && !absolute.startsWith(`${normalizedRoot}${path.sep}`)) {
        res.writeHead(403, { "content-type": "text/plain; charset=utf-8" });
        res.end("forbidden");
        return;
      }

      if (!(await fileExists(absolute))) {
        res.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
        res.end("not found");
        return;
      }

      res.writeHead(200, { "content-type": contentTypeForPath(absolute) });
      res.end(await readFile(absolute));
    } catch (err) {
      res.writeHead(500, { "content-type": "text/plain; charset=utf-8" });
      res.end(String(err.stack ?? err));
    }
  });

  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  return {
    url: `http://127.0.0.1:${address.port}/`,
    close: () => new Promise((resolve) => server.close(resolve)),
  };
}

function generatedHtmlForArtifacts(artifacts) {
  if (artifacts.js_files.length === 0) return null;
  const cssLinks = artifacts.css_files
    .map((rel) => `<link rel="stylesheet" href="/${escapeHtmlAttr(rel)}">`)
    .join("");
  const script = artifacts.js_files[0];
  return `<!DOCTYPE html><html><head><meta charset="utf-8"><title>React Bench</title>${cssLinks}</head><body><div id="root"></div><script src="/${escapeHtmlAttr(script)}"></script></body></html>`;
}

function escapeHtmlAttr(value) {
  return value.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");
}

function contentTypeForPath(file) {
  switch (path.extname(file).toLowerCase()) {
    case ".html":
      return "text/html; charset=utf-8";
    case ".js":
    case ".mjs":
      return "text/javascript; charset=utf-8";
    case ".css":
      return "text/css; charset=utf-8";
    case ".json":
      return "application/json; charset=utf-8";
    case ".svg":
      return "image/svg+xml";
    case ".png":
      return "image/png";
    case ".jpg":
    case ".jpeg":
      return "image/jpeg";
    case ".woff":
      return "font/woff";
    case ".woff2":
      return "font/woff2";
    default:
      return "application/octet-stream";
  }
}

async function walkFiles(root) {
  const out = [];
  async function visit(dir) {
    for (const entry of await readdir(dir, { withFileTypes: true })) {
      const absolute = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        await visit(absolute);
      } else if (entry.isFile()) {
        out.push(absolute);
      }
    }
  }
  await visit(root);
  return out.sort();
}

async function analyzeArtifacts(tool, dir) {
  const files = await walkFiles(dir);
  const fileEntries = [];
  let rawBytes = 0;
  let gzipBytes = 0;
  let brotliBytes = 0;
  let combinedText = "";
  const htmlFiles = [];
  const jsFiles = [];
  const cssFiles = [];

  for (const file of files) {
    const rel = path.relative(dir, file);
    const buf = await readFile(file);
    const ext = path.extname(file).toLowerCase();
    rawBytes += buf.length;
    gzipBytes += gzipSync(buf).length;
    brotliBytes += brotliCompressSync(buf).length;
    if ([".html", ".js", ".css", ".json", ".map", ".txt"].includes(ext)) {
      combinedText += `\n/* ${rel} */\n${buf.toString("utf8")}`;
    }
    if (ext === ".html") htmlFiles.push(rel);
    if (ext === ".js") jsFiles.push(rel);
    if (ext === ".css") cssFiles.push(rel);
    fileEntries.push({
      path: rel,
      bytes: buf.length,
      sha256: createHash("sha256").update(buf).digest("hex"),
    });
  }

  const htmlText = htmlFiles.length > 0 ? await readFile(path.join(dir, htmlFiles[0]), "utf8") : "";
  const htmlShell = {
    present: htmlFiles.length > 0,
    root_mount: /id=["']root["']/.test(htmlText),
    loads_script: /<script\b/i.test(htmlText),
    module_script: /<script\b[^>]*\btype=["']module["']/i.test(htmlText),
    linked_css: /<link\b[^>]*stylesheet/i.test(htmlText),
  };
  const semantic = Object.fromEntries(
    semanticStrings.map((needle) => [needle, combinedText.includes(needle)]),
  );
  const jsSyntax = [];
  for (const rel of jsFiles) {
    const file = path.join(dir, rel);
    const checked = await runCommand({
      name: `${tool}:node-check`,
      cwd: repoRoot,
      command: [process.execPath, "--check", file],
    });
    jsSyntax.push({
      path: rel,
      ok: checked.exit_code === 0,
      stdout: shortOutput(checked.stdout),
      stderr: shortOutput(checked.stderr),
    });
  }
  const semanticOk = Object.values(semantic).every(Boolean);
  const hasJs = jsFiles.length > 0;
  const hasCss = cssFiles.length > 0;
  const syntaxOk = jsSyntax.every((entry) => entry.ok);
  const hasRunnableHtml = htmlShell.present && htmlShell.root_mount && htmlShell.loads_script;
  const requiredPublicPresence = Object.fromEntries(
    requiredPublicFiles.map((required) => [
      required,
      fileEntries.some((entry) => entry.path === required),
    ]),
  );
  const publicOk = Object.values(requiredPublicPresence).every(Boolean);

  let staticResult = "green";
  const missing = [];
  if (!hasJs) missing.push("js_bundle");
  if (!syntaxOk) missing.push("js_syntax");
  if (!semanticOk) missing.push("semantic_strings");
  if (!hasRunnableHtml) missing.push("html_shell");
  if (requireCss && !hasCss) missing.push("css_bundle");
  if (requireCss && !htmlShell.linked_css) missing.push("css_link");
  if (!publicOk) missing.push("public_assets");
  if (missing.length > 0) {
    staticResult = hasJs && syntaxOk && semanticOk ? "yellow" : "red";
  }
  const result = staticResult === "red" ? "red" : "yellow";

  return {
    tool,
    dir,
    files: fileEntries,
    artifacts: {
      file_count: files.length,
      raw_bytes: rawBytes,
      gzip_bytes: gzipBytes,
      brotli_bytes: brotliBytes,
      html_files: htmlFiles,
      js_files: jsFiles,
      css_files: cssFiles,
    },
    functional: {
      result,
      static_result: staticResult,
      checks: {
        js_bundle: hasJs,
        js_syntax: jsSyntax,
        html_shell: htmlShell,
        css_bundle: {
          required: requireCss,
          present: hasCss,
          linked: htmlShell.linked_css,
        },
        public_assets: {
          required: requiredPublicFiles,
          present: requiredPublicPresence,
        },
        semantic_strings: semantic,
        runtime_smoke: "not_run",
      },
      missing,
    },
  };
}

async function applyRuntimeSmokeChecks(entries) {
  if (runtimeSmokeMode === "off") {
    for (const entry of entries) updateFunctionalResult(entry);
    return { mode: runtimeSmokeMode, result: "not_run" };
  }

  for (const entry of entries) {
    progress(`runtime: smoke ${entry.name}`);
    entry.functional.checks.runtime_smoke = await runToolRuntimeSmoke(entry);
    updateFunctionalResult(entry);
  }

  return compareRuntimeTraces(entries);
}

async function runToolRuntimeSmoke(entry) {
  const started = process.hrtime.bigint();
  let server;
  let sessionDir;
  const commands = [];
  try {
    server = await startStaticServer(entry.dir, entry.artifacts);
    sessionDir = path.join(outputRoot, `.browser-${entry.name}`);
    await mkdir(sessionDir, { recursive: true });
    progress(`runtime: ${entry.name} serving ${server.url}`);

    const launch = await runCommand({
      name: `${entry.name}:jet-bb-launch`,
      cwd: sessionDir,
      command: [jetBin, "bb", "launch", server.url],
      timeout_ms: runtimeToolTimeoutMs,
    });
    commands.push(commandEvidence(launch));
    if (launch.exit_code !== 0 || launch.timed_out) {
      return runtimeCommandFailure("launch", launch, started, server);
    }

    const evaluated = await runCommand({
      name: `${entry.name}:jet-bb-eval`,
      cwd: sessionDir,
      command: [jetBin, "bb", "eval", runtimeSmokeExpression()],
      timeout_ms: runtimeToolTimeoutMs,
    });
    commands.push(commandEvidence(evaluated));
    if (evaluated.exit_code !== 0 || evaluated.timed_out) {
      return runtimeCommandFailure("eval", evaluated, started, server, commands);
    }

    let payload;
    try {
      payload = JSON.parse(evaluated.stdout);
    } catch (err) {
      return {
        result: "red",
        served_url: server.url,
        duration_ms: Number(process.hrtime.bigint() - started) / 1_000_000,
        error: `failed to parse jet bb eval JSON: ${err.message}`,
        commands,
        stdout: shortOutput(evaluated.stdout),
      };
    }

    const errors = payload.errors ?? [];
    const trace = payload.trace ?? [];
    const durationMs = Number(process.hrtime.bigint() - started) / 1_000_000;
    if (errors.length > 0) {
      return {
        result: "red",
        served_url: server.url,
        duration_ms: durationMs,
        errors,
        trace,
        commands,
      };
    }

    return {
      result: "green",
      served_url: server.url,
      duration_ms: durationMs,
      trace_hash: hashJson(normalizeRuntimeTrace(trace)),
      trace,
      commands,
    };
  } catch (err) {
    const durationMs = Number(process.hrtime.bigint() - started) / 1_000_000;
    return {
      result: "red",
      served_url: server?.url ?? null,
      duration_ms: durationMs,
      error: String(err.stack ?? err),
      commands,
    };
  } finally {
    if (sessionDir) {
      const shutdown = await runCommand({
        name: `${entry.name}:jet-bb-shutdown`,
        cwd: sessionDir,
        command: [jetBin, "bb", "shutdown"],
        timeout_ms: 5_000,
      }).catch((err) => ({
        name: `${entry.name}:jet-bb-shutdown`,
        command: [jetBin, "bb", "shutdown"],
        exit_code: 1,
        timed_out: false,
        duration_ms: 0,
        stdout: "",
        stderr: String(err.stack ?? err),
      }));
      commands.push(commandEvidence(shutdown));
    }
    if (server) {
      await server.close();
    }
  }
}

function commandEvidence(result) {
  return {
    name: result.name,
    command: result.command.map((part) => path.isAbsolute(part) ? path.relative(repoRoot, part) || "." : part),
    exit_code: result.exit_code,
    timed_out: result.timed_out,
    duration_ms: result.duration_ms,
    stdout: shortOutput(result.stdout),
    stderr: shortOutput(result.stderr),
  };
}

function runtimeCommandFailure(phase, result, started, server, commands = [commandEvidence(result)]) {
  return {
    result: "red",
    phase,
    served_url: server?.url ?? null,
    duration_ms: Number(process.hrtime.bigint() - started) / 1_000_000,
    error: result.timed_out
      ? `${phase} timed out after ${runtimeToolTimeoutMs}ms`
      : `${phase} exited with ${result.exit_code}`,
    commands,
  };
}

function runtimeSmokeExpression() {
if (runtimeCase === "dom-production-assets") {
    return domProductionAssetsRuntimeSmokeExpression();
  }
  if (runtimeCase === "visual-library") {
    return visualLibraryRuntimeSmokeExpression();
  }
  if (runtimeCase !== "react-bench") {
    throw new Error(`unknown --runtime-case ${runtimeCase}`);
  }
  return `
    (async () => {
      const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
      const normalize = (value) => String(value || "").replace(/\\s+/g, " ").trim();
      const waitFor = async (predicate, label, timeoutMs = 5000) => {
        const started = Date.now();
        while (Date.now() - started < timeoutMs) {
          if (predicate()) return;
          await sleep(100);
        }
        throw new Error("timeout waiting for " + label);
      };
      const button = (label) => {
        const found = Array.from(document.querySelectorAll("button"))
          .find((candidate) => normalize(candidate.textContent) === label);
        if (!found) throw new Error("button not found: " + label);
        return found;
      };
      const snapshot = (label) => {
        const h1 = document.querySelector("h1");
        const h2 = document.querySelector("h2");
        return {
          label,
          body_text: normalize(document.body.innerText),
          h1: normalize(h1 && h1.textContent),
          h2: normalize(h2 && h2.textContent),
          buttons: Array.from(document.querySelectorAll("button")).map((item) => normalize(item.textContent)),
          inputs: Array.from(document.querySelectorAll("input")).map((input) => ({
            type: input.type,
            placeholder: input.placeholder || "",
            value: input.value || "",
            checked: Boolean(input.checked)
          })),
          todos: Array.from(document.querySelectorAll("li")).map((item) => {
            const checkbox = item.querySelector("input[type='checkbox']");
            return {
              text: normalize(item.textContent),
              done: Boolean(checkbox && checkbox.checked),
              decoration: getComputedStyle(item).textDecorationLine
            };
          })
        };
      };

      const trace = [];
      await waitFor(() => document.body && document.body.innerText.includes("React Bench") && document.body.innerText.includes("Counter"), "initial app");
      trace.push(snapshot("initial"));
      button("+").click();
      await waitFor(() => document.body.innerText.includes("Counter: 1"), "counter increment");
      trace.push(snapshot("counter_increment"));
      button("Todos").click();
      await waitFor(() => document.body.innerText.includes("Todos (0 remaining)"), "todos page");
      trace.push(snapshot("todos_empty"));
      const input = document.querySelector("input[placeholder='Add todo...']");
      if (!input) throw new Error("todo input not found");
      input.value = "gate todo";
      input.dispatchEvent(new Event("input", { bubbles: true }));
      button("Add").click();
      await waitFor(() => document.body.innerText.includes("gate todo") && document.body.innerText.includes("Todos (1 remaining)"), "todo added");
      trace.push(snapshot("todo_added"));
      const checkbox = document.querySelector("li input[type='checkbox']");
      if (!checkbox) throw new Error("todo checkbox not found");
      checkbox.click();
      await waitFor(() => document.body.innerText.includes("Todos (0 remaining)"), "todo completed");
      trace.push(snapshot("todo_completed"));
      return { errors: [], trace };
    })()
  `;
}

function domProductionAssetsRuntimeSmokeExpression() {
  return `
    (async () => {
      const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
      const normalize = (value) => String(value || "").replace(/\\s+/g, " ").trim();
      const waitFor = async (predicate, label, timeoutMs = 5000) => {
        const started = Date.now();
        while (Date.now() - started < timeoutMs) {
          if (await predicate()) return;
          await sleep(100);
        }
        throw new Error("timeout waiting for " + label);
      };
      const textFor = (selector) => normalize(document.querySelector(selector)?.textContent);
      const trace = [];
      await waitFor(() => textFor("h1") === "DOM Production Assets", "app title");
      await waitFor(() => textFor("[data-testid='mode']") === "Mode: production", "production mode");
      await waitFor(() => textFor("[data-testid='node-env']") === "Build target: production", "node env");
      const shell = document.querySelector(".shell");
      const status = document.querySelector(".status.active");
      const brand = document.querySelector("img.brand");
      if (!shell) throw new Error("shell not found");
      if (!status) throw new Error("status not found");
      if (!brand) throw new Error("brand image not found");
      const publicAssetResponse = await fetch("/brand.svg");
      const publicAssetText = publicAssetResponse.ok ? await publicAssetResponse.text() : "";
      if (!publicAssetResponse.ok) throw new Error("public brand.svg did not load");
      const shellStyle = getComputedStyle(shell);
      const statusStyle = getComputedStyle(status);
      trace.push({
        label: "initial",
        h1: textFor("h1"),
        mode: textFor("[data-testid='mode']"),
        node_env: textFor("[data-testid='node-env']"),
        status: normalize(status.textContent),
        shell_color: shellStyle.color,
        status_border_width: statusStyle.borderTopWidth,
        public_asset_has_svg: publicAssetText.includes("<svg"),
        counter: normalize(document.body.innerText).includes("Asset counter: 0") ? "0" : "missing"
      });
      const button = Array.from(document.querySelectorAll("button"))
        .find((candidate) => normalize(candidate.textContent) === "Increment asset counter");
      if (!button) throw new Error("increment button not found");
      button.click();
      await waitFor(() => document.body.innerText.includes("Asset counter: 1"), "counter increment");
      trace.push({
        label: "counter_increment",
        counter: normalize(document.body.innerText).includes("Asset counter: 1") ? "1" : "missing",
        shell_color: getComputedStyle(shell).color,
        status_border_width: getComputedStyle(status).borderTopWidth
      });
      return { errors: [], trace };
    })()
  `;
}

function visualLibraryRuntimeSmokeExpression() {
  return `
    (async () => {
      const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
      const normalize = (value) => String(value || "").replace(/\\s+/g, " ").trim();
      const waitFor = async (predicate, label, timeoutMs = 8000) => {
        const started = Date.now();
        while (Date.now() - started < timeoutMs) {
          if (await predicate()) return;
          await sleep(100);
        }
        throw new Error("timeout waiting for " + label);
      };
      await waitFor(() => window.__jetVisualFixture, "visual fixture metadata");
      const meta = window.__jetVisualFixture;
      try {
        await waitFor(
          () => document.body.innerText.includes(meta.tableTitle)
            && document.querySelectorAll(".ui-case").length >= meta.minComponentCases,
          "visual fixture render",
        );
      } catch (err) {
        const events = Array.from(window.__jetVisualEvents || []);
        const root = document.getElementById("root");
        throw new Error([
          err.message,
          "body=" + normalize(document.body.innerText).slice(0, 500),
          "root=" + normalize(root?.innerHTML).slice(0, 500),
          "events=" + JSON.stringify(events).slice(0, 1200),
        ].join("; "));
      }
      const firstCell = document.querySelector("#large-table td");
      if (!firstCell) throw new Error("large table first cell not found");
      const primaryButtonText = meta.primaryButtonText
        || (meta.libraryId === "mui" ? "MUI Primary" : "AntD Primary");
      const primaryButton = Array.from(document.querySelectorAll("button"))
        .find((candidate) => normalize(candidate.textContent).includes(primaryButtonText));
      if (!primaryButton) throw new Error("primary button not found");
      const events = Array.from(window.__jetVisualEvents || []);
      if (events.length > 0) {
        throw new Error("visual fixture emitted browser errors: " + JSON.stringify(events));
      }
      const matrix = document.querySelector(".component-matrix");
      const viewport = document.querySelector("#table-viewport");
      const trace = [];
      trace.push({
        label: "initial",
        library_id: meta.libraryId,
        title: normalize(document.querySelector("h1")?.textContent),
        component_cases: document.querySelectorAll(".ui-case").length,
        first_cell: normalize(firstCell.textContent),
        primary_button: normalize(primaryButton.textContent),
        matrix_present: Boolean(matrix),
        table_viewport_overflow: getComputedStyle(viewport).overflow,
        browser_events: events.length,
      });
      primaryButton.click();
      await sleep(100);
      trace.push({
        label: "primary_click",
        library_id: meta.libraryId,
        active_element_tag: document.activeElement ? document.activeElement.tagName.toLowerCase() : "",
        title: normalize(document.querySelector("h1")?.textContent),
        component_cases: document.querySelectorAll(".ui-case").length,
        browser_events: Array.from(window.__jetVisualEvents || []).length,
      });
      return { errors: [], trace };
    })()
  `;
}

function compareRuntimeTraces(entries) {
  const reference = entries.find((entry) => entry.name === "vite");
  const jet = entries.find((entry) => entry.name === "jet");
  if (!reference || reference.functional.checks.runtime_smoke.result !== "green") {
    for (const entry of entries) updateFunctionalResult(entry);
    return {
      mode: runtimeSmokeMode,
      result: runtimeSmokeMode === "required" ? "red" : "yellow",
      reference_tool: "vite",
      reason: "reference_runtime_smoke_not_green",
    };
  }

  const referenceTrace = normalizeRuntimeTrace(reference.functional.checks.runtime_smoke.trace);
  const mismatches = [];
  for (const entry of entries) {
    const runtime = entry.functional.checks.runtime_smoke;
    if (runtime.result !== "green") {
      updateFunctionalResult(entry);
      continue;
    }
    const trace = normalizeRuntimeTrace(runtime.trace);
    const matchesReference = JSON.stringify(trace) === JSON.stringify(referenceTrace);
    runtime.reference_tool = "vite";
    runtime.trace_match_reference = matchesReference;
    runtime.reference_trace_hash = hashJson(referenceTrace);
    if (!matchesReference) {
      mismatches.push(entry.name);
      if (entry.name === "jet") {
        runtime.result = "red";
        runtime.error = "runtime trace diverged from vite reference";
      }
    }
    updateFunctionalResult(entry);
  }

  return {
    mode: runtimeSmokeMode,
    result: jet?.functional.checks.runtime_smoke.result === "green" ? "green" : "red",
    reference_tool: "vite",
    reference_trace_hash: hashJson(referenceTrace),
    mismatches,
  };
}

function normalizeRuntimeTrace(trace) {
  return trace.map((step) => normalizeJsonValue(step));
}

function normalizeJsonValue(value) {
  if (Array.isArray(value)) {
    return value.map((item) => normalizeJsonValue(item));
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .filter((key) => value[key] !== undefined)
        .map((key) => [key, normalizeJsonValue(value[key])]),
    );
  }
  return value;
}

function updateFunctionalResult(entry) {
  const missing = new Set(entry.functional.missing);
  const runtime = entry.functional.checks.runtime_smoke;
  const runtimeResult = typeof runtime === "string" ? runtime : runtime.result;
  if (runtimeSmokeMode !== "off" && runtimeResult !== "green") {
    missing.add("runtime_smoke");
  } else {
    missing.delete("runtime_smoke");
  }

  let result;
  if (entry.functional.static_result === "red" || runtimeResult === "red") {
    result = "red";
  } else if (runtimeSmokeMode === "required" && runtimeResult !== "green") {
    result = "red";
  } else if (entry.functional.static_result === "green" && runtimeResult === "green") {
    result = "green";
  } else {
    result = "yellow";
  }

  entry.functional.result = result;
  entry.functional.missing = [...missing].sort();
}

function hashJson(value) {
  return createHash("sha256").update(JSON.stringify(value)).digest("hex");
}

function performanceIsGreen(durationRatio, gzipRatio) {
  return durationRatio <= durationRatioMax && gzipRatio <= gzipRatioMax;
}

function overallResultFor({
  jetFunctionalGreen,
  jetStaticFunctionalGreen,
  performanceGreen,
  jetFunctionalResult,
  runtimeComparisonResult,
}) {
  if (runtimeSmokeMode === "required" && runtimeComparisonResult === "red") {
    return "red";
  }
  if (jetFunctionalGreen && performanceGreen) {
    return "green";
  }
  if (jetFunctionalResult === "red" || !jetStaticFunctionalGreen || !performanceGreen) {
    return "red";
  }
  return "yellow";
}

function shortOutput(text) {
  const lines = text.trim().split(/\r?\n/).filter(Boolean);
  if (lines.length <= 20) return lines;
  return [...lines.slice(0, 10), "...", ...lines.slice(-10)];
}

async function runContractSelfTest() {
  const failures = [];
  const assert = (condition, message) => {
    if (!condition) failures.push(message);
  };

  assert(runtimeSmokeMode === "required", "--self-test must run with --runtime-smoke required");
  assert(requireCss, "--self-test must run with --require-css");
  assert(
    requiredPublicFiles.includes("brand.svg"),
    "--self-test must run with --require-public brand.svg",
  );
  assert(
    semanticStrings.includes("DOM Production Assets"),
    "--self-test must include the DOM Production Assets semantic string",
  );

  const tempRoot = await mkdtemp(path.join(tmpdir(), "jet-basic-build-contract-"));
  try {
    const complete = path.join(tempRoot, "complete");
    await mkdir(complete, { recursive: true });
    await writeFile(
      path.join(complete, "index.html"),
      '<!doctype html><div id="root"></div><link rel="stylesheet" href="/style.css"><script type="module" src="/main.js"></script>',
    );
    await writeFile(
      path.join(complete, "main.js"),
      'const title = "DOM Production Assets"; console.log(title);',
    );
    await writeFile(path.join(complete, "style.css"), ".shell { color: rgb(1, 2, 3); }");
    await writeFile(path.join(complete, "brand.svg"), "<svg></svg>");

    const completeAnalysis = await analyzeArtifacts("jet", complete);
    assert(
      completeAnalysis.functional.static_result === "green",
      `complete artifact set should be statically green: ${JSON.stringify(completeAnalysis.functional)}`,
    );

    const missingCss = path.join(tempRoot, "missing-css");
    await cp(complete, missingCss, { recursive: true });
    await rm(path.join(missingCss, "style.css"));
    await writeFile(
      path.join(missingCss, "index.html"),
      '<!doctype html><div id="root"></div><script type="module" src="/main.js"></script>',
    );
    const missingCssAnalysis = await analyzeArtifacts("jet", missingCss);
    assert(
      missingCssAnalysis.functional.static_result !== "green"
        && missingCssAnalysis.functional.missing.includes("css_bundle")
        && missingCssAnalysis.functional.missing.includes("css_link"),
      `missing CSS must not be green: ${JSON.stringify(missingCssAnalysis.functional)}`,
    );

    const missingPublic = path.join(tempRoot, "missing-public");
    await cp(complete, missingPublic, { recursive: true });
    await rm(path.join(missingPublic, "brand.svg"));
    const missingPublicAnalysis = await analyzeArtifacts("jet", missingPublic);
    assert(
      missingPublicAnalysis.functional.static_result !== "green"
        && missingPublicAnalysis.functional.missing.includes("public_assets"),
      `missing public asset must not be green: ${JSON.stringify(missingPublicAnalysis.functional)}`,
    );

    const runtimeEntry = (name, trace) => ({
      name,
      functional: {
        result: "yellow",
        static_result: "green",
        missing: [],
        checks: {
          runtime_smoke: {
            result: "green",
            trace,
          },
        },
      },
    });
    const referenceTrace = [
      {
        label: "initial",
        h1: "DOM Production Assets",
        mode: "Mode: production",
        shell_color: "rgb(10, 20, 30)",
        public_asset_has_svg: true,
      },
    ];
    const divergentTrace = [
      {
        label: "initial",
        h1: "DOM Production Assets",
        mode: "Mode: production",
        shell_color: "rgb(200, 20, 30)",
        public_asset_has_svg: true,
      },
    ];
    const jetRuntime = runtimeEntry("jet", divergentTrace);
    const runtimeComparison = compareRuntimeTraces([
      runtimeEntry("vite", referenceTrace),
      jetRuntime,
      runtimeEntry("webpack", referenceTrace),
    ]);
    assert(
      runtimeComparison.result === "red"
        && runtimeComparison.mismatches.includes("jet")
        && jetRuntime.functional.checks.runtime_smoke.result === "red",
      `runtime trace divergence must be red: ${JSON.stringify(runtimeComparison)}`,
    );
    assert(
      overallResultFor({
        jetFunctionalGreen: true,
        jetStaticFunctionalGreen: true,
        performanceGreen: true,
        jetFunctionalResult: "green",
        runtimeComparisonResult: "red",
      }) === "red",
      "required runtime comparison red must force overall red",
    );

    assert(
      !performanceIsGreen(durationRatioMax + 0.001, 1.0),
      "duration regression above threshold must fail performance",
    );
    assert(
      !performanceIsGreen(1.0, gzipRatioMax + 0.001),
      "gzip regression above threshold must fail performance",
    );
    assert(
      overallResultFor({
        jetFunctionalGreen: true,
        jetStaticFunctionalGreen: true,
        performanceGreen: false,
        jetFunctionalResult: "green",
        runtimeComparisonResult: "green",
      }) === "red",
      "performance regression must force overall red",
    );
  } finally {
    await rm(tempRoot, { recursive: true, force: true });
  }

  if (failures.length > 0) {
    console.error(`basic build contract self-test failed:\n- ${failures.join("\n- ")}`);
    process.exit(1);
  }
  console.log("basic build contract self-test: green");
}

if (selfTestMode) {
  await runContractSelfTest();
  process.exit(0);
}

await ensureExecutable("jet", jetBin);
if (toolNames.includes("vite")) {
  await ensureExecutable("vite", path.join(nodeBin, "vite"));
}
if (toolNames.includes("webpack")) {
  await ensureExecutable("webpack", path.join(nodeBin, "webpack"));
}
await mkdir(outputRoot, { recursive: true });
const fixture = await materializeFixtureRoot(fixtureSource, dependencyRoot, outputRoot, fixtureName);

const runByName = {
  jet: {
    name: "jet",
    cwd: fixture,
    output_dir: path.join(outputRoot, "jet"),
    command_for_output_dir: (outputDir) => [
      jetBin,
      "build",
      "--sourcemap",
      "none",
      "-o",
        outputDir,
    ],
  },
  vite: {
    name: "vite",
    cwd: fixture,
    output_dir: path.join(outputRoot, "vite"),
    command_for_output_dir: (outputDir) => [
      path.join(nodeBin, "vite"),
      "build",
      "--outDir",
      outputDir,
      "--emptyOutDir",
    ],
  },
  webpack: {
    name: "webpack",
    cwd: fixture,
    output_dir: path.join(outputRoot, "webpack"),
    command_for_output_dir: (outputDir) => [
      path.join(nodeBin, "webpack"),
      "--config",
      "webpack.config.cjs",
      "--output-path",
      outputDir,
    ],
  },
};
const runs = toolNames.map((name) => runByName[name]);

const executed = [];
for (const run of runs) {
  let best = null;
  const benchmarkSamples = [];
  for (let sampleIndex = 1; sampleIndex <= buildSamples; sampleIndex += 1) {
    const sampleOutputDir =
      buildSamples === 1
        ? run.output_dir
        : path.join(outputRoot, ".samples", run.name, String(sampleIndex));
    await rm(sampleOutputDir, { recursive: true, force: true });
    const sampleRun = {
      ...run,
      output_dir: sampleOutputDir,
      command: run.command_for_output_dir(sampleOutputDir),
      sample_index: sampleIndex,
    };
    progress(`build: ${run.name} sample=${sampleIndex}/${buildSamples}`);
    const result = await runCommand(sampleRun);
    progress(
      `build: ${run.name} sample=${sampleIndex}/${buildSamples} exit=${result.exit_code}${result.timed_out ? " timeout" : ""}`
    );
    benchmarkSamples.push({
      sample_index: sampleIndex,
      duration_ms: result.duration_ms,
      exit_code: result.exit_code,
      timed_out: result.timed_out,
      stdout: shortOutput(result.stdout),
      stderr: shortOutput(result.stderr),
    });
    if (result.exit_code !== 0) {
      const evidence = {
        contract_id: "basic.build.production",
        result: "red",
        fixture: path.relative(repoRoot, fixtureSource),
        fixture_name: fixtureName,
        output_root: outputRoot,
        failed_tool: run.name,
        command: result.command,
        exit_code: result.exit_code,
        timed_out: result.timed_out,
        sample_index: sampleIndex,
        build_samples: buildSamples,
        stdout: shortOutput(result.stdout),
        stderr: shortOutput(result.stderr),
      };
      await mkdir(path.dirname(evidencePath), { recursive: true });
      await writeFile(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`);
      console.log(JSON.stringify(evidence, null, 2));
      process.exit(1);
    }
    if (!best || result.duration_ms < best.duration_ms) {
      best = result;
    }
  }
  if (buildSamples > 1) {
    await rm(run.output_dir, { recursive: true, force: true });
    await cp(best.output_dir, run.output_dir, { recursive: true });
  }
  executed.push({
    ...best,
    output_dir: run.output_dir,
    command: run.command_for_output_dir(run.output_dir),
    selected_sample_index: best.sample_index,
    benchmark_samples: benchmarkSamples,
  });
}

const analyzed = [];
for (const run of executed) {
  progress(`analyze: ${run.name}`);
  analyzed.push({
    name: run.name,
    command: run.command.map((part) => path.isAbsolute(part) ? path.relative(repoRoot, part) || "." : part),
    duration_ms: run.duration_ms,
    selected_sample_index: run.selected_sample_index,
    benchmark_samples: run.benchmark_samples,
    stdout: shortOutput(run.stdout),
    stderr: shortOutput(run.stderr),
    ...(await analyzeArtifacts(run.name, run.output_dir)),
  });
}

const runtimeComparison = await applyRuntimeSmokeChecks(analyzed);
const byName = Object.fromEntries(analyzed.map((entry) => [entry.name, entry]));
const baselines = toolNames.filter((name) => name !== "jet").map((name) => byName[name]);
const fastestBaseline = baselines.reduce((best, item) =>
  item.duration_ms < best.duration_ms ? item : best
);
const smallestBaselineGzip = baselines.reduce((best, item) =>
  item.artifacts.gzip_bytes < best.artifacts.gzip_bytes ? item : best
);
const jet = byName.jet;
const durationRatio = jet.duration_ms / fastestBaseline.duration_ms;
const gzipRatio = jet.artifacts.gzip_bytes / smallestBaselineGzip.artifacts.gzip_bytes;
const jetStaticFunctionalGreen = jet.functional.static_result === "green";
const jetFunctionalGreen = jet.functional.result === "green";
const performanceGreen = performanceIsGreen(durationRatio, gzipRatio);
const overallResult = overallResultFor({
  jetFunctionalGreen,
  jetStaticFunctionalGreen,
  performanceGreen,
  jetFunctionalResult: jet.functional.result,
  runtimeComparisonResult: runtimeComparison.result,
});

const evidence = {
  contract_id: "basic.build.production",
  result: overallResult,
  fixture: path.relative(repoRoot, fixtureSource),
  fixture_source: path.relative(repoRoot, fixtureSource),
  dependency_root: path.relative(repoRoot, dependencyRoot),
  tool_root: path.relative(repoRoot, toolRoot),
  fixture_name: fixtureName,
  tools_selected: toolNames,
  generated_at: new Date().toISOString(),
  machine: {
    platform: process.platform,
    arch: process.arch,
    node: process.version,
  },
  output_root: outputRoot,
  note: runtimeSmokeMode === "off"
    ? "Runtime browser smoke is disabled for this run; pass --runtime-smoke required to turn this into a green/red gate."
    : "Runtime browser smoke runs deterministic fixture assertions and compares Jet's trace against the Vite reference trace.",
  tools: analyzed,
  comparison: {
    fastest_baseline: fastestBaseline.name,
    smallest_gzip_baseline: smallestBaselineGzip.name,
    jet_duration_ratio_to_fastest_baseline: Number(durationRatio.toFixed(3)),
    jet_gzip_ratio_to_smallest_baseline: Number(gzipRatio.toFixed(3)),
    thresholds: {
      duration_ratio_max: durationRatioMax,
      gzip_ratio_max: gzipRatioMax,
    },
    build_samples: buildSamples,
    performance_result: performanceGreen ? "green" : "red",
    static_functional_result: jetStaticFunctionalGreen ? "green" : jet.functional.static_result,
    functional_result: jet.functional.result,
    runtime_smoke: runtimeComparison,
  },
};

await mkdir(path.dirname(evidencePath), { recursive: true });
await writeFile(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`);
console.log(JSON.stringify(evidence, null, 2));
if (overallResult === "red") {
  process.exit(1);
}

// </HANDWRITE>
