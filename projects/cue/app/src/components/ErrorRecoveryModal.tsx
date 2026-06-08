/** @jsx createElement */
// ErrorRecoveryModal — modal that lets the user pick a recovery action.
//
// Spec: `.aw/tech-design/projects/cue/cue-app-protocol-mapping.md`
// (catalog row "ErrorRecoveryModal"). Tag composition:
//   `Modal(accent=from_kind){ Text(kind_label) + Markdown(message) +
//      ActionRow(ErrorRecoveryAction::ALL) }`
//
// Accent is keyed off the protocol's `ErrorKind` so each renderer can
// paint per-kind tokens (e.g. red for `score_process`, amber for `llm`).
// `ErrorRecoveryAction::ALL` is hand-rolled here as the static list
// `["retry", "dismiss", "new_issue"]` matching the protocol type
// (the protocol layer doesn't currently emit a runtime ALL constant —
// it's a string-literal union).

import { createElement, type Element } from "../jsx";
import type { ErrorRecoveryModalProps } from "../components.types";
import type { ErrorRecoveryAction } from "../protocol";

const RECOVERY_ACTIONS: ReadonlyArray<ErrorRecoveryAction> = [
  "retry",
  "dismiss",
  "new_issue",
];

const KIND_LABEL: { readonly [k: string]: string } = {
  score_process: "score binary error",
  llm: "LLM error",
  internal: "internal error",
};

export function ErrorRecoveryModal(props: ErrorRecoveryModalProps): Element {
  const { error_kind, message, cursor, on_pick, on_cursor } = props;
  return (
    <modal className={`error-recovery-modal kind-${error_kind}`}>
      <text className="kind-label">{KIND_LABEL[error_kind] ?? error_kind}</text>
      <markdown className="message">{message}</markdown>
      <action_row className="recovery-actions">
        {RECOVERY_ACTIONS.map((action, idx) => (
          <text
            key={action}
            className={`action ${action}${idx === cursor ? " cursor" : ""}`}
            onClick={() => on_pick(action)}
            onFocus={() => on_cursor(idx)}
          >
            {action}
          </text>
        ))}
      </action_row>
    </modal>
  );
}
