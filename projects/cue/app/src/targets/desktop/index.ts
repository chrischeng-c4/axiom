// Desktop target entry — Slice 2c-5 of #1240. Structurally mirrors
// `targets/web/index.ts` so #1242 (Tauri desktop) can swap the
// backend stub for a Tauri-IPC backend and the paint stub for a
// Tauri-webview-driven DOM walker without touching `boot.ts` or
// the App component. Existence of this file (alongside its TUI
// sibling) closes the "structured for the remaining targets" half
// of AC1 of #1240 — the per-target boot pattern is provably
// uniform across web/desktop/tui, the boot scaffold doesn't grow
// per-target branches, and the gap between stub and real impl is
// "swap two function references" rather than "design a new entry".
//
// No DOM, no Tauri SDK — keeps the @cue/app `lib: ES2020` profile
// honest. The real desktop entry pulls in Tauri's `@tauri-apps/api`
// and a webview-mounted `paint_to_dom`; both live behind the
// `BootConfig` seam, not in this file.

import { boot } from "../../boot";
import type { CueBackend } from "../../backend";
import type {
  CommandAck,
  IssueDetail,
  IssueSummary,
  RuntimeEvent,
} from "../../types";
import type { Element } from "../../jsx";

class StubDesktopBackend implements CueBackend {
  async list_issues(): Promise<ReadonlyArray<IssueSummary>> {
    return [];
  }
  async get_issue(_id: string): Promise<IssueDetail> {
    throw new Error(
      "StubDesktopBackend.get_issue: replace with Tauri IPC backend (#1242)",
    );
  }
  async submit_command(_cmd: string): Promise<CommandAck> {
    return { accepted: false, reason: "stub backend" };
  }
  async *subscribe_events(): AsyncIterable<RuntimeEvent> {
    // No events. Iterator ends immediately so `start()`'s detached
    // loop exits cleanly.
  }
}

function paint_stub(element: Element): void {
  void element.kind;
}

/**
 * Desktop-target boot entry. The Tauri shell calls `start_desktop()`
 * after the webview's `DOMContentLoaded`. Real impl swaps both the
 * `StubDesktopBackend` and `paint_stub` for Tauri-IPC + DOM-walker
 * implementations under #1242.
 */
export async function start_desktop(): Promise<void> {
  await boot({
    backend: new StubDesktopBackend(),
    paint: paint_stub,
  });
}
