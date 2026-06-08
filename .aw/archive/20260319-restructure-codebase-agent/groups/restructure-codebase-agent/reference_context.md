---
change: restructure-codebase-agent
group: restructure-codebase-agent
date: 2026-03-19
written_by: artifact_cli
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| cclab-agent/agents.md | Agent Core Framework | high | Agent trait with run() and run_with_handler() methods, AnalystAgent and CodingAgent implementations, Agent execution loop: context manager, LLM completion, tool execution, approval flow, Agent builders with configuration support |
| cclab-agent/architecture.md | System Architecture | high | Module dependency graph: agents depend on tools, security, context, storage, C4 container diagram showing integration with external APIs, Class diagram showing Agent trait and its relationships with LLMProvider, ToolRegistry, SecurityPolicy, ContextManager |
| cclab-agent/core-types.md | Core Types | high | Message schema with role, content, tool_calls, timestamp, ToolCall and ToolResult types for tool invocation, TokenUsage type for tracking prompt/completion/total tokens, Role enum: system, user, assistant, tool |
| cclab-agent/context.md | Context Management | high | ContextManager tracks estimated_tokens, max_tokens, available_tokens, Token estimation formula: content.len() / 4, Compression flow: keep system prompt, remove oldest messages when over budget, ContextStats interface for querying context state, Preset defaults: 128k, 32k token limits |
| cclab-agent/llm-providers.md | LLM Integration | high | LLMProvider trait: provider_name(), supported_models(), validate_model(), complete(), complete_stream(), CompletionRequest schema with messages, model, temperature, max_tokens, tools, CompletionResponse with content, tool_calls, finish_reason, usage, Supported models from Claude, OpenAI, Gemini providers |
| cclab-agent/tools.md | Tool System | medium | Tool trait: name(), description(), parameters(), execute(), definition(), validate_arguments(), ToolRegistry: register, unregister, get, contains, tool_names, ToolExecutor with timeout and retry logic, ToolParameter schema with type validation |
| cclab-agent/tools-coding.md | File/Codebase Tools | medium | ReadFileTool for reading manifest files (offset, limit, max 10MB), GlobTool for finding files matching patterns, GrepTool for searching code by regex, BashTool for executing commands with timeout and output truncation |
| cclab-agent/error-handling.md | Error Handling | medium | NovaError variants: LLMError, ToolError, ApprovalRequired, SecurityViolation, ContextOverflow, CommandFailed, etc., Error classification: is_retriable() vs requires_user_action(), NovaResult type wrapper for agent functions |
| cclab-agent/security.md | Security & Approval | medium | SecurityPolicy: blocked_paths, allowed_paths, blocked_commands, tools_requiring_approval, ApprovalRequest/Response types for tool execution approval, Path validation and command filtering logic, Tool approval flow integration with agents |
| cclab-agent/streaming.md | Streaming & Events | medium | StreamEvent schema: started, thinking, text_chunk, tool_call_requested, tool_execution_*, approval_requested, completed, error, Event flow state machine, StreamHandler implementations: NoOpHandler, PrintHandler, CollectingHandler, CallbackHandler |
| cclab-agent/tools-analysis.md | Analysis Tools | medium | AskUserTool for pausing execution and requesting user input, TakeNoteTool and RecordFindingTool for capturing insights, WebSearchTool and WebFetchTool for external research, PostCommentTool for asynchronous communication |
| cclab-agent/reference-context-agent.md | Related Agents | low | Reference context generation pattern with SpecStore search and read, Relevance scoring: high/medium/low, CRR cycle integration with ReviewAgent, Contradiction detection in specifications |
| cclab-agent/integrations.md | Platform Integrations | low | PlatformIntegration trait for issue and comment APIs, Issue schema with state, labels, comments, GitHub, GitLab, Jira OpenAPI mappings |
| cclab-agent/storage.md | Session Storage | low | Storage trait: save_session, load_session, list_sessions, delete_session, SessionState schema with messages, notes, findings, MemoryStorage and FileStorage implementations |
| cclab-agent/README.md | Overview | low | Spec organization: interfaces, core types, agents, tools, storage, security, Reference to all available specification files |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: restructure-codebase-agent

**Verdict**: APPROVED

### Summary

OK

### Issues

No issues found.
