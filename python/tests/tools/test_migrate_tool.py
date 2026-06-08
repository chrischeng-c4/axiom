"""Tests for pytest-to-data-bridge-test migration tool."""

import ast
import tempfile
from pathlib import Path

from cclab.probe import TestSuite, test, expect


# Import the migration tool
import sys
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "tools"))
from migrate_to_cclab_test import PytestToDataBridgeTransformer, transform_file


class TestPytestToDataBridgeTransformer(TestSuite):
    """Test the AST transformer."""

    @test
    def test_converts_import_pytest(self):
        """Test conversion of 'import pytest'."""
        source = "import pytest"
        tree = ast.parse(source)
        transformer = PytestToDataBridgeTransformer()
        new_tree = transformer.visit(tree)

        # Should have modified
        expect(transformer.modified).to_equal(True)
        expect(transformer.has_pytest_import).to_equal(True)

        # Should contain cclab.probe import
        code = ast.unparse(new_tree)
        expect("from cclab.probe import" in code).to_equal(True)
        expect("TestSuite" in code).to_equal(True)
        expect("expect" in code).to_equal(True)

    @test
    def test_converts_assert_equal(self):
        """Test conversion of assert x == y."""
        source = "assert x == y"
        tree = ast.parse(source)
        transformer = PytestToDataBridgeTransformer()
        new_tree = transformer.visit(tree)

        code = ast.unparse(new_tree)
        expect("expect" in code).to_equal(True)
        expect("to_equal" in code).to_equal(True)

    @test
    def test_converts_assert_not_equal(self):
        """Test conversion of assert x != y."""
        source = "assert x != y"
        tree = ast.parse(source)
        transformer = PytestToDataBridgeTransformer()
        new_tree = transformer.visit(tree)

        code = ast.unparse(new_tree)
        expect("expect" in code).to_equal(True)
        expect("to_not_equal" in code).to_equal(True)

    @test
    def test_converts_assert_greater_than(self):
        """Test conversion of assert x > y."""
        source = "assert x > y"
        tree = ast.parse(source)
        transformer = PytestToDataBridgeTransformer()
        new_tree = transformer.visit(tree)

        code = ast.unparse(new_tree)
        expect("expect" in code).to_equal(True)
        expect("to_be_greater_than" in code).to_equal(True)

    @test
    def test_converts_assert_in(self):
        """Test conversion of assert x in y."""
        source = "assert x in items"
        tree = ast.parse(source)
        transformer = PytestToDataBridgeTransformer()
        new_tree = transformer.visit(tree)

        code = ast.unparse(new_tree)
        expect("expect" in code).to_equal(True)
        expect("to_contain" in code).to_equal(True)

    @test
    def test_converts_assert_is_none(self):
        """Test conversion of assert x is None."""
        source = "assert x is None"
        tree = ast.parse(source)
        transformer = PytestToDataBridgeTransformer()
        new_tree = transformer.visit(tree)

        code = ast.unparse(new_tree)
        expect("expect" in code).to_equal(True)
        expect("to_be_none" in code).to_equal(True)

    @test
    def test_converts_assert_is_not_none(self):
        """Test conversion of assert x is not None."""
        source = "assert x is not None"
        tree = ast.parse(source)
        transformer = PytestToDataBridgeTransformer()
        new_tree = transformer.visit(tree)

        code = ast.unparse(new_tree)
        expect("expect" in code).to_equal(True)
        expect("to_not_be_none" in code).to_equal(True)

    @test
    def test_converts_pytest_fixture(self):
        """Test conversion of @pytest.fixture."""
        source = """
@pytest.fixture(scope="class")
def my_fixture():
    return "value"
"""
        tree = ast.parse(source)
        transformer = PytestToDataBridgeTransformer()
        new_tree = transformer.visit(tree)

        code = ast.unparse(new_tree)
        expect("@fixture" in code).to_equal(True)
        expect("pytest.fixture" in code).to_equal(False)

    @test
    def test_removes_pytest_mark_asyncio(self):
        """Test removal of @pytest.mark.asyncio."""
        source = """
@pytest.mark.asyncio
async def test_async():
    pass
"""
        tree = ast.parse(source)
        transformer = PytestToDataBridgeTransformer()
        new_tree = transformer.visit(tree)

        code = ast.unparse(new_tree)
        expect("pytest.mark.asyncio" in code).to_equal(False)

    @test
    def test_converts_parametrize(self):
        """Test conversion of @pytest.mark.parametrize."""
        source = """
@pytest.mark.parametrize("x", [1, 2, 3])
def test_param(x):
    assert x > 0
"""
        tree = ast.parse(source)
        transformer = PytestToDataBridgeTransformer()
        new_tree = transformer.visit(tree)

        code = ast.unparse(new_tree)
        expect("@parametrize" in code).to_equal(True)
        expect("pytest.mark.parametrize" in code).to_equal(False)

    @test
    def test_adds_testsuite_base_class(self):
        """Test adding TestSuite base class to test classes."""
        source = """
class TestExample:
    def test_something(self):
        assert True
"""
        tree = ast.parse(source)
        transformer = PytestToDataBridgeTransformer()
        new_tree = transformer.visit(tree)

        code = ast.unparse(new_tree)
        expect("class TestExample(TestSuite)" in code).to_equal(True)

    @test
    def test_adds_test_decorator(self):
        """Test adding @test decorator to test functions."""
        source = """
class TestExample:
    def test_something(self):
        assert True
"""
        tree = ast.parse(source)
        transformer = PytestToDataBridgeTransformer()
        new_tree = transformer.visit(tree)

        code = ast.unparse(new_tree)
        # Should have @test decorator
        expect("@test" in code).to_equal(True)


class TestFileTransformation(TestSuite):
    """Test full file transformation."""

    @test
    def test_transform_simple_file(self):
        """Test transforming a simple pytest file."""
        # Create temporary file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
            f.write("""
import pytest

class TestExample:
    def test_basic(self):
        assert 1 + 1 == 2
""")
            temp_path = Path(f.name)

        try:
            # Transform file
            success, warnings = transform_file(temp_path, dry_run=False)

            # Should succeed
            expect(success).to_equal(True)

            # Read transformed content
            with open(temp_path, 'r') as f:
                content = f.read()

            # Verify transformations
            expect("from cclab.probe import" in content).to_equal(True)
            expect("class TestExample(TestSuite)" in content).to_equal(True)
            expect("@test" in content).to_equal(True)
            expect("expect" in content).to_equal(True)

        finally:
            # Clean up
            temp_path.unlink()

    @test
    def test_dry_run_does_not_modify(self):
        """Test dry-run mode doesn't modify files."""
        # Create temporary file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
            original_content = """
import pytest

def test_example():
    assert True
"""
            f.write(original_content)
            temp_path = Path(f.name)

        try:
            # Transform file with dry-run
            success, warnings = transform_file(temp_path, dry_run=True)

            # Should succeed
            expect(success).to_equal(True)

            # File should be unchanged
            with open(temp_path, 'r') as f:
                content = f.read()

            expect(content).to_equal(original_content)

        finally:
            # Clean up
            temp_path.unlink()


# Run tests if executed directly
if __name__ == '__main__':
    from cclab.probe import run_suite
    report = run_suite(TestPytestToDataBridgeTransformer, verbose=True)
    report2 = run_suite(TestFileTransformation, verbose=True)

    # Exit with error code if tests failed
    import sys
    total_failed = report.summary.failed + report2.summary.failed
    sys.exit(0 if total_failed == 0 else 1)
