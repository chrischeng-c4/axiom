# Cue multi-target dogfood — minimal cross-target UI slice

> Issue: #1240 — `enhancement(jet): dogfood Cue as the first multi-target
> Jet app`. Parent epic: #1237. Sibling specs: `tui-renderer.md`
> (#1241), `desktop-runtime.md` (#1242), `element-contract.md` (#1238),
> `target-profiles.md`, `build-targets.md` (#1239).

## Goal
<!-- type: overview lang: markdown -->

Pin the **renderer-neutral Cue UI surface** that #1241 Slice 4 (TUI
demo) and #1242 Slice 5 (desktop demo) both need to build against.
Today both demo slices reference "the Cue navigation/list/detail/log
workflow from the issue's acceptance criteria" without a concrete
contract — the contract is split across the per-target specs and
the existing `cue-tui-lifecycle-ux.md` (which is ratatui-flavored,
not target-neutral). This spec collapses that ambiguity into one
hand-written element tree + state shape + command/query/event
boundary that every target profile can render.

The spec is target-agnostic. Any divergence between targets lives
in the per-target specs (web profile inherits the canvas pipeline,
desktop adds OS lifecycle pulses + IPC bridge, TUI degrades visuals
per the TUI capability profile). The Cue app layer never branches
on target.

## Slices
<!-- type: overview lang: markdown -->

The dogfood ships in five hand-written slices:

- **Slice 1 (this doc) — multi-target UI contract.** Pin the
  minimal element tree (`AppShell { NavRail | DetailPane }`),
  the `CueState` shape, and the command/query/event boundary
  the app layer crosses to talk to the runtime. Pin the
  capability-degradation rules per renderer (web full, desktop
  full + OS shell, TUI degraded per `target-profiles.md`).
  Pin the slice plan for 2-5. No code lands.
- **Slice 2a (shipped) — `cue-app` data layer.** New
  `projects/cue/app/` package (TypeScript, `private: true`,
  no runtime deps) ships the pure-data layer that future TSX
  components consume:
  - `src/types.ts` — `IssueStatus` / `IssueSummary` /
    `IssueDetail` / `TimelineEvent` / `LogLevel` / `LogLine` /
    `CommandAck` / `RuntimeEvent` (tagged union under `kind`)
    / `BackendError` / `BackendErrorKind`. 1:1 mirror of the
    Rust types from Slice 3 — wire shape stays in lockstep.
  - `src/backend.ts` — `CueBackend` interface (`list_issues`,
    `get_issue`, `submit_command`, `subscribe_events` returning
    `AsyncIterable<RuntimeEvent>`) plus a `CueBackendError`
    Error subclass that carries the `BackendError.kind`
    discriminant for catch-site dispatch.
  - `src/state.ts` — `CueState` shape, `INITIAL_STATE`,
    `CueAction` union, pure `reduce(state, action)` reducer.
    Covers selection (drops stale results when the user moves
    on before fetch resolves), command pending state, log-tail
    cap (`LOG_TAIL_CAP = 500` with rolling slice), and the
    three `RuntimeEvent` variants (log → tail append,
    issue_updated → upsert + selected detail merge, timeline →
    selected detail event append). Switch is exhaustive — any
    new event variant fails typecheck loud-fast.
  - `src/index.ts` — public-surface barrel.
  - `tsconfig.json` strict (noFallthroughCasesInSwitch +
    noUnusedLocals + noImplicitOverride) targeting ES2020 with
    `jsx: react / jsxFactory: createElement` so the future TSX
    Slice 2b inherits the same compile profile (matches
    `e2e/jet/tsconfig.json`).
  - `tsc --noEmit` passes with zero warnings.

  Slice 2b lands `App.tsx` / `NavRail.tsx` / `DetailPane.tsx`
  on top of this data layer + a `useBackend()` hook + the
  `jet build --target web` end-to-end smoke that closes the
  Slice 2 acceptance criterion.
- **Slice 2b-1 (shipped) — JSX runtime + first TSX component.**
  `projects/cue/app/src/jsx.ts` ships the `createElement` factory
  + `Fragment` symbol + `Element` discriminated union (`intrinsic`
  / `text` / `component` / `fragment` / `empty`) that mirrors
  `jet_wasm::Element` so TSX in this package `tsc --noEmit`
  cleans against the same vocabulary the renderer resolves.
  Children pass through `flatten_children` which normalizes
  string / number leaves to `Text` nodes, drops the
  `null` / `undefined` / boolean noise that JSX conditionals
  emit, and recursively flattens arrays from `.map(...)` so
  downstream consumers see exactly one shape. JSX `IntrinsicElements`
  uses an open `[tag: string]: IntrinsicProps` index — typed
  per-tag prop maps land once the four renderer-contract gap
  issues from `cue-app-protocol-mapping.md` (Spinner / Modal +
  focus-trap / ActionRow / Markdown capability) are filed and
  resolved. First TSX component `components/StatusBar.tsx`
  consumes `StatusBarProps` from Slice 2 to prove the
  factory + Props types + element-vocabulary loop is closed
  end-to-end (`box{ text(glyph) + text(status) + text(warn?) }`,
  conditional render via `null` rather than `<></>` so the
  fragment factory binding stays an open question for now).
  Barrel re-exports `jsx` + `StatusBar`. `tsc --noEmit` clean
  under the strict profile. Slice 2b-rest (`App.tsx`,
  `NavRail.tsx`, `DetailPane.tsx`, `useBackend()`, the web
  smoke) lands on top of this anchor; `#1246` Slice 3
  inherits the same JSX runtime when it materializes the
  remaining 9 catalog stubs.
