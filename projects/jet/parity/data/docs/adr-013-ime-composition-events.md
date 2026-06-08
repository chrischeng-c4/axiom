# ADR-013: IME composition event protocol (compositionstart/update/end + beforeinput)

| Field | Value |
|-------|-------|
| Issue | #2172 |
| Parent epic | #2138 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | The hidden input proxy (ADR-010) listens to exactly six DOM events (`compositionstart`, `compositionupdate`, `compositionend`, `beforeinput`, `input`, `keydown`) and forwards a normalized three-kind `ImeEvent` stream — `composition-start { sessionId }`, `composition-update { sessionId, preeditText, caretOffset }`, `composition-end { sessionId, committedText \| null, disposition }` — to the canvas text widget; non-composition `beforeinput` / `input` events are forwarded on a separate `InputEvent` channel; the proxy is the sole source of composition events and canvas widgets MUST NOT synthesize their own |

## Context

ADR-010 (#2171) mounted a hidden `<input>` (or `<textarea>`) inside
`<jet-semantics>` for every focused canvas text widget. That input is
the DOM target that receives the OS IME's composition events. ADR-010
intentionally said *nothing* about what the listeners do with those
events. This ADR is the listener contract.

The complication is that "the composition session" is not a single
event — it is a five-event sequence, fired in browser-specific order,
on a DOM element the canvas widget cannot consume directly:

1. `compositionstart` — a new composition session opens. The
   browser begins buffering not-yet-committed text in the focused
   element's `value`.
2. `compositionupdate` — the preedit text changed. The composing
   string and the caret offset within it are both available on the
   event payload. Fires 1+ times per session, sometimes hundreds for
   long Japanese phrases that re-segment on every kana.
3. `beforeinput` — the browser is about to mutate the DOM. `inputType`
   identifies the operation: `insertCompositionText` (composition
   preedit), `insertFromComposition` (commit), `deleteCompositionText`
   (preedit retract), `insertText` (non-composition single char),
   `deleteContentBackward` (Backspace), `insertLineBreak` (Enter),
   `insertFromPaste` (paste), `insertFromDrop` (drag-drop).
4. `input` — the DOM has been mutated. Fires with `isComposing: true`
   during a composition session and `isComposing: false` outside it.
5. `compositionend` — the session ends. **The disposition (commit vs
   cancel) is browser-dependent**: Chromium sets `event.data` to the
   empty string on cancel and to the committed text on commit; Safari
   sets it to the last preedit value in both cases; Firefox sets it to
   `null` on cancel. The event itself does not tell us "did this
   commit or cancel".

Three more facts that drive the design:

- **Order disagrees across engines.** Chromium fires
  `beforeinput { insertCompositionText }` *before* `compositionupdate`.
  Firefox fires it *after*. Safari fires it only on commit, not on
  update. A canvas widget that consumed raw DOM events would have to
  re-implement that reconciliation in every test fixture.
- **The proxy's `value` is the wrong source of truth.** If the canvas
  widget reads `proxy.value` during composition, it sees the OS preedit
  string mid-flight. If it reads after `compositionend` on Safari, it
  may see the last preedit even on cancel. The widget must consume
  *events*, not `value` polls.
- **The proxy must not double-apply.** If the browser mutates
  `proxy.value` for `insertCompositionText` *and* the canvas widget
  also mirrors the preedit into its own buffer, the proxy's value
  diverges from the widget on `compositionend` — the next composition
  session starts with stale state (the bug ADR-010 § Re-mirror tries
  to clean up after the fact). The proxy must `preventDefault()` the
  browser's own mutation during composition so the widget remains the
  sole text-buffer mutator.

ADR-013 fixes all three: a normalized three-kind stream the widget
consumes, a deterministic commit-vs-cancel rule, and a
`preventDefault()` policy on composition-related `beforeinput` that
turns the proxy into a pure channel.

Without this protocol, every downstream IME slice breaks on the next
browser version: #2173 (CJK input-method tests), #2174 (Selection API
bridge), #2175 (browser-quirk corpus), #2176 (`inputmode` /
`enterkeyhint`), and #2177 (autofill) all assume "the proxy forwards
composition events in a known shape". ADR-013 is that shape.

## Decision

The hidden input proxy from ADR-010 is the **sole source** of
composition events for the canvas widget. The proxy attaches six DOM
listeners — and exactly six. The listeners synthesize a normalized
event stream with three composition kinds and a separate
non-composition input channel. The canvas widget consumes only the
normalized stream; it never reads raw DOM events and never synthesizes
composition events of its own.

