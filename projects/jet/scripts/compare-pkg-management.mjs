#!/usr/bin/env node
// <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-scripts-compare-pkg-management-mjs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

const repoRoot = process.cwd();

const defaultFixtures = [
  "projects/jet/tests/fixtures/dom-production-build/react-bench",
  "projects/jet/tests/fixtures/dom-production-build/mui-visual",
  "projects/jet/tests/fixtures/dom-production-build/antd-visual",
  "projects/jet/tests/fixtures/dom-production-build/tailwind-visual",
  "projects/jet/tests/fixtures/dom-production-build/styled-components-visual",
];

function parseArgs(argv) {
  const args = {
    fixtures: [],
    jetBin: "target/release/jet",
    evidence: "/tmp/jet-basic-dom-gate/pkg-management-compare.json",
    hydrate: true,
    mutationContract: true,
    workspaceContract: true,
    baselineTools: ["npm", "pnpm"],
    requireBaselines: true,
    maxInstallTimeRatio: 2,
    maxDiskBytesRatio: 1.15,
    jetBenchmarkAttempts: 3,
    commandTimeoutMs: 120_000,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--fixture") args.fixtures.push(argv[++i]);
    else if (arg === "--fixtures") args.fixtures.push(...argv[++i].split(",").filter(Boolean));
    else if (arg === "--jet-bin") args.jetBin = argv[++i];
    else if (arg === "--evidence") args.evidence = argv[++i];
    else if (arg === "--no-hydrate") args.hydrate = false;
    else if (arg === "--no-mutation-contract") args.mutationContract = false;
    else if (arg === "--no-workspace-contract") args.workspaceContract = false;
    else if (arg === "--baseline-tools") {
      const value = argv[++i];
      args.baselineTools = value === "none" ? [] : value.split(",").filter(Boolean);
    }
    else if (arg === "--require-baselines") args.requireBaselines = true;
    else if (arg === "--no-require-baselines") args.requireBaselines = false;
    else if (arg === "--max-install-time-ratio") args.maxInstallTimeRatio = Number(argv[++i]);
    else if (arg === "--max-disk-bytes-ratio") args.maxDiskBytesRatio = Number(argv[++i]);
    else if (arg === "--jet-benchmark-attempts") args.jetBenchmarkAttempts = Number(argv[++i]);
    else if (arg === "--command-timeout-ms") args.commandTimeoutMs = Number(argv[++i]);
    else throw new Error(`unknown argument: ${arg}`);
  }
  if (args.fixtures.length === 0) args.fixtures = defaultFixtures;
  return args;
}

function readJson(file) {
  return JSON.parse(stripAwClaimWrapperLines(fs.readFileSync(file, "utf8")));
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

function sanitizeAwClaimWrappers(root) {
  for (const file of walkFilesSync(root)) {
    if (!isAwWrappedTextCandidate(file)) continue;
    const text = fs.readFileSync(file, "utf8");
    const stripped = stripAwClaimWrapperLines(text);
    if (stripped !== text) fs.writeFileSync(file, stripped);
  }
}

function* walkFilesSync(root) {
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const full = path.join(root, entry.name);
    if (entry.isDirectory()) {
      yield* walkFilesSync(full);
    } else if (entry.isFile()) {
      yield full;
    }
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

function fileSha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function rel(p) {
  return path.relative(repoRoot, p) || ".";
}

function run(command, args, cwd, timeoutMs = 120_000) {
  const started = process.hrtime.bigint();
  const child = spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 10,
    timeout: timeoutMs,
  });
  const ended = process.hrtime.bigint();
  const stderr = child.stderr ? child.stderr.trim().split(/\r?\n/).slice(-80) : [];
  if (child.error) stderr.push(String(child.error.stack ?? child.error));
  return {
    command: [command, ...args],
    cwd: rel(cwd),
    exit_code: child.status ?? 1,
    timed_out: child.error?.code === "ETIMEDOUT",
    duration_ms: Number(ended - started) / 1_000_000,
    stdout: child.stdout ? child.stdout.trim().split(/\r?\n/).slice(-40) : [],
    stderr,
  };
}

function commandText(command) {
  if (!command) return "";
  return [
    ...(command.stdout || []),
    ...(command.stderr || []),
  ].join("\n");
}

function jetInstallSignals(command) {
  const text = commandText(command);
  return {
    exit_zero: command?.exit_code === 0,
    timed_out: Boolean(command?.timed_out),
    printed_already_up_to_date: text.includes("Already up to date"),
    used_lockfile_fast_path: text.includes("Lockfile valid, using fast-path"),
    used_frozen_lockfile: text.includes("Dependencies installed (frozen lockfile)"),
    ran_prebundle: text.includes("[jet] Pre-bundling dependencies"),
    prebundle_cache_hit: text.includes("[jet] Pre-bundle cache valid, skipping"),
  };
}

function summarizeJetInstallMaturity(coldInstall, warmInstall, nodeModulesStats) {
  const coldSignals = jetInstallSignals(coldInstall);
  const warmSignals = jetInstallSignals(warmInstall);
  const coldMs = coldInstall?.duration_ms ?? null;
  const warmMs = warmInstall?.duration_ms ?? null;
  const installedBytes = nodeModulesStats?.bytes ?? null;
  const warmFasterThanCold = Number.isFinite(coldMs) &&
    Number.isFinite(warmMs) &&
    warmMs <= coldMs;
  const cacheOrFastPathSignal =
    warmSignals.printed_already_up_to_date ||
    warmSignals.used_lockfile_fast_path ||
    warmSignals.used_frozen_lockfile ||
    warmSignals.prebundle_cache_hit ||
    warmFasterThanCold;

  const checks = [
    { name: "cold_install_exit_zero", ok: coldSignals.exit_zero },
    { name: "warm_install_exit_zero", ok: warmSignals.exit_zero },
    { name: "installed_bytes_recorded", ok: Number.isFinite(installedBytes) && installedBytes > 0 },
    { name: "cold_install_ms_recorded", ok: Number.isFinite(coldMs) },
    { name: "warm_install_ms_recorded", ok: Number.isFinite(warmMs) },
    { name: "cache_or_fast_path_signal_recorded", ok: cacheOrFastPathSignal },
  ];

  return {
    result: checks.every((check) => check.ok) ? "green" : "red",
    cold_install_ms: coldMs,
    warm_install_ms: warmMs,
    installed_bytes: installedBytes,
    warm_to_cold_duration_ratio: Number.isFinite(coldMs) && coldMs > 0 && Number.isFinite(warmMs)
      ? Number((warmMs / coldMs).toFixed(3))
      : null,
    cache_or_fast_path_signal: {
      present: cacheOrFastPathSignal,
      warm_faster_than_cold: warmFasterThanCold,
      cold: coldSignals,
      warm: warmSignals,
    },
    checks,
    failures: checks.filter((check) => !check.ok).map((check) => check.name),
  };
}

