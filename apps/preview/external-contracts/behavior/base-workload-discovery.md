---
id: base-workload-discovery
summary: External contract for local base workload discovery and normalization.
fill_sections: [e2e-test]
---

# EC: Base workload discovery

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: base-workload-discovery
    capability_id: gke-uat-preview-environment-rendering
    claim_id: base-workload-discovery
    contract_id: base-workload-discovery
    category: behavior
    command: "cargo test -p preview --test base_discovery_contract"
    assertions:
      - "Base discovery normalizes Kubernetes Deployment and Service JSON fixtures into a BaseWorkloadContract."
      - "Discovery preserves cloneable selector, port, env, probe, and resource fields while excluding runtime identity and cluster-assigned fields."
      - "Discovery refuses ambiguous multi-container Deployments without a container matching the requested app."
      - "Render can consume a discovered base contract and embed it in plans/workload-clone.json without a live cluster."
```
