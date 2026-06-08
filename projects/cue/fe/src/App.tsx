import { type FormEvent, useEffect, useMemo, useState } from 'react'
import ClipboardCheck from 'lucide-react/dist/esm/icons/clipboard-check.js'
import Database from 'lucide-react/dist/esm/icons/database.js'
import LayoutDashboard from 'lucide-react/dist/esm/icons/layout-dashboard.js'
import PlusCircle from 'lucide-react/dist/esm/icons/plus-circle.js'
import Rocket from 'lucide-react/dist/esm/icons/rocket.js'
import Settings from 'lucide-react/dist/esm/icons/settings.js'
import {
  adminReviewTickets,
  appArtifacts,
  buildDraftSpec,
  getGovernanceChecks,
  initialClarifications,
  ownershipNamespace,
  runtimeTenant,
  seedSpec,
  storySteps,
  type ClarificationId,
  type ClarificationState,
} from './mockData'
import { ArtifactPanel, Badge, OwnerGuidePanel, PersonaSwitch, StoryStepper, WorkspaceBoundaryPanel } from './components'
import { configuredLocale, getCopy } from './i18n'
import { AdminView, PromptBuilder, RegistryView, SandboxView, StudioView } from './views'
import type { AdminReviewTicketStatus, AppSpec, Persona, RouteKey, StoryState, StudioTab } from './types'

const navRoutes: RouteKey[] = ['studio', 'sandbox', 'registry']

const navIcons: Record<RouteKey, typeof ClipboardCheck> = {
  new: PlusCircle,
  studio: LayoutDashboard,
  sandbox: Rocket,
  registry: Database,
  admin: Settings,
}

const ownerRoutes: RouteKey[] = ['new', 'studio', 'sandbox', 'registry']

function routeFromPath(pathname: string): RouteKey {
  if (pathname.includes('/studio')) return 'studio'
  if (pathname.includes('/sandbox')) return 'sandbox'
  if (pathname.startsWith('/registry')) return 'registry'
  if (pathname.startsWith('/admin')) return 'admin'
  return 'new'
}

function pathForRoute(route: RouteKey, appId: string): string {
  if (route === 'studio') return `/apps/${appId}/studio`
  if (route === 'sandbox') return `/apps/${appId}/sandbox`
  if (route === 'registry') return '/registry'
  if (route === 'admin') return '/admin'
  return '/apps/new'
}

function storyRank(state: StoryState): number {
  return storySteps.findIndex((step) => step.state === state)
}

function stateForRoute(route: RouteKey, current: StoryState): StoryState {
  if (route === 'new') return 'PromptDraft'
  if (route === 'studio' && storyRank(current) < storyRank('StudioPreview')) return 'StudioPreview'
  if (route === 'sandbox' && storyRank(current) < storyRank('SandboxReady')) return 'SandboxReady'
  if (route === 'admin' && storyRank(current) < storyRank('ProductionRequested')) return 'ProductionRequested'
  if (route === 'registry' && storyRank(current) < storyRank('SpecDraft')) return 'SpecDraft'
  return current
}

