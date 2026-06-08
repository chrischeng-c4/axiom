---
number: 1051
title: "epic: SDD section type coverage — all roles (FE/BE/SRE/MLE/Agent/QA/Security)"
state: open
labels: [priority:p2, type:epic]
group: "new-section-types"
---

# #1051 — epic: SDD section type coverage — all roles (FE/BE/SRE/MLE/Agent/QA/Security)

## Goal

`codebase = f(spec, tech-stack)` — every role's artifacts should be expressible as SDD section types, enabling `skeleton = codegen(spec, tech-stack)` followed by AI fill.

## Current 17 Section Types

`overview`, `rest-api`, `rpc-api`, `async-api`, `cli`, `schema`, `logic`, `interaction`, `state-machine`, `db-model`, `dependency`, `test-plan`, `wireframe`, `component`, `design-token`, `config`, `changes`

## Coverage by Role

### Frontend — well-covered
- With design system (MUI/Antd) as part of `tech_stack`, `wireframe` alone may suffice
- `design-token` and `component` become optional when design system handles them
- UX pattern library (SDD-side, design-system-agnostic) — deferred

### Backend — 2 gaps
- ❌ `grpc` — Protobuf IDL (different from OpenRPC)
- ❌ `graphql` — GraphQL SDL

### SRE — largest gap
- ❌ `container` — Dockerfile/compose (yaml DSL)
- ❌ `deploy` — K8s/Cloud Run scaling+networking (yaml DSL)
- ❌ `cloud-resource` — Terraform managed services (yaml DSL)
- ❌ `pipeline` — CI/CD DAG (yaml DSL)
- ❌ `observability` — Prometheus rules, SLO, dashboards (yaml)
- ❌ `runbook` — structured operational steps (markdown)
- Design principle: split by abstraction layer, NOT one mega `infra` type

### MLE — notable gaps
- ❌ `pipeline` — shared with SRE (data/ML pipeline DAG)
- ❌ `model` — model architecture, layer definition, tensor shapes
- ❌ `feature-schema` / data contract — ML-specific I/O (beyond `schema`)
- ❌ `experiment` — metrics, A/B test criteria, hyperparams

### Agent — mostly covered
- ⚠️ `prompt` — currently treated as `logic`, but prompts have variables, system instructions, few-shot examples

### QA — 3 gaps
- ❌ `e2e-scenario` — user journey steps + assertions → skeleton generates Playwright/Cypress test
- ❌ `test-fixture` — mock data schema, test datasets (`schema` partially covers)
- ❌ `perf-test` — load profile, RPS target, latency SLO

### Security — 3 gaps
- ❌ `threat-model` — attack surface, trust boundaries, STRIDE classification
- ❌ `auth-matrix` — RBAC/ABAC role × resource × action matrix → skeleton generates middleware/guards
- ❌ `security-test` — OWASP top 10 checks, injection/XSS/CSRF scenarios

## Frontend Direction

- Design system = part of `tech_stack` config, not spec
- `ux_patterns: true` (MUI) → thin wireframe, generator uses built-in recipes
- `ux_patterns: false` (Antd) → wireframe describes layout explicitly
- SDD-side UX pattern library (deferred) — `layout: dashboard-with-drawer` works across any component library

## Approach

Add section types incrementally — implement when a real change needs them. Each new type needs:
1. Section type definition (lang, code fence)
2. Section rule match keywords
3. Fill order position
4. CLI generator flags
5. (Optional) codegen generator for skeleton output
