---
id: fillback-main-specs-workflow
type: exploration
created_at: 2026-02-06T04:25:56.976979+00:00
needs_clarification: false
---

# Codebase Exploration

# Codebase Exploration: fillback-main-specs-workflow

## Codebase Analysis

### Existing Infrastructure

**1. Fillback Engine (`crates/cclab-genesis/src/fillback/`)**
- `ast.rs` - Tree-sitter based AST parsing (Rust, Python, TS, JS, Go)
- `code.rs` - `CodeStrategy` - scans directories, analyzes codebase, generates basic specs
- `graph.rs` - `DependencyGraph` - builds module dependency graphs
- `strategy.rs` - `ImportStrategy` trait interface
- `factory.rs` - Strategy factory for auto-detection
- `openspec.rs`, `speckit.rs` - Other import strategies

Current limitation: `CodeStrategy` generates **basic** module-level specs (symbol tables, dependency lists). It does NOT generate the rich spec format we need (Genesis-style specs with requirements, scenarios, Mermaid diagrams, OpenAPI/OpenRPC/AsyncAPI/JSON Schema).

**2. MCP Tools for Spec Creation**
- `genesis_create_spec` - Creates structured spec files with requirements, scenarios, diagrams, API specs
- `genesis_write_main_spec` - Writes directly to `cclab/specs/{group}/`
- `genesis_analyze_code_for_spec` - Analyzes code files and suggests spec structure
- `genesis_validate_spec_completeness` - Validates spec has all required elements

**3. Existing Skill Templates**
- Skills live in `.claude/skills/` with `SKILL.md` (or `skill.md`)
- Templates for `cclab init` are at `crates/cclab-genesis/templates/mainthread/skills/`
- Skills are registered in CLAUDE.md between `<!-- cclab:gen:start -->` and `<!-- cclab:gen:end -->`

**4. MCP Spec Generation Tools (Aurora)**
- `aurora_generate_flowchart`, `aurora_generate_sequence`, `aurora_generate_class`, etc.
- `aurora_generate_openapi`, `aurora_generate_asyncapi`, `aurora_generate_openrpc`
- These can produce the detailed diagrams and API specs needed

### Existing Main Specs
Over 150 specs organized by crate: `cclab-aurora`, `cclab-cli`, `cclab-genesis`, `cclab-orbit`, etc.
These show the target format — rich specs with Mermaid diagrams, requirements, scenarios, OpenAPI/OpenRPC where applicable.

## Architecture for the New Workflow

### Workflow Design: `/cclab:genesis:fillback-main-specs`

This is a **standalone workflow** (not part of the decide→plan→impl→merge pipeline). It directly writes to `cclab/specs/`.

```
Scan → Discover Components → User Selection → Per-Component Analysis → Generate Rich Spec → Write to cclab/specs/
```

### Two Modes

**Mode 1: Mono-repo** (e.g., Cargo workspace with `crates/`)
- Auto-detect workspace members
- Present list to user, process one at a time
- Each component → spec_group = component name

**Mode 2: Non-mono-repo** (single project)
- Dynamic chunking based on project structure
- AI analyzes codebase to identify functional domains
- User confirms chunks before processing
- Concern: large codebases need intelligent chunking (not just "batch fill all")

### Per-Component Pipeline

For each component/chunk:
1. **AST Scan** - Use existing `CodeStrategy.analyze_codebase()` for raw analysis
2. **Prism Analysis** - Use `prism_symbols`, `prism_check` for deeper analysis
3. **AI Enrichment** - LLM reads the code and generates:
   - Rich overview with requirements (R1, R2, ...)
   - Acceptance scenarios (Given/When/Then)
   - Mermaid diagrams (class, flowchart, sequence, state, ERD as appropriate)
   - API specs (OpenAPI for HTTP, OpenRPC for RPC, AsyncAPI for events, JSON Schema for data models)
4. **Write Spec** - Use `genesis_write_main_spec` to write directly to `cclab/specs/{group}/`
5. **Validate** - Optionally validate completeness

### Key Design Decisions

- **No change workflow** - Direct write to main specs (user confirmed)
- **Very detailed** - Must include Mermaid, OpenAPI/OpenRPC/AsyncAPI/JSON Schema (user confirmed)
- **Interactive** - Use AskUserQuestion for component selection (user confirmed)
- **Dynamic chunking** - AI-assessed, not fixed strategy (user confirmed)

## Impact Analysis

### Files to Create
1. **Skill definition**: `.claude/skills/cclab-genesis-fillback-main-specs/SKILL.md`
2. **Template**: `crates/cclab-genesis/templates/mainthread/skills/cclab-genesis-fillback-main-specs/SKILL.md`
3. **CLAUDE.md update**: Register new skill in the workflow table

### Files to Modify
- `crates/cclab-genesis/templates/mainthread/CLAUDE.md` - Add skill entry
- `.claude/CLAUDE.md` (project root) - Add skill entry

### No Rust Code Changes Needed
The skill is purely a **mainthread orchestration skill** — it uses existing MCP tools (`analyze_code_for_spec`, `write_main_spec`, `prism_*`, `aurora_*`) and AI enrichment. No new MCP tools or Rust code required.

## Technical Considerations

### Spec Type Detection
Based on code analysis, assign `spec_type`:
- HTTP endpoints → `http-api` (needs OpenAPI + sequence diagram)
- gRPC/JSON-RPC → `rpc-api` (needs OpenRPC + class diagram)
- Event handlers → `event-driven` (needs AsyncAPI + sequence diagram)
- Data models/schemas → `data-model` (needs ERD/class diagram + JSON Schema)
- Algorithms/business logic → `algorithm` (needs flowchart/state diagram)
- Utilities/helpers → `utility`

### Chunking Strategy for Non-Mono-Repo
1. First pass: scan directory structure, count files per top-level dir
2. Use AI (Gemini) to identify functional domains
3. Present to user with estimated size
4. Process one domain at a time

### Context Window Concern
For large components, a single spec generation may exceed context limits.
Strategy: Sub-chunk by file group (related files based on imports), generate multiple specs per component.

## Spec Recommendations

### Single Spec: `fillback-main-specs-skill`
- **spec_type**: `algorithm` (it's an orchestration workflow)
- **Focus**: Skill definition, workflow states, decision logic
- No new MCP tools needed → pure skill/template creation

## Risk Assessment

1. **Quality of AI-generated specs** - Generated specs may lack domain knowledge; mitigate by showing drafts for user review
2. **Large codebase handling** - Non-mono-repo with 1000+ files; mitigate by chunking + file limits
3. **Context window limits** - LLM may not process entire component at once; mitigate by sub-chunking
4. **Spec format consistency** - Generated specs should match existing main spec format; mitigate by using `genesis_create_spec` tool format as template

## Open Questions

None — all key decisions were clarified in the initial Q&A.
