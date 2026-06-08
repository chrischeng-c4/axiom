# Agent Framework Examples

Examples demonstrating the cclab agent framework.

## Phase 1 (MVP) - Current Status

✅ **Implemented**:
- Basic agent execution with OpenAI
- LLM provider integration
- Tool structure and registry
- Python bindings (PyO3)

⏳ **Phase 2 (Coming Soon)**:
- Python function tool wrapping (async support)
- Actual tool execution from agents
- Conversation memory
- Anthropic Claude provider
- Streaming responses

## Setup

1. Install cclab with agent support:
```bash
cd /path/to/cclab-agent
maturin develop --features agent
```

2. Set your OpenAI API key:
```bash
export OPENAI_API_KEY="sk-..."
```

## Examples

### 1. Simple Agent (`simple_agent.py`)

Basic agent usage with OpenAI provider.

**Features**:
- Creating an OpenAI LLM provider
- Creating an agent with system prompt
- Running agent with different queries
- Accessing response metadata (tokens, model, etc.)
- Using different models and parameters

**Run**:
```bash
python python/examples/agent/simple_agent.py
```

**Expected Output**:
```
✓ Created OpenAI provider: openai
  Supported models: ['gpt-4', 'gpt-3.5-turbo', ...]

✓ Created agent: assistant
  Configuration:
    - Agent ID: assistant
    - System Prompt: You are a helpful assistant...
    - Max Turns: 10
    - Tool Timeout: 30s

============================================================
Example 1: Simple Question
============================================================

User: What's the capital of France?