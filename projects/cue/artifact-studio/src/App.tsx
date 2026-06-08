import { type ChangeEvent, type FormEvent, useEffect, useMemo, useState } from 'react'
import AddRoundedIcon from '@mui/icons-material/AddRounded'
import ArrowForwardRoundedIcon from '@mui/icons-material/ArrowForwardRounded'
import AssignmentTurnedInOutlinedIcon from '@mui/icons-material/AssignmentTurnedInOutlined'
import ChatBubbleOutlineRoundedIcon from '@mui/icons-material/ChatBubbleOutlineRounded'
import CheckCircleOutlineRoundedIcon from '@mui/icons-material/CheckCircleOutlineRounded'
import RadioButtonUncheckedRoundedIcon from '@mui/icons-material/RadioButtonUncheckedRounded'
import SearchRoundedIcon from '@mui/icons-material/SearchRounded'
import ScheduleRoundedIcon from '@mui/icons-material/ScheduleRounded'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import CssBaseline from '@mui/material/CssBaseline'
import Divider from '@mui/material/Divider'
import InputAdornment from '@mui/material/InputAdornment'
import LinearProgress from '@mui/material/LinearProgress'
import List from '@mui/material/List'
import ListItemButton from '@mui/material/ListItemButton'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { type Theme, ThemeProvider, createTheme } from '@mui/material/styles'
import {
  fetchProjects,
  fetchWorkItemContext,
  createProject,
  diffAppSpec,
  previewAppSpec,
  postSessionMessage,
  runArtifact,
  simulatePermissions,
  type AppSpecDiff,
  type AppSpecPreview,
  type ArtifactStage,
  type PermissionSimulation,
  type Project,
  type ProjectStatus,
  type QcStatus,
  type Session,
  type WorkflowStep,
  type WorkItemContext,
  type WorkItemState,
} from './api'

type ChipColor = 'default' | 'primary' | 'secondary' | 'error' | 'info' | 'success' | 'warning'

const previewRoles = ['owner', 'editor', 'reviewer', 'finance', 'legal', 'manager', 'viewer']

const density = {
  shellPad: { xs: 1.25, md: 1.5 },
  panelPad: 1,
  rowGap: 0.75,
  cardGap: 0.75,
  iconTrack: '20px',
  text: {
    xs: 10.5,
    sm: 11,
    md: 12,
    lg: 13,
  },
}

const theme = createTheme({
  palette: {
    primary: { main: '#1f6aa7', dark: '#174f7c' },
    success: { main: '#287a45' },
    warning: { main: '#c17619' },
    error: { main: '#b13b3b' },
    background: { default: '#f5f7f9', paper: '#ffffff' },
    text: { primary: '#20242c', secondary: '#667085' },
    divider: '#d9e0e8',
  },
  shape: { borderRadius: 8 },
  typography: {
    fontFamily:
      '"Noto Sans TC", Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    h1: { fontSize: 20, lineHeight: 1.2, fontWeight: 850, letterSpacing: 0 },
    h2: { fontSize: 16, lineHeight: 1.25, fontWeight: 850, letterSpacing: 0 },
    button: { fontWeight: 850, letterSpacing: 0, textTransform: 'none' },
  },
  components: {
    MuiButton: {
      defaultProps: { size: 'small' },
      styleOverrides: {
        root: { minHeight: 30, borderRadius: 8, boxShadow: 'none', whiteSpace: 'nowrap', paddingBlock: 4 },
      },
    },
    MuiChip: {
      defaultProps: { size: 'small' },
      styleOverrides: {
        root: { borderRadius: 999, fontSize: 11, fontWeight: 850, height: 20 },
        label: { paddingInline: 7 },
      },
    },
    MuiTextField: {
      defaultProps: { size: 'small' },
    },
    MuiListItemButton: {
      styleOverrides: {
        root: { borderRadius: 8 },
      },
    },
  },
})

function alpha(hex: string, opacity: number) {
  const normalized = hex.replace('#', '')
  const value = Number.parseInt(normalized, 16)
  const red = (value >> 16) & 255
  const green = (value >> 8) & 255
  const blue = value & 255
  return `rgba(${red}, ${green}, ${blue}, ${opacity})`
}

const statusVisuals: Record<ProjectStatus, { label: string; color: ChipColor }> = {
  'needs-review': { label: 'Waiting for you', color: 'warning' },
  'in-progress': { label: 'In progress', color: 'info' },
  ready: { label: 'Ready to try', color: 'success' },
  blocked: { label: 'Waiting for approval', color: 'error' },
}

const statusDotColors: Record<ProjectStatus, string> = {
  'needs-review': '#c17619',
  'in-progress': '#1f6aa7',
  ready: '#287a45',
  blocked: '#b13b3b',
}

const stageVisuals: Record<ArtifactStage, { label: string; color: ChipColor; tone: string }> = {
  done: { label: 'Done', color: 'success', tone: '#287a45' },
  'in-progress': { label: 'In progress', color: 'info', tone: '#1f6aa7' },
  ready: { label: 'Ready', color: 'primary', tone: '#1f6aa7' },
  'not-started': { label: 'Not started', color: 'default', tone: '#8a94a6' },
  blocked: { label: 'Blocked', color: 'error', tone: '#b13b3b' },
}

const workItemVisuals: Record<WorkItemState, { label: string; color: ChipColor; tone: string }> = {
  collecting: { label: 'Needs info', color: 'warning', tone: '#c17619' },
  accepted: { label: 'Accepted', color: 'success', tone: '#287a45' },
  drafting: { label: 'Drafting', color: 'info', tone: '#1f6aa7' },
  blocked: { label: 'Waiting', color: 'error', tone: '#b13b3b' },
  done: { label: 'Done', color: 'success', tone: '#287a45' },
}

const qcVisuals: Record<QcStatus, { label: string; color: ChipColor; tone: string }> = {
  pass: { label: 'QC pass', color: 'success', tone: '#287a45' },
  needs_input: { label: 'QC needs input', color: 'warning', tone: '#c17619' },
  blocked: { label: 'QC blocked', color: 'error', tone: '#b13b3b' },
  pending: { label: 'QC pending', color: 'info', tone: '#1f6aa7' },
}

function StageIcon(props: { stage: ArtifactStage }) {
  if (props.stage === 'done') return <CheckCircleOutlineRoundedIcon aria-hidden="true" />
  if (props.stage === 'in-progress' || props.stage === 'ready') return <ScheduleRoundedIcon aria-hidden="true" />
  return <RadioButtonUncheckedRoundedIcon aria-hidden="true" />
}

