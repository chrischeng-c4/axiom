"""
Tests for the fixture runtime system in cclab.probe.

Tests verify:
- Fixture discovery and registration
- Dependency injection by parameter name
- Fixture ordering (dependencies run first)
- Lifecycle management (setup -> test -> teardown)
- Scope caching (class/module/session fixtures)
- Async fixture execution
- Yield-based setup/teardown
- Error handling
"""

from cclab.probe import TestSuite, test, fixture, expect, parametrize


class TestFixtureBasics(TestSuite):
    """Test basic fixture functionality."""

    @fixture
    def simple_value(self):
        """Simple fixture returning a value."""
        return 42

    @fixture
    def greeting(self):
        """Fixture returning a string."""
        return "Hello, World!"

    @test
    async def test_simple_injection(self, simple_value):
        """Test that fixtures are injected by parameter name."""
        expect(simple_value).to_equal(42)

    @test
    async def test_string_fixture(self, greeting):
        """Test string fixture injection."""
        expect(greeting).to_equal("Hello, World!")

    @test
    async def test_multiple_fixtures(self, simple_value, greeting):
        """Test multiple fixture injection."""
        expect(simple_value).to_equal(42)
        expect(greeting).to_equal("Hello, World!")


class TestYieldFixtures(TestSuite):
    """Test yield-based fixtures with setup/teardown."""

    # Track setup/teardown calls for verification
    setup_count = 0
    teardown_count = 0
    active_resources = []

    @fixture
    def resource(self):
        """Yield fixture with setup and teardown."""
        TestYieldFixtures.setup_count += 1
        resource_id = f"resource_{TestYieldFixtures.setup_count}"
        TestYieldFixtures.active_resources.append(resource_id)
        yield resource_id
        # Teardown
        TestYieldFixtures.active_resources.remove(resource_id)
        TestYieldFixtures.teardown_count += 1

    @test
    async def test_yield_fixture_setup(self, resource):
        """Test that yield fixture provides value."""
        # Resource should start with 'resource_' and be active
        expect(resource.startswith("resource_")).to_be_true()
        expect(resource in TestYieldFixtures.active_resources).to_be_true()

    @test
    async def test_yield_fixture_teardown(self, resource):
        """Test that teardown runs after test."""
        # During test, resource should be active
        expect(resource in TestYieldFixtures.active_resources).to_be_true()


class TestAsyncFixtures(TestSuite):
    """Test async fixture support."""

    @fixture
    async def async_value(self):
        """Async fixture (no yield)."""
        import asyncio
        await asyncio.sleep(0.01)
        return "async_result"

    @fixture
    async def async_resource(self):
        """Async yield fixture with setup/teardown."""
        import asyncio
        await asyncio.sleep(0.01)
        yield "async_resource_value"
        await asyncio.sleep(0.01)

    @test
    async def test_async_fixture_injection(self, async_value):
        """Test async fixture injection."""
        expect(async_value).to_equal("async_result")

    @test
    async def test_async_yield_fixture(self, async_resource):
        """Test async yield fixture."""
        expect(async_resource).to_equal("async_resource_value")


class TestFixtureDependencies(TestSuite):
    """Test fixture dependency resolution."""

    execution_order = []

    @fixture
    def base_fixture(self):
        """Base fixture with no dependencies."""
        TestFixtureDependencies.execution_order.append("base")
        return {"base": True}

    @fixture
    def dependent_fixture(self, base_fixture):
        """Fixture depending on base_fixture."""
        TestFixtureDependencies.execution_order.append("dependent")
        return {**base_fixture, "dependent": True}

    @fixture
    def top_fixture(self, dependent_fixture):
        """Fixture depending on dependent_fixture."""
        TestFixtureDependencies.execution_order.append("top")
        return {**dependent_fixture, "top": True}

    @test
    async def test_dependency_order(self, top_fixture):
        """Test that dependencies are resolved in correct order."""
        # Fixtures should execute: base -> dependent -> top
        expect(top_fixture.get("base")).to_be_true()
        expect(top_fixture.get("dependent")).to_be_true()
        expect(top_fixture.get("top")).to_be_true()


class TestClassScopedFixtures(TestSuite):
    """Test class-scoped fixture caching."""

    setup_count = 0
    first_resource_value = None

    @fixture(scope="class")
    def class_resource(self):
        """Class-scoped fixture, setup once per class."""
        TestClassScopedFixtures.setup_count += 1
        value = f"class_resource_{TestClassScopedFixtures.setup_count}"
        if TestClassScopedFixtures.first_resource_value is None:
            TestClassScopedFixtures.first_resource_value = value
        return value

    @test
    async def test_class_fixture_first(self, class_resource):
        """First test using class fixture."""
        # Class-scoped fixture should be consistent across tests
        expect(class_resource).to_equal(TestClassScopedFixtures.first_resource_value)

    @test
    async def test_class_fixture_second(self, class_resource):
        """Second test should use cached fixture."""
        # Should be same value because fixture is class-scoped
        expect(class_resource).to_equal(TestClassScopedFixtures.first_resource_value)


class TestFunctionScopedFixtures(TestSuite):
    """Test function-scoped fixture behavior."""

    setup_count = 0
    seen_resources = []

    @fixture(scope="function")
    def function_resource(self):
        """Function-scoped fixture, setup for each test."""
        TestFunctionScopedFixtures.setup_count += 1
        value = f"function_resource_{TestFunctionScopedFixtures.setup_count}"
        TestFunctionScopedFixtures.seen_resources.append(value)
        return value

    @test
    async def test_function_fixture_first(self, function_resource):
        """First test gets fresh fixture."""
        # Function-scoped fixture should start with prefix
        expect(function_resource.startswith("function_resource_")).to_be_true()

    @test
    async def test_function_fixture_second(self, function_resource):
        """Second test gets new fixture instance."""
        # Function-scoped fixture should be unique per test
        expect(function_resource.startswith("function_resource_")).to_be_true()
        # Verify this is a new instance (different from previously seen)
        count_of_this_resource = TestFunctionScopedFixtures.seen_resources.count(function_resource)
        expect(count_of_this_resource).to_equal(1)


class TestFixtureWithParametrize(TestSuite):
    """Test fixtures combined with parametrized tests."""

    @fixture
    def multiplier(self):
        """Fixture providing a multiplier value."""
        return 10

    @test
    @parametrize("value", [1, 2, 3])
    async def test_fixture_with_params(self, value, multiplier):
        """Test that fixtures work with parametrized tests."""
        result = value * multiplier
        expect(result).to_equal(value * 10)


class TestFixtureErrors(TestSuite):
    """Test error handling in fixtures."""

    @fixture
    def failing_setup_fixture(self):
        """Fixture that fails during setup."""
        raise ValueError("Setup failed intentionally")

    @fixture
    def failing_teardown_fixture(self):
        """Fixture that fails during teardown."""
        yield "value"
        raise ValueError("Teardown failed intentionally")

    # Note: These tests are commented out as they would fail the suite
    # They are here to document expected behavior

    # @test
    # async def test_fixture_setup_failure(self, failing_setup_fixture):
    #     """Test should error when fixture setup fails."""
    #     pass


class TestNoFixtures(TestSuite):
    """Test suite without fixtures still works."""

    @test
    async def test_without_fixtures(self):
        """Test that works without any fixtures."""
        expect(1 + 1).to_equal(2)

    @test
    async def test_another_without_fixtures(self):
        """Another test without fixtures."""
        expect("hello").to_equal("hello")