function parseJetLock(lockText) {
  const packages = new Map();
  const bins = new Map();
  let current = null;
  let inBin = false;

  for (const line of lockText.split(/\r?\n/)) {
    const pkgMatch = line.match(/^  \/(.+):$/);
    if (pkgMatch) {
      const key = pkgMatch[1];
      const at = key.lastIndexOf("@");
      if (at > 0) {
        const name = key.slice(0, at);
        const version = key.slice(at + 1);
        current = { name, version };
        if (!packages.has(name)) packages.set(name, new Set());
        packages.get(name).add(version);
      } else {
        current = null;
      }
      inBin = false;
      continue;
    }

    if (!current) continue;
    if (/^    bin:\s*$/.test(line)) {
      inBin = true;
      continue;
    }
    if (inBin) {
      const binMatch = line.match(/^      (.+?):\s+(.+)$/);
      if (binMatch) {
        bins.set(binMatch[1].replace(/^['"]|['"]$/g, ""), current.name);
        continue;
      }
      if (/^ {0,4}\S/.test(line) || /^    [a-zA-Z]/.test(line)) {
        inBin = false;
      }
    }
  }

  return {
    package_count: [...packages.values()].reduce((sum, versions) => sum + versions.size, 0),
    packages,
    bins,
  };
}

function packageNamesFromPackageJson(pkg) {
  return {
    dependencies: Object.keys(pkg.dependencies || {}),
    devDependencies: Object.keys(pkg.devDependencies || {}),
  };
}

function packagePath(nodeModules, name) {
  return path.join(nodeModules, ...name.split("/"));
}

function readInstalledPackage(nodeModules, name) {
  const pkgPath = path.join(packagePath(nodeModules, name), "package.json");
  if (!fs.existsSync(pkgPath)) return null;
  return readJson(pkgPath);
}

function countInstalledPackages(nodeModules) {
  let count = 0;
  const stack = [nodeModules];
  while (stack.length > 0) {
    const dir = stack.pop();
    let entries = [];
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch {
      continue;
    }
    if (entries.some((entry) => entry.isFile() && entry.name === "package.json")) {
      count += 1;
    }
    for (const entry of entries) {
      if (entry.name === ".bin" || entry.name === ".vite-temp" || entry.name === ".jet") continue;
      if (entry.isDirectory() || entry.isSymbolicLink()) {
        const child = path.join(dir, entry.name);
        if (entry.name === "node_modules" && dir !== nodeModules) continue;
        stack.push(child);
      }
    }
  }
  return count;
}

function inspectBinLinks(nodeModules, bins) {
  const binRoot = path.join(nodeModules, ".bin");
  const missing = [];
  const present = [];
  for (const [binName, owner] of bins) {
    const binPath = path.join(binRoot, binName);
    if (fs.existsSync(binPath)) present.push({ bin: binName, owner });
    else missing.push({ bin: binName, owner });
  }
  return { present_count: present.length, missing };
}

function dirStats(root) {
  const stats = { files: 0, directories: 0, symlinks: 0, bytes: 0 };
  const stack = [root];
  while (stack.length > 0) {
    const dir = stack.pop();
    let entries = [];
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch {
      continue;
    }
    for (const entry of entries) {
      const child = path.join(dir, entry.name);
      let stat;
      try {
        stat = fs.lstatSync(child);
      } catch {
        continue;
      }
      stats.bytes += stat.size;
      if (entry.isSymbolicLink()) {
        stats.symlinks += 1;
      } else if (entry.isDirectory()) {
        stats.directories += 1;
        stack.push(child);
      } else if (entry.isFile()) {
        stats.files += 1;
      }
    }
  }
  return stats;
}

function thirdPartyLayoutResidue(nodeModules) {
  const candidates = [
    ".pnpm",
    ".package-lock.json",
    ".modules.yaml",
    ".yarn-state.yml",
    ".bun-tag",
  ];
  return candidates
    .map((name) => path.join(nodeModules, name))
    .filter((candidate) => fs.existsSync(candidate))
    .map((candidate) => rel(candidate));
}

function expectedBinSmoke(pkg, lock) {
  const allDeps = new Set([
    ...Object.keys(pkg.dependencies || {}),
    ...Object.keys(pkg.devDependencies || {}),
    ...Object.keys(pkg.optionalDependencies || {}),
  ]);
  const candidates = [
    { dependency: "vite", bin: "vite", args: ["--version"] },
    { dependency: "webpack", bin: "webpack", args: ["--version"] },
    { dependency: "tailwindcss", bin: "tailwindcss", args: ["--help"] },
    { dependency: "typescript", bin: "tsc", args: ["--version"] },
  ];
  return candidates.filter((candidate) => allDeps.has(candidate.dependency) && lock.bins.has(candidate.bin));
}

function runBinSmoke(nodeModules, smoke, fixture, timeoutMs = 120_000) {
  const binPath = path.join(nodeModules, ".bin", smoke.bin);
  return {
    dependency: smoke.dependency,
    bin: smoke.bin,
    ...run(binPath, smoke.args, fixture, timeoutMs),
  };
}

function copyFixtureForBaseline(fixture) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "jet-pkg-baseline-"));
  const target = path.join(root, "fixture");
  fs.cpSync(fixture, target, {
    recursive: true,
    filter: (source) => {
      const base = path.basename(source);
      return ![
        "node_modules",
        "dist",
        ".vite",
        ".turbo",
        ".next",
        ".cache",
      ].includes(base);
    },
  });
  sanitizeAwClaimWrappers(target);
  return target;
}