### DOM event listener set (R1)

The proxy registers exactly these six listeners on the `<input>` /
`<textarea>` element at mount time:

| DOM event | Phase | Purpose |
|-----------|-------|---------|
| `compositionstart`  | bubble  | open a session, allocate `sessionId` |
| `compositionupdate` | bubble  | emit `composition-update` with current preedit + caret |
| `compositionend`    | bubble  | close the session, decide commit vs cancel |
| `beforeinput`       | capture | `preventDefault()` composition-related ops; classify non-composition ops |
| `input`             | bubble  | confirm DOM mutation; finalize commit disposition |
| `keydown`           | capture | detect `Escape` (cancel) and modifier-only keys that bypass composition |

Listeners are attached on the proxy element, not on `document` or
`window` — the at-most-one invariant from ADR-010 makes per-element
attachment the simplest model and isolates one widget's composition
from another (relevant when two `<jet-semantics>` roots co-exist on
one page, see ADR-010 § Open questions).

`beforeinput` is registered in capture phase because the
`preventDefault()` decision (R6) must be made before any framework
listener (React's synthetic-event delegate, ARIA polyfills) sees the
event and triggers its own side effects. `keydown` likewise uses
capture so `Escape`-on-composition does not race a host-app shortcut.

### Normalized event stream (R2)

The canvas widget receives only these three composition event kinds:

```ts
type ImeEvent =
  | { kind: "composition-start";  sessionId: u64 }
  | { kind: "composition-update"; sessionId: u64; preeditText: string; caretOffset: u32 }
  | { kind: "composition-end";    sessionId: u64; committedText: string | null;
      disposition: "commit" | "cancel" };
```

Non-composition `beforeinput` / `input` events are forwarded on a
**separate** `InputEvent` channel (R7):

```ts
type InputEvent = {
  inputType:    "insertText" | "deleteContentBackward" | "deleteContentForward"
              | "insertLineBreak" | "insertFromPaste" | "insertFromDrop"
              | "historyUndo"    | "historyRedo";
  data:         string | null;
  targetRanges: StaticRange[];  // from beforeinput.getTargetRanges()
};
```

The widget routes the two streams to different handlers: composition
events drive the underlined preedit overlay + commit-to-buffer
transitions; `InputEvent`s drive direct buffer mutations. Code paths
do not share state — "IME committed `ab`" and "user typed `a` then
`b`" produce identical text-buffer end-states but distinguishable
event traces, which #2173's test matrix relies on.

### Session id and overlap flattening (R3)

Every `compositionstart` allocates a fresh `sessionId` from a
monotonic u64 counter held on the proxy instance. If a `compositionstart`
arrives while a session is already open (the OS sometimes does this
when the user switches input methods mid-composition, and Firefox does
it on certain dead-key sequences), the proxy synthesizes a
`composition-end { sessionId: prior, committedText: null, disposition:
"cancel" }` event before emitting the new `composition-start`.

The widget thus sees a strictly serialized stream — no nesting, no
overlap. The acceptance test in R8 / R9 below depends on this
flattening: the widget's state machine is "either composing or not",
never "composing in two sessions at once".

### Preedit text and caret offset (R4)

`composition-update.preeditText` is `compositionupdate.event.data` as
the browser exposes it. `caretOffset` is read from
`inputProxy.selectionStart` at update time and expressed in **UTF-16
code units** relative to the start of the preedit run. The canvas
widget's text buffer also indexes in UTF-16 code units (matching the
DOM Selection API), so no conversion is required at the boundary.

UTF-16 was chosen over codepoints for one reason: the DOM
specification defines `selectionStart` in UTF-16, and translating to
codepoints requires the widget to know the preedit's surrogate
distribution before applying it. Surfacing UTF-16 keeps the widget's
indexing identical to the DOM's; #2174's `getSelection()` bridge then
becomes a copy, not a transform.

### Commit-vs-cancel disposition rule (R5)

`composition-end.disposition` is derived **not** from
`compositionend.data` (which is browser-divergent) but from whether
an `input` event with a composition-related `inputType` arrived
*within the same task* as the `compositionend`:

```
disposition = "commit"  if (within the current task) an `input` event
                          fired with inputType ∈ { "insertCompositionText",
                                                   "insertFromComposition" }
                          AND that input's isComposing was true
            = "cancel"  otherwise
```

