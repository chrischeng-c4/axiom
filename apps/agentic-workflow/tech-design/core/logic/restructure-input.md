---
id: restructure-input-logic
type: spec
title: "Restructure Input — Logic"
version: 1
status: archived
files:
  - tools/restructure_input.rs
  - services/restructure_service.rs
  - services/group_service.rs
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "This analysis/standardization logic TD supports brownfield semantic coverage and takeover readiness gates."
---

> **ARCHIVED**: This phase was absorbed by the issue lifecycle CRR (issue-lifecycle-crr).
> Pre-SDD preparation (clarifications, reference context) now happens during issue authoring
> via `aw wi create` + `aw wi validate`, before `score workflow` begins.

# Restructure Input

## Phase Transition
<!-- type: doc lang: markdown -->

```yaml
from: ChangeInited
to: InputRestructured
executor: [mainthread]
crr: false  # self-review checklist, no CRR loop
```

## Prompt Template
<!-- type: doc lang: markdown -->

```markdown
# Task: Restructure Input for '{{change_id}}'

Read `user_input.md` and any `issue_*.md` files to understand the change request.

\## Steps

1. Read artifacts: `sdd_read_artifact(scope="user_input")`, `sdd_read_artifact(scope="issues")`
2. Group related issues into logical spec units (1 group = 1 future spec)
3. For each group, write:
   - Consolidated requirements (synthesized from issues + description)
   - Pre-generated clarification questions (topics that need user input)
4. Self-review: verify no orphan issues, all requirements traced
5. Call `sdd_artifact_restructure_input` with the groups array

\## Grouping Rules

- Each issue belongs to exactly one group (no orphans, no duplicates)
- Group by logical spec boundary (e.g. "auth-api", "db-migration")
- Single-issue changes: one group with group_id derived from description
- group_id: kebab-case, descriptive (e.g. "auth-api", "queue-retry")

\## Self-Review Checklist

- [ ] Every issue assigned to exactly one group
- [ ] Requirements are synthesized, not copy-pasted from issues
- [ ] Questions target genuine ambiguities, not obvious answers
- [ ] group_id is meaningful and kebab-case
```

## Artifact Schema
<!-- type: doc lang: markdown -->

### Input: groups array

```json
{
  "type": "array",
  "minItems": 1,
  "items": {
    "type": "object",
    "required": ["id", "issues", "requirements", "questions"],
    "properties": {
      "id": { "type": "string", "pattern": "^[a-z0-9-]+$" },
      "issues": { "type": "array", "items": { "type": "integer" } },
      "requirements": { "type": "string" },
      "questions": {
        "type": "array",
        "items": {
          "type": "object",
          "required": ["topic", "question"],
          "properties": {
            "topic": { "type": "string" },
            "question": { "type": "string" }
          }
        }
      }
    }
  }
}
```

### Output files (per group)

```
.aw/changes/{change_id}/groups/{group_id}/
├── requirements.md        # consolidated requirements
└── pre_clarifications.md  # questions with status: pending
```

### pre_clarifications.md format

```yaml
---
status: pending
group_id: {group_id}
---

\## Questions

### {topic}

**Q:** {question}
**A:** <!-- pending -->
```

## Side Effects
<!-- type: doc lang: markdown -->

| Action | STATE.yaml change |
|--------|-------------------|
| `sdd_artifact_restructure_input` | `phase → InputRestructured`, `groups_progress` initialized (all keys empty) |

## Validation
<!-- type: doc lang: markdown -->

- No orphan issues (every issue in `user_input.md` or `issue_*.md` assigned to a group)
- No duplicate issues across groups
- At least 1 question per group
- group_id matches `^[a-z0-9-]+$`
