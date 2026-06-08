# IME / Text Input Observation Channel

## Goal

The IME / text input channel observes everything a browser does *for*
the user when they type into a text field — IME composition for CJK
scripts (zhuyin, pinyin, kana, hangul), autofill from password
managers and the browser-stored profile, mobile soft-keyboard layout
selection, hardware keyboard shortcuts that act on selection, and the
Selection API that screen readers and clipboard hotkeys (Ctrl+C /
Ctrl+X / Cmd+A) read from. None of these work on a `<canvas>`
natively: canvas is a pixel surface, not a DOM text node, so the
browser has nothing to attach composition state, an `inputmode` hint,
an `autocomplete` attribute, or a `Selection` range to.

For a React-on-WebGPU app that aims to be indistinguishable from
React+MUI on the open web, "indistinguishable" must include the
Chinese, Japanese, Korean, Hindi, and Arabic users who make up most
of the world. A jet text field that loses IME composition state, or
that 1Password silently refuses to autofill, or that brings up a
QWERTY keyboard on an iPhone numeric input, is not parity — it is a
regression that excludes entire user populations. This channel
proves the contrary by routing every text-input observable through a
DOM-real proxy element while the visible UI stays on canvas.

## Architecture

jet mounts a single, invisible `<input>` (or `<textarea>` for
multiline fields) on top of the currently focused jet text widget.
The proxy is positioned absolutely over the caret rect — not over
the whole widget — so the IME candidate window pops up next to the
cursor instead of at `(0, 0)` of the page. The proxy carries the
real DOM identity for the field: its `type`, `inputmode`,
`enterkeyhint`, `autocomplete`, `name`, `aria-*`, and current value
all live on this element. The canvas only paints what the proxy
already holds.

```
<canvas id="jet-root" />               <!-- visible UI, painted by WebGPU -->
<input  id="jet-ime-overlay"           <!-- mounted on focus, removed on blur -->
        type="..."                     <!-- password|email|tel|url|text|search -->
        inputmode="..."                <!-- mobile soft-keyboard layout hint -->
        enterkeyhint="..."             <!-- return-key label -->
        autocomplete="..."             <!-- autofill / password-manager hint -->
        aria-label="..."               <!-- screen-reader name -->
        style="position:absolute;
               left: <caret.x>px;
               top:  <caret.y>px;
               width: 1px; height: 1em;
               opacity: 0;
               pointer-events: none;" />
```

The browser then drives composition natively against the proxy.
jet's WASM event bridge listens for `compositionstart`,
`compositionupdate`, `compositionend`, and `beforeinput` on the
proxy, forwards each one into the WASM TextField widget, and
re-paints the canvas to mirror the proxy's `.value` plus the
pending composition string. During composition (`event.isComposing
=== true` or any time between `compositionstart` and
`compositionend`), keyboard shortcut handling on the canvas side is
suppressed so that Enter/Backspace/letter keys are consumed by the
IME, not by jet — otherwise typing 中文 collapses into typing the
individual pinyin letters as literal ASCII.

This is the same architecture Flutter's Web engine uses. See
`flutter/engine` `lib/web_ui/lib/src/engine/text_editing/`:
`text_editing.dart` is the lifecycle orchestrator,
`composition_aware_mixin.dart` plumbs composition state through
widgets, `input_type.dart` + `input_action.dart` map widget input
type to `<input>` `type` / `inputmode` / `enterkeyhint`, and
`text_capitalization.dart` handles the `autocapitalize` attribute.
jet's WASM bridge will be the structural mirror of that engine,
expressed in Rust + wasm-bindgen instead of Dart + JS interop.

The same proxy element is the carrier for every other text-input
observable, not just IME:

- **Autofill / password managers.** 1Password, Bitwarden, Chrome
  autofill, Safari Keychain all scan the live DOM for `<input
  type=password autocomplete=current-password>`, real `<form>`
  ancestry, real `name=` attributes, and real `id=` / `for=` label
  pairing. Canvas-painted "password fields" are invisible to them.
  Only a real proxy works.
- **Mobile soft-keyboard layout.** iOS Safari, Android Chrome, and
  Android Firefox pick a keyboard layout from `inputmode`,
  `enterkeyhint`, and `type` on the focused element. Email field
  must show the `@` key; tel field must show the numeric keypad;
  numeric field must show digits. The proxy is what they read.
