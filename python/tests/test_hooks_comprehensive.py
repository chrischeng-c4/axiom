"""
Comprehensive tests for the hooks system including error handling and edge cases.
"""
import asyncio
import pytest
from cclab.probe import TestSuite, test, expect


class TestHookExecutionOrder(TestSuite):
    """Verify hooks execute in pytest-compatible order"""

    def __init__(self):
        super().__init__()
        self.execution_order = []

    async def setup_class(self):
        self.execution_order.append("1-setup_class")
        self.shared_counter = 0

    async def teardown_class(self):
        self.execution_order.append("8-teardown_class")

    async def setup_method(self):
        self.execution_order.append(f"{len(self.execution_order)+1}-setup_method")
        self.method_data = []

    async def teardown_method(self):
        self.execution_order.append(f"{len(self.execution_order)+1}-teardown_method")

    @test
    async def test_one(self):
        self.execution_order.append(f"{len(self.execution_order)+1}-test_one")
        self.shared_counter += 1
        expect(self.shared_counter).to_equal(1)

    @test
    async def test_two(self):
        self.execution_order.append(f"{len(self.execution_order)+1}-test_two")
        self.shared_counter += 1
        expect(self.shared_counter).to_equal(2)


class TestHookErrorHandling(TestSuite):
    """Test that teardown hooks run even when tests fail"""

    def __init__(self):
        super().__init__()
        self.teardown_ran = False
        self.class_teardown_ran = False

    async def teardown_method(self):
        self.teardown_ran = True

    async def teardown_class(self):
        self.class_teardown_ran = True

    @test
    async def test_that_fails(self):
        # Teardown should still run
        expect(1).to_equal(2)


class TestSetupMethodFails(TestSuite):
    """Test that test is skipped when setup_method fails"""

    def __init__(self):
        super().__init__()
        self.test_ran = False
        self.teardown_ran = False

    async def setup_method(self):
        raise RuntimeError("Setup failed")

    async def teardown_method(self):
        self.teardown_ran = True

    @test
    async def test_should_not_run(self):
        self.test_ran = True


class TestSetupClassFails(TestSuite):
    """Test that all tests are skipped when setup_class fails"""

    def __init__(self):
        super().__init__()
        self.any_test_ran = False

    async def setup_class(self):
        raise RuntimeError("Class setup failed")

    @test
    async def test_one(self):
        self.any_test_ran = True

    @test
    async def test_two(self):
        self.any_test_ran = True


class TestHookDataIsolation(TestSuite):
    """Test that method hooks provide proper data isolation"""

    async def setup_method(self):
        self.test_list = []

    @test
    async def test_first_isolation(self):
        self.test_list.append("first")
        expect(len(self.test_list)).to_equal(1)

    @test
    async def test_second_isolation(self):
        # List should be fresh from setup_method
        self.test_list.append("second")
        expect(len(self.test_list)).to_equal(1)


class TestLegacyHooksStillWork(TestSuite):
    """Test that legacy setup_suite/teardown_suite still work"""

    def __init__(self):
        super().__init__()
        self.legacy_log = []

    async def setup_suite(self):
        self.legacy_log.append("setup_suite")

    async def teardown_suite(self):
        self.legacy_log.append("teardown_suite")

    async def setup(self):
        self.legacy_log.append("setup")

    async def teardown(self):
        self.legacy_log.append("teardown")

    @test
    async def test_legacy(self):
        self.legacy_log.append("test")
        expect(len(self.legacy_log)).to_equal(3)  # setup_suite, setup, test


@pytest.mark.asyncio
async def test_hook_execution_order():
    """Test hook execution order matches pytest"""
    suite = TestHookExecutionOrder()
    report = await suite.run()

    # Both tests should pass
    assert report.summary.passed == 2
    assert report.summary.failed == 0

    # Verify execution order
    expected = [
        "1-setup_class",
        "2-setup_method",
        "3-test_one",
        "4-teardown_method",
        "5-setup_method",
        "6-test_two",
        "7-teardown_method",
        "8-teardown_class",
    ]

    assert suite.execution_order == expected, f"Unexpected order: {suite.execution_order}"


@pytest.mark.asyncio
async def test_teardown_runs_on_failure():
    """Test that teardown hooks run even when test fails"""
    suite = TestHookErrorHandling()
    report = await suite.run()

    # Test should fail
    assert report.summary.failed == 1

    # But teardowns should have run
    assert suite.teardown_ran is True
    assert suite.class_teardown_ran is True


@pytest.mark.asyncio
async def test_setup_method_failure():
    """Test handling of setup_method failures"""
    suite = TestSetupMethodFails()
    report = await suite.run()

    # Test should be marked as error
    assert report.summary.errors == 1
    assert report.summary.passed == 0

    # Test should not have run
    assert suite.test_ran is False

    # Teardown might not run if setup failed
    # This behavior matches pytest


@pytest.mark.asyncio
async def test_setup_class_failure():
    """Test handling of setup_class failures"""
    suite = TestSetupClassFails()
    report = await suite.run()

    # All tests should be marked as errors
    assert report.summary.errors == 2
    assert report.summary.passed == 0

    # No tests should have run
    assert suite.any_test_ran is False


@pytest.mark.asyncio
async def test_data_isolation():
    """Test that setup_method provides proper data isolation"""
    suite = TestHookDataIsolation()
    report = await suite.run()

    # Both tests should pass (proving isolation works)
    assert report.summary.passed == 2
    assert report.summary.failed == 0


@pytest.mark.asyncio
async def test_legacy_hooks():
    """Test that legacy hooks still work"""
    suite = TestLegacyHooksStillWork()
    report = await suite.run()

    assert report.summary.passed == 1

    expected_order = [
        "setup_suite",  # Legacy class-level setup
        "setup",  # Legacy method-level setup
        "test",
        "teardown",  # Legacy method-level teardown
        "teardown_suite",  # Legacy class-level teardown
    ]

    assert suite.legacy_log == expected_order


def test_hook_registry_basics():
    """Test HookRegistry basic functionality"""
    from cclab.probe import HookRegistry, HookType

    registry = HookRegistry()

    # Initially empty
    assert registry.hook_count(HookType.SetupClass) == 0
    assert registry.hook_count(HookType.TeardownClass) == 0

    # Register a hook
    def dummy_hook():
        pass

    registry.register_hook(HookType.SetupClass, dummy_hook)
    assert registry.hook_count(HookType.SetupClass) == 1

    # Clear hooks
    registry.clear_hooks(HookType.SetupClass)
    assert registry.hook_count(HookType.SetupClass) == 0

    # Register multiple hooks
    registry.register_hook(HookType.SetupMethod, dummy_hook)
    registry.register_hook(HookType.SetupMethod, dummy_hook)
    assert registry.hook_count(HookType.SetupMethod) == 2

    # Clear all
    registry.clear_all()
    assert registry.hook_count(HookType.SetupMethod) == 0


def test_hook_type_enum():
    """Test HookType enum"""
    from cclab.probe import HookType

    # Test string representation
    assert str(HookType.SetupClass) == "setup_class"
    assert str(HookType.TeardownClass) == "teardown_class"
    assert str(HookType.SetupMethod) == "setup_method"
    assert str(HookType.TeardownMethod) == "teardown_method"
    assert str(HookType.SetupModule) == "setup_module"
    assert str(HookType.TeardownModule) == "teardown_module"

    # Test equality
    assert HookType.SetupClass == HookType.SetupClass
    assert HookType.SetupClass != HookType.TeardownClass


if __name__ == "__main__":
    # Run with pytest
    pytest.main([__file__, "-v"])
