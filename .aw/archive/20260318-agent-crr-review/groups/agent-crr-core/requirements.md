---
change: agent-crr-review
group: agent-crr-core
date: 2026-03-18
---

# Requirements

Implement a generic Create-Review-Revise (CRR) pattern and a specialized ReviewAgent in the 'agent' crate. 
1. ReviewAgent:
   - Must support both Spec Review (format, quality, completeness) and Code Review (spec compliance, security, test coverage).
   - Must return a 'ReviewVerdict' (Approved, NeedsRevision, Rejected) with 'ReviewIssue' details (severity, description, suggestion, location).
2. CRRCycle:
   - Generic builder-based API to wire up creator, reviewer, and reviser agents.
   - Configurable 'max_revisions' and event hooks for logging/SSE.
   - State machine logic: Create -> Review -> [Approved|NeedsRevision|Rejected]. If NeedsRevision and revisions < max_revisions, then Revise -> Review.
   - Must be reusable by SpecAgent and CodeAgent.
