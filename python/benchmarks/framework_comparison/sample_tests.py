"""
Sample test suite compatible with both pytest and data-bridge-test.

This file contains simple tests that can be run with both frameworks
for performance comparison.
"""

# Simple mathematical function for testing
def add(a: int, b: int) -> int:
    """Add two numbers."""
    return a + b


def multiply(x: int, y: int) -> int:
    """Multiply two numbers."""
    return x * y


def factorial(n: int) -> int:
    """Calculate factorial."""
    if n <= 1:
        return 1
    return n * factorial(n - 1)


# Test data for parametrization
SAMPLE_DATA = [
    (1, 2, 3),
    (5, 10, 15),
    (100, 200, 300),
    (-5, 10, 5),
    (0, 0, 0),
]

MULTIPLY_DATA = [
    (2, 3, 6),
    (4, 5, 20),
    (10, 10, 100),
]

FACTORIAL_DATA = [
    (0, 1),
    (1, 1),
    (5, 120),
    (10, 3628800),
]


# These tests are designed to be simple and fast
# so we can measure framework overhead accurately

def test_simple_addition():
    """Simple addition test."""
    result = add(1, 2)
    assert result == 3


def test_simple_multiplication():
    """Simple multiplication test."""
    result = multiply(3, 4)
    assert result == 12


def test_factorial_calculation():
    """Test factorial calculation."""
    result = factorial(5)
    assert result == 120


def test_string_operations():
    """Test basic string operations."""
    s = "hello world"
    assert s.upper() == "HELLO WORLD"
    assert s.split() == ["hello", "world"]
    assert len(s) == 11


def test_list_operations():
    """Test basic list operations."""
    lst = [1, 2, 3, 4, 5]
    assert len(lst) == 5
    assert sum(lst) == 15
    assert max(lst) == 5


def test_dict_operations():
    """Test basic dict operations."""
    d = {"a": 1, "b": 2, "c": 3}
    assert len(d) == 3
    assert d["a"] == 1
    assert list(d.keys()) == ["a", "b", "c"]


def test_boolean_logic():
    """Test boolean operations."""
    assert True is True
    assert False is False
    assert not False
    assert True and True
    assert True or False


def test_comparisons():
    """Test comparison operations."""
    assert 5 > 3
    assert 10 >= 10
    assert 2 < 5
    assert 3 <= 3
    assert 4 == 4
    assert 5 != 6


def test_type_checks():
    """Test type checking."""
    assert isinstance(42, int)
    assert isinstance("hello", str)
    assert isinstance([1, 2, 3], list)
    assert isinstance({"a": 1}, dict)


def test_exception_handling():
    """Test exception handling."""
    try:
        _ = 1 / 0
        assert False, "Should have raised ZeroDivisionError"
    except ZeroDivisionError:
        assert True


# Fixtures for testing (compatible with pytest)
class TestWithFixtures:
    """Test class that uses fixtures."""

    def setup_method(self):
        """Setup method (pytest-style)."""
        self.data = [1, 2, 3, 4, 5]
        self.config = {"timeout": 5, "retries": 3}

    def teardown_method(self):
        """Teardown method (pytest-style)."""
        self.data = None
        self.config = None

    def test_with_setup_data(self):
        """Test using setup data."""
        assert len(self.data) == 5
        assert sum(self.data) == 15

    def test_with_config(self):
        """Test using config data."""
        assert self.config["timeout"] == 5
        assert self.config["retries"] == 3
