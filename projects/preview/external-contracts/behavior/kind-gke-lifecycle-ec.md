---
id: kind-gke-lifecycle-ec
summary: External contract for Kind/GKE lifecycle EC.
fill_sections: [e2e-test]
---

# EC: Kind/GKE lifecycle EC

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: kind-gke-lifecycle-ec
    capability_id: preview-external-contracts
    claim_id: kind-gke-lifecycle-ec
    contract_id: kind-gke-lifecycle-ec
    category: behavior
    command: "PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture"
    assertions:
      - "When a kind/GKE kubectl context is configured, preview apply performs direct apply, server-side dry-run after namespace creation, idempotent re-apply, rollout, endpoint checks, and /readyz port-forward smoke."
      - "The kind/GKE gate creates a base Deployment/Service fixture and runs preview discover-base before rendering the preview namespace."
      - "The kind/GKE gate validates namespace-local workload RBAC and rejects an oversized pod through ResourceQuota/LimitRange admission."
      - "The kind/GKE gate cleans temporary preview/control namespaces after success or failure."
      - "Without a configured kubectl context, the test reports an explicit skip instead of falsely applying to an unknown cluster."
```