- **Selection API.** `window.getSelection()` reads from the DOM
  selection tree, which only includes DOM text nodes. Within a
  focused field the proxy *does* hold a real `selectionStart` /
  `selectionEnd`, so Ctrl+C, Ctrl+X, screen-reader "read
  selection", and `document.execCommand('copy')` all work *inside*
  one field. Cross-field selection (drag-select across three jet
  TextFields) is impossible without rebuilding the Selection API;
  we polyfill with a `jet.getSelection()` shim and accept the
  `window.getSelection()` divergence as a documented waiver.
- **Hardware-keyboard shortcuts.** Browser-built-in shortcuts
  (Ctrl+A select all, Ctrl+Z undo, Ctrl+Arrow word-jump) operate
  on the proxy. As long as the proxy holds the real text and real
  selection range, all of these "just work" without a single
  bespoke key handler on the canvas.

The focus channel (#2135) owns the focus-ring rendering and the
which-widget-is-focused state machine; the IME channel owns the
lifecycle of mounting and unmounting the proxy `<input>` element
that corresponds to the focused widget. The two channels share a
single source of truth — the focused widget id — and split
responsibility along the "what gets rendered on canvas" vs "what
goes on the DOM" line.

## Sub-issues

| #     | Priority | Subject                                                                 | Depends on            |
|-------|----------|-------------------------------------------------------------------------|-----------------------|
| #2171 | P1       | Hidden input proxy spec (lifecycle + caret positioning)                 | #2135 (focus)         |
| #2172 | P1       | Composition event protocol (`compositionstart/update/end` + `beforeinput`) | #2171              |
| #2173 | P1       | CJK manual test matrix (zhuyin / pinyin / kana / hangul)                | #2171, #2172          |
| #2174 | P2       | Selection API parity (`window.getSelection` polyfill)                   | #2171                 |
| #2175 | P2       | Browser quirks corpus (Safari `beforeinput` / Firefox composition timing) | #2172               |
| #2176 | P1       | `inputmode` / `enterkeyhint` mobile soft-keyboard parity                | #2171                 |
| #2177 | P1       | Autofill & password-manager integration (`autocomplete` attrs on proxy) | #2171                 |

Dependency rationale:

- #2171 is the substrate — the proxy must mount/unmount/reposition
  reliably before any of the higher-level observables (composition,
  selection, mobile keyboard, autofill) can be exercised.
- #2172 layers composition events on top of the proxy lifecycle.
- #2173 is the empirical CJK test matrix; it can only run once both
  the proxy and the composition protocol are in place.
- #2174 (Selection) needs the proxy because in-field selection is
  read from `proxy.selectionStart/End`.
- #2175 (browser quirks) is a fixture of recorded event sequences
  per browser; it observes #2172 and feeds back into its spec.
- #2176 (mobile keyboard) and #2177 (autofill) are attribute-mapping
  problems on the proxy; they don't need composition.

## Technical approach

- **Proxy lifecycle: mount on focus, unmount on blur, reposition on
  caret move.** A single `<input id="jet-ime-overlay">` is appended
  to the DOM at jet boot, kept hidden, and repurposed for each
  focused widget. On widget focus, jet sets `type`, `inputmode`,
  `enterkeyhint`, `autocomplete`, `name`, `aria-label`, `value`, and
  `selectionStart/End` from the widget state, then positions the
  element over the caret rect (`getBoundingClientRect` on the canvas
  + caret offset from jet's text layout). On widget blur or jet
  TextField unmount, the proxy is detached. Repositioning happens on
  every caret/scroll change so the IME candidate window follows the
  cursor.

- **Composition event forwarding.** jet's WASM bridge installs
  listeners for `compositionstart`, `compositionupdate`,
  `compositionend`, and `beforeinput` on the proxy. Each event is
  reduced to a `JetTextEvent` enum variant (`CompositionStart {
  data, locale }`, `CompositionUpdate { data }`, `CompositionEnd {
  data }`, `BeforeInput { input_type, data, target_ranges }`) and
  posted into the React-on-WASM event queue. The TextField widget
  reduces these into a `composing: Option<Range<usize>>` plus the
  committed value, and the canvas paint reflects both. References:
  MDN `compositionstart`, `compositionupdate`, `compositionend`,
  `beforeinput`.

- **`isComposing` guard on keyboard shortcuts.** The canvas-side
  keyboard shortcut layer (Ctrl+C, jet-defined hotkeys, Enter to
  submit form) checks `event.isComposing` or the
  `composing.is_some()` flag before acting. If composition is
  active, the key event is swallowed by the IME and jet does
  nothing. Without this guard, typing pinyin "z h o n g" with
  Enter-to-commit becomes "submit form on z then on h then on o…"
  — the most common IME bug on hand-rolled canvas apps. Reference:
  MDN `KeyboardEvent.isComposing`.

- **Mobile soft-keyboard attribute mapping.** The proxy carries
  `type` (semantic — `email`, `tel`, `url`, `password`, `search`,
  `text`), `inputmode` (layout hint — `none`, `text`, `decimal`,
  `numeric`, `tel`, `search`, `email`, `url`), and `enterkeyhint`
  (return-key label — `enter`, `done`, `go`, `next`, `previous`,
  `search`, `send`). jet ships a MUI-compatible mapping table:
  `TextField type="email"` → `type=email inputmode=email`, jet's
  custom `<PinInput>` → `inputmode=numeric
  autocomplete=one-time-code enterkeyhint=done`, etc. The full
  mapping is defined in #2176. References: MDN `inputmode`, HTML
  Living Standard `enterkeyhint`.

- **Autofill / password-manager mapping.** The proxy is wrapped in
  a real `<form>` ancestor when the jet widget is a `<Form>`
  descendant, and carries `autocomplete` tokens per HTML spec
  semantics: `username`, `current-password`, `new-password`,
  `one-time-code`, `email`, `tel`, `name` / `given-name` /
  `family-name`, `street-address`, `postal-code`, `cc-number`,
  `cc-exp`, `cc-csc`, `cc-name`. Password fields use `<input
  type="password">` for real, otherwise browsers will not save the
  credential. Fixture: jet renders the MUI sign-in template, an
  installed 1Password extension fills both fields, jet's TextField
  state and the proxy's `.value` agree after fill. Reference: HTML
  Living Standard autofill section.

- **Selection API: in-field native, cross-field polyfill.** Inside
  a focused jet TextField, the proxy holds a real selection so
  `Ctrl+A`, `Ctrl+C`, `Ctrl+Shift+Arrow`, screen-reader "read
  selection", and `document.execCommand('copy')` all work
  unchanged. For cross-widget selection (a user mouse-dragging
  from one jet text widget into another), jet exposes a
  `jet.getSelection()` shim that returns a jet-native Selection
  object; `window.getSelection()` parity across multiple canvas
  widgets is impossible without a Selection API rewrite and is
  declared out of scope. Reference: MDN `Window.getSelection`.

- **Browser quirks corpus.** WebKit, Blink, and Gecko disagree on
  the exact event ordering and on which fields `InputEvent`
  exposes during composition. Concrete known divergences: Safari
  fires `beforeinput` with `inputType="insertCompositionText"`
  *during* composition where Chromium fires `insertText` only at
  `compositionend`; Firefox delivers `compositionupdate` before
  the matching `beforeinput` whereas Chromium delivers them in the
  opposite order on some IMEs; Safari's `dataTransfer` is null for
  IME inserts. jet records the canonical event sequence per
  (browser, OS, IME) tuple as a fixture and normalises to a single
  `JetTextEvent` shape in WASM. Reference: WebKit Bug 261650 —
  `beforeinput` during composition, W3C UI Events spec issue #303
  — composition ordering.

## Dependencies

- **Foundation:**
  - #2139 — perceptual parity harness runner (drives keyboard +
    IME synthesis through Playwright / WebDriver BiDi)
  - #2144 — fixture manifest (declares per-fixture expected
    composition matrices and autofill credentials)
