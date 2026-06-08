import React from 'react'
import { createRoot } from 'react-dom/client'
import './styles.css'

const panes = [
  ['Review queue', 'Open approvals, policy exceptions, and production requests.'],
  ['Evidence', 'Audit events, WorkItem facts, registry snapshots, and hidden repo refs.'],
  ['Blockers', 'Missing owner context, backend blocks, policy exceptions, and approval delays.'],
  ['Policy and tests', 'Schema, permission, connector, regression, and risk gates.'],
  ['Repo and runtime diagnostics', 'Hidden repo refs, runtime tenants, usage, errors, workflow traces, and health score.'],
  ['Controls', 'Approve, deploy, rollback, disable, archive, and retire governed artifacts.'],
]

function AdminApp() {
  return (
    <main className="shell">
      <aside className="rail">
        <strong>Cue Admin</strong>
        <span>Platform operations</span>
      </aside>
      <section className="queue" aria-label="Review queue">
        <header>
          <span>Governance console</span>
          <h1>Review queue</h1>
        </header>
        {panes.map(([title, body]) => (
          <article key={title}>
            <h2>{title}</h2>
            <p>{body}</p>
          </article>
        ))}
      </section>
      <section className="evidence" aria-label="Evidence pane">
        <h2>Evidence pane</h2>
        <dl>
          <div>
            <dt>Policy</dt>
            <dd>Blocked until required approvals resolve.</dd>
          </div>
          <div>
            <dt>Deployment</dt>
            <dd>Sandbox, production, rollback, and disable actions are controlled here.</dd>
          </div>
          <div>
            <dt>Repository</dt>
            <dd>Hidden app repo refs are shown as audit evidence, not owner-facing UI.</dd>
          </div>
        </dl>
      </section>
    </main>
  )
}

createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <AdminApp />
  </React.StrictMode>,
)
