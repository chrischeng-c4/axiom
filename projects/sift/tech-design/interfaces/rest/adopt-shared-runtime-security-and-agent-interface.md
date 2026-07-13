---
id: "1604"
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: sift-runtime-security-flow
entry: request
nodes:
  request: { kind: start, label: "incoming h2c or HTTP/1.1 request" }
  probe: { kind: decision, label: "standard probe route?" }
  standard: { kind: terminal, label: "serve health, readiness, metrics, spec, or docs" }
  auth: { kind: decision, label: "SIFT_AUTH required?" }
  bearer: { kind: decision, label: "bearer token valid and authorized?" }
  denied: { kind: terminal, label: "return shared 401 or 403 error envelope" }
  route: { kind: process, label: "dispatch Sift data-plane route" }
  result: { kind: terminal, label: "return Sift result with shared metrics and error contract" }
edges:
  - { from: request, to: probe }
  - { from: probe, to: standard, label: "yes" }
  - { from: probe, to: auth, label: "no" }
  - { from: auth, to: route, label: "off" }
  - { from: auth, to: bearer, label: "required" }
  - { from: bearer, to: denied, label: "no" }
  - { from: bearer, to: route, label: "yes" }
  - { from: route, to: result }
---
flowchart TD
    request([incoming request]) --> probe{standard route?}
    probe -->|yes| standard([serve probe/admin route])
    probe -->|no| auth{SIFT_AUTH required?}
    auth -->|off| route[dispatch data-plane route]
    auth -->|required| bearer{valid bearer?}
    bearer -->|no| denied([401 or 403])
    bearer -->|yes| route
    route --> result([shared metrics and error result])
```