function replaceProject(projects: Project[], nextProject: Project) {
  return projects.map((project) => (project.id === nextProject.id ? nextProject : project))
}

function recentSessions(project: Project) {
  return project.sessions.slice(-3).reverse()
}

function workflowAgentLabel(step: { agent_label?: string; agent_role?: string }) {
  if (step.agent_label) return step.agent_label
  if (!step.agent_role) return null
  return `${step.agent_role.replace('_', '/')} agent`
}

function workflowDependencyLabel(step: WorkflowStep) {
  const dependsOn = step.depends_on ?? []
  if (dependsOn.length === 0) return 'Starts from WorkItem'
  return `After ${dependsOn.join(', ')}`
}

function projectPreviewSpec(project: Project) {
  return {
    schema_version: 'cue.app-spec.v0',
    app_id: project.id,
    name: project.name,
    owner_team: project.owner,
    owner_user: 'owner@example.com',
    lifecycle_status: project.lifecycle_status ?? 'draft',
    risk_tier: project.risk_tier ?? 'tier_1',
    goal: project.goal
      ? {
          statement: project.goal.statement,
          problem: project.goal.problem,
          success_metrics: project.goal.success_metrics,
          review_policy: project.goal.review_policy,
        }
      : undefined,
    permissions: {
      roles: {
        owner: ['read', 'write', 'request_production', 'view_sensitive'],
        editor: ['read', 'write'],
        reviewer: ['read'],
        finance: ['read', 'view_sensitive'],
        legal: ['read', 'export'],
        manager: ['read', 'approve'],
        viewer: ['read'],
      },
    },
    entities: [
      {
        id: 'request',
        fields: [
          { id: 'requester', sensitivity: 'normal' },
          { id: 'status', sensitivity: 'normal' },
          { id: 'amount', sensitivity: 'sensitive' },
        ],
      },
    ],
  }
}

