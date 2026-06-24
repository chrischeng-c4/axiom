# cue

## Brief

Web-based Prompt-to-Governed-Artifact control plane for internal work.

Cue is no longer positioned as a terminal SDD runner. It is a browser-based
enterprise control plane where teams describe internal work in natural
language, review a structured artifact graph, pass policy and approval gates,
and publish governed runtime artifacts such as workflow apps.

Cue is an AW Core client, not an `aw CLI` wrapper. It uses the shared Project,
WorkItem, Artifact, Gate, Evidence, HITL, and Rollup semantics, but its
web/backend product owns sessions, collaboration, artifact graph review,
approvals, registry, audit, ownership, runtime governance, and hidden app repo
orchestration. The CLI remains the repo-local developer and coding-agent
surface.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Prompt To WorkItem Artifact Graph | - | implemented | verified | smoke | not_ready | session prompt, WorkItem, PRD, TD, and runtime artifact dependency flow |
| Artifact Studio Front Office | - | implemented | verified | smoke | not_ready | project-owner browser workspace for intent, graph review, sandbox, and production requests |
| Admin Governance Back Office | - | implemented | verified | smoke | not_ready | platform review queues, grants, release gates, diagnostics, and audit evidence |
| Hidden App Repo And Registry | - | implemented | verified | smoke | not_ready | hidden GitLab app artifacts, App Spec template, registry, and release mapping |
| Policy Permission And Runtime Tenancy | - | implemented | verified | smoke | not_ready | ownership, permissions, policy, risk, connector, and runtime data contracts |
| Backend Mamba And Agent Team Boundary | - | implemented | verified | smoke | not_ready | Mamba-target backend API and temporary bridge around the future agent-team boundary |

### Prompt To WorkItem Artifact Graph

ID: prompt-to-workitem-artifact-graph
Type: AgentFirst
Surfaces: Web UI: Artifact Studio - session prompt, WorkItem, PRD, TD, runtime artifact dependency graph; HTTP API: Cue backend workstream routes - durable WorkItem and artifact state
EC Dimensions: behavior: projects/cue/backend/tests/test_workstream_api.py - WorkItem and artifact API contract
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cue turns user intent into a durable WorkItem and reviewed artifact graph before PRD, TD, runtime, test, deployment, or approval work can proceed.
Gate Inventory: projects/cue/backend/tests/test_workstream_api.py; projects/cue/e2e/full-product.e2e.mjs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| WorkItem and artifact graph contract | epic | - | implemented | verified | smoke | projects/cue/backend/tests/test_workstream_api.py; projects/cue/e2e/full-product.e2e.mjs |

### Artifact Studio Front Office

ID: artifact-studio-front-office
Type: AgentFirst
Surfaces: Web UI: `projects/cue/artifact-studio` - project-owner frontend for intent, graph review, sandbox, and production request flow
EC Dimensions: behavior: projects/cue/artifact-studio/tsconfig.json - Artifact Studio route/component type contract
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Artifact Studio gives project owners a browser workspace for describing intent, reviewing WorkItem-to-runtime artifacts, trying sandbox output, and requesting production.
Gate Inventory: projects/cue/artifact-studio/tsconfig.json; projects/cue/artifact-studio/e2e/workitems.e2e.mjs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Artifact Studio frontend contract | epic | - | implemented | verified | smoke | projects/cue/artifact-studio/tsconfig.json; projects/cue/artifact-studio/e2e/workitems.e2e.mjs |

### Admin Governance Back Office

ID: admin-governance-back-office
Type: Service
Surfaces: Web UI: `projects/cue/admin` - platform review queue, grants, release gates, diagnostics, and audit evidence; HTTP API: Cue backend Admin routes - review ticket and governance data
EC Dimensions: behavior: projects/cue/admin/tsconfig.json - Admin UI type contract; behavior: projects/cue/backend/tests/test_goal_governance_api.py - governance API contract
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cue Admin gives platform operators review queues, SaaS/API/resource grant workflows, release gates, diagnostics, and audit evidence for governed app delivery.
Gate Inventory: projects/cue/admin/tsconfig.json; projects/cue/backend/tests/test_goal_governance_api.py

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Admin UI and governance API | epic | - | implemented | verified | smoke | projects/cue/admin/tsconfig.json; projects/cue/backend/tests/test_goal_governance_api.py |

