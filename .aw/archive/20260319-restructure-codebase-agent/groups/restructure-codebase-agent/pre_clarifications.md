---
change: restructure-codebase-agent
group: restructure-codebase-agent
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Token budget
- **Answer**: Default 30K tokens per group. Configurable via RestructureCodebaseAgentConfig. Not in config.toml — passed at construction time.

### Q2: General
- **Question**: set_grouping schema
- **Answer**: Each group: { id: String, paths: Vec<String>, estimated_tokens: u32, description: String }. The id is kebab-case, paths are relative to repo root.

### Q3: General
- **Question**: Manifest scope
- **Answer**: Support all three: pyproject.toml (tool.uv.workspace.members), package.json (workspaces), Cargo.toml (workspace.members). These cover Python, JS/TS, Rust monorepos.

### Q4: General
- **Question**: Drill-down termination
- **Answer**: Stop when all groups fit within budget. Max depth 5 as safety limit. No minimum granularity — agent decides.

### Q5: General
- **Question**: Dependency availability
- **Answer**: #950 and #951 are already implemented. Groups artifact schema defined by this agent — downstream agents accept Vec<CodebaseGroup>.

### Q6: General
- **Question**: CLI entry point
- **Answer**: No CLI. Only invoked programmatically by orchestrator. Exposed via PyO3 for Conductor.

