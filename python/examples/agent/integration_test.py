"""
Comprehensive Integration Test for cclab.nova

Tests all Phase 1 + Phase 2 (tool execution) functionality with real OpenAI API.

Requirements:
- OpenAI API key set in OPENAI_API_KEY environment variable
- maturin develop --features agent (build Python bindings)

Usage:
    export OPENAI_API_KEY="sk-..."
    python python/examples/agent/integration_test.py
"""

import asyncio
import os
import sys
from datetime import datetime


def print_section(title: str):
    """Print a formatted section header"""
    print("\n" + "=" * 70)
    print(f"  {title}")
    print("=" * 70)


def print_success(message: str):
    """Print success message"""
    print(f"✓ {message}")


def print_error(message: str):
    """Print error message"""
    print(f"✗ {message}")


def print_info(message: str):
    """Print info message"""
    print(f"ℹ {message}")


async def test_imports():
    """Test 1: Import all agent modules"""
    print_section("Test 1: Module Imports")

    try:
        from cclab.nova import Agent, OpenAI, Tool, ToolRegistry, get_global_registry

        print_success("Imported Agent")
        print_success("Imported OpenAI")
        print_success("Imported Tool")
        print_success("Imported ToolRegistry")
        print_success("Imported get_global_registry")
        return True
    except ImportError as e:
        print_error(f"Import failed: {e}")
        return False


async def test_openai_provider():
    """Test 2: OpenAI provider creation"""
    print_section("Test 2: OpenAI Provider")

    try:
        from cclab.nova import OpenAI

        # Get API key
        api_key = os.getenv("OPENAI_API_KEY")
        if not api_key:
            print_error("OPENAI_API_KEY environment variable not set")
            return False

        # Create provider
        llm = OpenAI(api_key=api_key, model="gpt-4o-mini")
        print_success(f"Created OpenAI provider: {llm.provider_name}")

        # Check supported models
        models = llm.supported_models
        print_success(f"Supports {len(models)} models")
        print_info(f"Models: {', '.join(models[:5])}...")

        return True
    except Exception as e:
        print_error(f"Provider creation failed: {e}")
        return False


async def test_basic_agent():
    """Test 3: Basic agent execution"""
    print_section("Test 3: Basic Agent Execution")

    try:
        from cclab.nova import Agent, OpenAI

        api_key = os.getenv("OPENAI_API_KEY")
        if not api_key:
            print_error("OPENAI_API_KEY not set")
            return False

        # Create agent (using gpt-4o-mini - fast and cheap for testing)
        llm = OpenAI(api_key=api_key, model="gpt-4o-mini")
        agent = Agent(
            name="test_agent",
            llm=llm,
            system_prompt="You are a helpful assistant. Be very concise.",
            max_turns=10,
        )
        print_success(f"Created agent: {agent.name}")

        # Get config
        config = agent.get_config()
        print_info(f"Agent ID: {config['agent_id']}")
        print_info(f"Max turns: {config['max_turns']}")
        print_info(f"Tool timeout: {config['tool_timeout_secs']}s")

        # Simple query
        print_info("Sending query: 'What is 2+2? Answer in 3 words or less.'")
        start = datetime.now()
        response = await agent.run(
            "What is 2+2? Answer in 3 words or less.", temperature=0.0, max_tokens=50
        )
        duration = (datetime.now() - start).total_seconds()

        print_success(f"Got response in {duration:.2f}s")
        print_info(f"Content: {response['content']}")
        print_info(f"Model: {response['model']}")
        print_info(f"Finish reason: {response['finish_reason']}")
        print_info(
            f"Tokens: {response['usage']['total_tokens']} "
            f"(prompt: {response['usage']['prompt_tokens']}, "
            f"completion: {response['usage']['completion_tokens']})"
        )

        return True
    except Exception as e:
        print_error(f"Agent execution failed: {e}")
        import traceback

        traceback.print_exc()
        return False


async def test_tool_creation():
    """Test 4: Tool creation and registration"""
    print_section("Test 4: Tool Creation & Registration")

    try:
        from cclab.nova import Tool, ToolRegistry, get_global_registry

        # Create async tool
        async def calculate(expression: str) -> dict:
            """Calculate a mathematical expression"""
            try:
                result = eval(expression)  # Note: Use safer evaluation in production
                return {"expression": expression, "result": result, "success": True}
            except Exception as e:
                return {"expression": expression, "error": str(e), "success": False}

        tool = Tool(
            name="calculate",
            description="Evaluate a mathematical expression",
            parameters=[
                {
                    "name": "expression",
                    "description": "Math expression to evaluate",
                    "type": "string",
                    "required": True,
                }
            ],
            function=calculate,
        )

        print_success(f"Created tool: {tool.name}")
        print_info(f"Description: {tool.description}")
        print_info(f"Parameters: {len(tool.parameters)}")

        # Register tool
        registry = ToolRegistry()
        registry.register(tool)
        print_success(f"Registered tool (registry count: {registry.count()})")

        # Verify registration
        assert registry.contains("calculate")
        print_success("Tool found in registry")

        return True
    except Exception as e:
        print_error(f"Tool creation failed: {e}")
        import traceback

        traceback.print_exc()
        return False