export function App() {
  const [projects, setProjects] = useState<Project[]>([])
  const [activeId, setActiveId] = useState<string>('')
  const [activeSessionId, setActiveSessionId] = useState<string>('')
  const [activeWorkItemId, setActiveWorkItemId] = useState<string>('')
  const [context, setContext] = useState<WorkItemContext | null>(null)
  const [query, setQuery] = useState('')
  const [draft, setDraft] = useState('')
  const [showWorkItems, setShowWorkItems] = useState(false)
  const [previewRole, setPreviewRole] = useState('owner')
  const [permissionPreview, setPermissionPreview] = useState<PermissionSimulation | null>(null)
  const [appSpecPreviewData, setAppSpecPreviewData] = useState<AppSpecPreview | null>(null)
  const [specDiff, setSpecDiff] = useState<AppSpecDiff | null>(null)
  const [loading, setLoading] = useState(true)
  const [sending, setSending] = useState(false)
  const [runningArtifact, setRunningArtifact] = useState(false)
  const [creatingProject, setCreatingProject] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    setLoading(true)
    fetchProjects()
      .then((nextProjects) => {
        if (cancelled) return
        setProjects(nextProjects)
        setActiveId(nextProjects[0]?.id ?? '')
        setActiveSessionId(nextProjects[0]?.active_session_id ?? nextProjects[0]?.sessions[0]?.id ?? '')
        setActiveWorkItemId(nextProjects[0]?.workitems[0]?.id ?? '')
        setError(null)
      })
      .catch((reason: Error) => {
        if (!cancelled) setError(reason.message)
      })
      .finally(() => {
        if (!cancelled) setLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [])

  const filteredProjects = useMemo(() => {
    const normalized = query.trim().toLowerCase()
    if (!normalized) return projects
    return projects.filter((project) => project.name.toLowerCase().includes(normalized))
  }, [projects, query])

  const activeProject = projects.find((project) => project.id === activeId) ?? projects[0]
  const activeSession =
    activeProject?.sessions.find((session) => session.id === activeSessionId) ??
    activeProject?.sessions.find((session) => session.id === activeProject.active_session_id) ??
    activeProject?.sessions[0]
  const activeWorkItem = activeProject?.workitems.find((workItem) => workItem.id === activeWorkItemId) ?? activeProject?.workitems[0]

  useEffect(() => {
    if (!activeProject) return
    const belongsToProject = activeProject.workitems.some((workItem) => workItem.id === activeWorkItemId)
    if (!belongsToProject) {
      setActiveWorkItemId(activeProject.workitems[0]?.id ?? '')
    }
  }, [activeProject, activeWorkItemId])

  useEffect(() => {
    if (!activeProject) return
    const belongsToProject = activeProject.sessions.some((session) => session.id === activeSessionId)
    if (!belongsToProject) {
      setActiveSessionId(activeProject.active_session_id || activeProject.sessions[0]?.id || '')
    }
  }, [activeProject, activeSessionId])

  useEffect(() => {
    let cancelled = false
    if (!activeProject) {
      setContext(null)
      return
    }
    if (!activeWorkItem?.id) {
      setContext({ type: 'project_overview', project_id: activeProject.id, next_action: activeProject.next_action })
      return
    }
    fetchWorkItemContext(activeWorkItem.id)
      .then((nextContext) => {
        if (!cancelled) setContext(nextContext)
      })
      .catch(() => {
        if (!cancelled) {
          setContext({
            type: 'workflow_plan',
            project_id: activeProject.id,
            workitem: activeWorkItem,
            workflow_plan: activeWorkItem.workflow_plan,
            blockers: activeWorkItem.blockers,
            next_action: activeWorkItem.next_action,
          })
        }
      })
    return () => {
      cancelled = true
    }
  }, [activeProject, activeWorkItem])

  useEffect(() => {
    let cancelled = false
    if (!activeProject) {
      setPermissionPreview(null)
      setAppSpecPreviewData(null)
      setSpecDiff(null)
      return
    }
    const baseSpec = projectPreviewSpec(activeProject)
    const editedSpec = {
      ...baseSpec,
      risk_tier: activeProject.risk_tier === 'tier_2' ? 'tier_3' : 'tier_2',
      permissions: {
        roles: {
          ...baseSpec.permissions.roles,
          editor: ['read', 'write', 'export'],
        },
      },
    }
    Promise.all([simulatePermissions(editedSpec, previewRole), diffAppSpec(baseSpec, editedSpec), previewAppSpec(editedSpec)])
      .then(([nextPreview, nextDiff, nextSpecPreview]) => {
        if (cancelled) return
        setPermissionPreview(nextPreview)
        setSpecDiff(nextDiff)
        setAppSpecPreviewData(nextSpecPreview)
      })
      .catch(() => {
        if (!cancelled) {
          setPermissionPreview(null)
          setAppSpecPreviewData(null)
          setSpecDiff(null)
        }
      })
    return () => {
      cancelled = true
    }
  }, [activeProject, previewRole])

  const activeStatus = activeProject ? statusVisuals[activeProject.status] : statusVisuals['in-progress']
  const completedWorkItems =
    activeProject?.workitems.filter((workItem) => workItem.state === 'accepted' || workItem.state === 'done').length ?? 0

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!activeSession || !draft.trim()) return
    setSending(true)
    try {
      const response = await postSessionMessage(activeSession.id, draft.trim())
      setProjects((current) => replaceProject(current, response.project))
      setActiveId(response.project.id)
      setActiveSessionId(response.session.id)
      setActiveWorkItemId(response.workitem?.id ?? response.context.workitem?.id ?? activeWorkItem?.id ?? '')
      setContext(response.context)
      setDraft('')
      setError(null)
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : 'Failed to send message')
    } finally {
      setSending(false)
    }
  }

  async function handleCreateProject() {
    setCreatingProject(true)
    try {
      const project = await createProject()
      setProjects((current) => {
        const exists = current.some((candidate) => candidate.id === project.id)
        return exists ? replaceProject(current, project) : [project, ...current]
      })
      setActiveId(project.id)
      setActiveSessionId(project.active_session_id || project.sessions[0]?.id || '')
      setActiveWorkItemId(project.workitems[0]?.id ?? '')
      setContext({ type: 'project_overview', project_id: project.id, next_action: project.next_action })
      setError(null)
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : 'Failed to create project')
    } finally {
      setCreatingProject(false)
    }
  }

  function handleSelectProject(projectId: string) {
    const project = projects.find((candidate) => candidate.id === projectId)
    setActiveId(projectId)
    setActiveSessionId(project?.active_session_id || project?.sessions[0]?.id || '')
  }

  function handleSelectSession(projectId: string, sessionId: string) {
    setActiveId(projectId)
    setActiveSessionId(sessionId)
  }

  async function handleRunArtifact() {
    if (!activeWorkItem || !context?.next_artifact_kind) return
    setRunningArtifact(true)
    try {
      const response = await runArtifact(activeWorkItem.id, context.next_artifact_kind)
      setProjects((current) => replaceProject(current, response.project))
      setContext(response.context)
      setError(response.status === 'rejected' ? response.message ?? 'Artifact run rejected' : null)
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : 'Failed to run artifact')
    } finally {
      setRunningArtifact(false)
    }
  }

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box
        component="main"
        sx={{
          display: 'grid',
          gridTemplateColumns: {
            xs: '1fr',
            md: '210px minmax(0, 1fr)',
            lg: '230px minmax(360px, 0.82fr) minmax(460px, 1.18fr)',
          },
          minHeight: '100vh',
          bgcolor: 'background.default',
          color: 'text.primary',
        }}
      >
        <ProjectList
          activeId={activeId}
          activeSessionId={activeSession?.id ?? activeSessionId}
          filteredProjects={filteredProjects}
          loading={loading}
          query={query}
          onQueryChange={setQuery}
          onCreateProject={handleCreateProject}
          onSelectProject={handleSelectProject}
          onSelectSession={handleSelectSession}
          creatingProject={creatingProject}
        />

        <Box
          component="section"
          aria-label="Project conversation"
          sx={{
            display: 'grid',
            gridTemplateRows: 'auto auto minmax(0, 1fr) auto',
            minWidth: 0,
            minHeight: '100vh',
            maxHeight: { xs: 'none', md: '100vh' },
            overflow: 'hidden',
            p: density.shellPad,
          }}
        >
          {!activeProject ? (
            <EmptyCenter loading={loading} error={error} />
          ) : (
            <>
              <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1} sx={{ alignItems: { xs: 'stretch', sm: 'flex-start' }, justifyContent: 'space-between', pb: 1.1 }}>
                <Box sx={{ minWidth: 0 }}>
                  <Typography variant="caption" sx={{ color: 'text.secondary', fontSize: density.text.sm, fontWeight: 850, textTransform: 'uppercase' }}>
                    {activeProject.owner}
                  </Typography>
                  <Typography variant="h1" sx={{ mt: 0.5, overflowWrap: 'anywhere' }}>
                    {activeProject.name}
                  </Typography>
                </Box>
                <Chip color={activeStatus.color} label={activeStatus.label} variant="outlined" sx={{ alignSelf: { xs: 'flex-start', sm: 'center' } }} />
              </Stack>

              <Stack direction="row" spacing={1} sx={{ alignItems: 'center', border: 1, borderColor: 'divider', borderRadius: 1, bgcolor: 'background.paper', p: density.panelPad }}>
                <ChatBubbleOutlineRoundedIcon color="primary" fontSize="small" aria-hidden="true" />
                <Box sx={{ minWidth: 0 }}>
                  <Typography sx={{ fontSize: density.text.lg, fontWeight: 850 }}>Project workstream</Typography>
                  <Typography sx={{ color: 'text.secondary', fontSize: density.text.md, lineHeight: 1.35, overflowWrap: 'anywhere' }}>
                    {activeProject.summary}
                  </Typography>
                </Box>
              </Stack>

              <Stack spacing={density.cardGap} sx={{ alignContent: 'start', minHeight: 0, overflow: 'auto', py: 1 }}>
                {(activeSession?.messages ?? []).map((message) => {
                  const isOwner = message.speaker === 'owner'
                  return (
                    <Box
                      component="article"
                      key={message.id}
                      sx={{
                        width: 'fit-content',
                        maxWidth: { xs: '100%', md: 680, lg: 620 },
                        justifySelf: isOwner ? 'end' : 'start',
                        alignSelf: isOwner ? 'flex-end' : 'flex-start',
                        border: 1,
                        borderColor: isOwner ? '#cad7df' : 'divider',
                        borderRadius: 1,
                        bgcolor: isOwner ? '#eef3f5' : 'background.paper',
                        p: density.panelPad,
                      }}
                    >
                      <Typography variant="caption" sx={{ color: 'text.secondary', fontSize: density.text.sm, fontWeight: 850 }}>
                        {isOwner ? 'You' : 'Cue'}
                      </Typography>
                      <Typography sx={{ mt: 0.35, fontSize: density.text.lg, lineHeight: 1.45, overflowWrap: 'anywhere' }}>{message.body}</Typography>
                      {message.action && (
                        <Button variant="outlined" size="small" type="button" endIcon={<ArrowForwardRoundedIcon />} sx={{ mt: 1.5 }}>
                          {message.action}
                        </Button>
                      )}
                    </Box>
                  )
                })}
              </Stack>

              <Stack
                component="form"
                direction={{ xs: 'column', sm: 'row' }}
                spacing={1.25}
                onSubmit={handleSubmit}
                sx={{
                  borderTop: 1,
                  borderColor: 'divider',
                  bgcolor: 'background.default',
                  pt: 1,
                }}
              >
                <TextField
                  fullWidth
                  value={draft}
                  onChange={(event: ChangeEvent<HTMLInputElement>) => setDraft(event.target.value)}
                  placeholder="Describe a request, answer a WorkItem question, or ask for an artifact"
                />
                <Button variant="contained" type="submit" disabled={sending || !draft.trim()}>
                  {sending ? 'Sending' : 'Send'}
                </Button>
              </Stack>
            </>
          )}
        </Box>

        <ContextPanel
          activeProject={activeProject}
          activeWorkItemId={activeWorkItem?.id ?? ''}
          completedWorkItems={completedWorkItems}
          context={context}
          error={error}
          runningArtifact={runningArtifact}
          showWorkItems={showWorkItems}
          onRunArtifact={handleRunArtifact}
          onSelectPreviewRole={setPreviewRole}
          onSelectWorkItem={setActiveWorkItemId}
          onToggleWorkItems={() => setShowWorkItems((value) => !value)}
          appSpecPreview={appSpecPreviewData}
          permissionPreview={permissionPreview}
          previewRole={previewRole}
          specDiff={specDiff}
        />
      </Box>
    </ThemeProvider>
  )
}

