import type {
  AdminReviewTicket,
  AppArtifact,
  AppSpec,
  GovernanceCheck,
  OwnershipNamespace,
  Persona,
  RuntimeTenant,
  StoryState,
  StudioTab,
} from './types'
import { getCopy, type GovernanceCopy } from './i18n'

export const seedSpec: AppSpec = {
  schema_version: 'cue.app-spec.v0',
  app: {
    id: 'team-request-tracker',
    name: 'Team Request Tracker',
    description: 'Tracks internal support requests from intake through completion.',
    goal: {
      statement: 'Reduce dropped internal support requests.',
      problem: 'Requests arrive through chat and spreadsheets, so owners lose context and managers cannot see aging work.',
      success_metrics: [
        {
          name: 'median_resolution_time',
          baseline: '5 business days',
          target: '2 business days',
          window: '90 days',
        },
      ],
      review_policy: {
        cadence: 'monthly',
        owner_required: true,
      },
    },
    owner_team: 'Operations',
    owner_user: 'ops-lead@example.com',
    risk_tier: 'tier_1',
    lifecycle_status: 'draft',
  },
  users: {
    target_users: [
      { team: 'Operations', role: 'Request manager' },
      { team: 'All employees', role: 'Requester' },
    ],
  },
  data: {
    entities: [
      {
        name: 'request',
        fields: [
          { name: 'title', type: 'string', required: true, sensitivity: 'internal' },
          {
            name: 'status',
            type: 'enum',
            required: true,
            sensitivity: 'internal',
            enum_values: ['new', 'triaged', 'in_progress', 'done', 'cancelled'],
          },
          { name: 'owner', type: 'user', required: false, sensitivity: 'internal' },
          { name: 'due_date', type: 'date', required: false, sensitivity: 'internal' },
        ],
      },
    ],
    data_sources: [],
  },
  permissions: {
    roles: [
      {
        name: 'Requester',
        permissions: [
          { action: 'create', scope: 'entity', resource: 'request', condition: null },
          { action: 'view', scope: 'record', resource: 'request', condition: 'created_by == current_user' },
        ],
      },
      {
        name: 'Request manager',
        permissions: [
          { action: 'view', scope: 'app', condition: null },
          { action: 'edit', scope: 'entity', resource: 'request', condition: null },
        ],
      },
    ],
  },
  workflow: {
    states: [
      { name: 'New', type: 'start' },
      { name: 'Triaged', type: 'intermediate' },
      { name: 'In Progress', type: 'intermediate' },
      { name: 'Done', type: 'terminal' },
      { name: 'Cancelled', type: 'terminal' },
    ],
    transitions: [
      { from: 'New', to: 'Triaged', condition: null, actor_role: 'Request manager' },
      { from: 'Triaged', to: 'In Progress', condition: 'owner is set', actor_role: 'Request manager' },
      { from: 'In Progress', to: 'Done', condition: null, actor_role: 'Request manager' },
    ],
  },
  automation: {
    triggers: [
      {
        type: 'state_change',
        condition: "status == 'done'",
        actions: [{ type: 'notify', target: 'created_by' }],
      },
    ],
  },
  dashboard: {
    views: [
      {
        name: 'Operations overview',
        metrics: [
          { name: 'Open requests', formula: "count(request where status != 'done')" },
          { name: 'Overdue requests', formula: "count(request where due_date < today and status != 'done')" },
        ],
      },
    ],
  },
  audit: {
    enabled: true,
    retention_days: 365,
    log_events: ['record_created', 'record_updated', 'permission_changed', 'deployment_changed'],
  },
  tests: {
    required_tests: ['permission_test', 'workflow_test', 'data_validation_test', 'policy_test'],
  },
  deployment: {
    environments: ['sandbox', 'production'],
    approval_required: ['app_owner'],
  },
}

export const runtimeTenant: RuntimeTenant = {
  runtime_tenant_id: 'rt-team-request-tracker-sandbox-v1',
  app_id: 't-operations-team-request-tracker',
  environment: 'sandbox',
  app_version: 1,
  owner_namespace: 'team',
  owner_user: 'ops-lead@example.com',
  owner_team: 'Operations',
  storage: {
    backend: 'postgresql',
    local_backend: 'postgresql',
    cluster_id: 'local-dev-postgres',
    cluster_mode: 'shared',
    isolation_unit: 'database',
    database_name: 'cue_app_t_operations_team_request_tracker_sandbox',
    schema_name: 'app',
    backup_required: false,
  },
  retention: {
    policy_id: 'sandbox_30d_resettable',
    archive_after_days: 30,
    retire_after_days: 60,
    erase_supported: true,
  },
  migration: {
    state: 'current',
    last_checked_at: '2026-05-08T00:00:00Z',
  },
  runtime_families: [
    'record',
    'comment',
    'attachment',
    'workflow_state',
    'dashboard_materialization',
    'usage_metric',
    'runtime_audit',
  ],
}

