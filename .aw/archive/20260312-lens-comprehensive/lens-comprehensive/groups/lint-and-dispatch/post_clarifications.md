---
change: lens-comprehensive
group: lint-and-dispatch
date: 2026-03-12
status: clarified
---

# Post-Clarifications

## Questions

### Q1: No additional questions
- **Question**: Any spec contradictions needing user input?
- **Answer**: No. The only deviation (YamlDispatcher vs separate Language variants) was already resolved in pre-clarifications.

## Contradictions

### C1: lens-lang-support vs requirement
- **Spec**: lens-lang-support
- **Requirement**: YamlDispatcher composite checker
- **Conflict**: Spec R8/R9 assumed separate Language variants for K8s and GitLab CI, but implementation uses single Language::Yaml with content-based dispatch
- **Resolution**: YamlDispatcher approach is correct — single Language::Yaml with content detection is cleaner than adding Language::KubernetesYaml and Language::GitlabCiYaml variants

