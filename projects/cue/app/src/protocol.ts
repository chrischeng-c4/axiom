// Cue app-protocol types — 1:1 mirror of `projects/cue/src/tui/{app,chat,
// envelope}.rs`. Spec: `.aw/tech-design/projects/cue/
// cue-app-protocol-mapping.md` §"Protocol → props anchoring".
//
// The view-model layer is the only place that knows the transport (in-process
// for TUI, Tauri IPC for desktop, HTTP/SSE for web). Component prop shapes
// (see `components.types.ts`) consume these protocol types directly.

// --- Chat ----------------------------------------------------------------

// Mirrors `projects/cue/src/tui/chat.rs::ChatRole`. snake_case wire shape so
// the same JSON envelope crosses every transport.
export type ChatRole =
  | "user"
  | "assistant"
  | "author"
  | "reviewer"
  | "reviser"
  | "system";

// Stable role-label table — matches `ChatRole::label()` on the Rust side.
// Renderers use this for the bubble prefix; keep in lockstep.
export const CHAT_ROLE_LABEL: { readonly [K in ChatRole]: string } = {
  user: "user",
  assistant: "assistant",
  author: "author",
  reviewer: "reviewer",
  reviser: "reviser",
  system: "system",
};

export interface ChatMessage {
  readonly role: ChatRole;
  readonly content: string;
  // True while the assistant is still streaming this turn.
  readonly pending: boolean;
  // Model behind this turn (e.g. `gemini-2.5-pro`). Only set for subagent
  // roles (Author / Reviewer / Reviser); rendered as a parenthetical suffix.
  readonly model?: string;
}

// --- Lifecycle -----------------------------------------------------------

// Mirrors `LifecycleState` in `projects/cue/src/tui/app.rs`. Transitions are
// monotonic forward (`idle` → `running` → `done`/`error`); the StatusBar
// component uses this to drive the leading glyph.
export type LifecycleState = "idle" | "running" | "done" | "error";

// --- Errors --------------------------------------------------------------

// Mirrors `ErrorKind` — drives the ErrorRecoveryModal accent color and the
// default cursor position.
export type ErrorKind = "score_process" | "llm" | "internal";

// Mirrors `ErrorRecoveryAction::ALL`. Order is the cursor-index order rendered
// in the modal's ActionRow.
export type ErrorRecoveryAction = "retry" | "dismiss" | "new_issue";

export const ERROR_RECOVERY_ACTIONS: ReadonlyArray<ErrorRecoveryAction> = [
  "retry",
  "dismiss",
  "new_issue",
];

// --- Modal ---------------------------------------------------------------

// Discriminated union mirroring the Rust `Modal` enum. Renderers switch on
// `kind`; non-modal state is the `none` variant (NOT a nullable type) so the
// component contract is total and never reads `undefined`.
export type Modal =
  | { readonly kind: "none" }
  | {
      readonly kind: "gate";
      readonly flagged: ReadonlyArray<string>;
      readonly cursor: number;
    }
  | { readonly kind: "message"; readonly message: string }
  | {
      readonly kind: "error_recovery";
      readonly error_kind: ErrorKind;
      readonly message: string;
      readonly cursor: number;
    };

export const MODAL_NONE: Modal = { kind: "none" };

// --- Issues (mirror — these also appear in `types.ts`; re-exported with the
// same shape so component types can pull only from this protocol module). --

import type { IssueSummary, IssueDetail, TimelineEvent } from "./types";
export type { IssueSummary, IssueDetail, TimelineEvent };

// --- Cost (planned) ------------------------------------------------------

// Footer surface — `App.cost_summary` is not yet on the Rust side. The
// contract is pinned here so the protocol PR has a target shape; the
// CostFooter component consumes this `?` until the field lands.
export interface CostSummary {
  readonly in_tokens: number;
  readonly out_tokens: number;
  readonly usd: number;
}