export const ownershipNamespace: OwnershipNamespace = {
  app_id: 't-operations-team-request-tracker',
  namespace: 'team',
  display_name: 'Team Request Tracker',
  owner: {
    owner_user: 'ops-lead@example.com',
    owner_team: 'Operations',
    data_owner: null,
  },
  platform_owner: 'cue-platform@example.com',
  emergency_contact: 'cue-oncall@example.com',
  quota_policy: 'team_default',
  transfer_policy: 'manager_required',
  visibility: 'team_visible',
  orphan_state: 'healthy',
  quota_state: {
    sandbox_apps_count: 3,
    production_apps_count: 1,
    active_exceptions: [],
  },
  gitlab_mapping: {
    root_group: 'cue-generated-apps',
    group_path: 'teams/operations',
    project_path: 't-operations-team-request-tracker',
    full_path: 'cue-generated-apps/teams/operations/t-operations-team-request-tracker',
    user_visible: false,
  },
}

export const defaultPrompt = getCopy().mock.defaultPrompt

export const storySteps: Array<{ state: StoryState }> = [
  { state: 'PromptDraft' },
  { state: 'Clarifying' },
  { state: 'SpecDraft' },
  { state: 'StudioPreview' },
  { state: 'SandboxReady' },
  { state: 'ProductionRequested' },
  { state: 'ProductionReady' },
]

export const studioTabs: StudioTab[] = ['fields', 'workflow', 'permissions', 'notifications', 'dashboard']

export const clarificationItems = [
  { id: 'owner' },
  { id: 'fields' },
  { id: 'roles' },
  { id: 'runtime' },
  { id: 'approval' },
] as const

export type ClarificationId = (typeof clarificationItems)[number]['id']
export type ClarificationState = Record<ClarificationId, boolean>

export const initialClarifications: ClarificationState = {
  owner: true,
  fields: true,
  roles: true,
  runtime: false,
  approval: true,
}

export const sampleRequests = [
  { id: 'REQ-2041', title: 'Vendor access review', owner: 'Mina', status: 'Triaged', age: '1d' },
  { id: 'REQ-2040', title: 'Quarterly deck data pull', owner: 'Jon', status: 'In Progress', age: '2d' },
  { id: 'REQ-2038', title: 'Office move checklist', owner: 'Unassigned', status: 'New', age: '4h' },
]

export const deliveryTeam = [
  { id: 'pm', state: 'pass' },
  { id: 'designer', state: 'pass' },
  { id: 'dev', state: 'review' },
  { id: 'data', state: 'review' },
  { id: 'qaPolicy', state: 'review' },
] as const

export const personaOptions: Persona[] = ['owner', 'platform']

export const appArtifacts: AppArtifact[] = [
  {
    id: 'app-spec',
    label: 'App Spec',
    detail: 'cue.app-spec.v0 / request entity / workflow policy',
    status: 'ready',
    visibility: 'owner',
  },
  {
    id: 'ui-proposal',
    label: 'UI proposal',
    detail: 'table, field editor, approval action, dashboard summary',
    status: 'review',
    visibility: 'owner',
  },
  {
    id: 'permission-model',
    label: 'Permission model',
    detail: 'Requester + Request manager record access',
    status: 'ready',
    visibility: 'owner',
  },
  {
    id: 'sandbox-release',
    label: 'Sandbox release',
    detail: 'sample data, resettable tenant, production request gate',
    status: 'draft',
    visibility: 'owner',
  },
  {
    id: 'gitlab-project',
    label: 'Hidden GitLab project',
    detail: 'cue-generated-apps/teams/operations/t-operations-team-request-tracker',
    status: 'ready',
    visibility: 'platform',
  },
  {
    id: 'runtime-tenant',
    label: 'Runtime tenant binding',
    detail: 'shared cluster / app database isolation / schema app',
    status: 'ready',
    visibility: 'platform',
  },
  {
    id: 'policy-test',
    label: 'Policy and test pack',
    detail: 'permission, workflow, data validation, policy regression',
    status: 'review',
    visibility: 'platform',
  },
]

