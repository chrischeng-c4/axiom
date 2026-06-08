/** @jsx createElement */
// ModalRouter — dispatches one `Modal` payload to the matching
// catalog component.
//
// Spec: `.aw/tech-design/projects/cue/cue-app-protocol-mapping.md`
// §"Component catalog". The AppShell holds the active `Modal` and
// passes it here; this file is the only place that switches on
// `modal.kind`. Keeps each modal component dispatch-agnostic (its
// own props are already lifted from the union variant).
//
// `kind: "none"` returns EMPTY so the AppShell can unconditionally
// render <ModalRouter modal={state.modal} ...> without a guard.
// `kind: "message"` is an info-only modal with no routed action; we
// emit a minimal <modal> with the message text. The two routed
// variants (`gate`, `error_recovery`) lift their hooks from
// `ModalRouterProps` and forward to ApprovalModal / ErrorRecoveryModal.

import { createElement, type Element, EMPTY } from "../jsx";
import { ApprovalModal } from "./ApprovalModal";
import { ErrorRecoveryModal } from "./ErrorRecoveryModal";
import type { ModalRouterProps } from "../components.types";

export function ModalRouter(props: ModalRouterProps): Element {
  const {
    modal,
    on_approve_approve,
    on_approve_revise,
    on_approve_cursor,
    on_recovery_pick,
    on_recovery_cursor,
  } = props;

  switch (modal.kind) {
    case "none":
      return EMPTY;
    case "message":
      return (
        <modal className="message-modal">
          <text className="message">{modal.message}</text>
        </modal>
      );
    case "gate":
      return (
        <ApprovalModal
          flagged={modal.flagged}
          cursor={modal.cursor}
          on_approve={on_approve_approve}
          on_revise={on_approve_revise}
          on_cursor={on_approve_cursor}
        />
      );
    case "error_recovery":
      return (
        <ErrorRecoveryModal
          error_kind={modal.error_kind}
          message={modal.message}
          cursor={modal.cursor}
          on_pick={on_recovery_pick}
          on_cursor={on_recovery_cursor}
        />
      );
  }
}
