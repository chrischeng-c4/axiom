# ADR-036: Browser quirks corpus for IME

| Field | Value |
|-------|-------|
| Issue | #2175 |
| Parent epic | #2138 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Jet maintains a structured, on-disk corpus of known browser IME quirks under `projects/jet/data/parity/quirks/<browser>/<short-id>.md` — one markdown file per quirk, with a stable id (e.g. `JET-IME-Q-001`), a YAML frontmatter block declaring browser-and-version range plus disposition (`waiver` / `emulate` / `ignore`), a minimal DOM-event-sequence reproducer in JSON, the divergence from ADR-013's (#2172) normalized protocol stated as a diff, and per-browser expected output. A generated `projects/jet/data/parity/quirks/CATALOG.md` enumerates the corpus with one-line summaries and a per-browser status column (workaround applied / not applicable / fixed upstream in v N). Each `emulate`-disposition entry adds a normalization rule to #2172's spec and a regression test that replays the reproducer through the protocol and asserts the canonical `ImeEvent` stream; each `waiver`-disposition entry adds a pinning test that asserts the quirk's *current* shape stays unchanged so a silent upstream fix surfaces in CI rather than as a user-visible regression. The corpus is seeded with five entries on first ship — Safari `beforeinput` over-firing with `inputType: "insertCompositionText"` for non-composition typing, Firefox `compositionupdate`-after-`input` ordering on macOS, Chromium vs Safari `compositionend.data` divergence on Escape-cancel, WebKit double-`beforeinput` on Korean hangul commit, iOS Safari kana autocorrect bypassing `input` events — covering the three majors and the iOS mobile path. Discovery is dual-cadence: *passive* (regressions surfaced by ADR-014's (#2173) CJK matrix during release-candidate testing are promoted into the corpus via `aw wi create --type bug --project jet --priority p3` with title `parity/ime/quirk — <one-liner>`) and *active* (a quarterly manual sweep of crbug.com, bugzilla.mozilla.org, and bugs.webkit.org for IME-tagged entries against jet's supported browser floor). A CI workflow runs each quirk's Playwright fixture against the corresponding browser channel on every commit touching `projects/jet/data/parity/` or `crates/cclab-jet/`; quirks whose upstream-fixed-in version is reached cause the pinning test to fail with a "promote to ignore or retire entry" instruction, never an unattended pass. |

## Context

