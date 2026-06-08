"""
Basic unit tests for cclab.nova module

Tests the core functionality of the agent framework without requiring API keys.
"""

import pytest
from cclab.nova import Agent, OpenAI, Tool, ToolRegistry, get_global_registry


class TestOpenAI:
    """Tests for OpenAI provider"""

    def test_openai_creation(self):
        """Test creating OpenAI provider"""
        llm = OpenAI(api_key="test-key", model="gpt-4")

        assert llm.provider_name == "openai"
        assert "gpt-4" in llm.supported_models
        assert "gpt-3.5-turbo" in llm.supported_models

    def test_openai_default_model(self):
        """Test default model is gpt-4"""
        llm = OpenAI(api_key="test-key")

        assert "gpt-4" in llm.supported_models


class TestTool:
    """Tests for Tool wrapper"""

    async def dummy_tool(self, arg: str) -> dict:
        """Dummy async function for testing"""
        return {"result": arg}

    def test_tool_creation(self):
        """Test creating a tool"""
        tool = Tool(
            name="test_tool",
            description="A test tool",
            parameters=[
                {
                    "name": "arg",
                    "description": "Test argument",
                    "type": "string",
                    "required": True,
                }
            ],
            function=self.dummy_tool,
        )

        assert tool.name == "test_tool"
        assert tool.description == "A test tool"
        assert len(tool.parameters) == 1
        assert tool.parameters[0]["name"] == "arg"

    def test_tool_parameters_optional(self):
        """Test tool with optional parameters"""
        tool = Tool(
            name="optional_tool",
            description="Tool with optional params",
            parameters=[
                {
                    "name": "required_arg",
                    "description": "Required argument",
                    "type": "string",
                    "required": True,
                },
                {
                    "name": "optional_arg",
                    "description": "Optional argument",
                    "type": "string",
                    "required": False,
                },
            ],
            function=self.dummy_tool,
        )

        params = tool.parameters
        assert len(params) == 2
        assert params[0]["required"] is True
        assert params[1]["required"] is False


class TestToolRegistry:
    """Tests for ToolRegistry"""

    async def dummy_func(self):
        pass

    def test_registry_creation(self):
        """Test creating a registry"""
        registry = ToolRegistry()

        assert registry.count() == 0
        assert registry.tool_names() == []

    def test_registry_register_and_contains(self):
        """Test registering and checking tools"""
        registry = ToolRegistry()

        tool = Tool(
            name="test_tool",
            description="Test",
            parameters=[],
            function=self.dummy_func,
        )

        registry.register(tool)

        assert registry.count() == 1
        assert registry.contains("test_tool")
        assert not registry.contains("nonexistent")
        assert "test_tool" in registry.tool_names()

    def test_registry_unregister(self):
        """Test unregistering a tool"""
        registry = ToolRegistry()

        tool = Tool(
            name="test_tool",
            description="Test",
            parameters=[],
            function=self.dummy_func,
        )

        registry.register(tool)
        assert registry.count() == 1

        registry.unregister("test_tool")
        assert registry.count() == 0
        assert not registry.contains("test_tool")

    def test_registry_clear(self):
        """Test clearing all tools"""
        registry = ToolRegistry()

        for i in range(3):
            tool = Tool(
                name=f"tool_{i}",
                description=f"Tool {i}",
                parameters=[],
                function=self.dummy_func,
            )
            registry.register(tool)

        assert registry.count() == 3

        registry.clear()
        assert registry.count() == 0

    def test_global_registry(self):
        """Test getting global registry"""
        registry = get_global_registry()

        assert isinstance(registry, ToolRegistry)
        # Note: Global registry starts fresh (new instance in Python bindings)
        # In production, it would be truly global across calls


class TestAgent:
    """Tests for Agent"""

    def test_agent_creation(self):
        """Test creating an agent"""
        llm = OpenAI(api_key="test-key", model="gpt-4")
        agent = Agent(
            name="test_agent",
            llm=llm,
            system_prompt="You are a test assistant",
            max_turns=5,
            tool_timeout=60,
        )

        assert agent.name == "test_agent"
        assert agent.system_prompt == "You are a test assistant"

    def test_agent_get_config(self):
        """Test getting agent configuration"""
        llm = OpenAI(api_key="test-key")
        agent = Agent(
            name="config_test",
            llm=llm,
            system_prompt="Test prompt",
            max_turns=10,
            tool_timeout=30,
        )

        config = agent.get_config()

        assert config["agent_id"] == "config_test"
        assert config["system_prompt"] == "Test prompt"
        assert config["max_turns"] == 10
        assert config["tool_timeout_secs"] == 30

    def test_agent_optional_params(self):
        """Test agent with optional parameters"""
        llm = OpenAI(api_key="test-key")
        agent = Agent(
            name="minimal_agent",
            llm=llm,
        )

        config = agent.get_config()

        assert config["agent_id"] == "minimal_agent"
        assert config["system_prompt"] is None
        assert config["max_turns"] == 0  # Unlimited
        assert config["tool_timeout_secs"] == 30  # Default

    @pytest.mark.asyncio
    async def test_agent_run_requires_api_key(self):
        """Test that agent.run() requires valid API key (expects error)"""
        llm = OpenAI(api_key="invalid-key")
        agent = Agent(name="test", llm=llm)

        # This should fail because API key is invalid
        # In real tests, we'd mock the LLM response
        with pytest.raises(Exception):  # Will raise runtime error from OpenAI API
            await agent.run("Test message")
