---
number: 798
title: "fix(lens): YAML dispatcher — route K8s vs GitLab CI by content detection"
state: open
labels: [bug, P1, crate:lens]
group: "lint-and-dispatch"
---

# #798 — fix(lens): YAML dispatcher — route K8s vs GitLab CI by content detection

## Problem

Both `KubernetesChecker` and `GitlabCiChecker` target `Language::Yaml`, but `CheckerRegistry` uses `HashMap<Language, Box<dyn Checker>>` — only one checker per language key. Currently only `KubernetesChecker` is registered; `GitlabCiChecker` is compiled but never dispatched.

## Solution

Introduce a `YamlDispatcher` composite checker that delegates based on content:
- `.gitlab-ci.yml` filename → `GitlabCiChecker`
- YAML with `apiVersion:` + `kind:` → `KubernetesChecker`
- Other YAML → skip or basic YAML lint

## Files to modify

- `crates/cclab-lens/src/lint/mod.rs` — Add `YamlDispatcher`, register it for `Language::Yaml`
- `crates/cclab-lens/src/lint/kubernetes.rs` — Make `is_k8s_manifest` public
- `crates/cclab-lens/src/lint/gitlab_ci.rs` — Make `is_gitlab_ci` public

## Acceptance criteria

- [ ] `.gitlab-ci.yml` files get GL* diagnostics
- [ ] K8s manifests get K8* diagnostics
- [ ] Non-K8s/non-GitLab YAML files produce no diagnostics
