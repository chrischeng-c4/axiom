"""
Test fixture system for data-bridge-test

This test verifies that the fixture system works correctly.
"""

import pytest
from cclab.probe import expect
from cclab.probe import (
    fixture, test, expect, TestSuite, run_suite,
    FixtureScope, FixtureRegistry, FixtureMeta
)


def test_fixture_scope_enum():
    """Test FixtureScope enum"""
    assert FixtureScope.Function is not None
    assert FixtureScope.Class is not None
    assert FixtureScope.Module is not None
    assert FixtureScope.Session is not None

    # Test string conversion
    scope = FixtureScope.from_string("function")
    assert scope == FixtureScope.Function

    scope = FixtureScope.from_string("class")
    assert scope == FixtureScope.Class


def test_fixture_registry_basic():
    """Test basic FixtureRegistry operations"""
    registry = FixtureRegistry()

    # Initially empty
    assert len(registry) == 0
    assert not registry.has_fixture("my_fixture")

    # Register a fixture
    registry.register(
        name="my_fixture",
        scope=FixtureScope.Function,
        autouse=False,
        dependencies=[],
        has_teardown=False
    )

    # Now has the fixture
    assert len(registry) == 1
    assert registry.has_fixture("my_fixture")

    # Get metadata
    meta = registry.get_meta("my_fixture")
    assert meta is not None
    assert meta.name == "my_fixture"
    assert meta.scope == FixtureScope.Function
    assert not meta.autouse
    assert meta.dependencies == []
    assert not meta.has_teardown


def test_fixture_dependency_resolution():
    """Test fixture dependency resolution"""
    registry = FixtureRegistry()

    # Register fixtures with dependencies
    registry.register("fixture_a", FixtureScope.Function, False, [], False)
    registry.register("fixture_b", FixtureScope.Function, False, ["fixture_a"], False)
    registry.register("fixture_c", FixtureScope.Function, False, ["fixture_a", "fixture_b"], False)

    # Resolve order for fixture_c
    order = registry.resolve_order(["fixture_c"])

    # fixture_a should come before fixture_b and fixture_c
    assert order.index("fixture_a") < order.index("fixture_b")
    assert order.index("fixture_a") < order.index("fixture_c")
    assert order.index("fixture_b") < order.index("fixture_c")


def test_circular_dependency_detection():
    """Test circular dependency detection"""
    registry = FixtureRegistry()

    # Create circular dependency: a -> b -> c -> a
    registry.register("fixture_a", FixtureScope.Function, False, ["fixture_c"], False)
    registry.register("fixture_b", FixtureScope.Function, False, ["fixture_a"], False)
    registry.register("fixture_c", FixtureScope.Function, False, ["fixture_b"], False)

    # Should detect circular dependency
    expect(lambda: registry.detect_circular_deps()).to_raise(ValueError)


def test_autouse_fixtures():
    """Test autouse fixture filtering"""
    registry = FixtureRegistry()

    registry.register("auto_fixture", FixtureScope.Class, True, [], False)
    registry.register("manual_fixture", FixtureScope.Class, False, [], False)

    autouse = registry.get_autouse_fixtures(FixtureScope.Class)
    assert len(autouse) == 1
    assert autouse[0] == "auto_fixture"


def test_fixture_decorator():
    """Test the @fixture decorator"""

    @fixture(scope="function", autouse=False)
    def my_fixture():
        return "fixture_value"

    # Check that fixture metadata is attached
    assert hasattr(my_fixture, '_fixture_meta')
    assert my_fixture._fixture_meta['scope'] == "function"
    assert my_fixture._fixture_meta['autouse'] is False
    assert my_fixture._fixture_meta['name'] == "my_fixture"


def test_fixture_decorator_defaults():
    """Test @fixture decorator with defaults"""

    @fixture
    def my_fixture():
        return "fixture_value"

    # Check defaults
    assert hasattr(my_fixture, '_fixture_meta')
    assert my_fixture._fixture_meta['scope'] == "function"
    assert my_fixture._fixture_meta['autouse'] is False


def test_fixture_decorator_with_parentheses():
    """Test @fixture() decorator with parentheses"""

    @fixture()
    def my_fixture():
        return "fixture_value"

    # Check defaults
    assert hasattr(my_fixture, '_fixture_meta')
    assert my_fixture._fixture_meta['scope'] == "function"


def test_fixture_meta_repr():
    """Test FixtureMeta repr"""
    registry = FixtureRegistry()
    registry.register("test_fixture", FixtureScope.Class, True, ["dep1", "dep2"], True)

    meta = registry.get_meta("test_fixture")
    repr_str = repr(meta)

    assert "test_fixture" in repr_str
    assert "class" in repr_str.lower()
    assert "autouse=True" in repr_str or "autouse=true" in repr_str


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
