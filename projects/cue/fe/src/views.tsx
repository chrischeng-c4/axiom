import { type FormEvent } from 'react'
import Activity from 'lucide-react/dist/esm/icons/activity.js'
import ArrowRight from 'lucide-react/dist/esm/icons/arrow-right.js'
import Check from 'lucide-react/dist/esm/icons/check.js'
import CheckCircle2 from 'lucide-react/dist/esm/icons/check-circle-2.js'
import ClipboardCheck from 'lucide-react/dist/esm/icons/clipboard-check.js'
import Database from 'lucide-react/dist/esm/icons/database.js'
import FileText from 'lucide-react/dist/esm/icons/file-text.js'
import Filter from 'lucide-react/dist/esm/icons/filter.js'
import Gauge from 'lucide-react/dist/esm/icons/gauge.js'
import ListChecks from 'lucide-react/dist/esm/icons/list-checks.js'
import Lock from 'lucide-react/dist/esm/icons/lock.js'
import Play from 'lucide-react/dist/esm/icons/play.js'
import RefreshCw from 'lucide-react/dist/esm/icons/refresh-cw.js'
import Rocket from 'lucide-react/dist/esm/icons/rocket.js'
import Search from 'lucide-react/dist/esm/icons/search.js'
import Send from 'lucide-react/dist/esm/icons/send.js'
import ShieldCheck from 'lucide-react/dist/esm/icons/shield-check.js'
import SlidersHorizontal from 'lucide-react/dist/esm/icons/sliders-horizontal.js'
import Table2 from 'lucide-react/dist/esm/icons/table-2.js'
import {
  clarificationItems,
  deliveryTeam,
  ownershipNamespace,
  runtimeTenant,
  studioTabs,
  type ClarificationId,
  type ClarificationState,
} from './mockData'
import { Badge, GovernanceList, KeyValueList, PreviewTable } from './components'
import type { CueCopy } from './i18n'
import type {
  AdminReviewTicket,
  AppSpec,
  GovernanceCheck,
  Persona,
  StoryState,
  StudioTab,
} from './types'

export function PromptBuilder({
  clarifications,
  prompt,
  storyState,
  ui,
  onGenerate,
  onPromptChange,
  onSeed,
  onToggleClarification,
}: {
  clarifications: ClarificationState
  prompt: string
  storyState: StoryState
  ui: CueCopy
  onGenerate: (event: FormEvent<HTMLFormElement>) => void
  onPromptChange: (value: string) => void
  onSeed: () => void
  onToggleClarification: (id: ClarificationId) => void
}) {
  const missing = clarificationItems.filter((item) => !clarifications[item.id]).length

  return (
    <section className="route-grid two-col">
      <form className="tool-panel prompt-panel" onSubmit={onGenerate}>
        <div className="panel-heading">
          <div>
            <p className="eyebrow">{ui.promptBuilder.eyebrow}</p>
            <h2>{ui.promptBuilder.title}</h2>
          </div>
          <ClipboardCheck aria-hidden="true" />
        </div>
        <label className="field-label" htmlFor="prompt-input">
          {ui.promptBuilder.conversationLabel}
        </label>
        <textarea id="prompt-input" value={prompt} onChange={(event) => onPromptChange(event.target.value)} />
        <div className="action-row">
          <button className="primary-action" type="submit">
            <Play aria-hidden="true" />
            {ui.promptBuilder.generateDraft}
          </button>
          <button className="secondary-action" type="button" onClick={onSeed}>
            <RefreshCw aria-hidden="true" />
            {ui.promptBuilder.loadExample}
          </button>
        </div>
      </form>

      <section className="tool-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">{ui.promptBuilder.clarificationEyebrow}</p>
            <h2>{missing === 0 ? ui.promptBuilder.readyForStudio : ui.promptBuilder.answersPending(missing)}</h2>
          </div>
          <ListChecks aria-hidden="true" />
        </div>
        <div className="checklist">
          {clarificationItems.map((item) => {
            const itemCopy = ui.promptBuilder.clarifications[item.id]
            return (
              <label className="toggle-row" key={item.id}>
                <input
                  checked={clarifications[item.id]}
                  onChange={() => onToggleClarification(item.id)}
                  type="checkbox"
                />
                <span>
                  <strong>{itemCopy.label}</strong>
                  <small>{itemCopy.value}</small>
                </span>
              </label>
            )
          })}
        </div>
        <div className={storyState === 'Clarifying' ? 'notice amber' : 'notice'}>
          <ShieldCheck aria-hidden="true" />
          <span>
            {storyState === 'Clarifying' ? ui.promptBuilder.governanceRequired : ui.promptBuilder.draftInReview}
          </span>
        </div>
        <div className="agent-team-list" aria-label={ui.promptBuilder.agentTeamAria}>
          {deliveryTeam.map((agent) => {
            const agentCopy = ui.promptBuilder.agentTeam[agent.id]
            return (
              <div className={`finding-row ${agent.state}`} key={agent.id}>
                <CheckCircle2 aria-hidden="true" />
                <span>{agentCopy.label}</span>
                <strong>{agentCopy.value}</strong>
              </div>
            )
          })}
        </div>
      </section>
    </section>
  )
}

