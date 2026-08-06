---
id: lumen-cli-connect-query
summary: >
  Add `lumen connect` and `lumen query index|search|duplicates|collections
  list` so an agent can drive a k8s-deployed Lumen instance without hand-rolled
  `kubectl port-forward` process tracking or guessed HTTP bodies. `connect`
  resolves k8s coordinates (`--context`/`--namespace`/`--service`, or a `Lumen`
  CR name via `--cr`), spawns `kubectl port-forward` for the duration of a
  wrapped command, and tears it down (kill + wait) when the wrapped command
  exits regardless of its exit status. Reachability is the whole contract:
  #2873 removed the credential half — the flag, the environment variable
  behind it, and the Secret lookup behind that — so the wrapped command is
  handed `LUMEN_URL` and nothing else, and `lumen query` sends no
  `Authorization` header. Kubernetes-native request identity returns as a
  `TokenRequest`-minted, audience-bound ServiceAccount token held in the CLI's
  own memory (#2878), never in the child's environment. `query` subcommands
  assemble the exact wire body `lumen spec --shapes` publishes from structured
  flags/args — no interactive REPL and no new server-side endpoint.
capability_refs:
  - id: "cli-interface"
    role: primary
    gap: "service-process-interface"
    claim: "service-process-interface"
    coverage: partial
    rationale: >
      Extends lumen's command surface with an agent-facing connect+query
      workflow layered entirely on the existing HTTP API, closing the gap
      where an agent had to hand-track a port-forward process to drive a
      deployed node.
  - id: "cli-interface"
    role: primary
    gap: "lumen-connect-query-k8s-agent-workflow"
    claim: "lumen-connect-query-k8s-agent-workflow"
    coverage: full
    rationale: "Defines the bounded port-forward and typed query workflow advertised by the CLI capability."
  - id: "developer-agent-experience"
    role: primary
    gap: "interactive-tooling"
    claim: "interactive-tooling"
    coverage: full
    rationale: "Owns the interactive `lumen connect` and `lumen query` agent tooling contract."
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: lumen-connect-query-contract
entry: start
nodes:
  start:     { kind: start,    label: "lumen connect --namespace N (--service S | --cr CR) -- CMD..." }
  resolvesvc: { kind: process, label: "service = --service or --cr (Lumen CR shares the client Service name)" }
  port:      { kind: process,  label: "local_port = --local-port or an OS-assigned free ephemeral port" }
  spawn:     { kind: process,  label: "spawn `kubectl [--context C] port-forward -n N svc/<service> local:remote` under a ChildGuard — the only kubectl call the verb makes (#2873)" }
  wait:      { kind: process,  label: "poll 127.0.0.1:local_port until connectable or 30s timeout" }
  notice:    { kind: process,  label: "print on stderr that the forwarded connection carries no identity, so a 401 from a serving node is explained where it is caused" }
  runcmd:    { kind: process,  label: "run CMD with LUMEN_URL=http://127.0.0.1:local_port set — and no other variable added to its environment" }
  teardown:  { kind: terminal, label: "ChildGuard::drop kills+waits the port-forward on scope exit, whatever CMD's exit status was" }
  qstart:    { kind: start,    label: "lumen query index|search|duplicates|collections list --url U|$LUMEN_URL ..." }
  qbuild:    { kind: process,  label: "build_index_body / build_search_body / build_duplicates_body assemble the exact lumen::types wire request" }
  qsend:     { kind: process,  label: "POST/GET the assembled body against U with no Authorization header; print the JSON response, or fail with the status the server returned" }
edges:
  - { from: start,      to: resolvesvc }
  - { from: resolvesvc, to: port }
  - { from: port,       to: spawn }
  - { from: spawn,      to: wait }
  - { from: wait,       to: notice }
  - { from: notice,     to: runcmd }
  - { from: runcmd,     to: teardown }
  - { from: qstart,     to: qbuild }
  - { from: qbuild,     to: qsend }
---
flowchart TD
    start([lumen connect ... -- CMD]) --> resolvesvc[service = --service or --cr]
    resolvesvc --> port[pick local_port]
    port --> spawn[spawn kubectl port-forward under ChildGuard]
    spawn --> wait[poll local port until ready or 30s timeout]
    wait --> notice[stderr: this connection carries no identity]
    notice --> runcmd[run CMD with LUMEN_URL set and nothing else]
    runcmd --> teardown([ChildGuard drop kills+waits port-forward])

    qstart([lumen query index|search|duplicates|collections list]) --> qbuild[build_index_body/build_search_body/build_duplicates_body]
    qbuild --> qsend[POST/GET assembled body, no auth header; print response or fail on status]
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
  no_credential_path:
    id: R2
    text: "AC2 (#2873): the CLI carries no request credential. `lumen connect` makes exactly one kubectl call — the port-forward — and adds exactly one variable, `LUMEN_URL`, to the wrapped command's environment; `lumen query` sends no Authorization header even with a credential-shaped variable in its environment, and surfaces the server's 401 rather than substituting anything it found"
    kind: security
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
    r2[R2 no credential path] --> v2{one kubectl call, one env var, no auth header, honest 401?}
    r3[R3 build_*_body] --> v3{matches lumen::spec::query_shapes() and lumen::types wire shapes?}
    r4[R4 help/llm outline+quickstart] --> v4{connect/query mentioned?}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/bin/lumen.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Add the connect/query Command variants, k8s coordinate + query-target arg structs, the ChildGuard port-forward lifecycle, and the index/search/duplicates/collections-list body builders + HTTP dispatch. #2873 removed this file's credential half: the flag, the environment variable behind it, and the Secret lookup behind that."
  - path: apps/lumen/src/spec.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Publish the flat `index` write-path shape in query_shapes() (closing the exact gap the issue reporter hit) and document connect/query in the llm outline + quickstart topics."
  - path: apps/lumen/src/bin/lumen.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    reason: "Cover the process-lifecycle (ChildGuard, port polling) and body-builder pure logic without requiring a live cluster."
  - path: apps/lumen/tests/cli_credential_paths_retired.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "#2873 R2: prove the credential half stays deleted — a scan of the shipped surface, plus a live `lumen connect` under a fake kubectl that serves a canary to any lookup, plus a `lumen query` against a server that records what arrived and answers 401."
  - path: libs/cli-std/src/connect.rs
    action: create
    section: logic
    impl_mode: hand-written
    reason: >
      #1376: the R1 primitives this doc specifies — the ChildGuard
      port-forward lifecycle (spawn/free_local_port/wait_for_local_port_ready)
      — moved verbatim into the new shared `libs/cli-std/src/connect.rs`
      module (feature `k8s`) so any k8s-native service CLI's own `connect`
      verb can reuse them, not just lumen's.
      `apps/lumen/src/bin/lumen.rs` keeps this doc's R3/R4 scope (query body
      builders/dispatch, flag surface, discoverability) plus a thin adapter
      over `cli_std::connect` for R1: the `Lumen` CRD-name lookup convention
      (`resource_kind = "lumen"`). That shared module also holds a credential
      resolver chain for the services that have not migrated; #2873 stopped
      lumen calling any of it, so nothing on lumen's side reads, derives,
      prints, or passes on a credential. See
      `libs/cli-std/tech-design/semantic/source/libs-cli-std-src-connect-rs.md`
      for the extracted module's own spec.
```