### Hidden App Repo And Registry

ID: hidden-app-repo-and-registry
Type: Service
Surfaces: Generated app template: `projects/cue/app-repo-template` - hidden GitLab repo contents; HTTP API: Cue backend registry/template routes - app state and release mapping
EC Dimensions: behavior: projects/cue/backend/tests/test_template_library_api.py - template and registry API contract
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cue owns generated app artifacts through hidden GitLab project templates and a registry that maps business app identity to source refs, sandbox refs, and production releases.
Gate Inventory: projects/cue/app-repo-template; projects/cue/schemas/app-spec.v0.schema.json; projects/cue/backend/tests/test_template_library_api.py

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Hidden repo template and registry mapping | epic | - | implemented | verified | smoke | projects/cue/app-repo-template; projects/cue/schemas/app-spec.v0.schema.json; projects/cue/backend/tests/test_template_library_api.py |

### Policy Permission And Runtime Tenancy

ID: policy-permission-and-runtime-tenancy
Type: SecurityTool
Surfaces: Schema: Cue governance/runtime JSON schemas - ownership namespace, permissions, runtime tenancy, policy, and review ticket contracts
EC Dimensions: security: projects/cue/schemas/governance-contract.schema.json - governed approval and policy schema; stability: projects/cue/examples/tracker-runtime-tenant.v0.json - app/env/version runtime tenancy example
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cue constrains generated apps with ownership namespace, policy, permission, connector, risk-tier, approval, and runtime-tenancy contracts before production access is granted.
Gate Inventory: projects/cue/schemas/governance-contract.schema.json; projects/cue/schemas/ownership-namespace.v0.schema.json; projects/cue/schemas/runtime-tenant.v0.schema.json; projects/cue/examples/tracker-runtime-tenant.v0.json

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Governance and runtime tenancy schemas | epic | - | implemented | verified | smoke | projects/cue/schemas/governance-contract.schema.json; projects/cue/schemas/ownership-namespace.v0.schema.json; projects/cue/schemas/runtime-tenant.v0.schema.json; projects/cue/examples/tracker-runtime-tenant.v0.json |

### Backend Mamba And Agent Team Boundary

ID: backend-mamba-and-agent-team-boundary
Type: Service
Surfaces: HTTP API: `projects/cue/backend` - Mamba-target backend API and temporary bridge boundary; Agent API: `mambalibs.agentkit` task/result envelope - governed agent-team execution contract
EC Dimensions: behavior: projects/cue/backend/tests - backend contract suite; behavior: projects/cue/be/tests/test_agent_team.py - agent-team contract preview
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cue keeps backend product code behind the future Mamba and AgentKit task/result boundary while using the temporary bridge only as compatible scaffolding for thin product slices.
Gate Inventory: projects/cue/backend/tests; projects/cue/backend/src/mambalibs; projects/cue/be/tests/test_agent_team.py

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Backend API and bridge contract | epic | - | implemented | verified | smoke | projects/cue/backend/tests; projects/cue/backend/src/mambalibs; projects/cue/be/tests/test_agent_team.py |
| Agent team task/result envelope | epic | - | implemented | verified | smoke | projects/cue/be/tests/test_agent_team.py |

## Product Definition

```text
Prompt -> WorkItem -> PRD Artifact -> TD Artifact -> Runtime Artifact -> Hidden Project Repo -> Policy/Test -> Sandbox -> Approval -> Production
```

Cue exists to let business teams turn intent into governed internal work:
WorkItems, PRDs, TDs, trackers, approvals, triage queues, dashboards, and
lightweight workflows without creating a new shadow IT layer. Every generated
runtime artifact is registered, versioned, permissioned, audited, owned, and
backed by a hidden GitLab project.

Business users do not see GitLab. Cue owns the repo, CI, release refs, runtime
deployment, registry, and audit trail.

## Core Principles

