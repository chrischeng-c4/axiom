/** @jsx createElement */
// App — root component for the Cue UI.
//
// Spec: `.aw/tech-design/projects/cue/cue-multi-target-slice.md`
// §"Minimum element tree". Composes the spec's
// `AppShell { NavRail | DetailPane }` shape using the catalog
// pieces shipped in #1246: `StatusBar` + `IssueListPanel` + the
// detail surface from `IssueDetailPanel`.
//
// Pure render — `props.state` in, `props.dispatch` out. Per-target
// boot is responsible for: (1) `setBackend(...)` once, (2) running
// the reducer + effect loop, (3) calling `App({ state, dispatch,
// lifecycle })` on every render. The shell stays renderer-neutral;
// the same source paints into web canvas / desktop window / TUI
// frame buffer (TUI + desktop are p3 follow-ups — the contract is
// preserved on this branch but not exercised yet).

import { createElement, type Element } from "../jsx";
import type { AppProps } from "../components.types";
import type { IssueSummary, LogLine } from "../types";
import { StatusBar } from "./StatusBar";
import { IssueListPanel } from "./IssueListPanel";
import { IssueDetailPanel } from "./IssueDetailPanel";
import { CommandInput } from "./CommandInput";
import { LogStream } from "./LogStream";

// Map the reducer's id-keyed selection to the list panel's
// idx-keyed cursor. `null` covers both "nothing selected" and
// "id no longer in list" — the panel paints with no cursor.
function index_of_id(
  items: ReadonlyArray<IssueSummary>,
  id: string | null,
): number | null {
  if (id === null) return null;
  const i = items.findIndex((it) => it.id === id);
  return i === -1 ? null : i;
}

// Format a structured `LogLine` into the flat string the
// `LogStream` surface paints. The reducer holds rich `LogLine`
// values (ts / level / msg) so future renderers can colour by
// level, but the spec's catalog row for `LogStream` is plain
// `List<Text>(lines)` — so we collapse here at the shell rather
// than reshape the renderer-facing prop.
function format_log_line(line: LogLine): string {
  return `${line.ts} ${line.level.toUpperCase()} ${line.msg}`;
}

// Stable status string for the StatusBar. Keeps lifecycle text
// pinned in one place — the StatusBar component itself only owns
// the glyph; the human-readable label comes from here so the
// reducer doesn't have to.
function lifecycle_label(lifecycle: AppProps["lifecycle"]): string {
  switch (lifecycle) {
    case "idle":
      return "ready";
    case "running":
      return "working";
    case "done":
      return "done";
    case "error":
      return "error";
  }
}

export function App(props: AppProps): Element {
  const { state, dispatch, lifecycle, status_warning } = props;
  const selected_idx = index_of_id(state.issues, state.selected_id);

  const on_select = (idx: number): void => {
    const item = state.issues[idx];
    if (item !== undefined) {
      dispatch({ type: "select_issue", id: item.id });
    }
  };

  // `on_open` is the same dispatch in this slice — selection IS
  // the open affordance. Once the reducer grows a separate
  // "open detail" action (e.g. focus DetailPane), `on_open` can
  // re-target without touching the panel surface.
  const on_open = on_select;

  // CommandInput wiring. `value` is mirrored from the reducer
  // (`input_changed` keeps it controlled); `on_submit` lifts the
  // current input into a `command_submitted` action which the
  // per-target effect loop picks up and forwards to
  // `backend.submit_command`. The reducer clears `input` on submit
  // so the textbox drains immediately. Empty submits are dropped
  // here so the backend never sees a blank cmd.
  const on_input = (next: string): void => {
    dispatch({ type: "input_changed", value: next });
  };
  const on_submit = (): void => {
    const trimmed = state.input.trim();
    if (trimmed.length === 0) return;
    dispatch({ type: "command_submitted", cmd: trimmed });
  };

  return (
    <box className="app-shell">
      <StatusBar
        status={lifecycle_label(lifecycle)}
        lifecycle={lifecycle}
        config_warning={status_warning}
      />
      <box className="app-shell-split">
        <box className="app-shell-pane app-shell-pane-nav">
          <IssueListPanel
            items={state.issues}
            selected_idx={selected_idx}
            on_select={on_select}
            on_open={on_open}
          />
        </box>
        <box className="app-shell-pane app-shell-pane-detail">
          <IssueDetailPanel issue={state.selected} />
        </box>
      </box>
      <box className="app-shell-log">
        <LogStream lines={state.log_tail.map(format_log_line)} />
      </box>
      <box className="app-shell-command">
        <CommandInput
          value={state.input}
          placeholder="run a cue command…"
          disabled={state.pending_cmd !== null}
          on_input={on_input}
          on_submit={on_submit}
        />
      </box>
    </box>
  );
}