async def test_tool_execution():
    """Test 5: Tool execution (sync and async)"""
    print_section("Test 5: Tool Execution")

    try:
        from cclab.nova import Tool

        # Test 1: Sync function tool
        def sync_greet(name: str) -> dict:
            """Greet someone (sync)"""
            return {"message": f"Hello, {name}!", "type": "sync"}

        sync_tool = Tool(
            name="sync_greet",
            description="Greet someone",
            parameters=[
                {"name": "name", "description": "Name", "type": "string", "required": True}
            ],
            function=sync_greet,
        )

        result = await sync_tool.execute({"name": "Alice"})
        print_success(f"Sync tool executed: {result['message']}")
        assert result["type"] == "sync"

        # Test 2: Async function tool
        async def async_multiply(x: int, y: int) -> dict:
            """Multiply two numbers (async)"""
            await asyncio.sleep(0.01)  # Simulate async operation
            return {"result": x * y, "operation": "multiply", "type": "async"}

        async_tool = Tool(
            name="async_multiply",
            description="Multiply numbers",
            parameters=[
                {"name": "x", "description": "First number", "type": "integer", "required": True},
                {"name": "y", "description": "Second number", "type": "integer", "required": True},
            ],
            function=async_multiply,
        )

        result = await async_tool.execute({"x": 7, "y": 8})
        print_success(f"Async tool executed: {result['result']}")
        assert result["type"] == "async"
        assert result["result"] == 56

        # Test 3: Complex return values
        async def complex_data() -> dict:
            """Return complex nested data"""
            return {
                "status": "success",
                "data": {
                    "users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}],
                    "count": 2,
                },
                "metadata": {"timestamp": "2026-01-21", "version": "1.0"},
            }

        complex_tool = Tool(
            name="complex_data",
            description="Get complex data",
            parameters=[],
            function=complex_data,
        )

        result = await complex_tool.execute({})
        print_success("Complex data tool executed")
        assert result["status"] == "success"
        assert result["data"]["count"] == 2
        assert len(result["data"]["users"]) == 2

        return True
    except Exception as e:
        print_error(f"Tool execution failed: {e}")
        import traceback

        traceback.print_exc()
        return False


async def test_advanced_agent():
    """Test 6: Advanced agent queries"""
    print_section("Test 6: Advanced Agent Queries")

    try:
        from cclab.nova import Agent, OpenAI

        api_key = os.getenv("OPENAI_API_KEY")
        if not api_key:
            print_error("OPENAI_API_KEY not set")
            return False

        llm = OpenAI(api_key=api_key, model="gpt-3.5-turbo")  # Use cheaper model
        agent = Agent(
            name="advanced_agent",
            llm=llm,
            system_prompt="You are a helpful assistant. Be concise.",
        )

        # Test multiple queries
        queries = [
            ("Name a programming language", 20),
            ("What is the capital of France? One word answer.", 10),
            ("Calculate 15 * 3. Just give the number.", 10),
        ]

        for query, max_tokens in queries:
            print_info(f"Query: {query}")
            start = datetime.now()
            response = await agent.run(query, temperature=0.0, max_tokens=max_tokens)
            duration = (datetime.now() - start).total_seconds()

            print_success(f"Response ({duration:.2f}s): {response['content'][:50]}")
            print_info(f"Tokens used: {response['usage']['total_tokens']}")

        return True
    except Exception as e:
        print_error(f"Advanced queries failed: {e}")
        import traceback

        traceback.print_exc()
        return False


async def main():
    """Run all integration tests"""
    print("\n" + "🚀 " * 25)
    print("  cclab.nova Integration Test Suite")
    print("  Testing Phase 1 (MVP) + Phase 2 (Tool Execution)")
    print("🚀 " * 25)

    # Check API key first
    api_key = os.getenv("OPENAI_API_KEY")
    if not api_key:
        print_error("\nOPENAI_API_KEY environment variable not set!")
        print_info("Please set it with: export OPENAI_API_KEY='sk-...' or add to .env file")
        sys.exit(1)

    print_info(f"API key: {api_key[:10]}...{api_key[-4:]}")

    # Run tests
    tests = [
        ("Module Imports", test_imports),
        ("OpenAI Provider", test_openai_provider),
        ("Basic Agent", test_basic_agent),
        ("Tool Creation", test_tool_creation),
        ("Tool Execution", test_tool_execution),
        ("Advanced Queries", test_advanced_agent),
    ]

    results = []
    start_time = datetime.now()

    for test_name, test_func in tests:
        try:
            result = await test_func()
            results.append((test_name, result))
        except Exception as e:
            print_error(f"Test '{test_name}' crashed: {e}")
            results.append((test_name, False))

    # Summary
    duration = (datetime.now() - start_time).total_seconds()

    print_section("Test Summary")
    passed = sum(1 for _, result in results if result)
    total = len(results)

    for test_name, result in results:
        status = "✓ PASS" if result else "✗ FAIL"
        print(f"{status} - {test_name}")

    print(f"\nTotal: {passed}/{total} passed ({passed/total*100:.1f}%)")
    print(f"Duration: {duration:.2f}s")

    if passed == total:
        print("\n🎉 All tests passed! Agent framework is working correctly.")
        print("✅ Phase 1 (MVP): Complete")
        print("✅ Phase 2 (Tool Execution): Complete")
        sys.exit(0)
    else:
        print(f"\n⚠️  {total - passed} test(s) failed")
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
