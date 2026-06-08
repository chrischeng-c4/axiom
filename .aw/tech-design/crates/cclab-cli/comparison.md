# MCP Tools vs CLI Commands: Complete Comparison

This document provides a comprehensive mapping between MCP (Model Context Protocol) tools and their equivalent CLI commands.

## Quick Summary

- **27 MCP tools total**
- **14 CLI commands** implemented (9 core + 5 specialized)
- **8 Mermaid tools** remain MCP-only (LLMs generate diagrams directly)
- **Shared service layer** ensures consistency

## Complete Tool/Command Mapping

### Knowledge Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `read_knowledge` | `sdd knowledge read <path>` | 1 | ✅ Implemented |
| `list_knowledge` | `sdd knowledge list [path]` | 1 | ✅ Implemented |
| `write_knowledge` | `sdd knowledge write <path> --json-file <file>` | 3 | ✅ Implemented |
| `write_main_spec` | `sdd knowledge write-spec <path> --content-file <file>` | 3 | ✅ Implemented |

**Examples**:
```bash
# Read knowledge file
sdd knowledge read 00-architecture/index.md

# List all knowledge files
sdd knowledge list

# Write knowledge document
sdd knowledge write 30-claude/mcp-overview.md --json-file knowledge.json

# Write spec to main directory (archive workflow)
sdd knowledge write-spec oauth-flow.md --content-file spec.md
```

---

### File Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `read_file` | `sdd file read <change-id> [file]` | 1 | ✅ Implemented |
| `list_specs` | `sdd spec list <change-id>` | 1 | ✅ Implemented |

**Examples**:
```bash
# Read proposal
sdd file read my-change proposal

# Read tasks
sdd file read my-change tasks

# Read specific spec
sdd file read my-change oauth-spec

# List all specs
sdd spec list my-change
```

---

### Proposal Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `create_proposal` | `sdd proposal create <id> --json-file <file>` | 2 | ✅ Implemented |
| `append_review` | `sdd proposal review <id> --json-file <file>` | 2 | ✅ Implemented |

**Examples**:
```bash
# Create proposal
sdd proposal create my-change --json-file proposal.json

# Add review
sdd proposal review my-change --json-file review.json
```

---

### Spec Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `create_spec` | `sdd spec create <id> <spec-id> --json-file <file>` | 2 | ✅ Implemented |

**Example**:
```bash
# Create spec
sdd spec create my-change oauth-spec --json-file spec.json
```

---

### Tasks Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `create_tasks` | `sdd tasks create <id> --json-file <file>` | 2 | ✅ Implemented |

**Example**:
```bash
# Create tasks
sdd tasks create my-change --json-file tasks.json
```

---

### Implementation Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `read_all_requirements` | `sdd implementation read-all <id>` | 3 | ✅ Implemented |
| `list_changed_files` | `sdd implementation list-files <id> [options]` | 3 | ✅ Implemented |
| `read_implementation_summary` | ❌ No CLI equivalent | - | MCP only |

**Notes**:
- `read_implementation_summary` provides git diff + commit log
- Can be replaced by direct git commands if needed
- Primary use case is for MCP-based workflows

**Examples**:
```bash
# Read all requirements (proposal + tasks + specs)
sdd implementation read-all my-change

# List changed files
sdd implementation list-files my-change

# Filter by path
sdd implementation list-files my-change --filter src/

# Compare against different branch
sdd implementation list-files my-change --base-branch develop
```

---

### Clarifications Operations

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `create_clarifications` | `sdd clarifications create <id> --json-file <file>` | 3 | ✅ Implemented |

**Example**:
```bash
# Create clarifications
sdd clarifications create my-change --json-file clarifications.json
```

---

### Mermaid Diagram Generators

| MCP Tool | CLI Command | Phase | Status |
|----------|-------------|-------|--------|
| `generate_mermaid_flowchart` | ❌ No CLI | - | MCP only |
| `generate_mermaid_sequence` | ❌ No CLI | - | MCP only |
| `generate_mermaid_class` | ❌ No CLI | - | MCP only |
| `generate_mermaid_state` | ❌ No CLI | - | MCP only |
| `generate_mermaid_erd` | ❌ No CLI | - | MCP only |
| `generate_mermaid_mindmap` | ❌ No CLI | - | MCP only |
| `generate_mermaid_journey` | ❌ No CLI | - | MCP only |
| `generate_mermaid_requirement` | ❌ No CLI | - | MCP only |

**Rationale for MCP-only**:
- LLMs can generate Mermaid code directly in markdown
- Primarily used through MCP during planning
- CLI would add ~8 commands with limited practical value
- Human users rarely need programmatic diagram generation

---

## Command Categories

### Read-Only Commands (Phase 1)