- **Cross-channel coupling:**
  - **#2135 focus** — shared lifecycle. The focus channel owns
    "which widget is focused"; the IME channel owns "the proxy
    `<input>` that backs it". Focus transitions trigger proxy
    mount/unmount, and focus-ring paint must coexist with
    proxy-driven `:focus` semantics on the hidden DOM element.
  - **#2136 a11y** — screen readers (VoiceOver, NVDA, JAWS,
    TalkBack) read the proxy's `value`, `selectionStart/End`,
    `aria-label`, and `role`. Without the IME channel's proxy,
    every jet text field would be silent to AT.
  - **#2137 pointer** — text-field hit testing on canvas must
    translate a click into a caret position the proxy can be
    reseated to (`setSelectionRange`).
- **External standards:**
  - HTML Living Standard — autofill
    (https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#autofill)
  - HTML Living Standard — `enterkeyhint`
    (https://html.spec.whatwg.org/multipage/interaction.html#input-modalities:-the-enterkeyhint-attribute)
  - UI Events spec — composition events
    (https://w3c.github.io/uievents/#events-compositionevents)
  - W3C Selection API
    (https://w3c.github.io/selection-api/)
- **External tooling:**
  - Flutter engine `lib/web_ui/lib/src/engine/text_editing/`
    (https://github.com/flutter/engine/tree/main/lib/web_ui/lib/src/engine/text_editing)
    as structural reference for the Rust/wasm-bindgen port
  - Flutter `web_ui/dev/test_platform.dart` — manual test page
    setup we mirror for jet's CJK test matrix
- **External corpus gap:**
  - No public CJK / Korean / Japanese / mobile-keyboard parity
    test corpus exists. We ship our own internal manual test plan
    as part of #2173 and treat it as a project-owned artifact.

## Success criteria

- **CJK composition.** The manual CJK matrix (Zhuyin macOS,
  Zhuyin Windows, Pinyin Google IME, Pinyin Microsoft IME,
  Japanese Google IME romaji + kana, Japanese Apple IME, Korean
  2-set, Korean 3-set) passes on macOS + Windows + iOS + Android
  against the jet-rendered fixture and the React+MUI baseline
  produces byte-identical committed text and an indistinguishable
  candidate window position. Tester signs off per (IME, OS) cell.
- **Mobile soft-keyboard.** On iOS Safari and Android Chrome the
  keyboard layout that appears for each jet TextField variant
  matches the layout the MUI baseline produces for the same
  semantic field. Screenshot diff of the soft-keyboard region for
  `type=email`, `inputmode=numeric`, `inputmode=tel`,
  `enterkeyhint=search`, `enterkeyhint=send` is within tolerance.
- **Autofill.** 1Password (macOS + iOS) and Chrome built-in
  autofill correctly fill the jet sign-in fixture's username +
  password fields; jet TextField state and proxy `.value` agree
  post-fill; the password-save prompt fires on form submit. Same
  test re-run against React+MUI baseline produces the same
  outcome.
- **In-field selection + clipboard.** Ctrl+A / Ctrl+C / Ctrl+X /
  Ctrl+V / Cmd+A / Cmd+C / Cmd+X / Cmd+V inside a single jet
  TextField produce identical clipboard contents and identical
  post-paste field state vs the baseline. Screen-reader "read
  selection" announces the correct substring on VoiceOver, NVDA,
  and TalkBack.
- **`isComposing` guard.** A regression test types Chinese pinyin
  "zhong wen" + Enter through synthesised composition events; the
  Form `onSubmit` handler must NOT fire during composition, and
  must fire exactly once when the Enter key arrives after
  `compositionend`.
- **Browser quirks corpus.** Recorded event sequences for the
  IME + browser tuples we ship support for are checked into the
  fixture tree, and the WASM bridge's normalised `JetTextEvent`
  stream is byte-identical across browsers for the same input.

## Out of scope / waivers

- **Browser-native `window.getSelection()` across multiple jet
  widgets.** Impossible to fully match without rebuilding the
  Selection API and the DOM range model. We document the
  divergence, polyfill cross-widget selection through
  `jet.getSelection()`, and ship a migration shim for code that
  expects the global Selection. In-field selection (most use
  cases — Ctrl+C of selected text inside one TextField) remains
  native-correct.
- **Browser-built-in spell check.** `spellcheck=true` on the proxy
  enables it for the proxy but the red-underline overlay is drawn
  over the proxy, not over canvas text. We accept this divergence
  as "no spellcheck visible to user" rather than attempt a
  canvas-side dictionary.
- **Browser-built-in grammar / writing-assistance overlays**
  (Chrome "help me write", Safari Writing Tools) — these depend
  on visible DOM text and can't see canvas. Out of scope, no
  fallback.
- **Browser-native text-search-in-page (Ctrl+F / Cmd+F)** over
  canvas-painted text. We expose a jet-side Cmd+F polyfill in the
  a11y channel (#2136), but the browser's built-in Find Bar
  remains blind to canvas text — documented divergence.
- **System-level dictation** (macOS Dictation, Windows Speech
  Recognition) — these *do* work on the proxy because they
  deliver `beforeinput` events like any other text source, so
  they are in scope by default and only get a waiver if a
  specific IME-style dictation engine misbehaves.
- **Drag-and-drop of text from a jet TextField to an OS-native
  app** — depends on real DOM `dragstart` from a text node;
  proxy doesn't cover this. Possible follow-up if user demand
  shows up; not blocking parity baseline.

## Prior art and references

- Flutter Web engine — text_editing module
  (https://github.com/flutter/engine/tree/main/lib/web_ui/lib/src/engine/text_editing)
  — the reference architecture for canvas-IME bridging on the
  open web. Files of particular interest:
  `text_editing.dart` (lifecycle orchestration),
  `composition_aware_mixin.dart` (composition state plumbing),
  `input_type.dart` (HTML `type` mapping),
  `input_action.dart` (`enterkeyhint` mapping),
  `text_capitalization.dart` (autocapitalize mapping).
- Flutter engine `web_ui/dev/` manual test pages
  (https://github.com/flutter/engine/tree/main/lib/web_ui/dev)
  — Flutter's own manual IME test rig; jet mirrors its structure
  for the CJK matrix in #2173.
- MDN — CompositionEvent
  (https://developer.mozilla.org/en-US/docs/Web/API/CompositionEvent)
  — base type for `compositionstart/update/end`.
- MDN — `compositionstart` event
  (https://developer.mozilla.org/en-US/docs/Web/API/Element/compositionstart_event)
  — when it fires.
- MDN — `compositionupdate` event
  (https://developer.mozilla.org/en-US/docs/Web/API/Element/compositionupdate_event)
  — incremental composition data.
- MDN — `compositionend` event
  (https://developer.mozilla.org/en-US/docs/Web/API/Element/compositionend_event)
  — commit boundary.
- MDN — `beforeinput` event
  (https://developer.mozilla.org/en-US/docs/Web/API/Element/beforeinput_event)
  — cancelable input event, `inputType` enumeration.
- MDN — `InputEvent.inputType`
  (https://developer.mozilla.org/en-US/docs/Web/API/InputEvent/inputType)
  — the full enum of input change kinds (`insertText`,
  `insertCompositionText`, `deleteContentBackward`, etc.).
- MDN — `KeyboardEvent.isComposing`
  (https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/isComposing)
  — the guard flag.
- MDN — `inputmode` global attribute
  (https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/inputmode)
  — mobile keyboard layout hints.
- HTML Living Standard — `enterkeyhint`
  (https://html.spec.whatwg.org/multipage/interaction.html#input-modalities:-the-enterkeyhint-attribute)
  — return-key label hints.
- HTML Living Standard — autofill
  (https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#autofill)
  — full autocomplete token enumeration, password-manager
  semantics.
- MDN — `Window.getSelection`
  (https://developer.mozilla.org/en-US/docs/Web/API/Window/getSelection)
  — Selection API entry point.
- W3C Selection API
  (https://w3c.github.io/selection-api/)
  — Selection / Range model.
- UI Events spec — composition events
  (https://w3c.github.io/uievents/#events-compositionevents)
  — canonical event sequencing.
- Chromium IME documentation
  (https://chromium.googlesource.com/chromium/src/+/refs/heads/main/docs/ime_input.md)
  — Blink's view of composition lifecycle.
- WebKit Bug 261650 — `beforeinput` during composition
  (https://bugs.webkit.org/show_bug.cgi?id=261650)
  — known Safari divergence.
- W3C UI Events issue #303 — composition event ordering
  (https://github.com/w3c/uievents/issues/303)
  — known cross-browser ordering issue.
- 1Password autofill anatomy
  (https://developer.1password.com/docs/web/compatible-website-design/)
  — how password-manager extensions discover login fields;
  explains the `<form>` + `name=` + `autocomplete=` requirements
  jet's proxy must satisfy.
