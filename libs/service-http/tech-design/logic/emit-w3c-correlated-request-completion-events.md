---
id: '2420'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: service-http-request-completion-flow
entry: request
nodes:
  request:
    kind: start
    label: "Inbound HTTP request with optional W3C traceparent"
  span:
    kind: process
    label: "CorrelatingMakeSpan validates parent context or creates a fresh local root and records method, uri, trace fields"
  handler:
    kind: process
    label: "The service router handles the request and produces an HTTP response"
  completion:
    kind: process
    label: "The shared response callback emits one INFO http_request_complete event with status and latency_ms inside the request span"
  jsonl:
    kind: process
    label: "ServiceJsonFormatter serializes the event and inherited request-span fields as axiom.service.log.v1 JSONL"
  boundary:
    kind: terminal
    label: "stdout carries JSONL; external collector owns all routing, credentials, retention, and Sift integration"
edges:
  - { from: request, to: span }
  - { from: span, to: handler }
  - { from: handler, to: completion }
  - { from: completion, to: jsonl }
  - { from: jsonl, to: boundary }
---
flowchart LR
    request([HTTP request plus traceparent]) --> span[CorrelatingMakeSpan]
    span --> handler[Service router handler]
    handler --> completion[INFO http_request_complete]
    completion --> jsonl[axiom.service.log.v1 JSONL]
    jsonl --> boundary([stdout to external collector])
```