function ProjectList(props: {
  activeId: string
  activeSessionId: string
  creatingProject: boolean
  filteredProjects: Project[]
  loading: boolean
  query: string
  onCreateProject: () => void
  onQueryChange: (value: string) => void
  onSelectProject: (id: string) => void
  onSelectSession: (projectId: string, sessionId: string) => void
}) {
  return (
    <Box
      component="aside"
      aria-label="Projects"
      sx={{ minWidth: 0, minHeight: { xs: 'auto', md: '100vh' }, borderRight: { xs: 0, md: 1 }, borderBottom: { xs: 1, md: 0 }, borderColor: 'divider', bgcolor: 'background.paper', p: density.shellPad }}
    >
      <Stack direction="row" spacing={0.75} sx={{ alignItems: 'center', px: 0.25, pb: 1 }}>
        <Box sx={{ display: 'grid', width: 26, height: 26, placeItems: 'center', borderRadius: 1, bgcolor: 'primary.main', color: 'common.white', fontSize: density.text.md, fontWeight: 900 }}>
          C
        </Box>
        <Box sx={{ minWidth: 0 }}>
          <Typography component="strong" sx={{ display: 'block', fontSize: density.text.lg, fontWeight: 850 }}>
            Artifact Studio
          </Typography>
          <Typography variant="caption" sx={{ color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750 }}>
            Project workspace
          </Typography>
        </Box>
      </Stack>

      <Button fullWidth variant="contained" startIcon={<AddRoundedIcon />} type="button" disabled={props.creatingProject} onClick={props.onCreateProject}>
        {props.creatingProject ? 'Creating' : 'New Project'}
      </Button>

      <TextField
        fullWidth
        size="small"
        value={props.query}
        onChange={(event: ChangeEvent<HTMLInputElement>) => props.onQueryChange(event.target.value)}
        placeholder="Search projects"
        sx={{ my: 1 }}
        slotProps={{
          input: {
            startAdornment: (
              <InputAdornment position="start">
                <SearchRoundedIcon fontSize="small" aria-hidden="true" />
              </InputAdornment>
            ),
          },
        }}
      />

      {props.loading ? (
        <Stack direction="row" spacing={1} sx={{ alignItems: 'center', p: 1 }}>
          <CircularProgress size={18} />
          <Typography sx={{ color: 'text.secondary', fontSize: 13, fontWeight: 750 }}>Loading projects</Typography>
        </Stack>
      ) : (
        <List disablePadding sx={{ display: 'grid', gap: density.rowGap }}>
          {props.filteredProjects.map((project) => {
            const selected = project.id === props.activeId
            const sessions = recentSessions(project)
            return (
              <Box key={project.id} sx={{ display: 'grid', gap: 0.5 }}>
                <ListItemButton
                  selected={selected}
                  onClick={() => props.onSelectProject(project.id)}
                  sx={(muiTheme: Theme) => ({
                    alignItems: 'center',
                    gap: 0.75,
                    minHeight: 42,
                    border: '1px solid',
                    borderColor: selected ? 'primary.light' : 'transparent',
                    bgcolor: selected ? alpha(muiTheme.palette.primary.main, 0.08) : 'transparent',
                    '&.Mui-selected, &.Mui-selected:hover': { bgcolor: alpha(muiTheme.palette.primary.main, 0.1) },
                    '&:hover': { borderColor: 'primary.light', bgcolor: alpha(muiTheme.palette.primary.main, 0.08) },
                  })}
                >
                  <Box sx={{ width: 8, height: 8, flex: '0 0 8px', borderRadius: 999, bgcolor: statusDotColors[project.status] }} />
                  <Box sx={{ minWidth: 0 }}>
                    <Typography sx={{ fontSize: density.text.md, fontWeight: 850, overflowWrap: 'anywhere' }}>{project.name}</Typography>
                    <Typography variant="caption" sx={{ color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750, overflowWrap: 'anywhere' }}>
                      {project.next_action}
                    </Typography>
                  </Box>
                </ListItemButton>
                <Stack spacing={0.15} component="nav" aria-label={`${project.name} sessions`} sx={{ pl: 2 }}>
                  {sessions.map((session) => (
                    <SessionListItem
                      key={session.id}
                      projectId={project.id}
                      selected={selected && session.id === props.activeSessionId}
                      session={session}
                      onSelectSession={props.onSelectSession}
                    />
                  ))}
                </Stack>
              </Box>
            )
          })}
        </List>
      )}
    </Box>
  )
}

