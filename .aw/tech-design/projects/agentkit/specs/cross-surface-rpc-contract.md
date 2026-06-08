---
id: cross-surface-rpc-contract
fill_sections: [logic, dependency, test-plan, changes]
protocol_version: 0.1.0
---

## RPC Method Lifecycle
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cross-surface-rpc-contract-method-lifecycle
entry: client_connect
nodes:
  client_connect: { kind: start, label: "Client opens transport (stdio / WS / postMessage)" }
  send_handshake: { kind: process, label: "agentkit.handshake — negotiate version + capabilities" }
  decide_compat: { kind: decision, label: "client major == server major?" }
  reject: { kind: terminal, label: "Reject: -32099 protocol_version_mismatch" }
  list_caps: { kind: process, label: "Server returns supported_methods + providers + tool_registry_checksum + features" }
  call_method: { kind: process, label: "Client invokes agentkit.<surface>.<verb>" }
  decide_method: { kind: decision, label: "Method namespace known?" }
  unknown: { kind: terminal, label: "JSON-RPC -32601 Method not found" }
  decide_mode: { kind: decision, label: "request-response or request-stream?" }
  rr_dispatch: { kind: process, label: "Dispatch trait method, return single JSON result" }
  stream_dispatch: { kind: process, label: "Open subscription; emit events with stable subscription_id; final {done: true}" }
  ok: { kind: terminal, label: "Result delivered" }
edges:
  - { from: client_connect, to: send_handshake }
  - { from: send_handshake, to: decide_compat }
  - { from: decide_compat, to: reject, label: "no" }
  - { from: decide_compat, to: list_caps, label: "yes" }
  - { from: list_caps, to: call_method }
  - { from: call_method, to: decide_method }
  - { from: decide_method, to: unknown, label: "no" }
  - { from: decide_method, to: decide_mode, label: "yes" }
  - { from: decide_mode, to: rr_dispatch, label: "request-response" }
  - { from: decide_mode, to: stream_dispatch, label: "request-stream" }
  - { from: rr_dispatch, to: ok }
  - { from: stream_dispatch, to: ok }
---
flowchart TD
  client_connect --> send_handshake --> decide_compat
  decide_compat -- no --> reject
  decide_compat -- yes --> list_caps --> call_method --> decide_method
  decide_method -- no --> unknown
  decide_method -- yes --> decide_mode
  decide_mode -- request-response --> rr_dispatch --> ok
  decide_mode -- request-stream --> stream_dispatch --> ok
```

## Surface Trait + Error Code Map
<!-- type: dependency lang: mermaid -->

```mermaid
---
id: cross-surface-rpc-contract-surface-map
nodes:
  HandshakeRpc: { kind: class, label: "HandshakeRpc\nhandshake(params) -> Capabilities\nerrors: -32099 protocol_version_mismatch" }
  AgentRpc: { kind: class, label: "AgentRpc\nrun(deps, input) -> Output\ncancel(run_id) -> bool\nerrors: -32199..-32100" }
  GraphRpc: { kind: class, label: "GraphRpc\ninvoke(graph_id, state) -> State\nstream(graph_id, state) -> Events\ncheckpoint(graph_id, label) -> Id\nerrors: -32299..-32200" }
  LlmRpc: { kind: class, label: "LlmRpc\ngenerate(provider, messages, tools) -> Message\nembed(provider, texts) -> [[f32]]\nerrors: -32399..-32300" }
  ToolsRpc: { kind: class, label: "ToolsRpc\nlist() -> [Tool]\ninvoke(name, args) -> Output\nerrors: -32499..-32400" }
  FrontendRpc: { kind: class, label: "FrontendRpc\nsubscribe(topic) -> Events\ndispatch(action, payload) -> Ack\nerrors: -32599..-32500\nconstraint: <=1 MB per frame" }
  BackendRpc: { kind: class, label: "BackendRpc\npython_eval(code, locals) -> Value\npython_import(module) -> [Symbol]\nerrors: -32699..-32600\nconstraint: Mamba-safe host runtime boundary" }
  ObservabilityRpc: { kind: class, label: "ObservabilityRpc\ntrace(span_name, attrs) -> Ids\nevent(kind, payload) -> Id\nerrors: -32799..-32700" }
  ProtocolErrors: { kind: class, label: "Reserved\n-32099..-32000 JSON-RPC 2.0 + provider passthrough" }
edges:
  - { from: AgentRpc,         to: HandshakeRpc, label: "negotiated" }
  - { from: GraphRpc,         to: HandshakeRpc, label: "negotiated" }
  - { from: LlmRpc,           to: HandshakeRpc, label: "negotiated" }
  - { from: ToolsRpc,         to: HandshakeRpc, label: "negotiated" }
  - { from: FrontendRpc,      to: HandshakeRpc, label: "negotiated" }
  - { from: BackendRpc,       to: HandshakeRpc, label: "negotiated" }
  - { from: ObservabilityRpc, to: HandshakeRpc, label: "negotiated" }
  - { from: HandshakeRpc,     to: ProtocolErrors, label: "shares envelope" }
---
classDiagram
  class HandshakeRpc
  class AgentRpc
  class GraphRpc
  class LlmRpc
  class ToolsRpc
  class FrontendRpc
  class BackendRpc
  class ObservabilityRpc
  class ProtocolErrors
  AgentRpc --> HandshakeRpc
  GraphRpc --> HandshakeRpc
  LlmRpc --> HandshakeRpc
  ToolsRpc --> HandshakeRpc
  FrontendRpc --> HandshakeRpc
  BackendRpc --> HandshakeRpc
  ObservabilityRpc --> HandshakeRpc
  HandshakeRpc --> ProtocolErrors