- **Slice 2c-5 (shipped) — desktop + TUI target structural stubs.**
  `projects/cue/app/src/targets/desktop/index.ts` and
  `projects/cue/app/src/targets/tui/index.ts` ship structural mirrors
  of the web target entry: each has its own `Stub{Desktop,Tui}Backend`
  (with target-named throw messages pointing at #1242 / #1241) and
  reuses the same no-op `paint_stub` shape. Each exports a
  `start_{desktop,tui}()` that calls `boot({backend, paint})` — same
  three-line body as `start_web()`, no per-target branches in
  `boot.ts`. Existence of these files closes the "structured for the
  remaining targets" half of AC1: the per-target boot pattern is
  provably uniform across web/desktop/tui, and the gap between stub
  and real impl is "swap two function references" rather than
  "design a new entry". No DOM, no Tauri SDK, no Node `process` — the
  `@cue/app` package keeps its `lib: ES2020` profile, and DOM /
  Tauri-IPC / ratatui access lives downstream of the `BootConfig`
  seam where each target's real impl will plug in. AC2 is also
  served: the in-file references to #1241 / #1242 make the deferred
  cross-target work explicit and grep-able. `tsc --noEmit` clean.
- **Slice 2c-4 (shipped) — html shell + bootstrap auto-invoke
  entry.** `projects/cue/app/src/targets/web/index.html` ships the
  web target's build-input shell — minimal, hand-written, no inline
  styles or analytics — with a single mount node `<div id="cue-app">`
  + `<script type="module" src="./bootstrap.ts">`. Sibling
  `bootstrap.ts` is the side-effect-only auto-invoke wrapper:
  `DOMContentLoaded → document.getElementById("cue-app") →
  start_web(root)`, with a loud throw when the mount node is missing.
  Splitting the auto-invoke from `start_web()` keeps `index.ts`
  callable from non-DOM hosts (jsdom-based unit tests, an SSR shim,
  a future web-worker bootstrap) — the html shell is the only entry
  with a hard DOM dependency. Closes the consumer-side half of AC1
  alongside the `paint_to_dom` slice: the build pipeline (#1239's
  `jet build --target web`) now walks `index.html → bootstrap.ts →
  start_web → boot → paint_to_dom` end-to-end, swapping the
  `StubWebBackend` for a real HTTP impl is a one-file change. `tsc
  --noEmit` clean.
- **Slice 2c-3 (shipped) — real `paint_to_dom` DOM walker for the
  web target.** `projects/cue/app/src/targets/web/paint.ts` ships
  `paint_to_dom(root: HTMLElement, tree: Element)` — a minimal
  recursive walker that materializes the discriminated union from
  `jsx.ts` (`empty` / `text` / `fragment` / `component` / `intrinsic`)
  and replaces `root.children` on every state change. Naive replace-
  children is intentional (foundation slice); diffing / keyed
  reconciliation lands only when a measured render becomes the
  bottleneck. Prop application is an explicit closed switch over the
  `IntrinsicProps` catalog (`className` / `style` / `id` / `onClick`
  / `onChange`) with a string/boolean attribute fallback — new typed
  slots from the renderer-contract gaps in #1246 will land as new
  branches, not via a generic dispatch table. DOM types are scoped
  to this file + `targets/web/index.ts` via `/// <reference lib="DOM" />`
  triple-slash directives so the rest of `@cue/app` keeps `lib:
  ES2020` only and DOM access cannot leak into target-agnostic app
  code. `start_web()` becomes `start_web(root: HTMLElement)`: the
  html shell (next slice) calls it after `DOMContentLoaded` with the
  mount node, swapping `paint_stub` for `(el) => paint_to_dom(root,
  el)`. The `StubWebBackend` is unchanged — the next slice swaps
  it for an HTTP-backed impl. `tsc --noEmit` clean. Closes the
  paint-side half of AC1 — the only remaining gap on AC1 is the
  build-tooling glue (entry-point manifest + html shell +
  `jet build --target web` integration), which lives in Jet, not in
  `@cue/app`.
- **Slice 2c-2 (shipped) — target-agnostic `boot(...)` scaffold +
  web target stub entry.** `projects/cue/app/src/boot.ts` lifts the
  per-target boilerplate (`setBackend(backend)` + `create_runtime` +
  `subscribe(state → paint(App({state, dispatch, lifecycle, ...})))` +
  `start()`) into a single `boot(BootConfig): Promise<CueRuntime>`
  function. `BootConfig` is the per-target seam — two fields only
  (`backend: CueBackend`, `paint: (Element) => void`) plus an optional
  `status_warning` forwarded to `AppShell`. `derive_lifecycle(state)`
  is colocated here (render-time concern, not reducer state). The web
  target lands as `projects/cue/app/src/targets/web/index.ts`: a
  private `StubWebBackend` (empty `list_issues`, throwing `get_issue`,
  `{ accepted: false, reason: "stub backend" }` from `submit_command`,
  immediately-empty `subscribe_events` async-iterator) plus a no-op
  `paint_stub` that just touches `element.kind` to placate
  `noUnusedParameters` without leaning on DOM globals — the `@cue/app`
  tsconfig deliberately ships `lib: ES2020` only, so DOM access lives
  behind the `paint` boundary, never in app code. The entry exports
  `start_web()` (no auto-invoke) so the build pipeline picks the call
  site — the html shell in the next slice does `start_web()` after
  `DOMContentLoaded`. `tsc --noEmit` clean. Closes the structural
  half of AC1 ("built against ≥1 target"): every per-target entry is
  now a tiny file that swaps `paint` + `backend` and calls `boot()` —
  the next slice (`paint_to_dom` DOM walker, real HTTP backend, html
  shell, `jet build --target web` integration) is a one-file swap of
  the placeholders, not a refactor.
- **Slice 2c-1 (shipped) — `CueRuntime` adapter (reducer driver +
  effect loop).** `projects/cue/app/src/runtime.ts` ships
  `create_runtime(backend) → CueRuntime` — the adapter the spec's
  "Command / query / event boundary" section requires and the
  `state.ts` top-of-file comment promised ("effects live in the
  runtime adapter that dispatches actions back into this reducer").
  Surface: `state()` snapshot, `dispatch(action)`, `subscribe(fn) →
  unsubscribe` (fires immediately + on every change), `start()` /
  `stop()` for lifecycle. Side-effect dispatcher is keyed off action
  type — `command_submitted` → `backend.submit_command` →
  `command_settled` (regardless of success / failure, so
  `pending_cmd` always drains and the CommandInput re-enables);
  `select_issue` → `backend.get_issue` → `issue_loaded` (drops if
  the user moved on, matching the reducer's stale-result rule).
  `start()` runs the initial `list_issues` fetch + spawns the
  detached `subscribe_events` loop that pushes `runtime_event` into
  the reducer until `stop()`. No React, no DOM, no fiber — every
  per-target boot wires the same instance and only differs in the
  `render(...)` function that paints the `Element` tree the App
  component returns. Error-state action lands in a follow-up slice
  (the catches currently swallow on purpose so the surface stays
  small until the reducer grows error fields). `tsc --noEmit`
  clean. Sets the foundation for AC1 ("built against ≥1 target") —
  the next slice is the actual web target boot that calls
  `create_runtime` + mounts `App` to DOM.
- **Slice 2b-5 (shipped) — `LogStream` reading `state.log_tail`.**
  `App.tsx` mounts `<LogStream>` in an `app-shell-log` row between
  the NavRail|DetailPane split and the bottom CommandInput row.
  The reducer's structured `LogLine { ts, level, msg }` collapses
  to the catalog spec's flat `lines: ReadonlyArray<string>` via a
  local `format_log_line` helper (`<ts> <LEVEL> <msg>`) — keeps
  the `LogStream` prop surface unchanged so renderer-side level
  colouring is a follow-up extension, not a contract churn.
  `on_clear` is intentionally not wired in this slice; the
  reducer has no `log_cleared` action yet, and an unbound clear
  surface beats a half-wired one. `tsc --noEmit` clean.
- **Slice 2b-4 (shipped) — `CommandInput` wired into `App.tsx`
  with reducer-owned input.** `projects/cue/app/src/state.ts`
  grows a controlled `input: string` field on `CueState` (default
  `""`) plus an `{ type: "input_changed"; value }` action that
  the reducer applies pointwise. `command_submitted` clears
  `input` alongside setting `pending_cmd` so the textbox drains
  the moment the dispatch lands — the runtime adapter is free to
  ack via `command_settled` whenever the backend resolves. The
  `App.tsx` shell mounts `<CommandInput>` in a third `app-shell-
  command` row underneath the split: `value = state.input`,
  `disabled = state.pending_cmd !== null`, `on_input` dispatches
  `input_changed`, `on_submit` lifts `state.input.trim()` into
  `command_submitted` (empty submits drop on the floor so the
  backend never sees a blank cmd). The actual call to
  `backend.submit_command` is deliberately NOT in the render
  path — it belongs in the per-target effect loop that watches
  `pending_cmd` transitions. `tsc --noEmit` clean. Remaining
  catalog wiring (`LogStream` reading `state.log_tail`, modal
  routing, ChatTranscript) follows the same pattern: extend the
  reducer, dispatch from the shell, leave I/O to the boot.
- **Slice 2b-3 (shipped) — `App.tsx` root component.**
  `projects/cue/app/src/components/App.tsx` composes the spec's
  Minimum element tree (`AppShell { NavRail | DetailPane }`) using
  the catalog pieces shipped in #1246 — `StatusBar` at the top,
  `IssueListPanel` + `IssueDetailPanel` in a horizontal split.
  Pure render: `AppProps { state, dispatch, lifecycle,
  status_warning? }` in, `Element` out. Per-target boot owns
  `setBackend(...)` + the reducer / effect loop and calls `App(...)`
  on every render — the shell stays renderer-neutral. The
  reducer's id-keyed `state.selected_id` is mapped to the list
  panel's idx-keyed cursor via `index_of_id`; `null` covers both
  "nothing selected" and "id no longer in list", so the panel
  paints with no cursor in either case. `on_open` reuses
  `on_select` for now (selection IS the open affordance) — when
  the reducer grows a separate "open detail" action, `on_open`
  re-targets without touching the panel surface. `tsc --noEmit`
  clean. The remaining catalog wiring (`CommandInput` →
  `submit_command` action, `LogStream` reading `state.log_tail`,
  modal routing) lands once each gets a per-action handler in the
  reducer; the App.tsx surface picks them up through `AppProps`
  without re-shaping the shell.
- **Slice 2b-2 (shipped) — `useBackend()` boundary hook +
  per-target registration.** `projects/cue/app/src/hooks.ts`
  ships the spec-mandated boundary the spec's "Command / query /
  event boundary" section makes mandatory: app code never calls
  `fetch()` / `invoke()` / any target IPC API. Three exports:
  `setBackend(backend: CueBackend)` — each target's entry point
  (TUI bootstrap, Tauri main, web bundle entry) calls this once
  before mount; `useBackend(): CueBackend` — every component
  reads the registered backend, throws loud-fast with a
  call-site-pinpointing error if no backend is registered (vs.
  silent `null` which would surface only on the next async
  call); `clearBackend()` — test/hot-reload escape hatch.
  Implementation is a module-level singleton — the simplest
  binding that satisfies the contract on every target and keeps
  the public surface stable when jet hooks gain a context
  provider primitive (the upgrade path is an internal swap; the
  three exported names don't change). Re-exported from the
  package barrel. `tsc --noEmit` clean under the existing strict
  profile (no new tsconfig). Runtime unit tests defer to the
  Slice 4 (TUI) + Slice 5 (desktop) integration paths since the
  package has no JS test runner yet — the contract is purely
  typed today.
- **Slice 3 (shipped) — `CueBackend` runtime trait + in-memory
  impl.** `projects/cue/src/runtime/backend.rs` defines the
  `CueBackend` async trait (`list_issues`, `get_issue`,
  `submit_command`, `subscribe_events`) plus the typed value
  shapes (`IssueSummary`, `IssueDetail`, `IssueStatus`,
  `TimelineEvent`, `LogLine`, `LogLevel`, `RuntimeEvent`,
  `CommandAck`, `BackendError`). All types `serde`-derive with
  `snake_case` enum tags so the JSON wire shape matches the
  TS-side interface 1:1; `RuntimeEvent` uses `#[serde(tag =
  "kind")]` so the desktop IPC bridge and the future web HTTP
  adapter share the same envelope. `EventStream =
  Pin<Box<dyn Stream<Item = RuntimeEvent> + Send + 'static>>`
  is the subscription return type — picks `Pin<Box<>>` over
  `BoxStream<'_, _>` so the trait is object-safe and has no
  borrowing lifetime on the backend. `InMemoryCueBackend`
  ships as the deterministic stub powering tests, the
  conformance harness, and the web-target stub bundle:
  `with_default_seed()` returns three issues spanning every
  status (Closed/InProgress/Open) plus three log lines
  (info/warn/error) on the event stream; `submit_command`
  trims whitespace, rejects empty/blank loud-fast, and
  records every accepted command into a Mutex-guarded log
  the test layer reads via `submitted_commands()`. 11 unit
  tests cover seed-data shape, get-by-id happy/miss paths,
  command record/trim/reject, event-stream replay, custom
  seed with all three `RuntimeEvent` variants, summary →
  detail field projection, and serde tag/casing. The real
  in-process adapter wrapping the existing `AgentRuntime`
  lands in Slice 4 once the runtime exposes the necessary
  read APIs.
- **Slice 4 — TUI demo.** Wires Slice 2 + Slice 3 through the
  TUI renderer landing in #1241. The same `App.tsx` source
  paints into the ratatui frame; the in-process `CueBackend`
  impl is the data source. Acceptance criterion #1 from #1240
  (minimal Jet-authored app surface, at least one initial
  target) is satisfied here.
- **Slice 5 — Desktop demo + cross-target conformance harness.**
  Same `App.tsx` source running inside the Tauri shell from
  #1242. Adds the Cue-side `BackendBridge` impl wrapping the
  in-process runtime for IPC. Adds the conformance harness
  assertion that the Cue app bundle uses ZERO browser-only or
  Tauri-only globals at the app layer (third acceptance
  criterion of #1240). Web target is the dev-loop default; the
  Cue cloud backend stays explicitly out of scope per epic
  #1237.

## Minimum element tree (Slice 1 contract)
<!-- type: interfaces lang: markdown -->

The Cue app layer is a single root component `App` rendering an
`AppShell` with two children. The element tree is renderer-neutral
and uses only the `Element` vocabulary from `element-contract.md`
(C1-C10 invariants).

```
App
└── AppShell { layout: split-horizontal, primary-pct: 30 }
    ├── NavRail
    │   ├── ListHeader { title: "Issues" }
    │   ├── List { items: state.issues, selected: state.selected_id }
    │   │   └── ListItem { id, title, status, on_select: SelectIssue }
    │   └── CommandInput { placeholder: "/aw wi …", on_submit: SubmitCommand }
    └── DetailPane
        ├── DetailHeader { title: state.selected_issue?.title ?? "—" }
        ├── DetailBody   { markdown: state.selected_issue?.body ?? "" }
        ├── StatusTimeline { events: state.selected_issue?.events ?? [] }
        └── LogStream     { lines: state.log_tail, scroll: tail }
```

Five primitives (`AppShell`, `List`, `ListItem`, `CommandInput`,
`LogStream`) — the rest are leaf `Text` / `Markdown` / container
elements already covered by C1-C5 of the element contract. Any
target that satisfies the contract can render the tree; capability
degradation is the renderer's concern, not the app's.

## State shape
<!-- type: interfaces lang: markdown -->

```ts
interface CueState {
  issues:        ReadonlyArray<IssueSummary>;     // hydrated on mount
  selected_id:   string | null;                   // nav-rail selection
  selected:      IssueDetail | null;              // hydrated on selection change
  log_tail:      ReadonlyArray<LogLine>;          // last N lines, N=500
  pending_cmd:   string | null;                   // optimistic display
}

interface IssueSummary { id: string; title: string; status: IssueStatus }
interface IssueDetail  extends IssueSummary { body: string; events: ReadonlyArray<TimelineEvent> }
interface LogLine      { ts: string; level: "info" | "warn" | "error"; msg: string }
interface TimelineEvent { ts: string; kind: string; summary: string }
type IssueStatus = "open" | "in_progress" | "closed"
```

State transitions are pure reducers (no I/O). I/O happens through
the `CueBackend` boundary (next section). The reducer surface is
small enough to fit in `projects/cue/app/state.ts` without a state
library — `useReducer` is sufficient.

## Command / query / event boundary
<!-- type: interfaces lang: markdown -->

The app layer NEVER calls `fetch()`, `invoke()`, or any
target-specific IPC API directly. All runtime traffic goes through
the `CueBackend` interface, surfaced to TSX as a `useBackend()`
hook:

```ts
interface CueBackend {
  list_issues():            Promise<ReadonlyArray<IssueSummary>>;
  get_issue(id: string):    Promise<IssueDetail>;
  submit_command(cmd: string): Promise<{ accepted: boolean; reason?: string }>;
  subscribe_events():       AsyncIterable<RuntimeEvent>;
}

type RuntimeEvent =
  | { kind: "log";       line: LogLine }
  | { kind: "issue_updated"; issue: IssueSummary }
  | { kind: "timeline";  issue_id: string; event: TimelineEvent };
```

Per-target wiring (Slice 3 + 4 + 5 implement these adapters):

| Target  | `CueBackend` impl                                  |
|---------|----------------------------------------------------|
| web     | HTTP/SSE client → `cue serve` runtime (out of scope for #1240 Slice 5; stub returns empty for the web build) |
| desktop | Tauri `invoke()` adapter → in-process runtime via #1242 Slice 3's `BackendBridge` |
| TUI     | direct in-process call → in-process runtime (zero IPC)                            |

The Rust-side trait `cue::runtime::backend::CueBackend` mirrors the
TS surface 1:1 (Slice 3). The TS `CueBackend` interface is the
single contract every target satisfies; tests that exercise the
reducer + view layer mock it without target awareness.

## Capability degradation
<!-- type: interfaces lang: markdown -->

Per `target-profiles.md`, the Cue app surface degrades cleanly
across renderers:

| Element          | web (full)         | desktop (full + shell) | TUI (degraded)                                    |
|------------------|--------------------|------------------------|---------------------------------------------------|
| `AppShell`       | flex split         | flex split + window    | ratatui horizontal split                          |
| `List`           | virtualized        | virtualized            | line-per-item, viewport scroll                    |
| `CommandInput`   | text input         | text input             | bottom-of-pane line input                         |
| `LogStream`      | virtualized tail   | virtualized tail       | tail with whole-line drop on overflow             |
| `Markdown` body  | full markdown      | full markdown          | text + bold/italic only (per TUI profile)         |
| `StatusTimeline` | icon + ts + text   | icon + ts + text       | glyph + ts + text (Unicode glyph, no PNG)         |

All degradation is the renderer's responsibility. The app layer
emits the same element tree on all targets; the tree shape is
the contract, the painted output is profile-driven.

## Conformance assertions (Slice 5)
<!-- type: test_plan lang: markdown -->

The conformance harness from #1238 gains three Cue-specific checks:

1. **No target-specific globals.** Static scan of the bundled
   app JS rejects `window`, `document`, `navigator`,
   `__TAURI__`, `chrome`, `webkit` references inside any file
   under `projects/cue/app/`. The runtime adapter layer is
   exempt (it lives outside the app boundary).
2. **All `CueBackend` methods exercised.** The harness drives
   the app through mount → list-render → selection → command-
   submit → log-tail-update and asserts every method on the
   trait was hit at least once. Catches dead code paths early.
3. **Element tree round-trip.** Render the app under each
   target profile and snapshot the element tree (not the
   pixels). All three snapshots must be byte-equal — divergence
   means a renderer or the app smuggled target-specific
   branching.

## Cross-references
<!-- type: overview lang: markdown -->

- `.aw/tech-design/crates/jet/logic/multi-target/element-contract.md` — `Element`
  vocabulary the tree above uses; C1-C10 invariants every
  renderer honors.
- `.aw/tech-design/crates/jet/logic/multi-target/target-profiles.md` — per-target
  capability matrix that drives the degradation table.
- `.aw/tech-design/crates/jet/logic/multi-target/tui-renderer.md` — TUI renderer
  spec; #1241 Slice 4 consumes this contract.
- `.aw/tech-design/crates/jet/logic/multi-target/desktop-runtime.md` — desktop
  shell spec; #1242 Slice 5 consumes this contract via
  `BackendBridge`.
- `cue-tui-lifecycle-ux.md` — existing ratatui-flavored Cue
  UX spec; superseded for cross-target work but kept as the
  TUI-specific extension (chat surface + modals not in the
  multi-target slice).
- `crates/jet-multi-target/src/web.rs` — `WebRenderer` the
  web + desktop targets reuse verbatim.

## Out of scope
<!-- type: overview lang: markdown -->

- **Cue cloud backend.** Web-target serving requires a hosted
  runtime; epic #1237 explicitly defers it. Web build under
  this slice ships a stub `CueBackend` that returns empty
  results — enough to prove the bundle loads and renders the
  empty-state UI.
- **Chat / agent / modal surfaces** from
  `cue-tui-lifecycle-ux.md`. Those stay TUI-only until the
  cross-target chat element vocabulary lands (separate
  follow-up; not in #1240).
- **Codegen.** Hand-written following the same SDD-bypass
  pattern used for #1238 / #1241 / #1242. The generator gap
  is tracked under epic #1237's standardization stream;
  regen will replay this spec once the renderer-neutral
  vocabulary is in the template registry.
