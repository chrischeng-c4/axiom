---
change: change-impl-agent
group: code-agent-core
date: 2026-03-18
---

# Requirements

Implement 'CodeAgent' in the 'cclab-agent' crate to transform approved specifications into code implementations.

Key capabilities:
1. Parse specifications to identify and decompose implementation tasks.
2. Perform topological sorting of tasks based on dependencies.
3. Generate multi-file code implementations using an LLM.
4. Integrate with the existing CRRCycle and ReviewAgent for automated code review (CRR cycle).
5. Perform Git operations (create branches, commit changes, open PRs/MRs) via platform integrations (GitHub/GitLab) rather than local Git.

Integration points:
- Extension of 'PlatformIntegration' trait (src/integrations/mod.rs) and its implementations (github.rs, gitlab.rs) to support branch/commit/PR/MR operations.
- Coordination with 'ReviewAgent' (crates/cclab-agent/src/agents/review/) for code quality and spec compliance review.
- Utilization of 'CRRCycle' (crates/cclab-agent/src/agents/crr.rs) for the orchestration of the Create-Review-Revise loop.
- Error handling integration with 'NovaError' for max revisions or platform API failures.
