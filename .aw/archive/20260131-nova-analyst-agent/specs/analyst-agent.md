---
id: analyst-agent
type: spec
title: "Analyst Agent Specification"
version: 1
spec_type: integration
created_at: 2026-01-31T09:55:23.569209+00:00
updated_at: 2026-01-31T09:55:23.569209+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-01-31T09:55:23.569209+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-01-31T09:56:12.078719+00:00
    agent: "gemini-3-flash-preview"
    tool: "create_spec"
    action: "created"
    duration_secs: 134.40
  - timestamp: 2026-01-31T09:56:37.831087+00:00
    agent: "gpt-5.2-codex"
    tool: "review_spec"
    action: "reviewed"
    duration_secs: 25.75---

<spec>

# Analyst Agent Specification

## Overview

Analyst Agent is a specialized agent for requirements gathering, research, and technical analysis. It bridges the gap between raw ideas/issues and actionable specifications. It features composable integrations with external platforms like GitHub/Jira and pluggable storage backends for persisting analysis sessions.

## Requirements

### R1 - Generic Agent Interface

```yaml
id: R1
priority: medium
status: draft
```

Define a shared Agent trait in src/agents/mod.rs that provides a unified interface for all agent types (Coding, Analyst). It must support run, run_streaming, and run_with_handler.

### R2 - AnalystAgent Implementation

```yaml
id: R2
priority: medium
status: draft
```

Implement AnalystAgent in src/agents/analyst.rs using a builder pattern. It should support configuring LLM provider, tool registry, security policy, platform integrations, and storage.

### R3 - Pluggable Storage Backend

```yaml
id: R3
priority: medium
status: draft
```

Implement a Storage trait for persisting agent context and session state. Provide MemoryStorage and FileStorage implementations.

### R4 - Platform Integration Interface

```yaml
id: R4
priority: medium
status: draft
```

Implement a PlatformIntegration trait to allow the agent to interact with external project management tools (GitHub, GitLab, Jira).

### R5 - Default Analysis Configuration

```yaml
id: R5
priority: medium
status: draft
```

Provide a default system prompt and toolset optimized for analysis (e.g., search, fetching web content, taking notes).

## Acceptance Criteria

### Scenario: Analyst Agent Analysis Workflow

- **GIVEN** AnalystAgent is given a URL to a technical article or a GitHub issue ID
- **WHEN** The agent starts the analysis process.
- **THEN** it should fetch the content, analyze it, and store key points in its session notes.

### Scenario: Session Persistence across Agent Restarts

- **GIVEN** An AnalystAgent session is saved to FileStorage
- **WHEN** A new agent is initialized with the same session ID.
- **THEN** a new agent instance should be able to resume from that session ID with full context.

### Scenario: Composable Integration Configuration

- **GIVEN** AnalystAgent is built with both GitHub and Jira integrations
- **WHEN** The agent needs to gather information from multiple platforms.
- **THEN** it should be able to use tools from both integrations within the same analysis loop.

## Flow Diagram

```mermaid
sequenceDiagram
    actor User as User
    participant Analyst as AnalystAgent
    participant Platform as PlatformIntegration
    participant Tools as AnalysisTools
    participant Storage as StorageBackend
    User->Analyst: Analyze GitHub issue #123
    Analyst->Storage: Load session context
    Analyst->Platform: Get issue content
    Platform-->Analyst: Issue data (markdown/comments)
    Analyst->Tools: Research (Search/Fetch)
    Tools-->Analyst: Data
    Analyst->Tools: TakeNoteTool (Save findings)
    Analyst->Storage: Update session state
    Analyst-->User: Final Analysis Report

stateDiagram-v2
    [*] --> Idle
    Idle --> Initializing: run() / run_streaming()
    Initializing --> LoadingState
    LoadingState --> Processing
    Processing --> ExecutingTools: Tool Call Required
    ExecutingTools --> Processing
    Processing --> UpdatingState: Final Response Received
    UpdatingState --> Idle
    Processing --> ErrorState: Execution Error
    ErrorState --> Idle
```

</spec>
