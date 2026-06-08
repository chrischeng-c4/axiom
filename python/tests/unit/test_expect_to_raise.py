"""
Unit tests for expect().to_raise() assertion method.

This tests the Rust-backed to_raise implementation in cclab.probe.
"""

from cclab.probe import TestSuite, test, expect


class TestToRaise(TestSuite):
    """Test suite for to_raise assertion."""

    @test
    def test_raises_expected_exception(self):
        """to_raise passes when the expected exception is raised."""
        def raises_value_error():
            raise ValueError("test error")

        expect(lambda: raises_value_error()).to_raise(ValueError)

    @test
    def test_raises_no_exception_fails(self):
        """to_raise fails when no exception is raised."""
        def no_error():
            return 42

        try:
            expect(lambda: no_error()).to_raise(ValueError)
            raise AssertionError("Should have failed")
        except AssertionError as e:
            expect(str(e)).to_contain("no exception was raised")

    @test
    def test_raises_wrong_exception_fails(self):
        """to_raise fails when wrong exception type is raised."""
        def raises_type_error():
            raise TypeError("wrong type")

        try:
            expect(lambda: raises_type_error()).to_raise(ValueError)
            raise AssertionError("Should have failed")
        except AssertionError as e:
            expect(str(e)).to_contain("TypeError")

    @test
    def test_raises_subclass_passes(self):
        """to_raise passes when a subclass of expected exception is raised."""
        def raises_runtime_error():
            raise RuntimeError("runtime")

        # RuntimeError is a subclass of Exception
        expect(lambda: raises_runtime_error()).to_raise(Exception)

    @test
    def test_not_to_raise_passes_when_no_exception(self):
        """not().to_raise passes when no exception is raised."""
        def no_error():
            return 42

        expect(lambda: no_error()).not_.to_raise(ValueError)

    @test
    def test_not_to_raise_fails_when_exception_raised(self):
        """not().to_raise fails when the exception IS raised."""
        def raises_value_error():
            raise ValueError("error")

        try:
            expect(lambda: raises_value_error()).not_.to_raise(ValueError)
            raise AssertionError("Should have failed")
        except AssertionError as e:
            expect(str(e)).to_contain("NOT to be raised")

    @test
    def test_to_raise_requires_callable(self):
        """to_raise fails with clear error when value is not callable."""
        try:
            expect(42).to_raise(ValueError)
            raise AssertionError("Should have failed")
        except TypeError as e:
            expect(str(e)).to_contain("callable")


if __name__ == "__main__":
    import asyncio
    asyncio.run(TestToRaise().run())
