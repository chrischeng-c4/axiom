#!/usr/bin/env node
// <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-scripts-compare-prod-static-serve-mjs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
import { spawn, spawnSync } from "node:child_process";
import fsSync from "node:fs";
import { access, mkdir, rm, writeFile } from "node:fs/promises";
import http from "node:http";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const repoRoot = path.resolve(path.dirname(__filename), "../../..");

const args = new Map();
for (let i = 2; i < process.argv.length; i += 1) {
  const arg = process.argv[i];
  if (!arg.startsWith("--")) throw new Error(`unexpected positional argument: ${arg}`);
  const next = process.argv[i + 1];
  if (!next || next.startsWith("--")) {
    args.set(arg, "true");
  } else {
    args.set(arg, next);
    i += 1;
  }
}

const outputRoot = path.resolve(repoRoot, args.get("--out-dir") ?? "/tmp/jet-basic-dom-gate/prod-static");
const evidencePath = path.resolve(
  repoRoot,
  args.get("--evidence") ?? "/tmp/jet-basic-dom-gate/prod-static-serve.json",
);
const jetBin = path.resolve(repoRoot, args.get("--jet-bin") ?? "target/release/jet");
const requireNginx = args.get("--allow-missing-nginx") !== "true";
const requestCount = Number(args.get("--requests") ?? "240");
const concurrency = Number(args.get("--concurrency") ?? "16");
const nginxMode = args.get("--nginx-mode") ?? "auto";
const nginxImage = args.get("--nginx-image") ?? "nginx:1.27-alpine";

function progress(message) {
  if (!args.has("--quiet")) console.error(`[prod-static] ${message}`);
}

async function exists(file) {
  try {
    await access(file);
    return true;
  } catch {
    return false;
  }
}

function findOnPath(name, override) {
  if (override) return override;
  for (const dir of (process.env.PATH ?? "").split(path.delimiter)) {
    const candidate = path.join(dir, name);
    try {
      const stat = fsSync.statSync(candidate);
      if (stat.isFile()) return candidate;
    } catch {
      // keep scanning
    }
  }
  return null;
}

async function freePort() {
  const server = net.createServer();
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const port = server.address().port;
  await new Promise((resolve) => server.close(resolve));
  return port;
}

async function makeFixture(root) {
  const dist = path.join(root, "dist");
  await mkdir(path.join(dist, "assets"), { recursive: true });
  const js = `
document.querySelector("#app").textContent = "Jet prod static fixture ready";
window.__JET_PROD_STATIC_READY__ = true;
`;
  const repeated = "0123456789abcdef".repeat(4096);
  await writeFile(
    path.join(dist, "index.html"),
    `<!doctype html><html><head><title>Jet prod static</title><script type="module" src="/assets/app.1234abcd.js"></script></head><body><main id="app">loading</main></body></html>`,
  );
  await writeFile(path.join(dist, "assets", "app.1234abcd.js"), `${js}\n/* ${repeated} */\n`);
  await writeFile(path.join(dist, "brand.svg"), `<svg xmlns="http://www.w3.org/2000/svg"></svg>`);
  return dist;
}

function request(baseUrl, method, pathname, headers = {}) {
  return new Promise((resolve, reject) => {
    const url = new URL(pathname, baseUrl);
    const started = process.hrtime.bigint();
    const req = http.request(
      url,
      { method, headers, agent: false },
      (res) => {
        const firstByteMs = Number(process.hrtime.bigint() - started) / 1_000_000;
        const chunks = [];
        let bytes = 0;
        res.on("data", (chunk) => {
          bytes += chunk.length;
          chunks.push(chunk);
        });
        res.on("end", () => {
          resolve({
            status: res.statusCode,
            headers: res.headers,
            first_byte_ms: firstByteMs,
            bytes,
            body: Buffer.concat(chunks).toString("utf8"),
          });
        });
      },
    );
    req.on("error", reject);
    req.end();
  });
}

