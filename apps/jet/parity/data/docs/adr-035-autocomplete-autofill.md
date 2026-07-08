# ADR-035: autocomplete attr + password-manager integration

| Field | Value |
|-------|-------|
| Issue | #2177 |
| Parent epic | #2138 |
| Status | accepted |
| Date | 2026-05-16 |
| Decision | The canvas `TextField` widget exposes an `autoComplete: string \| null` prop (default `null`) that accepts any token (or space-separated combination) enumerated in the WHATWG HTML Standard's `autocomplete` attribute — `email`, `username`, `current-password`, `new-password`, `one-time-code`, `webauthn`, `name`, `given-name`, `family-name`, `tel`, `street-address`, `address-line1`-`3`, `postal-code`, `country`, `cc-number`, `cc-exp`, and ~40 other tokens. The value is mirrored, one-directionally, onto the hidden `<input>` proxy from ADR-010 (#2171) via `proxy.setAttribute("autocomplete", …)` at mount time (before the proxy receives its first `.focus()` call so the autofill UI binds correctly on the first paint) and again on every mid-session prop change inside the same RAF callback as the canvas re-render — riding the propagation pipeline ADR-032 (#2176) built for `inputmode` / `enterkeyhint`. A new `<JetForm>` canvas-tree parent emits a hidden `<form>` wrapper inside the `<jet-semantics>` shadow root from #2152, so sibling proxies (e.g. one `username` + one `current-password`) share the same `<form>` parent and password managers detect them as a paired credential structure; widgets not wrapped in `<JetForm>` mount stand-alone proxies and fall back to per-field manual fill. When the browser autofill or a password-manager extension writes back via `proxy.value = "…"; dispatchEvent(new InputEvent("input", { inputType: "insertReplacementText" \| "insertFromPaste", data: "…" }))`, ADR-013 / #2172's composition handler routes the event through an explicit autofill carve-out: outside an open composition session the event is forwarded to the canvas widget as a non-composition `InputEvent` and the value mirrors to the widget's text buffer; inside an open composition the in-progress session is closed with a synthesised `composition-end { disposition: "cancel" }` before the autofill `InputEvent` is delivered, so the canvas widget sees clean session boundaries. `autocomplete="webauthn"` is accepted on the proxy but jet does not call `navigator.credentials.get()` — the page owns that API call. A `data-jet-autofill="off"` opt-out on the canvas host disables propagation entirely (no `autocomplete` attr, no `<form>` wrapper) for canvas widgets that intentionally suppress autofill (PCI-scope payment forms). React prop alias: `autoComplete` (matches React's existing convention). Validation: a unit test asserts the proxy carries `autocomplete="email"` after mount and after prop change; a second unit test synthesises an autofill `InputEvent` and asserts the value reaches the canvas widget's buffer and that the widget's `change` event fires; a third E2E test mounts a `<JetForm>` with an email + current-password child pair and asserts the `<form>` wrapper exists with both proxies as children; a manual SR-matrix row (ADR-029, #2162) verifies 1Password / Bitwarden / Chrome autofill on the MUI login-form fixture. |

## Context

Browsers and password managers do not infer the *meaning* of a form
field from its position, its placeholder, or its surrounding text.
They read the `autocomplete` attribute on the `<input>` element, which
the WHATWG HTML Standard defines as a closed enum of ~40 tokens plus
the sentinels `on` and `off`. A `<input autocomplete="email">` lights
up the browser's email autofill dropdown; a `<input
type="password" autocomplete="current-password">` is visible to every
password manager extension (1Password, Bitwarden, LastPass, Dashlane,
Apple Passwords, Chrome's built-in manager) as a sign-in field; a
`<input autocomplete="new-password">` tells the manager "this is the
*new* password on a sign-up form — *save* it, do not *fill* it". Get
the attribute wrong and the user sees no autofill suggestions, has
to manually copy-paste from their vault every time, and (worse) the
password manager never offers to save the credential at all because
it never recognised the form.

For a real DOM page this is one attribute per `<input>`. For a
canvas-rendered surface like jet the work is non-trivial: the host
`<canvas>` has no `autocomplete` attribute; the autofill UI binds
to **whatever DOM element is currently focused**, which on jet is
the hidden `<input>` proxy from ADR-010 (#2171). The propagation
pipeline that ADR-010 set up for IME composition events, and that
ADR-032 (#2176) extended for `inputmode` / `enterkeyhint`, is the
same pipeline this ADR extends for `autocomplete` — a third
attribute on an already-paved road.

The autofill surface adds three complications that the mobile-keyboard
surface (ADR-032) did not face.

**First, autofill writes back.** When the user accepts a suggestion
from the autofill dropdown — or when a password manager extension
injects a button into the focused proxy and the user clicks "fill"
— the browser or extension sets `inputProxy.value = "…"` directly
and dispatches an `input` event with `inputType:
"insertReplacementText"` (or `"insertFromPaste"` for clipboard-based
fillers). ADR-013 / #2172's composition protocol currently
`preventDefault`-routes every `input` event on the proxy, because
inside a CJK composition the browser fires synthetic `input` events
on every Pinyin keystroke and the canvas widget must not see them as
direct text insertions — they belong to the composition stream.
Autofill events look identical from a JavaScript perspective; without
an explicit carve-out, jet would swallow the autofilled value and
the canvas widget would stay empty.

**Second, password managers fingerprint form structure**, not
individual fields. 1Password's heuristic is: find a focused
`autocomplete="current-password"` input, walk up to the nearest
`<form>` ancestor, find a sibling `autocomplete="username"` /
`"email"` / unattributed text input, treat the pair as a sign-in
form, fill both. The same logic underlies Chrome's built-in
autofill, Bitwarden's inline UI, and Apple Passwords on Safari.
Without a `<form>` ancestor the managers fall back to per-field
manual fill — the user has to click the extension icon and pick
the credential for each field separately. For jet's canvas widgets
to feel like real form fields, the proxies must mount inside a
hidden `<form>` wrapper that groups them per the canvas widget
tree's logical form structure.

**Third, PassKey / WebAuthn participation** is declared on the
proxy via `autocomplete="webauthn"` (or
`autocomplete="username webauthn"` for a paired username field) AND
the page must call `navigator.credentials.get({ mediation:
"conditional" })`. The attribute alone is not sufficient: the API
call is what tells the browser to surface passkey suggestions in
the autofill dropdown. Jet declares its participation (sets the
attribute on the proxy) but cannot make the API call on the page's
behalf — that's an application-level concern.

A fourth, smaller complication is the `<jet-semantics>` shadow root
from #2152. That root preserves DOM source order so screen readers
walk the proxies in the same order the canvas paints them. Password
managers also rely on DOM order (proximity heuristics: a
`current-password` two siblings after a `username` is paired; one
five siblings away is not). The same source-order invariant covers
both — no extra work, just a noted dependency.

Issue #2177 scopes this surface narrowly: which `autocomplete`
tokens are accepted (the entire HTML-spec closed enum, propagated
verbatim — jet does not curate a subset), the proxy attribute
writeback path, the autofill `input` event carve-out in the
composition protocol, the optional `<form>` wrapper for `<JetForm>`
groups, the PassKey participation boundary, and the opt-out for
canvas widgets that suppress autofill.

## Decision

### Canvas widget prop surface

`TextField` accepts one new prop:

- `autoComplete?: string | null` — default `null`. Accepts any
  token (or space-separated combination) enumerated in the WHATWG
  HTML Standard's `autocomplete` attribute, including the
  sentinels `"on"` and `"off"`. Jet does *not* validate the token
  set — the spec evolves (e.g. `"webauthn"` was added in 2022);
  the value is propagated verbatim. Invalid tokens are a browser
  concern (browsers silently ignore unknown tokens).

The React JSX-prop alias is `autoComplete` (matching React's
existing camel-cased convention; React already aliases
`autocomplete` ↔ `autoComplete` internally). When `null`, the
proxy carries no `autocomplete` attribute at all — distinct from
`"off"`, which actively tells the browser to suppress autofill.

### Mount-time propagation

When ADR-010's proxy element is created and inserted into the DOM
for a `TextField` whose `autoComplete` prop is non-`null`, jet
writes `proxy.setAttribute("autocomplete", value)` **before** the
proxy receives its first `.focus()` call. Browser autofill and
password-manager extensions inspect the attribute at focus time
(and on DOMNodeInserted observers) to decide whether to bind the
autofill dropdown / inject their button. Writing after focus risks
a frame where the proxy is bound to the wrong (or absent) autofill
channel.

If `autoComplete` is `null`, no attribute is written — distinct
from ADR-032's behaviour, where defaults (`"text"` / `"enter"`) are
written explicitly. Reason: the HTML spec gives semantically
distinct meanings to "no attribute" (browser may infer from
heuristics, default behaviour), `"on"` (autofill allowed,
unconstrained), and `"off"` (autofill suppressed). Jet preserves
the three-way distinction.

### Mid-session updates

When `autoComplete` mutates mid-session, the update runs inside
jet's canvas re-render frame, after the proxy is confirmed alive
(ADR-010 lifecycle) and before any synthetic composition or
autofill event for the same frame. Unlike `inputmode` /
`enterkeyhint`, `autocomplete` does not require a blur+focus cycle
on any supported platform — autofill UIs re-evaluate the attribute
on mutation observers, not on focus alone. A single
`proxy.setAttribute("autocomplete", newValue)` (or
`proxy.removeAttribute("autocomplete")` when the new value is
`null`) is sufficient. The autofill dropdown, if open, closes on
the next event tick; this is the browser's documented behaviour
and matches the user's mental model ("I changed what kind of field
this is — the old suggestions no longer apply").

### Composition-window deferral

The autofill carve-out (§ next) distinguishes the *event-routing*
deferral from the *attribute-write* deferral. The latter is
unnecessary: writing `autocomplete` on a focused proxy mid-
composition does not commit the composition or dismiss the
candidate window on any supported platform. The attribute write
runs immediately.

### Autofill event carve-out

Per R2 + R3 in #2177, the composition protocol from ADR-013
(#2172) gains an explicit carve-out for autofill `input` events.
The carve-out is implemented inside the proxy's `input` event
listener, which currently distinguishes composition events
(routed to the IME stream) from direct-typing events (routed to
the canvas widget's `InputEvent` handler) via the
`isComposing` flag.

The carve-out adds a third branch:

- **Branch A — direct typing** (existing): `isComposing === false`,
  `inputType ∈ { "insertText", "deleteContent…", "historyUndo", … }`.
  Routed to the canvas widget as an `InputEvent`.
- **Branch B — composition stream** (existing): `isComposing
  === true`. Routed to the IME stream (composition events handle
  the visible buffer).
- **Branch C — autofill** (new): `inputType ∈ {
  "insertReplacementText", "insertFromPaste" }` AND the event's
  `data` field is non-empty AND the event was not preceded by a
  user keystroke in the same task. Routed to the canvas widget as
  a non-composition `InputEvent` regardless of `isComposing`. The
  `proxy.value` is read and mirrored to the canvas widget's text
  buffer; the widget's `change` event fires.

If Branch C fires while an IME composition is open (Branch B was
active), jet first synthesises a `composition-end { disposition:
"cancel" }` for the in-progress session (so the canvas widget sees
a clean session boundary and discards the half-typed Pinyin
candidate buffer), then delivers the autofill `InputEvent`. The
ordering is: cancel-end → autofill-input. Reversing the order
would leave the canvas widget thinking it has an open composition
session whose buffer is suddenly the autofilled value, which is
inconsistent with the composition-event contract from ADR-013.

The "was not preceded by a user keystroke" heuristic distinguishes
genuine autofill (no preceding `keydown`) from a paste keystroke
that the user explicitly performed (preceded by `keydown` of
Ctrl/Cmd+V — handled via Branch A's normal paste path through the
canvas widget's clipboard handler).

### `<JetForm>` wrapper

A new canvas-tree widget `<JetForm>` is introduced. It is a pure
grouping node — no visual rendering, no layout — whose only
side-effect is to instruct the proxy-mounting machinery to emit a
hidden `<form>` element inside `<jet-semantics>` (the shadow root
from #2152) and to mount the proxies of every `<TextField>`
descendant of the `<JetForm>` as children of that `<form>`. DOM
source order inside the `<form>` matches the visual canvas order,
per #2152.

`<JetForm>` accepts no props in this ADR — its only purpose is the
structural grouping. Future enhancements (form submission
semantics, `method` / `action` attributes) are deferred. The
`<form>` element jet emits has no `action` and no `method`; it
exists solely as a structural anchor for password-manager
detection heuristics. Submission semantics remain owned by the
canvas widget tree (the React/JSX consumer wires `onSubmit` to
canvas-side state — jet's `<form>` is invisible to the page's
event handling).

When no `<JetForm>` parent is declared in the canvas tree, each
proxy stands alone (mounts directly into `<jet-semantics>`).
Password managers fall back to per-field manual fill, which is the
graceful-degradation path. This is the documented default
behaviour, not an error condition.

### PassKey / WebAuthn participation boundary

`autocomplete="webauthn"` (or `"username webauthn"`) is accepted
on the prop surface and propagated verbatim to the proxy. Jet
does **not** call `navigator.credentials.get({ mediation:
"conditional" })`; the page does. The proxy attribute is
necessary but not sufficient for conditional-mediation passkey
autofill — the API call is what tells the browser to surface
passkey suggestions in the autofill dropdown.

This boundary is deliberate: the API call has page-wide effects
(it can pop a modal, it interacts with the page's authentication
state machine), and jet has no policy basis for invoking it on
the consumer's behalf. The consumer wires the API call into
their own auth flow; jet ensures the proxy declares
participation.

### Opt-out

A `data-jet-autofill="off"` attribute on the canvas host element
disables the entire propagation pipeline: the proxy carries no
`autocomplete` attribute regardless of `autoComplete` prop value,
and `<JetForm>` parents do not emit `<form>` wrappers (their
descendants' proxies mount directly into `<jet-semantics>` as if
the group did not exist). This is the recovery path for canvas
widgets that intentionally suppress autofill — the canonical case
is a payment-card-entry form under PCI scope whose compliance
profile prohibits browser autofill of card numbers.

The opt-out is per-canvas-host, not per-widget. Reason: PCI-scope
forms typically occupy a dedicated route / page (a checkout
panel), which maps cleanly to a dedicated canvas host. Per-widget
opt-out is a complication we are not paying for in this ADR.

### Test surface

Three test layers, all runnable in headless CI, plus one manual
matrix row:

1. **Unit (Vitest, jsdom)** — proxy attribute mirroring:
   mount a `TextField` with `autoComplete: "email"`, assert
   `proxy.getAttribute("autocomplete") === "email"`. Mutate the
   prop to `"current-password"`, assert the proxy updates.
   Mutate to `null`, assert the attribute is removed (not just
   set to empty string). Set `data-jet-autofill="off"` on the
   host, assert the proxy carries no attribute regardless of the
   prop.
2. **Unit (Vitest, jsdom)** — autofill event roundtrip:
   mount a `TextField` with `autoComplete: "email"`. Synthesise
   `proxy.value = "user@example.com"; proxy.dispatchEvent(new
   InputEvent("input", { inputType: "insertReplacementText",
   data: "user@example.com", bubbles: true }))`. Assert the
   canvas widget's text buffer equals `"user@example.com"` and
   that the widget's `change` event fired exactly once.
3. **E2E (Playwright, headed Chromium with the 1Password extension
   installed in a sandboxed profile)** — `<JetForm>` structural
   grouping: render the MUI login-form fixture inside a
   `<JetForm>` wrapper with one `email` and one `current-
   password` child `TextField`. Assert
   `document.querySelector('jet-semantics form')` exists and has
   two `<input>` children with `autocomplete="email"` and
   `autocomplete="current-password"` respectively. (The
   1Password browser extension's actual autofill behaviour is the
   subject of the manual matrix row below — the E2E test
   verifies the structural shape, which is the contract.)
4. **Manual SR matrix row (ADR-029, #2162)**: on the MUI login-
   form fixture, with 1Password / Bitwarden / Chrome built-in
   autofill installed, verify the autofill suggestions surface on
   focus of the email proxy and that accepting a suggestion
   fills both fields. Run quarterly against the latest stable
   extension builds.

## Consequences

### Positive

- Canvas-rendered sign-in forms are visible to every password
  manager that follows the `<form autocomplete=…>` contract.
  Users of jet apps get the same one-tap fill experience they
  get on plain HTML pages — the most-requested missing UX in
  every canvas-app feedback channel to date.
- `autocomplete="one-time-code"` on a verification-code field
  triggers SMS autofill on iOS Safari and Android Chrome
  (suggestion bar above the keyboard pre-populated with the
  most recent OTP). Two-factor auth flows on jet apps gain
  parity with native form UX for free.
- The `<JetForm>` wrapper unblocks structural form detection.
  Even partial multi-field address autofill (browser-built-in
  on Chrome, Firefox) requires the full sibling set under a
  single `<form>` parent; without the wrapper, partial sets
  silently drop and the user sees nothing.
- Autofill `value` write-back via the carve-out keeps the canvas
  widget's buffer in sync with what the user sees in the proxy
  — without this the autofill UI would visibly fill the proxy
  and the canvas widget would render empty, the worst possible
  UX failure mode (looks broken, user retypes by hand).
- The propagation pipeline (mount-time `setAttribute`, RAF-frame
  flush, composition deferral) is now exercised by a third
  attribute family. Each future attribute (`autocapitalize`,
  `spellcheck`, anything else the HTML spec adds) rides the
  same path with proportionally less new code.
- PassKey participation is declared cheaply (one attribute
  token); the page owns the API call. Jet does not lock the
  consumer into a specific WebAuthn implementation.

### Negative

- The autofill carve-out in ADR-013's composition protocol is a
  genuine special case. The "was not preceded by a user
  keystroke" heuristic for distinguishing autofill from user-
  initiated paste is empirical — a future browser version could
  fire `keydown` before an autofill event in some scenario we
  haven't seen, and the carve-out would mis-classify. Mitigation:
  the unit test (test #2) is the regression boundary; if a new
  browser breaks the heuristic, the test fails on a real
  fixture before the failure ships to users.
- The mid-composition autofill deferral (synthesise
  `composition-end{cancel}` then deliver the autofill event)
  destroys any in-progress Pinyin / Zhuyin / Hangul candidate
  buffer. This is the right behaviour — autofill is a
  user-explicit action, the composition is implicitly abandoned
  — but it is *visible*: the candidate window closes, the
  half-typed jamo disappears. We accept this as the cost of
  consistent session boundaries.
- The `<JetForm>` wrapper is a new canvas-tree concept. Adding
  any new canvas-tree primitive has a cost (documentation,
  examples, future ADRs that depend on it). We are paying that
  cost specifically to make password-manager detection work;
  the wrapper has no other purpose in this ADR's scope.
- `<form>` wrapper emission inside `<jet-semantics>` is invisible
  to the consumer in devtools by default (shadow root). A
  consumer debugging a "why doesn't 1Password fill my form?"
  problem will need to know to expand the shadow root.
  Documented in the troubleshooting section of the jet IME
  spec; not a blocker but a noted onboarding cost.
- The PCI-scope opt-out (`data-jet-autofill="off"`) is host-
  scoped, not widget-scoped. A consumer with one PCI-scope
  field interleaved among non-PCI fields on the same canvas host
  cannot opt out per-field. We expect this to be vanishingly
  rare (PCI-scope fields cluster into dedicated checkout panels)
  but if a real case emerges we'll revisit with a per-widget
  opt-out follow-up.

### Neutral

- `autocomplete` is a no-op on canvas widgets that consumers
  never focus directly (display-only text). Setting the
  attribute costs one `setAttribute` per mount; irrelevant.
- The attribute does not interact with the screen-reader
  announcement pipeline (ADR-021, #2161): the proxy's
  `aria-label` / `aria-describedby` are scoped to AT
  announcements, the `autocomplete` token is scoped to
  autofill / password-manager detection. Two independent
  channels, both riding the proxy.
- The carve-out in ADR-013 / #2172 is additive — direct typing
  and composition events continue to flow through Branches A
  and B unchanged. Test coverage for those branches (the CJK
  matrix in ADR-014, #2173) is not affected.

## Alternatives considered

### A. Curate a subset of `autocomplete` tokens; reject unknowns

Validate the prop against a hard-coded enum (the ~40 tokens
documented in WHATWG today) and reject anything else with a
`console.warn` + fallback to `null`. Rejected because the spec
evolves — `webauthn` was added in 2022, future tokens (per the
ongoing WHATWG discussions on `cc-name-on-file`, payment-method
specificity, etc.) will appear before jet ships a new release.
Curating the list means lagging the spec; propagating verbatim
means the consumer can use any token the browser supports the
moment the browser supports it. Browsers silently ignore
unknown tokens — the failure mode of an invalid token is the
same as the failure mode of jet curating wrong (no autofill).

### B. Skip the `<form>` wrapper; rely on proximity heuristics alone

Password managers also use proximity (siblings within N DOM
nodes) when no `<form>` ancestor is present. Skipping the
wrapper would simplify the canvas-tree model — no `<JetForm>`
primitive. Rejected because the proximity heuristics are weaker
(Chrome's built-in autofill explicitly requires `<form>` for
address autofill; 1Password's structural detection is
significantly more reliable inside `<form>`). The cost of the
wrapper (one new canvas-tree node, one hidden DOM element per
group) is small; the benefit (full password-manager parity) is
disproportionately large.

### C. Inline the WebAuthn API call inside jet

Have jet call `navigator.credentials.get({ mediation:
"conditional" })` automatically when any proxy carries
`autocomplete="webauthn"`. Rejected because the API call has
page-wide side effects (it can prompt the user, it interacts
with the page's auth state machine), and jet has no policy basis
for deciding when to invoke it. The page knows whether it's on
a sign-in route, whether the user has already authenticated,
whether the auth flow is current; jet does not. The proxy
attribute is the right boundary.

### D. Synthesise a synthetic `change` event instead of mirroring `value`

Forward the autofill `InputEvent` to the canvas widget as just an
event, without touching the canvas widget's text buffer; rely on
the widget's normal `input`-event handler to read `proxy.value`
and update its buffer. Rejected because it puts the buffer-update
responsibility on every consumer widget — easy to forget, easy to
get wrong, and the canvas widget catalogue is large. The carve-
out reads `proxy.value` once and writes it to the buffer in one
place, which is the correct factoring.

### E. Use HTML `<form autocomplete="…">` instead of per-input attributes

The `<form>` element itself accepts an `autocomplete` attribute
(`"on"` / `"off"`). Use it as a coarse group-level toggle and
skip per-input attributes. Rejected: the group-level attribute
is a toggle, not a semantic descriptor. Password managers and
browser autofill still need per-input `autocomplete` tokens to
know *which* token corresponds to *which* field. The form-level
attribute is orthogonal (and is implicitly `"on"` when omitted).

## Open questions

- **`autocomplete-section-*` per-form scoping.** The HTML spec
  allows `autocomplete="section-shipping street-address"` to
  scope a token to a named section, useful when a checkout page
  has both shipping and billing address fields. #2177 defers
  this — the verbatim-propagation contract already supports it
  (the token is propagated as-is), but no jet fixture currently
  needs it. Tracking: add a fixture row when the MUI checkout
  fixture lands.
- **Password manager extension button anchoring.** 1Password and
  Bitwarden inject a small UI button into the focused proxy.
  Jet's proxy hiding (per ADR-010 / #2171) requires a non-zero
  paintable surface so the injected button has a bounding rect
  to anchor to. The 1×1px paintable surface is sufficient on
  Chrome / Firefox / Safari per manual verification, but a
  future extension version could anchor to a different
  reference point. Mitigation: the manual matrix row covers
  this; if a regression appears, ADR-010 grows a footnote.
- **`<JetForm>` submission semantics.** This ADR's `<form>`
  wrapper has no `action` / `method` / `onSubmit` — submission
  remains canvas-side state. A future canvas-tree enhancement
  could route the form's `submit` event back into the widget
  tree (e.g. for Enter-key form submission semantics). Deferred
  until a fixture demands it.
- **iOS Safari `one-time-code` cross-device parity.** SMS
  autofill on iOS Safari requires the input to be focused when
  the SMS arrives; the user has a ~30s window. Empirically the
  trigger works on jet's proxy because the proxy is focused
  whenever the canvas widget is focused. We have not yet
  verified the cross-device (macOS Safari receiving an SMS via
  Continuity) path; the manual matrix row should include it
  once a tester with the right device pair is rotated in.
- **Autofill writeback ordering vs canvas frame.** The carve-out
  delivers the autofill `InputEvent` synchronously inside the
  proxy's event listener. The canvas widget's buffer update is
  picked up by the next canvas re-render (next RAF). For a
  typical autofill ("user clicks fill" → buffer update → paint
  next frame) this is imperceptible. For pathological cases
  (autofill triggered by a programmatic script firing many
  events per frame) the canvas might render one or two stale
  frames. Acceptable; revisit if a real case appears.

## References

- Issue #2177 — parity/ime — autofill & password-manager integration (autocomplete attrs on proxy)
- Parent epic #2138 — IME parity
- ADR-010 (#2171) — hidden input proxy element (the DOM node `autocomplete` is written to)
- ADR-013 (#2172) — IME composition event protocol (autofill carve-out hooks into Branches A/B/C)
- ADR-032 (#2176) — `inputmode` / `enterkeyhint` propagation (sibling pipeline, this ADR rides the same mechanism)
- #2152 — `<jet-semantics>` shadow root (DOM source-order container for proxies; hosts the `<form>` wrapper)
- #2162 — SR / autofill manual matrix (ADR-029) — manual verification row for 1Password / Bitwarden / Chrome autofill
- WHATWG HTML Standard — `autocomplete` attribute (closed enum token vocabulary, normative)
- WebAuthn Level 2 — Conditional Mediation (external reference for `autocomplete="webauthn"` semantics)
- Spec `jet-ime-autofill-integration.md` — canvas → proxy attribute propagation contract
- Spec `jet-ime-autofill-event-carveout.md` — Branch C semantics in the composition protocol
- Spec `jet-ime-form-group-wrapper.md` — `<JetForm>` → hidden `<form>` emission
- Spec `jet-ime-attribute-propagation-pipeline.md` — shared with #2176
