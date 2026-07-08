#!/usr/bin/env node
/*
 * Storybook oracle harness for Jet stories parity.
 *
 * This script compares two official Storybook dev servers (usually Vite and
 * Webpack) with a Jet stories server. It intentionally treats official
 * Storybook as an oracle only: Jet still owns discovery, transforms,
 * dependency serving, HMR, and static export.
 *
 * Required runtime dependencies are loaded from the current Node environment:
 *   - playwright-core
 *   - pngjs
 *
 * Example:
 *   VITE_STORYBOOK_URL=http://127.0.0.1:6106 \
 *   WEBPACK_STORYBOOK_URL=http://127.0.0.1:6107 \
 *   JET_STORIES_URL=http://127.0.0.1:6131 \
 *   STORY_IDS=breadcrumblist--default,calendar--without-drag-and-drop \
 *   node apps/jet/tests/stories/oracle/compare_storybook_oracle.mjs
 */

import crypto from "node:crypto";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";

const viteUrl = normalizedBaseUrl(process.env.VITE_STORYBOOK_URL || "http://127.0.0.1:6106");
const webpackUrl = normalizedBaseUrl(process.env.WEBPACK_STORYBOOK_URL || "http://127.0.0.1:6107");
const jetUrl = normalizedBaseUrl(process.env.JET_STORIES_URL || "http://127.0.0.1:6131");
const requestedStoryIds = (process.env.STORY_IDS || "breadcrumblist--default")
  .split(",")
  .map((id) => id.trim())
  .filter(Boolean);
let storyIds = requestedStoryIds;
let managerStoryId = process.env.MANAGER_STORY_ID || storyIds[0] || "breadcrumblist--default";
const outDir = process.env.OUT_DIR || "/tmp/jet-storybook-oracle";
const viewport = {
  width: Number(process.env.VIEWPORT_WIDTH || 1366),
  height: Number(process.env.VIEWPORT_HEIGHT || 900),
};
const fixedNow = Number(process.env.FIXED_NOW || 1783289000000);
const managerPixelTolerance = Number(process.env.MANAGER_PIXEL_TOLERANCE || 512);
const iframePixelTolerance = Number(process.env.IFRAME_PIXEL_TOLERANCE || 0);
const textEqualPixelTolerance = Number(process.env.TEXT_EQUAL_PIXEL_TOLERANCE || 512);
const textEqualMeanAbsTolerance = Number(process.env.TEXT_EQUAL_MEAN_ABS_TOLERANCE || 8);
const textEqualRatioTolerance = Number(process.env.TEXT_EQUAL_RATIO_TOLERANCE || 0.25);
const emptyTextRetrySettleMs = Number(process.env.EMPTY_TEXT_RETRY_SETTLE_MS || 6000);
const chromeExecutable =
  process.env.CHROME_BIN || "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

function normalizedBaseUrl(value) {
  return value.replace(/\/+$/, "");
}

function sha256(buf) {
  return crypto.createHash("sha256").update(buf).digest("hex");
}

async function loadDeps() {
  const requireFromCwd = createRequire(`${process.cwd()}/`);
  const { chromium } = requireFromCwd("playwright-core");
  const { PNG } = requireFromCwd("pngjs");
  return { chromium, PNG };
}

async function jsonAt(base, route) {
  const res = await fetch(`${base}${route}`);
  if (!res.ok) throw new Error(`${base}${route} returned ${res.status}`);
  return res.json();
}

function entryIds(indexJson) {
  return Object.values(indexJson.entries || {})
    .filter((entry) => entry && entry.type === "story")
    .map((entry) => entry.id);
}

function compareIndex(aName, aIndex, bName, bIndex) {
  const aIds = entryIds(aIndex);
  const bIds = entryIds(bIndex);
  const bSet = new Set(bIds);
  const aSet = new Set(aIds);
  const missing = aIds.filter((id) => !bSet.has(id));
  const extra = bIds.filter((id) => !aSet.has(id));
  const orderDiff = [];
  for (let i = 0; i < Math.max(aIds.length, bIds.length); i += 1) {
    if (aIds[i] !== bIds[i]) {
      orderDiff.push({ index: i, [aName]: aIds[i] || null, [bName]: bIds[i] || null });
      if (orderDiff.length >= 20) break;
    }
  }
  return {
    [aName]: aIds.length,
    [bName]: bIds.length,
    missing,
    extra,
    orderSame: orderDiff.length === 0 && missing.length === 0 && extra.length === 0,
    orderDiff,
  };
}

