#!/usr/bin/env node
import { brotliCompressSync, gzipSync } from "node:zlib";
import { createHash } from "node:crypto";
import { cp, mkdir, mkdtemp, readFile, readdir, rm, stat, writeFile } from "node:fs/promises";
import { constants as fsConstants } from "node:fs";
import { access } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";
import { createServer } from "node:http";

const __filename = fileURLToPath(import.meta.url);
const repoRoot = path.resolve(path.dirname(__filename), "../../..");

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

const defaultFixture = "projects/jet/tests/fixtures/dom-production-build/react-bench";
const sourceFixture = path.resolve(repoRoot, args.get("--fixture") ?? defaultFixture);
const sourceDependencyRoot = path.resolve(repoRoot, args.get("--dependency-root") ?? sourceFixture);
const sourceToolRoot = path.resolve(repoRoot, args.get("--tool-root") ?? sourceFixture);
const fixtureName = args.get("--fixture-name") ?? path.basename(sourceFixture);
const selectedTools = parseToolList(args.get("--tools") ?? "jet,vite,webpack");
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
const runtimeCase = args.get("--runtime-case") ?? "react-bench";
const requireCssBundle = args.has("--require-css");
const requiredPublicAssets = parseCsv(args.get("--require-public") ?? "");

const semanticStrings = parseCsv(args.get("--semantic-strings") ?? "")
  .filter(Boolean);
if (semanticStrings.length === 0) {
  semanticStrings.push("React Bench", "Counter", "Todos", "Add todo");
}

