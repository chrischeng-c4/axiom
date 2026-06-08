# ADR-032: inputmode + enterkeyhint on the IME proxy

| Field | Value |
|-------|-------|
| Issue | #2176 |
| Parent epic | #2138 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | The canvas `TextField` widget exposes two semantic props — `inputMode: "text" \| "email" \| "tel" \| "numeric" \| "decimal" \| "search" \| "url" \| "none"` (default `"text"`) and `enterKeyHint: "enter" \| "done" \| "go" \| "next" \| "previous" \| "search" \| "send"` (default `"enter"`) — that are mirrored, one-directionally, onto the hidden `<input>` proxy from ADR-010 (#2171) via `setAttribute("inputmode", …)` / `setAttribute("enterkeyhint", …)`. Mount-time writes happen before the proxy receives its first `.focus()` call, so the OS soft keyboard's *initial* render reads the right values. Mid-session prop changes flush in the same animation frame as the canvas re-render; on platforms whose soft keyboard does not re-evaluate attributes on a focused element (Android Chrome ≤ a documented version, iOS Safari for `inputmode` transitions across the `"none"` boundary — see the per-platform matrix in §6) jet executes `proxy.blur(); setAttribute(…); proxy.focus({ preventScroll: true });` inside the same frame to force the keyboard to redraw. Unknown enum values fall back to the default with a `console.warn` and the attribute is set to the default string — the proxy never carries a value the HTML spec does not enumerate. `inputMode="none"` is a first-class signal: the proxy stays mounted and focused (so composition events from ADR-013/#2172 still fire and autofill from #2177 still resolves) but the soft keyboard is suppressed, which is the contract autocomplete dropdowns and on-canvas keypads rely on. A `data-jet-mobile-attrs="off"` opt-out on the canvas host disables propagation entirely. Validation is two-layered: a unit test asserts `proxy.getAttribute("inputmode")` / `…("enterkeyhint")` round-trip through mount and prop change; a headless-Chromium mobile-emulation test (`device: "Pixel 5"`) asserts the selector `input[inputmode="email"]` matches the proxy after a `TextField` is configured with `inputMode: "email"`. Real-device verification (the actual soft-keyboard sprite) lands as a row in ADR-014's (#2173) CJK manual matrix. |

## Context

A mobile soft keyboard is a separate, OS-owned UI surface that the
browser asks the operating system to draw whenever an editable
element receives focus. The browser does not draw the keyboard; it
forwards two pieces of metadata from the focused DOM node — the
element's `inputmode` attribute (which **layout** to use: alphabetic,
number pad, telephone, email QWERTY with a `@` key, URL QWERTY with
a `.com` key, search bar with a magnifier glyph, or a sentinel
"render no keyboard" value) and its `enterkeyhint` attribute (which
**label** to put on the action key in the bottom-right of the
keyboard: "Return", "Done", "Go", "Next", "Previous", "Search",
"Send"). The OS keyboard reads both, decides which sprite sheet to
mount, and fires its key events back into the focused element as
synthesised input.

This is one of the highest-leverage UX signals on mobile. A
`type="email"` field with `inputmode="email"` mounts a keyboard
where `@` and `.` are visible without a shift-key tap. A
`type="tel"` field with `inputmode="tel"` mounts the 12-key telephone
pad — `*` and `#` included, no alphabetic keys at all. A search bar
with `inputmode="search" enterkeyhint="search"` mounts the QWERTY
keyboard with the action key relabelled and recoloured to "Search".
Get the attributes wrong and the user sees the generic alphabetic
keyboard with a "Return" key every time, forced to tap shift +
number-row + special-character cycles to type a phone number.

