# Agentic Workflow Workspace

This directory stores Agentic Workflow artifacts for the current project. It is
created and managed by the `aw` CLI.

Agentic Workflow is an **agent development tool** — this workspace dir follows the
standard dev-tool convention of hidden dot-prefix state dirs, matching
`.git/`, `.cargo/`, `.claude/`, `.gemini/`.

## Layout

```
.aw/
├── config.toml        # Agentic Workflow configuration
├── tech-design/       # Tech design docs — the 'what and why'
├── issues/            # Local issue artifacts (pre-tracker, pre-GitHub)
├── changes/           # In-flight changes — the 'how' (specs + tasks + impl)
└── archive/           # Completed changes (historical record)
```

## Artifact types

| Directory | Content | Lifecycle |
|-----------|---------|-----------|
| `tech-design/` | Current design docs per crate/module | Long-lived, versioned with code |
| `issues/` | Local issue descriptions before they become changes | Short-lived, transitioned to changes |
| `changes/` | Active change proposals + specs + implementation tasks | In-flight (single change or multi-group) |
| `archive/` | Merged changes | Permanent record, read-only |

## Workflow

Use the Agentic Workflow CLI:

| Command | Purpose |
|---------|---------|
| `aw wi` | Work-item inventory and CRRR |
| `aw td` | Tech-design lifecycle |
| `aw cb` | Code-artifact lifecycle |
| `aw standardize` | Existing-project takeover loops |
| `aw capability` | Product capability report/next/run/check loop |

See `projects/agentic-workflow/` for the Agentic Workflow source code.
