# ADR-031: HTML5 drag-and-drop bridge — per-widget proxy `<div draggable="true">` in `<jet-semantics>`, OS-owned drag operation, `setDragImage` from offscreen canvas snapshot

| Field | Value |
|-------|-------|
| Issue | #2169 |
| Parent epic | #2137 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | HTML5 drag-and-drop is bridged into the canvas via a per-widget proxy `<div data-jet-dnd-proxy draggable="true">` mounted inside the `<jet-semantics>` shadow subtree from #2152, positioned 1:1 over each canvas-side drag source or drop target's bounding rect (same overlay rule as ADR-013/#2172 IME proxy — visible to the OS hit-tester, transparent to the user). On `dragstart` the bridge calls the canvas widget's `serializeForDrag()` → populates `event.dataTransfer` via `setData(type, value)` for each declared MIME type (`text/plain`, `application/json`, custom), calls `setDragImage(rasterizedNode, offsetX, offsetY)` where `rasterizedNode` is an `ImageBitmap` produced by rendering the source widget to an offscreen `<canvas>` and calling `getImageData → createImageBitmap`, and writes `effectAllowed` from the widget's declared drag-effects (default `copyMove` for list reorder, `copy` for file source). Per-frame `drag` event repositions the source proxy under the OS cursor so `dragenter`/`dragover`/`dragleave` fire on the correct drop-target proxy as the OS pointer crosses zones. Each canvas-side drop zone has its own proxy `<div>` (full-rect overlay, `pointer-events: auto`, `opacity: 0`); on `dragover` the bridge calls `event.preventDefault()` to signal acceptance per the W3C DnD contract and writes `event.dataTransfer.dropEffect` from the widget's `dropEffect` callback; on `drop` it reads `dataTransfer.getData(...)` (or `dataTransfer.files` for external file drags) and forwards to the canvas widget's `acceptDrop(payload, position)` with canvas-local coordinates resolved by the same hit-test path as #2164's pointer router. Cross-window / cross-app drops work for free — the OS owns the drag operation once `dragstart` fires; jet sees only the standard event stream. Inbound external file drops deliver `dragenter`/`drop` directly on the host `<canvas>` element when no proxy is under the cursor; the bridge hit-tests canvas-local coordinates to find the drop zone and synthesizes the same `acceptDrop` dispatch. Opt-out via the `data-jet-pointer-channel="off"` parity escape from #2164: when disabled, no proxies are mounted and the bridge ignores all native DnD events. Acceptance gate: a WPT-style fixture renders a canvas-side sortable list, replays the DOM reference oracle's MUI Sortable recording from #2151's corpus, and asserts byte-equivalent `dataTransfer` payload + drop position on each item move. |

## Context

HTML5 drag-and-drop (`dragstart`, `drag`, `dragend`,
`dragenter`, `dragover`, `dragleave`, `drop` + the
`DataTransfer` object) is one of the most peculiar
event surfaces in the platform: the browser ties the
entire drag operation to a *real* DOM element bearing
the `draggable="true"` attribute, and once the user
crosses the system drag threshold the OS — not the
page — owns the operation. Cross-window drops, cross-
application drops, drops onto external apps (Finder,
Slack, VS Code), and inbound drops *from* those
external sources are all served by the OS drag service,
reachable only via the native event stream.

A canvas renderer like jet has no DOM elements for
its internal widgets — every "draggable item" the
user sees is a paint node, not an element. The
browser's drag machinery cannot see canvas pixels and
cannot start a drag on them. Without intervention, a
user mousing down on a draggable canvas item and
moving the cursor does *nothing*: no `dragstart`
fires, no `DataTransfer` is created, no drop targets
receive events. We cannot synthesize DnD purely
inside the canvas — synthesized `DragEvent`s do not
participate in the OS drag service and do not
propagate to external windows.

