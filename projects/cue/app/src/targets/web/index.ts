/// <reference lib="DOM" />

// Web target entry. The smallest tsc-clean boot path that satisfies
// AC1 of #1240 ("built against ≥1 target"): wires the canonical
// `boot(...)` scaffold from `../../boot` with the real
// `paint_to_dom` DOM walker (Slice 2c-3) + a placeholder backend.
// The backend stub is local (not exported from `@cue/app`) because
// production targets MUST NOT use it — the next slice swaps it for
// an HTTP-backed impl pointing at the Cue Rust runtime.
//
// Spec: `cue-multi-target-slice.md` §"Slice 2c-3 — paint_to_dom DOM
// walker". The `jet build --target web` integration (entry-point
// manifest, html shell, dev-server config) sits on a follow-up
// slice — this file proves the wiring tsc-compiles and gives the
// html shell a stable entry point to call.

import { boot } from "../../boot";
import type { CueBackend } from "../../backend";
import type {
  CommandAck,
  IssueDetail,
  IssueSummary,
  RuntimeEvent,
} from "../../types";
import { paint_to_dom } from "./paint";

// Placeholder backend — fixed in-memory data so the web bundle
// boots without a server during early development. Swapped for an
// HTTP-backed impl pointing at the Cue Rust runtime in the next
// slice. The stub is local (not exported from `@cue/app`) because
// production targets MUST NOT use it.
class StubWebBackend implements CueBackend {
  async list_issues(): Promise<ReadonlyArray<IssueSummary>> {
    return [];
  }
  async get_issue(_id: string): Promise<IssueDetail> {
    throw new Error("StubWebBackend.get_issue: replace with real backend");
  }
  async submit_command(_cmd: string): Promise<CommandAck> {
    return { accepted: false, reason: "stub backend" };
  }
  async *subscribe_events(): AsyncIterable<RuntimeEvent> {
    // No events. Iterator ends immediately so `start()`'s detached
    // loop exits cleanly.
  }
}

/**
 * Web-target boot entry. The html shell (next slice) calls
 * `start_web(root)` after `DOMContentLoaded`, where `root` is the
 * mount node — typically `document.getElementById("cue-app")`. Caller
 * owns the lookup so this entry stays HTMLElement-typed and has no
 * implicit dependency on a particular DOM id.
 */
export async function start_web(root: HTMLElement): Promise<void> {
  await boot({
    backend: new StubWebBackend(),
    paint: (element) => paint_to_dom(root, element),
  });
}
