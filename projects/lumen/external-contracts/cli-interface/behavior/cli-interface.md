---
id: lumen-cli-interface-ec
summary: CLI contracts for schema, query catalog, and LLM topics.
fill_sections: [e2e-test]
---

# EC: CLI Interface

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-cli-interface-offline-cli
    capability_id: cli-interface
    claim_id: lumen-spec-schema-openapi-json-yaml-json-schema-offline
    contract_id: offline-cli-agent-onboarding
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "lumen spec emits valid OpenAPI JSON, OpenAPI YAML, and JSON-schema output offline."
      - "lumen spec exposes query-shape, field, analyzer, and vector-metric catalogs."
      - "lumen llm outline, workflow, integration, quickstart, and recipes preserve the ingest-search-hydrate agent workflow and non-goals."
      - "lumen llm integration recommends the Postgres/AlloyDB boundary: database commit/outbox or CDC, external adapter-owned Pub/Sub retry/DLQ, HTTP writes into lumen, and no direct external publishing to lumen's internal broker WAL."
  - id: lumen-cli-interface-query-catalog
    capability_id: cli-interface
    claim_id: query-shape-cookbook-field-analyzer-catalog
    contract_id: query-shape-cookbook-field-analyzer-catalog
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "lumen spec exposes query-shape, field, analyzer, and vector-metric catalogs."
      - "agent-facing query catalog output remains deterministic and offline."
  - id: lumen-cli-interface-generated-clients
    capability_id: cli-interface
    claim_id: lumen-spec-schema-openapi-json-yaml-json-schema-offline
    contract_id: spec-gen-generated-clients-public-api-journey
    category: behavior
    command: "cargo test -p lumen --test behavior_lumen_cli_interface_generated_clients -- --ignored --nocapture"
    assertions:
      - "lumen spec gen emits Python, TypeScript, and Rust clients from the offline OpenAPI document."
      - "generated clients drive health, readiness, version, collection creation, indexing, search, duplicates, stats, and forced drop against a live h2c Lumen service."
      - "the generated Python client validates recursive pydantic QueryNode union shapes while using the bundled h2c runtime."
  - id: lumen-cli-interface-llm-playbook
    capability_id: cli-interface
    claim_id: lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes
    contract_id: lumen-llm-agent-topics-outline-workflow-integration-quickstart-recipes
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "lumen llm outline, workflow, integration, quickstart, and recipes preserve the agent-facing topic set."
      - "lumen llm integration preserves the provider-neutral Postgres/AlloyDB adapter guidance and keeps Pub/Sub-specific ownership outside lumen core."
      - "agent-facing playbook output remains deterministic and offline."
```
