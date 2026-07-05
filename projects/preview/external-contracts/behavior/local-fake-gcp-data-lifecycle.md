---
id: local-fake-gcp-data-lifecycle
summary: External contract for local fake-GCP data lifecycle planning, Secret rewrite, and guarded cleanup.
fill_sections: [e2e-test]
---

# EC: Local fake-GCP data lifecycle

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: local-fake-gcp-data-lifecycle
    capability_id: gke-uat-preview-environment-rendering
    claim_id: local-fake-gcp-data-lifecycle
    contract_id: local-fake-gcp-data-lifecycle
    category: behavior
    command: "cargo test -p preview --test local_cicd_contract local_data_plan_fake_provider_and_secret_rewrite_are_deterministic"
    assertions:
      - "`preview render --data-*` emits plans/data-plan.json and k8s/data-secret.yaml only when a data contract is supplied."
      - "The data plan models a fake GCP Cloud SQL preview resource with read-only source, preview-* target naming, TTL, and ownership guardrails."
      - "The rendered Deployment rewrites DATABASE_URL to the namespace-local preview database Secret."
      - "`preview data apply` and `preview data cleanup` mutate only fake provider state and are idempotent for local CI."
```
