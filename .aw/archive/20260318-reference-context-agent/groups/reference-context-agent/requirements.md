---
change: reference-context-agent
group: reference-context-agent
date: 2026-03-18
---

# Requirements

Implement the ReferenceContextAgent to automate spec exploration and context building for downstream agents. The agent must:
1. Interface with SpecStore (search/read) to identify related specifications.
2. Score spec relevance as high, medium, or low.
3. Utilize ReviewAgent and the CRRCycle for internal quality review of the reference context.
4. Detect contradictions between existing specs and the new change requirements.
5. Produce a structured artifact containing specs, relevance, and key requirements.
6. Support SDD phases 4 (reference-context) and 5 (post-clarification).
