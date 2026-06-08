---
number: 928
title: "feat(agent): Add ReferenceContextAgent — spec exploration with CRR"
state: open
labels: [enhancement, crate:agent, P1]
group: "reference-context-agent"
---

# #928 — feat(agent): Add ReferenceContextAgent — spec exploration with CRR

**Parent**: #920

## Summary

Agent that explores existing specs to build reference context for downstream agents (ChangeSpecAgent). Maps to SDD phases 4+5 (reference-context + post-clarification).

## Flow

```
RestructureIssueAgent output (structured issues)
    ↓
ReferenceContextAgent
    ├→ SpecStore.search() → find related specs
    ├→ SpecStore.read() → read content
    ├→ Score relevance (high/medium/low)
    ├→ CRR review (coverage complete?)
    ├→ If contradictions → ask user (optional)
    ↓
Reference context artifact (specs + relevance + key_requirements)
    ↓
ChangeSpecAgent (uses context to write new specs)
```

## Output Schema

```json
{
  "specs": [
    {
      "spec_id": "cclab-agent/agents.md",
      "relevance": "high",
      "key_requirements": ["Agent trait interface", "Builder pattern"]
    }
  ],
  "contradictions": []
}
```

## Dependencies
- SpecStore trait (defined in RestructureIssueAgent)
- ReviewAgent + CRRCycle (for quality review of context)
