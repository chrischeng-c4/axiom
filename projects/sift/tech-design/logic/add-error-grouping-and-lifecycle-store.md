---
id: "1666"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-error-store
entry: exception
nodes:
  exception: { kind: start, label: "committed exception event" }
  normalize: { kind: process, label: "normalize type message and stack frames" }
  fingerprint: { kind: process, label: "versioned deterministic fingerprint" }
  occurrence: { kind: process, label: "append ordered occurrence to group" }
  reopen: { kind: decision, label: "newer than resolved transition?" }
  checkpoint: { kind: terminal, label: "persist error projection checkpoint" }
  transition: { kind: start, label: "authorized lifecycle transition" }
  validate: { kind: decision, label: "valid state and mute expiry?" }
  commit: { kind: process, label: "commit through Sift state machine" }
  evidence: { kind: process, label: "append audit and change events" }
  done: { kind: terminal, label: "durable lifecycle result" }
edges:
  - { from: exception, to: normalize }
  - { from: normalize, to: fingerprint }
  - { from: fingerprint, to: occurrence }
  - { from: occurrence, to: reopen }
  - { from: reopen, to: checkpoint }
  - { from: transition, to: validate }
  - { from: validate, to: commit, label: "yes" }
  - { from: commit, to: evidence }
  - { from: evidence, to: done }
---
flowchart TD
    exception([exception]) --> normalize[normalize]
    normalize --> fingerprint[fingerprint v1]
    fingerprint --> occurrence[group occurrence]
    occurrence --> reopen{reopen?}
    reopen --> checkpoint([checkpoint])
    transition([lifecycle transition]) --> validate{valid?}
    validate -->|yes| commit[state machine commit]
    commit --> evidence[audit and change events]
    evidence --> done([durable result])
```
