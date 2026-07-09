---
id: lumen-cli-connect-query
summary: >
  Add `lumen connect` and `lumen query index|search|duplicates|collections
  list` so an agent can drive a k8s-deployed Lumen instance without hand-rolled
  `kubectl port-forward` process tracking, manual token-registry Secret
  decoding, or guessed HTTP bodies. `connect` resolves k8s coordinates
  (`--context`/`--namespace`/`--service`, or a `Lumen` CR name via `--cr`),
  spawns `kubectl port-forward` for the duration of a wrapped command, and
  tears it down (kill + wait) when the wrapped command exits regardless of its
  exit status. Both `connect` and `query` share one token-resolution helper
  that reads the same token-registry Secret convention documented by `lumen
  llm --topic auth`/`--topic quickstart` (map key IS the bearer token) and
  picks the first token whose role covers the request for the target
  collection or the wildcard resource. `query` subcommands assemble the exact
  wire body `lumen spec --shapes` publishes from structured flags/args — no
  interactive REPL, no new server-side endpoint, no token-registry format
  change.
capability_refs:
  - id: "cli-interface"
    role: primary
    claim: "service-process-interface"
    coverage: partial
    rationale: >
      Extends lumen's command surface with an agent-facing connect+query
      workflow layered entirely on the existing HTTP API and token-registry
      Secret convention, closing the gap where an agent had to hand-track a
      port-forward process and hand-decode a Secret to drive a deployed node.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-connect-query-contract
entry: start
nodes:
  start:     { kind: start,    label: "lumen connect --namespace N (--service S | --cr CR) [--secret SEC] -- CMD..." }
  resolvesvc: { kind: process, label: "service = --service or --cr (Lumen CR shares the client Service name)" }
  resolvesec: { kind: decision, label: "--secret given?" }
  crsecret:  { kind: process,  label: "kubectl get lumen <cr> -n N -o json; read spec.tokensSecret" }
  port:      { kind: process,  label: "local_port = --local-port or an OS-assigned free ephemeral port" }
  spawn:     { kind: process,  label: "spawn `kubectl [--context C] port-forward -n N svc/<service> local:remote` under a ChildGuard" }
  wait:      { kind: process,  label: "poll 127.0.0.1:local_port until connectable or 30s timeout" }
  token:     { kind: process,  label: "resolve_token: explicit token wins; else fetch+decode the resolved Secret's token-registry.json and select_token(role, collection)" }
  runcmd:    { kind: process,  label: "run CMD with LUMEN_URL=http://127.0.0.1:local_port and LUMEN_TOKEN=<resolved> set" }
  teardown:  { kind: terminal, label: "ChildGuard::drop kills+waits the port-forward on scope exit, whatever CMD's exit status was" }
  qstart:    { kind: start,    label: "lumen query index|search|duplicates|collections list --url U|$LUMEN_URL --token T|$LUMEN_TOKEN ..." }
  qtoken:    { kind: process,  label: "same resolve_token helper (explicit token, else --namespace/--secret Secret lookup)" }
  qbuild:    { kind: process,  label: "build_index_body / build_search_body / build_duplicates_body assemble the exact lumen::types wire request" }
  qsend:     { kind: process,  label: "POST/GET the assembled body against U; print the JSON response" }
edges:
  - { from: start,      to: resolvesvc }
  - { from: resolvesvc, to: resolvesec }
  - { from: resolvesec, to: port,       label: "yes" }
  - { from: resolvesec, to: crsecret,   label: "no, --cr given" }
  - { from: crsecret,   to: port }
  - { from: port,       to: spawn }
  - { from: spawn,      to: wait }
  - { from: wait,       to: token }
  - { from: token,      to: runcmd }
  - { from: runcmd,     to: teardown }
  - { from: qstart,     to: qtoken }
  - { from: qtoken,     to: qbuild }
  - { from: qbuild,     to: qsend }
---
flowchart TD
    start([lumen connect ... -- CMD]) --> resolvesvc[service = --service or --cr]
    resolvesvc --> resolvesec{--secret given?}
    resolvesec -->|yes| port[pick local_port]
    resolvesec -->|no, --cr| crsecret[kubectl get lumen CR -> spec.tokensSecret]
    crsecret --> port
    port --> spawn[spawn kubectl port-forward under ChildGuard]
    spawn --> wait[poll local port until ready or 30s timeout]
    wait --> token[resolve_token: explicit or Secret token-registry.json + select_token]
    token --> runcmd[run CMD with LUMEN_URL/LUMEN_TOKEN set]
    runcmd --> teardown([ChildGuard drop kills+waits port-forward])

    qstart([lumen query index|search|duplicates|collections list]) --> qtoken[resolve_token]
    qtoken --> qbuild[build_index_body/build_search_body/build_duplicates_body]
    qbuild --> qsend[POST/GET assembled body; print response]
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: lumen-connect-query-contract-verification
requirements:
  connect_teardown:
    id: R1
    text: "AC1: the port-forward child process is spawned under a ChildGuard and is no longer running once the guard drops (wrapped command exit, success or failure)"
    kind: functional
    risk: high
    verify: test
  token_resolution:
    id: R2
    text: "AC2: resolve_token/select_token return a usable bearer token from a parsed token-registry.json map (map key IS the token) without the caller decoding Secret/base64/JSON by hand; role hierarchy (Admin covers Read/Write) and the wildcard `*` resource are honored"
    kind: functional
    risk: high
    verify: test
  query_body_shapes:
    id: R3
    text: "AC3: build_index_body assembles the FLAT {items:[{external_id,field,value}]} shape published by lumen::spec::query_shapes()'s `index` entry (not a nested {id, fields:{...}} shape); build_search_body/build_duplicates_body assemble their corresponding lumen::types request shapes"
    kind: functional
    risk: high
    verify: test
  discoverability:
    id: R4
    text: "AC4: `lumen --help`/`lumen connect --help`/`lumen query --help` and `lumen llm --topic outline`/`--topic quickstart` document `connect` and `query`"
    kind: functional
    risk: medium
    verify: test
---
flowchart TD
    r1[R1 ChildGuard spawn/drop] --> v1{process gone after drop, both exit paths?}
    r2[R2 resolve_token/select_token] --> v2{role-covers + wildcard fallback, no manual decode?}
    r3[R3 build_*_body] --> v3{matches lumen::spec::query_shapes() and lumen::types wire shapes?}
    r4[R4 help/llm outline+quickstart] --> v4{connect/query mentioned?}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Add the connect/query Command variants, k8s coordinate + query-target arg structs, the ChildGuard port-forward lifecycle, the shared token-registry resolution helper, and the index/search/duplicates/collections-list body builders + HTTP dispatch."
  - path: projects/lumen/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Publish the flat `index` write-path shape in query_shapes() (closing the exact gap the issue reporter hit) and document connect/query in the llm outline + quickstart topics."
  - path: projects/lumen/src/bin/lumen.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    reason: "Cover the process-lifecycle (ChildGuard, port polling), token-resolution, and body-builder pure logic without requiring a live cluster."
```