export function App() {
  const [locale] = useState(configuredLocale)
  const ui = getCopy(locale)
  const [activeRoute, setActiveRoute] = useState<RouteKey>(() => routeFromPath(window.location.pathname))
  const [prompt, setPrompt] = useState(ui.mock.defaultPrompt)
  const [draftSpec, setDraftSpec] = useState<AppSpec>(() => buildDraftSpec(ui.mock.defaultPrompt, ui.mock.defaultPrompt))
  const [storyState, setStoryState] = useState<StoryState>(() =>
    stateForRoute(routeFromPath(window.location.pathname), 'PromptDraft'),
  )
  const [clarifications, setClarifications] = useState<ClarificationState>(initialClarifications)
  const [studioTab, setStudioTab] = useState<StudioTab>('fields')
  const [persona, setPersona] = useState<Persona>(() =>
    routeFromPath(window.location.pathname) === 'admin' ? 'platform' : 'owner',
  )
  const [approvedTicketIds, setApprovedTicketIds] = useState<string[]>([])
  const ownerNavItems = ownerRoutes.map((route) => ({ route, icon: navIcons[route], ...ui.routes[route] }))
  const adminNavItems = navRoutes.map((route) => ({ route, icon: navIcons[route], ...ui.routes[route] }))
  const storyStepperItems = storySteps.map((step) => ({ ...step, label: ui.storyStates[step.state] }))
  const checks = useMemo(() => getGovernanceChecks(draftSpec, ui.governance), [draftSpec, ui])
  const reviewTickets = adminReviewTickets.map((ticket) => ({
    ...ticket,
    status: (approvedTicketIds.includes(ticket.id) ? 'approved' : 'pending') as AdminReviewTicketStatus,
  }))
  const pendingReviewCount = reviewTickets.filter((ticket) => ticket.status === 'pending').length
  const effectivePersona = activeRoute === 'admin' ? persona : 'owner'
  const visibleArtifacts = appArtifacts.filter((artifact) => effectivePersona === 'platform' || artifact.visibility === 'owner')
  const artifactTitle = effectivePersona === 'platform' ? ui.artifacts.platformOnlyTitle : ui.artifacts.ownerOnlyTitle
  const blockedCount = checks.filter((check) => check.state === 'block').length
  const reviewCount = checks.filter((check) => check.state === 'review').length

  useEffect(() => {
    const handlePopState = () => {
      const nextRoute = routeFromPath(window.location.pathname)
      setActiveRoute(nextRoute)
      setPersona(nextRoute === 'admin' ? 'platform' : 'owner')
      setStoryState((current) => stateForRoute(nextRoute, current))
    }
    window.addEventListener('popstate', handlePopState)
    return () => window.removeEventListener('popstate', handlePopState)
  }, [])

  function navigate(route: RouteKey, nextState?: StoryState) {
    setActiveRoute(route)
    setPersona(route === 'admin' ? 'platform' : 'owner')
    setStoryState((current) => nextState ?? stateForRoute(route, current))
    window.history.pushState(null, '', pathForRoute(route, ownershipNamespace.app_id))
  }

  function toggleClarification(id: ClarificationId) {
    setClarifications((current) => ({ ...current, [id]: !current[id] }))
  }

  function handleGenerate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const nextSpec = buildDraftSpec(prompt, ui.mock.defaultPrompt)
    const ready = Object.values(clarifications).every(Boolean)
    setDraftSpec(nextSpec)
    if (ready) {
      navigate('studio', 'StudioPreview')
    } else {
      setStoryState('Clarifying')
    }
  }

  function loadSeedSpec() {
    setDraftSpec(seedSpec)
    setPrompt(seedSpec.app.description)
    setStoryState('SpecDraft')
  }

  function requestSandbox() {
    navigate('sandbox', 'SandboxReady')
  }

  function requestProduction() {
    navigate('registry', 'ProductionRequested')
  }

  function approveProduction() {
    navigate('registry', 'ProductionReady')
  }

  function approveTicket(ticketId: string) {
    setApprovedTicketIds((current) => (current.includes(ticketId) ? current : [...current, ticketId]))
  }

  if (activeRoute === 'admin') {
    return (
      <main className="cue-shell admin-shell">
        <aside className="sidebar admin-sidebar" aria-label={ui.shell.adminNavAria}>
          <div className="brand-lockup">
            <div className="brand-mark">C</div>
            <div>
              <strong>Cue Admin</strong>
              <span>{ui.shell.adminSubtitle}</span>
            </div>
          </div>
          <nav className="nav-stack">
            <button type="button" className="nav-item active" onClick={() => navigate('admin')}>
              <Settings aria-hidden="true" />
              <span className="nav-copy">
                <span className="nav-label">{ui.routes.admin.label}</span>
                <span className="nav-description">{ui.routes.admin.description}</span>
              </span>
            </button>
            {adminNavItems.map((item) => {
              const Icon = item.icon
              return (
                <button type="button" className="nav-item" onClick={() => navigate(item.route)} key={item.route}>
                  <Icon aria-hidden="true" />
                  <span className="nav-copy">
                    <span className="nav-label">{item.label}</span>
                    <span className="nav-description">{item.description}</span>
                  </span>
                </button>
              )
            })}
          </nav>
          <div className="sidebar-status">
            <span>{ui.shell.workspace}</span>
            <strong>{ui.workspaces.admin.title}</strong>
            <span>{ui.shell.owner}</span>
            <strong>{ownershipNamespace.platform_owner}</strong>
            <span>{ui.shell.environment}</span>
            <strong>{runtimeTenant.environment}</strong>
          </div>
        </aside>

        <section className="main-surface admin-surface">
          <header className="topbar">
            <div>
              <p className="eyebrow">{ui.routes.admin.label}</p>
              <h1>{ui.workspaces.admin.title}</h1>
            </div>
            <div className="topbar-actions">
              <PersonaSwitch active={persona} ui={ui} onChange={setPersona} />
              <div className="status-strip" aria-label={ui.shell.appStateAria}>
                <Badge label={ui.shell.reviewBadge(reviewCount)} tone="blue" />
                <Badge label={ui.workspaces.reviewSummary(pendingReviewCount)} tone={pendingReviewCount > 0 ? 'amber' : 'green'} />
              </div>
            </div>
          </header>

          <WorkspaceBoundaryPanel kind="admin" pendingReviewCount={pendingReviewCount} ui={ui} />
          <ArtifactPanel artifacts={visibleArtifacts} title={artifactTitle} ui={ui} />
          <AdminView
            persona={persona}
            reviewTickets={reviewTickets}
            ui={ui}
            onApprove={approveProduction}
            onApproveTicket={approveTicket}
          />
        </section>
      </main>
    )
  }

  return (
    <main className="front-office-shell">
      <header className="owner-header">
        <div className="brand-lockup">
          <div className="brand-mark">C</div>
          <div>
            <strong>Cue</strong>
            <span>{ui.shell.brandSubtitle}</span>
          </div>
        </div>
        <nav className="owner-nav" aria-label={ui.shell.ownerNavAria}>
          {ownerNavItems.map((item) => {
            const Icon = item.icon
            return (
              <button
                type="button"
                className={activeRoute === item.route ? 'owner-nav-item active' : 'owner-nav-item'}
                onClick={() => navigate(item.route)}
                key={item.route}
              >
                <Icon aria-hidden="true" />
                <span>
                  <strong>{item.label}</strong>
                  <small>{item.description}</small>
                </span>
              </button>
            )
          })}
        </nav>
        <div className="owner-header-status" aria-label={ui.shell.appStateAria}>
          <Badge label={storyStateLabel(storyState, ui.storyStates)} tone={blockedCount > 0 ? 'red' : 'green'} />
          <Badge label={draftSpec.app.risk_tier} tone="amber" />
        </div>
      </header>

      <section className="main-surface owner-surface">
        <header className="topbar">
          <div>
            <p className="eyebrow">{activeRouteLabel(activeRoute, ui.routes)}</p>
            <h1>{draftSpec.app.name}</h1>
          </div>
        </header>

        <StoryStepper ariaLabel={ui.storyStepper.ariaLabel} current={storyState} steps={storyStepperItems} />
        <OwnerGuidePanel activeRoute={activeRoute} pendingReviewCount={pendingReviewCount} ui={ui} />

        {activeRoute === 'new' && (
          <PromptBuilder
            clarifications={clarifications}
            prompt={prompt}
            storyState={storyState}
            ui={ui}
            onGenerate={handleGenerate}
            onPromptChange={setPrompt}
            onSeed={loadSeedSpec}
            onToggleClarification={toggleClarification}
          />
        )}
        {activeRoute === 'studio' && (
          <StudioView
            activeTab={studioTab}
            checks={checks}
            draftSpec={draftSpec}
            persona={effectivePersona}
            ui={ui}
            onRequestProduction={requestProduction}
            onRequestSandbox={requestSandbox}
            onTabChange={setStudioTab}
          />
        )}
        {activeRoute === 'sandbox' && (
          <SandboxView persona={effectivePersona} ui={ui} onRequestProduction={requestProduction} />
        )}
        {activeRoute === 'registry' && (
          <RegistryView
            draftSpec={draftSpec}
            persona={effectivePersona}
            storyState={storyState}
            ui={ui}
            onOpenStudio={() => navigate('studio')}
          />
        )}
      </section>
    </main>
  )
}

function activeRouteLabel(route: RouteKey, routes: Record<RouteKey, { label: string }>): string {
  return routes[route]?.label ?? 'Cue'
}

function storyStateLabel(state: StoryState, storyStates: Record<StoryState, string>): string {
  return storyStates[state]
}
