// CueState reducer — single source of truth for the Cue UI app layer.
// Pure: no I/O. Effects (CueBackend calls) live in the runtime adapter
// that dispatches actions back into this reducer.

import type {
  IssueDetail,
  IssueSummary,
  LogLine,
  RuntimeEvent,
  TimelineEvent,
} from "./types";

export const LOG_TAIL_CAP = 500;

export interface CueState {
  readonly issues: ReadonlyArray<IssueSummary>;
  readonly selected_id: string | null;
  readonly selected: IssueDetail | null;
  readonly log_tail: ReadonlyArray<LogLine>;
  readonly pending_cmd: string | null;
  // Controlled value for CommandInput. The reducer is the source of
  // truth; the renderer is a projection. Cleared on command_submitted
  // so the input drains as soon as the dispatch lands.
  readonly input: string;
}

export const INITIAL_STATE: CueState = {
  issues: [],
  selected_id: null,
  selected: null,
  log_tail: [],
  pending_cmd: null,
  input: "",
};

export type CueAction =
  | { type: "issues_loaded"; issues: ReadonlyArray<IssueSummary> }
  | { type: "select_issue"; id: string }
  | { type: "issue_loaded"; detail: IssueDetail }
  | { type: "input_changed"; value: string }
  | { type: "command_submitted"; cmd: string }
  | { type: "command_settled" }
  | { type: "runtime_event"; event: RuntimeEvent };

export function reduce(state: CueState, action: CueAction): CueState {
  switch (action.type) {
    case "issues_loaded":
      return { ...state, issues: action.issues };

    case "select_issue":
      // Selection clears the stale detail so the UI can show a loading
      // affordance until `issue_loaded` arrives.
      return { ...state, selected_id: action.id, selected: null };

    case "issue_loaded":
      // Drop the result if the user moved on before the fetch resolved.
      if (state.selected_id !== action.detail.id) return state;
      return { ...state, selected: action.detail };

    case "input_changed":
      return { ...state, input: action.value };

    case "command_submitted":
      // Clear the input on submit — `pending_cmd` carries the in-flight
      // text until the runtime acks via `command_settled`.
      return { ...state, pending_cmd: action.cmd, input: "" };

    case "command_settled":
      return { ...state, pending_cmd: null };

    case "runtime_event":
      return applyRuntimeEvent(state, action.event);
  }
}

function applyRuntimeEvent(state: CueState, event: RuntimeEvent): CueState {
  switch (event.kind) {
    case "log":
      return { ...state, log_tail: appendCapped(state.log_tail, event.line) };

    case "issue_updated":
      return {
        ...state,
        issues: upsertIssue(state.issues, event.issue),
        selected:
          state.selected && state.selected.id === event.issue.id
            ? mergeSummaryIntoDetail(state.selected, event.issue)
            : state.selected,
      };

    case "timeline":
      return {
        ...state,
        selected:
          state.selected && state.selected.id === event.issue_id
            ? appendTimelineEvent(state.selected, event.event)
            : state.selected,
      };
  }
}

function appendCapped(
  tail: ReadonlyArray<LogLine>,
  line: LogLine,
): ReadonlyArray<LogLine> {
  const next = tail.length >= LOG_TAIL_CAP ? tail.slice(-LOG_TAIL_CAP + 1) : tail.slice();
  next.push(line);
  return next;
}

function upsertIssue(
  issues: ReadonlyArray<IssueSummary>,
  next: IssueSummary,
): ReadonlyArray<IssueSummary> {
  const idx = issues.findIndex((i) => i.id === next.id);
  if (idx === -1) return [...issues, next];
  const copy = issues.slice();
  copy[idx] = next;
  return copy;
}

function mergeSummaryIntoDetail(
  detail: IssueDetail,
  next: IssueSummary,
): IssueDetail {
  return {
    ...detail,
    title: next.title,
    status: next.status,
  };
}

function appendTimelineEvent(
  detail: IssueDetail,
  event: TimelineEvent,
): IssueDetail {
  return { ...detail, events: [...detail.events, event] };
}
