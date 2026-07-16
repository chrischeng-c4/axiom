<!-- HANDWRITE-BEGIN gap="missing-generator:logic:35524bd2" tracker="pending-tracker" reason="Tape-owned CLI, offline OpenAPI, generated-client, h2c, and llm contract cases adapted from Lumen's taxonomy. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)." -->
---
id: tape-cli-interface-ec
summary: CLI, offline OpenAPI, generated-client, transport, and LLM topic contracts for Tape.
fill_sections: [e2e-test]
---

# EC: CLI Interface

Lumen's CLI EC taxonomy applies to Tape's service archetype, but every
executable assertion remains specific to Tape's append, replay, checkpoint,
spec, and agent-facing command surface.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-cli-interface-offline-cli
    capability_id: cli-interface
    claim_id: tape-cli-replay-admin-contract
    contract_id: tape-offline-cli-agent-onboarding
    category: behavior
    command: "cargo test -p tape --test cli_contract --test behavior_tape_claim_cli_interface -- --nocapture"
    assertions:
      - "tape append, replay, checkpoint, spec, llm, upgrade, and issue parse through the real Tape binary."
      - "tape spec emits deterministic route, OpenAPI JSON, OpenAPI YAML, and JSON-schema output offline."
      - "tape llm publishes deterministic, offline guidance for topic replay and checkpoint workflows."
  - id: tape-cli-interface-generated-clients
    capability_id: http2-api-list
    claim_id: tape-openapi-generated-client-contract
    contract_id: tape-spec-gen-public-api-journey
    category: behavior
    command: "cargo test -p tape --test backup -- --nocapture"
    assertions:
      - "tape spec gen emits typed TypeScript, Python, and Rust client artifacts from Tape's offline OpenAPI document."
      - "The generated client contract stays scoped to Tape's topic append, replay, checkpoint, and administrative backup surfaces."
  - id: tape-cli-interface-protocol-transport
    capability_id: http2-api-list
    claim_id: tape-http1-h2c-listener-contract
    contract_id: tape-service-listener-http1-and-h2c
    category: behavior
    command: "cargo test -p tape --test http_transport -- --nocapture"
    assertions:
      - "Tape serves HTTP/1.1 and h2c prior-knowledge HTTP/2 on the same listener."
      - "Both protocols reach the same Tape topic replay API and standard operational endpoints."
```
<!-- HANDWRITE-END -->