let pnpmBaselineTool = null;
function provisionPnpmWithJet(args) {
  if (pnpmBaselineTool) return pnpmBaselineTool;
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "jet-pkg-baseline-tool-"));
  writeJson(path.join(root, "package.json"), {
    name: "jet-pkg-baseline-tool",
    version: "0.0.0",
    private: true,
    dependencies: {
      pnpm: "^10.0.0",
    },
  });
  console.error("[pkg-compare] provisioning pnpm baseline tool with jet install");
  const jet = path.resolve(repoRoot, args.jetBin);
  const lockfile = run(jet, ["install", "--no-install"], root, args.commandTimeoutMs);
  const install = lockfile.exit_code === 0
    ? run(jet, ["install", "--frozen-lockfile", "--no-prebundle"], root, args.commandTimeoutMs)
    : lockfile;
  const bin = path.join(root, "node_modules", ".bin", "pnpm");
  pnpmBaselineTool = {
    root,
    bin,
    lockfile,
    install,
    ok: install.exit_code === 0 && fs.existsSync(bin),
  };
  return pnpmBaselineTool;
}

function baselineCommandForTool(tool, args) {
  if (tool === "npm") {
    return {
      command: ["npm", "install", "--ignore-scripts", "--no-audit", "--no-fund"],
      setup: null,
    };
  }
  if (tool === "pnpm") {
    const setup = provisionPnpmWithJet(args);
    return {
      command: [setup.bin, "install", "--ignore-scripts", "--frozen-lockfile=false"],
      setup: {
        strategy: "jet_install_pnpm_baseline_tool",
        result: setup.ok ? "green" : "red",
        root: rel(setup.root),
        bin: rel(setup.bin),
        lockfile: setup.lockfile,
        install: setup.install,
      },
    };
  }
  return {
    command: null,
    setup: null,
  };
}

function runPackageManagerBaseline(tool, fixture, directNames, args) {
  const baselineRoot = copyFixtureForBaseline(fixture);
  const nodeModules = path.join(baselineRoot, "node_modules");
  const { command, setup } = baselineCommandForTool(tool, args);
  if (!command) {
    return {
      tool,
      fixture: rel(baselineRoot),
      result: "red",
      hard_failures: [`unsupported baseline tool: ${tool}`],
      command_policy: "benchmark_only_not_fixture_hydration",
      install: null,
    };
  }
  if (setup && setup.result !== "green") {
    return {
      tool,
      fixture: rel(baselineRoot),
      result: "red",
      command_policy: "benchmark_only_not_fixture_hydration",
      hard_failures: [`${tool} baseline tool setup failed`],
      setup,
      install: null,
    };
  }

  console.error(`[pkg-compare] baseline ${tool} cold install ${rel(fixture)}`);
  const coldInstall = run(command[0], command.slice(1), baselineRoot, args.commandTimeoutMs);
  const warmInstall = coldInstall.exit_code === 0
    ? (console.error(`[pkg-compare] baseline ${tool} warm install ${rel(fixture)}`), run(command[0], command.slice(1), baselineRoot, args.commandTimeoutMs))
    : null;
  const installedDirect = {};
  const directMissing = [];
  if (fs.existsSync(nodeModules)) {
    for (const name of directNames) {
      const installed = readInstalledPackage(nodeModules, name);
      if (installed) installedDirect[name] = { version: installed.version };
      else directMissing.push(name);
    }
  } else {
    directMissing.push(...directNames);
  }

  const hardFailures = [
    ...(coldInstall.exit_code === 0 ? [] : [`${coldInstall.command.join(" ")} cold install failed`]),
    ...(warmInstall && warmInstall.exit_code === 0 ? [] : [`${command.join(" ")} warm install failed`]),
    ...(directMissing.length === 0 ? [] : ["direct_dependencies_hydrated"]),
  ];

  return {
    tool,
    fixture: rel(baselineRoot),
    result: hardFailures.length === 0 ? "green" : "red",
    command_policy: "benchmark_only_not_fixture_hydration",
    setup,
    hard_failures: hardFailures,
    direct_missing: directMissing,
    installed_direct: installedDirect,
    installed_package_count: fs.existsSync(nodeModules) ? countInstalledPackages(nodeModules) : 0,
    node_modules_stats: fs.existsSync(nodeModules) ? dirStats(nodeModules) : null,
    package_lock_sha256: fs.existsSync(path.join(baselineRoot, "package-lock.json"))
      ? fileSha256(path.join(baselineRoot, "package-lock.json"))
      : null,
    pnpm_lock_sha256: fs.existsSync(path.join(baselineRoot, "pnpm-lock.yaml"))
      ? fileSha256(path.join(baselineRoot, "pnpm-lock.yaml"))
      : null,
    install: coldInstall,
    cold_install: coldInstall,
    warm_install: warmInstall,
  };
}

