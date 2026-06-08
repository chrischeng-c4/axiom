// Cue app domain types — 1:1 mirror of `projects/cue/src/runtime/backend.rs`.
// Spec: `.aw/tech-design/projects/cue/cue-multi-target-slice.md`
// §"State shape". Keep the wire shape in lockstep with the Rust side
// (snake_case enum tags, tagged-union RuntimeEvent under "kind").

export type IssueStatus = "open" | "in_progress" | "closed";

export interface IssueSummary {
  readonly id: string;
  readonly title: string;
  readonly status: IssueStatus;
}

export interface TimelineEvent {
  readonly ts: string;
  readonly kind: string;
  readonly summary: string;
}

export interface IssueDetail extends IssueSummary {
  readonly body: string;
  readonly events: ReadonlyArray<TimelineEvent>;
}

export type LogLevel = "info" | "warn" | "error";

export interface LogLine {
  readonly ts: string;
  readonly level: LogLevel;
  readonly msg: string;
}

export interface CommandAck {
  readonly accepted: boolean;
  readonly reason?: string;
}

// Tagged union — matches `#[serde(tag = "kind", rename_all = "snake_case")]`
// on the Rust `RuntimeEvent`. Discriminant order MUST stay in lockstep.
export type RuntimeEvent =
  | { readonly kind: "log"; readonly line: LogLine }
  | { readonly kind: "issue_updated"; readonly issue: IssueSummary }
  | {
      readonly kind: "timeline";
      readonly issue_id: string;
      readonly event: TimelineEvent;
    };

export type BackendErrorKind = "issue_not_found" | "invalid_command" | "internal";

export interface BackendError {
  readonly kind: BackendErrorKind;
  readonly message: string;
}