- **Conversation-first, artifact-backed**: app owners mainly speak or type
  intent, but Cue converts every accepted change into durable artifacts such as
  PRD, TD, App Spec, preview, policy, test, release, and audit records.
- **Session entry, WorkItem boundary**: app owners enter through Project
  Sessions. A WorkItem is the durable goal context attached to a Session and is
  the execution boundary for agents. Without an active WorkItem, Cue only
  creates or clarifies one; with an active WorkItem, Cue advances its delivery
  plan.
- **Flexible goal size, normalized tasks**: WorkItem granularity follows the
  user's goal boundary. "Reporting system" can be one WorkItem with import and
  export tasks, while another owner can split import and export into separate
  WorkItems. Cue normalizes each WorkItem into comparable todo-sized stage
  tasks.
- **Artifact dependencies before runtime**: PRD and TD are artifacts too. A
  runtime app or workflow artifact cannot start until its required upstream
  PRD and TD artifacts exist and pass their review gates.
- **One project, one governed agent team**: each project is assisted by a
  standard delivery group of PM, designer, developer, data, QA/policy, and
  release agents trained on Cue platform rules.
- **One project, one hidden artifact repo**: every project gets a hidden GitLab
  project that stores PRD, TD, runtime spec, policy, permissions, connector
  config, tests, generated assets, CI, and release metadata.
- **Runtime data is not Git**: records, comments, attachments, workflow state,
  usage, and runtime audit live in a runtime data substrate keyed by app,
  environment, version, and owner namespace.
- **Governance by default**: every project and runtime artifact has owner, risk
  tier, permissions, audit log, version history, deployment state, rollback
  target, and retirement policy.
- **Enterprise primitives over free code**: MVP apps are composed from approved
  primitives such as table, form, approval, workflow, SLA, notification, and
  dashboard.
- **Self-service, not free-for-all**: low-risk apps move quickly; higher-risk
  apps require explicit data-owner, security, platform, or legal approval.
- **Reuse before create**: new requests must check for similar apps and support
  use, fork, extend, or create-new decisions.

## Control Plane and Hidden App Repos

Cue has three layers:

| Layer | Source of truth | Purpose |
|-------|-----------------|---------|
| Cue control plane | This repo under `projects/cue/` and `.aw/tech-design/projects/cue/` | Product code, contracts, examples, and implementation specs |
| Generated app artifacts | Hidden GitLab project per generated app | Versioned App Spec, policy, permissions, connectors, tests, generated assets, CI, release refs |
| Runtime data | Runtime store outside Git | Records, comments, attachments, workflow state, usage, audit streams |

The Cue Registry maps business app identity to hidden repo identity:

```text
app_id -> gitlab_project_id -> current_spec_ref -> sandbox_ref -> production_release_tag
```

The UI can show version, changelog, health, owner, risk, and release status, but
it should not require business users to understand Git, branches, merge
requests, or CI details.

## MVP Surface

The first web product should ship two governed prompt routes before expanding
runtime generation:

```text
Prompt-to-WorkItem
  - create or update a durable goal context from a Session prompt
  - classify intent, project, owner, target artifacts, blockers, and stage plan
  - ask only the clarification needed to make the WorkItem actionable
  - do not execute PRD, TD, codebase, test, or deployment work before WorkItem creation
  - redirect general chat out of Cue

Prompt-to-PRD
  - require an active WorkItem as the execution boundary
  - generate a PRD artifact from the WorkItem and clarified intent
  - let the project owner review status, blockers, and next action
  - write lifecycle/audit state for downstream TD and runtime artifacts
```

Prompt-to-TD and prompt-to-runtime are next. Tracker, approval, triage, report,
and workflow artifacts should reuse the same WorkItem gate, artifact graph,
hidden repo, policy, permission, testing, deployment, registry, and runtime data
substrate.

## Local Validation

Run the local product dev servers with one command:

```bash
cd projects/cue
npm run dev
```

This starts the backend API on `http://127.0.0.1:43219`, Artifact Studio on
`http://127.0.0.1:3212`, and Admin on `http://127.0.0.1:3216`. The default
backend mode is Mamba. If the current Mamba substrate cannot serve the API yet,
use the explicit bridge fallback only for thin local product work:

