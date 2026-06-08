# ADR-029: Screen-reader smoke matrix — NVDA / JAWS / VoiceOver / TalkBack, per release-candidate, human-checklist gate

| Field | Value |
|-------|-------|
| Issue | #2162 |
| Parent epic | #2136 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | Run a manual, human-checklist screen-reader (SR) smoke matrix once per release-candidate (RC) across six SR × OS × browser combinations: NVDA × Windows 11 × Chrome, NVDA × Windows 11 × Firefox, JAWS × Windows 11 × Chrome (org-floating commercial license), VoiceOver × macOS Sonoma × Safari, VoiceOver × iOS 17+ × Safari (touch SR pattern), TalkBack × Android 14 × Chrome. Per cell, a single channel maintainer (rotating per release) executes a versioned walk-through script that lives at `projects/jet/data/parity/manual-tests/sr/<sr>-<os>.md`. Each script covers a fixed 10-fixture smoke corpus — Button, TextField, Checkbox, Radio, Dialog, Menu, Tabs, DataGrid, live-region, Disclosure — with ~12 numbered actions per fixture and the expected SR announcement string (or speech-buffer prefix) per action. Reviewer marks Pass / Fail per row, optionally captures local audio of the SR announcement for confused outcomes (audio is *not* committed; privacy + binary bloat). Outputs are checked-in human-readable markdown checklists under `projects/jet/data/parity/manual-tests/sr/runs/<rc>/<sr>-<os>.md`; a `scorecard.md` aggregates pass-rate per cell per RC. Release gating: zero fails on the two flagship cells — NVDA × Chrome and VoiceOver × Safari — blocks the RC; the other four cells are advisory and tracked across RCs for regression trend. Coverage budget: 10 fixtures × ~12 actions × 6 cells ≈ 720 manual checks per RC. Linux SR (Orca) and braille-display output are out of scope at v1. |

## Context

The jet a11y channel layers four lines of defense:

1. **ADR-027 / #2159 axe-core in CI** — static rule engine
   over the `<jet-semantics>` shadow subtree. Catches missing
   labels, broken `aria-*` references, role/attribute mismatches,
   contrast failures. Runs sub-second per fixture; deterministic
   pass/fail.
2. **ADR-008 / #2160 AX-tree shape diff** — CDP-driven structural
   equivalence between jet's computed accessibility tree and
   react-dom's oracle. Catches wrong tree shape, wrong accessible
   name, missing parent/child relationships, wrong `role` mapping.
3. **ADR-018 / #2163 accname WPT subset** — name computation
   conformance against the canonical W3C accessible-name test
   suite. Catches divergence from the accessible-name algorithm
   independent of fixture choice.
4. **ADR-021 / #2161 live-region announcement observer** —
   mutation-observer instrumentation that asserts `aria-live`
   timing + content under real interactions.

All four are automated, deterministic, machine-checkable, and
sit on the CI critical path. They are *necessary* but not
*sufficient* to claim screen-reader parity.

Screen readers are an OS-level interpretation layer on top of
the accessibility tree, with their own quirks, version drift,
and proprietary speech-buffering behavior. They sometimes
ignore valid ARIA, announce in surprising orders, suppress
mutations that arrive within a buffer-flush window, route
content to braille displays differently from speech, and have
OS-version-specific bugs that no headless tool models. A
computed AX tree that passes ADR-008 byte-for-byte against
react-dom can still announce *differently* on NVDA × Chrome,
because NVDA buffers virtual-cursor mutations on a heuristic
timer that react-dom happens to satisfy and jet happens to
miss by a few milliseconds.

