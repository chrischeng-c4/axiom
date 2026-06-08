---
id: toolchain-reorg
type: exploration
created_at: 2026-01-28T08:18:07.154687+00:00
needs_clarification: false
---

# Codebase Exploration

### Codebase Analysis for toolchain-reorg

#### 1. Current Routing Architecture
The \`UnifiedMcpRouter\` in \`crates/cclab-server/src/mcp/router.rs\` currently acts as a proxy for all Prism tools, routing them to a per-project \`RequestHandler\` (daemon) instance via \`PrismHandlerPool\`.

#### 2. Tool Classification
Based on the analysis and clarifications:
- **Daemon Path (Analysis)**: Tools like \`prism_check\`, \`prism_hover\`, etc., depend on the in-memory index and file watching managed by the daemon.
- **MCP Handler Path (Local)**: Tools like \`prism_generate_from_spec\` and the new state machine tools are pure functional tools. They do not require the daemon's project-wide state and can be executed directly in the server process using handlers in \`cclab_prism::mcp::spec_handler\`.
- **Remove Path (Environment)**: Sensitive tools (\`prism_get_config\`, etc.) that modify \`pyproject.toml\` are to be removed for security and lack of immediate requirement.

#### 3. State Machine Integration
Prism already has a robust \`statemachine\` module with schema, validation, and Mermaid+ generation. Exposing this via MCP involves:
- Updating \`ArgusTools::list()\` in \`crates/cclab-prism/src/mcp/tools.rs\`.
- Implementing new handlers in \`crates/cclab-prism/src/mcp/spec_handler.rs\`.

#### 4. Impact Analysis
- **cclab-server**: Significant refactoring of \`UnifiedMcpRouter\` to support multiple routing paths.
- **cclab-prism**: Minor additions to MCP tools list and spec handlers.
- **Security**: Improved security posture by removing environment configuration tools.

#### 5. Recommended Spec Structure
- \`prism-mcp-refactor\`: Focuses on the router logic and tool filtering.
- \`state-machine-tools\`: Focuses on the new state machine capabilities.
