export type RiskTier = 'tier_0' | 'tier_1' | 'tier_2' | 'tier_3' | 'tier_4'
export type LifecycleStatus = 'draft' | 'sandbox' | 'production' | 'archived' | 'retired'
export type StoryState =
  | 'PromptDraft'
  | 'Clarifying'
  | 'SpecDraft'
  | 'StudioPreview'
  | 'SandboxReady'
  | 'ProductionRequested'
  | 'ProductionReady'

export type RouteKey = 'new' | 'studio' | 'sandbox' | 'registry' | 'admin'
export type StudioTab = 'fields' | 'workflow' | 'permissions' | 'notifications' | 'dashboard'
export type Persona = 'owner' | 'platform'
export type ArtifactStatus = 'draft' | 'ready' | 'review' | 'blocked'
export type ArtifactVisibility = 'owner' | 'platform'
export type WorkspaceKind = 'user' | 'admin'
export type AdminReviewTicketStatus = 'pending' | 'approved'
export type AdminReviewTicketKind = 'deployment_test' | 'saas_api' | 'resource_budget' | 'capability'

export type AppArtifact = {
  id: string
  label: string
  detail: string
  status: ArtifactStatus
  visibility: ArtifactVisibility
}

export type AppSpec = {
  schema_version: string
  app: {
    id: string
    name: string
    description: string
    goal: {
      statement: string
      problem: string
      success_metrics: Array<{
        name: string
        baseline: string | number | null
        target: string | number
        window: string
      }>
      review_policy: {
        cadence: string
        owner_required: true
      }
    }
    owner_team: string
    owner_user: string
    risk_tier: RiskTier
    lifecycle_status: LifecycleStatus
  }
  users: {
    target_users: Array<{ team: string; role: string }>
  }
  data: {
    entities: Array<{
      name: string
      fields: Array<{
        name: string
        type: string
        required: boolean
        sensitivity: string
        enum_values?: string[]
      }>
    }>
    data_sources: Array<unknown>
  }
  permissions: {
    roles: Array<{
      name: string
      permissions: Array<{
        action: string
        scope: string
        resource?: string
        condition: string | null
      }>
    }>
  }
  workflow: {
    states: Array<{ name: string; type: string }>
    transitions: Array<{
      from: string
      to: string
      condition: string | null
      actor_role: string
    }>
  }
  automation: {
    triggers: Array<{
      type: string
      condition: string
      actions: Array<{ type: string; target: string }>
    }>
  }
  dashboard: {
    views: Array<{
      name: string
      metrics: Array<{ name: string; formula: string }>
    }>
  }
  audit: {
    enabled: boolean
    retention_days: number
    log_events: string[]
  }
  tests: {
    required_tests: string[]
  }
  deployment: {
    environments: string[]
    approval_required: string[]
  }
}

export type OwnershipNamespace = {
  app_id: string
  namespace: string
  display_name: string
  owner: {
    owner_user: string
    owner_team: string
    data_owner: string | null
  }
  platform_owner: string
  emergency_contact: string
  quota_policy: string
  transfer_policy: string
  visibility: string
  orphan_state: string
  quota_state: {
    sandbox_apps_count: number
    production_apps_count: number
    active_exceptions: string[]
  }
  gitlab_mapping: {
    root_group: string
    group_path: string
    project_path: string
    full_path: string
    user_visible: boolean
  }
}

export type RuntimeTenant = {
  runtime_tenant_id: string
  app_id: string
  environment: string
  app_version: number
  owner_namespace: string
  owner_user: string
  owner_team: string
  storage: {
    backend: string
    local_backend: string
    cluster_id: string
    cluster_mode: string
    isolation_unit: string
    database_name: string
    schema_name: string
    backup_required: boolean
  }
  retention: {
    policy_id: string
    archive_after_days: number
    retire_after_days: number
    erase_supported: boolean
  }
  migration: {
    state: string
    last_checked_at: string
  }
  runtime_families: string[]
}

export type GovernanceCheck = {
  label: string
  detail: string
  state: 'pass' | 'review' | 'block'
}

export type AdminReviewTicket = {
  id: string
  title: string
  kind: AdminReviewTicketKind
  status: AdminReviewTicketStatus
  workspace_id: string
  app_id: string
  requested_by: string
  resource: string
  risk: 'medium' | 'high'
  environment_scope: string
  data_scope: string
  agent_output: string
  rationale: string
}