export function StudioView({
  activeTab,
  checks,
  draftSpec,
  persona,
  ui,
  onRequestProduction,
  onRequestSandbox,
  onTabChange,
}: {
  activeTab: StudioTab
  checks: GovernanceCheck[]
  draftSpec: AppSpec
  persona: Persona
  ui: CueCopy
  onRequestProduction: () => void
  onRequestSandbox: () => void
  onTabChange: (tab: StudioTab) => void
}) {
  const entity = draftSpec.data.entities[0]
  const blockers = checks.filter((check) => check.state === 'block').length

  return (
    <section className="route-grid studio-grid">
      <section className="tool-panel preview-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">{ui.studio.previewEyebrow}</p>
            <h2>{ui.studio.previewTitle}</h2>
          </div>
          <Table2 aria-hidden="true" />
        </div>
        <PreviewTable ariaLabel={ui.previewTable.ariaLabel} headers={ui.previewTable.headers} />
      </section>

      <section className="tool-panel">
        <div className="panel-heading compact">
          <div>
            <p className="eyebrow">{ui.studio.editorEyebrow}</p>
            <h2>{entity.name}</h2>
          </div>
          <SlidersHorizontal aria-hidden="true" />
        </div>
        <div className="tabs" role="tablist" aria-label={ui.studio.tabsAria}>
          {studioTabs.map((tab) => (
            <button
              className={activeTab === tab ? 'tab active' : 'tab'}
              key={tab}
              onClick={() => onTabChange(tab)}
              role="tab"
              type="button"
            >
              {ui.studio.tabs[tab]}
            </button>
          ))}
        </div>
        <StudioTabContent activeTab={activeTab} draftSpec={draftSpec} ui={ui} />
      </section>

      <section className="tool-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">{ui.studio.riskEyebrow}</p>
            <h2>{blockers === 0 ? ui.studio.sandboxEligible : ui.studio.blocked}</h2>
          </div>
          <Gauge aria-hidden="true" />
        </div>
        <GovernanceList checks={checks} />
        <div className="notice">
          <ShieldCheck aria-hidden="true" />
          <span>{persona === 'platform' ? ui.studio.platformGuidance : ui.studio.ownerGuidance}</span>
        </div>
        <div className="action-row sticky-actions">
          <button className="primary-action" disabled={blockers > 0} onClick={onRequestSandbox} type="button">
            <Rocket aria-hidden="true" />
            {ui.studio.requestSandbox}
          </button>
          <button className="secondary-action" onClick={onRequestProduction} type="button">
            <Send aria-hidden="true" />
            {ui.studio.requestProduction}
          </button>
        </div>
      </section>
    </section>
  )
}

function StudioTabContent({ activeTab, draftSpec, ui }: { activeTab: StudioTab; draftSpec: AppSpec; ui: CueCopy }) {
  if (activeTab === 'workflow') {
    return (
      <KeyValueList
        rows={draftSpec.workflow.transitions.map((step) => [
          step.from,
          `${step.to} ${ui.studio.workflowTransitionBy} ${step.actor_role}`,
        ])}
      />
    )
  }
  if (activeTab === 'permissions') {
    return (
      <KeyValueList
        rows={draftSpec.permissions.roles.map((role) => [
          role.name,
          ui.studio.permissionsCount(role.permissions.length),
        ])}
      />
    )
  }
  if (activeTab === 'notifications') {
    return <KeyValueList rows={draftSpec.automation.triggers.map((trigger) => [trigger.type, trigger.condition])} />
  }
  if (activeTab === 'dashboard') {
    return <KeyValueList rows={draftSpec.dashboard.views[0].metrics.map((metric) => [metric.name, metric.formula])} />
  }
  return <KeyValueList rows={draftSpec.data.entities[0].fields.map((field) => [field.name, `${field.type} / ${field.sensitivity}`])} />
}

