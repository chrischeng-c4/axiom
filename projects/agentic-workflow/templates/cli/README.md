# AW Templates

Templates embedded into the `agentic-workflow` crate and installed via `aw init`.

## Directory Structure

```
templates/
├── README.md                 # This file
├── config.toml              # AW config template
├── knowledge/
│   └── index.md             # Knowledge base index template
├── mainthread/
│   ├── CLAUDE.md            # AW section for project CLAUDE.md
│   └── skills/              # Claude Code skills
│       ├── aw-capability/
│       ├── aw-wi/
│       ├── aw-cb-claim/
│       └── aw-standardize/
└── prompts/                 # Task prompts for orchestrator
    ├── create_proposal.md
    ├── create_spec.md
    ├── review_*.md       # Tasks are auto-generated from specs
    ├── revise_*.md
    └── ...
```

## What `aw init` Installs

### Project Files

| Destination | Source | Mode |
|-------------|--------|------|
| `.aw/config.toml` | Generated | Create/Update |
| `CLAUDE.md` | `mainthread/CLAUDE.md` | Merge AW section |

### Claude Code Skills

| Destination | Source |
|-------------|--------|
| `.claude/skills/aw-capability/` | `mainthread/skills/aw-capability/` |
| `.claude/skills/aw-wi/` | `mainthread/skills/aw-wi/` |
| `.claude/skills/aw-cb-claim/` | `mainthread/skills/aw-cb-claim/` |
| `.claude/skills/aw-standardize/` | `mainthread/skills/aw-standardize/` |

## Usage

```bash
# Fresh install
aw init

# Update (preserves user data)
aw init
```

## Skills Usage in Claude Code

```bash
# Capability alignment
/aw:capability "clarify this project's capabilities"

# Work-item planning and CRRR
/aw:wi "split this capability into a roadmap"

# TD lifecycle
/aw:td:create <slug>

# Existing-code adoption
/aw:cb-claim <path>

# Existing-project takeover
/aw:standardize <project>
```