The fix is the bridge pattern we already use for IME
(ADR-013/#2172), text-input (ADR-016/#2174), and
focus management (ADR-020/#2156): mount a real DOM
proxy element that the browser *can* hit-test, route
the native events through it, and adapt the inputs
and outputs to the canvas-side widget model. This
ADR is the HTML5 DnD adaptation of that pattern.

The bridge has two cooperating halves:

1. **Drag-source proxies.** Each canvas widget that
   declares a `draggable` capability gets its own
   per-widget proxy `<div>` overlaid on its bounding
   rect. The proxy is the element the browser sees
   the mousedown on; the proxy is what `dragstart`
   fires on; the proxy is what the OS tracks as the
   drag source. On `dragstart` the bridge populates
   `DataTransfer` from the widget and the OS takes
   over.

2. **Drop-target proxies.** Each canvas widget that
   declares a `droppable` capability gets its own
   per-widget proxy `<div>` overlaid on its bounding
   rect. The proxy is what `dragenter`/`dragover`/
   `dragleave`/`drop` fire on as the cursor moves
   over the canvas-side drop zone. On `dragover` the
   bridge calls `preventDefault()` (the standard
   accept-this-drop signal per W3C DnD); on `drop`
   it forwards the payload to the widget.

The `<jet-semantics>` shadow subtree from #2152 is
the host of both: it already aggregates the per-
widget DOM proxies (text inputs, focus rings, AX
nodes); the DnD proxies are one more proxy family in
that subtree, mounted/unmounted as canvas widgets
mount/unmount, repositioned on layout changes and on
the OS drag pointer.

The drag-image (the visual the OS shows under the
cursor during the drag) is set via `DataTransfer.
setDragImage(image, offsetX, offsetY)`. The OS
captures the image *synchronously* in the `dragstart`
handler, so we must produce it eagerly: the bridge
renders the source widget to an offscreen `<canvas>`,
calls `getImageData → createImageBitmap`, and passes
the `ImageBitmap` to `setDragImage`. This gives the
OS a pixel-perfect copy of the canvas-rendered
widget as the drag preview — matching what the user
sees on screen, not a generic ghost.

External drag-in (file from desktop, text from
another tab, image from another window) lands as a
native `dragenter`/`drop` event on the host
`<canvas>` element itself (no proxy is under the
cursor — the cursor is over the canvas surface, and
the proxies cover only declared widgets). The bridge
listens on the canvas, hit-tests the canvas-local
coordinates to find the drop zone, and synthesizes
the same `acceptDrop` dispatch as for canvas-internal
drops. For file drops, `dataTransfer.files` is
forwarded as-is — the widget receives a `FileList`
identical to what it would see in a native form.

## Decision

### Proxy mount surface — per-widget, in `<jet-semantics>`

For every canvas widget that declares either a
`draggable` capability or a `droppable` capability,
the bridge mounts a dedicated proxy `<div>` inside
the `<jet-semantics>` shadow subtree. The proxies
are siblings of the widget's other DOM proxies
(focus, AX, IME) and follow the same lifecycle:

```html
<jet-semantics>
  <div data-jet-widget-id="list-item-0">
    <!-- existing proxies from #2152, #2156, #2172 -->
    <div data-jet-dnd-proxy="source"
         data-jet-dnd-widget="list-item-0"
         draggable="true"
         style="position: absolute;
                left: 120px; top: 240px;
                width: 200px; height: 32px;
                pointer-events: auto;
                opacity: 0;">
    </div>
    <div data-jet-dnd-proxy="target"
         data-jet-dnd-widget="list-item-0"
         style="position: absolute;
                left: 120px; top: 240px;
                width: 200px; height: 32px;
                pointer-events: auto;
                opacity: 0;">
    </div>
  </div>
  ...
</jet-semantics>
```

A single widget can be both a source and a target
(typical for list reorder): the bridge mounts both
proxies overlaid on the same rect. The source proxy
carries `draggable="true"`; the target proxy does
not (drop targets do not need the attribute — they
only need to live where the cursor will be when the
OS dispatches `dragover`/`drop`).

`opacity: 0` and `pointer-events: auto` together
give the W3C "visible to hit-test, invisible to
sight" trick. The proxy receives mousedown,
dragstart, drop, etc., while the user sees the
canvas pixels underneath. Same trick as ADR-013's
IME proxy.

### Canvas widget contract — `serializeForDrag` and `acceptDrop`

Canvas widgets opt into DnD by implementing two
methods on their widget interface:

```ts
interface DraggableWidget {
  serializeForDrag(): {
    payload: Record<string, string>;     // MIME → data
    effectAllowed: DataTransfer["effectAllowed"];
    dragImage: { offsetX: number; offsetY: number };
  };
}

interface DroppableWidget {
  acceptedTypes: string[];               // MIME whitelist
  dropEffect(event: {
    types: string[];
    position: { x: number; y: number };  // canvas-local
  }): DataTransfer["dropEffect"];        // "copy" | "move" | "link" | "none"
  acceptDrop(
    payload: Record<string, string>,
    position: { x: number; y: number },
  ): void;
}
```

`serializeForDrag` returns the MIME-typed payload the
widget wants placed on the `DataTransfer` object. The
bridge calls `setData(type, value)` for each entry.
The drag-image offset is in widget-local coordinates
and tells the OS where the cursor should sit relative
to the rendered preview (typically the click point).

`dropEffect` is the per-`dragover` callback that
tells the OS whether this drop will be a copy, a
move, a link, or refused. The bridge writes the
returned value into `event.dataTransfer.dropEffect`
on every `dragover`; the OS updates the cursor
overlay accordingly (the +copy badge on macOS, the
move arrow on Windows).

`acceptDrop` is the per-`drop` callback that
receives the final payload and the canvas-local drop
position. The widget mutates its model — re-orders
the list, inserts the dropped file, etc.

### `setDragImage` — offscreen canvas snapshot

On `dragstart` the bridge produces the drag image
*synchronously* inside the event handler (the OS
captures the bitmap at that moment; later mutations
do not propagate):

```js
function buildDragImage(widget) {
  const rect = widget.getBoundingRect();
  const off = new OffscreenCanvas(rect.width, rect.height);
  const ctx = off.getContext("2d");
  widget.paintTo(ctx);                   // canvas widget paints itself
  const bitmap = off.transferToImageBitmap();
  return bitmap;
}

proxy.addEventListener("dragstart", (event) => {
  const widget = resolveWidgetByProxyId(proxy);
  const { payload, effectAllowed, dragImage } = widget.serializeForDrag();
  for (const [mime, value] of Object.entries(payload)) {
    event.dataTransfer.setData(mime, value);
  }
  event.dataTransfer.effectAllowed = effectAllowed;
  const bitmap = buildDragImage(widget);
  event.dataTransfer.setDragImage(bitmap, dragImage.offsetX, dragImage.offsetY);
  router.dispatchSynthesized("dragstart", widget, payload);
});
```

`OffscreenCanvas` + `transferToImageBitmap` is the
fast path: no `getImageData` round-trip, no Blob
encoding, no canvas-to-element conversion. The OS
gets a GPU-resident bitmap directly.

### Per-frame `drag` reposition

While the drag is in progress, the OS fires `drag`
on the source proxy at the display refresh rate.
The bridge uses each `drag` event to reposition the
source proxy under the current cursor:

```js
proxy.addEventListener("drag", (event) => {
  if (event.clientX === 0 && event.clientY === 0) return; // dragend-adjacent noise
  proxy.style.left = `${event.clientX - HIT_W / 2}px`;
  proxy.style.top = `${event.clientY - HIT_H / 2}px`;
});
```

This keeps the source proxy under the OS cursor so
`dragenter`/`dragover`/`dragleave` fire on the
correct drop-target proxy when the cursor crosses
into a drop zone. Without this reposition, the
source proxy stays at the widget's original location
and the OS hit-tests would miss the drop targets
that overlap intermediate canvas regions.

### Drop-target events — `preventDefault` to accept

The W3C DnD contract requires `dragover` to call
`preventDefault()` to signal "I am a valid drop
target"; without it the OS shows the no-drop cursor
and refuses the drop. Each drop-target proxy
listens:

```js
targetProxy.addEventListener("dragover", (event) => {
  const widget = resolveWidgetByProxyId(targetProxy);
  const position = canvasLocalFromClient(event.clientX, event.clientY);
  const effect = widget.dropEffect({
    types: Array.from(event.dataTransfer.types),
    position,
  });
  if (effect !== "none") {
    event.preventDefault();
    event.dataTransfer.dropEffect = effect;
  }
});

targetProxy.addEventListener("drop", (event) => {
  event.preventDefault();
  const widget = resolveWidgetByProxyId(targetProxy);
  const position = canvasLocalFromClient(event.clientX, event.clientY);
  const payload = {};
  for (const type of event.dataTransfer.types) {
    payload[type] = event.dataTransfer.getData(type);
  }
  if (event.dataTransfer.files.length > 0) {
    payload["__files__"] = event.dataTransfer.files;
  }
  widget.acceptDrop(payload, position);
});
```

The `__files__` slot is the bridge's convention for
forwarding `FileList` (which `getData` cannot
return). Widgets that accept file drops read
`payload["__files__"]` as a `FileList`.

### External drag-in — host-canvas listeners

When the drag originates outside the page (file from
desktop, image from another tab), the cursor enters
the page over the host `<canvas>` element directly
(no widget proxy is under it because the OS pointer
moved straight in from outside). The bridge
duplicates the drop-target handlers on the canvas
element itself:

```js
canvasElement.addEventListener("dragover", (event) => {
  const position = canvasLocalFromClient(event.clientX, event.clientY);
  const widget = hitTestDroppable(position);
  if (!widget) return;
  const effect = widget.dropEffect({
    types: Array.from(event.dataTransfer.types),
    position,
  });
  if (effect !== "none") {
    event.preventDefault();
    event.dataTransfer.dropEffect = effect;
  }
});
// ... same shape for `drop`
```

`hitTestDroppable` reuses #2164's pointer router
hit-test path. The synthesized dispatch on
`acceptDrop` is identical to the proxy path — the
widget cannot tell whether the drop came from
another canvas item or from the desktop.

### Lifecycle — mount, reposition, unmount

The bridge subscribes to the same widget-lifecycle
events as the other `<jet-semantics>` proxy
families:

- **Widget mounted with DnD capability** → mount
  source proxy and/or target proxy at the widget's
  current bounding rect.
- **Widget rect changed (layout / scroll)** → update
  proxy `left`/`top`/`width`/`height`.
- **Widget DnD capability removed** → unmount proxy.
- **Widget unmounted** → unmount proxy.
- **During active drag** → source proxy
  repositioned per-frame via `drag` event (see
  above); target proxies stay anchored to their
  widget rects.

### Opt-out — `data-jet-pointer-channel="off"`

Same escape hatch as #2164. When the host opts the
pointer channel off (typically for testing
fallbacks), the bridge:

- Mounts no proxies.
- Does not register canvas-element listeners.
- Lets native `dragover`/`drop` on the canvas bubble
  to the host page unmolested.

### Acceptance fixture

`projects/jet/data/parity/fixtures/dnd/sortable-list-v1`:
a canvas-side list of 10 items, each draggable and
droppable, modeled on the MUI Sortable
(`@mui/x-tree-view` / `dnd-kit`) demo captured in
#2151's corpus. The fixture replays the DOM oracle's
recorded gesture (drag item 3 onto item 7) and
asserts:

1. `serializeForDrag` is called once on dragstart.
2. The synthesized `DragEvent.dataTransfer.types` is
   `["application/x-jet-listitem", "text/plain"]`
   (or whatever the oracle recorded).
3. `dropEffect` returns `"move"` on each
   `dragover`.
4. `acceptDrop` is called exactly once with the
   recorded payload and a canvas-local position
   inside item 7's rect.
5. The post-drop list order matches the oracle's
   post-drop DOM order.

The fixture is part of the parity gate (#2144); it
gates green on byte-equivalent payload + position
versus the recorded oracle.

## Consequences

**Pro:** Full OS-DnD participation — cross-window,
cross-app, file-in, file-out — without abandoning
canvas rendering. The bridge is symmetric: canvas
widgets export DnD through proxies, external
sources import DnD through host-canvas listeners,
both paths converge on `acceptDrop`. Drag images
match the canvas-rendered widget pixel-for-pixel
(via `OffscreenCanvas` snapshot). Per-widget
proxies localize state — adding a new draggable
widget mounts one proxy, no central registry
mutation. The `<jet-semantics>` shadow root from
#2152 contains the DOM churn so the host page DOM
stays clean. Opt-out via #2164's pointer-channel
flag composes cleanly with the rest of the input
stack. The contract (`serializeForDrag`,
`acceptDrop`) is small, mirrors the React `react-
dnd` shape closely enough that MUI Sortable
adapters port with minimal change, and is fully
declarative — no imperative event wiring leaks into
widget code.

**Con:** Proxy count grows linearly with draggable/
droppable widget count; a list of 1000 sortable
items has ≥1000 proxies in the shadow subtree. We
mitigate by virtualization (only mount proxies for
widgets currently in viewport — same trick MUI
Sortable uses for its DOM), but virtualization is
the host's responsibility, not the bridge's, and
mis-configured hosts will pay the cost. `OffscreenCanvas` requires a working 2D context inside
the dragstart handler synchronously; if `paintTo`
is slow, the dragstart handler is slow, and the OS
may begin the drag with a stale or default drag
image. We document a budget of ≤4ms per `paintTo`
call. `setDragImage` works in all major engines but
has known quirks (Firefox renders the bitmap at
device-pixel ratio 1; Safari ignores the offset if
the bitmap is detached); we paper over these in the
bridge with engine-specific shims. External drag-in
hit-tests the canvas pointer position against the
*current* droppable layout — if widgets are
mid-animation when the cursor enters, the hit-test
sees the instantaneous layout, which may
flicker-mismatch the user's expectation. We
document this as a known edge.

**Risk:** The
proxy-repositioning-via-`drag`-event trick is the
load-bearing piece for canvas-internal drag
tracking; if `drag` events stop firing (browser
bug, OS-side drag pause), the source proxy
stagnates and `dragenter`/`dragover` on drop
targets break. We watch `dragover` on the host
canvas as a heartbeat fallback and reposition the
source proxy from those if `drag` is silent for
>100ms. Synthesized `DragEvent` objects (our
re-dispatched events on the canvas widget side) are
*not* native `DragEvent`s — they cannot be
`preventDefault`'d to refuse the OS drop. The
bridge owns the `preventDefault` call on the native
event before the synthesized event fires; widgets
that want to refuse mid-drop must set the
`dropEffect` to `"none"` on the `dragover` instead
of trying to cancel the `drop`. We document this
loudly in the widget contract.

**Cost:** ~1100 lines of JS in
`projects/jet/data/parity/runtime/dnd-bridge.ts` (proxy
manager, event handlers, hit-test glue, offscreen-
canvas snapshot helper, engine shims). ~250 lines
of widget interface plumbing in the canvas runtime.
~400 lines of fixture and acceptance harness
(`sortable-list-v1`). One new parity-gate channel
(`dnd-trace.json`) in the corpus snapshot from
ADR-030; the IME-trace channel pattern is the model,
recording the synthesized event sequence per fixture
for byte-equivalent diff.

## Alternatives considered

**Alternative A — Single global proxy under the
cursor (per the issue's R1).** One `<div data-jet-
dnd-proxy>` element repositioned to follow the
pointer on `pointermove`, with `draggable="true"`
toggled when the cursor is over a draggable widget.
*Rejected* for drop targets: a single proxy cannot
simultaneously be the source (under the cursor) and
the target (over the drop zone) — the OS
`dragenter`/`dragover` fire on whatever element is
under the cursor *now*, and a global proxy is
always under the cursor by construction, so drop
targets never get the events. The issue's R4 hand-
waves this by suggesting the bridge "hit-tests the
underlying canvas coordinates to resolve the jet
drop target" from `dragover` on the host canvas;
but that conflates source-proxy events with drop-
target events, and breaks the symmetry that lets
external drops use the same `acceptDrop` path. Per-
widget target proxies are O(N) more DOM nodes but
keep the event semantics straight and make
synthesis a no-op (event already fires on the
right target).

**Alternative B — Pure pointer-event synthesis, no
real DnD.** Translate the user's drag gesture into
canvas-internal pointer move/up sequences and let
the canvas runtime do its own drag tracking,
bypassing HTML5 DnD entirely. *Rejected*: cross-
window and cross-application drops require the OS
drag service, which is reachable only via
`dragstart` on a real DOM element. A pure pointer-
event simulation cannot drag a canvas item out to
Finder, Slack, or another browser tab — those use
cases are first-class for jet's target apps. We
keep this option as the fallback when the host opts
the pointer channel off, but it is not the default.

**Alternative C — Mount draggable elements as the
canvas's actual content, not proxies.** Render
draggable widgets as DOM elements (real `<div>`s
with the widget's visual) and skip the canvas
entirely for those widgets. *Rejected*: defeats the
canvas-first rendering model that the entire jet
parity track depends on, mixes two rendering
pipelines for visually adjacent content (with
inevitable z-index / blending bugs), and shifts the
contract from "canvas widgets with a DnD bridge" to
"DOM widgets when draggable". The hybrid surface
area is much larger than the bridge.

**Alternative D — Use the Pointer Events Drag
spec instead of HTML5 DnD.** Some platforms expose
pointer-event-based drag APIs (`pointerdown` +
`setPointerCapture`) that avoid the
`draggable`-attribute requirement. *Rejected*: this
is a different gesture (a non-OS drag), does not
participate in cross-window drops, and is the
mechanism #2167 (chained scroll) and other gesture
slices already cover for non-DnD pointer drags.
HTML5 DnD is specifically the OS-mediated drag and
must be implemented as such.

## Open questions

1. **Virtualization story for 1000+ draggable lists.**
   The bridge mounts one proxy per draggable
   widget. At 1000 items the shadow subtree has
   2000+ DnD proxies, plus the existing focus/AX/
   IME proxies — likely >10k DOM nodes. We rely on
   the host to virtualize (only mount widgets in
   viewport), but a stress fixture is needed to
   confirm the bridge does not contribute its own
   leak. Tracking under a follow-up.

2. **`setDragImage` engine quirks.** The spec is
   underspecified about DPR scaling, bitmap
   ownership after `setDragImage` returns, and
   offset handling when the cursor is outside the
   bitmap. Firefox and Safari diverge from Chromium
   in observable ways. The bridge ships per-engine
   shims; the long-term question is whether to
   upstream a clarification to the WHATWG HTML
   spec.

3. **Auto-scroll during drag.** The issue
   explicitly defers "pinned scrolling during drag
   (auto-scroll when cursor nears viewport edge)"
   to a follow-up. The bridge does not implement
   it; widgets that need auto-scroll wire it
   themselves via `dragover` callbacks on the
   scroll-container widget. We expect a future ADR
   to standardize this.

4. **Drag-cancel via Escape.** Pressing Escape
   during an active drag cancels the OS drag and
   fires `dragend` with `dataTransfer.dropEffect ==
   "none"`. The bridge forwards this to the widget
   as `acceptDrop(null, null)` by convention, but
   the widget contract should make this explicit
   (e.g. a separate `cancelDrop` method). Folding
   into the contract is a small cleanup for a
   follow-up.

5. **Touch DnD.** Mobile browsers do not fire
   HTML5 DnD events from touch gestures (they fire
   `touchstart`/`touchmove` instead). A polyfill
   layer (e.g. `mobile-drag-drop`) bridges touch to
   synthetic `DragEvent`s, but the synthesis loses
   OS participation (no cross-app drag on mobile
   anyway, so this is acceptable). The bridge
   accepts synthetic `DragEvent`s as a future
   extension; the issue does not cover touch DnD
   and we defer.

## References

- Issue #2169 — feat(jet): parity/pointer — HTML5
  drag-and-drop bridge (proxy element under cursor)
- Parent epic #2137 — pointer/gesture parity
- #2152 — `<jet-semantics>` shadow subtree (proxy
  host)
- #2156 — programmatic focus API (sibling proxy
  family pattern, ADR-020)
- #2164 — pointer router (hit-test + opt-out flag)
- #2167 — chained scroll (sibling non-DnD drag
  gesture)
- #2172 — IME composition (sibling proxy pattern,
  ADR-013)
- #2174 — text-input bridge (sibling proxy pattern,
  ADR-016)
- #2151 — MUI reference corpus (Sortable recording
  oracle, ADR-030)
- #2145 — Playwright launch flags (ADR-001)
- #2139 — DOM reference runner (parity oracle)
- #2144 — parity gate
- W3C HTML Living Standard, §6.5 "Drag and drop"
- WHATWG `DataTransfer` interface
- W3C Pointer Events Level 3
- MDN: `HTMLElement.draggable`, `DragEvent`,
  `DataTransfer.setDragImage`
- MUI Sortable / `dnd-kit` integration docs (the
  reference oracle for the acceptance fixture)
