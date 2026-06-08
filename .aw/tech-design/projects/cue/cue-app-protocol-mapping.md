# Cue app-protocol → Jet UI contract mapping

> Issue: #1246 — `enhancement(cue): map app protocol to Jet UI
> contract`. Parent epic: #1243 (Cue app protocol). Related Jet
> epic: #1237 (multi-target). Sibling specs:
> `cue-tui-lifecycle-ux.md` (existing TUI-flavored UX),
> `cue-multi-target-slice.md` (#1240 dogfood scaffold —
> AppShell + NavRail + DetailPane).

## Goal
<!-- type: overview lang: markdown -->

Pin the **target-neutral component contract** for every
user-visible Cue surface so the same Jet-authored TSX runs on
the TUI (#1241), desktop (#1242), and the future web target.
Today the surfaces live in `projects/cue/src/tui/` as
ratatui-flavored Rust widgets bound to `crossterm`; this spec is
the lift from those widgets into props/events expressed in terms
of Cue's app protocol (`Action`, `Envelope`, `Modal`,
`ErrorKind`, lifecycle states) — never `tui::Frame`,
`crossterm::event::Event`, `web_sys::HtmlElement`, or any
target-specific API.

The deliverable is a contract that:

- Catalogs every component with its props, emitted events, and
  capability fallbacks.
- Anchors each prop in a concrete Cue protocol field (so the
  view-model side stays a pure projection, not a copy).
- Names the Jet element-vocabulary primitive each component
  composes from (so renderer authors know which contract piece
  to satisfy).

`cue-multi-target-slice.md` covers the **app shell** (NavRail
+ DetailPane + the issue list/detail surfaces) for the dogfood
slice. This spec is its sibling — it covers the **chat /
control / status surfaces** that the dogfood doesn't yet need
but the full Cue app does. Chat is the next bottleneck once
#1240 Slice 2b lands, so pinning the contract here unblocks
both the TUI demo (#1241 Slice 4) and the desktop demo
(#1242 Slice 5).

## Slices
<!-- type: overview lang: markdown -->

The mapping ships in three hand-written slices:

- **Slice 1 (this doc) — contract.** Pin the seven Cue
  components, their props/events shape (anchored in protocol
  fields), and the capability-fallback table. No code lands.
- **Slice 2 (shipped) — TS contract types.** `projects/cue/app/
  src/protocol.ts` mirrors the Rust side 1:1: `ChatRole` (closed
  union of six roles) + `CHAT_ROLE_LABEL` lookup, `ChatMessage`
  (role / content / pending / model?), `LifecycleState`,
  `ErrorKind`, `ErrorRecoveryAction` + `ERROR_RECOVERY_ACTIONS`
  ordered array, `Modal` discriminated union (`none` / `gate` /
  `message` / `error_recovery`) with the `MODAL_NONE` constant for
  the no-modal sentinel, `CostSummary` placeholder for the planned
  Rust field, and a re-export of `IssueSummary` / `IssueDetail` /
  `TimelineEvent` from `types.ts` so component types pull only
  from the protocol module. `components.types.ts` ships eleven
  read-only Props interfaces — `ChatBubbleProps`,
  `ChatTranscriptProps`, `CommandInputProps`, `IssueListPanelProps`,
  `IssueDetailPanelProps`, `StatusBarProps`, `ApprovalModalProps`,
  `ErrorRecoveryModalProps`, `LogStreamProps`, `CostFooterProps`,
  plus the `ModalRouterProps` convenience type the AppShell uses
  to dispatch one `Modal` payload to the matching component —
  each anchored to a row in the catalog above. Optional event
  handlers are `?:`; the modal Props carry `cursor` + `on_cursor`
  so the parent can re-emit with an updated cursor without
  re-dispatching submit. The barrel `index.ts` re-exports both
  modules. `tsc --noEmit` clean under the strict
  `noFallthroughCasesInSwitch + noUnusedLocals +
  noImplicitOverride` profile.
- **Slice 3 (shipped — catalog stubs + gap issues filed) —
  TSX skeletons.** Materialize each component as a stub TSX
  function in `projects/cue/app/src/components/` that takes the
  props from Slice 2 and returns a placeholder Jet element tree.
  Drives the renderer-side capability surface to completeness;
  gaps go back as Jet issues per the acceptance criterion (third
  bullet). All ten catalog rows + the `ModalRouter` dispatch are
  now stubbed, on top of #1240's Slice 2b-1 JSX runtime:
  `StatusBar.tsx` (anchor — Slice 2b-1), `ChatBubble.tsx`
  (display-only, role label + dim model + content + Spinner gap),
  `LogStream.tsx` (Scroll/List vocabulary + an inline `clear`
  action_row when `on_clear` is bound), `CostFooter.tsx` (returns
  `EMPTY` when `summary` is undefined, matching the "no-op until
  protocol lands" contract), `ChatTranscript.tsx` (vertical Scroll
  wrapping a List of ChatBubble rows; transcript-level
  `on_scroll_to_end` / `on_bubble_clicked` hooks are optional so
  renderers can omit bindings the target can't surface),
  `CommandInput.tsx` (single `<text_input>` controlled-input
  projection of `ChatState.input`, with `disabled` driven by
  `App.pending_turn`), `IssueListPanel.tsx` (List of selectable
  rows; `on_select` on cursor move, `on_open` on enter/dblclick —
  uses real `IssueStatus` field, no speculative `state` prop),
  `IssueDetailPanel.tsx` (null-issue empty state; otherwise
  `Box{ DetailHeader + Markdown(body) + StatusTimeline }` with the
  `events` array rendered in-place), `ApprovalModal.tsx`
  (`<modal>` + flagged-key `<list>` + `<action_row>` with `approve`
  / `revise` actions — exercises three of the four
  primitive-gap intrinsics — see #1333/#1334/#1335),
  `ErrorRecoveryModal.tsx`
  (`<modal>` keyed by `ErrorKind`, `<markdown>` body, and an
  `<action_row>` cycling through `ErrorRecoveryAction`'s three
  variants with cursor highlight), and `ModalRouter.tsx`
  (single `switch (modal.kind)` site that returns `EMPTY` for
  `"none"`, a minimal `<modal>` text card for `"message"`, and
  forwards to ApprovalModal / ErrorRecoveryModal for the routed
  `"gate"` / `"error_recovery"` variants — this is the only
  place that switches on the discriminated union). The four
  primitive-gap issues are now filed against `crates/jet`:
  Spinner (#1333), Modal + focus-trap (#1334), ActionRow with
  cursor (#1335), Markdown capability flag on `TargetProfile`
  (#1336). With those filed, all three AC of #1246 are met
  and the issue closes.

## Component catalog
<!-- type: interfaces lang: markdown -->

Seven user-visible components. Each row is a target-neutral
contract; renderers pick presentation (canvas div / Tauri
WebView / ratatui Paragraph) per `target-profiles.md`.

| Component | Cue protocol source | Jet element composition | Emitted events |
|---|---|---|---|
| `ChatTranscript` | `ChatState.messages: Vec<ChatMessage>` (`tui::chat`) | `Scroll(direction=vertical){ List<ChatBubble> }` | `scroll_to_end()`, `bubble_clicked(idx)` |
| `ChatBubble` | `ChatMessage` (role / content / pending / model) | `Box{ Text(role_label) + Text(model?, dim) + Markdown(content) + Spinner(if pending) }` | none — display-only |
| `CommandInput` | `ChatState.input: String` + `App.pending_turn: bool` | `TextInput(value, placeholder, disabled)` | `input(value)`, `submit()`, `cancel()` |
| `IssueListPanel` | `App.issues: Vec<IssueEntry>` + `App.selected` | `List<IssueListItem>(items, selected_idx)` | `select(idx)`, `open(idx)` |
| `IssueDetailPanel` | `App.detail_body: Option<String>` + `selected_issue` | `Box{ DetailHeader + Markdown(body) + StatusTimeline }` | none — display-only |
| `StatusBar` | `App.status: String` + `App.lifecycle_state` + `App.config_warning?` | `Box(direction=horizontal){ Text(lifecycle_glyph) + Text(status) + Text(warning?, accent=warn) }` | none — display-only |
| `ApprovalModal` | `Modal::Gate { flagged, cursor }` | `Modal{ Text(prompt) + List<Text>(flagged) + ActionRow([approve, revise]) }` | `approve()`, `revise(flagged_keys)` |
| `ErrorRecoveryModal` | `Modal::ErrorRecovery { kind, message, cursor }` | `Modal(accent=from_kind){ Text(kind_label) + Markdown(message) + ActionRow(ErrorRecoveryAction::ALL) }` | `pick(action: ErrorRecoveryAction)` |
| `LogStream` | `App.log: VecDeque<String>` (cap `LOG_CAPACITY=200`) | `Scroll(scroll=tail){ List<Text>(lines) }` | `clear()` |
| `CostFooter` | new — `App.cost_summary: { in_tokens, out_tokens, usd }` (planned) | `Box(direction=horizontal){ Text(in/out tokens) + Text(usd, dim) }` | none — display-only |

> The catalog has 10 rows because `ChatBubble` is a
> sub-component of `ChatTranscript` and `ApprovalModal` /
> `ErrorRecoveryModal` are sibling modals that share the
> `Modal` wrapper primitive but carry different prop shapes.
> The seven user-visible *surfaces* the issue body lists
> (`transcript, command input, issue list/detail, status bar,
> approval modal, log stream, cost footer`) are all present;
> `ChatBubble` and `ErrorRecoveryModal` are implementation
> peers.

## Protocol → props anchoring
<!-- type: interfaces lang: markdown -->

Every prop above is derived 1:1 from a Rust field in
`projects/cue/src/tui/{app,chat,envelope}.rs`. Slice 2's
`protocol.ts` is the boundary type module — it reflects:

```ts
// from projects/cue/src/tui/chat.rs
export type ChatRole =
  | "user"
  | "assistant"
  | "author"
  | "reviewer"
  | "reviser"
  | "system";

