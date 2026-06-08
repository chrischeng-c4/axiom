---
id: agent-architecture-spec
main_spec_ref: "agent/logic/architecture.md"
fill_sections: [overview, logic, changes]
---

# agent Architecture Spec

## Overview
<!-- type: overview lang: markdown -->

`agent` is an LLM agent framework built around shared ports for agents,
LLM providers, tools, storage, streaming, security, context management, and
platform integrations. The architecture keeps provider-specific API clients and
platform integrations behind traits while higher-level agents compose those
ports for coding and analysis workflows.

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: agent-module-dependency
title: agent Module Dependency
---
graph TD
    agents --> llm
    agents --> tools
    agents --> security
    agents --> stream
    agents --> context
    agents --> storage
    agents --> integrations

    tools --> types
    tools --> storage

    llm --> types
    llm --> cclab-fetch

    storage --> types

    integrations --> cclab-fetch
    integrations --> tools
```

```mermaid
---
id: agent-system-context
title: agent System Context
---
C4Context
    Person(user, "User")
    System(agent, "agent", "LLM Agent Framework")

    System_Ext(claude, "Anthropic API")
    System_Ext(openai, "OpenAI API")
    System_Ext(gemini, "Gemini API")
    System_Ext(github, "GitHub API")
    System_Ext(gitlab, "GitLab API")
    System_Ext(jira, "Jira API")
    System_Ext(fs, "File System")

    Rel(user, agent, "run / run_conversation")
    Rel(agent, claude, "Messages API")
    Rel(agent, openai, "Chat Completions")
    Rel(agent, gemini, "generateContent")
    Rel(agent, github, "REST v3")
    Rel(agent, gitlab, "REST v4")
    Rel(agent, jira, "REST v3")
    Rel(agent, fs, "read / write / glob / grep")
```

```mermaid
---
id: agent-module-classes
title: agent Module Classes
---
classDiagram
    class Agent {
        <<trait>>
        +run(input) NovaResult~String~
        +run_with_handler(input, handler) NovaResult~String~
    }

    class LLMProvider {
        <<trait>>
        +provider_name() str
        +supported_models() Vec~String~
        +validate_model(model) bool
        +complete(request) NovaResult~CompletionResponse~
        +complete_stream(request) NovaResult~StreamResponse~
    }

    class Tool {
        <<trait>>
        +name() str
        +description() str
        +parameters() Vec~ToolParameter~
        +execute(arguments) NovaResult~Value~
        +definition() ToolDefinition
        +validate_arguments(args) NovaResult
    }

    class Storage {
        <<trait>>
        +save_session(session) NovaResult
        +load_session(id) NovaResult~SessionState~
        +list_sessions() NovaResult~Vec~SessionInfo~~
        +delete_session(id) NovaResult
        +session_exists(id) NovaResult~bool~
    }

    class PlatformIntegration {
        <<trait>>
        +name() str
        +get_issue(id) NovaResult~Issue~
        +list_issues(filter) NovaResult~Vec~IssueSummary~~
        +get_comments(issue_id) NovaResult~Vec~IssueComment~~
        +post_comment(issue_id, body) NovaResult
        +into_tools() Vec~Arc~dyn Tool~~
    }

    class StreamHandler {
        <<trait>>
        +on_event(event)
        +should_cancel() bool
    }

    class ApprovalHandler {
        <<trait>>
        +request_approval(request) ApprovalResponse
    }

    CodingAgent ..|> Agent
    AnalystAgent ..|> Agent

    CodingAgent --> LLMProvider
    CodingAgent --> ToolRegistry
    CodingAgent --> SecurityPolicy
    CodingAgent --> ContextManager

    AnalystAgent --> LLMProvider
    AnalystAgent --> ToolRegistry
    AnalystAgent --> SecurityPolicy
    AnalystAgent --> ContextManager
    AnalystAgent --> Storage
    AnalystAgent --> PlatformIntegration

    ClaudeProvider ..|> LLMProvider
    OpenAIProvider ..|> LLMProvider
    GeminiProvider ..|> LLMProvider

    MemoryStorage ..|> Storage
    FileStorage ..|> Storage

    GitHubIntegration ..|> PlatformIntegration
    GitLabIntegration ..|> PlatformIntegration
    JiraIntegration ..|> PlatformIntegration

    ToolRegistry "1" --> "*" Tool
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agent/core/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose the agent module surface used by agents, LLM providers, tools, storage, stream, context, security, and integrations."
  - path: projects/agent/core/src/agents/
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Compose framework ports into concrete coding and analysis agents."
  - path: projects/agent/core/src/llm/
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Keep model-provider implementations behind the shared LLMProvider boundary."
```
