"""
Tool execution tests for cclab.nova module

Tests that Python async functions can be executed as tools.
"""

import pytest
from cclab.nova import Tool, ToolRegistry


class TestToolExecution:
    """Tests for Python function tool execution"""

    @pytest.mark.asyncio
    async def test_sync_function_tool(self):
        """Test executing a synchronous Python function as a tool"""

        def sync_add(a: int, b: int) -> dict:
            """Add two numbers"""
            return {"result": a + b}

        tool = Tool(
            name="add",
            description="Add two numbers",
            parameters=[
                {"name": "a", "description": "First number", "type": "integer", "required": True},
                {"name": "b", "description": "Second number", "type": "integer", "required": True},
            ],
            function=sync_add,
        )

        assert tool.name == "add"
        assert tool.description == "Add two numbers"

        # Execute the tool
        result = await tool.execute({"a": 5, "b": 3})
        assert result["result"] == 8

    @pytest.mark.asyncio
    async def test_async_function_tool(self):
        """Test executing an async Python function as a tool"""

        async def async_multiply(x: int, y: int) -> dict:
            """Multiply two numbers asynchronously"""
            return {"result": x * y, "operation": "multiply"}

        tool = Tool(
            name="multiply",
            description="Multiply two numbers",
            parameters=[
                {"name": "x", "description": "First number", "type": "integer", "required": True},
                {"name": "y", "description": "Second number", "type": "integer", "required": True},
            ],
            function=async_multiply,
        )

        assert tool.name == "multiply"

        # Execute the tool
        result = await tool.execute({"x": 4, "y": 7})
        assert result["result"] == 28
        assert result["operation"] == "multiply"

    @pytest.mark.asyncio
    async def test_tool_with_string_args(self):
        """Test tool with string arguments"""

        async def greet(name: str, greeting: str = "Hello") -> dict:
            """Greet someone"""
            return {"message": f"{greeting}, {name}!"}

        tool = Tool(
            name="greet",
            description="Greet someone",
            parameters=[
                {"name": "name", "description": "Person's name", "type": "string", "required": True},
                {
                    "name": "greeting",
                    "description": "Greeting word",
                    "type": "string",
                    "required": False,
                },
            ],
            function=greet,
        )

        # Test with required args only
        result = await tool.execute({"name": "Alice"})
        assert "Alice" in result["message"]

        # Test with optional args
        result = await tool.execute({"name": "Bob", "greeting": "Hi"})
        assert result["message"] == "Hi, Bob!"

    @pytest.mark.asyncio
    async def test_tool_with_dict_return(self):
        """Test tool that returns a complex dict"""

        async def get_user_info(user_id: int) -> dict:
            """Get user information (mock)"""
            return {
                "user_id": user_id,
                "name": f"User {user_id}",
                "email": f"user{user_id}@example.com",
                "active": True,
                "metadata": {"created_at": "2026-01-01", "role": "user"},
            }

        tool = Tool(
            name="get_user",
            description="Get user information",
            parameters=[
                {"name": "user_id", "description": "User ID", "type": "integer", "required": True},
            ],
            function=get_user_info,
        )

        result = await tool.execute({"user_id": 123})
        assert result["user_id"] == 123
        assert result["name"] == "User 123"
        assert result["active"] is True
        assert "metadata" in result
        assert result["metadata"]["role"] == "user"

    @pytest.mark.asyncio
    async def test_tool_execution_error(self):
        """Test that tool execution errors are handled"""

        async def failing_tool(value: int) -> dict:
            """Tool that raises an error"""
            if value < 0:
                raise ValueError("Value must be non-negative")
            return {"value": value}

        tool = Tool(
            name="failing_tool",
            description="A tool that can fail",
            parameters=[
                {"name": "value", "description": "Input value", "type": "integer", "required": True},
            ],
            function=failing_tool,
        )

        # Should succeed with valid input
        result = await tool.execute({"value": 5})
        assert result["value"] == 5

        # Should fail with invalid input
        with pytest.raises(Exception):  # Will raise PyRuntimeError with sanitized message
            await tool.execute({"value": -1})

    @pytest.mark.asyncio
    async def test_tool_in_registry(self):
        """Test tool execution from registry"""

        async def search(query: str) -> dict:
            """Mock search function"""
            return {"query": query, "results": ["result1", "result2"]}

        tool = Tool(
            name="search",
            description="Search for information",
            parameters=[
                {"name": "query", "description": "Search query", "type": "string", "required": True},
            ],
            function=search,
        )

        # Register the tool
        registry = ToolRegistry()
        registry.register(tool)

        assert registry.contains("search")
        assert registry.count() == 1

        # Execute the tool
        result = await tool.execute({"query": "AI agents"})
        assert result["query"] == "AI agents"
        assert len(result["results"]) == 2

        # Cleanup
        registry.unregister("search")
        assert not registry.contains("search")