export function SandboxView({
  persona,
  ui,
  onRequestProduction,
}: {
  persona: Persona
  ui: CueCopy
  onRequestProduction: () => void
}) {
  return (
    <section className="route-grid sandbox-grid">
      <section className="tool-panel preview-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">{ui.sandbox.eyebrow}</p>
            <h2>Team request tracker</h2>
          </div>
          <Activity aria-hidden="true" />
        </div>
        <div className="sandbox-toolbar">
          <Badge label={ui.sandbox.sampleData} tone="blue" />
          <Badge label={runtimeTenant.storage.local_backend} tone="green" />
          <button className="icon-action" aria-label={ui.sandbox.resetAria} type="button">
            <RefreshCw aria-hidden="true" />
          </button>
        </div>
        <PreviewTable ariaLabel={ui.previewTable.ariaLabel} headers={ui.previewTable.headers} />
      </section>
      <section className="tool-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">
              {persona === 'platform' ? ui.sandbox.runtimeTenantEyebrow : ui.sandbox.ownerReviewEyebrow}
            </p>
            <h2>{persona === 'platform' ? runtimeTenant.environment : ui.sandbox.ownerReviewTitle}</h2>
          </div>
          <Database aria-hidden="true" />
        </div>
        {persona === 'platform' ? (
          <KeyValueList
            rows={[
              [ui.sandbox.rows.tenant, runtimeTenant.runtime_tenant_id],
              [ui.sandbox.rows.cluster, `${runtimeTenant.storage.cluster_mode} / ${runtimeTenant.storage.cluster_id}`],
              [ui.sandbox.rows.database, runtimeTenant.storage.database_name],
              [ui.sandbox.rows.isolation, runtimeTenant.storage.isolation_unit],
              [ui.sandbox.rows.retention, ui.sandbox.rows.dayArchive(runtimeTenant.retention.archive_after_days)],
              [ui.sandbox.rows.migration, runtimeTenant.migration.state],
              [ui.sandbox.rows.families, ui.sandbox.rows.runtimeFamilies(runtimeTenant.runtime_families.length)],
            ]}
          />
        ) : (
          <KeyValueList rows={ui.sandbox.ownerReviewRows.map((row) => [row.label, row.value])} />
        )}
        <div className="notice">
          <ShieldCheck aria-hidden="true" />
          <span>{persona === 'platform' ? ui.sandbox.platformGuidance : ui.sandbox.ownerGuidance}</span>
        </div>
        <button className="primary-action full-width" onClick={onRequestProduction} type="button">
          <Send aria-hidden="true" />
          {ui.sandbox.requestProduction}
        </button>
      </section>
    </section>
  )
}

export function RegistryView({
  draftSpec,
  persona,
  storyState,
  ui,
  onOpenStudio,
}: {
  draftSpec: AppSpec
  persona: Persona
  storyState: StoryState
  ui: CueCopy
  onOpenStudio: () => void
}) {
  return (
    <section className="tool-panel registry-view">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">{ui.registry.eyebrow}</p>
          <h2>{ui.registry.title}</h2>
        </div>
        <Filter aria-hidden="true" />
      </div>
      <div className="filter-bar">
        <div className="search-box">
          <Search aria-hidden="true" />
          <input aria-label={ui.registry.searchAria} value="Team Request Tracker" readOnly />
        </div>
        {ui.registry.filters.map((filter) => (
          <button className="filter-chip" type="button" key={filter}>
            {filter}
          </button>
        ))}
      </div>
      <div className="notice registry-notice">
        <ShieldCheck aria-hidden="true" />
        <span>{persona === 'platform' ? ui.registry.platformView : ui.registry.ownerView}</span>
      </div>
      <div className="data-table registry-table" role="table" aria-label={ui.registry.tableAria}>
        <div className="table-head" role="row">
          {ui.registry.headers.map((header) => (
            <span key={header}>{header}</span>
          ))}
        </div>
        <div className="table-row" role="row">
          <span>{ownershipNamespace.display_name}</span>
          <span>{draftSpec.app.owner_team}</span>
          <span>{ownershipNamespace.namespace}</span>
          <span>{storyState === 'ProductionReady' ? ui.registry.production : runtimeTenant.environment}</span>
          <span>{draftSpec.app.risk_tier}</span>
          <span>{ui.registry.healthy}</span>
          <span>v{runtimeTenant.app_version}</span>
          <button className="text-action" onClick={onOpenStudio} type="button">
            {ui.registry.open} <ArrowRight aria-hidden="true" />
          </button>
        </div>
      </div>
    </section>
  )
}