async function waitFor(baseUrl) {
  const deadline = Date.now() + 10_000;
  let last = null;
  while (Date.now() < deadline) {
    try {
      const res = await request(baseUrl, "GET", "/__jet_ready");
      if (res.status === 200) return;
      last = `status=${res.status}`;
    } catch (err) {
      last = err.message;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`server not ready at ${baseUrl}: ${last}`);
}

async function startJet(root) {
  if (!(await exists(jetBin))) throw new Error(`jet binary not found: ${jetBin}`);
  const child = spawn(jetBin, ["serve", "--prod", "--host", "127.0.0.1", "--port", "0"], {
    cwd: root,
    env: { ...process.env, JET_SERVE_CHILD: "1" },
    stdio: ["ignore", "pipe", "pipe"],
  });
  const logs = [];
  child.stderr.on("data", (chunk) => logs.push(chunk.toString("utf8")));
  const port = await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`jet listen timeout:\n${logs.join("")}`)), 10_000);
    child.stdout.on("data", (chunk) => {
      const text = chunk.toString("utf8");
      logs.push(text);
      const match = text.match(/jet-prod-server:listening\s+(\{.*\})/);
      if (match) {
        clearTimeout(timer);
        resolve(JSON.parse(match[1]).port);
      }
    });
    child.once("exit", (code) => {
      clearTimeout(timer);
      reject(new Error(`jet exited before listening code=${code}\n${logs.join("")}`));
    });
  });
  const baseUrl = `http://127.0.0.1:${port}/`;
  await waitFor(baseUrl);
  return {
    name: "jet",
    baseUrl,
    logs,
    async stop() {
      try {
        await request(baseUrl, "POST", "/__jet_shutdown");
      } catch {
        child.kill("SIGTERM");
      }
      await new Promise((resolve) => child.once("exit", resolve));
    },
  };
}

async function startNginx(root, dist) {
  if (nginxMode !== "auto" && nginxMode !== "binary" && nginxMode !== "docker") {
    throw new Error(`unsupported --nginx-mode ${nginxMode}; expected auto, binary, or docker`);
  }

  const nginx = nginxMode === "docker" ? null : findOnPath("nginx", args.get("--nginx-bin"));
  if (nginx) return startNginxBinary(root, dist, nginx);

  const docker = nginxMode === "binary" ? null : findOnPath("docker", args.get("--docker-bin"));
  if (docker) return startNginxDocker(root, dist, docker);

  return null;
}

function nginxConfig(listen, rootDirective) {
  return `
events { worker_connections 1024; }
http {
  access_log off;
  error_log stderr notice;
  etag on;
  types {
    text/html html;
    text/javascript js mjs;
    text/css css;
    image/svg+xml svg;
    application/wasm wasm;
  }
  default_type application/octet-stream;
  server {
    listen ${listen};
    root ${rootDirective};
    location = /__jet_health { add_header Cache-Control "no-store" always; return 200 "ok\\n"; }
    location = /__jet_ready { add_header Cache-Control "no-store" always; return 200 "ready\\n"; }
    location = / { add_header Cache-Control "no-cache" always; try_files /index.html =404; }
    location = /index.html { add_header Cache-Control "no-cache" always; try_files /index.html =404; }
    location ~* \\.[0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]\\. { add_header Cache-Control "public, max-age=31536000, immutable" always; try_files $uri =404; }
    location ~ \\.[^/]+$ { add_header Cache-Control "no-cache" always; try_files $uri =404; }
    location / { add_header Cache-Control "no-cache" always; try_files $uri $uri/ /index.html; }
  }
}
`;
}

