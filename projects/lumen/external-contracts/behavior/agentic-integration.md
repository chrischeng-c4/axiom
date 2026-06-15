---
id: lumen-agentic-integration-ec
summary: Agent-facing offline CLI contracts for schema, query catalog, and LLM topics.
fill_sections: [e2e-test]
---

# EC: Agentic Integration

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-agentic-integration-offline-cli
    capability_id: agentic-integration
    claim_id: lumen-spec-schema-openapi-json-yaml-json-schema-offline
    contract_id: offline-cli-agent-onboarding
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "lumen spec emits valid OpenAPI JSON, OpenAPI YAML, and JSON-schema output offline."
      - "lumen spec exposes query-shape, field, analyzer, and vector-metric catalogs."
      - "lumen llm outline, workflow, integration, quickstart, and recipes preserve the ingest-search-hydrate agent workflow and non-goals."
      - "lumen llm integration recommends the Postgres/AlloyDB boundary: database commit/outbox or CDC, external adapter-owned Pub/Sub retry/DLQ, HTTP writes into lumen, and no direct external publishing to lumen's NATS WAL."
  - id: lumen-agentic-integration-query-catalog
    capability_id: agentic-integration
    claim_id: query-shape-cookbook-field-analyzer-catalog
    contract_id: query-shape-cookbook-field-analyzer-catalog
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "lumen spec exposes query-shape, field, analyzer, and vector-metric catalogs."
      - "agent-facing query catalog output remains deterministic and offline."
  - id: lumen-agentic-integration-llm-playbook
    capability_id: agentic-integration
    claim_id: lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes
    contract_id: lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "lumen llm outline, workflow, integration, quickstart, and recipes preserve the agent-facing topic set."
      - "lumen llm integration preserves the provider-neutral Postgres/AlloyDB adapter guidance and keeps Pub/Sub-specific ownership outside lumen core."
      - "agent-facing playbook output remains deterministic and offline."
```