Fast, simple operations with no JSON files needed:

```bash
sdd knowledge list
sdd knowledge read <path>
sdd spec list <id>
sdd file read <id> [file]
```

### Creation Commands (Phase 2 & 3)

Complex operations using `--json-file` for structured input:

```bash
sdd proposal create <id> --json-file proposal.json
sdd proposal review <id> --json-file review.json
sdd spec create <id> <spec-id> --json-file spec.json
sdd tasks create <id> --json-file tasks.json
sdd clarifications create <id> --json-file clarifications.json
sdd knowledge write <path> --json-file knowledge.json
```

### Specialized Commands (Phase 3)

Advanced workflow support:

```bash
sdd implementation read-all <id>
sdd implementation list-files <id> [--filter <pattern>]
sdd knowledge write-spec <path> --content-file spec.md
```

---

## Interface Comparison

### Input Formats

| Aspect | MCP Tools | CLI Commands |
|--------|-----------|--------------|
| Simple inputs | JSON parameters via stdin | Command-line arguments |
| Complex inputs | JSON objects via stdin | JSON files on disk (`--json-file`) |
| File content | Embedded in JSON | Separate file (`--content-file`) |

### Output Formats

| Aspect | MCP Tools | CLI Commands |
|--------|-----------|--------------|
| Success | JSON-RPC response with result | Plain text with formatting |
| Errors | JSON-RPC error object | stderr + non-zero exit code |
| Formatting | Wrapped in XML tags (e.g., `<result>`) | Direct output with ANSI colors |

### Tool Discovery

| Aspect | MCP Tools | CLI Commands |
|--------|-----------|--------------|
| List tools | `tools/list` JSON-RPC method | `sdd --help` |
| Tool details | `tools/definition` with JSON Schema | `sdd <command> --help` |
| Examples | JSON Schema in tool definition | Examples in docs + JSON files |

---

## Client Compatibility

| Client | MCP Support | CLI Support | Recommended |
|--------|-------------|-------------|-------------|
| Claude Code | ✅ Full support | ✅ Full support | Use MCP (native) |
| Gemini CLI | ❌ Connection issues | ✅ Works reliably | Use CLI |
| Codex | ❌ MCP client unavailable | ✅ Works reliably | Use CLI |
| Custom Python/Node | ✅ Via MCP SDK | ✅ Via subprocess | Choose based on needs |

---

## Architecture Benefits

Both MCP and CLI interfaces share the same **service layer**, providing:

1. **Zero Code Duplication**: Business logic written once
2. **Consistency**: Same behavior across interfaces
3. **Easy Testing**: Pure functions in service layer
4. **Future Extensibility**: Can add HTTP API without duplication

```
┌─────────────┐         ┌─────────────┐
│  MCP Tools  │         │ CLI Commands│
│ (JSON-RPC)  │         │ (clap args) │
└──────┬──────┘         └──────┬──────┘
       │                       │
       │   ┌───────────────────┘
       │   │
       ▼   ▼
   ┌──────────────┐
   │Service Layer │ ← Single source of truth
   │(Rust structs)│
   └──────┬───────┘
          │
          ▼
   ┌──────────────┐
   │  Core Logic  │ ← Validators, parsers, models
   └──────────────┘
```

---

## Usage Recommendations

### For LLMs (Gemini, Codex)

**Use CLI commands** due to MCP client compatibility issues:

1. Generate JSON from templates
2. Write to temporary file
3. Execute CLI command with `--json-file`
4. Parse output to verify success

### For Claude Code

**Use MCP tools** (native integration):

- Automatic tool discovery
- Structured JSON-RPC protocol
- Better error handling
- Cleaner integration

### For Human Users

**Use existing workflow commands**:

```bash
sdd run-change <id> "<description>"       # Unified workflow
```

Use CLI utility commands only when:
- Debugging specific issues
- Integrating with scripts
- Working around MCP issues

---

## Future Enhancements

### Potential CLI Additions (Not Planned)

- Interactive mode for creation commands
  - `sdd proposal create --interactive`
  - Prompts for each field with validation

- Hybrid flag-based input
  - `sdd proposal create <id> --summary "..." --why "..."`
  - Alternative to JSON files for power users

- HTTP API
  - REST alternative to MCP/CLI
  - Could share the same service layer

### Planned Improvements

- Auto-completion for CLI commands
- Shell integration (bash, zsh, fish)
- Progress indicators for long operations
- Better error messages with suggestions

---

## Getting Help

- **CLI help**: `sdd <command> --help`
- **JSON examples**: See `.aw/tech-design/cli-guide/examples/`
- **MCP tools**: See `src/mcp/tools/` documentation
- **Issues**: https://github.com/anthropics/cclab-sdd/issues
