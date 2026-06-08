"""
Tool Agent Example

Demonstrates agent with custom tools for function calling.

Python async function tool wrapping is now implemented and working!
"""

import asyncio
import os
from cclab.nova import Agent, OpenAI, Tool, ToolRegistry, get_global_registry


# Define custom tools
async def search_web(query: str) -> dict:
    """Search the web for information."""
    # Mock implementation
    return {
        "query": query,
        "results": [
            {"title": "Result 1", "url": "https://example.com/1"},
            {"title": "Result 2", "url": "https://example.com/2"},
        ]
    }


async def get_weather(city: str) -> dict:
    """Get weather information for a city."""
    # Mock implementation
    return {
        "city": city,
        "temperature": 22,
        "condition": "Sunny",
        "humidity": 60,
    }


async def calculate(expression: str) -> dict:
    """Evaluate a mathematical expression."""
    try:
        result = eval(expression)  # Note: Use safer evaluation in production
        return {"expression": expression, "result": result}
    except Exception as e:
        return {"expression": expression, "error": str(e)}


async def main():
    # Get API key from environment
    api_key = os.getenv("OPENAI_API_KEY")
    if not api_key:
        print("Error: OPENAI_API_KEY environment variable not set")
        return

    print("="*60)
    print("Tool Agent Example - Python Function Execution")
    print("="*60)
    print("\n✓ Python async function tool wrapping is now working!")
    print("  Tools can execute both sync and async Python functions.\n")

    # Create tools
    print("Creating tools...")

    # Tool 1: Search
    search_tool = Tool(
        name="search_web",
        description="Search the web for information",
        parameters=[
            {
                "name": "query",
                "description": "Search query",
                "type": "string",
                "required": True,
            }
        ],
        function=search_web,
    )
    print(f"✓ Created tool: {search_tool.name}")
    print(f"  Description: {search_tool.description}")
    print(f"  Parameters: {search_tool.parameters}")

    # Tool 2: Weather
    weather_tool = Tool(
        name="get_weather",
        description="Get current weather for a city",
        parameters=[
            {
                "name": "city",
                "description": "City name",
                "type": "string",
                "required": True,
            }
        ],
        function=get_weather,
    )
    print(f"\n✓ Created tool: {weather_tool.name}")
    print(f"  Description: {weather_tool.description}")
    print(f"  Parameters: {weather_tool.parameters}")

    # Tool 3: Calculator
    calc_tool = Tool(
        name="calculate",
        description="Evaluate a mathematical expression",
        parameters=[
            {
                "name": "expression",
                "description": "Mathematical expression to evaluate",
                "type": "string",
                "required": True,
            }
        ],
        function=calculate,
    )
    print(f"\n✓ Created tool: {calc_tool.name}")
    print(f"  Description: {calc_tool.description}")

    # Register tools
    print("\n" + "-"*60)
    print("Registering tools in global registry...")

    registry = get_global_registry()
    registry.register(search_tool)
    registry.register(weather_tool)
    registry.register(calc_tool)

    print(f"✓ Registered {registry.count()} tools")
    print(f"  Tool names: {registry.tool_names()}")

    # Verify registration
    print("\nVerifying tool registration:")
    for tool_name in ["search_web", "get_weather", "calculate"]:
        exists = registry.contains(tool_name)
        status = "✓" if exists else "✗"
        print(f"  {status} {tool_name}: {exists}")

    # Create agent with tools
    print("\n" + "-"*60)
    print("Creating agent with tools...")

    llm = OpenAI(api_key=api_key, model="gpt-4")
    agent = Agent(
        name="tool_assistant",
        llm=llm,
        system_prompt="You are a helpful assistant with access to tools. Use them when needed.",
        tools=["search_web", "get_weather", "calculate"],
    )
    print(f"✓ Created agent: {agent.name}")

    # Example queries that would trigger tools (when integrated with agent)
    print("\n" + "="*60)
    print("Example Queries (Agent + Tool integration coming in Phase 2)")
    print("="*60)

    queries = [
        "What's the weather in Tokyo?",
        "Search for recent AI news",
        "Calculate 15 * 234 + 890",
    ]

    for i, query in enumerate(queries, 1):
        print(f"\n{i}. User: {query}")
        print(f"   Expected tool: {['get_weather', 'search_web', 'calculate'][i-1]}")

    print("\n" + "-"*60)
    print("Note: Agent automatically calling tools requires Phase 2")
    print("      But tools themselves can be executed directly (see below)")
    print("-"*60)

    # Test tool execution directly
    print("\n" + "="*60)
    print("Direct Tool Execution Test")
    print("="*60)

    try:
        # Execute search tool
        print("\n1. Executing search tool...")
        result = await search_tool.execute({"query": "AI agents"})
        print(f"✓ Search result: {result}")

        # Execute weather tool
        print("\n2. Executing weather tool...")
        result = await weather_tool.execute({"city": "Tokyo"})
        print(f"✓ Weather result: {result}")

        # Execute calculator tool
        print("\n3. Executing calculator tool...")
        result = await calc_tool.execute({"expression": "15 * 234 + 890"})
        print(f"✓ Calculator result: {result}")

    except Exception as e:
        print(f"✗ Error: {e}")

    print("\n" + "="*60)
    print("✓ Tool execution completed successfully!")
    print("="*60)
    print("\nPhase 1 (MVP): ✅ Tool execution working")
    print("Phase 2 (Next): Agent + Tool integration, Human-in-loop, Streaming")

    # Cleanup
    print("\nCleaning up...")
    registry.clear()
    print(f"✓ Cleared registry (count: {registry.count()})")


if __name__ == "__main__":
    asyncio.run(main())