function SessionListItem(props: {
  projectId: string
  selected: boolean
  session: Session
  onSelectSession: (projectId: string, sessionId: string) => void
}) {
  const lastMessage = props.session.messages[props.session.messages.length - 1]
  return (
    <Box
      component="button"
      type="button"
      onClick={() => props.onSelectSession(props.projectId, props.session.id)}
      sx={(muiTheme: Theme) => ({
        display: 'grid',
        gap: 0.25,
        width: '100%',
        border: 0,
        borderLeft: '2px solid',
        borderLeftColor: props.selected ? muiTheme.palette.primary.main : muiTheme.palette.divider,
        borderRadius: 1,
        bgcolor: props.selected ? alpha(muiTheme.palette.primary.main, 0.08) : 'transparent',
        color: 'text.primary',
        cursor: 'pointer',
        p: 0.5,
        textAlign: 'left',
        '&:hover': { bgcolor: alpha(muiTheme.palette.primary.main, 0.08), borderLeftColor: muiTheme.palette.primary.light },
      })}
    >
      <Typography sx={{ fontSize: density.text.sm, fontWeight: 850, overflowWrap: 'anywhere' }}>{props.session.title}</Typography>
      {lastMessage && (
        <Typography sx={{ color: 'text.secondary', fontSize: density.text.xs, lineHeight: 1.3, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
          {lastMessage.body}
        </Typography>
      )}
    </Box>
  )
}

function EmptyCenter(props: { loading: boolean; error: string | null }) {
  return (
    <Box sx={{ display: 'grid', minHeight: 320, placeItems: 'center', textAlign: 'center' }}>
      <Stack spacing={1} sx={{ alignItems: 'center' }}>
        {props.loading && <CircularProgress size={24} />}
        <Typography sx={{ fontWeight: 850 }}>{props.error ?? 'Loading Artifact Studio'}</Typography>
      </Stack>
    </Box>
  )
}

function ContextPanel(props: {
  activeProject?: Project
  activeWorkItemId: string
  completedWorkItems: number
  context: WorkItemContext | null
  error: string | null
  runningArtifact: boolean
  showWorkItems: boolean
  onRunArtifact: () => void
  onSelectPreviewRole: (role: string) => void
  onSelectWorkItem: (id: string) => void
  onToggleWorkItems: () => void
  appSpecPreview: AppSpecPreview | null
  permissionPreview: PermissionSimulation | null
  previewRole: string
  specDiff: AppSpecDiff | null
}) {
  const project = props.activeProject
  return (
    <Box
      component="aside"
      aria-label="Project status"
      sx={{ display: 'grid', alignContent: 'start', gap: density.rowGap, gridColumn: { xs: '1', md: '1 / -1', lg: 'auto' }, minWidth: 0, minHeight: { xs: 'auto', lg: '100vh' }, borderLeft: { xs: 0, lg: 1 }, borderTop: { xs: 1, lg: 0 }, borderColor: 'divider', bgcolor: 'background.paper', p: density.shellPad }}
    >
      {!project ? (
        <Typography sx={{ color: 'text.secondary', fontSize: 13, fontWeight: 750 }}>No project selected.</Typography>
      ) : (
        <>
          <Box>
            <Typography variant="caption" sx={{ color: 'text.secondary', fontSize: density.text.xs, fontWeight: 850, textTransform: 'uppercase' }}>
              Next action
            </Typography>
            <Typography variant="h2" sx={{ mt: 0.25, mb: 0.75, overflowWrap: 'anywhere' }}>
              {props.context?.next_action ?? project.next_action}
            </Typography>
            <Button fullWidth variant="contained" type="button" disabled={!props.context?.next_artifact_kind || props.runningArtifact} onClick={props.onRunArtifact}>
              {props.context?.next_artifact_kind ? `Create ${props.context.next_artifact_kind.toUpperCase()}` : 'Continue'}
            </Button>
            <Button fullWidth variant="outlined" type="button" startIcon={<AssignmentTurnedInOutlinedIcon />} onClick={props.onToggleWorkItems} sx={{ mt: 0.5 }}>
              {props.showWorkItems ? 'Hide WorkItems' : `WorkItems ${props.completedWorkItems}/${project.workitems.length}`}
            </Button>
          </Box>

          {props.error && (
            <Box sx={{ border: 1, borderColor: 'error.main', borderRadius: 1, p: density.panelPad }}>
              <Typography sx={{ color: 'error.main', fontSize: density.text.md, fontWeight: 800 }}>{props.error}</Typography>
            </Box>
          )}

          <Divider />

          {props.showWorkItems && (
            <WorkItemsList
              activeWorkItemId={props.activeWorkItemId}
              completedWorkItems={props.completedWorkItems}
              project={project}
              onSelectWorkItem={props.onSelectWorkItem}
            />
          )}

          <WorkflowContext context={props.context} />
          <StageList project={project} />
          <GoalPanel project={project} />
          <ArtifactsList project={project} />
          <AppSpecPreviewPanel diff={props.specDiff} preview={props.appSpecPreview} />
          <PermissionPreviewPanel
            diff={props.specDiff}
            onSelectRole={props.onSelectPreviewRole}
            preview={props.permissionPreview}
            role={props.previewRole}
          />
          <OperationsDashboard project={project} />
        </>
      )}
    </Box>
  )
}

function WorkItemsList(props: {
  activeWorkItemId: string
  completedWorkItems: number
  project: Project
  onSelectWorkItem: (id: string) => void
}) {
  return (
    <Stack spacing={density.rowGap} aria-label="Project WorkItems">
      <Box>
        <Typography variant="h2" sx={{ fontSize: density.text.lg }}>
          WorkItems
        </Typography>
        <Typography sx={{ mt: 0.15, color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750 }}>
          {props.completedWorkItems} of {props.project.workitems.length} ready
        </Typography>
      </Box>
      {props.project.workitems.map((workItem) => {
        const visual = workItemVisuals[workItem.state]
        const selected = workItem.id === props.activeWorkItemId
        return (
          <Box
            component="button"
            type="button"
            key={workItem.id}
            onClick={() => props.onSelectWorkItem(workItem.id)}
            sx={{
              display: 'grid',
              gap: 0.65,
              width: '100%',
              border: 1,
              borderColor: selected ? 'primary.light' : 'divider',
              borderRadius: 1,
              bgcolor: selected ? '#eef6fb' : 'background.paper',
              cursor: 'pointer',
              p: density.panelPad,
              textAlign: 'left',
            }}
          >
            <Stack direction="row" spacing={1} sx={{ alignItems: 'start', justifyContent: 'space-between', minWidth: 0 }}>
              <Box sx={{ minWidth: 0 }}>
                <Typography sx={{ fontSize: density.text.md, fontWeight: 850, overflowWrap: 'anywhere' }}>{workItem.title}</Typography>
                <Typography sx={{ color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750 }}>
                  {workItem.route} to {workItem.target}
                </Typography>
              </Box>
              <Chip size="small" color={visual.color} label={visual.label} variant="outlined" />
            </Stack>
            <Stack direction="row" spacing={0.75} sx={{ flexWrap: 'wrap' }}>
              <Chip size="small" color={qcVisuals[workItem.qc_status].color} label={qcVisuals[workItem.qc_status].label} variant="outlined" />
            </Stack>
            <LinearProgress
              aria-label={`${workItem.title} progress`}
              variant="determinate"
              value={workItem.progress}
              sx={{ height: 6, borderRadius: 999, bgcolor: '#e4e9ef', '& .MuiLinearProgress-bar': { bgcolor: visual.tone, borderRadius: 999 } }}
            />
            <Stack direction="row" spacing={1} sx={{ alignItems: 'center', justifyContent: 'space-between', minWidth: 0 }}>
              <Typography sx={{ color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750, overflowWrap: 'anywhere' }}>{workItem.next_action}</Typography>
              <Typography sx={{ color: visual.tone, fontSize: density.text.xs, fontWeight: 850 }}>{workItem.progress}%</Typography>
            </Stack>
          </Box>
        )
      })}
    </Stack>
  )
}

function WorkflowContext(props: { context: WorkItemContext | null }) {
  const workitem = props.context?.workitem
  if (!props.context || !workitem) return null
  const context = props.context
  return (
    <Stack spacing={density.rowGap} aria-label="WorkItem context">
      <Box>
        <Typography variant="h2" sx={{ fontSize: density.text.lg }}>
          Active workflow
        </Typography>
        <Typography sx={{ mt: 0.15, color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750 }}>
          {workitem.route} to {workitem.target}
        </Typography>
      </Box>
      {context.qc_status && (
        <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: density.panelPad }}>
          <Stack direction="row" spacing={1} sx={{ alignItems: 'center', justifyContent: 'space-between' }}>
            <Typography sx={{ fontSize: density.text.md, fontWeight: 850 }}>Quality gate</Typography>
            <Chip size="small" color={qcVisuals[context.qc_status].color} label={qcVisuals[context.qc_status].label} variant="outlined" />
          </Stack>
          {(context.qc_checks ?? []).map((check) => (
            <Typography key={check.id} sx={{ mt: 0.35, color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750, overflowWrap: 'anywhere' }}>
              {check.label}: {check.summary}
            </Typography>
          ))}
        </Box>
      )}
      {context.blockers && context.blockers.length > 0 && (
        <Box sx={{ border: 1, borderColor: 'warning.main', borderRadius: 1, p: density.panelPad }}>
          <Typography sx={{ fontSize: density.text.md, fontWeight: 850 }}>Clarifications</Typography>
          {context.blockers.map((blocker) => (
            <Typography key={blocker} sx={{ mt: 0.35, color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750, overflowWrap: 'anywhere' }}>
              {blocker}
            </Typography>
          ))}
        </Box>
      )}
      {context.artifact_gates && context.artifact_gates.length > 0 && (
        <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: density.panelPad }}>
          <Typography sx={{ fontSize: density.text.md, fontWeight: 850 }}>Artifact dependencies</Typography>
          {context.artifact_gates.map((gate) => (
            <Stack key={gate.kind} direction="row" spacing={0.75} sx={{ alignItems: 'start', justifyContent: 'space-between', mt: 0.5 }}>
              <Box sx={{ minWidth: 0 }}>
                <Typography sx={{ fontSize: density.text.xs, fontWeight: 850 }}>{gate.kind}</Typography>
                <Typography sx={{ color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750, overflowWrap: 'anywhere' }}>{gate.reason}</Typography>
              </Box>
              <Chip size="small" color={gate.unlocked ? 'success' : 'warning'} label={gate.unlocked ? 'Unlocked' : gate.gate} variant="outlined" />
            </Stack>
          ))}
        </Box>
      )}
      <WorkflowGraph steps={context.workflow_plan ?? []} />
    </Stack>
  )
}