"Within the same task" is enforced with a per-task flag set in the
`beforeinput` capture-phase handler and cleared at the end of the
microtask queue (via `queueMicrotask`). This handles the three engines
uniformly:

- **Chromium**: fires `input(insertCompositionText, true)` before
  `compositionend` on commit, and no such `input` on cancel. → matches.
- **Safari**: fires `input(insertCompositionText, true)` only on
  commit; on cancel the `compositionend.data` is the last preedit
  but no `input` fires. → matches.
- **Firefox**: fires `input(insertCompositionText, true)` before
  `compositionend` on commit; on cancel `compositionend.data` is
  `null` and no `input` fires. → matches.

`committedText` is the `event.data` of the qualifying `input` event
(commit case) or `null` (cancel case). It is **not** read from
`compositionend.data`, which by R5 is treated as untrusted.

### preventDefault policy on composition-related beforeinput (R6)

While a composition session is open (between `compositionstart` and
`compositionend`), the capture-phase `beforeinput` handler calls
`event.preventDefault()` whenever `event.inputType` is one of:

- `insertCompositionText`
- `insertFromComposition`
- `deleteCompositionText`

This prevents the browser from mutating `proxy.value` for the
composition. The canvas widget is the sole text-buffer mutator while
a session is open; the proxy's `value` remains whatever it was on
`compositionstart` until the session ends (at which point ADR-010's
re-mirror step copies the widget's authoritative state back).

Non-composition `inputType`s (`insertText`, `deleteContentBackward`,
`insertLineBreak`, `insertFromPaste`, `insertFromDrop`) are
**not** `preventDefault()`-ed — those events are forwarded on the
`InputEvent` channel and the proxy's `value` is allowed to mutate so
the OS-level autofill / spellcheck / clipboard surfaces continue to
function. The re-mirror step (ADR-010 § Re-mirror) keeps the proxy in
sync after each.

### Escape cancels the session

When `keydown` fires with `event.key === "Escape"` while a session is
open, the proxy:

1. Clears `proxy.value` to the empty string.
2. Synthesizes a `compositionend` DOM event with `data: ""` on the
   proxy. The browser-native `compositionend` may or may not also fire
   (engines disagree); if a native one arrives within the same task it
   is suppressed by sessionId comparison.
3. Calls `event.preventDefault()` and `event.stopPropagation()` on
   the `keydown` so the Escape key does not also reach the host app's
   global shortcut listener (which would close a modal or similar
   side-effect while the user is just trying to abort an IME
   composition).
4. The composition handler then emits `composition-end { sessionId,
   committedText: null, disposition: "cancel" }` per R5 (no
   qualifying `input` event arrived).

This is the rule the parent epic #2138 calls out as "the cancel path
must be deterministic, not OS-dependent" — different OSes bind the IME
cancel gesture to different keys (Escape on Windows/Linux, Esc or
Cmd-. on macOS), so the proxy treats Escape as canonical and lets
ADR-010's per-frame reposition / blur path handle anything else.

### Event-ordering contract (WHATWG UI Events)

The normalized stream preserves the canonical order from the WHATWG UI
Events spec, regardless of the underlying engine's order:

```
compositionstart
  → composition-start { sessionId: N }
[zero or more updates:]
compositionupdate          (or beforeinput-insertCompositionText, whichever the engine fires first)
beforeinput-insertCompositionText  (the other one)
input { isComposing: true, inputType: "insertCompositionText" }
  → composition-update { sessionId: N, preeditText, caretOffset }
[on commit:]
input { isComposing: true, inputType: "insertCompositionText" }   ← the commit input
compositionend
beforeinput-insertCompositionText { isComposing: false }            ← Chromium only
input { isComposing: false }                                        ← Chromium only
  → composition-end { sessionId: N, committedText, disposition: "commit" }
[or on cancel:]
keydown Escape (or OS cancel gesture)
compositionend (data: "")
  → composition-end { sessionId: N, committedText: null, disposition: "cancel" }
```

The reconciliation is implemented as a per-session buffer that
coalesces the engine-specific orderings into the canonical
`composition-start → (composition-update)+ → composition-end`
sequence. The buffer is drained on `compositionend` (or on session
overlap per R3).

The canvas widget sees the canonical sequence and only the canonical
sequence. Test fixtures (#2175's quirk corpus) record the *raw*
engine sequences; the protocol's job is to make all three converge
on the same normalized output.

### Canvas-side rendering (during composition)

While the canvas widget is composing (between `composition-start` and
`composition-end`), it paints the preedit text underlined / highlighted
at the caret position derived from `caretOffset`. The visual treatment
mirrors the WHATWG-recommended "composition span" style (single-pixel
underline, mild background tint at 12% opacity of the text color).
The committed buffer is unchanged during the session; the preedit is
an overlay, not a mutation.

On `composition-end { disposition: "commit" }`, the widget atomically:

1. Inserts `committedText` into the buffer at the original caret
   position (saved at `composition-start`).
2. Advances the caret to the end of the inserted text.
3. Drops the preedit overlay.

On `composition-end { disposition: "cancel" }`, the widget drops the
preedit overlay and restores the caret to its `composition-start`
position. The buffer is unchanged.

## Acceptance tests (R8, R9)

Both tests synthesize the raw DOM event sequence with
`new CompositionEvent(...)` / `new InputEvent(...)` and
`proxy.dispatchEvent(...)`, then assert the canvas widget's normalized
event queue matches the expected stream byte-for-byte. No real IME is
required — the tests run headless in CI.

### R8 — three-event Chromium commit

Input sequence dispatched on the proxy:

1. `compositionstart`
2. `compositionupdate { data: "a" }`
3. `compositionupdate { data: "ab" }`
4. `input { inputType: "insertCompositionText", data: "ab", isComposing: true }`
5. `compositionend { data: "ab" }`

Expected normalized queue on the canvas widget:

```
[ composition-start  { sessionId: N },
  composition-update { sessionId: N, preeditText: "a",  caretOffset: 1 },
  composition-update { sessionId: N, preeditText: "ab", caretOffset: 2 },
  composition-end    { sessionId: N, committedText: "ab", disposition: "commit" } ]
```

### R9 — Escape cancel

Input sequence dispatched on the proxy:

1. `compositionstart`
2. `compositionupdate { data: "x" }`
3. `keydown { key: "Escape" }`
4. `compositionend { data: "" }` (no preceding `input { insertCompositionText }`)

Expected normalized queue:

```
[ composition-start  { sessionId: N },
  composition-update { sessionId: N, preeditText: "x", caretOffset: 1 },
  composition-end    { sessionId: N, committedText: null, disposition: "cancel" } ]
```

Both tests live as fixtures under
`projects/jet/data/parity/fixtures/ime-composition/` and feed the parity
gate (#2144) on a `diff_kind: event_log_diff` channel — the diff is
between the synthesized raw stream and the captured normalized queue,
not against a Flutter reference (since Flutter consumes its own
`TextInput` abstraction, not DOM events).

## Consequences

- **The canvas widget has a stable, browser-independent contract.**
  Future browser changes to composition event ordering are absorbed
  by the normalization layer; no widget code changes.
- **The proxy is the single composition-event author.** Canvas
  widgets that try to synthesize their own `composition-start` / etc.
  for tests or scripted automation are rejected at build time (a
  lint rule on the widget's mutation API forbids direct emission of
  the three `ImeEvent` kinds outside the proxy module).
- **`preventDefault()` on composition-related `beforeinput` decouples
  proxy `value` from widget buffer.** The widget is the source of
  truth during composition; the proxy is a transient channel. ADR-010
  § Re-mirror remains the post-session sync point.
- **Escape is a privileged key.** Host apps that want to bind Escape
  to something during a composition (close-modal, blur-input) will
  see the keydown after `composition-end` only; during composition
  Escape is swallowed by the proxy. This is intentional — IME cancel
  must work — and is documented in the widget API surface.
- **Two streams, not one.** The canvas widget routes `ImeEvent`
  (composition) and `InputEvent` (non-composition) to different
  handlers. A widget that handles only one stream will silently miss
  the other; this is enforced by a panic-on-unknown-event guard in
  the widget's event dispatcher.
- **Session ids are not stable across reloads.** The monotonic
  counter resets on proxy mount, which happens on every focus. Tests
  that compare session ids across reloads will spuriously fail; tests
  assert *equality within a session*, not absolute values.
- **The `keydown` listener is composition-aware, not general.** It
  exists only to detect `Escape` while composing. All other keys are
  ignored by the proxy and reach the host app via the normal
  bubbling path (jet's glass-pane router, ADR-006). The widget
  receives keydowns only for non-composition input.

## Alternatives considered

- **Forward raw DOM events to the canvas widget.** Rejected: every
  widget would re-implement the same Chromium-vs-Firefox-vs-Safari
  reconciliation, and a browser-version bump would break every
  fixture. The normalization layer is the whole point.
- **Use `compositionend.data` for the commit/cancel decision.**
  Rejected by R5's analysis: the field is browser-divergent. The
  in-task-input rule is the only reliable signal across all three
  engines.
- **Always `preventDefault()` all `beforeinput`s during composition,
  not just composition-related ones.** Rejected: non-composition
  events (e.g. a paste that arrives mid-composition because the
  user hit Cmd-V to abort and paste instead) must reach the proxy
  so its `value` updates and the OS clipboard surface continues to
  work. The narrower policy in R6 preserves those paths.
- **Emit one merged event stream (composition + non-composition).**
  Rejected: the widget's state machine for IME composition is a
  strict overlay (preedit is non-buffer state) while non-composition
  input is a direct mutation (writes the buffer). Mixing them forces
  the widget to demux on every event; two channels make the demux
  the proxy's job, once.
- **Read preedit text by polling `proxy.value` on each rAF.**
  Rejected: timing is wrong (the OS sets `value` after the
  composition events fire on some engines, before on others), and
  polling cannot distinguish "preedit changed to ab" from "preedit
  was ab last frame, became a, became ab again" — sub-frame state
  changes are lost. Events are the right granularity.
- **Use the WICG `EditContext` API instead of `compositionstart`/etc.**
  Same answer as ADR-010 § Open questions: Chromium-only in 2026-05,
  spec still moving. Tracked as a follow-up.

## Open questions

1. **Coalescing rapid `compositionupdate`s.** Japanese IMEs can fire
   `compositionupdate` 30+ times per second during candidate-list
   navigation. The canvas widget repaints on every update, which is
   wasteful when the preedit text is unchanged (only the candidate
   highlight moved, which the OS owns). A future refinement could
   suppress `composition-update`s whose `(preeditText, caretOffset)`
   matches the previous event; deferred until #2173's perf
   measurements land.
2. **`compositionend` without a preceding `compositionstart`.**
   Observed on Firefox 119 with a specific IBus version under
   Wayland: a `compositionend` arrives with no opening event. The
   protocol currently drops these (no open session to close).
   Whether to instead synthesize a degenerate session pair is open;
   the answer depends on whether any in-the-wild app relies on the
   stray event.
3. **`isComposing: true` outside a session.** Engineering on Safari
   Tech Preview has shown `input { isComposing: true }` firing
   *after* `compositionend`, breaking R5's in-task assumption. The
   per-task flag is held for one extra microtask via
   `queueMicrotask` to absorb this, but the upper bound is unverified.
   Track under the browser-quirks corpus (#2175).
4. **Touch-bar / candidate-strip events.** macOS Big Sur+ exposes an
   `predictiveTextInputs` surface that fires no standard composition
   events at all — the OS just inserts the prediction as an
   `insertText`. These flow through the `InputEvent` channel today;
   no special handling. Whether to promote them to a composition
   session for parity with on-device prediction on Android is open.
5. **Multi-segment compositions.** A Japanese IME may report a single
   `compositionupdate` whose preedit text contains multiple
   underlined segments (clauses) with one "active" segment. The
   protocol currently flattens to a single preedit string and a
   single caret offset; segment metadata is not surfaced. A future
   `composition-update.segments?: Segment[]` field can extend
   without breaking the three-kind shape.

## References

- #2172 — this slice (composition event protocol).
- #2138 — parent epic: text input + IME on canvas.
- #2171 / ADR-010 — IME hidden input proxy (substrate this protocol
  runs on top of).
- #2173 — CJK input-method test matrix (consumer of this protocol).
- #2174 — Selection API bridge (consumer of this protocol).
- #2175 — browser-quirks corpus (raw event traces this protocol
  normalizes).
- #2176 — mobile keyboard / `inputmode` surface (separate concern,
  same proxy).
- #2177 — autofill events (separate channel, same proxy).
- #2139 / `dom-reference-runner.md` — `ime-trace.json` schema; raw
  composition sequences the protocol replays in tests.
- #2142 / `wpt-subset.md` — WPT uievents composition subset.
- #2144 — parity gate (the `event_log_diff` channel R8 / R9 ride on).
- WHATWG UI Events specification — canonical event ordering.
- MDN: `CompositionEvent`, `InputEvent`, `beforeinput`.
- WICG `EditContext` proposal — future migration target.
