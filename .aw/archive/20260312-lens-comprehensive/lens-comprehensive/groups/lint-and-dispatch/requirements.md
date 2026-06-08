---
change: lens-comprehensive
group: lint-and-dispatch
date: 2026-03-12
---

# Requirements

1. YamlDispatcher: composite checker that routes Language::Yaml to KubernetesChecker (apiVersion+kind) or GitlabCiChecker (.gitlab-ci.yml) based on content/filename detection. Replace single KubernetesChecker entry in CheckerRegistry.
2. Expand lint rules: add ~20 missing rules across Dockerfile (DK004/DK006/DK010), Terraform (TF002/TF003/TF007/TF009/TF010), Kubernetes (K8002/K8005/K8008-K8010), GitLab CI (GL002/GL005/GL006/GL009-GL012), Python security (PY301-PY305).
3. Bundle JSON schemas: K8s API schemas for 1.28/1.29/1.30 and GitLab CI schema via include_bytes!() for offline validation. Add SchemaRegistry struct. Wire into K8s and GitLab CI checkers for structural validation.
