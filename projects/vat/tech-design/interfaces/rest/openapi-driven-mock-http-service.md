---
id: vat-openapi-driven-mock-http-service
summary: Read an OpenAPI spec and auto-generate mock HTTP responses in vat — a standalone openapi preset server and a transparent source inside the http-mock proxy.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "Adds an OpenAPI mock engine (spec -> response from example/schema) as both a standalone openapi preset and an http-mock proxy source, so an agent can stand up a working fake of a documented API with no stubs/recording and no app code change."
---

# Vat OpenAPI-Driven Mock HTTP Service

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-openapi-driven-mock-http-service-logic
entry: start
nodes:
  start: { kind: start, label: "openapi preset or http-mock with registered spec" }
  load: { kind: process, label: "parse OpenAPI spec as value yaml or json" }
  request: { kind: process, label: "incoming method and path" }
  match: { kind: decision, label: "match operation by templated path and method" }
  miss: { kind: terminal, label: "404 not in spec" }
  resp: { kind: process, label: "select 2xx else default response" }
  body: { kind: decision, label: "example or examples or schema" }
  example: { kind: process, label: "use response example or examples value" }
  synth: { kind: process, label: "synthesize from schema resolve ref depth-guarded" }
  serve: { kind: process, label: "return status content-type body" }
  proxy: { kind: decision, label: "http-mock resolution stub openapi cassette forward" }
  done: { kind: terminal, label: "response delivered" }
edges:
  - { from: start, to: load }
  - { from: load, to: request }
  - { from: request, to: match }
  - { from: match, to: miss, label: "no match" }
  - { from: match, to: resp, label: "match" }
  - { from: resp, to: body }
  - { from: body, to: example, label: "example present" }
  - { from: body, to: synth, label: "schema only" }
  - { from: example, to: serve }
  - { from: synth, to: serve }
  - { from: serve, to: proxy }
  - { from: proxy, to: done }
---
flowchart TD
    start([openapi preset or http-mock with registered spec]) --> load[parse OpenAPI spec as value yaml or json]
    load --> request[incoming method and path]
    request --> match{match operation by templated path and method}
    match -- no match --> miss([404 not in spec])
    match -- match --> resp[select 2xx else default response]
    resp --> body{example or examples or schema}
    body -- example present --> example[use response example or examples value]
    body -- schema only --> synth[synthesize from schema resolve ref depth-guarded]
    example --> serve[return status content-type body]
    synth --> serve
    serve --> proxy{http-mock resolution stub openapi cassette forward}
    proxy --> done([response delivered])
```
