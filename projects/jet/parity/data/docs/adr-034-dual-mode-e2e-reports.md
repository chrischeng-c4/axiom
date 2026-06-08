# ADR-034: dual-mode e2e reports — human HTML + agent JSONL

| Field | Value |
|-------|-------|
| Issue | #2180 |
| Parent epic | #2133 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | A single `jet-parity-report` CLI emits two report formats from one canonical source — the per-test artifact directory written by the parity gate — selected via `--format human-html | agent-jsonl | both`. The human format is a self-contained static `parity-report/<run-id>/human/index.html` styled with the same theme as the parity-gate dashboard, embedding the ADR-024 (#2150) pixel-diff viewer side-by-side, per-test stack traces, an expandable per-section JSON pane, and a folding console-log panel; all image assets are linked relative paths into the same artifact tree, never base64-inlined. The agent format is a newline-delimited `parity-report/<run-id>/agent/results.jsonl` whose every line is one test record with a stable identity tuple `{fixture_id, channel, browser, os, dpr}`, a boolean `passed`, a machine-typed `diff` payload discriminated by `channel` (`PixelDiffReport` | `FocusTraceDiff` | `AxTreeDiff` | `PointerHitMapDiff` | `ImeTraceDiff`), and a single-paragraph `triage_hint` string pre-computed by a per-channel **pinned-rule generator** (no free-form natural language) — pixel cites tier+region+delta, focus cites first diverging tab index + swapped elements, AX cites first diverging node path + changed attribute, pointer cites click index + resolved-vs-expected widget id, IME cites first diverging composition event + its kind. The JSONL carries a top-of-stream `schema_version` line (semver, monotonic) so LLM consumers can pin a shape; schema bumps require an ADR. CI uploads both directories as build artifacts on every parity-gate run, the PR-comment bot links the human HTML, and the agent-autoreview workflow ingests the JSONL. Neither format duplicates image bytes; both reference the per-fixture artifact tree by relative path. Acceptance: a smoke test runs the gate against a known-failing fixture and asserts both `index.html` and `results.jsonl` exist, that the JSONL has at least one well-formed record validating against the published schema, and that the HTML's `<img>` srcs resolve. |

## Context

The parity gate produces test evidence — pixel diffs, focus-order
traces, AX-tree snapshots, pointer hit-maps, IME composition traces,
console logs, stack traces — and today writes a single Markdown
summary file that two very different audiences are expected to
consume. A human reviewer opens it during PR review and wants to
see the failing screenshot inline, the expected pixel next to the
actual, a stack trace they can copy, and a folding pane to dig into
the raw JSON if they need to. An agent autoreview workflow
(typically a Claude or Codex run wired into the PR queue) wants to
slurp the same evidence as a structured stream so it can summarise
the failure class, propose a remediation, and post a comment.

These two audiences need fundamentally different shapes. Humans
need rendering — images, colour, expand/collapse, hyperlinks — and
they need it in one file they can scp to a laptop without losing
fidelity. Agents need machine-typed payloads they can iterate over,
discriminated unions they can pattern-match on, a schema version
they can pin, and **no rendered HTML at all** because parsing HTML
to extract the failure class is a waste of context window.

Trying to serve both from one shape has produced two failure modes
in the previous parity-gate iteration: the Markdown summary lost
its image renders when piped through the agent ingest path, and the
agent had to fall back to filesystem walks to recover the underlying
PNG paths and JSON sidecars — at which point the summary file was
no longer the source of truth and could drift from the artifact
tree it claimed to describe. The fix is to acknowledge that *both
audiences consume the same evidence* but they need different
**projections** of it, and to make the artifact tree itself — not
any rendered report — the canonical source.

A second pressure point is schema stability. The agent autoreview
workflow is consumed by an LLM that has been prompt-tuned against a
specific JSONL shape. Renaming `diff_kind` to `channel`, or
splitting `PixelDiffReport.region` into `region_label` +
`region_geometry`, silently breaks the prompt. A `schema_version`
field anchored to an ADR commit makes the breakage loud: the
consumer asserts the version it was trained against, refuses to
proceed on a mismatch, and the PR that bumped the version is the
PR that bumps the consumer prompt.