function parseCsv(value) {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

function parseToolList(value) {
  const tools = parseCsv(value);
  const allowed = new Set(["jet", "vite", "webpack"]);
  for (const tool of tools) {
    if (!allowed.has(tool)) {
      throw new Error(`--tools supports jet,vite,webpack; got ${tool}`);
    }
  }
  if (!tools.includes("jet")) {
    tools.unshift("jet");
  }
  if (tools.length === 0) {
    throw new Error("--tools must select at least one tool");
  }
  return [...new Set(tools)];
}

function normalizeRuntimeSmokeMode(value) {
  if (["off", "optional", "required"].includes(value)) {
    return value;
  }
  throw new Error(`--runtime-smoke must be one of off, optional, or required; got ${value}`);
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
  const child = spawn(run.command[0], run.command.slice(1), {
    cwd: run.cwd,
    env: { ...process.env, CI: "1" },
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

function shouldCopyPackageTreeEntry(source) {
  return ![
    "node_modules",
    "dist",
    "dist-jet",
    "dist-vite",
    "dist-webpack",
    ".vite",
    ".turbo",
    ".next",
    ".cache",
  ].includes(path.basename(source));
}

async function copyPackageTree(source, target) {
  await rm(target, { recursive: true, force: true });
  await cp(source, target, {
    recursive: true,
    filter: shouldCopyPackageTreeEntry,
  });
}

async function copyDependencyLockIfNeeded(target) {
  const targetLock = path.join(target, "jet-lock.yaml");
  if (await fileExists(targetLock)) {
    return false;
  }

  const dependencyLock = path.join(sourceDependencyRoot, "jet-lock.yaml");
  if (!(await fileExists(dependencyLock))) {
    return false;
  }

  await cp(dependencyLock, targetLock);
  return true;
}

async function hydrateWithJetInstall(name, cwd) {
  progress(`pkg: jet install ${name}`);
  return runCommand({
    name: `${name}:jet-install`,
    cwd,
    command: [jetBin, "install", "--frozen-lockfile"],
    timeout_ms: commandTimeoutMs,
  });
}

async function writeSetupFailure(reason, setup) {
  const evidence = {
    contract_id: "basic.build.production",
    result: "red",
    fixture_name: fixtureName,
    fixture: path.relative(repoRoot, sourceFixture),
    dependency_root: path.relative(repoRoot, sourceDependencyRoot),
    tool_root: path.relative(repoRoot, sourceToolRoot),
    output_root: outputRoot,
    phase: "jet_package_hydration",
    reason,
    setup,
  };
  await mkdir(path.dirname(evidencePath), { recursive: true });
  await writeFile(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`);
  console.log(JSON.stringify(evidence, null, 2));
  process.exit(1);
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
  const publicAssets = Object.fromEntries(
    requiredPublicAssets.map((asset) => [
      asset,
      fileEntries.some((entry) => entry.path === asset || entry.path.endsWith(`/${asset}`)),
    ]),
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
  const cssOk = !requireCssBundle || cssFiles.length > 0;
  const publicAssetsOk = Object.values(publicAssets).every(Boolean);
  const hasJs = jsFiles.length > 0;
  const syntaxOk = jsSyntax.every((entry) => entry.ok);
  const hasRunnableHtml = htmlShell.present && htmlShell.root_mount && htmlShell.loads_script;

  let staticResult = "green";
  const missing = [];
  if (!hasJs) missing.push("js_bundle");
  if (!syntaxOk) missing.push("js_syntax");
  if (!semanticOk) missing.push("semantic_strings");
  if (!cssOk) missing.push("css_bundle");
  if (!publicAssetsOk) missing.push("public_assets");
  if (!hasRunnableHtml) missing.push("html_shell");
  if (missing.length > 0) {
    staticResult = hasJs && syntaxOk && semanticOk && cssOk && publicAssetsOk ? "yellow" : "red";
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
          required: requireCssBundle,
          present: cssFiles.length > 0,
        },
        public_assets: publicAssets,
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
    throw new Error(`unsupported --runtime-case: ${runtimeCase}`);
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
  const requiredTexts = JSON.stringify(semanticStrings);
  return `
    (async () => {
      const requiredTexts = ${requiredTexts};
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
      const snapshot = (label) => {
        const body = normalize(document.body && document.body.innerText);
        const img = document.querySelector("img[alt='public asset']");
        return {
          label,
          body_text: requiredTexts.map((text) => text + "=" + body.includes(text)).join(";"),
          h1: normalize(document.querySelector("h1") && document.querySelector("h1").textContent),
          h2: "",
          buttons: Array.from(document.querySelectorAll("button")).map((item) => normalize(item.textContent)),
          inputs: [],
          todos: [
            {
              text: normalize(document.querySelector(".counter") && document.querySelector(".counter").textContent),
              done: Boolean(img && img.complete),
              decoration: getComputedStyle(document.querySelector(".status.active")).borderStyle
            }
          ]
        };
      };

      const trace = [];
      await waitFor(() => requiredTexts.every((text) => document.body.innerText.includes(text)), "required semantic text");
      trace.push(snapshot("initial"));
      const button = Array.from(document.querySelectorAll("button"))
        .find((candidate) => normalize(candidate.textContent) === "Increment asset counter");
      if (!button) throw new Error("increment button not found");
      button.click();
      await waitFor(() => document.body.innerText.includes("Asset counter: 1"), "asset counter increment");
      trace.push(snapshot("asset_counter_increment"));
      return { errors: [], trace };
    })()
  `;
}

function visualLibraryRuntimeSmokeExpression() {
  const requiredTexts = JSON.stringify(semanticStrings);
  return `
    (async () => {
      const requiredTexts = ${requiredTexts};
      const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
      const normalize = (value) => String(value || "").replace(/\\s+/g, " ").trim();
      const waitFor = async (predicate, label, timeoutMs = 7000) => {
        const started = Date.now();
        while (Date.now() - started < timeoutMs) {
          if (predicate()) return;
          await sleep(100);
        }
        throw new Error("timeout waiting for " + label);
      };
      const snapshot = (label) => {
        const body = normalize(document.body && document.body.innerText);
        const uiCases = document.querySelectorAll(".ui-case").length;
        const tableCells = Array.from(document.querySelectorAll("td")).slice(0, 6)
          .map((item) => normalize(item.textContent));
        return {
          label,
          body_text: requiredTexts.map((text) => text + "=" + body.includes(text)).join(";"),
          h1: normalize(document.querySelector("h1") && document.querySelector("h1").textContent),
          h2: normalize(document.querySelector("h2") && document.querySelector("h2").textContent),
          buttons: Array.from(document.querySelectorAll("button")).slice(0, 12).map((item) => normalize(item.textContent)),
          inputs: Array.from(document.querySelectorAll("input")).slice(0, 8).map((input) => ({
            type: input.type,
            placeholder: input.placeholder || "",
            value: input.value || "",
            checked: Boolean(input.checked)
          })),
          todos: [
            {
              text: "uiCases=" + uiCases + ";cells=" + tableCells.join("|"),
              done: body.includes("cell 0") && uiCases >= 4,
              decoration: getComputedStyle(document.querySelector(".component-matrix")).display
            }
          ]
        };
      };

      const trace = [];
      await waitFor(
        () => requiredTexts.every((text) => document.body.innerText.includes(text)) &&
          document.querySelectorAll(".ui-case").length >= 4 &&
          document.body.innerText.includes("cell 0"),
        "visual fixture ready"
      );
      trace.push(snapshot("initial"));
      const primary = Array.from(document.querySelectorAll("button"))
        .find((candidate) => /Primary/.test(normalize(candidate.textContent)));
      if (primary) {
        primary.click();
        await sleep(100);
        trace.push(snapshot("primary_click"));
      }
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
  return trace.map((step) => ({
    label: step.label,
    body_text: step.body_text,
    h1: step.h1,
    h2: step.h2,
    buttons: step.buttons,
    inputs: step.inputs,
    todos: step.todos,
  }));
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

function shortOutput(text) {
  const lines = text.trim().split(/\r?\n/).filter(Boolean);
  if (lines.length <= 20) return lines;
  return [...lines.slice(0, 10), "...", ...lines.slice(-10)];
}

await mkdir(outputRoot, { recursive: true });
await ensureExecutable("jet", jetBin);

const workspaceRoot = path.join(outputRoot, ".workspace");
const fixture = path.join(workspaceRoot, "fixture");
await copyPackageTree(sourceFixture, fixture);
const dependencyLockCopied = await copyDependencyLockIfNeeded(fixture);
const fixtureInstall = await hydrateWithJetInstall("fixture", fixture);
const setup = {
  fixture: {
    source: path.relative(repoRoot, sourceFixture),
    working_dir: path.relative(repoRoot, fixture),
    dependency_root: path.relative(repoRoot, sourceDependencyRoot),
    dependency_lock_copied: dependencyLockCopied,
    install: commandEvidence(fixtureInstall),
  },
  tool_root: null,
};
if (fixtureInstall.exit_code !== 0 || fixtureInstall.timed_out) {
  await writeSetupFailure("fixture_jet_install_failed", setup);
}

const needsBaselineTools = selectedTools.some((tool) => tool === "vite" || tool === "webpack");
let toolFixture = fixture;
if (needsBaselineTools && path.resolve(sourceToolRoot) !== path.resolve(sourceFixture)) {
  toolFixture = path.join(workspaceRoot, "tool-root");
  await copyPackageTree(sourceToolRoot, toolFixture);
  const toolInstall = await hydrateWithJetInstall("tool-root", toolFixture);
  setup.tool_root = {
    source: path.relative(repoRoot, sourceToolRoot),
    working_dir: path.relative(repoRoot, toolFixture),
    install: commandEvidence(toolInstall),
  };
  if (toolInstall.exit_code !== 0 || toolInstall.timed_out) {
    await writeSetupFailure("tool_root_jet_install_failed", setup);
  }
} else if (needsBaselineTools) {
  setup.tool_root = {
    source: path.relative(repoRoot, sourceToolRoot),
    working_dir: path.relative(repoRoot, toolFixture),
    install: commandEvidence(fixtureInstall),
  };
}

const nodeBin = path.join(toolFixture, "node_modules/.bin");
if (selectedTools.includes("vite")) {
  await ensureExecutable("vite", path.join(nodeBin, "vite"));
}
if (selectedTools.includes("webpack")) {
  await ensureExecutable("webpack", path.join(nodeBin, "webpack"));
}

const runDefinitions = {
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

const runs = selectedTools.map((tool) => runDefinitions[tool]);

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
        fixture: path.relative(repoRoot, fixture),
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
const baselines = [byName.vite, byName.webpack].filter(Boolean);
const fastestBaseline = baselines.reduce(
  (best, item) => !best || item.duration_ms < best.duration_ms ? item : best,
  null,
);
const smallestBaselineGzip = baselines.reduce(
  (best, item) => !best || item.artifacts.gzip_bytes < best.artifacts.gzip_bytes ? item : best,
  null,
);
const jet = byName.jet;
const durationRatio = fastestBaseline ? jet.duration_ms / fastestBaseline.duration_ms : Number.POSITIVE_INFINITY;
const gzipRatio = smallestBaselineGzip
  ? jet.artifacts.gzip_bytes / smallestBaselineGzip.artifacts.gzip_bytes
  : Number.POSITIVE_INFINITY;
const jetStaticFunctionalGreen = jet.functional.static_result === "green";
const jetFunctionalGreen = jet.functional.result === "green";
const performanceGreen = baselines.length > 0 && durationRatio <= 1.25 && gzipRatio <= 1.05;
const overallResult =
  jetFunctionalGreen && performanceGreen
    ? "green"
    : (jet.functional.result === "red" || !jetStaticFunctionalGreen || !performanceGreen)
      ? "red"
      : "yellow";

const evidence = {
  contract_id: "basic.build.production",
  result: overallResult,
  fixture_name: fixtureName,
  fixture: path.relative(repoRoot, sourceFixture),
  working_fixture: path.relative(repoRoot, fixture),
  dependency_root: path.relative(repoRoot, sourceDependencyRoot),
  tool_root: path.relative(repoRoot, sourceToolRoot),
  generated_at: new Date().toISOString(),
  machine: {
    platform: process.platform,
    arch: process.arch,
    node: process.version,
  },
  output_root: outputRoot,
  setup,
  note: runtimeSmokeMode === "off"
    ? "Runtime browser smoke is disabled for this run; pass --runtime-smoke required to turn this into a green/red gate."
    : "Runtime browser smoke runs deterministic fixture assertions and compares Jet's trace against the Vite reference trace.",
  tools: analyzed,
  comparison: {
    fastest_baseline: fastestBaseline?.name ?? null,
    smallest_gzip_baseline: smallestBaselineGzip?.name ?? null,
    jet_duration_ratio_to_fastest_baseline: Number(durationRatio.toFixed(3)),
    jet_gzip_ratio_to_smallest_baseline: Number(gzipRatio.toFixed(3)),
    thresholds: {
      duration_ratio_max: 1.25,
      gzip_ratio_max: 1.05,
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
