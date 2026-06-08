---
number: 802
title: "feat(lens): add symbol tables for new languages (Dockerfile, HCL, K8s, GitLab CI)"
state: open
labels: [enhancement, P2, crate:lens]
group: "symbol-tables"
---

# #802 — feat(lens): add symbol tables for new languages (Dockerfile, HCL, K8s, GitLab CI)

## Context

New language checkers (#767-#771) lack symbol table builders. This blocks hover, go-to-definition, and references for these languages.

## Symbol tables needed

### Dockerfile
- FROM stages (multi-stage build names)
- ENV variables
- EXPOSE ports
- LABEL keys
- ARG declarations

### Terraform/HCL
- Resources (`resource "aws_s3_bucket" "my_bucket"`)
- Data sources
- Variables and outputs
- Locals
- Modules

### Kubernetes YAML
- Resources (name + kind + namespace)
- Labels and selectors
- Service → Deployment references
- ConfigMap/Secret references

### GitLab CI
- Jobs
- Stages
- Variables (global + per-job)
- Templates (extends targets)
- Include references

## Existing pattern to follow
- `semantic/symbols/python.rs` — Python symbol extractor
- `semantic/symbols/typescript.rs` — TypeScript symbol extractor
- `semantic/symbols/rust.rs` — Rust symbol extractor

## Files to create
- `crates/cclab-lens/src/semantic/symbols/javascript.rs`
- `crates/cclab-lens/src/semantic/symbols/dockerfile.rs`
- `crates/cclab-lens/src/semantic/symbols/terraform.rs`
- `crates/cclab-lens/src/semantic/symbols/kubernetes.rs`
- `crates/cclab-lens/src/semantic/symbols/gitlab_ci.rs`
