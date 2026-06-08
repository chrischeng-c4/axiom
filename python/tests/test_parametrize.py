"""
Test parametrize functionality for data-bridge test framework.
"""

import asyncio
from cclab.probe import (
    TestSuite,
    test,
    parametrize,
    expect,
    run_suite,
)


class TestParametrizeBasic(TestSuite):
    """Test basic parametrize functionality"""

    @test
    @parametrize("value", [10, 100, 1000])
    async def test_single_parameter(self, value):
        """Test with single parameter"""
        expect(value).to_be_greater_than(0)
        # Check value is one of the expected values
        is_valid = value in [10, 100, 1000]
        expect(is_valid).to_be_true()

    @test
    @parametrize("x", [1, 2])
    @parametrize("y", [10, 20])
    async def test_cartesian_product(self, x, y):
        """Test with Cartesian product of parameters"""
        # Verify parameters are in expected ranges
        expect(x).to_be_less_than(3)
        expect(x).to_be_greater_than(0)
        expect(y).to_be_greater_than(5)
        expect(x * y).to_be_greater_than(0)


class TestParametrizeTypes(TestSuite):
    """Test parametrize with different types"""

    @test
    @parametrize("batch_size", [10, 100, 1000, 10000, 50000])
    async def test_integers(self, batch_size):
        """Test with integer parameters"""
        expect(isinstance(batch_size, int)).to_be_true()
        expect(batch_size).to_be_greater_than(0)

    @test
    @parametrize("ratio", [0.5, 1.0, 1.5, 2.0])
    async def test_floats(self, ratio):
        """Test with float parameters"""
        expect(isinstance(ratio, float)).to_be_true()
        expect(ratio).to_be_greater_than(0.0)

    @test
    @parametrize("flag", [True, False])
    async def test_booleans(self, flag):
        """Test with boolean parameters"""
        expect(isinstance(flag, bool)).to_be_true()

    @test
    @parametrize("method", ["GET", "POST", "PUT", "DELETE"])
    async def test_strings(self, method):
        """Test with string parameters"""
        expect(isinstance(method, str)).to_be_true()
        is_valid = method in ["GET", "POST", "PUT", "DELETE"]
        expect(is_valid).to_be_true()


class TestParametrizeMultiple(TestSuite):
    """Test parametrize with multiple parameters"""

    @test
    @parametrize("method", ["GET", "POST"])
    @parametrize("auth", [True, False])
    @parametrize("status", [200, 404])
    async def test_three_parameters(self, method, auth, status):
        """Test with three parameters (2 × 2 × 2 = 8 instances)"""
        is_method_valid = method in ["GET", "POST"]
        expect(is_method_valid).to_be_true()
        expect(isinstance(auth, bool)).to_be_true()
        is_status_valid = status in [200, 404]
        expect(is_status_valid).to_be_true()


class TestParametrizeNaming(TestSuite):
    """Test that parametrized test names are formatted correctly"""

    def __init__(self):
        super().__init__()
        # Store test names for verification
        self.test_names = [t.instance_name if hasattr(t, 'instance_name') else t.func.__name__
                          for t in self._tests]

    @test
    @parametrize("size", [10, 100])
    async def test_naming(self, size):
        """Test name should include parameter"""
        # This test itself verifies the parameter is injected correctly
        is_valid = size in [10, 100]
        expect(is_valid).to_be_true()


def main():
    """Run all parametrize tests"""
    print("\n" + "=" * 60)
    print("Testing Parametrize Functionality")
    print("=" * 60)

    # Test basic parametrize
    print("\n1. Testing basic parametrize...")
    suite1 = TestParametrizeBasic()
    print(f"   Found {suite1.test_count} tests (expected: 7)")
    # 3 from test_single_parameter + 4 from test_cartesian_product = 7
    assert suite1.test_count == 7, f"Expected 7 tests, got {suite1.test_count}"
    report1 = run_suite(TestParametrizeBasic, verbose=True)
    assert report1.summary.all_passed(), "Basic parametrize tests failed"
    print("   ✓ All basic tests passed")

    # Test different types
    print("\n2. Testing different parameter types...")
    suite2 = TestParametrizeTypes()
    print(f"   Found {suite2.test_count} tests (expected: 15)")
    # 5 integers + 4 floats + 2 booleans + 4 strings = 15
    assert suite2.test_count == 15, f"Expected 15 tests, got {suite2.test_count}"
    report2 = run_suite(TestParametrizeTypes, verbose=True)
    assert report2.summary.all_passed(), "Type parametrize tests failed"
    print("   ✓ All type tests passed")

    # Test multiple parameters
    print("\n3. Testing multiple parameters (Cartesian product)...")
    suite3 = TestParametrizeMultiple()
    print(f"   Found {suite3.test_count} tests (expected: 8)")
    # 2 × 2 × 2 = 8
    assert suite3.test_count == 8, f"Expected 8 tests, got {suite3.test_count}"
    report3 = run_suite(TestParametrizeMultiple, verbose=True)
    assert report3.summary.all_passed(), "Multiple parameter tests failed"
    print("   ✓ All multiple parameter tests passed")

    # Test naming
    print("\n4. Testing parametrized test naming...")
    suite4 = TestParametrizeNaming()
    print(f"   Found {suite4.test_count} tests")
    test_names = suite4.test_names
    print(f"   Test names: {test_names}")
    # Should have names like test_naming[size=10], test_naming[size=100]
    assert any("size=10" in name for name in test_names), "Missing test_naming[size=10]"
    assert any("size=100" in name for name in test_names), "Missing test_naming[size=100]"
    report4 = run_suite(TestParametrizeNaming, verbose=True)
    assert report4.summary.all_passed(), "Naming tests failed"
    print("   ✓ Test naming verified")

    print("\n" + "=" * 60)
    print("All Parametrize Tests Passed!")
    print("=" * 60)
    print(f"Total tests run: {report1.summary.total + report2.summary.total + report3.summary.total + report4.summary.total}")
    print(f"All passed: {report1.summary.all_passed() and report2.summary.all_passed() and report3.summary.all_passed() and report4.summary.all_passed()}")


if __name__ == "__main__":
    main()