function WorkflowGraph(props: { steps: WorkflowStep[] }) {
  const steps = props.steps
  if (steps.length === 0) return null
  const nodeWidth = 152
  const nodeGap = 38
  const graphHeight = 136
  const totalWidth = steps.length * nodeWidth + Math.max(0, steps.length - 1) * nodeGap
  const indexById = new Map(steps.map((step, index) => [step.id, index]))
  const edges = steps.flatMap((step, targetIndex) =>
    (step.depends_on ?? [])
      .map((dependency) => {
        const sourceIndex = indexById.get(dependency)
        if (sourceIndex === undefined || sourceIndex >= targetIndex) return null
        return { id: `${dependency}-${step.id}`, sourceIndex, targetIndex }
      })
      .filter((edge): edge is { id: string; sourceIndex: number; targetIndex: number } => edge !== null),
  )

  return (
    <Box aria-label="Workflow DAG" sx={{ border: 1, borderColor: 'divider', borderRadius: 1, bgcolor: '#fbfcfe', p: density.panelPad }}>
      <Stack direction="row" spacing={1} sx={{ alignItems: 'center', justifyContent: 'space-between', mb: 0.75 }}>
        <Typography sx={{ fontSize: density.text.md, fontWeight: 850 }}>Workflow graph</Typography>
        <Chip size="small" color="default" label={`${steps.length} nodes`} variant="outlined" />
      </Stack>
      <Box sx={{ overflowX: 'auto', pb: 0.25 }}>
        <Box sx={{ position: 'relative', width: totalWidth, minWidth: '100%', height: graphHeight }}>
          <Box
            component="svg"
            aria-hidden="true"
            data-workflow-edges="true"
            viewBox={`0 0 ${totalWidth} ${graphHeight}`}
            preserveAspectRatio="none"
            sx={{ position: 'absolute', inset: 0, width: totalWidth, height: graphHeight, pointerEvents: 'none' }}
          >
            <defs>
              <marker id="workflow-arrow" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto" markerUnits="strokeWidth">
                <path d="M 0 0 L 8 4 L 0 8 z" fill="#8a94a6" />
              </marker>
            </defs>
            {edges.map((edge) => {
              const x1 = edge.sourceIndex * (nodeWidth + nodeGap) + nodeWidth + 4
              const x2 = edge.targetIndex * (nodeWidth + nodeGap) - 8
              const y = 60
              const curve = Math.max(22, Math.min(54, (x2 - x1) / 2))
              return (
                <path
                  key={edge.id}
                  data-workflow-edge={edge.id}
                  d={`M ${x1} ${y} C ${x1 + curve} ${y}, ${x2 - curve} ${y}, ${x2} ${y}`}
                  stroke="#8a94a6"
                  strokeWidth="1.6"
                  fill="none"
                  markerEnd="url(#workflow-arrow)"
                />
              )
            })}
          </Box>
          <Box sx={{ display: 'grid', gridTemplateColumns: `repeat(${steps.length}, ${nodeWidth}px)`, columnGap: `${nodeGap}px`, position: 'relative', zIndex: 1 }}>
            {steps.map((step) => {
              const visual = stageVisuals[step.state]
              const agentLabel = workflowAgentLabel(step)
              return (
                <Box
                  key={step.id}
                  data-workflow-node={step.id}
                  sx={{
                    height: 124,
                    border: 1,
                    borderColor: alpha(visual.tone, 0.55),
                    borderRadius: 1,
                    bgcolor: 'background.paper',
                    boxShadow: `inset 0 3px 0 ${visual.tone}`,
                    p: 0.75,
                  }}
                >
                  <Stack spacing={0.45} sx={{ minWidth: 0 }}>
                    <Stack direction="row" spacing={0.5} sx={{ alignItems: 'center', justifyContent: 'space-between', minWidth: 0 }}>
                      <Stack direction="row" spacing={0.4} sx={{ alignItems: 'center', minWidth: 0 }}>
                        <Box sx={{ color: visual.tone, lineHeight: 0, '& svg': { fontSize: 16 } }}>
                          <StageIcon stage={step.state} />
                        </Box>
                        <Typography sx={{ fontSize: density.text.md, fontWeight: 900, overflowWrap: 'anywhere' }}>{step.label}</Typography>
                      </Stack>
                      <Chip size="small" color={visual.color} label={visual.label} variant="outlined" />
                    </Stack>
                    {agentLabel && <Chip size="small" color="secondary" label={agentLabel} variant="outlined" sx={{ alignSelf: 'start', maxWidth: '100%' }} />}
                    <Typography sx={{ color: 'text.secondary', fontSize: density.text.xs, fontWeight: 800, overflowWrap: 'anywhere' }}>
                      {workflowDependencyLabel(step)}
                    </Typography>
                    {step.agent_task && (
                      <Typography
                        sx={{
                          color: 'text.secondary',
                          display: '-webkit-box',
                          fontSize: density.text.xs,
                          lineHeight: 1.25,
                          overflow: 'hidden',
                          overflowWrap: 'anywhere',
                          WebkitBoxOrient: 'vertical',
                          WebkitLineClamp: 2,
                        }}
                      >
                        {step.agent_task}
                      </Typography>
                    )}
                  </Stack>
                </Box>
              )
            })}
          </Box>
        </Box>
      </Box>
    </Box>
  )
}