```bash
cd projects/cue
CUE_BACKEND_MODE=bridge npm run dev
```

Run the current Cue product checks with one command:

```bash
cd projects/cue
npm run test:all
```

This runs backend contract tests, Artifact Studio typecheck, and browser e2e
against the deterministic fixture path. To exercise the temporary Claude Code
headless adapter for backend agent calls, set:

```bash
cd projects/cue
CUE_AGENT_PROVIDER=claude_headless npm run test:all
```

The Claude adapter calls `claude -p <prompt>` behind the `mambalibs.agentkit`
task/result boundary. Product code should continue to depend on the boundary,
not on the shell command directly.

## Web Product Areas

| Area | Purpose |
|------|---------|
| Artifact Studio | Project owner front office for describing intent, reviewing the WorkItem -> PRD -> TD -> runtime artifact dependency graph, trying Sandbox, requesting Production, and managing owned projects |
| Admin | Platform back office for review tickets, SaaS/API/resource grants, release gates, runtime visibility, workflow evidence, and diagnostics |
| Cue backend | API, jobs, source-code lifecycle orchestration, repo backend adapters, deploy adapters, registry, audit, and policy services |
| Generated apps | Independently built and deployed apps from hidden GitLab repos; they are not Cue frontend routes |

## Workspace Model

Cue splits workspaces by authority:

| Workspace | Owner | Can Do | Cannot Directly Grant |
|-----------|-------|--------|-----------------------|
| User workspace | App owner and assigned agent team | Turn conversation into reviewed WorkItems and artifacts: PRD, TD, runtime specs, implementation, tests, release packages, and Admin review tickets | deployment publication, SaaS API, `train_model`, costly compute, PII access, production runtime permission |
| Admin workspace | Platform admin | Review deployment tickets, SaaS API grants, resource budgets, runtime/data boundaries, policy exceptions, release gates | Business workflow edits owned by the app owner |

Users only describe intent. Cue agents organize the request into durable
artifacts and, when work is ready to deploy or needs sensitive access, emit
review tickets. The Admin workspace decides whether to grant the tool, SaaS API,
data scope, runtime permission, or resource budget to that workspace.

## Required Platform Capabilities

| Capability | Why it matters |
|------------|----------------|
| WorkItem lifecycle | Required ticket and routing state for every prompt-to-X flow |
| App Spec schema | Contract that generation, runtime, policy, and tests share; canonical v0 lives at [`schemas/app-spec.v0.schema.json`](schemas/app-spec.v0.schema.json) |
| Hidden app repo provisioner | Creates and manages the GitLab project per app while keeping it invisible to users |
| App repo template | Standardizes `app-spec.json`, `policy.json`, `permissions.json`, `connectors.json`, tests, generated assets, CI, and release metadata |
| App Registry | Stores current app state plus the hidden GitLab project/ref/release mapping |
| Ownership namespace | Distinguishes personal, team, cross-team, and platform-owned apps; canonical v0 lives at [`schemas/ownership-namespace.v0.schema.json`](schemas/ownership-namespace.v0.schema.json) |
| Runtime data tenancy | Keeps records and events outside Git with app/env/version tenancy; canonical v0 lives at [`schemas/runtime-tenant.v0.schema.json`](schemas/runtime-tenant.v0.schema.json) |
| Policy engine | Blocks unsafe data, permission, automation, and deployment choices |
| Risk tiering | Routes approvals and blocks Tier 4 MVP use cases |
| Permission model | App/entity/field/record/action/environment access control |
| Connector catalog | Prevents generated apps from direct database access |
| Test harness | Verifies spec, permission, workflow, policy, and regression behavior |
| Deployment manager | Sandbox, production, rollback, emergency disable |
| Audit and observability | Trace usage, failures, permission changes, cost, and lifecycle events |

## Target Product Layout

