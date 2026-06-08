/** @jsx createElement */
// IssueListPanel — vertical list of issue summaries with cursor.
//
// Spec: `.aw/tech-design/projects/cue/cue-app-protocol-mapping.md`
// (catalog row "IssueListPanel"). Tag composition:
//   `List<IssueListItem>(items, selected_idx)`
//
// `selected_idx` is `null` when no issue is selected (matches the
// `App.selected: Option<usize>` field on the Rust side). The
// renderer paints the selected row with the active-cursor accent;
// non-selected rows use the default accent.
//
// `on_select` fires on cursor move (j/k or arrow keys / mouse
// hover); `on_open` fires on the keyboard <enter> / dblclick
// affordance — routers may navigate, dispatch a fetch, etc.

import { createElement, type Element } from "../jsx";
import type { IssueListPanelProps } from "../components.types";

export function IssueListPanel(props: IssueListPanelProps): Element {
  const { items, selected_idx, on_select, on_open } = props;
  return (
    <list className="issue-list-panel">
      {items.map((item, idx) => {
        const selected = idx === selected_idx;
        return (
          <box
            key={item.id}
            className={`issue-list-item${selected ? " selected" : ""}`}
            onClick={() => on_select(idx)}
            onActivate={() => on_open(idx)}
          >
            <text className="issue-id dim">{`#${item.id}`}</text>
            <text className="issue-title">{item.title}</text>
            <text className={`issue-status status-${item.status}`}>
              {item.status}
            </text>
          </box>
        );
      })}
    </list>
  );
}
