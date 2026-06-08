---
number: 934
title: "feat(sdd): deployment spec type — infra-as-code integration"
state: open
labels: [enhancement, P2, crate:sdd]
group: "new-spec-types"
---

# #934 — feat(sdd): deployment spec type — infra-as-code integration

## Summary

SDD covers code specs (API, schema, logic, state machine) but has zero coverage for deployment concerns. When a change adds an API endpoint or background worker, the deployment impact (new service, config change, scaling, migration) is not captured in any spec type. This creates a gap between "code works" and "code is deployed correctly."

## Current Gap

No SDD section type covers:
- Container / service definitions (Dockerfile, docker-compose, k8s manifests)
- Infrastructure changes (new DB, new queue, new cache)
- Environment variable requirements
- Migration ordering (DB migration before code deploy)
- Rollback strategy
- Resource requirements (CPU, memory, replicas)

## Proposal

### New section type: `deploy`

```yaml
section_rules:
  - match: "deploy|container|k8s|docker|helm|terraform|infra|migration|rollback"
    sections: [deploy]
```

### Deploy section spec lang: YAML DSL

```yaml
_sdd:
  id: deploy-auth-service
  refs:
    - $ref: "#auth-api"
deploy:
  services:
    - name: auth-service
      image: cclab/auth:{{version}}
      replicas: 2
      env:
        - JWT_SECRET (secret)
        - DATABASE_URL (config)
      ports: [8080]
      health_check: /health
  migrations:
    - type: sql
      path: migrations/20260318_add_auth_tables.sql
      run_before: deploy  # migration ordering
  rollback:
    strategy: blue-green
    auto_rollback_on: health_check_failure
```

### Integration points

- `changes` section already lists file changes — `deploy` adds operational context
- Cross-ref to `db-model` section for migration validation
- Cross-ref to `rest-api` section for port/route validation
- Future: generate Helm chart / docker-compose from deploy spec

## Acceptance Criteria

- [ ] `deploy` section type added to section type system
- [ ] CLI flags defined for deploy section
- [ ] Validation: referenced services/migrations are consistent
- [ ] At least one codegen target (docker-compose or k8s manifest)
