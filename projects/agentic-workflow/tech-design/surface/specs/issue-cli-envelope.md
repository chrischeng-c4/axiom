---
id: score-issue-cli-envelope
main_spec_ref: projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md
merge_strategy: new
amended_by: aw-mainthread-only-execution.md
amended_on: 2026-05-03
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# Score Work-Item CLI Envelope

> **AMENDMENT (2026-05-03).** The `agent` field on the `dispatch` envelope
> is being dropped as part of `aw-mainthread-only-execution.md`. While
> the rollout is in progress, the field is **optional and nullable**:
> emitters MAY still include `"agent": "score-<role>"` for backward
> compatibility, but mainthread no longer dispatches subagents — it
> invokes `invoke.command` directly regardless of the field. Once the
> rollout completes, the field will be removed from the schema and from
> all emit sites in `projects/agentic-workflow/src/cli/`. New code MUST NOT rely on
> the `agent` field for routing decisions.

> **Phase C root note.** Current Score handlers resolve the filesystem
> root from the CLI process CWD via `find_project_root()` and write
> `.aw/issues`, `.aw/payloads`, and `.aw/tech-design` under that
> current checkout. The JSON envelope remains compatible; this note fixes
> storage/root interpretation for all examples in this file.

## Schema
<!-- type: schema lang: yaml -->

```yaml
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "IssueEnvelope",
  "oneOf": [
    {
      "type": "object",
      "required": ["action", "slug", "invoke"],
      "additionalProperties": false,
      "properties": {
        "action": { "const": "dispatch" },
        "slug":   { "type": "string", "pattern": "^[a-z0-9-]+$" },
        "agent":  {
          "type": ["string", "null"],
          "pattern": "^score-",
          "description": "DEPRECATED — see amendment banner. Optional + nullable during rollout; removed after subagents retire (aw-mainthread-only-execution.md). Mainthread routes by invoke.command alone."
        },
        "invoke": {
          "type": "object",
          "required": ["command", "args"],
          "additionalProperties": false,
          "properties": {
            "command": { "type": "string" },
            "args":    { "type": "object" }
          }
        }
      }
    },
    {
      "type": "object",
      "required": ["action", "slug"],
      "additionalProperties": false,
      "properties": {
        "action": { "const": "done" },
        "slug":   { "type": "string", "pattern": "^[a-z0-9-]+$" }
      }
    },
    {
      "type": "object",
      "required": ["action", "slug", "message"],
      "additionalProperties": false,
      "properties": {
        "action":  { "const": "error" },
        "slug":    { "type": "string", "pattern": "^[a-z0-9-]+$" },
        "message": { "type": "string" }
      }
    }
  ]
}
```

## Interaction
<!-- type: interaction lang: mermaid -->

```mermaid
---
id: issue-cli-envelope-interaction
participants:
  user: User
  skill: "/aw:issue skill"
  mainthread: Mainthread
  cli: score
  author: score-issue-author
  hook: SubagentStop hook
---
sequenceDiagram
    actor U as User
    participant S as /aw:issue skill
    participant M as Mainthread
    participant C as score
    participant A as score-issue-author
    participant H as SubagentStop hook

    U->>S: /aw:issue (create)
    S->>M: ask title + type
    M->>C: aw wi create --title ... --type ...
    C->>C: activate issue branch + skeleton + commit(create)
    C-->>M: {action: dispatch, agent, slug, invoke}

    M->>A: Agent(prompt=envelope JSON)
    A->>C: aw wi fill-section --slug --section (brief mode)
    C-->>A: plain-text brief (issue path, payload path, constraints)
    A->>A: Read issue + fill sections
    A->>A: Write .aw/payloads/<slug>/body.md
    A-->>H: subagent stop

    H->>C: aw wi fill-section --slug --section --apply
    C->>C: validate → merge → commit(fill) → rm payload
    C-->>H: {action: done, slug}
    H-->>M: additionalContext

    M-->>U: summary
```

## Changes
<!-- type: doc lang: markdown -->

| File | Action | Purpose |
|------|--------|---------|
| `projects/agentic-workflow/src/cli/issues.rs` | modify | `IssueEnvelope` enum, envelope emission in `run_create`, new `FillSection` subcommand + `run_fill_section` (brief + apply) |
| `CLAUDE.md` | modify | Insert "Score envelope (mainthread protocol)" section |
| `projects/agentic-workflow/templates/mainthread/CLAUDE.md` | modify | Same as CLAUDE.md (template source of truth) |
| `projects/agentic-workflow/templates/mainthread/agents/score-issue-author.md` | modify | Rewrite Inputs/Process/Output to consume envelope JSON + write `.aw/payloads/<slug>/body.md`. Add `Write` tool + PreToolUse guard |
| `projects/agentic-workflow/templates/mainthread/skills/score-issue/SKILL.md` | modify | Slim to intent-router; delegate envelope handling to CLAUDE.md |
| `projects/agentic-workflow/templates/mainthread/settings.json` | modify | Register `score-issue-author` SubagentStop hook ahead of generic `score-*` validate hook |
| `projects/agentic-workflow/templates/mainthread/hooks/agents/issue-author/pretooluse-write-guard.sh` | create | Allow Write only to `.aw/payloads/<slug>/body.md` |
| `projects/agentic-workflow/templates/mainthread/hooks/agents/issue-author/subagentstop-apply.sh` | create | Parse transcript first row as envelope JSON, call `fill-section --apply`, return hook decision |
| `projects/agentic-workflow/templates/mainthread/hooks/global/subagentstop-validate.sh` | modify | Add early-skip for `score-issue-author` so the new specific hook owns it |
| `projects/agentic-workflow/src/cli/init.rs` | modify | `include_str!` + installation for the two new hook scripts |
| `projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md` | create | This TD |

## Traceability Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: interaction
    impl_mode: hand-written
    description: "Traceability metadata edge for the interaction section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```