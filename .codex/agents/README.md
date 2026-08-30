# Codex project development agents

This directory contains one Codex development role for every direct project
under `apps/*` and `libs/*`.

- Each role is named `<project>-dev`.
- Every role fixes the model to `gpt-5.6-terra`.
- Role files intentionally omit `model_reasoning_effort`.
- The dispatcher must select the effort for each task.
- Effort changes reasoning depth only. It never changes scope or authority.

Use this task-based effort guide:

| Effort | Use it for |
|---|---|
| `low` | Mechanical, narrow work with no public behavior change. |
| `medium` | Contained behavior in one owner with focused tests. |
| `high` | Material public behavior or several modules and consumers. |
| `xhigh` | Cross-project, concurrency, durability, security, compatibility, release, or supply-chain work. |
| `max` | The hardest quality-first work where failure would be costly and deeper verification has measured value. |

Set the effort in the dispatch call. If the dispatcher omits it, the runtime
default applies. Never treat a higher effort as permission to widen the task.

The Claude agent fleet under `.claude/agents/` is a separate runtime and is
not defined by this directory.
