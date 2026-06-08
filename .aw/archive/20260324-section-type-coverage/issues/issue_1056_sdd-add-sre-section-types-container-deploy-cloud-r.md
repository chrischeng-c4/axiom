---
number: 1056
title: "sdd: add SRE section types — container, deploy, cloud-resource, pipeline, observability"
state: open
labels: [type:enhancement, priority:p3, crate:sdd]
group: "new-section-types"
---

# #1056 — sdd: add SRE section types — container, deploy, cloud-resource, pipeline, observability

Parent: #1051

## SRE section types (deferred — add when encountered)

Design principle: split by abstraction layer, NOT one mega `infra` type. Each uses tool-agnostic YAML DSL.

### Types

| type | describes | skeleton output |
|------|-----------|-----------------|
| `container` | Image build/runtime — base image, ports, env vars | Dockerfile, docker-compose service |
| `deploy` | Orchestration — replicas, scaling, health check, networking | K8s manifests, Cloud Run config |
| `cloud-resource` | Managed services — DB, queue, storage, IAM | Terraform modules |
| `pipeline` | CI/CD + data/ML DAG — stages, triggers, dependencies | GitHub Actions, Airflow DAG |
| `observability` | Monitoring — alert rules, SLO, dashboards | Prometheus rules, Grafana JSON |

### Also considered (lower priority)
- `runbook` — structured operational steps (markdown) — `overview` can substitute
