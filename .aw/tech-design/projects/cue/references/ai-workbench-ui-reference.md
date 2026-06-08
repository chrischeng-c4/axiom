# AI Workbench UI Reference

Status: reference
Accessed: 2026-05-08

This reference captures external UI patterns from official Firebase Studio and
Google AI Studio sources. It is durable reference context for Cue App Studio,
User Workspace, and Admin Workspace design work. It is not a product spec and
does not override Cue's governed App Spec, hidden repo, runtime tenancy, policy,
or approval-ticket contracts.

## Sources

| Source | URL | Reference use |
|--------|-----|---------------|
| Firebase Studio introduction | https://firebase.google.com/docs/studio | Browser-based agentic development environment, templates, preview/publish, and current deprecation status. |
| Firebase Studio App Prototyping agent | https://firebase.google.com/docs/studio/get-started-ai | Prompt-to-blueprint flow, blueprint review, generated preview, iterative refinement, publishing, rollback, and service provisioning patterns. |
| Google AI Studio Build mode | https://ai.google.dev/gemini-api/docs/aistudio-build-mode | Prompt-first app generation, App Gallery remix, live preview, Code tab, annotation/chat iteration, Cloud Run deploy, GitHub export, secrets, and API key warnings. |
| Firebase blog: AI Studio integration and Firebase Studio sunset | https://firebase.blog/posts/2026/03/announcing-ai-studio-integration | Confirms Firebase Studio should be treated as a pattern reference only; forward-looking comparison should lean on Google AI Studio and governed Cue needs. |

## Observed Patterns

| Pattern | Observation | Cue relevance |
|---------|-------------|---------------|
| Prompt-first workspace | The primary entry point is natural language. Firebase Studio also accepts images/drawings; AI Studio Build mode starts from a prompt or gallery app. | Cue should keep conversation as the main intake surface, but immediately convert accepted intent into durable App Spec artifacts. |
| Blueprint/spec review | Firebase Studio's App Prototyping agent generates a blueprint with proposed app name, features, and style guidelines before coding. | Cue should make requirements, ownership, fields, workflow, permissions, and data boundary reviewable before sandbox work starts. |
| Live preview plus artifacts | Generated apps show a live preview; AI Studio exposes generated code through a Code tab in the preview pane. | Cue should show preview alongside App Spec, workflow, permissions, tests, policy result, and ticket artifacts instead of only chat history. |
| Iterative edit loop | Both products support continued refinement through chat; AI Studio adds annotation mode and direct code editing. | Cue should allow user-language requests and controlled primitive edits, but every accepted change must become an artifact diff or proposal. |
| Deploy/test flow | Firebase Studio and AI Studio expose publish/deploy/export paths such as Firebase App Hosting, Cloud Run, ZIP, and GitHub. | Cue must split sandbox deploy, production request, Admin review, release tag, registry update, and rollback into visible governed states. |
| Resource and API setup | Firebase Studio can provision Firebase projects, API keys, Firestore, Authentication, and hosting; AI Studio documents secrets and API key limitations. | Cue should treat SaaS APIs, credentials, expensive resources, and dedicated infrastructure as Admin-reviewed permission/resource tickets. |
| Gallery/template/remix model | Firebase Studio offers templates/samples; AI Studio offers App Gallery copy/remix. | Cue should offer governed app templates and approved primitive bundles, not open-ended public remix behavior. |
| In-product warnings | Official docs warn that AI output must be validated and that client-side API keys are unsafe. | Cue should surface validation, test, policy, and security state as first-class UI instead of fine-print disclaimers. |

## Cue Adaptation

Cue should adapt the workbench pattern into two coordinated workspaces:

| Workspace | Owner | Primary job | Required surfaces |
|-----------|-------|-------------|-------------------|
| User Workspace | App owner | Describe need, review agent proposals, edit governed settings, request sandbox/production. | Conversation, questions, App Spec draft, preview, artifact diff, agent team, sandbox status, production request status. |
| Admin Workspace | Platform maintainer | Review policy/resource/risk tickets and approve, reject, or request changes. | Ticket queue, resource/API requests, hidden repo refs, runtime tenant binding, policy/test evidence, cost/risk labels, approval actions. |

The User Workspace should behave like a delivery-team interface. The owner can
say "build a training model request app" or "add Google Sheets export", but Cue
must translate that into structured requirements, resource needs, permission
changes, data contracts, tests, and review tickets.

The Admin Workspace should not be a second generic app builder. It is the
governance console for requests created by owner intent and agent execution:
production deploy, external SaaS API access, dedicated AlloyDB cluster, costly
compute, sensitive connector scope, cross-team data access, and policy
exceptions.

## Non-Goals

- Do not copy Firebase Studio as a product direction. It is officially being
  sunset, so it is useful only as a historical/workbench pattern reference.
- Do not adopt free-form vibe coding as Cue's core experience.
- Do not let app owners deploy directly to production without Admin ticket
  review.
- Do not expose GitLab projects, branches, pipelines, AlloyDB database names,
  or cloud credentials as app-owner concepts.
- Do not make credential or API key setup an owner task. Owners can request a
  capability; Admin and platform automation own approval and provisioning.
- Do not make model/API playground behavior the primary Cue product. Cue builds
  governed internal work apps, not standalone prompt experiments.

## Design Implications

- App Studio should move toward a workbench layout:
  left navigation for workspace/app scope, center for conversation and spec
  review, right side for preview, artifacts, tickets, and state.
- "New App" and "App Studio" should share one lifecycle. New App is the intake
  and creation entry; App Studio is the ongoing workbench for the same app.
- The first visible artifact after conversation should be a reviewable App Spec
  draft, not generated UI alone.
- Preview must sit next to the authoritative artifacts: App Spec, permissions,
  runtime data boundary, policy result, test result, sandbox release, and
  production approval package.
- Admin ticket UI must show request kind, requested resource/API, risk tier,
  estimated cost/blast radius, related app, agent evidence, policy result, and
  approve/reject/request-change actions.
- Deploy controls should be policy-aware: request sandbox, request production,
  awaiting Admin review, approved, rejected, released, rollback available.
- Templates should be governed examples with approved primitives and policy
  defaults. "Remix" means starting from a governed template, not bypassing
  artifact review.
- UI copy should default to zh-TW while preserving i18n keys. Professional terms
  such as App Spec, Sandbox, Production, Admin, API, GitLab, AlloyDB, and
  Release can remain English where that is clearer.