export interface ChatMessage {
  readonly role: ChatRole;
  readonly content: string;
  readonly pending: boolean;
  readonly model?: string;
}

// from projects/cue/src/tui/app.rs (LifecycleState enum)
export type LifecycleState =
  | "idle"
  | "running"
  | "done"
  | "error";

// from projects/cue/src/tui/app.rs (ErrorKind enum)
export type ErrorKind = "score_process" | "llm" | "internal";

export type ErrorRecoveryAction = "retry" | "dismiss" | "new_issue";

// Modal — discriminated union mirroring the Rust enum.
export type Modal =
  | { readonly kind: "none" }
  | { readonly kind: "gate"; readonly flagged: ReadonlyArray<string>; readonly cursor: number }
  | { readonly kind: "message"; readonly message: string }
  | {
      readonly kind: "error_recovery";
      readonly error_kind: ErrorKind;
      readonly message: string;
      readonly cursor: number;
    };
```

The view-model layer (CueState in `state.ts`) is the only place
that knows whether the protocol type came from an in-process
direct call (TUI), Tauri IPC (desktop), or HTTP/SSE (web). Each
component receives plain TS objects shaped exactly like the
protocol types — no async iteration, no transport plumbing.

## Capability fallbacks per target
<!-- type: interfaces lang: markdown -->

Per `target-profiles.md`. The view-model surface is identical
across targets; only the renderer presentation degrades.

| Element / cue   | web (full) | desktop (full) | TUI (degraded — explicit, behavior-preserving) |
|---|---|---|---|
| `Markdown(body)` | full GFM | full GFM | bold/italic only; lists rendered as `- ` prefixes; code blocks as monospace boxes |
| `Spinner` | CSS animation, 60fps | CSS animation, 60fps | rotating glyph (`⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`) at the App's `spinner_tick` cadence |
| `Modal` | overlay with dim backdrop | overlay with dim backdrop | inline panel that captures the top-pane area; backdrop is a single dim bg color |
| `ChatBubble.role_label` color | per-role accent | per-role accent | per-role color from the existing `cue-tui-lifecycle-ux.md` design tokens (no opacity) |
| `Scroll(tail)` | virtualized + smooth scroll | virtualized + smooth scroll | whole-line drop on overflow (no smooth scroll); cap matches `LOG_CAPACITY` |
| `TextInput` typing | native input + IME | native input + IME | char-by-char `crossterm::event::KeyCode::Char(_)`; no IME composition |
| `ActionRow` | mouse + keyboard | mouse + keyboard | keyboard only (left/right cursor, enter to pick) |

All degradations are pre-existing in `cue-tui-lifecycle-ux.md`;
this table just rolls them up in a renderer-keyed view so #1241
Slice 4 has a checklist to verify against.

## Renderer contract gaps surfaced
<!-- type: overview lang: markdown -->

Working through the ten-row catalog uncovers four primitive
gaps in the current `element-contract.md` vocabulary that #1241
+ #1242 will need to satisfy. Per the acceptance criterion
("feed any renderer contract gaps back into Jet issues"), these
file as separate Jet work-items and are NOT silently absorbed
into Cue-only rendering branches:

1. **`Spinner`** (#1333) — animated indicator with a pluggable
   cadence source (web: CSS keyframes / TUI: app-driven `tick`
   counter). Today `element-contract.md` has only static
   primitives.
2. **`Modal`** (#1334) — overlay primitive with a focus-trap
   contract (any renderer must signal which child captures
   keyboard focus while the modal is open).
3. **`ActionRow`** (#1335) — composite of action items with a
   first-class "selected" cursor (so TUI keyboard arrows and
   web/desktop tab-stop both have a consistent contract).
4. **`Markdown` capability matrix** (#1336) — degraded TUI
   rendering needs an explicit capability flag the app can
   read to skip features (e.g. tables, images) that the
   active renderer does not support.

All four were filed in Slice 3 once the TSX skeletons made the
gap concrete (avoiding paper-tiger issues). They now own the
follow-through under epic #1237.

## Cross-references
<!-- type: overview lang: markdown -->

- `cue-tui-lifecycle-ux.md` — TUI-specific UX for the same
  components. This spec lifts those into target-neutral form;
  any divergence is a bug in this mapping (not in the TUI
  spec).
- `cue-multi-target-slice.md` — #1240 dogfood scaffold;
  defines the AppShell + NavRail + DetailPane surfaces. This
  spec covers the chat / control / status complement.
- `.aw/tech-design/crates/jet/logic/multi-target/element-contract.md` — Jet element
  vocabulary; the four gaps above will extend it.
- `.aw/tech-design/crates/jet/logic/multi-target/target-profiles.md` — capability
  matrix that drives the per-target degradation table.
- `projects/cue/src/tui/{app,chat,envelope}.rs` — current
  Rust source of truth for the protocol types Slice 2 mirrors.

## Out of scope
<!-- type: overview lang: markdown -->

- **Implementation of any component.** Slice 2/3 ship types
  + skeletons; full TSX implementations land per-component as
  the dogfood needs them.
- **Implementation of the four primitive-gap issues.** Filing
  closes #1246 (per AC3); satisfying #1333/#1334/#1335/#1336
  is the work of `crates/jet/multi-target` under epic #1237.
- **Cue protocol changes.** Adding a `cost_summary` field to
  the lifecycle envelope (the `CostFooter` row above hints
  at it) is a separate Cue-side issue; this spec only lists
  the field as planned and notes the contract lands once that
  protocol PR merges.
- **Animation / transition timing.** Renderer-specific.
