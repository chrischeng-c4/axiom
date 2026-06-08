# ADR-014: CJK IME manual test matrix (zhuyin / pinyin / kana / hangul)

| Field        | Value |
|--------------|-------|
| Issue        | #2173 |
| Parent epic  | #2138 |
| Status       | accepted |
| Date         | 2026-05-16 |
| Decision     | Ship a human-driven 12-phrase × 4-IME recording matrix under `projects/jet/data/parity/manual-tests/ime/<lang>.md`, with per-cell JSON recordings under `projects/jet/conformance/ime-cjk/<family>/<ime>/<phrase-id>.json` re-validated headlessly against the #2172 normalization rules on every PR that touches the IME channel. |

## Context

Jet's IME channel is built on the hidden-input proxy from ADR-010 / #2171
and the composition-event normalization protocol from #2172. Those two
specs describe the *shape* of an IME session — when `compositionstart`
fires, what `compositionupdate` carries, how `beforeinput` interleaves
with `input`, what bytes land in the canvas-side text buffer on
`compositionend`. They do **not** exercise a real OS-level IME. Headless
synthesis cannot: candidate-window selection, romaji-to-kana
auto-conversion, zhuyin tone-mark handling, and Korean batchim
re-decomposition all happen inside the OS IME process, which is
proprietary, version-pinned to the OS, and never quite emits the same
event sequence twice for the same key sequence.

We therefore cannot reach parity with the HTML reference renderer on CJK
input by automation alone. Every Flutter, CKEditor, ProseMirror, and
Monaco team that has tried — and we have read all of their post-mortems
in `flutter-integration-test-study.md` — has reached the same
conclusion: **the long tail of CJK IME parity bugs is only catchable by
human operators driving real IMEs against a captured fixture corpus,
with the captured event streams replayed headlessly in CI thereafter**.
This ADR makes that division of labor explicit.

The matrix has three independent forcing functions:

- **OS IMEs are non-deterministic at the DOM layer.** The same `か`
  composed through Google Japanese Input on the same macOS build will
  emit a different `compositionupdate` count depending on whether the
  candidate window is warm, whether the user's learned-phrase cache
  contains the previous word, and whether autocorrect promoted the
  candidate before the human pressed space. We cannot assert on raw
  event counts; we can only assert that the *normalized* `ImeEvent`
  stream from #2172 matches the canvas-side text buffer.