async function startNginxBinary(root, dist, nginx) {
  const port = await freePort();
  const prefix = path.join(root, "nginx");
  await mkdir(path.join(prefix, "logs"), { recursive: true });
  const conf = path.join(prefix, "nginx.conf");
  await writeFile(conf, nginxConfig(`127.0.0.1:${port}`, dist));
  const child = spawn(nginx, ["-p", `${prefix}/`, "-c", conf, "-g", "daemon off;"], {
    cwd: root,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const logs = [];
  child.stdout.on("data", (chunk) => logs.push(chunk.toString("utf8")));
  child.stderr.on("data", (chunk) => logs.push(chunk.toString("utf8")));
  const baseUrl = `http://127.0.0.1:${port}/`;
  await waitFor(baseUrl);
  return {
    name: "nginx-binary",
    baseUrl,
    logs,
    async stop() {
      child.kill("SIGTERM");
      await new Promise((resolve) => child.once("exit", resolve));
    },
  };
}

async function startNginxDocker(root, dist, docker) {
  const port = await freePort();
  const prefix = path.join(root, "nginx-docker");
  await mkdir(prefix, { recursive: true });
  const conf = path.join(prefix, "nginx.conf");
  const containerName = `jet-prod-nginx-${process.pid}-${Date.now()}`;
  await writeFile(conf, nginxConfig("8080", "/usr/share/nginx/html"));
  const child = spawn(
    docker,
    [
      "run",
      "--rm",
      "--name",
      containerName,
      "-p",
      `127.0.0.1:${port}:8080`,
      "-v",
      `${conf}:/etc/nginx/nginx.conf:ro`,
      "-v",
      `${dist}:/usr/share/nginx/html:ro`,
      nginxImage,
    ],
    {
      cwd: root,
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  const logs = [];
  child.stdout.on("data", (chunk) => logs.push(chunk.toString("utf8")));
  child.stderr.on("data", (chunk) => logs.push(chunk.toString("utf8")));
  const baseUrl = `http://127.0.0.1:${port}/`;
  try {
    await waitFor(baseUrl);
  } catch (err) {
    child.kill("SIGTERM");
    throw new Error(`docker nginx did not become ready: ${err.message}\n${logs.join("")}`);
  }
  return {
    name: "nginx-docker",
    baseUrl,
    logs,
    async stop() {
      spawnSync(docker, ["rm", "-f", containerName], { stdio: "ignore" });
      await new Promise((resolve) => child.once("exit", resolve));
    },
  };
}

function check(condition, name, details = {}) {
  return { name, ok: Boolean(condition), ...details };
}

function markNginxBaselineSkipped(evidence, reason, error = null) {
  evidence.nginx_baseline = {
    result: "skipped",
    reason,
  };
  if (error) evidence.nginx_baseline.error = error.stack || error.message;
  evidence.result = evidence.jet_protocol?.ok ? "gray" : "red";
  evidence.reason = reason;
}

async function protocolChecks(server) {
  const checks = [];
  const index = await request(server.baseUrl, "GET", "/");
  checks.push(check(index.status === 200, "index_status", { status: index.status }));
  checks.push(check((index.headers["cache-control"] ?? "").includes("no-cache"), "index_no_cache"));
  checks.push(check(Boolean(index.headers.etag || index.headers["last-modified"]), "index_revalidation_header"));

  const asset = await request(server.baseUrl, "GET", "/assets/app.1234abcd.js");
  checks.push(check(asset.status === 200, "asset_status", { status: asset.status }));
  checks.push(check((asset.headers["cache-control"] ?? "").includes("immutable"), "hashed_asset_immutable"));
  checks.push(check((asset.headers["accept-ranges"] ?? "") === "bytes", "asset_accept_ranges"));

  const etag = asset.headers.etag;
  if (etag) {
    const cached = await request(server.baseUrl, "GET", "/assets/app.1234abcd.js", {
      "if-none-match": etag,
    });
    checks.push(check(cached.status === 304, "asset_if_none_match", { status: cached.status }));
  } else {
    checks.push(check(false, "asset_if_none_match", { reason: "missing etag" }));
  }

  const range = await request(server.baseUrl, "GET", "/assets/app.1234abcd.js", {
    range: "bytes=0-3",
  });
  checks.push(check(range.status === 206, "asset_range_status", { status: range.status }));
  checks.push(check(String(range.headers["content-range"] ?? "").startsWith("bytes 0-3/"), "asset_range_header"));
  checks.push(check(range.bytes === 4, "asset_range_body_length", { bytes: range.bytes }));

  const fallback = await request(server.baseUrl, "GET", "/dashboard");
  checks.push(check(fallback.status === 200 && fallback.body.includes("Jet prod static"), "spa_fallback"));

  const missing = await request(server.baseUrl, "GET", "/missing.js");
  checks.push(check(missing.status === 404, "asset_missing_404", { status: missing.status }));

  const health = await request(server.baseUrl, "GET", "/__jet_health");
  checks.push(check(health.status === 200 && health.body === "ok\n", "health_endpoint"));

  const ready = await request(server.baseUrl, "GET", "/__jet_ready");
  checks.push(check(ready.status === 200 && ready.body === "ready\n", "ready_endpoint"));

  return {
    ok: checks.every((item) => item.ok),
    checks,
  };
}

function percentile(values, p) {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.min(sorted.length - 1, Math.floor((p / 100) * sorted.length))] ?? 0;
}

async function benchmark(server) {
  for (let i = 0; i < 20; i += 1) {
    await request(server.baseUrl, "GET", "/assets/app.1234abcd.js");
  }
  const latencies = [];
  let completed = 0;
  let totalBytes = 0;
  const started = process.hrtime.bigint();
  async function worker() {
    while (completed < requestCount) {
      completed += 1;
      const res = await request(server.baseUrl, "GET", "/assets/app.1234abcd.js");
      latencies.push(res.first_byte_ms);
      totalBytes += res.bytes;
    }
  }
  await Promise.all(Array.from({ length: concurrency }, () => worker()));
  const durationMs = Number(process.hrtime.bigint() - started) / 1_000_000;
  return {
    requests: requestCount,
    concurrency,
    duration_ms: durationMs,
    throughput_rps: requestCount / (durationMs / 1000),
    bytes_per_second: totalBytes / (durationMs / 1000),
    first_byte_p50_ms: percentile(latencies, 50),
    first_byte_p95_ms: percentile(latencies, 95),
  };
}

async function main() {
  await rm(outputRoot, { recursive: true, force: true });
  await mkdir(path.dirname(evidencePath), { recursive: true });
  await mkdir(outputRoot, { recursive: true });
  const fixture = path.join(outputRoot, "fixture");
  const dist = await makeFixture(fixture);

  const evidence = {
    contract_id: "basic.serve.prod-static",
    result: "red",
    fixture,
    dist,
    thresholds: {
      first_byte_p95: "jet <= nginx * 1.10",
      throughput: "jet >= nginx",
    },
  };

  let jet = null;
  let nginx = null;
  try {
    progress("start jet");
    jet = await startJet(fixture);
    evidence.jet_protocol = await protocolChecks(jet);
    evidence.jet_benchmark = await benchmark(jet);

    progress("start nginx");
    try {
      nginx = await startNginx(fixture, dist);
    } catch (err) {
      if (requireNginx) throw err;
      markNginxBaselineSkipped(evidence, "nginx baseline unavailable", err);
      nginx = null;
    }
    if (!nginx) {
      if (requireNginx) {
        evidence.result = "red";
        evidence.reason = "nginx baseline not found; install nginx or docker, or pass --allow-missing-nginx";
      } else if (!evidence.nginx_baseline) {
        markNginxBaselineSkipped(evidence, "nginx baseline not found; install nginx or docker to compare throughput");
      }
    } else {
      evidence.nginx_source = nginx.name;
      evidence.nginx_protocol = await protocolChecks(nginx);
      evidence.nginx_benchmark = await benchmark(nginx);

      const firstByteOk =
        evidence.jet_benchmark.first_byte_p95_ms <= evidence.nginx_benchmark.first_byte_p95_ms * 1.1;
      const throughputOk =
        evidence.jet_benchmark.throughput_rps >= evidence.nginx_benchmark.throughput_rps;
      evidence.comparison = {
        first_byte_p95_ratio:
          evidence.jet_benchmark.first_byte_p95_ms / evidence.nginx_benchmark.first_byte_p95_ms,
        throughput_ratio:
          evidence.jet_benchmark.throughput_rps / evidence.nginx_benchmark.throughput_rps,
        first_byte_ok: firstByteOk,
        throughput_ok: throughputOk,
      };
      evidence.result =
        evidence.jet_protocol.ok && evidence.nginx_protocol.ok && firstByteOk && throughputOk
          ? "green"
          : "red";
    }
  } catch (err) {
    evidence.result = "red";
    evidence.error = err.stack || err.message;
  } finally {
    if (nginx) await nginx.stop();
    if (jet) await jet.stop();
    await writeFile(evidencePath, JSON.stringify(evidence, null, 2));
  }
  progress(`result=${evidence.result} evidence=${evidencePath}`);
  process.exit(evidence.result === "green" || evidence.result === "gray" ? 0 : 1);
}

await main();

// </HANDWRITE>
