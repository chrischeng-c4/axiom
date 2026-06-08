// TUI target entry — Slice 2c-5 of #1240. Structurally mirrors
// `targets/web/index.ts` so #1241 (TUI renderer) can swap the
// backend stub for an in-process backend (calling the Cue Rust
// runtime directly, no transport) and the paint stub for a
// ratatui frame-buffer walker without touching `boot.ts` or the
// App component. Existence of this file (alongside its desktop
// sibling) closes the "structured for the remaining targets" half
// of AC1 of #1240.
//
// No `process`, no Node APIs, no ratatui — keeps the @cue/app
// `lib: ES2020` profile honest. The real TUI entry runs inside the
// Rust binary that links against jet-tui-renderer; the TS stub is
// here purely so the per-target seam is provably symmetric across
// every target the spec lists.

import { boot } from "../../boot";
import type { CueBackend } from "../../backend";
import type {
  CommandAck,
  IssueDetail,
  IssueSummary,
  RuntimeEvent,
} from "../../types";
import type { Element } from "../../jsx";

class StubTuiBackend implements CueBackend {
  async list_issues(): Promise<ReadonlyArray<IssueSummary>> {
    return [];
  }
  async get_issue(_id: string): Promise<IssueDetail> {
    throw new Error(
      "StubTuiBackend.get_issue: replace with in-process Cue runtime backend (#1241)",
    );
  }
  async submit_command(_cmd: string): Promise<CommandAck> {
    return { accepted: false, reason: "stub backend" };
  }
  async *subscribe_events(): AsyncIterable<RuntimeEvent> {
    // No events.
  }
}

function paint_stub(element: Element): void {
  void element.kind;
}

/**
 * TUI-target boot entry. The Rust TUI binary's TS shim calls
 * `start_tui()` once the terminal handle is acquired. Real impl
 * swaps both the `StubTuiBackend` and `paint_stub` for an
 * in-process backend + a ratatui-driven walker under #1241.
 */
export async function start_tui(): Promise<void> {
  await boot({
    backend: new StubTuiBackend(),
    paint: paint_stub,
  });
}