The literature on this is well-known to the a11y community.
Empirical reports — Deque's "WebAIM Screen Reader User Survey"
series, the W3C ARIA-AT project (https://aria-at.w3.org/),
Adrian Roselli's running fixture catalogue — all converge on
the same conclusion: **the only reliable acceptance signal for
SR-perceived correctness is a human listening to the SR.**
Automated AX-tree assertions catch ~85% of the bugs at ~5% of
the cost; the remaining ~15% is what eats users alive and
what the smoke matrix is here to catch.

This ADR defines the *human-verified* line of defense that
sits on top of the automated stack. It is deliberately a
*smoke* matrix — broad-but-shallow — not a full conformance
suite. Full per-fixture per-SR walkthroughs at MUI's component
cardinality would consume the entire channel-maintainer
rotation; instead we cover a curated 10-fixture corpus that
exercises the patterns most likely to surface SR-specific
divergence: rapid-state mutations (Checkbox), focus traps
(Dialog), composite widgets with non-trivial keyboard models
(Menu, DataGrid), and live-region timing (live-region fixture).

## Decision

### Matrix shape

Six SR × OS × browser cells, fixed for v1:

| # | SR | OS | Browser | Tier |
|---|----|----|---------|------|
| 1 | NVDA 2024.x | Windows 11 23H2 | Chrome stable | **blocking** |
| 2 | NVDA 2024.x | Windows 11 23H2 | Firefox stable | advisory |
| 3 | JAWS 2024 | Windows 11 23H2 | Chrome stable | advisory |
| 4 | VoiceOver | macOS Sonoma 14.x | Safari 17.x | **blocking** |
| 5 | VoiceOver | iOS 17+ | Mobile Safari | advisory |
| 6 | TalkBack | Android 14 | Chrome stable | advisory |

Rationale for the two **blocking** cells:

- **NVDA × Chrome** — NVDA is free, the most-used SR worldwide
  per WebAIM survey, and Chrome is the most-used browser in
  the world. This is the modal user experience.
- **VoiceOver × Safari** — VoiceOver is the only SR shipped
  by Apple, used by ~100% of macOS / iOS SR users, and Safari
  is the only browser on iOS. This is the entire Apple SR
  surface.

Other cells are **advisory** — failures are recorded in the
RC scorecard and tracked as regression trend, but do not
block the release tag. Rationale:

- **NVDA × Firefox** — Firefox AT-SPI plumbing on Windows
  is historically the most divergent from Chrome; advisory
  catches Firefox-specific regressions without blocking on
  what is a sub-15% browser share among NVDA users.
- **JAWS × Chrome** — JAWS is commercial, organizationally
  licensed (single floating seat), maintained for enterprise
  customers who require it. Blocking on JAWS would gate
  releases on license-pool availability.
- **VoiceOver iOS × Safari** — touch SR pattern is materially
  different from desktop (rotor, swipe nav, double-tap
  activation); advisory at v1 until mobile parity moves
  from "supported" to "first-class".
- **TalkBack × Android × Chrome** — Android SR share is
  meaningful but TalkBack version drift across OEMs is so
  wide that a single TalkBack-on-Pixel cell is a regression
  *trend* signal, not a *gate* signal.

### Smoke corpus

Ten fixtures, fixed for v1, chosen to exercise the patterns
most likely to expose SR-specific divergence:

| # | Fixture | Why in corpus |
|---|---------|---------------|
| 1 | Button | Baseline — accessible name, pressed state, disabled |
| 2 | TextField | Label association, error announcement, helper text |
| 3 | Checkbox | Rapid state mutation — exercises NVDA's mutation buffer |
| 4 | Radio (group) | Radio-group semantics, arrow-key navigation, group label |
| 5 | Dialog | Focus trap, modal announcement, return focus on close |
| 6 | Menu | Composite roving-tabindex (ADR-023), submenu announcement |
| 7 | Tabs | `aria-controls` / `aria-selected` orientation, panel switch |
| 8 | DataGrid | Two-dimensional roving-tabindex, row/col header announce |
| 9 | live-region | `aria-live=polite` and `assertive` timing per ADR-021 |
| 10 | Disclosure | `aria-expanded` toggle, content reveal announcement |

Each fixture has roughly ~12 numbered actions in its walk-through
script. Action examples for the **Checkbox** fixture:

1. Tab to the checkbox. Expected: SR announces "<label>, checkbox, not checked".
2. Press Space. Expected: SR announces "checked".
3. Press Space again. Expected: SR announces "not checked".
4. Tab to the next focusable element. Expected: focus leaves the checkbox; no spurious re-announcement.
5. Shift+Tab back to the checkbox. Expected: SR re-announces "<label>, checkbox, not checked".
6. Programmatically set `checked=true` from devtools. Expected: SR announces the state change (or, on NVDA without focus, silent — documented per-SR).
7. ... (per-SR variations continue)

Expected-announcement strings are recorded as a **substring or
buffer-prefix** match, not exact equality. SRs vary in
verbosity settings and the reviewer is instructed to use the
*default* speech-verbosity preset; minor wording divergence
("not checked" vs "unchecked" vs "off") is recorded as Pass
with a comment noting the wording variant.

### Walk-through script format

Each cell's script lives at
`projects/jet/data/parity/manual-tests/sr/<sr>-<os>.md` and follows a
fixed template:

```markdown
# SR smoke walk-through: NVDA 2024.x × Windows 11 × Chrome

## Pre-flight
- NVDA version: <X.Y.Z>
- Chrome version: <X.Y.Z>
- Speech: eSpeak NG, rate 50, verbosity "some"
- Browse mode: ON; Focus mode: auto

## Fixture 1: Button
- URL: http://localhost:5173/fixtures/button.html
- Action 1: Tab to the button.
  Expected: "Submit, button"
  [ ] Pass [ ] Fail  Notes:
- Action 2: Press Enter.
  Expected: "pressed" or click-handler announcement
  [ ] Pass [ ] Fail  Notes:
- ... (10 more)

## Fixture 2: TextField
...
```

Run results live at
`projects/jet/data/parity/manual-tests/sr/runs/<rc>/<sr>-<os>.md`,
copied from the template and filled with checkboxes + notes.

### Aggregation: `scorecard.md`

After all six cells are run, the channel maintainer aggregates
into `projects/jet/data/parity/manual-tests/sr/runs/<rc>/scorecard.md`:

```markdown
# RC <rc> SR smoke scorecard

| Cell | Pass | Fail | Skipped | Tier | Blocking? |
|------|------|------|---------|------|-----------|
| NVDA × Win11 × Chrome   | 118 | 2 | 0 | blocking | YES — fix or revert |
| NVDA × Win11 × Firefox  | 115 | 5 | 0 | advisory | trend regression  |
| JAWS × Win11 × Chrome   | 110 | 8 | 2 | advisory | trend regression  |
| VO × Sonoma × Safari    | 120 | 0 | 0 | blocking | clean             |
| VO × iOS × Safari       | 100 | 18| 2 | advisory | trend baseline    |
| TB × Android14 × Chrome | 95  | 22| 3 | advisory | trend baseline    |

## Release gate

- NVDA × Chrome: 2 fails — **release blocked**.
- VoiceOver × Safari: 0 fails — clean.

## Action items
- Fail #1: <fixture>/<action> — <one-line summary>; issue #NNNN.
- Fail #2: ...
```

### Cadence + owner rotation

Once per release-candidate (typically every 2–3 weeks).
A single channel maintainer per RC takes the matrix end-to-end.
Rotation pinned in `projects/jet/data/parity/manual-tests/sr/ROTATION.md`.

Solo execution by design — splitting the matrix across
reviewers introduces per-reviewer variance in SR
verbosity-preset interpretation that's worse than the
~4-hour cost of one person running all six cells.

### Recording protocol

Reviewers are *encouraged but not required* to record local
audio of the SR announcement for any action where the outcome
is ambiguous (e.g. "I heard something but couldn't tell if it
was the live-region announcement or NVDA re-announcing the
focused element on tab"). Audio is:

- captured with the OS's built-in screen recorder
  (QuickTime on macOS, Game Bar on Windows, screen recorder
  on Android),
- stored only on the reviewer's local machine,
- referenced in the run checklist by relative-path note
  (`see local recording: button-action-3.mov`) so the
  reviewer or an author can re-listen and resolve the
  ambiguity asynchronously,
- **never committed** to the repo: audio is large, contains
  the reviewer's voice / ambient audio (privacy), and the
  resolution is the run checklist row, not the recording.

Audio is an aid to the reviewer, not part of the artifact.

## Consequences

### Positive

- Catches SR quirks the four-line automated stack provably
  cannot: speech buffering, mutation suppression, rotor
  inclusion/exclusion, focus-mode toggling, virtual-cursor
  positioning, OS-version-specific bugs.
- Forces a human in the loop per RC — a deliberate friction
  that keeps the team's ears on the actual user experience,
  not just on green CI dashboards.
- Scorecard trend across RCs surfaces gradual SR-perceived
  regressions even when no single RC trips the blocking gate.
- Walk-through scripts are checked-in markdown — discoverable,
  reviewable, diffable, easy to extend.
- Two-tier gate (blocking vs advisory) keeps the matrix
  feasible: we don't gate releases on JAWS license-pool
  availability or Android-OEM variance, but we still track
  those cells.
- Owner rotation distributes the SR-fluency load across the
  channel maintainers — no single person becomes the SR
  bottleneck.

### Negative

- ~4 hours of one maintainer's time per RC. Real cost, real
  pace impact on a small team.
- Manual checklists carry inter-reviewer variance even with
  a fixed verbosity preset. The "Pass / Fail per row"
  granularity is coarse on purpose; finer granularity would
  drive variance higher.
- JAWS license is commercial + single-seat floating —
  scheduling friction when two RCs land close together.
- iOS VoiceOver + TalkBack require real devices in the
  reviewer's hands; cloud device-farm SR support is poor.
- Audio recordings are not committed, so a stale checklist
  row with "see local recording" becomes useless once the
  reviewer wipes their disk. We accept that — the in-repo
  checklist row + notes is the artifact of record.

### Neutral

- The corpus is fixed at 10 fixtures for v1. Adding a fixture
  is a deliberate ADR-amendment-class change, not a casual
  PR — adding a fixture adds 6 × 12 = 72 manual checks per RC.
- The matrix does not test braille-display output; speech
  is the v1 acceptance signal. Braille is a future channel.
- "Expected announcement" strings are intentionally
  substring/prefix matches, not exact equality. Locking on
  exact strings would couple jet to SR-version-specific
  wording drift outside our control.

## Alternatives considered

### Alt-1: No SR matrix — rely on the four-line automated stack

Tempting on cost grounds; rejected on substance. The
automated stack is a necessary condition for SR parity, not
a sufficient one. The history of web a11y is littered with
sites that pass axe + AX-tree-equivalence and ship a broken
SR experience because the SR-specific quirks were never
exercised. We accept the ~4-hour-per-RC cost as the price of
not lying to users about SR support.

### Alt-2: Full per-fixture per-SR walkthrough (not a smoke)

Cover all ~60 fixtures across all six cells, not just a
curated 10. Rejected on cost: 60 × 12 × 6 = 4320 manual
checks per RC, ~24 hours per RC, infeasible on a small team.
The 10-fixture smoke is the Pareto frontier — it covers the
patterns where SR-specific divergence empirically lives
(rapid mutations, composite widgets, live regions).

### Alt-3: Cloud SR farm (BrowserStack / Sauce)

BrowserStack and Sauce offer SR-equipped VMs. Rejected
because: (a) audio output through the VM's speech channel
is unreliable on remote desktop, (b) the latency between
input and SR announcement is variable enough to invalidate
mutation-timing checks (especially live regions), (c)
iOS VoiceOver + Android TalkBack farm support is limited
or absent, (d) the cost per RC for six concurrent VM-hours
is non-trivial and the audio quality is worse than a
maintainer's own laptop.

### Alt-4: Automate the SR layer via NVDA's controller client API

NVDA exposes a controller client API that can drive
announcements and capture speech buffer contents. JAWS has
a similar (commercial) API. Rejected at v1: VoiceOver has
*no* equivalent automation surface (Apple does not expose
it), and TalkBack's automation is limited to instrumented
Android apps. Building an automation rig that covers only
the two Windows SRs would create a false symmetry — green
on the Windows side, no signal on the Apple/Android side —
that we would inevitably over-trust. The W3C ARIA-AT
project is the upstream effort to standardize SR
automation; we'll revisit when ARIA-AT covers all four SRs
with comparable fidelity.

### Alt-5: Include Orca (Linux GNOME SR) in the gating matrix

Orca is the GNOME SR; Linux desktop browser share is
single-digit percent and Orca user share within that is
smaller still. Tracked as a *separate* matrix
(`projects/jet/data/parity/manual-tests/sr/orca-*.md`), not part
of the RC gate. Reviewer rotation will run Orca opportunistically
when a Linux-equipped maintainer is on rotation.

### Alt-6: Include braille-display output in v1

Braille displays consume the same a11y tree but through a
different routing path; SR speech behavior and braille
output can diverge. Out of scope at v1 — braille hardware
is expensive, reviewer expertise is rare, and the v1
acceptance signal is speech. Future channel.

## Open questions

- **Verbosity-preset drift.** Each SR's "default" verbosity
  changes across versions. We pin a verbosity-preset
  description in the pre-flight section of each script; if
  the SR's default semantics drift, the maintainer is
  responsible for re-anchoring the preset and noting the
  drift in the RC's scorecard.
- **Wording-variant tolerance.** "Not checked" vs "unchecked"
  vs "off" — we treat all three as Pass. The exact set of
  tolerated variants per state lives in the per-SR script
  pre-flight; it will evolve as we encounter new SR builds.
- **Mobile device pool.** iOS + Android testing requires
  real devices. We have not yet pinned a device-rotation
  strategy (BYO-device vs shared org pool). v1 starts BYO.
- **ARIA-AT alignment.** The W3C ARIA-AT project publishes
  per-pattern SR test plans (https://aria-at.w3.org/). Our
  10-fixture corpus deliberately reuses ARIA-AT's pattern
  vocabulary (Button, Checkbox, Menu, Tabs, ...) so we can
  cross-reference when ARIA-AT publishes per-SR expected
  output. Open question: do we wholesale adopt ARIA-AT's
  per-pattern walk-through scripts when they stabilize, or
  keep our own? Lean: adopt where they exist, keep ours for
  composites (DataGrid, live-region) where ARIA-AT is
  incomplete.

## References

- #2162 — this ADR's issue
- Parent epic #2136 — a11y channel
- ADR-008 / #2160 — AX-tree shape diff (automated structural baseline)
- ADR-018 / #2163 — accname WPT subset (name-computation baseline)
- ADR-021 / #2161 — live-region announcement observer
- ADR-027 / #2159 — axe-core in CI (static-rule baseline)
- ADR-026 / #2158 — `<jet-semantics>` shadow subtree emitter (the AX surface SRs read)
- ADR-023 — roving-tabindex (composite-widget keyboard model)
- W3C ARIA-AT — https://aria-at.w3.org/
- NVDA — https://www.nvaccess.org/
- JAWS — https://www.freedomscientific.com/products/software/jaws/
- VoiceOver (macOS) — https://www.apple.com/accessibility/mac/vision/
- VoiceOver (iOS) — https://www.apple.com/accessibility/iphone/vision/
- TalkBack — https://support.google.com/accessibility/android/answer/6283677
- WebAIM Screen Reader User Survey — https://webaim.org/projects/screenreadersurvey/
- Adrian Roselli — running ARIA fixture catalogue — https://adrianroselli.com/