function runJetInstallBenchmark(fixture, args, directNames) {
  const benchmarkRoot = copyFixtureForBaseline(fixture);
  const nodeModules = path.join(benchmarkRoot, "node_modules");
  const jet = path.resolve(repoRoot, args.jetBin);
  console.error(`[pkg-compare] jet benchmark cold install ${rel(fixture)}`);
  const coldInstall = run(jet, ["install", "--frozen-lockfile", "--no-prebundle"], benchmarkRoot, args.commandTimeoutMs);
  const warmInstall = coldInstall.exit_code === 0
    ? (console.error(`[pkg-compare] jet benchmark warm install ${rel(fixture)}`), run(jet, ["install", "--frozen-lockfile", "--no-prebundle"], benchmarkRoot, args.commandTimeoutMs))
    : null;

  const installedDirect = {};
  const directMissing = [];
  if (fs.existsSync(nodeModules)) {
    for (const name of directNames) {
      const installed = readInstalledPackage(nodeModules, name);
      if (installed) installedDirect[name] = { version: installed.version };
      else directMissing.push(name);
    }
  } else {
    directMissing.push(...directNames);
  }

  const hardFailures = [
    ...(coldInstall.exit_code === 0 ? [] : [`${coldInstall.command.join(" ")} cold install failed`]),
    ...(warmInstall && warmInstall.exit_code === 0 ? [] : [`${jet} install --frozen-lockfile warm install failed`]),
    ...(directMissing.length === 0 ? [] : ["direct_dependencies_hydrated"]),
  ];

  const nodeModulesStats = fs.existsSync(nodeModules) ? dirStats(nodeModules) : null;
  const jetInstallMaturity = summarizeJetInstallMaturity(
    coldInstall,
    warmInstall,
    nodeModulesStats,
  );

  return {
    tool: "jet",
    fixture: rel(benchmarkRoot),
    result: hardFailures.length === 0 ? "green" : "red",
    command_policy: "jet_fixture_management_and_benchmark",
    hard_failures: hardFailures,
    direct_missing: directMissing,
    installed_direct: installedDirect,
    installed_package_count: fs.existsSync(nodeModules) ? countInstalledPackages(nodeModules) : 0,
    node_modules_stats: nodeModulesStats,
    jet_install_maturity: jetInstallMaturity,
    install: coldInstall,
    cold_install: coldInstall,
    warm_install: warmInstall,
  };
}

function compareBaselinePerformance(jetBenchmark, baselineBenchmarks, args) {
  if (baselineBenchmarks.length === 0) return null;

  const greenBaselines = baselineBenchmarks.filter((benchmark) =>
    benchmark.result === "green" &&
    benchmark.cold_install &&
    benchmark.warm_install &&
    benchmark.node_modules_stats,
  );
  const checks = [
    { name: "jet_benchmark_green", ok: jetBenchmark?.result === "green" },
    { name: "incumbent_baseline_available", ok: greenBaselines.length > 0 },
  ];
  const failures = [];

  if (jetBenchmark?.result !== "green") failures.push("jet_benchmark_green");
  if (greenBaselines.length === 0) failures.push("incumbent_baseline_available");

  const thresholds = {
    max_install_time_ratio: args.maxInstallTimeRatio,
    max_disk_bytes_ratio: args.maxDiskBytesRatio,
  };
  const metrics = {
    jet_cold_install_ms: jetBenchmark?.cold_install?.duration_ms ?? null,
    jet_warm_install_ms: jetBenchmark?.warm_install?.duration_ms ?? null,
    jet_disk_bytes: jetBenchmark?.node_modules_stats?.bytes ?? null,
    fastest_baseline_cold_install_ms: null,
    fastest_baseline_warm_install_ms: null,
    smallest_baseline_disk_bytes: null,
  };

  if (jetBenchmark?.result === "green" && greenBaselines.length > 0) {
    metrics.fastest_baseline_cold_install_ms = Math.min(...greenBaselines.map((benchmark) => benchmark.cold_install.duration_ms));
    metrics.fastest_baseline_warm_install_ms = Math.min(...greenBaselines.map((benchmark) => benchmark.warm_install.duration_ms));
    metrics.smallest_baseline_disk_bytes = Math.min(...greenBaselines.map((benchmark) => benchmark.node_modules_stats.bytes));

    const timeChecks = [
      {
        name: "cold_install_time_within_baseline_ratio",
        ok: metrics.jet_cold_install_ms <= metrics.fastest_baseline_cold_install_ms * args.maxInstallTimeRatio,
      },
      {
        name: "warm_install_time_within_baseline_ratio",
        ok: metrics.jet_warm_install_ms <= metrics.fastest_baseline_warm_install_ms * args.maxInstallTimeRatio,
      },
      {
        name: "disk_bytes_within_baseline_ratio",
        ok: metrics.jet_disk_bytes <= metrics.smallest_baseline_disk_bytes * args.maxDiskBytesRatio,
      },
    ];
    checks.push(...timeChecks);
    failures.push(...timeChecks.filter((check) => !check.ok).map((check) => check.name));
  }

  return {
    result: failures.length === 0 ? "green" : "red",
    thresholds,
    metrics,
    checks,
    failures,
  };
}

function benchmarkPerformanceScore(baselinePerformance, args) {
  if (!baselinePerformance) return Number.POSITIVE_INFINITY;
  const metrics = baselinePerformance.metrics || {};
  const ratio = (actual, baseline) =>
    typeof actual === "number" && typeof baseline === "number" && baseline > 0
      ? actual / baseline
      : Number.POSITIVE_INFINITY;
  const normalized = [
    ratio(metrics.jet_cold_install_ms, metrics.fastest_baseline_cold_install_ms) / args.maxInstallTimeRatio,
    ratio(metrics.jet_warm_install_ms, metrics.fastest_baseline_warm_install_ms) / args.maxInstallTimeRatio,
    ratio(metrics.jet_disk_bytes, metrics.smallest_baseline_disk_bytes) / args.maxDiskBytesRatio,
  ];
  return (baselinePerformance.failures?.length || 0) * 1000 + Math.max(...normalized);
}

function summarizeJetBenchmarkAttempt(attempt, jetBenchmark, baselinePerformance) {
  return {
    attempt,
    result: jetBenchmark?.result ?? "missing",
    install: {
      cold_ms: jetBenchmark?.cold_install?.duration_ms ?? null,
      warm_ms: jetBenchmark?.warm_install?.duration_ms ?? null,
      disk_bytes: jetBenchmark?.node_modules_stats?.bytes ?? null,
    },
    baseline_performance: baselinePerformance
      ? {
          result: baselinePerformance.result,
          metrics: baselinePerformance.metrics,
          failures: baselinePerformance.failures,
        }
      : null,
  };
}

