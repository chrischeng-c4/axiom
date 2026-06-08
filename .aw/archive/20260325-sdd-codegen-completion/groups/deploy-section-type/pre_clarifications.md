---
change: sdd-codegen-completion
group: deploy-section-type
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: First codegen target: docker-compose vs k8s
- **Answer**: k8s Deployment + Service manifests. Already partially implemented in DeployGenerator from previous change.

### Q2: General
- **Question**: Deploy YAML DSL finalization
- **Answer**: The DSL from the proposal is final for this change. Resource limits already supported in DeploySpec. No Helm/Terraform needed.

### Q3: General
- **Question**: Cross-ref validation strictness
- **Answer**: Soft warning — proceed with best-effort output. Cross-ref validation deferred to later iteration.

