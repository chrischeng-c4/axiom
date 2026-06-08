# ADR-016: Selection API parity (window.getSelection polyfill via input proxy + shadow-tree)

| Field | Value |
|-------|-------|
| Issue | #2174 |
| Parent epic | #2138 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | The hidden input proxy (ADR-010) is the DOM-side anchor for jet's Selection API surface; `inputProxy.selectionStart` / `selectionEnd` / `setSelectionRange` are intercepted and bidirectionally bridged to the focused canvas text widget in UTF-16 code units; `<jet-semantics>.getSelection()` and a focus-gated override of `document.getSelection()` return a jet-aware `Selection` whose single `Range` addresses `(inputProxy, start) → (inputProxy, end)`; external `setBaseAndExtent` / `addRange` calls targeting the proxy are forwarded to the canvas as `set-selection { start, end, direction }`; `selectionchange` is dispatched at most once per animation frame on canvas selection mutation; `Selection.prototype` is never monkey-patched; cross-widget selection is forbidden — each widget owns one mirror node, selections clamp to one widget |

## Context

ADR-010 (#2171) mounted a hidden `<input>` (or `<textarea>`) inside
`<jet-semantics>` for every focused canvas text widget — the "input
proxy". ADR-013 (#2172) turned it into a normalized composition
event source. Both ADRs left the Selection-API surface unspecified:
the proxy has `selectionStart` / `selectionEnd` / `setSelectionRange`
on it because it is a real `<input>`, but those accessors today
return the proxy element's *own* (empty, never-written) selection,
not the canvas widget's selection. Every external Selection-API
consumer therefore sees zero.

The complication is that "the selection" is not one object — it is
*two* objects in different coordinate systems that must agree:

1. **Canvas-side selection.** The widget stores its caret/range as
   `(start_u32, end_u32, direction)` over its internal text buffer.
   The buffer's indexing is currently UTF-16 code units (matching
   ADR-013 § R4), but the widget API treats this as an internal
   detail. Only the canvas paints this selection.
2. **DOM-side Selection.** A `Selection` object hangs off the
   document and the shadow root. Each of its zero-or-more `Range`s
   is a `(startContainer, startOffset, endContainer, endOffset)`
   tuple where the containers are real DOM nodes and the offsets
   index into those nodes' character data in **UTF-16 code units**.
   Every screen reader, every clipboard surface, every IME
   candidate-anchor, every browser extension reads this.

Bridging them sounds trivial — copy `start, end` across — but five
things make it subtle:

- **The proxy is an `<input>`, not a text node.** A `Range` whose
  `startContainer` is an `<input>` is well-defined but useless as a
  text-range anchor (offsets index children, of which there are
  none). The shadow-tree `getSelection()` must therefore *synthesize*
  Ranges whose offsets are interpreted as if the proxy were a text
  node whose `data` is `proxy.value`.
- **IME composition mutates `Selection` mid-flight.** During Japanese
  IME composition the OS sets `Selection` to the *target-clause range*
  inside the preedit so the candidate window can anchor to the right
  characters. Readers want that target-clause range, not the canvas's
  pre-composition caret. R9 codifies this: between ADR-013's
  `composition-start` and `composition-end`, reads return the IME's
  last-known target-clause range.
- **`Selection.prototype` is shared globally.** Patching it would
  break every native `<textarea>` / `<input>` / `contenteditable` on
  the page. The polyfill instead lives on (a) the proxy element's own
  selection accessors via per-instance `Object.defineProperty`, and
  (b) a lexically scoped `<jet-semantics>.getSelection()`. The global
  `document.getSelection()` is overridden once, but focus-gated: it
  delegates only when the proxy is the active element, otherwise the
  native call passes through.
- **`selectionchange` fires from two sides.** The browser fires it on
  the proxy when the user drags / `setSelectionRange`s; the canvas
  fires it on arrow keys, click-drag-on-canvas, or IME caret update.
  Both must converge on one observable stream — at most one
  `selectionchange` per animation frame.
- **UTF-16 conversion at the boundary.** The WHATWG spec defines all
  offsets in UTF-16 code units. The canvas backend *currently* indexes
  in UTF-16 too, but the widget API does not guarantee that — a
  future grapheme-cluster or codepoint backend would break the bridge
  silently. The polyfill converts at every boundary call site even
  when the conversion is presently a no-op.

