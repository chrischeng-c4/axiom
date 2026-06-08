export type CueWorkspaceName =
  | 'artifact_studio'
  | 'admin'
  | 'backend'
  | 'shared'
  | 'schemas'
  | 'examples'
  | 'legacy_docs'

export type CueWorkspaceAudience = 'project_owner' | 'platform_operator' | 'developer' | 'generated_app_runtime'

export type CueWorkspaceRole =
  | 'frontend_site'
  | 'api_service'
  | 'shared_contracts'
  | 'contract_store'
  | 'fixture_store'
  | 'history_only'

export type CueWorkspace = {
  name: CueWorkspaceName
  path: string
  audience: CueWorkspaceAudience
  role: CueWorkspaceRole
}

export type CueWorkAudience = 'owner' | 'operator' | 'backend' | 'contract' | 'legacy'

export function routeCueWork(audience: CueWorkAudience): CueWorkspaceName {
  if (audience === 'owner') return 'artifact_studio'
  if (audience === 'operator') return 'admin'
  if (audience === 'backend') return 'backend'
  if (audience === 'contract') return 'shared'
  return 'legacy_docs'
}
