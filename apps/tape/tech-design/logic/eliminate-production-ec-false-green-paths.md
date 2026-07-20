---
id: '2159'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-production-ec-independent-oracles
entry: load_review
nodes:
  load_review:
    kind: start
    label: "Load the six digest-bound false-green findings"
  local_perf:
    kind: process
    label: "Compare observed local metrics to EC-owned fixed limits"
  peer_perf:
    kind: process
    label: "Run real NATS and Kafka peers, fail closed, and compute ratios in the test layer"
  generated_clients:
    kind: process
    label: "Generate and inspect TypeScript, Python, and Rust Tape clients"
  competitor_oracle:
    kind: process
    label: "Read a versioned official-source competitor baseline fixture"
  security_oracle:
    kind: process
    label: "Align auth, audit, guard, and meter commands with executed journeys"
  regenerate:
    kind: process
    label: "Regenerate all EC cases and reject zero-test execution"
  semantic_review:
    kind: decision
    label: "Does independent agent review accept the current digest?"
  revise:
    kind: process
    label: "Revise only the finding's EC source or independent runner"
  verify:
    kind: process
    label: "Run EC verify and the owning TD code-check"
  done:
    kind: terminal
    label: "Tape production EC cannot pass through the six false-green paths"
edges:
  - { from: load_review, to: local_perf }
  - { from: local_perf, to: peer_perf }
  - { from: peer_perf, to: generated_clients }
  - { from: generated_clients, to: competitor_oracle }
  - { from: competitor_oracle, to: security_oracle }
  - { from: security_oracle, to: regenerate }
  - { from: regenerate, to: semantic_review }
  - { from: semantic_review, to: verify, label: "accepted" }
  - { from: semantic_review, to: revise, label: "needs revision" }
  - { from: revise, to: regenerate }
  - { from: verify, to: done }
---
flowchart TD
  load_review([Load six digest-bound findings]) --> local_perf[Independent fixed local limits]
  local_perf --> peer_perf[Fail-closed real peer ratios]
  peer_perf --> generated_clients[Inspect three generated client languages]
  generated_clients --> competitor_oracle[Consume versioned official-source baseline]
  competitor_oracle --> security_oracle[Align security claims and commands]
  security_oracle --> regenerate[Regenerate all EC cases]
  regenerate --> semantic_review{Independent review accepted?}
  semantic_review -->|accepted| verify[Run EC verify and TD code-check]
  semantic_review -->|needs revision| revise[Revise bounded source or runner]
  revise --> regenerate
  verify --> done([False-green paths closed])
```
