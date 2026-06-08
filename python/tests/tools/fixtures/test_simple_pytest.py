"""Simple pytest test file for testing migration."""
from cclab.probe import TestSuite, test, fixture, expect, parametrize

class TestSimple(TestSuite):
from cclab.probe import expect
    """Simple test class."""
    @test
    def test_basic_assertion(self):
        """Test basic assertions."""
        expect(1 + 1).to_equal(2)
        expect('hello').to_not_equal('world')
        expect(5).to_be_greater_than(3)
        expect(2).to_be_less_than(10)

    @test
    def test_boolean_assertions(self):
        """Test boolean assertions."""
        value = True
        expect(value).to_be_truthy()
        expect(False).to_be_falsy()

    @test
    def test_none_assertions(self):
        """Test None assertions."""
        value = None
        expect(value).to_be_none()
        other = 'not none'
        expect(other).to_not_be_none()

    @test
    def test_membership_assertions(self):
        """Test membership assertions."""
        items = [1, 2, 3, 4, 5]
        expect(items).to_contain(3)
        expect(items).to_not_contain(10)

class TestWithFixtures(TestSuite):
    """Test class with fixtures."""

    @fixture(scope='class')
    def class_fixture(self):
        """Class-level fixture."""
        return {'data': 'test'}

    @fixture
    def function_fixture(self):
        """Function-level fixture."""
        return [1, 2, 3]

    @test
    def test_with_fixture(self, class_fixture, function_fixture):
        """Test using fixtures."""
        expect(class_fixture['data']).to_equal('test')
        expect(len(function_fixture)).to_equal(3)

class TestParametrized(TestSuite):
    """Test class with parametrized tests."""

    @test
    @parametrize('value', [1, 2, 3, 4, 5])
    def test_parametrize_single(self, value):
        """Test with single parameter."""
        expect(value).to_be_greater_than(0)

    @test
    @parametrize('x', [1, 2])
    @parametrize('y', [3, 4])
    def test_parametrize_multiple(self, x, y):
        """Test with multiple parameters (cartesian product)."""
        expect(x + y).to_be_greater_than(0)

class TestAsync(TestSuite):
    """Test async functions."""

    @test
    async def test_async_function(self):
        """Test async function."""
        result = await self.async_operation()
        expect(result).to_equal('done')

    async def async_operation(self):
        """Async operation."""
        return 'done'

class TestExceptions(TestSuite):
    """Test exception handling."""

    @test
    def test_raises_exception(self):
        """Test exception is raised."""
        expect(lambda: raise ValueError('test error')).to_raise(ValueError)

    @test
    def test_raises_with_message(self):
        """Test exception with specific message."""
        expect(lambda: int('not a number' * 100)).to_raise(TypeError)