function pixelDiff(PNG, aFile, bFile) {
  const a = PNG.sync.read(fs.readFileSync(aFile));
  const b = PNG.sync.read(fs.readFileSync(bFile));
  if (a.width !== b.width || a.height !== b.height) {
    return { comparable: false, changed: null, ratio: null };
  }
  let changed = 0;
  let sumAbs = 0;
  let maxDelta = 0;
  for (let i = 0; i < a.data.length; i += 4) {
    let pixelChanged = false;
    for (let channel = 0; channel < 4; channel += 1) {
      const delta = Math.abs(a.data[i + channel] - b.data[i + channel]);
      sumAbs += delta;
      if (delta > maxDelta) maxDelta = delta;
      if (delta > 0) pixelChanged = true;
    }
    if (pixelChanged) changed += 1;
  }
  const total = a.width * a.height;
  return {
    comparable: true,
    changed,
    total,
    ratio: changed / total,
    meanAbsPerChannel: sumAbs / (total * 4),
    maxDelta,
  };
}

function normalizeVolatileText(text) {
  return String(text || "")
    .replace(/\b\d{13,}\b/g, "<timestamp>")
    .replace(/\b(row|Key)-<timestamp>(-\d+)?/g, "$1-<timestamp>$2");
}

function classifyIframeResidual(row) {
  const changed = row.compare.iframe.viteVsJet.pixelDiff.changed;
  const viteText = row.iframe.vite.dom.text;
  const jetText = row.iframe.jet.dom.text;
  if (changed === 0) {
    return { kind: "exact", gatePass: true };
  }
  if (viteText === jetText && changed <= textEqualPixelTolerance) {
    return {
      kind: "text-equal-pixel-residual",
      gatePass: true,
      reason: `body text matches exactly and changed pixels <= ${textEqualPixelTolerance}`,
    };
  }
  if (
    normalizeVolatileText(viteText) === normalizeVolatileText(jetText) &&
    row.compare.iframe.viteVsJet.pixelDiff.meanAbsPerChannel <= textEqualMeanAbsTolerance &&
    row.compare.iframe.viteVsJet.pixelDiff.ratio <= textEqualRatioTolerance
  ) {
    return {
      kind: "text-equal-perceptual-residual",
      gatePass: true,
      reason:
        `normalized body text matches, mean channel delta <= ${textEqualMeanAbsTolerance}, ` +
        `and changed-pixel ratio <= ${textEqualRatioTolerance}`,
    };
  }
  if (normalizeVolatileText(viteText) === normalizeVolatileText(jetText)) {
    return {
      kind: "volatile-text-normalized",
      gatePass: false,
      reason: "body text matches after timestamp/key normalization, but pixels still differ",
    };
  }
  if (viteText === jetText) {
    return {
      kind: "text-equal-large-pixel-diff",
      gatePass: false,
      reason: `body text matches but changed pixels > ${textEqualPixelTolerance}`,
    };
  }
  return {
    kind: "unclassified-content-diff",
    gatePass: false,
    reason: "body text differs after normalization",
  };
}

function managerUrl(base, storyId) {
  return `${base}/?path=/story/${encodeURIComponent(storyId)}`;
}

function iframeUrl(base, storyId) {
  return `${base}/iframe.html?viewMode=story&id=${encodeURIComponent(storyId)}&globals=`;
}