export const adminReviewTickets: AdminReviewTicket[] = [
  {
    id: 'ticket-deploy-test-001',
    title: 'Deploy test build and publish Sandbox result',
    kind: 'deployment_test',
    status: 'pending',
    workspace_id: 'ws-operations',
    app_id: ownershipNamespace.app_id,
    requested_by: 'Cue agent team',
    resource: 'sandbox deployment + test result publication',
    risk: 'medium',
    environment_scope: 'sandbox runtime only',
    data_scope: 'sample request rows and generated fixtures',
    agent_output: 'App Spec, UI preview, permission tests, workflow tests, and sandbox release candidate',
    rationale: 'Agent finished implementation and needs Admin review before the test deployment is published to the owner.',
  },
  {
    id: 'ticket-saas-api-001',
    title: 'Enable Slack notification API',
    kind: 'saas_api',
    status: 'pending',
    workspace_id: 'ws-operations',
    app_id: ownershipNamespace.app_id,
    requested_by: 'Operations / ops-lead@example.com',
    resource: 'Slack Web API / chat:write',
    risk: 'high',
    environment_scope: 'sandbox first, production after policy approval',
    data_scope: 'request title owner status and due date notification payload',
    agent_output: 'Connector proposal, permission scope, event trigger, and rollback plan',
    rationale: 'Owner wants request status changes to notify assigned managers in Slack.',
  },
  {
    id: 'ticket-train-model-001',
    title: 'Training model for request triage',
    kind: 'resource_budget',
    status: 'pending',
    workspace_id: 'ws-operations',
    app_id: ownershipNamespace.app_id,
    requested_by: 'Operations / ops-lead@example.com',
    resource: 'train_model / GPU budget',
    risk: 'high',
    environment_scope: 'sandbox first, production after admin grant',
    data_scope: 'request metadata and de-identified historical status labels',
    agent_output: 'Model plan, feature list, training data boundary, cost estimate, and evaluation policy',
    rationale: 'Agent team needs a classifier to suggest owner and priority for incoming requests.',
  },
]

export function buildDraftSpec(prompt: string, fallbackPrompt = defaultPrompt): AppSpec {
  const promptSummary = prompt.trim() || fallbackPrompt
  const lower = promptSummary.toLowerCase()
  const appName = lower.includes('triage') || promptSummary.includes('分流') || promptSummary.includes('分派')
    ? 'Operations Triage Tracker'
    : lower.includes('incident') || promptSummary.includes('事故') || promptSummary.includes('事件')
      ? 'Incident Response Tracker'
      : lower.includes('approval') || promptSummary.includes('簽核') || promptSummary.includes('核准')
        ? 'Governed Approval Tracker'
        : 'Team Request Tracker'

  return {
    ...seedSpec,
    app: {
      ...seedSpec.app,
      name: appName,
      description: promptSummary,
      goal: {
        ...seedSpec.app.goal,
        statement: `Govern ${appName.toLowerCase()} from intake through approved release.`,
        problem: promptSummary,
      },
      lifecycle_status: 'draft',
    },
  }
}

export function getGovernanceChecks(spec: AppSpec, copy: GovernanceCopy = getCopy().governance): GovernanceCheck[] {
  const hasOwner = spec.app.owner_team.length > 0 && spec.app.owner_user.length > 0
  const hasAudit = spec.audit.enabled && spec.audit.log_events.length > 0
  const hasApprover = spec.deployment.approval_required.length > 0
  const isBlockedTier = spec.app.risk_tier === 'tier_4'

  return [
    {
      label: copy.labels.spec,
      detail: `${spec.schema_version}, ${copy.details.governedEntity(spec.data.entities.length)}`,
      state: spec.schema_version === 'cue.app-spec.v0' ? 'pass' : 'block',
    },
    {
      label: copy.labels.ownership,
      detail: hasOwner ? `${spec.app.owner_team} / ${spec.app.owner_user}` : copy.details.missingOwner,
      state: hasOwner ? 'pass' : 'block',
    },
    {
      label: copy.labels.risk,
      detail: isBlockedTier ? copy.details.riskBlocked : copy.details.sandboxEligible(spec.app.risk_tier),
      state: isBlockedTier ? 'block' : 'pass',
    },
    {
      label: copy.labels.runtimeData,
      detail: copy.details.runtimeData,
      state: 'pass',
    },
    {
      label: copy.labels.production,
      detail: hasApprover ? copy.details.approvalGate(spec.deployment.approval_required.length) : copy.details.noApprover,
      state: hasApprover ? 'review' : 'block',
    },
    {
      label: copy.labels.audit,
      detail: hasAudit ? copy.details.auditRetention(spec.audit.retention_days) : copy.details.auditDisabled,
      state: hasAudit ? 'pass' : 'block',
    },
  ]
}
