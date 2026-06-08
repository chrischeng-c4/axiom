---
change: sdd-codegen-completion
group: deploy-section-type
date: 2026-03-20
---

# Requirements

Add a `deploy` section type to the SDD section type system. Register section_rules keyword matchers for deploy|container|k8s|docker|helm|terraform|infra|migration|rollback. Implement the section type matcher and define the YAML DSL schema (services with image/replicas/env/ports/health_check, migrations with type/path/run_before, rollback with strategy/auto_rollback_on). Implement k8s validation rules (referenced services/migrations are internally consistent). Deliver at least one codegen target: docker-compose YAML or k8s Deployment+Service manifests. Cross-ref validation: deploy section referencing db-model must validate migration paths; deploy referencing rest-api must validate port/route consistency.