async function capturePage(browser, url, file, options = {}) {
  const {
    freezeTime = true,
    maskSelectors = [],
    settleMs = Number(process.env.SETTLE_MS || 3000),
  } = options;
  const context = await browser.newContext({
    viewport,
    deviceScaleFactor: 1,
    serviceWorkers: "block",
  });
  const page = await context.newPage();
  const errors = [];
  const logs = [];
  page.on("pageerror", (err) => errors.push(String((err && err.stack) || err).slice(0, 800)));
  page.on("console", (msg) => {
    const line = `${msg.type()}: ${msg.text()}`;
    if (/error|warn|failed/i.test(line)) logs.push(line.slice(0, 800));
  });
  if (freezeTime) {
    await page.addInitScript((value) => {
      const NativeDate = Date;
      class FixedDate extends NativeDate {
        constructor(...args) {
          super(...(args.length ? args : [value]));
        }
        static now() {
          return value;
        }
        static parse(input) {
          return NativeDate.parse(input);
        }
        static UTC(...args) {
          return NativeDate.UTC(...args);
        }
      }
      Object.setPrototypeOf(FixedDate, NativeDate);
      FixedDate.prototype = NativeDate.prototype;
      globalThis.Date = FixedDate;
    }, fixedNow);
  }
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 60000 });
  await page.waitForTimeout(settleMs);
  await page.evaluate(() => document.fonts && document.fonts.ready).catch(() => undefined);
  if (maskSelectors.length > 0) {
    await page.evaluate((selectors) => {
      for (const selector of selectors) {
        for (const element of document.querySelectorAll(selector)) {
          const rect = element.getBoundingClientRect();
          if (rect.width <= 0 || rect.height <= 0) continue;
          const mask = document.createElement("div");
          mask.setAttribute("data-jet-oracle-mask", selector);
          mask.style.cssText = [
            "position:fixed",
            `left:${rect.left}px`,
            `top:${rect.top}px`,
            `width:${rect.width}px`,
            `height:${rect.height}px`,
            "background:#fff",
            "pointer-events:none",
            "z-index:2147483646",
          ].join(";");
          document.body.appendChild(mask);
        }
      }
    }, maskSelectors);
  }
  const buf = await page.screenshot({ path: file, fullPage: false, timeout: 60000 });
  const dom = await page.evaluate(() => ({
    title: document.title,
    bodyClass: document.body.className,
    storybookRoot: Boolean(document.querySelector("#storybook-root")),
    storybookDocs: Boolean(document.querySelector("#storybook-docs")),
    jetRoot: Boolean(document.querySelector("#jet-root")),
    preparingStory: Boolean(document.querySelector(".sb-preparing-story")),
    errorDisplay: Boolean(document.querySelector(".sb-errordisplay")),
    text: document.body.innerText.slice(0, 500),
  }));
  await context.close();
  return {
    url,
    file,
    sha: sha256(buf),
    dom,
    errors,
    logs,
  };
}

function hasText(capture) {
  return Boolean(capture && capture.dom && capture.dom.text.trim());
}

async function retryEmptyTextCaptures(browser, row, storyOut, targets) {
  const captures = Object.values(row.iframe);
  if (!captures.some(hasText) || captures.every(hasText)) return;

  for (const [name, base] of Object.entries(targets)) {
    if (hasText(row.iframe[name])) continue;
    row.iframe[name] = await capturePage(
      browser,
      iframeUrl(base, row.id),
      path.join(storyOut, `iframe-${name}.png`),
      { settleMs: emptyTextRetrySettleMs },
    );
  }
}

