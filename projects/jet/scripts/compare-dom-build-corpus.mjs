#!/usr/bin/env node
// <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-scripts-compare-dom-build-corpus-mjs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";

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

const outputRoot = path.resolve(repoRoot, args.get("--out-dir") ?? "/tmp/jet-basic-dom-gate");
const evidencePath = path.resolve(
  repoRoot,
  args.get("--evidence") ?? path.join(outputRoot, "basic-build-corpus.json"),
);
const comparator = path.join(repoRoot, "projects/jet/scripts/compare-basic-builds.mjs");
const runtimeSmoke = args.get("--runtime-smoke") ?? "required";
const buildSamples = args.get("--build-samples") ?? "3";
const jetBin = args.get("--jet-bin") ?? null;
const commandTimeoutMs = args.get("--command-timeout-ms") ?? null;
const runtimeTimeoutMs = args.get("--runtime-timeout-ms") ?? null;
const defaultToolRoot = "projects/jet/tests/fixtures/dom-production-build/react-bench";

const cases = [
  {
    name: "react-bench",
    fixture: "projects/jet/tests/fixtures/dom-production-build/react-bench",
    dependencyRoot: "projects/jet/tests/fixtures/dom-production-build/react-bench",
    toolRoot: defaultToolRoot,
    tools: ["jet", "vite", "webpack"],
    runtimeCase: "react-bench",
    semanticStrings: ["React Bench", "Counter", "Todos", "Add todo"],
    requireCss: false,
    requirePublic: [],
  },
  {
    name: "dom-production-assets",
    fixture: "projects/jet/tests/fixtures/dom-production-build/dom-production-assets",
    dependencyRoot: "projects/jet/tests/fixtures/dom-production-build/react-bench",
    toolRoot: defaultToolRoot,
    tools: ["jet", "vite", "webpack"],
    runtimeCase: "dom-production-assets",
    semanticStrings: [
      "DOM Production Assets",
      "Mode: ",
      "Build target: ",
      "production",
      "Styled status: active",
      "Increment asset counter",
    ],
    requireCss: true,
    requirePublic: ["brand.svg"],
  },
  {
    name: "mui-visual-demo",
    fixture: "projects/jet/tests/fixtures/dom-production-build/mui-visual",
    dependencyRoot: "projects/jet/tests/fixtures/dom-production-build/mui-visual",
    toolRoot: defaultToolRoot,
    tools: ["jet", "vite"],
    runtimeCase: "visual-library",
    semanticStrings: [
      "MUI visual table fixture",
      "MUI component matrix",
      "MUI Primary",
      "MUI success alert",
      "cell 0",
    ],
    requireCss: false,
    requirePublic: [],
  },
  {
    name: "antd-visual-demo",
    fixture: "projects/jet/tests/fixtures/dom-production-build/antd-visual",
    dependencyRoot: "projects/jet/tests/fixtures/dom-production-build/antd-visual",
    toolRoot: defaultToolRoot,
    tools: ["jet", "vite"],
    runtimeCase: "visual-library",
    semanticStrings: [
      "AntD visual table fixture",
      "AntD component matrix",
      "AntD Primary",
      "AntD success alert",
      "cell 0",
    ],
    requireCss: false,
    requirePublic: [],
  },
  {
    name: "tailwind-visual-demo",
    fixture: "projects/jet/tests/fixtures/dom-production-build/tailwind-visual",
    dependencyRoot: "projects/jet/tests/fixtures/dom-production-build/tailwind-visual",
    toolRoot: defaultToolRoot,
    tools: ["jet", "vite"],
    runtimeCase: "visual-library",
    semanticStrings: [
      "Tailwind CSS visual table fixture",
      "Tailwind component matrix",
      "Tailwind Primary",
      "Tailwind count",
      "cell 0",
    ],
    requireCss: true,
    requirePublic: [],
  },
  {
    name: "styled-components-visual-demo",
    fixture: "projects/jet/tests/fixtures/dom-production-build/styled-components-visual",
    dependencyRoot: "projects/jet/tests/fixtures/dom-production-build/styled-components-visual",
    toolRoot: defaultToolRoot,
    tools: ["jet", "vite"],
    runtimeCase: "visual-library",
    semanticStrings: [
      "styled-components visual table fixture",
      "styled-components component matrix",
      "Styled Primary",
      "Styled count",
      "cell 0",
    ],
    requireCss: false,
    requirePublic: [],
  },
];

