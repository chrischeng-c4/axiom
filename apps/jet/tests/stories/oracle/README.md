# Storybook Oracle Harness

This harness compares Jet stories against official Storybook dev servers.
Official Storybook is used only as an oracle; Jet still owns discovery,
module transforms, dependency serving, HMR, and static export.

Run three servers first:

```bash
pnpm exec storybook dev -p 6106 --ci --no-open
pnpm exec storybook dev -p 6107 -c /path/to/webpack/.storybook --ci --no-open
target/debug/jet stories --host 127.0.0.1 --port 6131
```

Then run:

```bash
STORY_IDS="breadcrumblist--default,calendar--without-drag-and-drop" \
MANAGER_STORY_ID="breadcrumblist--default" \
OUT_DIR="/tmp/jet-storybook-oracle" \
node apps/jet/tests/stories/oracle/compare_storybook_oracle.mjs
```

Use `STORY_IDS=all` to compare every story id from the official Vite
`index.json`:

```bash
STORY_IDS=all \
MANAGER_STORY_ID="breadcrumblist--default" \
OUT_DIR="/tmp/jet-storybook-oracle-all" \
node apps/jet/tests/stories/oracle/compare_storybook_oracle.mjs
```

For long all-story runs, resume or bound a run with:

```bash
STORY_IDS=all \
STORY_START_AFTER="parsedrafttolexicaleditor--lexicaljs-data" \
STORY_LIMIT=50 \
OUT_DIR="/tmp/jet-storybook-oracle-resume" \
node apps/jet/tests/stories/oracle/compare_storybook_oracle.mjs
```

The report checks:

- `index.json` order and story ids for Vite vs Webpack vs Jet.
- One canonical manager shell screenshot, with the preview iframe masked so the
  shell gate does not double-count story content diffs.
- Per-story iframe screenshots.
- Preview DOM contract: `#storybook-root`, `#storybook-docs`, no `#jet-root`,
  and matching `sb-main-* sb-show-main` body class.

Default tolerances are intentionally strict:

- `IFRAME_PIXEL_TOLERANCE=0`
- `MANAGER_PIXEL_TOLERANCE=512`
- `TEXT_EQUAL_PIXEL_TOLERANCE=512`
- `TEXT_EQUAL_MEAN_ABS_TOLERANCE=8`
- `TEXT_EQUAL_RATIO_TOLERANCE=0.25`
- `EMPTY_TEXT_RETRY_SETTLE_MS=6000`

The manager tolerance covers tiny official-shell chrome residuals such as
toolbar/status-icon anti-alias and timing deltas while keeping the raw
changed-pixel count in the JSON report. It is intentionally far below a real
manager-contract mismatch; the old Jet-owned shell measured tens of thousands of
changed pixels. Iframe content remains strict by default because it exercises
Jet's build-core behavior.

Each screenshot capture uses a fresh Playwright browser context. That keeps
Chromium's HTTP cache, ESM module graph cache, service workers, and failed
module loads from leaking between stories during long `STORY_IDS=all` runs.

If one iframe target captures an empty body while another target already has
story text, the harness retries only the empty capture once with
`EMPTY_TEXT_RETRY_SETTLE_MS`. This filters official Storybook loading-spinner
races during long runs without hiding a real Jet failure: a target that remains
empty after the retry is still compared and classified normally.

The summary reports both strict and classified results:

- `iframeExact`: byte-identical iframe screenshots.
- `iframePass`: iframe screenshots within `IFRAME_PIXEL_TOLERANCE`.
- `iframeClassifiedPass`: exact iframe screenshots plus residuals whose body
  text is identical and whose changed pixels are within
  `TEXT_EQUAL_PIXEL_TOLERANCE`.
- `residuals`: every non-exact iframe comparison with its raw changed-pixel
  count, ratio, mean absolute channel delta, max channel delta, and
  classification.

Do not treat `iframeClassifiedPass` as permission to ignore real content
differences. It is for separating known animation/subpixel/perceptual residuals
from resolver, transform, or runtime failures while preserving the raw exact
pixel evidence in the report.
