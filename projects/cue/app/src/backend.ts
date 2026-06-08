// `CueBackend` — TS-side mirror of the Rust trait at
// `projects/cue/src/runtime/backend.rs`. Every target adapter (TUI direct
// call, desktop Tauri IPC, web HTTP) provides an implementation; the app
// layer only ever sees this interface, never the underlying transport.

import type {
  BackendError,
  CommandAck,
  IssueDetail,
  IssueSummary,
  RuntimeEvent,
} from "./types";

export interface CueBackend {
  list_issues(): Promise<ReadonlyArray<IssueSummary>>;
  get_issue(id: string): Promise<IssueDetail>;
  submit_command(cmd: string): Promise<CommandAck>;
  // Async iterable matches the Rust `EventStream`. Adapters fan in from
  // a real subscription (broadcast / SSE / IPC) — the in-memory stub
  // replays a fixed buffer and ends.
  subscribe_events(): AsyncIterable<RuntimeEvent>;
}

export class CueBackendError extends Error {
  readonly kind: BackendError["kind"];

  constructor(err: BackendError) {
    super(err.message);
    this.name = "CueBackendError";
    this.kind = err.kind;
  }
}
