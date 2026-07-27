---
id: workflow.agentic-workflow
scope: []
activation: always
targets: [claude, codex, agy]
enforcement: guard
required_references:
  - AGENTS.md
  - CONTRIBUTING.md
  - apps/agentic-workflow/CAPABILITIES.md
---
# Agentic Workflow protocol

## Intent

Keep lifecycle work driven by the live `aw` protocol and its machine-checked terminal state.

## Rules

- Treat `aw` stdout, payload paths, `invoke.command`, validation errors, and `next.command` as the current protocol.
- Drive managed work through `wi -> ec -> td -> cb`; use `aw goal wi`, `aw goal capability`, or `aw goal backlog` for run-to-terminal work.
- Do not claim completion until the goal envelope reports `completion.workflow_complete=true`.
- When a lifecycle command is broken, fix Agentic Workflow first instead of bypassing the defect in another project.
- Agentic Workflow dogfoods the same Python-first goal roots as every other project. Use a bounded direct-repair commit with a capability work root and `Refs #<issue>` trailer only when the exact worker verb required by the current root is itself broken.
- Treat HITL as blocking human input and never fabricate approval.

## Verification

- Run `aw llm` to inspect the binary-owned lifecycle orientation.
- Run the narrow command named by the current envelope.
- Run `aw health --project <project>` for read-only readiness and remediation routing.

## References

- `AGENTS.md` for authority order and runtime bootstrap.
- `CONTRIBUTING.md` for the repo-wide lifecycle and authoring contract.
- `apps/agentic-workflow/CAPABILITIES.md` for Agentic Workflow product claims.
