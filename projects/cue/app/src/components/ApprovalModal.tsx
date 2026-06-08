/** @jsx createElement */
// ApprovalModal — gate-modal for the human approve/revise step.
//
// Spec: `.aw/tech-design/projects/cue/cue-app-protocol-mapping.md`
// (catalog row "ApprovalModal"). Tag composition:
//   `Modal{ Text(prompt) + List<Text>(flagged) + ActionRow([approve, revise]) }`
//
// `Modal` and `ActionRow` (with cursor semantics) are two of the four
// primitive-gap issues called out in the mapping spec — until the
// renderer-contract gap closes, we emit a `<modal>` and `<action_row>`
// intrinsic and let the renderer pick a fallback. The TUI fallback is
// "inline panel that captures the top-pane area" per the capability
// table in target-profiles.md.

import { createElement, type Element } from "../jsx";
import type { ApprovalModalProps } from "../components.types";

export function ApprovalModal(props: ApprovalModalProps): Element {
  const { flagged, cursor, on_approve, on_revise, on_cursor } = props;
  return (
    <modal className="approval-modal" role="gate">
      <text className="modal-prompt">
        Approve flagged sections, or revise:
      </text>
      <list className="flagged-keys">
        {flagged.map((key, idx) => (
          <text
            key={key}
            className={`flagged-key${idx === cursor ? " cursor" : ""}`}
            onClick={() => on_cursor(idx)}
          >
            {key}
          </text>
        ))}
      </list>
      <action_row className="approval-actions">
        <text className="action approve" onClick={on_approve}>
          approve
        </text>
        <text
          className="action revise"
          onClick={() => on_revise(flagged)}
        >
          revise
        </text>
      </action_row>
    </modal>
  );
}