Every IME-related change in jet — composition-event protocol
(ADR-013 / #2172), CJK conformance matrix (ADR-014 / #2173),
selection API parity (ADR-016), inputmode propagation (ADR-032 /
#2176) — is measured against a spec-shaped contract: the
W3C UI Events and Input Events specifications describe an
idealised event ordering for a focused editable element under
IME composition. Real browsers diverge. They diverge in
documented ways (bugs filed against Chromium, Gecko, WebKit
with public tracking ids), in undocumented ways (behaviour that
ships in a release without a corresponding bug entry), and in
edge cases that show up only when a specific input method
(Japanese kana, Korean hangul, Pinyin) intersects with a
specific browser version on a specific operating system.

Without a structured, named catalogue of these divergences,
every CJK matrix failure (ADR-014 / #2173) routes back to
first-principles browser debugging: an engineer reproduces the
failure, manually checks whether the event stream matches the
W3C spec, manually checks each major browser's public bug
tracker for an open ticket, and only then decides whether jet's
normalization layer (the part of #2172 that papers over
browser-side weirdness) needs a new rule or whether the
behaviour is "expected" and the test fixture is wrong. That
debugging path is multiplied by every test failure across every
browser channel across every supported input method. It does
not scale. It also forgets — a quirk debugged in Q1 is
re-debugged from scratch in Q3 when a teammate hits the same
failure shape without remembering the prior investigation.

The fix is a corpus: a named, stable, on-disk record of every
quirk jet knows about, with its disposition committed. The
record answers four questions at lookup time. (1) Has anyone
seen this divergence before? (2) Is it a known browser bug,
and if so, which tracker entry? (3) Is jet's normalization
layer supposed to paper over it, or does the canvas widget
see it raw? (4) When does this entry expire — what browser
version fixes it, and what does CI do when that version
ships?

The corpus also feeds back into the normalization spec. A
quirk with `emulate` disposition is, in effect, a new
normalization rule with a worked example attached: a test that
replays the quirky event sequence through the protocol and
asserts the canonical stream comes out the other side. Drop
the rule and the test fails — the corpus is the regression
suite for the normalization layer's coverage.

Scope of "IME quirks" for this ADR is narrow on purpose.
Quirks about *keyboard event ordering* without composition
(plain ASCII typing) are out — they belong to the keyboard
event spec compliance tracker (#2160 family). Quirks about
*pixel-level rendering jitter* are out — ADR-015 (#2148) owns
determinism. Quirks against legacy IE or pre-Chromium Edge are
out — jet's supported floor excludes them. What is in scope
is the narrow band where IME composition events
(`compositionstart` / `compositionupdate` / `compositionend`)
or composition-adjacent input events (`beforeinput` /
`input` with `isComposing: true` or `inputType:
"insertCompositionText"`) diverge from the W3C-specified
ordering, payload shape, or firing pattern in a way that
ADR-013's protocol cares about.

## Decision

### 1. On-disk layout

```
projects/jet/data/parity/quirks/
├── CATALOG.md                 # generated index
├── chromium/
│   ├── JET-IME-Q-003.md       # compositionend empty-data on Escape
│   └── ...
├── firefox/
│   ├── JET-IME-Q-002.md       # compositionupdate-after-input on macOS
│   └── ...
├── webkit/
│   ├── JET-IME-Q-001.md       # beforeinput over-firing
│   ├── JET-IME-Q-004.md       # double-beforeinput on hangul commit
│   └── ...
├── ios-safari/
│   └── JET-IME-Q-005.md       # kana autocorrect bypasses input event
└── tests/
    ├── q-001.spec.ts          # Playwright test, WebKit channel
    ├── q-002.spec.ts          # Playwright test, Firefox channel
    └── ...
```

Browser directory is the *primary* surface ("which engine
emits this?"). An iOS Safari quirk that also reproduces on
desktop Safari gets the entry filed under `webkit/` with the
mobile-only path called out in the frontmatter; an entry that
reproduces in *both* desktop Safari and iOS Safari only when
the mobile path differs in event payload gets a second
`ios-safari/` entry cross-referencing the first.

### 2. Per-quirk file schema

```markdown
---
id: JET-IME-Q-001
browser: webkit
versions: ">=16.0 <17.5"
upstream_tracker: "https://bugs.webkit.org/show_bug.cgi?id=NNNNNN"
disposition: emulate
related_adr: adr-013
related_issue: 2172
discovered: 2026-04-22
discovered_via: cjk-matrix-run-2026-04-22
---

# Safari `beforeinput.inputType: insertCompositionText` for non-composition typing

## Symptom
One-paragraph description of what the user sees / what the
event stream contains that violates the W3C-specified shape.

## Reproducer
Minimal DOM scenario: which element, which focus state, which
input method active, which keys pressed.

## Event sequence (observed)
```json
[
  {"type": "beforeinput", "inputType": "insertCompositionText", "data": "a", "isComposing": false},
  {"type": "input",       "inputType": "insertText",            "data": "a", "isComposing": false}
]
```

## Event sequence (W3C-specified)
```json
[
  {"type": "beforeinput", "inputType": "insertText", "data": "a", "isComposing": false},
  {"type": "input",       "inputType": "insertText", "data": "a", "isComposing": false}
]
```

## Divergence diff
- `beforeinput.inputType`: spec says `"insertText"`, WebKit says
  `"insertCompositionText"` despite `isComposing: false`.
- All other fields match.

## Disposition: emulate
ADR-013's normalization layer rewrites `inputType:
"insertCompositionText"` to `"insertText"` when `isComposing`
is `false`, before the canvas widget sees the event.

## Jet-side handling
Rule lives in `crates/cclab-jet/src/ime/normalize.rs:rule_q001`
keyed by `JET-IME-Q-001`. Replayed-event test at
`projects/jet/data/parity/quirks/tests/q-001.spec.ts`.

## Upstream status
Tracked at WebKit Bugzilla #NNNNNN, last triaged 2026-04.
No fix announced.
```

The frontmatter is the machine-readable surface; the body is
the human-readable explanation with embedded JSON fixtures.

### 3. Disposition rubric

| Disposition | When to choose | Test added |
|---|---|---|
| `emulate` | Quirk would break #2172's normalized stream if passed through raw. Canvas widget would see an event shape the protocol forbids. | Replay test that pipes the reproducer through normalization and asserts the canonical stream emerges. |
| `waiver` | Quirk is observable to the user (e.g. an extra `compositionupdate` with redundant `data`) but does not break parity; the canvas widget tolerates it. | Pinning test that asserts the quirk's *current* shape stays unchanged; fails loudly when upstream fixes it. |
| `ignore` | Quirk has no user-visible effect after normalization or after the `waiver` layer. Documented for posterity. | None. |

A quirk's disposition is set when the entry lands and may
change exactly twice: `waiver` → `ignore` (when the
normalization layer subsumes the workaround) or `waiver` →
`emulate` (when a new CJK matrix entry surfaces a user-visible
regression that the waiver did not catch). `emulate` is
terminal — once a normalization rule exists, retiring the
entry means deleting both the rule and the test, which
requires a fresh ADR.

### 4. Seed corpus (five entries)

| Id | Browser | Symptom | Disposition |
|---|---|---|---|
| JET-IME-Q-001 | webkit | `beforeinput.inputType: "insertCompositionText"` fires for non-composition typing when an IME is active; `isComposing` is `false`. | emulate |
| JET-IME-Q-002 | firefox | On macOS, `compositionupdate` dispatches *after* the matching `input` event for certain key sequences (spec says before). | waiver |
| JET-IME-Q-003 | chromium | `compositionend` fires with `data: ""` on Escape-cancel; Safari fires with `data: <committed-text>`. | emulate |
| JET-IME-Q-004 | webkit | `beforeinput` fires twice for a single Korean hangul commit (one for syllable composition, one for jamo confirmation). | emulate |
| JET-IME-Q-005 | ios-safari | Kana autocorrect overrides `value` directly without firing `input`; only `MutationObserver`-style polling detects the change. | emulate |

Each seed entry's file lands with the ADR; each emulate
entry's normalization rule lands in the next #2172 revise
cycle; each test lands in the Playwright suite under
`projects/jet/data/parity/quirks/tests/`.

### 5. CATALOG.md format

A one-line summary per quirk plus a per-channel status grid:

```markdown
| Id | Symptom | Chrome stable | Firefox stable | Safari TP | iOS Safari | Disposition |
|---|---|---|---|---|---|---|
| Q-001 | beforeinput.inputType over-fires | n/a | n/a | active | active | emulate |
| Q-002 | compositionupdate-after-input | n/a | active | n/a | n/a | waiver |
| Q-003 | compositionend empty-data on Escape | active | n/a | n/a | n/a | emulate |
| Q-004 | double-beforeinput on hangul commit | n/a | n/a | active | active | emulate |
| Q-005 | kana autocorrect bypasses input | n/a | n/a | n/a | active | emulate |
```

A pre-commit hook regenerates the catalog from the frontmatter
blocks; the hook fails if the file is hand-edited out of sync
with the per-quirk frontmatter.

### 6. CI

Every commit that touches `projects/jet/data/parity/quirks/**`,
`projects/jet/data/parity/docs/adr-013-*`, `projects/jet/data/parity/docs/adr-014-*`,
or `crates/cclab-jet/src/ime/**` triggers the quirk replay
suite against three browser channels (Chrome stable, Firefox
stable, WebKit/Safari Technology Preview). The iOS Safari
path runs as a manual gate during release-candidate testing
(Playwright's WebKit driver does not faithfully reproduce iOS
soft-keyboard autocorrect, so the ADR-014 manual matrix owns
that path).

Each quirk's test asserts one of two contracts: for
`emulate` entries, that the canonical `ImeEvent` stream
emerges from the normalization layer; for `waiver` entries,
that the *quirky* stream is still observed unchanged. A
`waiver` test that starts passing the *canonical* assertion
fails loudly with the message "quirk JET-IME-Q-NNN appears
fixed upstream — re-evaluate disposition (promote to
`ignore` or document the fix and retire the entry)."

### 7. Discovery cadence

Two channels feed the corpus.

**Passive.** ADR-014's (#2173) CJK manual recordings include
an "anomalies" section per session. Every entry in that
section maps, at session close, to one of: an existing quirk
id (logged with a recording-id reference), a new quirk file
(created via `aw wi create --type bug --project jet
--priority p3` titled `parity/ime/quirk — <one-liner>`, with
the recording embedded as the reproducer), or a non-quirk
classification (logged as "out of corpus scope — keyboard
event ordering" or similar).

**Active.** Once per quarter a manual sweep of crbug.com
(component `Blink>Input`), bugzilla.mozilla.org (component
`Core::DOM: Events`), and bugs.webkit.org (component `WebKit
Misc.::Text Input`) for entries filed in the prior three
months that mention `IME`, `composition`, `beforeinput`, or
known IME locale names. Each hit is triaged: relevant ones
get a new quirk file with `discovered_via:
quarterly-sweep-<YYYY>-<QQ>` in the frontmatter.

## Consequences

**Positive.**
The corpus turns "we vaguely remember Safari does something
weird here" into a lookup that takes seconds and answers
with a stable id, a tracker link, and a known disposition.
Every IME regression filed against jet can be triaged in
two steps: search the corpus for matching symptom keywords;
if absent, run the `aw wi` promotion workflow. The
normalization layer's coverage gets a regression suite for
free — every `emulate` entry's test fails the moment the
corresponding rule is broken or deleted. Upstream fixes
surface in CI rather than silently weakening jet's
workaround stack.

**Negative.**
The corpus is staffing: every quirk added is a markdown file
plus a Playwright test plus (for `emulate`) a normalization
rule plus a pinning commitment. The quarterly active sweep
is a recurring time-cost. WebKit's Bugzilla in particular
is noisy and the triage step is non-trivial.

**Neutral.**
The corpus does not replace ADR-014's CJK matrix — they are
adjacent. The matrix is *prospective* (we will run these
sessions to find regressions); the corpus is *retrospective*
(here are the known divergences). A finding from one
becomes an entry in the other.

## Alternatives considered

1. **Inline normalization rules in code with comments.** The
   default position. Rejected: comments are not searchable
   across the codebase as a unit, do not carry frontmatter
   metadata, and do not generate a catalog. The corpus is
   what comments wanted to be.

2. **External tracker mirror (sync from crbug / bugzilla
   into a database).** Rejected as out of scope and as a
   future epic. The active-sweep cadence is the manual
   pre-cursor; if it grows beyond quarterly we revisit.

3. **One giant `BROWSER_QUIRKS.md` file.** Rejected on
   size grounds (the corpus is expected to grow past 30+
   entries within a year) and on schema grounds (frontmatter
   per entry is needed for the catalog generator).

4. **Skip the corpus, document quirks only in #2172's
   normalization spec.** Rejected: #2172's spec is the
   normalized *contract*, not the catalogue of inputs that
   normalize *into* the contract. Mixing them obscures both.

## Open questions

- **How aggressively to expire entries.** A `waiver` entry
  whose pinning test starts failing on a stable channel is
  unambiguous (promote or retire). But a `waiver` entry
  whose test passes on Tech Preview but fails on stable is
  ambiguous: do we wait for the stable rollout? Current
  policy is wait — retire the entry only after the fix
  reaches stable and the channel grace window (six weeks)
  passes. Revisit if this proves brittle.

- **Mobile coverage beyond iOS Safari.** Android Chrome
  IME quirks (Gboard, SwiftKey, Samsung keyboard) are
  out of scope for the seed. A separate ADR will scope
  the mobile-Chromium path once ADR-014's mobile rows are
  in place.

- **Triage automation.** The promotion-workflow command
  (`aw wi create --type bug --project jet --priority p3
  --title 'parity/ime/quirk — …'`) is manual today. A
  future skill could parse an ADR-014 recording's anomalies
  section and emit the create commands directly.

## References

- Issue #2175 (this ADR)
- Parent epic #2138 (jet parity / IME channel)
- ADR-013 / #2172 — composition event protocol
- ADR-014 / #2173 — CJK conformance manual matrix
- ADR-016 — IME selection API parity
- ADR-032 / #2176 — inputmode + enterkeyhint propagation
- W3C UI Events: https://www.w3.org/TR/uievents/
- W3C Input Events Level 2: https://www.w3.org/TR/input-events-2/
- WebKit Bugzilla, component "Text Input"
- Mozilla Bugzilla, component "Core :: DOM: Events"
- Chromium issue tracker, component "Blink>Input"
