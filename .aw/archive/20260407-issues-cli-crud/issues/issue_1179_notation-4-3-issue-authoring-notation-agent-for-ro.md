---
number: 1179
title: "notation(4.3): Issue authoring notation + agent for rough-idea → well-formed issue"
state: open
labels: [type:enhancement, priority:p1, project:conductor, project:score]
group: "issues-cli-crud"
---

# #1179 — notation(4.3): Issue authoring notation + agent for rough-idea → well-formed issue

Part of #1159 — formal notation for non-code artifact kinds.

## Problem

Currently in Conductor, issues come from GitLab sync (already well-formed) or from a minimal FastAPI create endpoint (no structure). There is no agent-assisted authoring flow that turns a rough user complaint into a well-formed issue.

This issue fills that gap with a formal issue notation + authoring agent.

## Schema candidate (v0)

\`\`\`yaml
type: object
required: [kind, title, problem_statement]
properties:
  kind:
    enum: [bug, feature, question, task, epic]
  title: { type: string, minLength: 10 }
  problem_statement: { type: string }  # the "why"
  current_behavior: { type: string }   # for bugs
  expected_behavior: { type: string }  # for bugs
  reproduction_steps:
    type: array
    items: { type: string }
  affected_users: { type: array, items: { type: string } }  # persona ids or free text
  priority: { enum: [p0, p1, p2, p3] }
  severity: { enum: [critical, major, minor, trivial] }
  proposed_solution: { type: string }  # optional
  acceptance_criteria: { type: array, items: { type: string } }
  related_artifacts: { type: array, items: { type: string } }  # references to BRD/PRD/TD that this issue stems from
  environment:
    type: object
    properties:
      os: { type: string }
      version: { type: string }
      config: { type: string }
\`\`\`

## Authoring agent

Agent that takes a free-form user complaint ("The thing is slow when I click X") and produces a populated issue schema. Should:
1. Ask clarifying questions to fill required fields (repro steps, expected vs actual)
2. Suggest priority/severity based on user description
3. Search existing issues for duplicates
4. Reference existing artifacts (PRDs, TDs) that the issue relates to

Agent lives in \`cclab-agent::agents::issue_author\` (new), prompts in \`cclab-sdd/prompts/issue/\`.

## Depends on

- Arsenal 1.2, 1.3, 1.4 (artifact model + kind + state machine for the authoring phase)
- 4.1, 4.2 (so issues can reference PRDs/BRDs via \`related_artifacts\`)
