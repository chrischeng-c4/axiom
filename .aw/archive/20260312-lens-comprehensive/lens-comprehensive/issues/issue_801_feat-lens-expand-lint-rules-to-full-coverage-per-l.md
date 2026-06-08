---
number: 801
title: "feat(lens): expand lint rules to full coverage per language"
state: open
labels: [enhancement, P2, crate:lens]
group: "lint-and-dispatch"
---

# #801 — feat(lens): expand lint rules to full coverage per language

## Context

Spec `lens-lang-support` defined 10+ rules per language, but MVP implemented 5-7 each. This issue tracks expanding to full coverage.

## Missing rules by language

### Dockerfile (3 missing)
- DK004: COPY/ADD `--chown` best practices
- DK006: Missing HEALTHCHECK
- DK010: Missing .dockerignore reference

### Terraform (5 missing)
- TF002: Deprecated resource attributes
- TF003: Missing required attributes (needs provider schema)
- TF007: Missing `required_providers`
- TF009: Missing tags on taggable resources
- TF010: S3 bucket without encryption/versioning

### Kubernetes (5 missing)
- K8002: Missing required fields (needs JSON schema)
- K8005: Missing liveness/readiness probes
- K8008: Deprecated API versions
- K8009: Duplicate resource names
- K8010: Kustomization referencing non-existent files

### GitLab CI (7 missing)
- GL002: Unknown job keywords
- GL005: `needs` referencing non-existent job
- GL006: Circular `needs` dependencies (cycle detection)
- GL009: Missing timeout on long-running jobs
- GL010: `allow_failure` without `when:manual`
- GL011: Unused `extends` templates
- GL012: Invalid `include` references

### Python (potential additions)
- PY301-PY305: Security rules (eval, exec, pickle, subprocess shell)
- PY801-PY805: Docstring rules (missing, format, params)

## Total: ~20 new rules across 5 languages