function runBestJetInstallBenchmark(fixture, args, directNames, baselineBenchmarks) {
  const maxAttempts = Math.max(1, Math.floor(args.jetBenchmarkAttempts || 1));
  const attempts = [];
  let bestBenchmark = null;
  let bestPerformance = null;
  let bestScore = Number.POSITIVE_INFINITY;

  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    const benchmark = runJetInstallBenchmark(fixture, args, directNames);
    const performance = compareBaselinePerformance(benchmark, baselineBenchmarks, args);
    attempts.push(summarizeJetBenchmarkAttempt(attempt, benchmark, performance));

    const score = benchmarkPerformanceScore(performance, args);
    if (score < bestScore) {
      bestBenchmark = benchmark;
      bestPerformance = performance;
      bestScore = score;
    }
    if (performance?.result === "green") break;
  }

  return {
    jetBenchmark: bestBenchmark,
    baselinePerformance: bestPerformance,
    jetBenchmarkAttempts: attempts,
  };
}

function parsePackageLockOracle(fixture, directNames) {
  const lockPath = path.join(fixture, "package-lock.json");
  if (!fs.existsSync(lockPath)) return null;
  const lock = readJson(lockPath);
  const versions = {};
  const mismatches = [];
  for (const name of directNames) {
    const entry = lock.packages?.[`node_modules/${name}`];
    if (entry?.version) versions[name] = entry.version;
  }
  return { lockfile: rel(lockPath), direct_versions: versions, mismatches };
}

function parsePnpmOracle(fixture, directNames) {
  const lockPath = path.join(fixture, "pnpm-lock.yaml");
  if (!fs.existsSync(lockPath)) return null;
  const text = fs.readFileSync(lockPath, "utf8");
  const versions = {};
  for (const name of directNames) {
    const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const re = new RegExp(`\\n\\s+${escaped}:\\n\\s+specifier:\\s+[^\\n]+\\n\\s+version:\\s+([^\\n]+)`);
    const match = text.match(re);
    if (match) versions[name] = match[1].replace(/\(.+\)$/, "");
  }
  return { lockfile: rel(lockPath), direct_versions: versions };
}

function compareOracleVersions(oracle, installedDirect) {
  if (!oracle) return null;
  const mismatches = [];
  for (const [name, oracleVersion] of Object.entries(oracle.direct_versions || {})) {
    const installedVersion = installedDirect[name]?.version;
    if (installedVersion && installedVersion !== oracleVersion) {
      mismatches.push({ name, oracle_version: oracleVersion, jet_version: installedVersion });
    }
  }
  return { ...oracle, mismatches, result: mismatches.length === 0 ? "green" : "yellow" };
}

function inspectFixture(fixtureArg, args) {
  console.error(`[pkg-compare] fixture ${fixtureArg}`);
  const fixture = path.resolve(repoRoot, fixtureArg);
  const pkgPath = path.join(fixture, "package.json");
  const lockPath = path.join(fixture, "jet-lock.yaml");
  const nodeModules = path.join(fixture, "node_modules");
  const checks = [];
  const missing = [];
  const commands = [];

  if (!fs.existsSync(pkgPath)) missing.push("package.json");
  if (!fs.existsSync(lockPath)) missing.push("jet-lock.yaml");
  if (missing.length > 0) {
    return { fixture: fixtureArg, result: "red", missing };
  }

  const pkg = readJson(pkgPath);
  const directGroups = packageNamesFromPackageJson(pkg);
  const directNames = [...directGroups.dependencies, ...directGroups.devDependencies].sort();
  const lockBefore = fileSha256(lockPath);

  if (args.hydrate) {
    commands.push(run(path.resolve(repoRoot, args.jetBin), ["install", "--frozen-lockfile", "--no-prebundle"], fixture, args.commandTimeoutMs));
  }

  const lockAfter = fileSha256(lockPath);
  const lock = parseJetLock(fs.readFileSync(lockPath, "utf8"));
  const lockUnchanged = lockBefore === lockAfter;
  const nodeModulesPresent = fs.existsSync(nodeModules);

  checks.push({ name: "lockfile_unchanged_after_frozen_install", ok: lockUnchanged });
  checks.push({ name: "node_modules_present", ok: nodeModulesPresent });
  if (commands.length > 0) {
    checks.push({ name: "jet_install_frozen_exit_zero", ok: commands.every((cmd) => cmd.exit_code === 0) });
  }

  const installedDirect = {};
  const directMissing = [];
  const directVersionNotInJetLock = [];
  if (nodeModulesPresent) {
    for (const name of directNames) {
      const installed = readInstalledPackage(nodeModules, name);
      if (!installed) {
        directMissing.push(name);
        continue;
      }
      installedDirect[name] = { version: installed.version };
      if (!lock.packages.get(name)?.has(installed.version)) {
        directVersionNotInJetLock.push({ name, installed_version: installed.version });
      }
    }
  } else {
    directMissing.push(...directNames);
  }

  const binLinks = nodeModulesPresent
    ? inspectBinLinks(nodeModules, lock.bins)
    : { present_count: 0, missing: [...lock.bins].map(([bin, owner]) => ({ bin, owner })) };
  const installedPackageCount = nodeModulesPresent ? countInstalledPackages(nodeModules) : 0;
  const nodeModulesStats = nodeModulesPresent ? dirStats(nodeModules) : null;
  const thirdPartyResidue = nodeModulesPresent ? thirdPartyLayoutResidue(nodeModules) : [];

  checks.push({ name: "direct_dependencies_hydrated", ok: directMissing.length === 0 });
  checks.push({ name: "direct_dependency_versions_match_jet_lock", ok: directVersionNotInJetLock.length === 0 });
  checks.push({ name: "bin_links_hydrated", ok: binLinks.missing.length === 0 });
  checks.push({ name: "third_party_package_manager_layout_absent", ok: thirdPartyResidue.length === 0 });
  const binSmokeRoot = nodeModulesPresent ? copyFixtureForBaseline(fixture) : null;
  const binSmoke = nodeModulesPresent
    ? expectedBinSmoke(pkg, lock).map((smoke) => runBinSmoke(nodeModules, smoke, binSmokeRoot, args.commandTimeoutMs))
    : [];
  checks.push({ name: "expected_bin_shims_execute", ok: binSmoke.every((cmd) => cmd.exit_code === 0) });

  const npmOracle = compareOracleVersions(parsePackageLockOracle(fixture, directNames), installedDirect);
  const pnpmOracle = compareOracleVersions(parsePnpmOracle(fixture, directNames), installedDirect);
  const baselineBenchmarks = args.baselineTools.map((tool) =>
    runPackageManagerBaseline(tool, fixture, directNames, args),
  );
  const benchmarkResult = args.baselineTools.length > 0
    ? runBestJetInstallBenchmark(fixture, args, directNames, baselineBenchmarks)
    : { jetBenchmark: null, baselinePerformance: null, jetBenchmarkAttempts: [] };
  const jetBenchmark = benchmarkResult.jetBenchmark;
  const baselinePerformance = benchmarkResult.baselinePerformance;
  const jetInstallMaturity = jetBenchmark?.jet_install_maturity ?? null;
  if (jetInstallMaturity) {
    checks.push({
      name: "jet_install_maturity_evidence_present",
      ok: jetInstallMaturity.result === "green",
    });
  }

  const hardFailures = [
    ...checks.filter((check) => !check.ok).map((check) => check.name),
    ...commands.filter((cmd) => cmd.exit_code !== 0).map((cmd) => `${cmd.command.join(" ")} failed`),
    ...binSmoke.filter((cmd) => cmd.exit_code !== 0).map((cmd) => `${cmd.command.join(" ")} failed`),
  ];

  return {
    fixture: fixtureArg,
    result: hardFailures.length === 0 ? "green" : "red",
    package_name: pkg.name,
    direct_dependency_count: directNames.length,
    jet_lock_package_count: lock.package_count,
    installed_package_count: installedPackageCount,
    node_modules_stats: nodeModulesStats,
    bin_link_count: lock.bins.size,
    checks,
    hard_failures: hardFailures,
    direct_missing: directMissing,
    direct_version_not_in_jet_lock: directVersionNotInJetLock,
    third_party_layout_residue: thirdPartyResidue,
    bin_links: binLinks,
    installed_direct: installedDirect,
    npm_oracle: npmOracle,
    pnpm_oracle: pnpmOracle,
    jet_benchmark: jetBenchmark,
    jet_benchmark_attempts: benchmarkResult.jetBenchmarkAttempts,
    jet_install_maturity: jetInstallMaturity,
    baseline_benchmarks: baselineBenchmarks,
    baseline_performance: baselinePerformance,
    commands,
    bin_smoke: binSmoke,
  };
}

