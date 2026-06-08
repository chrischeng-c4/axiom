---
id: cue-governance-contract-v0
summary: Governance contract for Cue App Spec lifecycle, risk tiers, production approvals, registry snapshots, and audit events.
fill_sections: [state-machine, schema, logic, changes, tests]
---

# Cue Governance Contract v0

Status: implemented

## Lifecycle
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: cue-governance-lifecycle
initial: Draft
nodes:
  Draft: { kind: initial, label: Draft }
  SpecValidated: { kind: normal, label: SpecValidated }
  SandboxDeployed: { kind: normal, label: SandboxDeployed }
  PendingApproval: { kind: normal, label: PendingApproval }
  ProductionDeployed: { kind: normal, label: ProductionDeployed }
  Maintained: { kind: normal, label: Maintained }
  Disabled: { kind: normal, label: Disabled }
  Archived: { kind: normal, label: Archived }
  Retired: { kind: terminal, label: Retired }
edges:
  - { from: Draft, to: SpecValidated, event: validate_spec }
  - { from: SpecValidated, to: SandboxDeployed, event: deploy_sandbox }
  - { from: SandboxDeployed, to: PendingApproval, event: request_production }
  - { from: PendingApproval, to: ProductionDeployed, event: approve_production }
  - { from: PendingApproval, to: SandboxDeployed, event: reject_production }
  - { from: ProductionDeployed, to: Maintained, event: observe_normal_use }
  - { from: Maintained, to: ProductionDeployed, event: release_update }
  - { from: Maintained, to: Archived, event: archive }
  - { from: Archived, to: Retired, event: retire }
  - { from: ProductionDeployed, to: Disabled, event: emergency_disable }
  - { from: Disabled, to: ProductionDeployed, event: restore }
  - { from: Disabled, to: Archived, event: archive_after_incident }
---
```

## Governance Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "https://cclab.dev/cue/governance-contract/v0"
title: Cue Governance Contract v0
type: object
additionalProperties: false
required: [transition_rules, risk_tiers, approval_request, registry_snapshot, audit_event]
properties:
  transition_rules:
    type: object
    additionalProperties: false
    required:
      - validate_spec
      - deploy_sandbox
      - request_production
      - approve_production
      - reject_production
      - release_update
      - emergency_disable
    properties:
      validate_spec:
        type: object
        properties:
          from: { const: Draft }
          to: { const: SpecValidated }
          requires:
            type: array
            items: { enum: [app_spec_schema_valid, owner_user_present, owner_team_present, target_users_present, at_least_one_entity] }
      deploy_sandbox:
        type: object
        properties:
          from: { const: SpecValidated }
          to: { const: SandboxDeployed }
          requires:
            type: array
            items: { enum: [risk_tier_assigned, policy_check_passed, required_tests_passed, tier_4_blocklist_not_matched] }
      request_production:
        type: object
        properties:
          from: { const: SandboxDeployed }
          to: { const: PendingApproval }
          requires:
            type: array
            items: { enum: [app_owner_requested, sandbox_test_run_passed, production_approvers_resolved] }
      approve_production:
        type: object
        properties:
          from: { const: PendingApproval }
          to: { const: ProductionDeployed }
          requires:
            type: array
            items: { enum: [all_required_approvals_granted, no_open_policy_blockers, deployment_plan_current] }
      reject_production:
        type: object
        properties:
          from: { const: PendingApproval }
          to: { const: SandboxDeployed }
          requires:
            type: array
            items: { enum: [at_least_one_required_approver_rejected] }
      release_update:
        type: object
        properties:
          from: { const: Maintained }
          to: { const: ProductionDeployed }
          requires:
            type: array
            items: { enum: [app_spec_version_incremented, regression_tests_passed, approval_delta_resolved] }
      emergency_disable:
        type: object
        properties:
          from: { const: ProductionDeployed }
          to: { const: Disabled }
          requires:
            type: array
            items: { enum: [security_or_platform_operator, incident_reason_recorded] }
  risk_tiers:
    type: object
    additionalProperties: false
    properties:
      tier_0: { type: object, properties: { name: { const: Personal Utility }, mvp_behavior: { const: sandbox_only } } }
      tier_1: { type: object, properties: { name: { const: Team Productivity }, mvp_behavior: { const: app_owner_approval } } }
      tier_2: { type: object, properties: { name: { const: Cross-functional Workflow }, mvp_behavior: { const: app_owner_plus_conditional_data_owner } } }
      tier_3: { type: object, properties: { name: { const: Operational Critical }, mvp_behavior: { const: platform_security_review } } }
      tier_4:
        type: object
        properties:
          name: { const: High-risk / Regulated }
          mvp_behavior: { const: blocked }
          blocklist:
            type: array
            items:
              enum:
                - automatic_refund
                - payment_update
                - order_state_write
                - inventory_write
                - price_change
                - member_benefit_change
                - unrestricted_personal_data_export
                - external_legal_document_send
                - external_contract_document_send
  approval_request:
    type: object
    additionalProperties: false
    required: [id, app_id, app_version, risk_tier, requested_by, required_approvers, status]
    properties:
      id: { type: string, format: uuid }
      app_id: { type: string }
      app_version: { type: integer, minimum: 1 }
      risk_tier: { enum: [tier_0, tier_1, tier_2, tier_3, tier_4] }
      requested_by: { type: string, minLength: 1 }
      required_approvers:
        type: array
        items:
          type: object
          additionalProperties: false
          required: [kind, principal, decision]
          properties:
            kind: { enum: [app_owner, team_manager, data_owner, security, legal] }
            principal: { type: string, minLength: 1 }
            decision: { enum: [pending, approved, rejected] }
            comment: { type: ["string", "null"] }
      status: { enum: [pending, approved, rejected, cancelled] }
  registry_snapshot:
    type: object
    additionalProperties: false
    required: [app_id, app_version, owner_team, risk_tier, lifecycle_status, health]
    properties:
      app_id: { type: string }
      app_version: { type: integer, minimum: 1 }
      name: { type: string }
      owner_team: { type: string }
      owner_user: { type: string }
      risk_tier: { enum: [tier_0, tier_1, tier_2, tier_3, tier_4] }
      lifecycle_status: { enum: [sandbox, production, disabled, archived, retired] }
      deployment_environment: { enum: [sandbox, production] }
      health:
        type: object
        required: [status, last_test_run_status, open_policy_findings]
        properties:
          status: { enum: [healthy, degraded, failing, disabled] }
          last_test_run_status: { enum: [passed, failed, skipped] }
          open_policy_findings: { type: integer, minimum: 0 }
  audit_event:
    type: object
    additionalProperties: false
    required: [actor, app_id, app_version, event_type, before, after, created_at]
    properties:
      actor: { type: string }
      app_id: { type: string }
      app_version: { type: integer, minimum: 1 }
      event_type:
        enum:
          - app_spec_created
          - app_spec_validated
          - risk_tier_assigned
          - policy_check_completed
          - sandbox_deployed
          - production_requested
          - approval_decision_recorded
          - production_deployed
          - app_disabled
          - app_restored
          - app_archived
          - app_retired
          - permission_changed
          - connector_access_changed
      before: { type: ["object", "null"] }
      after: { type: ["object", "null"] }
      created_at: { type: string, format: date-time }
```

