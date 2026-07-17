# AW Templates

Templates embedded into the `agentic-workflow` crate and installed by explicit
Agentic Workflow producer commands.

## Directory Structure

```
templates/
├── README.md                 # This file
├── aw.toml                  # AW root config template
├── knowledge/
│   └── index.md             # Knowledge base index template
├── mainthread/
│   ├── CLAUDE.md.tmpl       # AW section for project CLAUDE.md
│   └── skills/              # Claude Code skills
│       ├── aw-wi/
│       ├── aw-guard/
│       └── aw-health/
└── prompts/                 # Task prompts for orchestrator
    ├── create_proposal.md
    ├── create_spec.md
    ├── review_*.md       # Tasks are auto-generated from specs
    ├── revise_*.md
    └── ...
```

## Project Asset Producers

### Project Files

| Destination | Source | Mode |
|-------------|--------|------|
| `aw.toml` | Generated | Create/Update |
| `CLAUDE.md` | `mainthread/CLAUDE.md.tmpl` | Merge AW section |

### Claude Code Skills

| Destination | Source |
|-------------|--------|
| `.claude/skills/aw-wi/` | `mainthread/skills/aw-wi/` |
| `.claude/skills/aw-guard/` | `mainthread/skills/aw-guard/` |
| `.claude/skills/aw-health/` | `mainthread/skills/aw-health/` |

## Usage

```bash
# Create a greenfield project with managed assets
aw new my-project

# Refresh the generated project registry block
aw conf sync
```

## Skills Usage in Claude Code

```bash
# Work-item planning and validation
/aw:wi "split this capability into a roadmap"

# Agent-runtime edit/create guard
/aw:guard

# Project readiness aggregate
/aw:health --project <project>
```