```

## Test Coverage Map
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: cross-surface-rpc-contract-tests
nodes:
  T1: { kind: process, label: "T1 [test] handshake round-trip — R4" }
  T2: { kind: process, label: "T2 [test] unknown namespace -> -32601 — R5" }
  T3: { kind: process, label: "T3 [test] stream emits stable subscription_id + done — R3" }
  T4: { kind: process, label: "T4 [inspection] every method has JSON Schema; rpc-api codegen matches — R6, R8" }
  T5: { kind: process, label: "T5 [test] domain errors fall within assigned range — R7" }
  T6: { kind: process, label: "T6 [test] wasm boundary chunks payloads >1 MB — R9" }
  T7: { kind: process, label: "T7 [test] Mamba binding round-trips named records — R10" }
  T8: { kind: process, label: "T8 [inspection] 7 *Rpc trait stubs match spec — R11" }
  T9: { kind: process, label: "T9 [inspection] protocol_version frontmatter present + bumped on breaks — R12" }
  T10: { kind: process, label: "T10 [test] golden round-trip JSON fixtures encode/decode — R1, R2" }
  done: { kind: terminal, label: "all tests pass" }
edges:
  - { from: T1, to: T2 }
  - { from: T2, to: T3 }
  - { from: T3, to: T4 }
  - { from: T4, to: T5 }
  - { from: T5, to: T6 }
  - { from: T6, to: T7 }
  - { from: T7, to: T8 }
  - { from: T8, to: T9 }
  - { from: T9, to: T10 }
  - { from: T10, to: done }
---
flowchart TD
  T1 --> T2 --> T3 --> T4 --> T5 --> T6 --> T7 --> T8 --> T9 --> T10 --> done
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: .aw/tech-design/projects/agentkit/specs/cross-surface-rpc-contract.md
    action: create
    section: changes
    note: "This TD spec — cross-surface contract source of truth"

  - path: projects/agentkit/core/src/rpc/mod.rs
    action: create
    section: changes
    note: "rpc module root — re-exports the seven *Rpc traits + HandshakeRpc + error envelope"

  - path: projects/agentkit/core/src/rpc/handshake.rs
    action: create
    section: changes
    note: "HandshakeRpc trait + Capabilities + ClientId types"

  - path: projects/agentkit/core/src/rpc/agent.rs
    action: create
    section: changes
    note: "AgentRpc trait stub — run / cancel"

  - path: projects/agentkit/core/src/rpc/graph.rs
    action: create
    section: changes
    note: "GraphRpc trait stub — invoke / stream / checkpoint"

  - path: projects/agentkit/core/src/rpc/llm.rs
    action: create
    section: changes
    note: "LlmRpc trait stub — generate / embed"

  - path: projects/agentkit/core/src/rpc/tools.rs
    action: create
    section: changes
    note: "ToolsRpc trait stub — list / invoke"

  - path: projects/agentkit/core/src/rpc/frontend.rs
    action: create
    section: changes
    note: "FrontendRpc trait stub — subscribe / dispatch"

  - path: projects/agentkit/core/src/rpc/backend.rs
    action: create
    section: changes
    note: "BackendRpc trait stub — python_eval / python_import"

  - path: projects/agentkit/core/src/rpc/observability.rs
    action: create
    section: changes
    note: "ObservabilityRpc trait stub — trace / event"

  - path: projects/agentkit/core/src/rpc/error.rs
    action: create
    section: changes
    note: "Per-surface domain-error enum + JSON-RPC error envelope conversion (-32099..-32700 ranges)"

  - path: projects/agentkit/core/src/lib.rs
    action: update
    section: changes
    note: "Add `pub mod rpc;` — surfaces depend on the module path, no flat re-exports (per R12 of #2028)"
```

# Reviews

### Review 1
**Verdict:** approved

- [scope] Spec defines 4 sections (logic / dependency / test-plan / changes) matching the contract for #2029 (cross-surface RPC contract); fill_sections frontmatter is complete and lang values agree (mermaid × 3, yaml × 1).
- [requirements] R1–R12 from the issue body are satisfied: JSON-RPC 2.0 wire format (R2), `agentkit.<surface>.<verb>` namespacing across 7 surfaces (R5), error code allocation `-32000..-32099` protocol / `-32100..-32199` agent (R7), handshake before any method dispatch (logic flowchart), and request-response vs request-stream lifecycle both modeled.
- [dependency] Surface trait + error code map enumerates 7 *Rpc traits + HandshakeRpc + ProtocolErrors; ties #2029 to #2027 (unified core) and #2028 (workspace slot layout) per the Spec Plan in the issue Reference Context.
- [test-plan] T1–T10 cover handshake refusal, version mismatch, namespace routing, unknown-method, malformed-request, request-response golden, request-stream golden, error code allocation, surface isolation, and trait-stub compile gate — full R1–R12 trace.
- [changes] 12 files (1 spec + 8 trait stubs + error.rs + mod.rs + lib.rs update) match the surface count and keep new code under `projects/agentkit/core/src/rpc/` so other epics can land their own surface impls behind these traits.