function StageList(props: { project: Project }) {
  return (
    <Stack spacing={density.rowGap} aria-label="Artifact progress">
      {props.project.stages.map((stage) => {
        const visual = stageVisuals[stage.state]
        const agentLabel = workflowAgentLabel(stage)
        return (
          <Box key={stage.id} sx={{ display: 'grid', gridTemplateColumns: `${density.iconTrack} minmax(0, 1fr) max-content`, gap: density.rowGap, alignItems: 'start', border: 1, borderColor: 'divider', borderRadius: 1, p: density.panelPad }}>
            <Box sx={{ color: visual.tone, lineHeight: 0 }}>
              <StageIcon stage={stage.state} />
            </Box>
            <Box sx={{ minWidth: 0 }}>
              <Stack direction="row" spacing={0.5} sx={{ alignItems: 'center', flexWrap: 'wrap' }}>
                <Typography sx={{ fontSize: density.text.md, fontWeight: 850 }}>{stage.label}</Typography>
                {agentLabel && <Chip size="small" color="secondary" label={agentLabel} variant="outlined" />}
              </Stack>
              <Typography sx={{ mt: 0.2, color: 'text.secondary', fontSize: density.text.xs, lineHeight: 1.3, overflowWrap: 'anywhere' }}>{stage.detail}</Typography>
            </Box>
            <Chip size="small" color={visual.color} label={visual.label} variant="outlined" />
          </Box>
        )
      })}
    </Stack>
  )
}

function ArtifactsList(props: { project: Project }) {
  return (
    <Stack spacing={density.rowGap} aria-label="Artifacts">
      <Typography variant="h2" sx={{ fontSize: density.text.lg }}>
        Artifacts
      </Typography>
      {props.project.artifacts.length === 0 ? (
        <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: density.panelPad }}>
          <Typography sx={{ color: 'text.secondary', fontSize: density.text.md, fontWeight: 750 }}>No artifacts yet. Complete the WorkItem first.</Typography>
        </Box>
      ) : (
        props.project.artifacts.map((artifact) => (
          <Box key={artifact.id} sx={{ display: 'grid', gridTemplateColumns: 'minmax(0, 1fr) max-content', gap: density.rowGap, alignItems: 'start', border: 1, borderColor: 'divider', borderRadius: 1, p: density.panelPad }}>
            <Box sx={{ minWidth: 0 }}>
              <Typography sx={{ fontSize: density.text.md, fontWeight: 850, overflowWrap: 'anywhere' }}>{artifact.label}</Typography>
              <Typography sx={{ color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750 }}>{artifact.kind}</Typography>
            </Box>
            <Typography sx={{ color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750 }}>{artifact.status}</Typography>
            {artifact.qc_status && (
              <Chip size="small" color={qcVisuals[artifact.qc_status].color} label={qcVisuals[artifact.qc_status].label} variant="outlined" sx={{ gridColumn: '1 / -1', justifySelf: 'start' }} />
            )}
            {artifact.summary && (
              <Typography sx={{ color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750, gridColumn: '1 / -1', overflowWrap: 'anywhere' }}>
                {artifact.summary}
              </Typography>
            )}
          </Box>
        ))
      )}
    </Stack>
  )
}