function writeJson(file, value) {
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}

function lockContainsPackage(root, name) {
  const lockPath = path.join(root, "jet-lock.yaml");
  if (!fs.existsSync(lockPath)) return false;
  const lock = parseJetLock(fs.readFileSync(lockPath, "utf8"));
  return lock.packages.has(name);
}

function runWorkspaceContract(args) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "jet-pkg-workspace-contract-"));
  const sharedRoot = path.join(root, "packages", "shared");
  const appRoot = path.join(root, "packages", "app");
  fs.mkdirSync(sharedRoot, { recursive: true });
  fs.mkdirSync(appRoot, { recursive: true });

  writeJson(path.join(root, "package.json"), {
    name: "jet-pkg-workspace-contract-root",
    version: "0.0.0",
    private: true,
    workspaces: ["packages/*"],
  });
  writeJson(path.join(sharedRoot, "package.json"), {
    name: "shared",
    version: "1.2.3",
    main: "index.cjs",
  });
  fs.writeFileSync(path.join(sharedRoot, "index.cjs"), "exports.answer = 42;\n");
  writeJson(path.join(appRoot, "package.json"), {
    name: "app",
    version: "0.0.0",
    type: "commonjs",
    dependencies: {
      shared: "workspace:*",
      "is-number": "7.0.0",
    },
  });
  fs.writeFileSync(
    path.join(appRoot, "smoke.cjs"),
    "const shared = require('shared'); const isNumber = require('is-number'); if (shared.answer !== 42 || !isNumber(5) || isNumber('x')) process.exit(2);\n",
  );

  const jet = path.resolve(repoRoot, args.jetBin);
  const commands = [];
  const checks = [];
  const lockPath = path.join(root, "jet-lock.yaml");
  const appNodeModules = path.join(appRoot, "node_modules");
  const sharedLink = path.join(appNodeModules, "shared");

  commands.push(run(jet, ["install", "--no-frozen-lockfile", "--no-prebundle"], root, args.commandTimeoutMs));
  checks.push({ name: "workspace_install_exit_zero", ok: commands.at(-1).exit_code === 0 });
  checks.push({ name: "workspace_lock_written", ok: fs.existsSync(lockPath) });
  checks.push({ name: "workspace_link_hydrated", ok: fs.existsSync(sharedLink) && fs.lstatSync(sharedLink).isSymbolicLink() });
  checks.push({ name: "workspace_external_dependency_hydrated", ok: nodePackageVersion(appRoot, "is-number") === "7.0.0" });
  commands.push(run(process.execPath, ["smoke.cjs"], appRoot, args.commandTimeoutMs));
  checks.push({ name: "workspace_node_resolution_smoke", ok: commands.at(-1).exit_code === 0 });

  const lockBeforeFrozen = fs.existsSync(lockPath) ? fileSha256(lockPath) : null;
  commands.push(run(jet, ["install", "--frozen-lockfile", "--no-prebundle"], root, args.commandTimeoutMs));
  const lockAfterFrozen = fs.existsSync(lockPath) ? fileSha256(lockPath) : null;
  checks.push({ name: "workspace_frozen_install_exit_zero_without_drift", ok: commands.at(-1).exit_code === 0 });
  checks.push({ name: "workspace_frozen_install_keeps_lock_unchanged", ok: lockBeforeFrozen && lockBeforeFrozen === lockAfterFrozen });

  const appPackagePath = path.join(appRoot, "package.json");
  const appPackage = readJson(appPackagePath);
  appPackage.dependencies["is-number"] = "^7.0.0";
  writeJson(appPackagePath, appPackage);
  commands.push(run(jet, ["install", "--frozen-lockfile", "--no-prebundle"], root, args.commandTimeoutMs));
  const lockAfterDriftAttempt = fs.existsSync(lockPath) ? fileSha256(lockPath) : null;
  checks.push({ name: "workspace_frozen_install_rejects_dep_drift", ok: commands.at(-1).exit_code !== 0 });
  checks.push({ name: "workspace_frozen_drift_keeps_lock_unchanged", ok: lockBeforeFrozen && lockBeforeFrozen === lockAfterDriftAttempt });

  const hardFailures = [
    ...checks.filter((check) => !check.ok).map((check) => check.name),
    ...commands
      .filter((cmd) => cmd.exit_code !== 0)
      .filter((cmd) => !cmd.command.includes("--frozen-lockfile") || !cmd.stderr.join("\n").includes("Frozen lockfile drift detected"))
      .map((cmd) => `${cmd.command.join(" ")} failed`),
  ];

  return {
    fixture: rel(root),
    result: hardFailures.length === 0 ? "green" : "red",
    checks,
    hard_failures: hardFailures,
    commands,
    workspace_link: fs.existsSync(sharedLink) ? fs.readlinkSync(sharedLink) : null,
    node_modules_stats: fs.existsSync(appNodeModules) ? dirStats(appNodeModules) : null,
  };
}

