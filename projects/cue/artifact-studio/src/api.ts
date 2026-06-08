export type ProjectStatus = 'needs-review' | 'in-progress' | 'ready' | 'blocked'
export type ArtifactStage = 'done' | 'in-progress' | 'not-started' | 'blocked' | 'ready'
export type WorkItemState = 'collecting' | 'accepted' | 'drafting' | 'blocked' | 'done'
export type QcStatus = 'pass' | 'needs_input' | 'blocked' | 'pending'
export type AgentRole = 'pm' | 'architect' | 'designer' | 'dev' | 'data' | 'qa_policy' | 'release'

export type QcCheck = {
  id: string
  label: string
  status: QcStatus
  summary: string
}

export type WorkflowStep = {
  id: 'prd' | 'td' | 'website' | string
  label: string
  state: ArtifactStage
  depends_on?: string[]
  agent_role?: AgentRole | string
  agent_label?: string
  agent_task?: string
}

export type Stage = WorkflowStep & {
  detail: string
}

export type Message = {
  id: string
  speaker: 'cue' | 'owner'
  body: string
  action?: string
  created_at?: string
}

export type Session = {
  id: string
  project_id: string
  title: string
  messages: Message[]
}

export type WorkItem = {
  id: string
  project_id: string
  title: string
  route: string
  target: 'WorkItem' | 'PRD' | 'TD' | 'Website' | 'Artifact' | string
  state: WorkItemState
  progress: number
  next_action: string
  blockers: string[]
  workflow_plan: WorkflowStep[]
  qc_status: QcStatus
  qc_checks: QcCheck[]
  prompt?: string
  missing_fields?: string[]
  risk_hints?: string[]
  target_artifact_type?: string
  artifact_id?: string | null
  updated_at?: string
}

export type Artifact = {
  id: string
  workitem_id?: string
  label: string
  kind: string
  status: string
  summary?: string
  qc_status?: QcStatus
  qc_checks?: QcCheck[]
  entrypoints?: string[]
  versions?: Array<{
    id: string
    version: number
    status: string
  }>
}

export type OperationsDashboard = {
  health: string
  latest_release: string
  ci_status: string
  deployment_environment: string
  open_blockers: string[]
  runtime_metrics: {
    todo_count: number
    completed_count: number
    error_rate: number
  }
}

export type PermissionSimulation = {
  role: string
  allowed_actions: string[]
  field_access: Array<{
    entity?: string
    field?: string
    visible: boolean
    editable: boolean
    masked: boolean
  }>
  approval_requirements: string[]
}

export type AppSpecDiff = {
  changed: boolean
  categories: Record<string, Array<{ path: string; old: unknown; new: unknown }>>
}

export type AppSpecPreview = {
  form_preview: Array<{ entity?: string; fields: unknown[] }>
  table_preview: Array<{ entity?: string; columns: unknown[] }>
  workflow_preview: unknown[]
  permission_editor: Record<string, string[]>
  notification_editor: unknown[]
  dashboard_preview: unknown[]
  goal_preview?: {
    statement?: string | null
    success_metrics?: Array<{ name: string; baseline?: string | number | null; target: string | number; window: string }>
    review_policy?: { cadence: string; owner_required: boolean } | null
  }
}

export type ProjectGoal = {
  version: number
  statement: string
  problem: string
  target_users: string[]
  success_metrics: Array<{ name: string; baseline?: string | number | null; target: string | number; window: string; current?: string | number | null }>
  review_policy: { cadence: string; owner_required: boolean }
  updated_at?: string
  updated_by?: string
}

export type GoalStatus = {
  status: 'missing' | 'needs_review' | 'at_risk' | 'on_track' | string
  reason: string
  review_due: boolean
}