function GoalPanel(props: { project: Project }) {
  const goal = props.project.goal
  if (!goal) return null
  const status = props.project.goal_status
  const metricRows = props.project.goal_metrics && props.project.goal_metrics.length > 0 ? props.project.goal_metrics : goal.success_metrics
  return (
    <Stack spacing={density.rowGap} aria-label="App goal">
      <Stack direction="row" spacing={0.75} sx={{ alignItems: 'center', justifyContent: 'space-between' }}>
        <Typography variant="h2" sx={{ fontSize: density.text.lg }}>
          Goal
        </Typography>
        {status && (
          <Chip
            size="small"
            color={status.status === 'on_track' ? 'success' : status.status === 'at_risk' ? 'error' : 'warning'}
            label={status.status.replace('_', ' ')}
            variant="outlined"
          />
        )}
      </Stack>
      <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, p: density.panelPad }}>
        <Typography sx={{ fontSize: density.text.md, fontWeight: 850, overflowWrap: 'anywhere' }}>{goal.statement}</Typography>
        <Typography sx={{ mt: 0.35, color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750, overflowWrap: 'anywhere' }}>
          {goal.problem}
        </Typography>
        <Box sx={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: density.rowGap, mt: 0.8 }}>
          <Metric label="Review" value={goal.review_policy.cadence} />
          <Metric label="Users" value={goal.target_users.join(', ')} />
        </Box>
        <Stack spacing={0.5} sx={{ mt: 0.8 }}>
          {metricRows.map((metric) => (
            <Box key={metric.name} sx={{ display: 'grid', gridTemplateColumns: 'minmax(0, 1fr) max-content', gap: density.rowGap, alignItems: 'start' }}>
              <Box sx={{ minWidth: 0 }}>
                <Typography sx={{ fontSize: density.text.xs, fontWeight: 850, overflowWrap: 'anywhere' }}>{metric.name}</Typography>
                <Typography sx={{ color: 'text.secondary', fontSize: density.text.xs, fontWeight: 750, overflowWrap: 'anywhere' }}>
                  {metric.baseline ?? 'no baseline'} {'->'} {metric.target} / {metric.window}
                </Typography>
              </Box>
              <Chip size="small" color="default" label={metric.current === undefined ? 'tracking' : String(metric.current)} variant="outlined" />
            </Box>
          ))}
        </Stack>
      </Box>
    </Stack>
  )
}

function AppSpecPreviewPanel(props: { diff: AppSpecDiff | null; preview: AppSpecPreview | null }) {
  const changedCategories = Object.entries(props.diff?.categories ?? {}).filter(([, changes]) => changes.length > 0)
  return (
    <Stack spacing={density.rowGap} aria-label="App Spec preview">
      <Typography variant="h2" sx={{ fontSize: density.text.lg }}>
        App Spec preview
      </Typography>
      <Box sx={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: density.rowGap, border: 1, borderColor: 'divider', borderRadius: 1, p: density.panelPad }}>
        <Metric label="Forms" value={String(props.preview?.form_preview.length ?? 0)} />
        <Metric label="Tables" value={String(props.preview?.table_preview.length ?? 0)} />
        <Metric label="Workflow" value={String(props.preview?.workflow_preview.length ?? 0)} />
        <Metric label="Permissions" value={String(Object.keys(props.preview?.permission_editor ?? {}).length)} />
        <Metric label="Notifications" value={String(props.preview?.notification_editor.length ?? 0)} />
        <Metric label="Dashboard" value={String(props.preview?.dashboard_preview.length ?? 0)} />
        <Metric label="Goal metrics" value={String(props.preview?.goal_preview?.success_metrics?.length ?? 0)} />
        <Metric label="Diff" value={changedCategories.map(([category]) => category).join(', ') || 'none'} />
        <Metric label="Sandbox" value="gate ready" />
      </Box>
    </Stack>
  )
}

function PermissionPreviewPanel(props: {
  diff: AppSpecDiff | null
  onSelectRole: (role: string) => void
  preview: PermissionSimulation | null
  role: string
}) {
  const changedCategories = Object.entries(props.diff?.categories ?? {}).filter(([, changes]) => changes.length > 0)
  const visibleFields = props.preview?.field_access.filter((field) => field.visible) ?? []
  const editableFields = props.preview?.field_access.filter((field) => field.editable) ?? []
  const maskedFields = props.preview?.field_access.filter((field) => field.masked) ?? []
  return (
    <Stack spacing={density.rowGap} aria-label="Permission and risk preview">
      <Typography variant="h2" sx={{ fontSize: density.text.lg }}>
        Permission preview
      </Typography>
      <Stack direction="row" spacing={0.5} sx={{ flexWrap: 'wrap' }}>
        {previewRoles.map((role) => (
          <Button key={role} variant={role === props.role ? 'contained' : 'outlined'} type="button" onClick={() => props.onSelectRole(role)}>
            {role}
          </Button>
        ))}
      </Stack>
      <Box sx={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: density.rowGap, border: 1, borderColor: 'divider', borderRadius: 1, p: density.panelPad }}>
        <Metric label="Actions" value={props.preview?.allowed_actions.join(', ') || 'none'} />
        <Metric label="Approvals" value={props.preview?.approval_requirements.join(', ') || 'none'} />
        <Metric label="Visible fields" value={String(visibleFields.length)} />
        <Metric label="Editable fields" value={String(editableFields.length)} />
        <Metric label="Masked fields" value={maskedFields.map((field) => field.field).join(', ') || 'none'} />
        <Metric label="Risk changes" value={changedCategories.map(([category]) => category).join(', ') || 'none'} />
      </Box>
    </Stack>
  )
}

function OperationsDashboard(props: { project: Project }) {
  const dashboard = props.project.operations_dashboard
  if (!dashboard) return null
  return (
    <Stack spacing={density.rowGap} aria-label="Operations dashboard">
      <Typography variant="h2" sx={{ fontSize: density.text.lg }}>
        Operations dashboard
      </Typography>
      <Box sx={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: density.rowGap, border: 1, borderColor: 'divider', borderRadius: 1, p: density.panelPad }}>
        <Metric label="Health" value={dashboard.health} />
        <Metric label="Release" value={dashboard.latest_release} />
        <Metric label="Tests" value={dashboard.ci_status} />
        <Metric label="Environment" value={dashboard.deployment_environment} />
        <Metric label="Open blockers" value={String(dashboard.open_blockers.length)} />
        <Metric label="Todos" value={`${dashboard.runtime_metrics.completed_count}/${dashboard.runtime_metrics.todo_count}`} />
      </Box>
    </Stack>
  )
}

function Metric(props: { label: string; value: string }) {
  return (
    <Box sx={{ minWidth: 0 }}>
      <Typography sx={{ color: 'text.secondary', fontSize: density.text.xs, fontWeight: 850, textTransform: 'uppercase' }}>
        {props.label}
      </Typography>
      <Typography sx={{ fontSize: density.text.md, fontWeight: 850, overflowWrap: 'anywhere' }}>{props.value}</Typography>
    </Box>
  )
}
