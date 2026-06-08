import CheckCircle2 from 'lucide-react/dist/esm/icons/check-circle-2.js'
import { sampleRequests } from './mockData'
import type { AppArtifact, GovernanceCheck, Persona, RouteKey, StoryState, WorkspaceKind } from './types'
import type { CueCopy } from './i18n'

export function Badge({ label, tone }: { label: string; tone: 'green' | 'amber' | 'blue' | 'red' }) {
  return <span className={`badge ${tone}`}>{label}</span>
}

export function PersonaSwitch({
  active,
  ui,
  onChange,
}: {
  active: Persona
  ui: CueCopy
  onChange: (persona: Persona) => void
}) {
  return (
    <div className="persona-switch" role="group" aria-label={ui.shell.personaAria}>
      {(['owner', 'platform'] as Persona[]).map((persona) => (
        <button
          className={active === persona ? 'persona-option active' : 'persona-option'}
          key={persona}
          onClick={() => onChange(persona)}
          type="button"
        >
          <strong>{ui.shell.personas[persona].label}</strong>
          <span>{ui.shell.personas[persona].description}</span>
        </button>
      ))}
    </div>
  )
}

export function StoryStepper({
  ariaLabel,
  current,
  steps,
}: {
  ariaLabel: string
  current: StoryState
  steps: Array<{ state: StoryState; label: string }>
}) {
  const currentRank = steps.findIndex((step) => step.state === current)
  return (
    <ol className="story-stepper" aria-label={ariaLabel}>
      {steps.map((step, index) => (
        <li className={index <= currentRank ? 'complete' : 'pending'} key={step.state}>
          <span>{index + 1}</span>
          <strong>{step.label}</strong>
        </li>
      ))}
    </ol>
  )
}

export function WorkspaceBoundaryPanel({
  kind,
  pendingReviewCount,
  ui,
}: {
  kind: WorkspaceKind
  pendingReviewCount: number
  ui: CueCopy
}) {
  const workspace = ui.workspaces[kind]
  const hasPendingReview = pendingReviewCount > 0
  return (
    <section className="workspace-panel" aria-label={ui.workspaces.currentAria}>
      <div className="workspace-summary">
        <p className="eyebrow">{workspace.eyebrow}</p>
        <h2>{workspace.title}</h2>
        <p>{workspace.detail}</p>
      </div>
      <div className="workspace-detail">
        <span>{ui.workspaces.labels.allowedScope}</span>
        <strong>{workspace.scope.join(' / ')}</strong>
      </div>
      <div className="workspace-detail">
        <span>{ui.workspaces.labels.adminGate}</span>
        <strong>{workspace.gate}</strong>
      </div>
      <Badge
        label={ui.workspaces.reviewSummary(pendingReviewCount)}
        tone={hasPendingReview ? 'amber' : 'green'}
      />
    </section>
  )
}

export function OwnerGuidePanel({
  activeRoute,
  pendingReviewCount,
  ui,
}: {
  activeRoute: RouteKey
  pendingReviewCount: number
  ui: CueCopy
}) {
  return (
    <section className="owner-guide-panel" aria-label={ui.ownerGuide.ariaLabel}>
      <div className="owner-guide-summary">
        <p className="eyebrow">{ui.ownerGuide.eyebrow}</p>
        <h2>{ui.ownerGuide.title}</h2>
        <p>{ui.ownerGuide.detail}</p>
      </div>
      <div className="owner-guide-item">
        <span>{ui.ownerGuide.nextActionLabel}</span>
        <strong>{ui.ownerGuide.nextByRoute[activeRoute]}</strong>
      </div>
      <div className="owner-guide-item">
        <span>{ui.ownerGuide.adminGateLabel}</span>
        <strong>{ui.ownerGuide.adminGate}</strong>
      </div>
      <Badge label={ui.workspaces.reviewSummary(pendingReviewCount)} tone={pendingReviewCount > 0 ? 'amber' : 'green'} />
    </section>
  )
}

export function PreviewTable({ ariaLabel, headers }: { ariaLabel: string; headers: string[] }) {
  return (
    <div className="data-table preview-table" role="table" aria-label={ariaLabel}>
      <div className="table-head" role="row">
        {headers.map((header) => (
          <span key={header}>{header}</span>
        ))}
      </div>
      {sampleRequests.map((request) => (
        <div className="table-row" role="row" key={request.id}>
          <span>{request.id}</span>
          <span>{request.title}</span>
          <span>{request.owner}</span>
          <span>{request.status}</span>
          <span>{request.age}</span>
        </div>
      ))}
    </div>
  )
}

export function GovernanceList({ checks }: { checks: GovernanceCheck[] }) {
  return (
    <div className="finding-list">
      {checks.map((check) => (
        <div className={`finding-row ${check.state}`} key={check.label}>
          <CheckCircle2 aria-hidden="true" />
          <span>{check.label}</span>
          <strong>{check.detail}</strong>
        </div>
      ))}
    </div>
  )
}

export function ArtifactPanel({
  artifacts,
  title,
  ui,
}: {
  artifacts: AppArtifact[]
  title: string
  ui: CueCopy
}) {
  return (
    <section className="tool-panel artifact-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">{ui.artifacts.eyebrow}</p>
          <h2>{title}</h2>
        </div>
        <CheckCircle2 aria-hidden="true" />
      </div>
      <div className="artifact-list">
        {artifacts.length === 0 && <p className="empty-state">{ui.artifacts.empty}</p>}
        {artifacts.map((artifact) => (
          <div className="artifact-row" key={artifact.id}>
            <span className={`artifact-state ${artifact.status}`}>{ui.artifacts.status[artifact.status]}</span>
            <div>
              <strong>{artifact.label}</strong>
              <small>{artifact.detail}</small>
            </div>
          </div>
        ))}
      </div>
    </section>
  )
}

export function KeyValueList({ rows }: { rows: Array<[string, string]> }) {
  return (
    <dl className="kv-list">
      {rows.map(([key, value]) => (
        <div key={key}>
          <dt>{key}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  )
}
