// Per-component prop / event types for the target-neutral Cue surfaces.
// Spec: `.aw/tech-design/projects/cue/cue-app-protocol-mapping.md`
// §"Component catalog". One row in that table → one Props interface here.
//
// Slice 2 of #1246 ships interfaces only — no JSX, no runtime. Slice 3
// materializes each as a TSX skeleton in `components/`.

import type {
  ChatMessage,
  CostSummary,
  ErrorKind,
  ErrorRecoveryAction,
  IssueDetail,
  IssueSummary,
  LifecycleState,
  Modal,
} from "./protocol";

// --- ChatBubble ----------------------------------------------------------

// Display-only sub-component of ChatTranscript. Carries one ChatMessage plus
// the spinner_tick the renderer animates the pending indicator from.
export interface ChatBubbleProps {
  readonly message: ChatMessage;
  // Frame counter from `App.spinner_tick`. Tests pass 0 for deterministic
  // snapshots; production passes the live tick. Renderers % into their own
  // glyph table.
  readonly spinner_tick: number;
}

// --- ChatTranscript ------------------------------------------------------

export interface ChatTranscriptProps {
  readonly messages: ReadonlyArray<ChatMessage>;
  readonly spinner_tick: number;
  // Optional click-through: surface the transcript-level event hooks the
  // catalog spec lists. `undefined` means the renderer omits the binding.
  readonly on_scroll_to_end?: () => void;
  readonly on_bubble_clicked?: (idx: number) => void;
}

// --- CommandInput --------------------------------------------------------

export interface CommandInputProps {
  // Controlled value mirrored from `ChatState.input`. The reducer in
  // `state.ts` is the source of truth — the renderer is a projection.
  readonly value: string;
  readonly placeholder?: string;
  // Disabled while a turn is pending (`App.pending_turn === true`) so the
  // user can't queue a second submit.
  readonly disabled?: boolean;
  readonly on_input: (next: string) => void;
  readonly on_submit: () => void;
  // Optional escape binding (TUI: ESC; web/desktop: Esc key). Renderer is
  // free to ignore if the target has no equivalent affordance.
  readonly on_cancel?: () => void;
}

// --- IssueListPanel ------------------------------------------------------

export interface IssueListPanelProps {
  readonly items: ReadonlyArray<IssueSummary>;
  readonly selected_idx: number | null;
  readonly on_select: (idx: number) => void;
  // `open` is the affordance for keyboard <enter> / dblclick; routers may
  // navigate, dispatch a fetch, etc.
  readonly on_open: (idx: number) => void;
}

// --- IssueDetailPanel ----------------------------------------------------

export interface IssueDetailPanelProps {
  // null while the user has selected an issue but the detail fetch is still
  // in-flight (matches the `selected: IssueDetail | null` field in CueState
  // — the reducer drops stale results in this window).
  readonly issue: IssueDetail | null;
}

// --- StatusBar -----------------------------------------------------------

export interface StatusBarProps {
  readonly status: string;
  readonly lifecycle: LifecycleState;
  // Surfaced from `App.config_warning`; the renderer paints this with the
  // `warn` accent token when present.
  readonly config_warning?: string;
}

// --- ApprovalModal -------------------------------------------------------

// Carries only the Gate variant of `Modal`. The dispatcher in CueState
// pattern-matches `Modal.kind === "gate"` and lifts the payload to props.
export interface ApprovalModalProps {
  readonly flagged: ReadonlyArray<string>;
  readonly cursor: number;
  readonly on_approve: () => void;
  readonly on_revise: (flagged_keys: ReadonlyArray<string>) => void;
  // Cursor-only navigation (no submit) so the parent can re-emit the modal
  // with an updated cursor without re-dispatching approve/revise.
  readonly on_cursor: (next: number) => void;
}

// --- ErrorRecoveryModal --------------------------------------------------

export interface ErrorRecoveryModalProps {
  readonly error_kind: ErrorKind;
  readonly message: string;
  readonly cursor: number;
  readonly on_pick: (action: ErrorRecoveryAction) => void;
  readonly on_cursor: (next: number) => void;
}

// --- LogStream -----------------------------------------------------------

export interface LogStreamProps {
  // Bounded by `LOG_CAPACITY = 200` on the Rust side; the reducer in
  // `state.ts` caps log_tail to LOG_TAIL_CAP = 500 (web/desktop can hold
  // more frames). The renderer trims to its own viewport.
  readonly lines: ReadonlyArray<string>;
  readonly on_clear?: () => void;
}

// --- CostFooter ----------------------------------------------------------

// Optional — the CostSummary protocol field is planned but not yet shipped
// on the Rust side. Renderer skips this surface when `summary` is undefined.
export interface CostFooterProps {
  readonly summary?: CostSummary;
}

// --- App (root) ----------------------------------------------------------

import type { CueState, CueAction } from "./state";

// Root component. Composes the spec's Minimum element tree
// (`AppShell { NavRail | DetailPane }`) using the existing catalog
// pieces. Pure render: state in, dispatch out — per-target boot wires
// `setBackend(...)` + the reducer driver before mounting.
export interface AppProps {
  readonly state: CueState;
  readonly dispatch: (action: CueAction) => void;
  readonly lifecycle: LifecycleState;
  // Optional warning surfaced on the StatusBar (e.g. config validation
  // hint). Renderer paints with the `warn` accent when present.
  readonly status_warning?: string;
}

// --- Root modal switch ---------------------------------------------------

// Convenience type — the AppShell sits over the catalog and dispatches one
// Modal payload to the matching component. Slice 3's TSX wires this switch.
export type ModalRouterProps = {
  readonly modal: Modal;
  readonly on_approve_approve: ApprovalModalProps["on_approve"];
  readonly on_approve_revise: ApprovalModalProps["on_revise"];
  readonly on_approve_cursor: ApprovalModalProps["on_cursor"];
  readonly on_recovery_pick: ErrorRecoveryModalProps["on_pick"];
  readonly on_recovery_cursor: ErrorRecoveryModalProps["on_cursor"];
};