For a real DOM page the work is two attributes per `<input>`. For a
canvas-rendered surface like jet, the work is non-trivial: the
single host `<canvas>` element has no `inputmode`; the OS keyboard
will never read a canvas attribute. The keyboard reads from
**whatever element is currently focused**, and on jet that is the
hidden `<input>` proxy from ADR-010 (#2171), which exists precisely
so that the IME, autofill, and OS soft keyboard have a real DOM node
to bind to. Therefore: the propagation pipeline that ADR-010 set up
for IME composition events is the same pipeline this ADR extends for
soft-keyboard attribute metadata.

The complication is that propagation is not just a one-shot
mount-time write. A canvas `TextField` widget can mutate its props
mid-session — the canonical example is a multi-step form widget that
advances from a name field to an email field to a phone field
without unmounting the proxy element, because unmounting would tear
down the focus, fire blur events, and dismiss the keyboard. The
proxy's `inputmode` attribute must change *while focused*, and the
keyboard must redraw without a focus-cycle hitch the user can see.
Each mobile browser handles "attribute change on a focused element"
differently — some redraw the keyboard live, some require a blur+
focus cycle within the same frame, one (iOS Safari, for the
`"none"` ↔ non-`"none"` transition specifically) requires the cycle
*and* a sub-frame delay before refocus. The pipeline must read a
per-platform behaviour matrix and pick the right update path.

A second complication is the interaction with composition. An open
IME composition (Pinyin, Zhuyin, Kana, Hangul) carries internal
state inside the OS keyboard — half-typed jamo, candidate window
position. Naively executing a blur+focus cycle in the middle of a
composition will commit the in-progress composition string and
dismiss the candidate window, which is destructive. Jet must
suppress the cycle while a composition is open (the `compositionstart`
/ `compositionend` window from ADR-013 / #2172) and queue the
attribute change for the next composition-free frame.

A third complication is `inputmode="none"`. This is not a layout
value; it is a sentinel for "I want the proxy focused for autofill,
accessibility, and composition routing, but suppress the soft
keyboard entirely". Jet uses this when it shows its own custom
on-canvas keypad (a date picker, a numeric stepper, an emoji
selector) and does not want the OS keyboard to also pop up and
fight for screen real estate. Composition events must still fire
as normal — the proxy is focused, the IME is bound, the OS keyboard
just isn't drawn. All three supported mobile platforms (Android
Chrome, Android Firefox, iOS Safari) honour `inputmode="none"`;
that is the contract.

Issue #2176 scopes this surface narrowly: which canvas-widget props
map to which proxy attributes, the update protocol for mid-session
changes, the per-platform behaviour matrix, the opt-out, and the
test surface that proves the mapping holds in CI without a physical
device farm. Other proxy-attribute pipelines (`autocomplete` →
#2177, `autocapitalize` / `spellcheck` → deferred) share the
mid-frame propagation mechanism described here but are not the
subject of this ADR.

## Decision

### Canvas widget prop surface

`TextField` accepts two new props:

- `inputMode?: "text" | "email" | "tel" | "numeric" | "decimal" | "search" | "url" | "none"` — default `"text"`.
- `enterKeyHint?: "enter" | "done" | "go" | "next" | "previous" | "search" | "send"` — default `"enter"`.

Both are propagated, one-directionally, to the hidden `<input>`
proxy from ADR-010. No other canvas-widget attributes participate
in this issue; future attributes ride the same pipeline but each
gets its own ADR + issue.

Per-widget defaults — applied when the consumer does not set
`inputMode` / `enterKeyHint` explicitly but does set a semantic
`type`:

| `type` prop | implied `inputmode` | implied `enterkeyhint` |
|-------------|---------------------|------------------------|
| `"number"`  | `decimal`           | (no default — caller wins) |
| `"tel"`     | `tel`               | (no default) |
| `"email"`   | `email`             | (no default) |
| `"url"`     | `url`               | (no default) |
| `"search"`  | `search`            | `search` |
| `"text"`    | `text`              | `enter` |

Explicit props always win; the table is a fallback so that a caller
who writes `<TextField type="email" />` gets the email keyboard
without typing `inputMode="email"` a second time.

### Mount-time propagation

When ADR-010's proxy element is created and inserted into the DOM
for a given canvas `TextField`, jet writes both attributes via
`setAttribute("inputmode", …)` / `setAttribute("enterkeyhint", …)`
**before** the proxy receives its first `.focus()` call. Order
matters: every supported mobile browser reads the attributes at
focus time to pick the keyboard sprite sheet. Writing after focus
risks a frame of the default alphabetic keyboard.

If either attribute is at its default (`"text"` / `"enter"`), jet
still writes it explicitly rather than omitting it — explicit
attribute values make the proxy state inspectable from devtools and
unit tests without per-default fallback logic.

### Mid-session updates

When the canvas widget mutates either prop mid-session, the update
runs inside jet's canvas re-render frame, after the proxy element
has been confirmed to still exist (per ADR-010's lifecycle) and
*before* any synthetic composition or autofill event for the same
frame. The update path branches on the per-platform behaviour
matrix (§6):

- **Live update** (Android Chrome on a recent build, both attributes,
  when no `"none"` transition is involved): `proxy.setAttribute(…)`
  in place; the keyboard redraws on the next OS-keyboard frame.
- **Blur+focus cycle** (Android Firefox; Android Chrome when
  `enterkeyhint` changes; iOS Safari for `enterkeyhint` only):
  `proxy.blur(); proxy.setAttribute(…); proxy.focus({ preventScroll: true });`
  all in the same frame. The `preventScroll: true` is required —
  without it iOS Safari scrolls the input back into view, which on
  a canvas surface jumps the host element under our own viewport
  manager.
- **Blur+rAF+focus** (iOS Safari, `inputmode` crossing the `"none"`
  boundary in either direction): `proxy.blur(); proxy.setAttribute(…);
  requestAnimationFrame(() => proxy.focus({ preventScroll: true }))`.
  The single-frame delay is the empirical minimum we found for
  WebKit to release the "no keyboard" state before re-evaluating.

The update protocol is implemented as a single function that reads
the platform matrix, the current attribute, and the next attribute,
and returns one of three strategies. The strategy is applied
inside the same RAF callback as the canvas paint, so the soft
keyboard, the canvas widget, and the cursor sprite (ADR-028) all
update in the same visible frame.

### Composition-window deferral

If `proxy.compositionstart` has fired without a matching
`compositionend` (i.e., an IME composition is open — see ADR-013 /
#2172), jet does **not** run the blur+focus cycle even on platforms
that would otherwise require it. The new attribute value is
written to the proxy's `dataset.jetPendingInputmode` (or
`…EnterKeyHint`), the proxy keeps its current focus and current
keyboard, and on the next `compositionend` jet flushes the pending
attribute via the platform's normal strategy. Composition state is
preserved; the user does not see a half-typed Pinyin candidate get
committed prematurely.

Live-update platforms (Android Chrome on attribute changes that
don't require a cycle) still write the attribute mid-composition;
the keyboard's sprite swap is non-destructive there.

### Validation

Invalid enum values for either prop fall back to the default
(`"text"` / `"enter"`) and emit a single `console.warn(...)` per
invalid value per `TextField` instance (deduplicated by value, so a
flapping prop does not spam the console). The proxy never carries
an attribute string outside the HTML-spec enumeration — the spec
defines a closed set, and treating that set as closed is what lets
the per-platform matrix in §6 be exhaustive.

### Opt-out

A `data-jet-mobile-attrs="off"` attribute on the canvas host
element disables the entire propagation pipeline: the proxy carries
neither `inputmode` nor `enterkeyhint`, and prop changes on canvas
widgets are no-ops for this channel. This is the recovery path for
downstream consumers integrating a custom mobile keyboard
(third-party SDKs that bind to a focused-but-unattributed input)
who do not want jet's mapping to interfere.

### Test surface

Two test layers, both runnable in headless CI:

1. **Unit (Vitest, jsdom)**: mount a `TextField` with given props,
   assert `proxy.getAttribute("inputmode")` and `…("enterkeyhint")`
   equal the expected strings. Mutate props, assert the proxy
   updates. Set an invalid value, assert fallback + a single
   `console.warn`. Set `data-jet-mobile-attrs="off"` on the host,
   assert the proxy carries neither attribute.
2. **Mobile-emulation E2E (Playwright + `devices['Pixel 5']` and
   `devices['iPhone 13']`)**: render a fixture page with one
   `TextField` per `inputMode` value; for each, focus the field and
   assert `document.querySelector('input[inputmode="email"]')` (etc.)
   matches the proxy. Same for `enterKeyHint`. These tests do not
   verify the actual rendered keyboard sprite — that is impossible
   in headless emulation; the device-frame screenshot row in
   ADR-014 (#2173) covers the visual side.

### Platform behaviour matrix (R7)

Per #2176 R7, the propagation implementation reads this matrix to
pick the update strategy. The matrix is documented in the spec
file `jet-ime-mobile-platform-behaviour-matrix.md` and re-checked
quarterly against the latest stable browser builds:

| Platform | `inputmode` change | `enterkeyhint` change | `inputmode="none"` ↔ other |
|----------|--------------------|-----------------------|----------------------------|
| Android Chrome (recent stable) | live update | blur+focus cycle | blur+focus cycle |
| Android Firefox | blur+focus cycle | blur+focus cycle | blur+focus cycle |
| iOS Safari | live update for non-`"none"`; blur+rAF+focus for `"none"` boundary | blur+focus cycle | blur+rAF+focus |

Behaviour for older browser versions (Android Chrome before the
"recent stable" cutoff documented in the spec, iOS Safari ≤ a
documented version) is treated as `blur+focus cycle` as the
universal-fallback strategy — slightly more expensive, always
correct.

## Consequences

### Positive

- `TextField type="email"` on iOS Safari mounts the email-QWERTY
  keyboard with `@` and `.` visible on the first key row. Same
  for `tel`, `numeric`, `decimal`, `url`, `search`. The mobile
  keyboard UX matches what the consumer would have gotten from a
  plain DOM `<input>`.
- `enterKeyHint="search"` on a search bar mounts the keyboard with
  the action key labelled "Search" (Android) / "Search" with a
  magnifier glyph (iOS), instead of "Return". The visual difference
  is immediate and is the kind of polish detail that gets called
  out in mobile UX reviews of canvas apps.
- The propagation pipeline is reusable: #2177 (`autocomplete`)
  rides the same one-directional canvas → proxy `setAttribute`
  path, the same RAF-frame flush, and the same composition-window
  deferral. Building it correctly here reduces #2177 to "a third
  attribute on the existing pipeline".
- `inputMode="none"` unblocks on-canvas pickers (date, emoji,
  custom numeric stepper) without sacrificing focus, autofill, or
  composition routing — a critical building block for the rest of
  the canvas-widget catalogue.
- The opt-out (`data-jet-mobile-attrs="off"`) preserves the option
  for downstream SDK integrations to retain control of the focused
  input's attributes, which we want to be possible even if we
  cannot test it ourselves.

### Negative

- The mid-session update path is genuinely complex (three
  strategies × two attributes × composition-window deferral), and
  the platform matrix in §6 is empirical — when a browser version
  changes its keyboard-redraw heuristics we will only find out by
  shipping a regression. The quarterly re-check is a real
  maintenance commitment, not a one-time setup.
- We cannot fully verify the keyboard renders correctly in CI;
  the closest we get is `input[inputmode="…"]` selector matching
  in mobile emulation, plus the manual ADR-014 device-frame row.
  Regressions where the attribute is set correctly but the
  keyboard mis-renders (e.g., a future iOS bug) escape headless CI.
- The `console.warn` on invalid enum values is a low-frequency but
  user-visible signal — consumers who pass unvalidated user input
  (`<TextField inputMode={user.preference} />`) will see warnings
  in the console. We accept this as the cost of the closed-enum
  contract: silently accepting the value would let a typo
  (`"numerc"`) ship to the proxy and cause the OS keyboard to
  ignore the attribute entirely, which is the worse failure mode.
- The blur+focus cycle, even with `preventScroll: true`, is a
  user-visible event on platforms that animate the keyboard
  dismissal (Android Firefox briefly slides the keyboard down 1-2
  px before it slides back up). This is a known cosmetic blemish
  on the mid-session path; the alternative — never updating
  attributes mid-session — is worse.

### Neutral

- `inputmode` / `enterkeyhint` are no-ops on desktop browsers
  (and on mobile when a hardware keyboard is attached). Setting
  them costs two `setAttribute` calls per mount + per mutation;
  measurable in microbenchmarks, irrelevant in practice.
- Attribute propagation does not interact with the screen-reader
  announcement pipeline (ADR-021 / #2161): the proxy carries
  `aria-label` / `aria-describedby` independently, and the
  `inputmode` / `enterkeyhint` attributes are scoped to the OS
  soft keyboard.

## Alternatives considered

### A. Always blur+focus cycle on every mid-session update

Skip the platform matrix; just always do the cycle. Simpler code
(one strategy, no matrix lookup). Rejected because the cycle is
visible to the user on Android Firefox's keyboard-dismiss animation
and on iOS Safari when the rAF gap is forced even where it isn't
needed. The matrix exists specifically to take the cheapest path
per platform and use the expensive path only where required.

### B. Defer all updates to canvas frame N+1 via a global queue

Batch all proxy-attribute writes into a separate queue flushed
after the canvas paint. Decouples the attribute write from the
widget render. Rejected because the OS keyboard's redraw is
already on a different visual clock from the canvas; adding a
second-frame lag means a user who taps a "next" button on a form
sees the canvas advance to the email field a frame before the
keyboard updates to the email layout. Same-frame flush keeps the
two visually synchronised.

### C. Use the Virtual Keyboard API (`navigator.virtualKeyboard`)

The browser-native VK API exposes `overlaysContent` and a
`geometrychange` event for the keyboard's screen rectangle. It
does **not** influence which layout the keyboard renders — that
remains a function of `inputmode` / `enterkeyhint` on the focused
element. The VK API is a different channel (viewport / inset
management); it does not substitute for this ADR's pipeline. It
will likely have its own ADR once jet adds in-canvas keyboard-inset
awareness for sticky toolbars.

### D. Render jet's own software keyboard

Skip `inputmode` entirely; draw an in-canvas keyboard above the
field. Rejected as a non-goal in #2176's Out of Scope: jet defers
to the OS soft keyboard, period. (Native-style consistency, IME
support across CJK / Indic / Arabic input methods, accessibility
audio cues, haptics — the OS keyboard wins on all five, and the
maintenance cost of a hand-rolled keyboard is enormous.)
`inputMode="none"` is the narrow exception, used by on-canvas
*pickers* that are *not* general-purpose keyboards.

### E. Propagate attributes via React DOM (no proxy)

Render a real DOM `<input>` element in the React tree instead of a
hidden proxy. Rejected: that's the design ADR-010 already rejected
upstream (#2171) — a real DOM input fights the canvas for layout,
focus, and event handling. The proxy exists so the OS keyboard
sees a real `<input>` while the canvas remains the single visual
surface; this ADR rides ADR-010, not around it.

## Open questions

- **Quarterly re-check ownership.** §6's platform matrix needs an
  owner who runs the keyboard-redraw probe on the four targets
  (Android Chrome stable, Android Chrome beta, Android Firefox,
  iOS Safari stable) once a quarter and PRs any matrix changes.
  Proposed: rotates with the ADR-014 (#2173) CJK matrix re-check;
  same physical-device sessions can cover both.
- **`autocapitalize` / `spellcheck` deferral.** #2176 explicitly
  defers these two attributes. Their propagation will share this
  ADR's pipeline; the only open question is whether they get
  per-attribute ADRs or fold into a single "remaining mobile-input
  attributes" follow-up. Tracking issue: TBD.
- **`inputmode="none"` on Android Firefox.** Empirical: Android
  Firefox sometimes shows a one-frame flicker of the alphabetic
  keyboard when transitioning *into* `"none"` from a focused
  alphabetic field. The transition is correct (keyboard dismisses
  on the next frame) but the flicker is user-visible. Mitigation:
  either an extra `requestAnimationFrame` before the `setAttribute`
  to coincide the dismissal with a paint boundary, or accept the
  flicker as an upstream browser bug. Decision deferred to the
  first user report.
- **Localised `enterkeyhint` labels.** The OS keyboard chooses the
  display string for the action key based on the user's system
  locale (German `enterkeyhint="search"` renders "Suchen"). Jet
  does not influence this — the attribute names the *semantic*,
  the OS chooses the label. No action required, noted here so
  future contributors do not look for a hook that does not exist.

## References

- Issue #2176 — parity/ime — inputmode / enterkeyhint mobile soft-keyboard parity
- Parent epic #2138 — IME parity
- ADR-010 (#2171) — hidden input proxy element (the DOM node these attributes are written to)
- ADR-013 (#2172) — IME composition event protocol (composition-window deferral)
- ADR-014 (#2173) — CJK manual matrix (real-device verification row)
- #2177 — autocomplete attribute propagation (sibling pipeline, shares the mechanism)
- HTML Standard — `inputmode` attribute (closed enum, normative)
- HTML Standard — `enterkeyhint` attribute (closed enum, normative)
- Spec `jet-ime-mobile-soft-keyboard-attrs.md` — propagation pipeline contract
- Spec `jet-ime-mobile-platform-behaviour-matrix.md` — per-platform update strategy
- Spec `jet-ime-attribute-propagation-pipeline.md` — shared with #2177