async function main() {
  fs.mkdirSync(outDir, { recursive: true });
  const { chromium, PNG } = await loadDeps();
  const browser = await chromium.launch({
    executablePath: chromeExecutable,
    headless: true,
    args: ["--no-sandbox", "--disable-dev-shm-usage"],
  });

  const indexes = {
    vite: await jsonAt(viteUrl, "/index.json"),
    webpack: await jsonAt(webpackUrl, "/index.json"),
    jet: await jsonAt(jetUrl, "/index.json"),
  };
  if (storyIds.length === 1 && (storyIds[0] === "all" || storyIds[0] === "__all__")) {
    storyIds = entryIds(indexes.vite);
    if (!process.env.MANAGER_STORY_ID) managerStoryId = storyIds[0] || "breadcrumblist--default";
  }
  if (process.env.STORY_START_AFTER) {
    const index = storyIds.indexOf(process.env.STORY_START_AFTER);
    if (index >= 0) storyIds = storyIds.slice(index + 1);
  }
  if (process.env.STORY_START_AT) {
    const index = storyIds.indexOf(process.env.STORY_START_AT);
    if (index >= 0) storyIds = storyIds.slice(index);
  }
  if (process.env.STORY_LIMIT) {
    storyIds = storyIds.slice(0, Number(process.env.STORY_LIMIT));
  }

  const report = {
    generatedAt: new Date().toISOString(),
    targets: { vite: viteUrl, webpack: webpackUrl, jet: jetUrl },
    storyIds,
    managerStoryId,
    fixedNow,
    tolerances: {
      managerPixels: managerPixelTolerance,
      iframePixels: iframePixelTolerance,
      textEqualPixels: textEqualPixelTolerance,
      textEqualMeanAbs: textEqualMeanAbsTolerance,
      textEqualRatio: textEqualRatioTolerance,
    },
    viewport,
    index: {
      viteVsWebpack: compareIndex("vite", indexes.vite, "webpack", indexes.webpack),
      viteVsJet: compareIndex("vite", indexes.vite, "jet", indexes.jet),
    },
    managerShell: {},
    stories: [],
  };

  for (const [name, base] of Object.entries({ vite: viteUrl, webpack: webpackUrl, jet: jetUrl })) {
    report.managerShell[name] = await capturePage(
      browser,
      managerUrl(base, managerStoryId),
      path.join(outDir, `manager-${name}.png`),
      { freezeTime: false, maskSelectors: ["#storybook-preview-iframe"] },
    );
  }
  report.managerShell.compare = {
    viteVsWebpack: {
      pngEqual: report.managerShell.vite.sha === report.managerShell.webpack.sha,
      pixelDiff: pixelDiff(PNG, report.managerShell.vite.file, report.managerShell.webpack.file),
    },
    viteVsJet: {
      pngEqual: report.managerShell.vite.sha === report.managerShell.jet.sha,
      pixelDiff: pixelDiff(PNG, report.managerShell.vite.file, report.managerShell.jet.file),
    },
  };

  for (const storyId of storyIds) {
    const storyOut = path.join(outDir, storyId);
    fs.mkdirSync(storyOut, { recursive: true });
    const row = { id: storyId, iframe: {} };
    const iframeTargets = { vite: viteUrl, webpack: webpackUrl, jet: jetUrl };
    for (const [name, base] of Object.entries(iframeTargets)) {
      row.iframe[name] = await capturePage(
        browser,
        iframeUrl(base, storyId),
        path.join(storyOut, `iframe-${name}.png`),
      );
    }
    await retryEmptyTextCaptures(browser, row, storyOut, iframeTargets);

    row.compare = {
      iframe: {
        viteVsWebpack: {
          pngEqual: row.iframe.vite.sha === row.iframe.webpack.sha,
          pixelDiff: pixelDiff(PNG, row.iframe.vite.file, row.iframe.webpack.file),
        },
        viteVsJet: {
          pngEqual: row.iframe.vite.sha === row.iframe.jet.sha,
          pixelDiff: pixelDiff(PNG, row.iframe.vite.file, row.iframe.jet.file),
        },
      },
      jetPreviewContract: {
        storybookRoot: row.iframe.jet.dom.storybookRoot,
        storybookDocs: row.iframe.jet.dom.storybookDocs,
        noJetRoot: !row.iframe.jet.dom.jetRoot,
        bodyClassMatchesVite: row.iframe.jet.dom.bodyClass === row.iframe.vite.dom.bodyClass,
      },
    };
    row.residual = classifyIframeResidual(row);
    report.stories.push(row);
    console.log(
      [
        storyId,
        `iframe=${row.compare.iframe.viteVsJet.pixelDiff.changed}`,
        `contract=${Object.values(row.compare.jetPreviewContract).every(Boolean) ? "ok" : "bad"}`,
        `class=${row.residual.kind}`,
      ].join("\t"),
    );
  }

  await browser.close();
  const summary = {
    storyCount: report.stories.length,
    managerShellExact: report.managerShell.compare.viteVsJet.pngEqual,
    managerShellChanged: report.managerShell.compare.viteVsJet.pixelDiff.changed,
    managerShellPass:
      report.managerShell.compare.viteVsJet.pixelDiff.changed <= managerPixelTolerance,
    iframeExact: report.stories.filter((row) => row.compare.iframe.viteVsJet.pngEqual).length,
    iframePass: report.stories.filter(
      (row) => row.compare.iframe.viteVsJet.pixelDiff.changed <= iframePixelTolerance,
    ).length,
    iframeClassifiedPass: report.stories.filter(
      (row) => row.residual.gatePass && Object.values(row.compare.jetPreviewContract).every(Boolean),
    ).length,
    contractOk: report.stories.filter((row) =>
      Object.values(row.compare.jetPreviewContract).every(Boolean),
    ).length,
    residuals: report.stories
      .filter((row) => row.residual.kind !== "exact")
      .map((row) => ({
        id: row.id,
        kind: row.residual.kind,
        gatePass: row.residual.gatePass,
        changed: row.compare.iframe.viteVsJet.pixelDiff.changed,
        ratio: row.compare.iframe.viteVsJet.pixelDiff.ratio,
        meanAbsPerChannel: row.compare.iframe.viteVsJet.pixelDiff.meanAbsPerChannel,
        maxDelta: row.compare.iframe.viteVsJet.pixelDiff.maxDelta,
        reason: row.residual.reason,
      })),
    diffs: report.stories
      .filter((row) => row.compare.iframe.viteVsJet.pixelDiff.changed > iframePixelTolerance)
      .map((row) => ({
        id: row.id,
        iframeChanged: row.compare.iframe.viteVsJet.pixelDiff.changed,
        viteText: row.iframe.vite.dom.text,
        jetText: row.iframe.jet.dom.text,
      })),
  };
  report.summary = summary;
  fs.writeFileSync(path.join(outDir, "report.json"), JSON.stringify(report, null, 2));
  console.log(JSON.stringify(summary, null, 2));
  console.log(`WROTE ${path.join(outDir, "report.json")}`);
}

main().catch((err) => {
  console.error((err && err.stack) || err);
  process.exit(1);
});