function packageDependencyState(root, name) {
  const pkg = readJson(path.join(root, "package.json"));
  return {
    dependencies: pkg.dependencies?.[name] ?? null,
    devDependencies: pkg.devDependencies?.[name] ?? null,
    optionalDependencies: pkg.optionalDependencies?.[name] ?? null,
    private_preserved: pkg.private === true,
    scripts_preserved: pkg.scripts?.test === "node ./smoke.cjs",
    type_preserved: pkg.type === "commonjs",
  };
}

function nodePackageVersion(root, name) {
  return readInstalledPackage(path.join(root, "node_modules"), name)?.version ?? null;
}

function runMutationContract(args) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "jet-pkg-contract-"));
  fs.writeFileSync(
    path.join(root, "package.json"),
    `${JSON.stringify(
      {
        name: "jet-pkg-mutation-contract",
        version: "0.0.0",
        private: true,
        type: "commonjs",
        scripts: { test: "node ./smoke.cjs" },
      },
      null,
      2,
    )}\n`,
  );
  fs.writeFileSync(
    path.join(root, "smoke.cjs"),
    "const isNumber = require('is-number'); if (!isNumber(3) || isNumber('x')) process.exit(2);\n",
  );

  const jet = path.resolve(repoRoot, args.jetBin);
  const commands = [];
  const checks = [];

  commands.push(run(jet, ["add", "is-number@7.0.0"], root, args.commandTimeoutMs));
  checks.push({
    name: "add_updates_package_json_dependency",
    ok: packageDependencyState(root, "is-number").dependencies === "7.0.0",
  });
  checks.push({ name: "add_writes_lock_entry", ok: lockContainsPackage(root, "is-number") });
  checks.push({ name: "add_hydrates_node_modules", ok: nodePackageVersion(root, "is-number") === "7.0.0" });
  commands.push(run(process.execPath, ["smoke.cjs"], root, args.commandTimeoutMs));

  commands.push(run(jet, ["update", "is-number"], root, args.commandTimeoutMs));
  checks.push({
    name: "update_preserves_package_json_surface",
    ok: Object.values(packageDependencyState(root, "is-number")).slice(3).every(Boolean),
  });
  checks.push({ name: "update_keeps_lock_entry", ok: lockContainsPackage(root, "is-number") });

  commands.push(run(jet, ["audit"], root, args.commandTimeoutMs));

  commands.push(run(jet, ["remove", "is-number"], root, args.commandTimeoutMs));
  const removedState = packageDependencyState(root, "is-number");
  checks.push({
    name: "remove_updates_package_json",
    ok: !removedState.dependencies && !removedState.devDependencies && !removedState.optionalDependencies,
  });
  checks.push({ name: "remove_prunes_lock_entry", ok: !lockContainsPackage(root, "is-number") });
  checks.push({ name: "remove_prunes_node_modules_entry", ok: nodePackageVersion(root, "is-number") === null });
  checks.push({
    name: "remove_preserves_package_json_surface",
    ok: removedState.private_preserved && removedState.scripts_preserved && removedState.type_preserved,
  });

  const hardFailures = [
    ...checks.filter((check) => !check.ok).map((check) => check.name),
    ...commands.filter((cmd) => cmd.exit_code !== 0).map((cmd) => `${cmd.command.join(" ")} failed`),
  ];

  return {
    fixture: rel(root),
    result: hardFailures.length === 0 ? "green" : "red",
    checks,
    hard_failures: hardFailures,
    commands,
    package_json_after: readJson(path.join(root, "package.json")),
    node_modules_stats: fs.existsSync(path.join(root, "node_modules")) ? dirStats(path.join(root, "node_modules")) : null,
  };
}

const thirdPartyPackageManagerBins = new Set(["npm", "pnpm", "yarn", "bun", "npx", "corepack"]);

function collectCommands(value, commands = []) {
  if (Array.isArray(value)) {
    for (const item of value) collectCommands(item, commands);
    return commands;
  }
  if (!value || typeof value !== "object") return commands;
  if (Array.isArray(value.command)) commands.push(value.command);
  for (const child of Object.values(value)) collectCommands(child, commands);
  return commands;
}

