---
id: kubernetes-object-ec
summary: External contract for Kubernetes object EC.
fill_sections: [e2e-test]
---

# EC: Kubernetes object EC

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: kubernetes-object-ec
    capability_id: preview-external-contracts
    claim_id: kubernetes-object-ec
    contract_id: kubernetes-object-ec
    category: behavior
    command: "cargo test -p preview --test k8s_object_contract"
    assertions:
      - "Rendered Kubernetes YAML parses as Namespace, ServiceAccount, ResourceQuota, LimitRange, Role, RoleBinding, Deployment, Service, and ConfigMap objects."
      - "Service selectors match Deployment pod labels."
      - "Rendered Deployment includes readiness/liveness probes, explicit Workload Identity service account, bounded resources, and base workload clone annotations."
```
