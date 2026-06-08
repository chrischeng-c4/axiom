---
id: nova-async-clarification
type: exploration
created_at: 2026-02-01T10:19:41.644461+00:00
needs_clarification: false
---

# Codebase Exploration

The codebase currently implements `AnalystAgent` in `crates/cclab-nova` with basic session persistence and platform integrations (GitHub, GitLab, Jira). However, session persistence lacks the full message history (LLM context), and the clarification workflow is synchronous via `AskUserTool`.

To support async clarification:
1. **Session State Persistence**: `SessionState` must be expanded to include `messages: Vec<Message>`. This ensures that when a session is reloaded, the LLM has the full context of the previous turns.
2. **Platform Commenting**: The `PlatformIntegration` trait needs a `post_comment` method. This allows the agent to interact with stakeholders on the platforms where the requirements reside.
3. **Async Workflow Tooling**: A new `post_comment` tool (specialized for each platform) will be added. It will format questions with markdown checkboxes and return a `user_input_required` status, signaling the `AnalystAgent` to save state and pause.
4. **Response Parsing & Resume**: `AnalystAgent` needs logic to parse markdown comments to extract checkbox selections and supplementary text. A `resume` mechanism will fetch new comments since the last interaction and feed them into the context.

Relevant files:
- `crates/cclab-nova/src/storage/mod.rs`: Update `SessionState`.
- `crates/cclab-nova/src/integrations/mod.rs`: Update `PlatformIntegration` trait.
- `crates/cclab-nova/src/integrations/github.rs`, `gitlab.rs`, `jira.rs`: Implement `post_comment`.
- `crates/cclab-nova/src/agents/analyst.rs`: Implement session-aware builder and resume logic.
- `crates/cclab-nova/src/tools/analysis.rs`: Add `PostCommentTool`.
- `crates/cclab-nova/src/context.rs`: Ensure it can be initialized from a message list.

Technical considerations:
- Markdown parsing for checkboxes: Use regex to find `[x]` or `[X]`.
- Session Resume: The agent needs to know which comment/issue it's waiting on. This can be stored in `SessionState.metadata`.
- Rate limiting: Ensure tool calls don't spam comments.