function progress(message) {
  if (!args.has("--quiet")) {
    console.error(`[dom-build-corpus] ${message}`);
  }
}

async function runCommand(command, cwd) {
  const stdout = [];
  const stderr = [];
  const started = process.hrtime.bigint();
  const child = spawn(command[0], command.slice(1), {
    cwd,
    env: { ...process.env, CI: "1" },
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", (chunk) => stdout.push(chunk));
  child.stderr.on("data", (chunk) => stderr.push(chunk));
  const exitCode = await new Promise((resolve, reject) => {
    child.on("error", reject);
    child.on("close", resolve);
  });
  return {
    command,
    exit_code: exitCode,
    duration_ms: Number(process.hrtime.bigint() - started) / 1_000_000,
    stdout: Buffer.concat(stdout).toString("utf8"),
    stderr: Buffer.concat(stderr).toString("utf8"),
  };
}

function shortOutput(text) {
  const lines = text.trim().split(/\r?\n/).filter(Boolean);
  if (lines.length <= 40) return lines;
  return [...lines.slice(0, 20), "...", ...lines.slice(-20)];
}

await mkdir(outputRoot, { recursive: true });

const caseResults = [];
for (const testCase of cases) {
  const caseOut = path.join(outputRoot, testCase.name);
  const caseEvidence = path.join(caseOut, "basic-build-compare.json");
  const command = [
    process.execPath,
    comparator,
    "--fixture",
    path.join(repoRoot, testCase.fixture),
    "--dependency-root",
    path.join(repoRoot, testCase.dependencyRoot),
    "--tool-root",
    path.join(repoRoot, testCase.toolRoot),
    "--fixture-name",
    testCase.name,
    "--tools",
    testCase.tools.join(","),
    "--runtime-case",
    testCase.runtimeCase,
    "--semantic-strings",
    testCase.semanticStrings.join(","),
    "--runtime-smoke",
    runtimeSmoke,
    "--build-samples",
    buildSamples,
    "--out-dir",
    caseOut,
    "--evidence",
    caseEvidence,
  ];
  if (testCase.requireCss) {
    command.push("--require-css");
  }
  if (testCase.requirePublic.length > 0) {
    command.push("--require-public", testCase.requirePublic.join(","));
  }
  if (jetBin) {
    command.push("--jet-bin", jetBin);
  }
  if (commandTimeoutMs) {
    command.push("--command-timeout-ms", commandTimeoutMs);
  }
  if (runtimeTimeoutMs) {
    command.push("--runtime-timeout-ms", runtimeTimeoutMs);
  }

  progress(`case=${testCase.name}`);
  const run = await runCommand(command, repoRoot);
  let evidence = null;
  try {
    evidence = JSON.parse(await readFile(caseEvidence, "utf8"));
  } catch (err) {
    evidence = {
      contract_id: "basic.build.production",
      fixture_name: testCase.name,
      result: "red",
      error: `failed to read case evidence: ${err.message}`,
    };
  }

  caseResults.push({
    name: testCase.name,
    fixture: testCase.fixture,
    dependency_root: testCase.dependencyRoot,
    tool_root: testCase.toolRoot,
    evidence: path.relative(repoRoot, caseEvidence),
    result: evidence.result,
    comparison: evidence.comparison ?? null,
    run: {
      exit_code: run.exit_code,
      duration_ms: run.duration_ms,
      stdout: shortOutput(run.stdout),
      stderr: shortOutput(run.stderr),
    },
  });

  if (run.exit_code !== 0) {
    progress(`case=${testCase.name} result=${evidence.result} exit=${run.exit_code}`);
  }
}

const result = caseResults.every((item) => item.result === "green") ? "green" : "red";
const evidence = {
  contract_id: "basic.build.production.corpus",
  result,
  phase: 3,
  prerequisite_contracts: [
    "basic.install.replacement",
    "basic.browser-bridge.replacement",
  ],
  generated_at: new Date().toISOString(),
  cases: caseResults,
  thresholds: {
    duration_ratio_max: 1.25,
    gzip_ratio_max: 1.05,
  },
};

await mkdir(path.dirname(evidencePath), { recursive: true });
await writeFile(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`);
console.log(JSON.stringify(evidence, null, 2));

if (result !== "green") {
  process.exit(1);
}

// </HANDWRITE>