## Transition Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cue-governance-transition-logic
entry: EvaluateTransition
nodes:
  EvaluateTransition: { kind: start, label: evaluate requested transition }
  ValidatePreconditions: { kind: process, label: validate required facts }
  Tier4Blocked: { kind: decision, label: matches Tier 4 blocklist? }
  RejectTransition: { kind: terminal, label: reject or escalate }
  ApprovalRequired: { kind: decision, label: approval required? }
  OpenApprovalRequest: { kind: process, label: create approval request }
  ApplyTransition: { kind: process, label: update lifecycle state }
  WriteAudit: { kind: process, label: write audit event }
  RegistryRequired: { kind: decision, label: registry snapshot required? }
  WriteRegistry: { kind: process, label: write registry snapshot }
  Done: { kind: terminal, label: transition complete }
edges:
  - { from: EvaluateTransition, to: ValidatePreconditions, label: request }
  - { from: ValidatePreconditions, to: Tier4Blocked, label: valid }
  - { from: Tier4Blocked, to: RejectTransition, label: yes }
  - { from: Tier4Blocked, to: ApprovalRequired, label: no }
  - { from: ApprovalRequired, to: OpenApprovalRequest, label: yes }
  - { from: ApprovalRequired, to: ApplyTransition, label: no }
  - { from: OpenApprovalRequest, to: ApplyTransition, label: approved }
  - { from: ApplyTransition, to: WriteAudit, label: state updated }
  - { from: WriteAudit, to: RegistryRequired, label: audit saved }
  - { from: RegistryRequired, to: WriteRegistry, label: yes }
  - { from: RegistryRequired, to: Done, label: no }
  - { from: WriteRegistry, to: Done, label: snapshot saved }
---
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: .aw/tech-design/projects/cue/governance-contract.md
    action: modify
    impl_mode: hand-written
    description: Convert governance contract to current TD section format and preserve lifecycle, risk tier, approval, registry, and audit semantics.
  - path: projects/cue/schemas/
    action: modify
    impl_mode: hand-written
    description: Add or extend governance-related schemas when implementation needs executable validation.
  - path: projects/cue/backend/src/
    action: modify
    impl_mode: hand-written
    description: Implement governance transition validation, risk-tier blocklist checks, approval request state, registry snapshot writes, and audit event writes.
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  lifecycle_transition_contract:
    kind: unit
    verifies: [valid transition advances state, invalid transition is rejected]
  tier_4_blocklist_contract:
    kind: unit
    verifies: [blocked actions do not reach sandbox or production]
  approval_request_schema:
    kind: schema
    verifies: [required approvers, decision enum, status enum]
  registry_snapshot_required:
    kind: unit
    verifies: [production, maintained, disabled, archived, retired transitions write registry snapshots]
  audit_event_required:
    kind: unit
    verifies: [every transition writes actor, app_id, app_version, event_type, before, after, created_at]
```
