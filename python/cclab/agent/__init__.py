"""
cclab.agent: LLM-powered agent framework with tool calling.

This module provides a comprehensive agent framework similar to LangChain/LangGraph/PydanticAI,
with deep Rust integration for high performance and GIL-free execution.

Features:
    - LLM Providers (OpenAI, Claude)
    - Tool calling and registration
    - Memory management
    - Multi-agent collaboration
    - Workflow orchestration
    - Type-safe agent execution

Quick Start:
    >>> from cclab.agent import Agent, OpenAI
    >>>
    >>> # Create an LLM-powered agent
    >>> llm = OpenAI(api_key="sk-...")
    >>> agent = Agent(name="assistant", llm=llm, system_prompt="You are a helpful assistant")
    >>>
    >>> # Run the agent
    >>> response = await agent.run("What's the capital of France?")
    >>> print(response["content"])  # "Paris"

Tool Calling:
    >>> from cclab.agent import Agent, Tool, get_global_registry
    >>>
    >>> # Define a tool
    >>> @Tool(name="search", description="Search the web")
    >>> async def search(query: str) -> dict:
    ...     return {"results": ["result1", "result2"]}
    >>>
    >>> # Register the tool
    >>> registry = get_global_registry()
    >>> registry.register(search)
    >>>
    >>> # Create agent with tools
    >>> agent = Agent(llm=OpenAI(), tools=["search"])
    >>> response = await agent.run("Find news about AI")

Classes:
    Agent: LLM-powered agent with tool calling capability.
        Args:
            name: Agent name/identifier
            llm: LLM provider (OpenAI, Claude, etc.)
            system_prompt: System prompt (optional)
            max_turns: Maximum conversation turns (0 = unlimited)
            tool_timeout: Tool execution timeout in seconds (default: 30)
            tools: List of tool names to use (optional)

        Methods:
            run(input, model=None, temperature=None, max_tokens=None) -> dict
            run_with_context(context, input, model=None, temperature=None, max_tokens=None) -> dict
            get_config() -> dict

    OpenAI: OpenAI LLM provider (GPT-4, GPT-3.5, etc.).
        Args:
            api_key: OpenAI API key
            model: Default model to use (default: "gpt-4")

        Methods:
            complete(messages, model=None, temperature=None, max_tokens=None, tools=None) -> dict
            complete_stream(messages, model=None, temperature=None, max_tokens=None) -> AsyncIterator

    Tool: Tool wrapper for Python async functions.
        Args:
            name: Tool name
            description: Tool description
            parameters: List of parameter definitions
            function: Python async function to execute

        Methods:
            execute(arguments) -> Any

    ToolRegistry: Thread-safe tool registry.
        Methods:
            register(tool) -> None
            unregister(name) -> None
            contains(name) -> bool
            tool_names() -> List[str]
            count() -> int
            clear() -> None

Functions:
    get_global_registry() -> ToolRegistry: Get the global tool registry singleton

Example Response Format:
    {
        "content": "The capital of France is Paris.",
        "finish_reason": "stop",
        "model": "gpt-4",
        "usage": {
            "prompt_tokens": 15,
            "completion_tokens": 8,
            "total_tokens": 23
        },
        "tool_calls": [  # Optional, if tools were called
            {
                "id": "call_123",
                "name": "search",
                "arguments": "{\"query\": \"AI news\"}"
            }
        ]
    }
"""

from __future__ import annotations

# Import from the Rust extension (cclab.abi3.so in parent directory)
from ..cclab import agent as _agent_rust

# LLM Providers
OpenAI = _agent_rust.OpenAI

# Agent
Agent = _agent_rust.Agent

# Tools
Tool = _agent_rust.Tool
ToolRegistry = _agent_rust.ToolRegistry
get_global_registry = _agent_rust.get_global_registry

__all__ = [
    # LLM Providers
    "OpenAI",
    # Agent
    "Agent",
    # Tools
    "Tool",
    "ToolRegistry",
    "get_global_registry",
]