```text
projects/cue/
  artifact-studio/ Project owner frontend site on Jet
  admin/           Platform admin frontend site on Jet
  backend/         Cue API, jobs, adapters, and domain logic on Mamba/mambalibs.http
    src/mambalibs/ Temporary Python preview of future Mamba library contracts
  shared/          Shared domain types/API client/status mapping only
  schemas/         Artifact and runtime contracts
  examples/        Versioned example manifests
  docs/            Legacy terminal/TUI notes only

.aw/tech-design/projects/cue/
  README.md Product architecture and implementation sequence
  *.md      Cue TD files for governance, lifecycle, UI, backend, and delivery models
  references/ External research and reference context
```

Current implementation is still transitioning from the earlier `fe/` and `be/`
scaffold. The product direction is two separate frontend sites plus one backend:

```text
artifact-studio FE -> backend API
admin FE           -> backend API
backend            -> repo backend / DB / deploy / agents / policy
```

Generated apps are outside the Cue repo:

```text
cue-generated-apps/
  teams/<team>/<app>/
  personal/<user>/<app>/
```

Planned generated-app infrastructure:

```text
projects/cue/app-repo-template/   hidden GitLab repo template, not created yet
```

Cue owns generated app source-code lifecycle through hidden GitLab projects. App
owners do not see GitLab, CI, branches, release refs, database names, or deploy
internals; they see needs, proposals, preview, Sandbox results, approval
status, and owned apps. There is no Rust product crate under `projects/cue`;
Rust remains platform substrate elsewhere in cclab.

Artifact Studio commands run from `projects/cue/artifact-studio` when you need
to work on that app in isolation:

```bash
npm run typecheck
npm run dev
npm run build
```

The legacy `projects/cue/fe` scaffold remains transitional until Artifact
Studio and Admin have separate first-class app directories. The backend target
is `projects/cue/backend`; the existing `projects/cue/be` scaffold is also
transitional.

The backend control-plane MVP is implemented under `projects/cue/backend`.
It exposes WorkItem and PRD artifact routes for Artifact Studio plus Admin
evidence routes for blockers, repository state, review tickets, diagnostics,
and audit events. Run its contract tests from the repo root:

```bash
uv run --with pytest python -m pytest projects/cue/backend/tests/test_workstream_api.py -q
```

## Relationship to cclab

Cue's product stack is fixed:

```text
Backend: Mamba
Frontend: Jet
Generated app artifact store: hidden GitLab projects
Runtime data: app-aware runtime substrate, not Git
Development lifecycle for Cue itself: score
```

Mamba owns backend application code: APIs, App Spec services, hidden app repo
provisioning, policy/risk evaluation, registry, deployment workflow, runtime
tenancy, audit, connector orchestration, and test harness logic.

Cue's backend should call SDD and cclab-agent through Mamba library surfaces,
not by shelling out to their CLIs. The CLI remains a developer/operator surface;
the product runtime needs stable in-process APIs for work item state,
artifact transitions, agent orchestration, and review-ticket emission.

The managed runtime store target is AlloyDB. Local development and default
contract tests use PostgreSQL; Cue should avoid requiring local AlloyDB.
Managed Cue defaults to one shared AlloyDB cluster per environment and region,
with each generated app isolated by its own database, schema, and runtime role.
Dedicated clusters are platform-approved exceptions for high-risk, high-load,
residency, backup, encryption, or isolation requirements.

Jet owns the Cue browser product surface: web shell, routes, build pipeline,
dev server, bundling, assets, and frontend test/dev workflow. Vite is only a
minimal repro path when isolating a Jet substrate blocker. If Vite works and Jet
does not, open or link a `project:jet` / `crate:jet` issue with the exact repro,
runtime error, and expected behavior.

Rust remains substrate/tooling, not Cue product code. Cue should use
Mamba-facing mambalibs bindings for backend capabilities wherever they exist:

| Backend capability | Mamba-facing dependency |
|--------------------|-------------------------|
| HTTP API | `httpkit` (`mambalibs.http`) |
| Postgres persistence | `pgkit` |
| App Spec validation | `cclab-schema-mamba` |
| SDD work items, artifacts, state machine, review gates | `sdd-mamba` |
| Runtime / async | `cclab-runtime-mamba` |
| Data fetching / connector calls | `cclab-fetch-mamba` |
| Governed agent teams, task execution, artifact production, review tickets | `cclab-agent-mamba` |
| Logging / observability | `cclab-log-mamba` |
| Test harness | `cclab-qc-mamba` |