export type Project = {
  id: string
  name: string
  owner: string
  owner_namespace?: 'personal' | 'team' | 'cross_team' | 'platform'
  risk_tier?: 'tier_0' | 'tier_1' | 'tier_2' | 'tier_3' | 'tier_4'
  lifecycle_status?: 'draft' | 'active' | 'blocked' | 'sandbox' | 'production' | 'archived' | 'retired'
  current_workstream_id?: string | null
  status: ProjectStatus
  next_action: string
  summary: string
  active_session_id: string
  sessions: Session[]
  stages: Stage[]
  workitems: WorkItem[]
  artifacts: Artifact[]
  operations_dashboard?: OperationsDashboard
  goal?: ProjectGoal
  goal_status?: GoalStatus
  goal_metrics?: ProjectGoal['success_metrics']
  admin_evidence?: {
    hidden_repo_path?: string
    local_gitlab_simulation?: boolean
    local_gcp_simulation?: boolean
  }
}

export type WorkItemContext = {
  type: 'project_overview' | 'workflow_plan' | 'artifact' | 'blockers'
  project_id: string
  workitem?: WorkItem
  workflow_plan?: WorkflowStep[]
  artifact_gates?: Array<{
    kind: string
    unlocked: boolean
    gate: string
    reason: string
  }>
  artifacts?: Artifact[]
  blockers?: string[]
  qc_status?: QcStatus
  qc_checks?: QcCheck[]
  next_action?: string
  next_artifact_kind?: string | null
}

type ProjectsResponse = {
  projects: Project[]
}

type MessageResponse = {
  classification: 'project_work' | 'general_chat_redirect'
  project: Project
  session: Session
  workitem?: WorkItem
  context: WorkItemContext
  message: Message
}

type ArtifactRunResponse = {
  status: 'created' | 'rejected'
  reason?: string
  message?: string
  qc_result?: {
    status: QcStatus
    checks: QcCheck[]
  }
  project: Project
  context: WorkItemContext
}

async function requestJson<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...(init?.headers ?? {}),
    },
  })
  if (!response.ok) {
    throw new Error(`Cue API request failed: ${response.status} ${path}`)
  }
  return response.json() as Promise<T>
}

export async function fetchProjects() {
  const response = await requestJson<ProjectsResponse>('/api/projects')
  return response.projects
}

export async function createProject() {
  const response = await requestJson<{ project: Project }>('/api/projects', {
    method: 'POST',
    body: JSON.stringify({
      name: 'Todo App Project',
      prompt: 'Create a todo app for the Operations team.',
    }),
  })
  return response.project
}

export async function postSessionMessage(sessionId: string, content: string) {
  return requestJson<MessageResponse>(`/api/sessions/${sessionId}/messages`, {
    method: 'POST',
    body: JSON.stringify({ content }),
  })
}

export async function fetchWorkItemContext(workItemId: string) {
  return requestJson<WorkItemContext>(`/api/workitems/${workItemId}/context`)
}

export async function runArtifact(workItemId: string, kind: string) {
  return requestJson<ArtifactRunResponse>(`/api/workitems/${workItemId}/artifact-runs`, {
    method: 'POST',
    body: JSON.stringify({ kind }),
  })
}

export async function simulatePermissions(spec: unknown, role: string) {
  const response = await requestJson<{ ok: boolean; data: PermissionSimulation }>('/api/admin/app-spec/permissions/simulate', {
    method: 'POST',
    body: JSON.stringify({ spec, role }),
  })
  return response.data
}

export async function diffAppSpec(oldSpec: unknown, newSpec: unknown) {
  const response = await requestJson<{ ok: boolean; data: AppSpecDiff }>('/api/admin/app-spec/diff', {
    method: 'POST',
    body: JSON.stringify({ old: oldSpec, new: newSpec }),
  })
  return response.data
}

export async function previewAppSpec(spec: unknown) {
  const response = await requestJson<{ ok: boolean; data: AppSpecPreview }>('/api/admin/app-spec/preview', {
    method: 'POST',
    body: JSON.stringify({ spec }),
  })
  return response.data
}
