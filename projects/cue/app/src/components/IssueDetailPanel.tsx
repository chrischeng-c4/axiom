/** @jsx createElement */
// IssueDetailPanel — display-only header / body / status timeline.
//
// Spec: `.aw/tech-design/projects/cue/cue-app-protocol-mapping.md`
// (catalog row "IssueDetailPanel"). Tag composition:
//   `Box{ DetailHeader + Markdown(body) + StatusTimeline }`
//
// `issue` is `null` while the user has selected an issue but the
// detail fetch is still in-flight. The reducer drops stale results
// in this window, so the renderer only paints once a real
// `IssueDetail` lands. The empty-state is surfaced as a single
// dim line, matching the spec's "no events bound — display-only"
// row in the catalog.
//
// `DetailHeader` and `StatusTimeline` are inlined here rather than
// pulled out into their own files: neither has a Props interface
// in the catalog (they're sub-components of `IssueDetailPanel`),
// and the renderer-vocabulary gap for nested timelines is wide
// enough that splitting now would just produce another layer of
// stub-files.

import { createElement, type Element, EMPTY } from "../jsx";
import type { IssueDetailPanelProps } from "../components.types";

export function IssueDetailPanel(props: IssueDetailPanelProps): Element {
  const { issue } = props;
  if (issue === null) {
    return (
      <box className="issue-detail-panel empty">
        <text className="dim">no issue selected</text>
      </box>
    );
  }
  return (
    <box className="issue-detail-panel">
      <box className="detail-header">
        <text className="issue-id dim">{`#${issue.id}`}</text>
        <text className="issue-title">{issue.title}</text>
        <text className={`issue-status status-${issue.status}`}>
          {issue.status}
        </text>
      </box>
      <markdown className="issue-body">{issue.body}</markdown>
      {issue.events.length > 0 ? (
        <list className="status-timeline">
          {issue.events.map((event, idx) => (
            <box key={idx} className={`timeline-event kind-${event.kind}`}>
              <text className="event-ts dim">{event.ts}</text>
              <text className="event-kind">{event.kind}</text>
              <text className="event-summary">{event.summary}</text>
            </box>
          ))}
        </list>
      ) : (
        EMPTY
      )}
    </box>
  );
}
