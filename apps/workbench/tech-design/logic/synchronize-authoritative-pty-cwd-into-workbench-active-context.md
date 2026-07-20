---
id: '2194'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: workbench-authoritative-cwd-context
entry: launch
nodes:
  launch: { kind: start, label: "initialize active context from canonical selected launch folder" }
  output: { kind: process, label: "feed raw PTY output bytes into bounded OSC 7 decoder" }
  frame: { kind: decision, label: "complete ESC ] 7 ; file URI frame present?" }
  ignore: { kind: process, label: "ignore ordinary output and preserve only incomplete control-prefix bytes" }
  validate: { kind: decision, label: "localhost file URI resolves to an existing canonical directory?" }
  reject: { kind: process, label: "discard malformed, remote-host, missing, or non-directory telemetry" }
  update: { kind: process, label: "replace active cwd and emit a source-disclosed context update" }
  stable: { kind: terminal, label: "registered launch-folder registry remains unchanged" }
edges:
  - { from: launch, to: output }
  - { from: output, to: frame }
  - { from: frame, to: ignore, label: "no" }
  - { from: frame, to: validate, label: "yes" }
  - { from: validate, to: reject, label: "no" }
  - { from: validate, to: update, label: "yes" }
  - { from: ignore, to: stable }
  - { from: reject, to: stable }
  - { from: update, to: stable }
---
flowchart LR
    launch([Initial canonical launch cwd]) --> output[Feed PTY output bytes]
    output --> frame{Complete OSC 7 frame?}
    frame -->|No| ignore[Ignore ordinary output]
    frame -->|Yes| validate{Local existing directory?}
    validate -->|No| reject[Reject telemetry]
    validate -->|Yes| update[Update active cwd]
    ignore --> stable([Folder registry unchanged])
    reject --> stable
    update --> stable
```

Workbench uses the explicit OSC 7 current-directory protocol: `ESC ] 7 ; file://localhost/<percent-encoded-path> BEL` (and the standard ST terminator). `CwdTelemetryDecoder` accepts raw PTY byte chunks, survives frames split across arbitrary read boundaries, bounds retained incomplete data, and returns only complete file-URI payloads. Ordinary output is never parsed for paths, prompts, `cd`, or shell syntax.

`ActiveCwdContext` starts from the canonical selected launch folder and owns only the active context path plus the decoder. For each decoded URI it requires the `file` scheme, an empty or `localhost` host, a percent-decoded local path that canonicalizes successfully, and directory metadata. A valid changed directory replaces the active path and returns `CwdContextUpdate { path, source: Osc7 }`; malformed, remote, missing, and non-directory frames leave the prior context untouched. Duplicate frames are idempotent.

The PTY command environment discloses `WORKBENCH_CWD_TELEMETRY=osc7-file-uri-v1`. Integrated shells emit one frame after each successful directory transition; deterministic fixtures call the same `cwd_telemetry_frame` encoder. Failed `cd` operations emit ordinary error text but no successful frame. Direct vendor CLIs remain authoritative and may opt into the same terminal protocol without Workbench scraping their display output.

The tracker has no mutable access to `ShellState`; the registered folder list and selected launch id remain identity/launch configuration while active cwd is ephemeral runtime context. Tests retain a registry snapshot across successful and failed real-PTY transitions and assert byte-for-byte equality. Renderer selection remains outside this WI.
