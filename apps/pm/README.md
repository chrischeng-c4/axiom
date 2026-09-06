# pm — Local Agent-First Project Management MCP Server

`pm` is an ultra-lightweight, high-performance Project Management system built in Rust, designed specifically for **autonomous AI agents** (Claude Code, Codex, Antigravity, Cursor) communicating exclusively via the **Model Context Protocol (MCP)** over `stdio`.

## Key Features & Space-Saving Design

- **Zero Runtime Bloat**: Compiled to a single native binary (~3-5MB). No Python virtualenvs, no node_modules, no external database server.
- **Compact & Crash-Safe Storage**: All entities (`Project`, `PRD`, `TechDesign`, `Feature`, `Task`) are stored in an atomic `.pm/state.json` file (< 10KB initial size).
- **Agent Scheduling Engine**:
  - `pm_get_next_actionable_task`: Dispatches the highest-priority `todo` task whose `blocked_by` prerequisites are all `done`.
  - `pm_get_task_context`: Bundles Task description + Parent Feature + PRD + Tech Design into a single, prompt-ready markdown block.
- **Fast Cold-Start**: Responds to MCP JSON-RPC requests in < 10ms with RSS memory usage < 15MB.

---

## Installation & Configuration

### In `.mcp.json`

Add the server entry to your repository's `.mcp.json`:

```json
{
  "mcpServers": {
    "pm": {
      "command": "cargo",
      "args": ["run", "-p", "pm", "--", "serve"]
    }
  }
}
```

Or when using the compiled release binary for maximum speed:

```json
{
  "mcpServers": {
    "pm": {
      "command": "/path/to/axiom/target/release/pm",
      "args": ["serve"]
    }
  }
}
```

---

## MCP Tools Inventory

### Project Management
- `pm_init_project(id, name, description, root_path)`: Register/initialize project.
- `pm_get_project_summary(project_id)`: Progress dashboard with task counts, active features, and PRD status.
- `pm_list_projects()`: List all registered projects.

### PRD (Product Requirements Document)
- `pm_save_prd(project_id, prd_id, title, content, status, version)`: Create/update PRD in full Markdown.
- `pm_get_prd(prd_id)`: Retrieve PRD content and metadata.
- `pm_list_prds(project_id)`: List all PRDs for a project.

### Tech Design (Technical Architecture)
- `pm_save_tech_design(project_id, td_id, title, content, prd_id, status, version)`: Create/update Technical Design specification.
- `pm_get_tech_design(td_id)`: Retrieve Tech Design document.
- `pm_list_tech_designs(project_id)`: List Tech Designs for a project.

### Feature (Capabilities / Epics)
- `pm_create_feature(project_id, feature_id, title, description, prd_id, tech_design_id, priority)`: Define high-level feature.
- `pm_update_feature(feature_id, status, priority, description)`: Update feature state.
- `pm_get_feature(feature_id)`: Retrieve feature details and associated tasks.
- `pm_list_features(project_id, status)`: Query features by status.

### Task (Agent Execution & Scheduling)
- `pm_create_task(project_id, task_id, title, description, feature_id, priority, assignee, blocked_by)`: Create task with prerequisites.
- `pm_update_task_status(task_id, status, result_summary)`: Advance task state (`todo`, `in_progress`, `in_review`, `done`, `blocked`) and record execution evidence.
- `pm_list_tasks(project_id, feature_id, status, assignee)`: Filter tasks.
- `pm_get_next_actionable_task(project_id, assignee)`: **Autonomous Dispatcher**. Finds the highest-priority `todo` task whose prerequisites are all `done`.
- `pm_get_task_context(task_id)`: **Context Aggregator**. Packages Task spec + Feature spec + PRD + Tech Design into a single markdown block for prompt injection.

### Local Gate Check & Auto-Merge Engine (Replacing `gh pr`)
- `pm_verify_gate(task_id, command)`: Executes a local verification/test gate command for a task, records exit code, duration, stdout/stderr, and current HEAD commit hash.
- `pm_get_gate_history(task_id)`: Retrieves all historical verification runs for a task.
- `pm_merge_change(task_id, target_branch, strategy)`: **Zero-Network PR Replacement**. Ensures latest gate run passed and all blocking review comments are resolved, then merges changes locally into target branch and transitions task to `done`.

### Local Defect Tracking (Replacing `gh issue`)
- `pm_report_defect(project_id, title, description, severity, reproduction_steps, auto_create_task)`: File a defect (`critical`, `major`, `minor`, `trivial`). If `auto_create_task=true`, automatically spawns a high-priority actionable fix task.
- `pm_resolve_defect(defect_id, resolution_note)`: Mark a defect as resolved with an explanatory note.
- `pm_list_defects(project_id, status, severity)`: Query defects for a project.

### Local Review Comments (Replacing `gh pr review`)
- `pm_add_review_comment(task_id, reviewer, content, severity, file_path, line_number)`: Add inline/blocking review feedback (`blocking`, `suggestion`, `nitpick`). Blocking comments prevent `pm_merge_change` until resolved.
- `pm_resolve_review_comment(comment_id, resolution_note)`: Resolve a review comment.
- `pm_list_review_comments(task_id, unresolved_only)`: List review feedback on a task.

---

## MCP Resources

- `pm://projects/{project_id}`: JSON project metadata.
- `pm://prds/{prd_id}`: Raw Markdown PRD.
- `pm://tech-designs/{td_id}`: Raw Markdown Tech Design.
- `pm://tasks/{task_id}`: Complete Task context and execution logs.
- `pm://defects/{defect_id}`: JSON defect details and associated fix task link.
- `pm://gates/{task_id}`: History of local gate runs and verification proofs.

---

## Dual Transport: Stdio & HTTP

`pm` natively supports both **stdio** (default for Claude Desktop, Cursor, and IDE plugins) and **HTTP JSON-RPC 2.0** (for network-connected agents and HTTP simulation):

```bash
# Start MCP server on stdio (default)
cargo run -p pm -- serve

# Start MCP server over HTTP JSON-RPC on port 3200
cargo run -p pm -- serve --http 127.0.0.1:3200

# Client test with curl
curl -X POST http://127.0.0.1:3200/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}'
```

---

## E2E Testing

Comprehensive integration test suites test the entire agent lifecycle end-to-end:

```bash
# E2E test driving HTTP MCP JSON-RPC agent flow
cargo test -p pm --test e2e_http_mcp

# E2E test driving Local Gate verification, Review comments, and Merge engine
cargo test -p pm --test local_gate_merge_e2e
```