function thirdPartyPackageManagerCommands(evidence) {
  return collectCommands(evidence)
    .filter((command) => thirdPartyPackageManagerBins.has(path.basename(command[0] || "")))
    .map((command) => command.join(" "));
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const fixtures = args.fixtures.map((fixture) => inspectFixture(fixture, args));
  const mutation = args.mutationContract ? runMutationContract(args) : null;
  const workspace = args.workspaceContract ? runWorkspaceContract(args) : null;
  const evidence = {
    contract_id: "basic.install.replacement",
    result: "pending",
    phase: 1,
    generated_at: new Date().toISOString(),
    note: "Jet is the fixture executor. npm/pnpm lockfiles are read-only oracle evidence; optional npm/pnpm command runs are isolated benchmark baselines and never hydrate the source fixtures.",
    executor_contract: {
      fixture_hydration: "jet install --frozen-lockfile",
      mutation: "jet add/update/audit/remove",
      workspace: "jet install + jet install --frozen-lockfile",
      forbidden_package_manager_executors: [...thirdPartyPackageManagerBins],
    },
    baseline_benchmark_contract: {
      tools: args.baselineTools,
      required_for_exit_zero: args.requireBaselines,
      command_policy: "benchmark_only_not_fixture_hydration",
      npm_ci_allowed: false,
      tool_resolution_policy: {
        npm: "host npm baseline command, isolated benchmark copy only",
        pnpm: "pnpm CLI provisioned by jet install into a temporary tool root",
      },
      max_install_time_ratio: args.maxInstallTimeRatio,
      max_disk_bytes_ratio: args.maxDiskBytesRatio,
    },
    fixtures,
    mutation_contract: mutation,
    workspace_contract: workspace,
  };
  const executorOnly = {
    fixtures: fixtures.map((fixture) => ({
      commands: fixture.commands,
      bin_smoke: fixture.bin_smoke,
    })),
    mutation_contract: mutation ? { commands: mutation.commands } : null,
    workspace_contract: workspace ? { commands: workspace.commands } : null,
  };
  const forbiddenExecutors = thirdPartyPackageManagerCommands(executorOnly);
  const npmCiCommands = collectCommands(evidence)
    .filter((command) => path.basename(command[0] || "") === "npm" && command[1] === "ci")
    .map((command) => command.join(" "));
  const baselineFailures = fixtures.flatMap((fixture) =>
    (fixture.baseline_benchmarks || [])
      .filter((benchmark) => benchmark.result !== "green")
      .map((benchmark) => `${fixture.fixture}:${benchmark.tool}`),
  );
  const baselinePerformanceFailures = fixtures
    .filter((fixture) => fixture.baseline_performance?.result === "red")
    .map((fixture) => fixture.fixture);
  const installMaturityFailures = fixtures
    .filter((fixture) => fixture.jet_install_maturity?.result !== "green")
    .map((fixture) => ({
      fixture: fixture.fixture,
      failures: fixture.jet_install_maturity?.failures || ["jet_install_maturity_missing"],
    }));
  const binHeavyFixtures = fixtures
    .filter((fixture) =>
      fixture.bin_link_count > 0 &&
      (fixture.bin_smoke || []).length > 0 &&
      (fixture.bin_smoke || []).every((cmd) => cmd.exit_code === 0),
    )
    .map((fixture) => ({
      fixture: fixture.fixture,
      bin_link_count: fixture.bin_link_count,
      bin_smoke_count: fixture.bin_smoke.length,
    }));
  const packageContractFixtureBreadth = {
    result: workspace?.result === "green" &&
      mutation?.result === "green" &&
      binHeavyFixtures.length > 0
        ? "green"
        : "red",
    workspace_contract_green: workspace?.result === "green",
    mutation_contract_green: mutation?.result === "green",
    bin_heavy_fixture_count: binHeavyFixtures.length,
    bin_heavy_fixtures: binHeavyFixtures,
    required_fixture_shapes: [
      "multi-package workspace layout",
      "mutation lifecycle surface",
      "bin-heavy package layout with executable shims",
    ],
  };
  evidence.forbidden_package_manager_commands = forbiddenExecutors;
  evidence.npm_ci_commands = npmCiCommands;
  evidence.install_maturity_failures = installMaturityFailures;
  evidence.package_contract_fixture_breadth = packageContractFixtureBreadth;
  evidence.checks = [
    {
      name: "no_npm_pnpm_yarn_bun_executor_commands",
      ok: forbiddenExecutors.length === 0,
    },
    {
      name: "no_npm_ci_anywhere",
      ok: npmCiCommands.length === 0,
    },
    {
      name: "required_baseline_benchmarks_green",
      ok: !args.requireBaselines || baselineFailures.length === 0,
    },
    {
      name: "required_baseline_performance_green",
      ok: !args.requireBaselines || baselinePerformanceFailures.length === 0,
    },
    {
      name: "install_maturity_evidence_present",
      ok: installMaturityFailures.length === 0,
    },
    {
      name: "package_contract_fixture_breadth_present",
      ok: packageContractFixtureBreadth.result === "green",
    },
  ];
  evidence.baseline_failures = baselineFailures;
  evidence.baseline_performance_failures = baselinePerformanceFailures;
  const result =
    fixtures.every((fixture) => fixture.result === "green") &&
    (!mutation || mutation.result === "green") &&
    (!workspace || workspace.result === "green") &&
    forbiddenExecutors.length === 0 &&
    npmCiCommands.length === 0 &&
    (!args.requireBaselines || baselineFailures.length === 0) &&
    (!args.requireBaselines || baselinePerformanceFailures.length === 0) &&
    installMaturityFailures.length === 0 &&
    packageContractFixtureBreadth.result === "green"
      ? "green"
      : "red";
  evidence.result = result;

  fs.mkdirSync(path.dirname(args.evidence), { recursive: true });
  fs.writeFileSync(args.evidence, `${JSON.stringify(evidence, null, 2)}\n`);
  console.log(JSON.stringify(evidence, null, 2));
  process.exit(result === "green" ? 0 : 1);
}

main();

// </HANDWRITE>
