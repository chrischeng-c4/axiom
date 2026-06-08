---
change: change-spec-agent
group: spec-agent-core
date: 2026-03-18
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| agents | cclab-agent | high | Agent execution loop (turn-based, tool calls, message history), AgentId, Message, Role types, AgentBuilder pattern convention |
| reference-context-agent | cclab-agent | high | ReferenceContext artifact structure (specs, requirements, contradictions), SpecStore integration with search and read methods |
| restructure-agent | cclab-agent | high | StructuredIssue object schema (title, description, acceptance_criteria, etc.) |
| review-agent | cclab-agent | high | ReviewAgent with Spec/Code review modes, ReviewVerdict discriminated union (Approved/NeedsRevision/Rejected), CRRCycle orchestrator with creator/reviewer/reviser loop |
| README | cclab-sdd | high | SDD format priority (OpenRPC > JSON Schema > Mermaid > YAML > Markdown > Prose), Diagram selection rules (FSM -> stateDiagram, DAG -> flowchart, etc.), Spec section structure (Overview, Requirements, Scenarios, etc.), Quality standards (< 10% natural language, no real code) |