The intended product call path is:

```text
Cue Mamba API
  -> sdd-mamba        # durable work item, spec/artifact state, review gates
  -> cclab-agent-mamba # PM/designer/dev/data/QA/release agent execution
  -> Cue services     # hidden repo, registry, deployment, runtime tenancy, audit
```

`sdd-mamba` should expose the artifact/state contract; `cclab-agent-mamba`
should consume that contract and return structured outputs plus Admin review
tickets. Cue owns presentation, workspace policy, and ticket approval.
AgentKit core is the target agent interface: typed `Agent<Deps, Output>`,
provider abstraction, tool calls, and structured response schema. The current
`cclab.agent` Mamba binding exposes builder/provider/team/schema handles, but
`run` and `team_run` are still stubbed. Until those are live, local Cue may
route through deterministic generation or a Claude headless adapter behind the
same task/result envelope. Cue product code should not call `claude -p`
directly.

Mamba readiness is a release gate for Cue backend work. Until the relevant
Mamba path is stable, Cue should prefer thin vertical slices and explicit
readiness issues over broad dependency fan-out across every cclab library.

Mamba follows the same validation rule as Jet. When backend behavior looks like
a Mamba substrate issue, verify the equivalent contract in CPython or a mature
reference stack first. If the reference path works and Mamba fails, isolate the
minimal repro and open or link a `project:mamba` / `crate:mamba` issue. Cue may
carry a temporary bridge only to unblock a thin product slice, and the bridge
must keep the future Mamba API and artifact contracts intact.

For MVP speed, a CPython backend bridge is allowed only as explicit temporary
scaffolding when the equivalent Mamba path is blocked. The bridge must obey the
future Mamba API shape, keep App Spec, hidden repo, policy, registry, runtime
tenancy, and audit contracts portable, and have a linked Mamba migration issue
before it lands. CPython is not the product backend target.

The temporary Python bridge keeps future Mamba-facing code under
`projects/cue/backend/src/mambalibs/`. That package is a contract preview, not a
separate framework. Cue services call its typed task/result boundary; later
Mamba builds can replace the package implementation without changing WorkItem
or frontend contracts. `mambalibs.pgkit` is the Postgres preview surface.

`score` is still used to build Cue itself, not as Cue's end-user workflow.
Generated business apps run Cue's App Spec and app release lifecycle.

## Legacy Notes

The old Rust TUI/CRRR prototype has been removed from the active Cue product
workspace. Its design notes live under [`docs/legacy/`](docs/legacy/) as
implementation history only.
The retirement contract and deletion plan live in
[`docs/legacy/RETIREMENT.md`](docs/legacy/RETIREMENT.md). New Cue product work
must not extend TUI, CRRR driver, transport profile, or SDD app-protocol
tracking.

See [`../../.aw/tech-design/projects/cue/README.md`](../../.aw/tech-design/projects/cue/README.md)
for the web control-plane architecture.
See [`../../.aw/tech-design/projects/cue/governance-contract.md`](../../.aw/tech-design/projects/cue/governance-contract.md)
for lifecycle, risk, approval, registry, and audit rules.
See [`../../.aw/tech-design/projects/cue/web-mvp-user-story.md`](../../.aw/tech-design/projects/cue/web-mvp-user-story.md)
for the first Prompt Builder, App Studio, sandbox, approval, and Registry user story.
See [`../../.aw/tech-design/projects/cue/cue-user-story-contract.md`](../../.aw/tech-design/projects/cue/cue-user-story-contract.md)
for the Cue-local user story contract pending SDD `user-story` section support.
See [`examples/tracker-app-spec.v0.json`](examples/tracker-app-spec.v0.json)
for the first tracker vertical manifest shape.
See [`examples/team-ownership-namespace.v0.json`](examples/team-ownership-namespace.v0.json)
for the first team-owned app ownership metadata shape.
See [`examples/tracker-runtime-tenant.v0.json`](examples/tracker-runtime-tenant.v0.json)
for the first Tracker runtime tenant metadata shape.