export function AdminView({
  persona,
  reviewTickets,
  ui,
  onApprove,
  onApproveTicket,
}: {
  persona: Persona
  reviewTickets: AdminReviewTicket[]
  ui: CueCopy
  onApprove: () => void
  onApproveTicket: (ticketId: string) => void
}) {
  return (
    <section className="route-grid admin-grid">
      <section className="tool-panel review-ticket-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">{ui.admin.reviewEyebrow}</p>
            <h2>{ui.admin.reviewQueue}</h2>
          </div>
          <ShieldCheck aria-hidden="true" />
        </div>
        <div className="notice">
          <Lock aria-hidden="true" />
          <span>{ui.admin.reviewGuidance}</span>
        </div>
        <div className="review-ticket-list">
          {reviewTickets.map((ticket) => (
            <article className="review-ticket" key={ticket.id}>
              <div className="review-ticket-heading">
                <div>
                  <p className="eyebrow">{ui.admin.ticketKind[ticket.kind]}</p>
                  <h3>{ticket.title}</h3>
                </div>
                <div className="status-strip compact-status">
                  <Badge label={ui.admin.ticketStatus[ticket.status]} tone={ticket.status === 'approved' ? 'green' : 'amber'} />
                  <Badge label={ticket.risk} tone={ticket.risk === 'high' ? 'red' : 'amber'} />
                </div>
              </div>
              <KeyValueList
                rows={[
                  [ui.admin.reviewRows.request, ticket.id],
                  [ui.admin.reviewRows.workspace, ticket.workspace_id],
                  [ui.admin.reviewRows.app, ticket.app_id],
                  [ui.admin.reviewRows.requestedBy, ticket.requested_by],
                  [ui.admin.reviewRows.kind, ui.admin.ticketKind[ticket.kind]],
                  [ui.admin.reviewRows.resource, ticket.resource],
                  [ui.admin.reviewRows.environment, ticket.environment_scope],
                  [ui.admin.reviewRows.data, ticket.data_scope],
                  [ui.admin.reviewRows.agentOutput, ticket.agent_output],
                  [ui.admin.reviewRows.rationale, ticket.rationale],
                ]}
              />
              <button
                className="primary-action full-width"
                disabled={ticket.status === 'approved'}
                onClick={() => onApproveTicket(ticket.id)}
                type="button"
              >
                <Check aria-hidden="true" />
                {ticket.status === 'approved' ? ui.admin.ticketApproved : ui.admin.approveTicket}
              </button>
            </article>
          ))}
        </div>
      </section>
      <section className="tool-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">{ui.admin.approvalEyebrow}</p>
            <h2>{ui.admin.productionRequest}</h2>
          </div>
          <ShieldCheck aria-hidden="true" />
        </div>
        <KeyValueList rows={ui.admin.releaseEvents.map((event) => [event.label, event.value])} />
        <div className="notice">
          <ShieldCheck aria-hidden="true" />
          <span>{persona === 'platform' ? ui.admin.platformMutable : ui.admin.ownerReadonly}</span>
        </div>
        <button className="primary-action full-width" onClick={onApprove} type="button">
          <Check aria-hidden="true" />
          {ui.admin.approveRelease}
        </button>
      </section>
      <section className="tool-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">{ui.admin.policyEyebrow}</p>
            <h2>{ui.admin.findings}</h2>
          </div>
          <FileText aria-hidden="true" />
        </div>
        <div className="finding-list">
          {ui.admin.policyFindings.map((finding) => (
            <div className={`finding-row ${finding.state}`} key={finding.label}>
              <CheckCircle2 aria-hidden="true" />
              <span>{finding.label}</span>
              <strong>{finding.value}</strong>
            </div>
          ))}
        </div>
      </section>
      <section className="tool-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">{ui.admin.platformEyebrow}</p>
            <h2>{ui.admin.hiddenInfrastructure}</h2>
          </div>
          <Lock aria-hidden="true" />
        </div>
        <KeyValueList
          rows={[
            [ui.admin.hiddenRows.gitlabProject, ownershipNamespace.gitlab_mapping.full_path],
            [ui.admin.hiddenRows.userVisible, String(ownershipNamespace.gitlab_mapping.user_visible)],
            [ui.admin.hiddenRows.runtimeCluster, runtimeTenant.storage.cluster_id],
            [ui.admin.hiddenRows.runtimeDatabase, runtimeTenant.storage.database_name],
            [
              ui.admin.hiddenRows.quota,
              ui.admin.hiddenRows.sandboxApps(ownershipNamespace.quota_state.sandbox_apps_count),
            ],
            [ui.admin.hiddenRows.emergencyContact, ownershipNamespace.emergency_contact],
          ]}
        />
      </section>
    </section>
  )
}