Without this protocol every downstream Selection-dependent surface
breaks: screen readers read empty selection, copy-paste fails to
copy the canvas widget's selected text, IME candidate windows
anchor to `(0, 0)` of the proxy instead of the target clause, and
the upcoming Selection-API-based AT virtual-cursor (#2178) has
nothing to read. ADR-016 is the bridge.

## Decision

The input proxy from ADR-010 is the **sole DOM-side anchor** for
jet's Selection API surface. The polyfill installs in three places:

1. Per-instance selection-accessor interception on the proxy element.
2. A `getSelection()` override on the `<jet-semantics>` shadow root.
3. A focus-gated `document.getSelection()` override that delegates
   to (2) when the proxy is the active element and falls through to
   the native implementation otherwise.

The canvas widget exposes one ingress and one egress command:

- Ingress: `set-selection { start, end, direction }` — applied to
  the canvas widget's selection state. Offsets are UTF-16 code units
  over the canvas buffer.
- Egress: `selection-changed { start, end, direction }` — fired by
  the canvas widget when its selection mutates (caret move, drag,
  IME caret update, programmatic API call). Offsets are UTF-16.

The polyfill is the only consumer of `selection-changed` and the
only producer of `set-selection` for selection-bridge purposes.

### Per-instance selection-accessor interception (R1, R2)

At proxy mount time, the polyfill replaces the proxy element's
`selectionStart`, `selectionEnd`, and `selectionDirection`
properties with per-instance accessor descriptors via
`Object.defineProperty(proxy, 'selectionStart', { get, set, configurable: true })`:

- **`get`** returns the polyfill's cached canvas selection
  start/end/direction, refreshed on every `selection-changed`
  egress. Reads do not call into the canvas widget; the cache is
  the read source so a screen reader polling `selectionStart` 60
  times/sec does not wake the canvas event loop.
- **`set`** is forbidden — assignments throw `TypeError: cannot
  assign to selectionStart; use setSelectionRange(start, end)`
  (matching native `<input>` behaviour, which treats these as
  read-only-ish for direct assignment in most engines).

`setSelectionRange(start, end, direction = "none")` is intercepted
by replacing the method on the proxy *instance*:

```ts
proxy.setSelectionRange = (start: number, end: number,
                            direction?: "forward" | "backward" | "none") => {
  const [s, e] = clampToBuffer(start, end);
  const d = direction ?? "none";
  // Update the cache synchronously so a subsequent read sees the new value.
  cache.start = s; cache.end = e; cache.direction = d;
  // Forward to the canvas widget. The widget will fire
  // selection-changed, which the polyfill ignores (matches the cache
  // by sessionId so the round-trip is idempotent).
  widget.dispatch({ kind: "set-selection", start: s, end: e, direction: d, originSessionId: bumpSessionId() });
};
```

The synchronous cache update is the load-bearing detail of R2:
external callers expect `setSelectionRange(2, 4)` followed
immediately by `inputProxy.selectionStart` to return `2`, with no
microtask gap. The cache write happens before `dispatch` returns.

`clampToBuffer` clamps to `[0, proxy.value.length]` per native
`<input>` semantics; negative or out-of-range arguments are not
errors, they are clamped, matching WHATWG.

### Shadow-root `getSelection()` (R3)

`<jet-semantics>`'s shadow root exposes a `getSelection()` method.
While a canvas text widget is focused, it returns a jet-`Selection`
object — a plain object that quacks like a DOM `Selection` for the
methods external consumers actually use:

```ts
interface JetSelection {
  readonly anchorNode:    Element | null;  // always inputProxy or null
  readonly anchorOffset:  number;          // UTF-16, matches selectionStart
  readonly focusNode:     Element | null;  // always inputProxy or null
  readonly focusOffset:   number;          // UTF-16, matches selectionEnd
  readonly rangeCount:    0 | 1;
  readonly isCollapsed:   boolean;
  readonly type:          "None" | "Caret" | "Range";

  getRangeAt(index: 0): Range;             // synthesized Range, see below
  toString():            string;           // proxy.value.slice(start, end)
  setBaseAndExtent(anchorNode: Node, anchorOffset: number,
                   focusNode: Node, focusOffset: number): void;
  addRange(range: Range): void;
  removeAllRanges():     void;
  collapse(node: Node | null, offset?: number): void;
  collapseToStart():     void;
  collapseToEnd():       void;
  empty():               void;             // alias for removeAllRanges
}
```

`anchorNode` / `focusNode` are always the input proxy element when
the widget is focused (collapsed counts; `type === "Caret"`) and
`null` otherwise. Offsets respect `selectionDirection` (forward →
anchor=start, focus=end; backward → swapped).

`rangeCount` is `1` while focused, `0` otherwise. The Range returned
by `getRangeAt(0)` is constructed via `document.createRange()` +
`setStart(proxy, start)` / `setEnd(proxy, end)` — valid native
mutations because the proxy is a real element, with offsets
interpreted by jet as character offsets into `proxy.value` (the
WHATWG spec permits unbounded offsets for childless elements).
`range.toString()` is intercepted per-instance to return
`proxy.value.slice(start, end)`. `range.startOffset` / `endOffset`
return the canvas offsets directly. Selection-level `toString()`
returns the same substring without constructing a Range — fast path
for the `document.getSelection().toString()` clipboard pattern (R8).

### `document.getSelection()` focus-gated override (R3, R6)

`document.getSelection` is replaced (once, at jet runtime init) with
a wrapper:

```ts
const nativeGetSelection = document.getSelection.bind(document);
document.getSelection = (): Selection | JetSelection => {
  const active = document.activeElement;
  if (active && active === currentProxy()) {
    return jetSemanticsShadowRoot.getSelection();
  }
  return nativeGetSelection();
};
```

This is the **only** global patch the polyfill installs. The native
`document.getSelection` is preserved via `bind` and re-invoked
whenever the active element is not the proxy. `window.getSelection`
is the same function on every engine (it's an alias for
`document.getSelection` on the realm), so the override covers both.
`Selection.prototype` is untouched.

When the active element is a native `<textarea>`, a `<input>`, or a
`contenteditable`, the wrapper falls through to native and the page
behaves exactly as if jet were not loaded — R6's hard requirement.

### External `setBaseAndExtent` / `addRange` forwarding (R4)

`JetSelection.setBaseAndExtent(anchorNode, anchorOffset, focusNode, focusOffset)`:

- If `anchorNode === focusNode === currentProxy()`: forward to the
  canvas as `set-selection { start: min(a, f), end: max(a, f),
  direction: a <= f ? "forward" : "backward" }`. Same cache-write-
  then-dispatch pattern as `setSelectionRange`.
- If `anchorNode !== focusNode` (cross-widget or proxy-to-page):
  no-op. Cross-widget selection is **forbidden** — each canvas
  widget owns its own DOM mirror anchor (the proxy is per-focused-
  widget; only one proxy exists at a time per ADR-010 § at-most-one).
  Selections that span two widgets cannot be expressed in canvas
  coordinates and would corrupt the buffer-offset cache.
- If either node is not the proxy (e.g. a page element handed in by
  an extension): no-op. The canvas widget cannot resolve foreign
  nodes to buffer offsets.

`addRange(range)`:

- If `range.startContainer === range.endContainer === currentProxy()`:
  forward as `set-selection { start: range.startOffset, end:
  range.endOffset, direction: "forward" }`.
- Otherwise: no-op (same reasoning).

`removeAllRanges()` / `empty()` collapse the canvas selection to
the current caret position (`set-selection { start: cache.start,
end: cache.start, direction: "none" }`).

`collapse(node, offset)` is forwarded only when `node === proxy`;
otherwise no-op.

The no-op policy matches WHATWG Selection's "drop invalid Range"
semantics — extensions that try to drive jet via a Range over their
own DOM nodes silently fail rather than corrupting the canvas, and
the page's other selection-aware code is unaffected.

### `selectionchange` dispatch policy (R5)

The polyfill is the single emitter of `selectionchange` events for
proxy-anchored selection. The dispatch rule:

```
on canvas selection-changed { start, end, direction, originSessionId }:
  if (originSessionId !== lastForwardedSessionId) {
    // Real canvas-side mutation (user click-drag on canvas, arrow
    // key, IME caret move) — not an echo of our own set-selection.
    cache.start = start; cache.end = end; cache.direction = direction;
    scheduleSelectionChange();
  }
  // Else: this is the echo of a setSelectionRange/setBaseAndExtent
  // we just dispatched; cache is already up to date; do not re-fire.

scheduleSelectionChange():
  if (selectionChangePending) return;
  selectionChangePending = true;
  requestAnimationFrame(() => {
    selectionChangePending = false;
    proxy.dispatchEvent(new Event("selectionchange", { bubbles: false }));
    document.dispatchEvent(new Event("selectionchange", { bubbles: false }));
  });
```

The session-id round-trip suppression (R2's cache write tagged with
a fresh sessionId, echoed back on the canvas's `selection-changed`)
is what guarantees "at most once per animation frame" — without it,
every `setSelectionRange` would fire two `selectionchange`s (the
synchronous cache-write echo and the canvas's confirmation).

Both `document` and the proxy element are notified per the
WHATWG `selectionchange` algorithm: the spec dispatches on document
only, but in practice extensions listen on the proxy too (because
they treat it like an `<input>`), so the polyfill fires both.

### IME-composition-aware Range reads (R9)

While a composition session is open (between ADR-013's
`composition-start` and `composition-end`), the polyfill's cached
`(start, end)` is overlaid with the IME's target-clause range:

```
on composition-start { sessionId }:
  preCompositionCache = { ...cache };
  imeRange = null;

on composition-update { sessionId, preeditText, caretOffset }:
  // The target-clause range is the (caretOffset, caretOffset)
  // collapsed caret within the preedit, unless the engine supplied
  // multi-segment metadata (deferred per ADR-013 § Open Q5).
  imeRange = { start: compositionStartOffset + caretOffset,
               end:   compositionStartOffset + caretOffset,
               direction: "none" };
  scheduleSelectionChange();

on composition-end:
  imeRange = null;
  // cache is re-synced from the canvas's selection-changed
  // (which fires after commit/cancel buffer mutation).
```

`getSelection()` reads check `imeRange` first; if non-null, that
range is returned in place of `cache`. The native `<textarea>`
behaviour an IME consumer expects is "during composition, Selection
points into the preedit"; this rule replicates it.

The pre-composition selection is preserved in
`preCompositionCache` and restored if `composition-end` fires with
`disposition: "cancel"`: the cache is reset to
`preCompositionCache` and a final `scheduleSelectionChange()` is
called so AT sees the restored selection.

### UTF-16 ↔ canvas index conversion at the boundary (R7)

Every value crossing the bridge passes through
`canvasToUtf16(canvasOffset)` (egress) and
`utf16ToCanvas(utf16Offset)` (ingress). Today both are identity
functions because the canvas backend indexes in UTF-16 too, but
the conversion call sites exist so a future codepoint or
grapheme-cluster backend can swap the implementations without
hunting through call sites.

The conversion uses the canvas widget's buffer as the truth source
— `proxy.value` is *not* used for the conversion (it can be stale
during composition per ADR-013 § R6).

### At-most-one proxy, at-most-one jet selection

ADR-010 guarantees at most one proxy exists at a time. The cache
lives on the proxy instance and dies with it. On focus move A → B:
old proxy tears down (cache discarded), `document.getSelection()`
briefly falls through to native, new proxy mounts and primes its
cache from the new widget's `selection-changed`, one
`selectionchange` fires one frame later. External consumers see one
selection at a time, addressed to whichever proxy is current.

## Acceptance tests (R8, R9)

Both tests run headless in CI against a synthetic canvas widget
fixture; no real IME or OS is required.

### R8 — round-trip through `setSelectionRange`

Setup: mount a canvas text widget with buffer `"hello"` and focus
it; programmatically set canvas selection to `(0, 5, "forward")`.

Assertions:

1. `document.getSelection().toString() === "hello"`.
2. `inputProxy.selectionStart === 0 && inputProxy.selectionEnd === 5`.
3. `document.getSelection().rangeCount === 1`.
4. `document.getSelection().getRangeAt(0).startOffset === 0`.
5. `document.getSelection().getRangeAt(0).endOffset === 5`.
6. Call `inputProxy.setSelectionRange(2, 4)`.
7. `inputProxy.selectionStart === 2 && inputProxy.selectionEnd === 4` *synchronously*.
8. The canvas widget's selection is `(2, 4, "none")` *synchronously*.
9. Exactly one `selectionchange` event fires on `document` during the
   next animation frame (not two).

### R9 — IME target-clause read during composition

Setup: mount a canvas text widget with buffer `""`; focus it;
dispatch ADR-013's composition sequence for Japanese "あい":

1. `composition-start { sessionId: 1 }`
2. `composition-update { sessionId: 1, preeditText: "あ", caretOffset: 1 }`
3. `composition-update { sessionId: 1, preeditText: "あい", caretOffset: 2 }`

Assertions during the open session (after event 3):

1. `document.getSelection().rangeCount === 1`.
2. `document.getSelection().getRangeAt(0).startOffset === 2`.
3. `document.getSelection().getRangeAt(0).endOffset === 2`.
4. The canvas widget's pre-composition selection (`(0, 0, "none")`)
   is **not** returned — the IME target-clause range is.

Then dispatch `composition-end { sessionId: 1, committedText: "あい",
disposition: "commit" }` and assert:

5. `document.getSelection().getRangeAt(0).startOffset === 2`
   (post-commit caret-at-end-of-inserted-text).
6. `document.getSelection().toString() === ""` (collapsed caret,
   not a selection over the just-committed text).

Both tests live under
`projects/jet/data/parity/fixtures/ime-selection-api/` and feed the
parity gate (#2144) on a `diff_kind: dom_selection_diff` channel —
each frame's `(anchorNode, anchorOffset, focusNode, focusOffset,
toString())` tuple is compared against a Flutter reference where
available (Flutter's `TextInputConnection` exposes equivalent
state), with a separate channel for IME-composition frames against
the WPT `selection-api` subset.

## Consequences

- **External Selection consumers see canvas selections correctly.**
  Screen readers, extensions (Grammarly, 1Password, LanguageTool),
  and the platform context menu all read `getSelection()` and now
  get the canvas widget's selection in DOM-native form. Copy via
  `document.execCommand('copy')` or
  `navigator.clipboard.writeText(document.getSelection().toString())`
  works without canvas-specific code paths.
- **`Selection.prototype` is untouched.** Native inputs,
  textareas, and contenteditables on the same page retain full
  native behaviour. The override is exactly one assignment to
  `document.getSelection`, focus-gated.
- **Cross-widget selection is impossible.** A drag across two canvas
  widgets selects in one (whichever was hit first); the drag does
  not extend into the second. Hard restriction of the per-widget
  proxy model; multi-widget selection is deferred.
- **IME consumers anchor correctly.** During CJK composition,
  candidate windows anchor to the target-clause range per R9,
  matching native `<textarea>`. ADR-013's `sessionId` is reused as
  the overlay key.
- **`selectionchange` is rate-limited.** One event per animation
  frame regardless of how many caret moves; AT polling sees at
  most 60 Hz; reads are O(1) against the cache.
- **Range objects are partly synthetic.** `range.toString()` and
  `startOffset` / `endOffset` work as expected; `cloneRange`,
  `compareBoundaryPoints`, `getBoundingClientRect` are not
  intercepted and return native (mostly degenerate) results. The
  polyfill scope is `setBaseAndExtent` / `getRangeAt(0).{startOffset,
  endOffset, toString}` only.
- **The polyfill is single-range.** `rangeCount > 1` (Firefox
  Ctrl-click) is not implemented; `addRange` after the first is a
  no-op.
- **Cache invariants are session-id-gated.** Echo suppression on
  `selection-changed` depends on `originSessionId` round-tripping;
  a fixture asserts this for every `set-selection` ingress.

## Alternatives considered

- **Patch `Selection.prototype` globally.** Rejected: breaks every
  native input on the page (R6 forbids).
- **Make the proxy a `<span contenteditable="false">` with a text
  child so native Ranges work.** Rejected: contenteditable's own
  selection state machine interferes with the canvas; native
  `<input>` is the cleaner substrate (ADR-010 already chose it).
- **Return native `Selection` from `getSelection()` with the proxy's
  real internal selection mirroring the canvas.** Tried: relies on
  the spec's "input selection sometimes exposed on Selection" rule,
  which is engine-divergent (Chromium yes, Safari no) and
  `Selection.toString()` returns empty for input selections
  everywhere. The synthesized `JetSelection` is required.
- **Forward every `getSelection()` read synchronously to the canvas.**
  Rejected: 60 Hz AT polling would wake the canvas event loop for an
  unchanged value. Cache + invalidation is the right shape.
- **Synchronous `selectionchange` (no rAF coalesce).** Rejected:
  drag-select fires sub-frame and would re-enter listeners faster
  than they can process. Native browsers themselves batch to next
  render frame.
- **Allow cross-widget selection via shadow-root anchor.** Rejected:
  shadow root is a `DocumentFragment`; Range offsets would index its
  element list, meaningless to clipboard. Single-widget is right.
- **Use the WICG `EditContext` selection surface.** Same answer as
  ADR-010 / ADR-013: Chromium-only in 2026-05; tracked as a follow-up.

## Open questions

1. **Range methods beyond `startOffset` / `endOffset` / `toString`.**
   Extensions that call `range.getBoundingClientRect()` expect a
   canvas-pixel-accurate rect, but the native method returns the
   proxy element's rect (off-screen, 1×1px). Whether to intercept
   `getBoundingClientRect` on synthesized ranges to call into the
   canvas widget's glyph-position table is open; deferred until an
   in-the-wild extension is observed to need it.
2. **`Selection.modify(alter, direction, granularity)`.** The
   platform's "Look Up" gesture and macOS's word-boundary cursor
   movement use `modify`. The polyfill currently does not implement
   it (calls fall through to native, which does nothing on a Range
   over a childless element). Whether to implement it depends on
   #2173's CJK manual test results — if word-boundary movement
   surfaces in the corpus, it ships here as a follow-up.
3. **Multi-range selection.** Firefox supports `rangeCount > 1` via
   Ctrl-click. Jet does not. Whether to model multi-range as
   multiple `set-selection` events on the canvas (each tagged with
   a `rangeIndex`) or to keep single-range as a hard limit is open.
4. **Selection direction propagation on cross-widget focus.** When
   focus moves from widget A (selection direction "backward") to
   widget B (no prior selection), the polyfill primes B's selection
   to `(0, 0, "none")`. Whether the direction should carry over
   (matching browser behaviour where Tab between inputs preserves
   "selection direction" for keyboard nav) is open and depends on
   ADR-003 (tab order) follow-ups.
5. **Selection within a composition's pre-edit, not just the
   target clause.** Some IMEs (Pinyin with tone selection) expose
   a *region* selection inside the preedit, not a collapsed caret.
   The R9 rule currently collapses to caretOffset; a future
   `composition-update.targetClauseRange?: { start, end }` field on
   ADR-013 would let R9 honour the region. Tracked under
   ADR-013 § Open Q5.

## References

- #2174 — this slice (Selection API parity).
- #2138 — parent epic: text input + IME on canvas.
- #2171 / ADR-010 — IME hidden input proxy (DOM anchor for the
  Selection bridge).
- #2172 / ADR-013 — IME composition event protocol (composition
  session boundaries that gate R9).
- #2152 / `jet-semantics-shadow-subtree.md` — host of the
  shadow-scoped `getSelection()` entrypoint.
- #2173 / ADR-014 — CJK IME manual test matrix (consumer of this
  protocol).
- #2175 — browser-quirks corpus (engine-divergent Selection
  behaviour traces).
- #2178 — Selection-API-based AT virtual-cursor (downstream
  consumer).
- #2139 / `dom-reference-runner.md` — WPT `selection-api` subset
  the R8 / R9 fixtures replay.
- #2144 — parity gate (the `dom_selection_diff` channel R8 / R9
  ride on).
- WHATWG Selection API specification — normative reference for
  `setBaseAndExtent` / `addRange` / `selectionchange`.
- MDN: `Selection`, `Range`, `selectionchange`, `Document.getSelection`.
- WICG `EditContext` proposal — future migration target.
