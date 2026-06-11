---
id: lumen-external-contracts
summary: External contract gates for Lumen production-scoped README capabilities.
fill_sections: [e2e-test]
capability_refs:
  - id: agentic-integration
    role: primary
    gap: lumen-llm-agent-integration-playbook-guide-quickstart-recipes
    claim: lumen-llm-agent-integration-playbook-guide-quickstart-recipes
    coverage: full
    rationale: "The EC gate verifies the offline spec and llm CLI surfaces used by agents."
---

# External Contracts: lumen

## Agentic Integration Offline CLI EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-agentic-integration-offline-cli
    capability_id: agentic-integration
    contract_id: offline-cli-agent-onboarding
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "lumen spec emits valid OpenAPI and JSON-schema output offline."
      - "lumen spec exposes query-shape, field, analyzer, and vector-metric catalogs."
      - "lumen llm guide, quickstart, and recipes preserve the ingest-search-hydrate agent workflow and non-goals."
```
