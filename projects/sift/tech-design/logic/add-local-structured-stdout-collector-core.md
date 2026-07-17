---
id: '1873'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: structured-stdout-collector-core
entry: configure
nodes:
  configure:
    kind: start
    label: "Sift collect receives source, stable source id, endpoint, project, environment, checkpoint, quarantine, batch, retry, and follow settings"
  resume:
    kind: process
    label: "load durable checkpoint and verify its source identity; seek or discard to acknowledged byte offset"
  read:
    kind: process
    label: "read a bounded JSONL window from file or stdin while tracking line start and end offsets"
  decode:
    kind: decision
    label: "line is bounded valid axiom.service.log.v1?"
  quarantine:
    kind: process
    label: "stage bounded rejection with source id, line, offset, code, message, and preview"
  map:
    kind: process
    label: "map service identity, event payload, attributes, severity, timestamps, and correlation to OperationalEventV2"
  id:
    kind: process
    label: "derive deterministic event id from source identity, line offset, and content digest"
  batch:
    kind: process
    label: "append to bounded delivery batch"
  window:
    kind: decision
    label: "batch full or current source window exhausted?"
  deliver:
    kind: process
    label: "POST EventWriteRequest through existing /v1/events:write with project and optional bearer token"
  outcome:
    kind: decision
    label: "all items accepted or duplicate?"
  retry:
    kind: process
    label: "retry retryable transport, 429, or 5xx failures with bounded exponential backoff"
  exhausted:
    kind: terminal
    label: "stop without advancing checkpoint and report runnable endpoint/token remediation"
  commit:
    kind: process
    label: "append staged quarantine records then atomically fsync checkpoint at the acknowledged window end"
  eof:
    kind: decision
    label: "EOF in follow mode?"
  wait:
    kind: process
    label: "wait bounded interval and continue reading appended bytes"
  done:
    kind: terminal
    label: "emit accepted, duplicate, rejected, line, and final-offset summary"
edges:
  - { from: configure, to: resume }
  - { from: resume, to: read }
  - { from: read, to: decode }
  - { from: decode, to: quarantine, label: "no" }
  - { from: decode, to: map, label: "yes" }
  - { from: map, to: id }
  - { from: id, to: batch }
  - { from: quarantine, to: window }
  - { from: batch, to: window }
  - { from: window, to: read, label: "no" }
  - { from: window, to: deliver, label: "valid items" }
  - { from: window, to: commit, label: "rejections only" }
  - { from: deliver, to: outcome }
  - { from: outcome, to: commit, label: "yes" }
  - { from: outcome, to: retry, label: "retryable" }
  - { from: retry, to: deliver, label: "attempt remains" }
  - { from: retry, to: exhausted, label: "exhausted" }
  - { from: commit, to: eof }
  - { from: eof, to: wait, label: "yes" }
  - { from: wait, to: read }
  - { from: eof, to: done, label: "no" }
---
flowchart TD
    config[collector config] --> resume[load source-matched checkpoint]
    resume --> read[read bounded JSONL window]
    read --> valid{v1 line valid?}
    valid -->|no| reject[stage bounded quarantine record]
    valid -->|yes| map[map OperationalEventV2 plus deterministic id]
    reject --> window{window ready?}
    map --> window
    window -->|no| read
    window -->|events| post[bounded POST to Sift ingest]
    post --> ack{accepted or duplicate?}
    ack -->|retryable| retry[bounded backoff retry]
    retry --> post
    ack -->|terminal or exhausted| stop([stop; checkpoint unchanged])
    ack -->|yes| commit[quarantine then atomic checkpoint]
    window -->|rejections only| commit
    commit --> eof{follow at EOF?}
    eof -->|yes| wait[wait for append]
    wait --> read
    eof -->|no| done([terminal summary])
```

The checkpoint is an acknowledgment boundary, not merely a read cursor: it never advances past a valid line until Sift durably accepts or identifies that event as a duplicate. Invalid structured lines use the explicit quarantine policy and may advance only after their bounded diagnostic is recorded. File one-shot, file follow, and stdin all call the same line/window pipeline; #1675 later supplies CRI records to that core without changing validation, mapping, delivery, or checkpoint semantics.