- **There is no public CJK IME corpus.** The W3C IME workshop minutes
  reference one (Editor's Note, 2018) but it was never published.
  Flutter's `web_ui/dev/test_platform.dart` ships a hand-curated
  12-phrase set that has held up for four years across three Chromium
  major-version cycles. We adopt their corpus shape (3 phrases per
  family × 4 families = 12) and extend it with the acceptance criteria
  this team has shipped for #2172.
- **Human cost is the binding constraint.** A single matrix run is
  ~30 minutes per operator if everything is warm: 12 phrases × ~90s of
  setup-record-export-verify per phrase, plus IME-switching overhead.
  At the four-IME baseline that's ~2 operator-hours per channel. The
  matrix must therefore (a) re-run only on release-candidate cut, not
  per-PR, and (b) bank its output as replayable JSON so per-PR CI
  re-validates the *recordings*, not the human.

Related ADRs and issues:

- **ADR-010 (#2171)** — IME hidden input proxy. The DOM target every
  recording fires against.
- **#2172** — IME composition event protocol. Defines the normalized
  `ImeEvent` stream that recordings encode as ground truth.
- **#2175** — IME browser-quirks corpus. This ADR's matrix runs are the
  *trigger* for quirks-corpus entries; we file divergences against
  #2175, we do not own the catalogue.
- **#2176** — mobile IME parity (Gboard / iOS). Explicitly carved out
  of this ADR's scope; tracked separately.
- **#2147 / ADR-011** — pixel tolerance ladder. Composing-text
  underline rendering is asserted via the IME tier of that ladder, not
  re-litigated here.
- **#2149 / ADR-012** — DPR matrix. Candidate-window caret-pinning
  ("≤1 CSS px under caret") is asserted at every DPR tier the parity
  gate certifies.

## Decision

### 1. Coverage matrix shape

The matrix is **four axes × per-cell acceptance**:

```
{Traditional, Simplified, Japanese, Korean}     -- language family
  × {macOS, Windows, ChromeOS, Android, iOS}    -- OS-IME platform
  × {Chromium, Firefox, WebKit}                 -- browser engine
  × {jet, reference}                            -- renderer under test
```

Not every cell exists: WebKit on Windows is N/A, Firefox on iOS is N/A,
ChromeOS only ships Chromium. The fully-populated matrix is **40 cells
per language family × 4 families = 160 cells**, of which the
release-candidate gate exercises the **24-cell baseline** below.

### 2. Per-family baseline IME (the regression bar)

Per R2, exactly one IME per family is the bar everything else extends:

| Family | Platform | Baseline IME | Why this one |
|---|---|---|---|
| Traditional Chinese | macOS | Built-in 注音 (Zhuyin) | Largest TW desktop share. |
| Traditional Chinese | Windows | Microsoft Zhuyin | Largest TW Windows share. |
| Simplified Chinese | macOS / Win | Google Pinyin | Cross-platform, predictable candidate algo. |
| Japanese | macOS / Win | Google Japanese Input | Cross-platform, romaji-kana rules public. |
| Korean | macOS | macOS Hangul (2-set) | Default on macOS, deterministic batchim. |
| Korean | Windows | Microsoft IME (2-set) | Default on Windows. |

Extensibility (R2): adding Boshiamy, Microsoft Pinyin, Apple Hiragana,
or Microsoft Korean 3-set is a documented additive change — drop a new
`<ime>` directory under `projects/jet/conformance/ime-cjk/<family>/`
and the replay harness picks it up by glob.

### 3. The 12-phrase fixture corpus (R1)

Three phrases per family, covering R1's six required behaviours
(single commit, compound commit, candidate cycling, mid-composition
delete, mid-composition Escape, tone/dakuten/batchim handling):

| Phrase ID | Family | Phrase | Exercises |
|---|---|---|---|
| `tw-01` | Traditional | 你好 | Single-commit, two-char compound. |
| `tw-02` | Traditional | 台北市 | Candidate-window cycling (台 vs 颱 vs 抬). |
| `tw-03` | Traditional | 我ㄉ朋友 | Tone-mark mid-composition delete + Escape cancel. |
| `cn-01` | Simplified | 你好 | Single-commit, two-char compound. |
| `cn-02` | Simplified | 中国人民 | Compound + candidate cycling. |
| `cn-03` | Simplified | 重 (chóng vs zhòng) | Tone disambiguation, mid-composition Escape. |
| `jp-01` | Japanese | こんにちは | Romaji-to-kana, `nn`→`ん` rule. |
| `jp-02` | Japanese | 日本語 | Kanji conversion via space, candidate cycling. |
| `jp-03` | Japanese | がっこう | Dakuten + sokuon, mid-composition delete. |
| `kr-01` | Korean | 안녕 | 2-set hangul, batchim formation. |
| `kr-02` | Korean | 안녕하세요 | Multi-syllable, batchim re-decomposition. |
| `kr-03` | Korean | 한국어 | Mid-composition Escape, jamo split rendering. |

### 4. Per-cell recording (R3, R4)

Each (phrase, IME, browser, jet|reference) cell produces one JSON file
at:

```
projects/jet/conformance/ime-cjk/<family>/<ime>/<phrase-id>.json
```

Schema:

```jsonc
{
  "frontmatter": {
    "os": "darwin 24.2.0",
    "browser": "chromium",
    "browser_version": "131.0.6778.86",
    "ime": "macos-zhuyin",
    "ime_version": "macOS built-in 14.2",
    "operator": "<github-handle>",
    "recorded_at": "2026-05-16T14:22:00Z",
    "renderer": "jet"        // or "reference"
  },
  "key_sequence": [          // (a) what the human pressed
    {"t_ms": 0,   "key": "ㄋ"},
    {"t_ms": 120, "key": "ㄧ"},
    ...
  ],
  "dom_events": [            // (b) raw DOM stream from proxy
    {"t_ms": 8,   "type": "compositionstart", "data": ""},
    {"t_ms": 9,   "type": "compositionupdate", "data": "ㄋ"},
    {"t_ms": 130, "type": "compositionupdate", "data": "ㄋㄧ"},
    {"t_ms": 420, "type": "beforeinput", "inputType": "insertCompositionText", "data": "你"},
    {"t_ms": 421, "type": "compositionend", "data": "你"},
    {"t_ms": 422, "type": "input", "inputType": "insertCompositionText", "data": "你"}
  ],
  "normalized": [            // (c) #2172 ImeEvent stream
    {"kind": "start"},
    {"kind": "update", "text": "ㄋ"},
    {"kind": "update", "text": "ㄋㄧ"},
    {"kind": "commit", "text": "你"}
  ],
  "buffer_final": "你"       // (d) canvas-side buffer after compositionend
}
```

The path-encoded fields (family, ime, phrase-id) MUST match the
frontmatter — the replay harness asserts that.

### 5. Per-cell acceptance (release-candidate gate)

Each cell passes only if **all six** boxes tick. The
`projects/jet/data/parity/manual-tests/ime/<lang>.md` checklist enumerates
these per row (see §7):

1. **Candidate window pinned to caret.** Bounding box of OS IME
   candidate popup is within 1 CSS px of the caret rectangle reported
   by the proxy at `compositionupdate` time. Measured by screenshot
   diff against the reference renderer at the same DPR.
2. **Composing text underlined at caret.** Canvas paints the
   in-progress composition with the dotted underline ADR-010 / #2172
   specifies, anchored at the proxy's caret position. Visually
   verified by the operator against the side-by-side reference.
3. **Commit inserts on confirm.** On `compositionend`, the canvas
   text buffer contains exactly `buffer_final`; no trailing IME
   markers, no stray U+200B.
4. **Escape clears composing buffer.** Pressing Escape mid-composition
   leaves the canvas buffer at its pre-`compositionstart` contents and
   emits a `commit` of empty text in the normalized stream.
5. **Arrow keys do not bleed.** While the candidate window is open,
   ↑/↓/←/→ navigate candidates and do NOT fire `keydown` into the
   canvas. The proxy swallows them; the recording's `dom_events`
   stream contains zero `keydown` between `compositionstart` and
   `compositionend`.
6. **Normalized stream replays clean.** The headless replay harness
   re-feeds `dom_events` through the #2172 normalizer and produces a
   byte-equivalent `normalized` array.

Boxes 1, 2, 3, 4, 5 are human-judged at recording time. Box 6 is the
*CI* gate — it re-runs on every PR.

### 6. Headless replay harness (R5)

The replay harness lives at `projects/jet/conformance/ime-cjk/replay/`
and runs on every PR via the existing parity workflow. For each
recording it:

1. Loads the JSON.
2. Constructs a synthetic `EventTarget` and dispatches `dom_events`
   in order, preserving `t_ms` gaps with fake-timer advances.
3. Pipes events through the #2172 normalizer.
4. Asserts the emitted `ImeEvent[]` matches `normalized` byte-for-byte.
5. Asserts `buffer_final` equals the canvas-side buffer after the
   last `commit`.

Any divergence is a `#2172` regression — the recording is the source
of truth because the human signed off on it at recording time.

### 7. Operator-facing checklist (R6, R8)

One markdown file per family under
`projects/jet/data/parity/manual-tests/ime/`:

- `traditional-chinese.md`
- `simplified-chinese.md`
- `japanese.md`
- `korean.md`

Each file is a rendered checklist. One row per (phrase, IME, browser),
with six pass/fail boxes mapping to §5. Row template:

```markdown
### tw-01 你好 — macOS Zhuyin — Chromium

- Fixture:   https://parity.jet.local/ime?phrase=tw-01&renderer=jet
- Reference: https://parity.jet.local/ime?phrase=tw-01&renderer=reference
- [ ] (1) candidate window pinned to caret (≤1 CSS px)
- [ ] (2) composing text underlined at caret
- [ ] (3) commit text == 你好 on Enter
- [ ] (4) Escape mid-composition clears buffer
- [ ] (5) ↑/↓ navigate candidates, do not fire keydown
- [ ] (6) replay harness passes (filled in by CI)
```

Per-language acceptance specifics (R6) live inline in each file:

- **Traditional Chinese**: tone marks 1-5 are accepted during
  composition, not committed; 注音 input emits exactly one
  `compositionupdate` per phonetic.
- **Simplified Chinese**: pinyin tone marks (1-4 or numeric) MUST NOT
  appear in the committed text; candidate cycling via number keys
  selects but does not auto-commit.
- **Japanese**: romaji `nn` auto-converts to `ん` before space; dakuten
  combines with the preceding vowel within one `compositionupdate`;
  sokuon `っ` is single-codepoint, not `tt`.
- **Korean**: batchim re-decomposes when the next jamo arrives —
  `안` + `ㅏ` → `아나`, not `안ㅏ`; jamo composition emits
  one `compositionupdate` per jamo, not per syllable.

### 8. Cadence + owner rotation (R7)

- **Per release-candidate cut**: full 24-cell baseline matrix run by
  the rotating owner-of-the-week.
- **Per major-version browser bump** (Chrome, Firefox, Safari): re-run
  the affected browser's 8 cells, file divergences against #2175.
- **Per #2172 normalization rule change**: replay harness runs over
  the entire recording corpus on PR; no human re-recording needed
  unless the rule change invalidates an existing recording.
- **Per user-filed CJK IME bug**: re-record the affected (family, IME)
  pair to confirm reproduction before triage.

The release-candidate gate **blocks ship on any failing cell**. There
is no "known flaky" allowlist; a failing cell either gets a fix or
gets explicitly carved out with a follow-up issue link.

### 9. Out of scope (carved-out)

- **Linux ibus / fcitx**: deferred to a separate matrix (#TBD,
  follow-up). Linux desktop CJK share is small enough that the
  human-cost trade-off doesn't land here.
- **Emoji keyboard**: #2177-adjacent; emoji input uses the
  IME-but-not-really protocol that has its own quirks.
- **Handwriting input**: pen + trackpad handwriting input on macOS /
  iPadOS is its own surface, owned by mobile parity (#2176).
- **Performance** (composition latency): out of scope here; the
  determinism budget (#2148) covers steady-state but composition
  latency under load is its own benchmark.

## Consequences

### Positive

- **Real-IME parity finally has a gate.** Before this ADR, CJK IME
  regressions surfaced only via user bug reports — by definition,
  after release. The 24-cell baseline catches them before RC cut.
- **CI re-validates without humans after first record.** The replay
  harness means a #2172 normalization change is verified against
  every banked recording on every PR. Human cost is paid once per
  RC, not per PR.
- **Extensibility is structural.** Adding a fifth IME, a new
  browser, or a new phrase is a glob-discoverable addition; no
  central manifest to update.
- **Quirks corpus (#2175) gets a real feeder.** Every divergence
  found during a matrix run is a candidate quirks-corpus entry —
  the matrix run is the canonical *source* of those entries.

### Negative

- **Two operator-hours per RC.** Real cost, paid by a rotating
  owner. We accept this; CJK parity without it is best-effort.
- **Recordings drift.** OS IMEs update; a recording captured against
  macOS 14.2 Zhuyin may diverge after a macOS 14.3 patch. The
  replay harness will flag this as "recording vs normalizer
  mismatch", which is ambiguous — could be a #2172 regression OR
  a stale recording. Operators triage by re-recording.
- **No coverage of Linux CJK** in this matrix. Documented carve-out;
  separate matrix to follow.

### Neutral

- **Storage cost is negligible.** 12 phrases × 4 IMEs × 3 browsers
  × 2 renderers ≈ 288 JSON files at ~8KB each ≈ 2.3 MB. Git, not
  LFS.
- **Mobile is explicitly elsewhere.** #2176 owns Gboard / iOS soft
  keyboard parity; this ADR's matrix is desktop OS IMEs only.

## Alternatives considered

### A. Fully-automated CJK testing via headless IME simulation

**Rejected.** Every team that has tried — Flutter, Monaco, ProseMirror,
CKEditor, even Google's own gMail web client — has reverted to
human-driven matrices for the long tail. OS IMEs are not a public
contract; their event emissions vary by IME-server warm-state, learned
vocabulary, and OS minor version. Asserting on raw event counts is
inherently flaky. We learn from their post-mortems and skip the
detour.

### B. Recording at the OS-event level, replaying through the browser

**Rejected.** macOS `NSTextInputClient` and Windows TSF protocols are
private; capturing them requires accessibility-API plumbing that
Apple revokes per-release. The DOM-level recording is the highest
abstraction layer we can capture reliably and the lowest layer that
matters for canvas-side parity.

### C. Crowd-sourcing recordings via opt-in user telemetry

**Rejected for the baseline.** Privacy + consent ceremony makes this
slower than a rotating internal operator. We keep it as a possible
extension once the baseline is humming, but not on the critical path.

### D. Skip CJK parity for v1, ship Latin/Cyrillic only

**Rejected.** The parity gate's value proposition is "Jet renders like
the browser for everyone". A CJK carve-out at v1 invalidates the gate
for ~1.5B users. Cost is two operator-hours per RC; we pay it.

## Open questions

- **Should the replay harness fuzz event timing?** Real IMEs emit
  events with variable inter-event gaps; the recording captures one
  realisation. A future enhancement could perturb `t_ms` deltas
  ±50ms and re-assert normalization invariance. Filed as a
  stretch follow-up against #2172.
- **Do we record both `jet` and `reference` renderers, or only one?**
  Current decision: both, so the visual side-by-side acceptance
  (boxes 1, 2) has captured ground truth. If banked-storage cost
  ever becomes a concern, drop `reference` recordings — the live
  reference is reproducible from the fixture URL.
- **How do we represent the "IME version" for OS built-ins?** Right
  now we use the OS version string (e.g. "macOS built-in 14.2").
  This is coarse — Apple ships IME-component updates outside the
  marketing-version cadence. Acceptable for now; revisit if we see
  unexplained drift between two recordings on nominally-identical
  OS versions.

## References

- Issue: https://github.com/cclab/cclab/issues/2173
- Parent epic: https://github.com/cclab/cclab/issues/2138
- ADR-010 (IME hidden input proxy): `adr-010-ime-input-proxy.md`
- #2172 (composition event protocol): tracking branch `td-jet-ime-composition-event-protocol`
- #2175 (browser-quirks corpus): tracking branch `td-jet-ime-browser-quirks-corpus`
- #2176 (mobile IME parity): epic-tracked, see #2138
- Flutter web IME test platform: `flutter/engine` `web_ui/dev/test_platform.dart`
- W3C IME workshop minutes (2018): editor's note re corpus — never published.
- `flutter-integration-test-study.md` in this directory — prior-art survey
  that informed the human-driven decision.
