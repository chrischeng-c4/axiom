export type CueSessionPrincipal = {
  id: string
  displayName: string
  role: 'project_owner' | 'platform_operator'
}

export type CueSessionBoundary = {
  authMode: 'placeholder'
  principal: CueSessionPrincipal
  apiBasePath: '/api'
}

export const ownerSessionBoundary: CueSessionBoundary = {
  authMode: 'placeholder',
  principal: {
    id: 'owner-placeholder',
    displayName: 'Project Owner',
    role: 'project_owner',
  },
  apiBasePath: '/api',
}

export const adminSessionBoundary: CueSessionBoundary = {
  authMode: 'placeholder',
  principal: {
    id: 'admin-placeholder',
    displayName: 'Platform Operator',
    role: 'platform_operator',
  },
  apiBasePath: '/api',
}