A third pressure point is image weight. A parity-gate run on the
MUI corpus snapshot (ADR-030, #2179) can produce 400+ failing
fixtures with three images each (expected, actual, diff). Inlining
all of those into a single Markdown blob produced 80 MB summary
files that crashed PR comments and triggered GitHub artifact size
caps. The human HTML linking back into the artifact tree as
sibling-relative paths keeps the rendered report under 2 MB and
defers the image bytes to lazy `<img>` loads. The agent JSONL
never carries image bytes at all — it carries paths.

## Decision

### 1. One generator, two formats

A new binary `jet-parity-report` lives under
`crates/jet/jet-parity-report/`. Its input is a single argument:
the absolute path to a parity-gate run directory
(`parity-runs/<run-id>/`). Its output is controlled by
`--format`:

| Flag | Effect |
|------|--------|
| `--format human-html` | Emit `parity-report/<run-id>/human/index.html` and its asset symlinks. |
| `--format agent-jsonl` | Emit `parity-report/<run-id>/agent/results.jsonl` plus a sidecar `schema.json`. |
| `--format both` (default) | Emit both. |

The same Rust struct tree — `ParityRunReport` — is the in-memory
source for both renderers. It is built once by walking the run
directory; it is then projected through one of two terminal
renderers. Adding a third audience (e.g. a Slack-bot one-pager) is
a third renderer on the same `ParityRunReport`, never a third walk
of the artifact tree.

### 2. Artifact tree is canonical

The per-fixture artifact tree under
`parity-runs/<run-id>/<fixture_id>/<channel>/` is **not** rebuilt
by the report generator. The report points back into it by
relative path. Concretely:

```
parity-runs/<run-id>/
  meta.json                 # run-level metadata: commit, timestamp, matrix
  <fixture_id>/
    pixel/
      expected.png
      actual.png
      diff.png
      report.json           # PixelDiffReport, the typed payload
    focus/
      trace.json            # FocusTraceDiff
    ax/
      tree.json             # AxTreeDiff
    pointer/
      hitmap.json           # PointerHitMapDiff
    ime/
      trace.json            # ImeTraceDiff
    console.log
    stack.txt

parity-report/<run-id>/
  human/
    index.html              # links: ../parity-runs/<run-id>/<fixture_id>/...
    assets/
      style.css
      viewer.js             # ADR-024 pixel-diff viewer, vendored
  agent/
    results.jsonl
    schema.json             # the JSON Schema for one record
```

The human HTML reaches into the artifact tree via a relative path
walk (`../../parity-runs/<run-id>/<fixture_id>/pixel/diff.png`).
For CI uploads this is preserved by uploading
`parity-runs/<run-id>/` and `parity-report/<run-id>/` as a single
artifact bundle.

### 3. Human HTML contract

The human renderer emits one HTML document. Its layout, top to
bottom:

1. **Run banner.** Commit SHA (linked), timestamp, matrix summary
   `(N browsers × M OS × K DPRs)`, pass/fail counts, link to the
   gating-manifest commit pinned for this run.
2. **Filter chrome.** Sticky-top toolbar with checkboxes per
   channel and per browser, free-text search over `fixture_id`,
   pass/fail toggle. State persists to URL hash so a reviewer can
   share a filtered view in a PR comment.
3. **Fixture cards.** One `<details>` per failing fixture (passing
   fixtures collapse into a footer counter). Card header: status
   pill, fixture id, channel chips. Card body:
   - **Pixel:** the ADR-024 (#2150) side-by-side diff viewer
     embedded inline, expected / actual / diff with the same
     scrubber.
   - **Focus / AX / pointer / IME:** an expandable JSON pane with
     the raw typed payload, syntax-highlighted, plus a one-line
     summary above (e.g. "tab order diverged at index 3").
   - **Console:** a folding pane of the captured console log,
     errors highlighted.
   - **Stack:** preformatted stack trace, copy-to-clipboard
     button.
4. **Footer.** Schema-version pin for the agent JSONL, link to
   ADR-034 itself, build/upload metadata.

The CSS imports the parity-gate dashboard theme (the same
`theme.css` consumed by ADR-024's viewer) so the report and the
dashboard feel like one product. There is **no** runtime backend —
the HTML is fully static and can be opened with `file://`.

Image rendering is lazy: every `<img>` carries
`loading="lazy" decoding="async"` and the fixture-card `<details>`
elements default closed, so opening a 400-fixture report does not
decode 1200 PNGs up front.

### 4. Agent JSONL contract

The agent renderer emits a stream of newline-delimited JSON. The
**first line** is always a schema banner:

```json
{"kind":"schema","schema_version":"1.0.0","record_kind":"jet.parity.test_result"}
```

Every subsequent line is exactly one test result:

```json
{
  "kind": "result",
  "fixture_id": "mui/button/disabled-state",
  "channel": "pixel",
  "browser": "chromium",
  "os": "linux",
  "dpr": 2.0,
  "passed": false,
  "diff": {
    "type": "PixelDiffReport",
    "tier": "a",
    "region_label": "button-label-text",
    "delta": 0.0712,
    "threshold": 0.05,
    "expected_png": "../parity-runs/2026-05-16T1024/mui_button_disabled-state/pixel/expected.png",
    "actual_png":   "../parity-runs/2026-05-16T1024/mui_button_disabled-state/pixel/actual.png",
    "diff_png":     "../parity-runs/2026-05-16T1024/mui_button_disabled-state/pixel/diff.png"
  },
  "triage_hint": "tier-a text region 'button-label-text' drifted 0.0712 vs threshold 0.05; pixel delta concentrated in label glyph anti-aliasing band.",
  "artifacts": {
    "console": "../parity-runs/.../console.log",
    "stack":   "../parity-runs/.../stack.txt"
  }
}
```

The `diff.type` field is the discriminator for the typed-payload
union:

| `diff.type` | Payload spec |
|-------------|--------------|
| `PixelDiffReport` | `{tier, region_label, delta, threshold, expected_png, actual_png, diff_png}` |
| `FocusTraceDiff` | `{first_divergent_index, expected_sequence[], actual_sequence[]}` |
| `AxTreeDiff` | `{first_divergent_path, attr_changed, expected_value, actual_value}` |
| `PointerHitMapDiff` | `{click_index, expected_widget_id, actual_widget_id}` |
| `ImeTraceDiff` | `{first_divergent_event_index, expected_kind, actual_kind}` |

Each payload is the **machine-typed** form of the failure — no
prose, no human-formatted strings, no concatenated diagnostic
blobs. The single prose surface is `triage_hint`.

### 5. Pinned-rule triage hints

`triage_hint` is generated by per-channel pinned rules — never
free-form. The rules table:

| Channel | Rule template |
|---------|---------------|
| `pixel` | `"tier-{tier} {region_label} region drifted {delta:.4f} vs threshold {threshold:.4f}; {delta_locator_clause}"` |
| `focus` | `"Tab order at index {first_divergent_index} swapped {expected_sequence[i].role}[{expected_sequence[i].name}] ↔ {actual_sequence[i].role}[{actual_sequence[i].name}]"` |
| `ax` | `"AX node {first_divergent_path}: attribute '{attr_changed}' expected {expected_value} got {actual_value}"` |
| `pointer` | `"Click {click_index} resolved to widget '{actual_widget_id}', expected '{expected_widget_id}'"` |
| `ime` | `"IME composition event {first_divergent_event_index} kind '{actual_kind}', expected '{expected_kind}'"` |

`{delta_locator_clause}` is itself a closed enum of phrases keyed
off the pixel-diff sub-region histogram (anti-aliasing band,
solid-fill region, hairline edge, etc.); the catalogue lives in
the sibling doc `projects/jet/data/parity/docs/agent-triage-hints.md`
(referenced as future work — **not** shipped in this slice). When
an unknown rule input lands, the generator emits a placeholder
`"triage_hint": "[unrecognised diff shape; see diff payload]"`
rather than guessing. The rules are stable across `schema_version`
patch bumps; minor bumps may extend them; major bumps may rewrite
them.

### 6. Schema versioning

`schema_version` is semver and lives at the top of the JSONL
stream. The current version is **`1.0.0`**, anchored to this ADR.
Every change to either the record envelope, the `diff.type` union,
or a per-rule template is one of:

- **patch** (`1.0.x`): a `triage_hint` wording fix, a new optional
  field with a backward-compatible default, a new `artifacts.*`
  link.
- **minor** (`1.x.0`): a new `diff.type` variant, a new optional
  required-on-write-but-absent-on-read field, a new pinned-rule
  channel.
- **major** (`x.0.0`): a rename, a removal, a meaning change. Hard
  break.

Every minor and major bump requires its own follow-up ADR that
amends this one with the diff. The consumer prompt for the agent
autoreview workflow asserts the major version and warns on minor
mismatch.

`schema.json` (the JSON Schema document) is emitted alongside the
JSONL on every run and is the machine-checkable form of the
contract. The acceptance smoke test validates every record line in
the JSONL against `schema.json`.

### 7. CI integration

Both directories are uploaded on every parity-gate workflow run as
artifact `parity-report-<run-id>`. The PR-comment bot reads
`agent/results.jsonl` (counts, top-failing-channel summary), posts
the human-readable summary into the PR, and links the
`human/index.html` via the artifact download URL. The agent
autoreview workflow downloads only `agent/results.jsonl` plus
`agent/schema.json`, validates the schema version, iterates over
records, and posts a triage comment. Neither workflow re-walks
the artifact tree.

### 8. Out of scope

- Interactive web app for triage. The ADR-024 (#2150) pixel-diff
  viewer is the entire interactive surface; it is embedded in the
  human HTML and that is the limit of "interactive" in this slice.
- JSON-Schema-driven type-codegen for downstream consumers. The
  schema doc is published; consumers may codegen against it
  themselves but jet does not ship a codegen pipeline as part of
  ADR-034.
- Cross-run trend dashboards (failure-rate-over-time charts,
  flake detection, etc.). Single-run reports only.
- Pixel-perfect screenshot approval flows beyond surfacing the
  diff. Approval lives in the gating manifest, not the report.
- Long-term storage / retention policies for the report
  directories. Artifacts inherit the workflow's default retention
  (90 days). Cleanup is the CI's job.

## Consequences

### Positive

- **One walk, two projections.** The artifact tree is the source
  of truth; the report is a view. Drift between the two becomes
  impossible because the report is regenerated from the tree on
  every run.
- **Image weight is bounded.** Neither format inlines image
  bytes. The human HTML stays under ~2 MB even for 400-fixture
  runs; the agent JSONL stays under ~200 KB for the same.
- **Agent context is dense.** A 200-byte triage hint plus a
  typed payload is roughly an order of magnitude cheaper to feed
  to an LLM than a screenshot-rendered Markdown summary.
- **Schema stability is loud.** Consumers assert
  `schema_version`; mismatches halt the workflow rather than
  silently drift. The version field is the contract.
- **Pinned rules eliminate hallucinated triage.** The
  `triage_hint` is mechanical; an LLM ingesting the JSONL is
  reading a deterministic string, not a model-generated one. The
  agent autoreview's job is to **act on** the hint, not to
  invent it.
- **Theme cohesion.** The human report and the parity-gate
  dashboard share the same CSS; reviewers see one product, not
  two.

### Negative

- **A second binary to maintain.** `jet-parity-report` is its own
  crate with its own dependency surface (the static-site
  generator, the JSON serialiser, the schema-emit logic). It is
  small but it is real maintenance.
- **Pinned-rule rigidity.** Some failures will not fit the rule
  templates; those get the placeholder hint and the agent has to
  fall through to the typed payload. Acceptable for now; the
  fallback clause names the gap.
- **Two-tree CI upload.** Both `parity-runs/` and
  `parity-report/` must travel together; partial uploads (one
  without the other) produce broken `<img>` srcs and dangling
  JSONL paths. The upload step is one atomic action to mitigate.
- **Schema bumps are ADR-gated.** This is positive (loud) and
  negative (slow); a triage-rule wording fix that should be a
  patch bump still costs an ADR amendment.

### Neutral

- **Per-test stack trace surfacing.** The stack file lives in the
  artifact tree; both formats link to it but neither tries to
  parse it. Stack-parse heuristics, if any, are a future-work
  item.
- **Console-log redaction.** The console log is captured raw; no
  PII redaction in this slice. Console logs in parity fixtures
  are synthetic anyway, but if real-traffic console logs ever
  feed in this becomes a concern.

## Alternatives considered

### Alt-A: keep one Markdown summary, render-on-read

Status quo plus a small renderer. Rejected: this is what we have,
and it has produced the drift and image-weight failures described
in the Context.

### Alt-B: render the human HTML server-side, ship a live web app

A backend service that reads the artifact tree and serves a React
app. Rejected: needs hosting, auth, and rotation; the artifact
tree is already a static site if we let it be. ADR-024's
diff-viewer is also already static.

### Alt-C: agent format = Markdown with structured front-matter

YAML front-matter per test, Markdown body for the diff prose.
Rejected: still requires the consumer to parse Markdown, still
mixes prose and structure, and still no schema-version pin.

### Alt-D: agent format = OpenTelemetry traces

Emit OTLP spans per test with attribute bags carrying the diff
payload. Rejected: OTLP is wire-bound (gRPC/HTTP collector); the
consumer is a CLI/workflow that wants a flat file. The
impedance mismatch is real.

### Alt-E: per-channel separate JSONL files

`pixel.jsonl`, `focus.jsonl`, `ax.jsonl`, etc. Rejected: the
consumer almost always wants per-test cross-channel correlation
(a fixture that fails pixel often fails AX too). Merging on read
is a tax. One stream, discriminated by `diff.type`, is cheaper.

### Alt-F: inline base64 images in the human HTML

Single-file portability. Rejected: see Context — 400 fixtures × 3
images × ~200 KB each blows past every artifact and inline limit
we have.

### Alt-G: free-form LLM-generated triage hints

Have an LLM read the diff payload and write the prose. Rejected:
the parity gate runs in CI without an LLM available, the prose
would be non-deterministic across runs, and the agent autoreview
would be reading text generated by another model — a worse signal
than reading the typed payload directly. Pinned rules win.

## Open questions

- **`schema.json` source of truth.** Hand-written under
  `crates/jet/jet-parity-report/schema/` or generated from the
  Rust struct tree with `schemars`? Current lean is generate, so
  the struct definitions stay the source of truth; needs a
  follow-up to validate `schemars` output matches the hand-written
  reference we shipped in this slice.
- **Triage-hint catalogue ownership.** The
  `projects/jet/data/parity/docs/agent-triage-hints.md` sibling doc
  (future-work hook) needs an owner who curates the
  `{delta_locator_clause}` enum and the per-channel rule
  templates. Proposed: same owner as the gating-manifest, since
  the rules are gate-adjacent.
- **`pointer/hitmap.json` payload shape.** Pointer hit-map diffs
  are still gaining shape under #2169 (HTML5 drag-and-drop bridge,
  ADR-031). The payload schema named here is the current
  best-guess; minor bumps likely as ADR-031's e2e fixtures land.
- **Cross-run trend hook.** Out of scope for this slice but
  obvious next step: a `parity-report-trends` tool that ingests
  many JSONL files. The JSONL is designed to be trivially
  concatenable for that use; no schema changes needed.
- **Human-HTML accessibility.** The report itself should pass
  ADR-027 (#2178) axe-core CI. Not gated in this slice but should
  be added once the renderer stabilises.

## References

- Issue #2180 — enhancement(jet) — add dual-mode e2e reports for agent and human review
- Parent epic #2133 — parity-gate ecosystem
- ADR-024 (#2150) — pixel-diff viewer (embedded in the human HTML)
- ADR-027 (#2178) — axe-core CI (future accessibility gate for the human HTML)
- ADR-030 (#2179) — MUI corpus snapshot (stress source for 400-fixture runs)
- ADR-031 (#2169) — HTML5 drag-and-drop bridge (pointer-hitmap payload shape provider)
- Spec `jet-e2e-test-evidence` — `.aw/tech-design/crates/jet/interfaces/test/evidence.md` (artifact tree schema)
- Spec `jet-e2e-agent-report` — `.aw/tech-design/crates/jet/logic/e2e-agent-report.md` (JSONL renderer)
- Spec `jet-e2e-human-review-report` — `.aw/tech-design/crates/jet/logic/e2e-human-review-report.md` (HTML renderer)
- Spec `jet-test-runner-report-mode` — `.aw/tech-design/crates/jet/logic/test-runner.md` (CLI flag wiring)
- Spec `jet-trace-format` — `.aw/tech-design/crates/jet/interfaces/trace/format.md` (existing trace assets referenced from evidence)
- Future doc `projects/jet/data/parity/docs/agent-triage-hints.md` — pinned-rule catalogue (not shipped in this slice)
