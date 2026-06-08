"""
Test lifecycle hooks system (setup_class, teardown_class, setup_method, teardown_method).
"""
import asyncio
from cclab.probe import TestSuite, test, expect


class TestHooksExecution(TestSuite):
    """Test that hooks execute in the correct order"""

    def __init__(self):
        super().__init__()
        # Track execution order
        self.execution_log = []

    async def setup_class(self):
        """Run once before all tests in the class"""
        self.execution_log.append("setup_class")
        self.class_data = "initialized"

    async def teardown_class(self):
        """Run once after all tests in the class"""
        self.execution_log.append("teardown_class")

    async def setup_method(self):
        """Run before each test method"""
        self.execution_log.append("setup_method")
        self.test_data = []

    async def teardown_method(self):
        """Run after each test method"""
        self.execution_log.append("teardown_method")

    @test
    async def test_first(self):
        """First test"""
        self.execution_log.append("test_first")
        expect(self.class_data).to_equal("initialized")
        expect(self.test_data).to_equal([])
        self.test_data.append("modified")

    @test
    async def test_second(self):
        """Second test"""
        self.execution_log.append("test_second")
        expect(self.class_data).to_equal("initialized")
        # test_data should be fresh from setup_method
        expect(self.test_data).to_equal([])


class TestSyncHooks(TestSuite):
    """Test synchronous hooks"""

    def __init__(self):
        super().__init__()
        self.sync_log = []

    def setup_class(self):  # Not async
        """Synchronous setup_class"""
        self.sync_log.append("sync_setup_class")
        self.sync_data = "sync"

    def teardown_class(self):  # Not async
        """Synchronous teardown_class"""
        self.sync_log.append("sync_teardown_class")

    def setup_method(self):  # Not async
        """Synchronous setup_method"""
        self.sync_log.append("sync_setup_method")

    def teardown_method(self):  # Not async
        """Synchronous teardown_method"""
        self.sync_log.append("sync_teardown_method")

    @test
    async def test_sync_hooks(self):
        """Test that sync hooks work"""
        self.sync_log.append("test_sync_hooks")
        expect(self.sync_data).to_equal("sync")


class TestHookErrors(TestSuite):
    """Test error handling in hooks"""

    def __init__(self):
        super().__init__()
        self.error_log = []

    async def setup_method(self):
        """Setup that always succeeds"""
        self.error_log.append("setup_method_ok")

    async def teardown_method(self):
        """Teardown that might fail"""
        self.error_log.append("teardown_method")
        # This should not prevent other teardowns from running

    @test
    async def test_will_pass(self):
        """Test that passes"""
        self.error_log.append("test_will_pass")
        expect(True).to_be_true()


class TestMixedSyncAsync(TestSuite):
    """Test mixing sync and async hooks"""

    def __init__(self):
        super().__init__()
        self.mixed_log = []

    def setup_class(self):  # Sync
        """Sync setup_class"""
        self.mixed_log.append("sync_setup_class")

    async def teardown_class(self):  # Async
        """Async teardown_class"""
        await asyncio.sleep(0.001)  # Simulate async work
        self.mixed_log.append("async_teardown_class")

    async def setup_method(self):  # Async
        """Async setup_method"""
        await asyncio.sleep(0.001)
        self.mixed_log.append("async_setup_method")

    def teardown_method(self):  # Sync
        """Sync teardown_method"""
        self.mixed_log.append("sync_teardown_method")

    @test
    async def test_mixed(self):
        """Test with mixed hooks"""
        self.mixed_log.append("test_mixed")
        expect(len(self.mixed_log)).to_equal(3)  # setup_class, setup_method, test


async def main():
    """Run all hook tests"""
    print("\n=== Testing Hooks Execution Order ===")
    suite1 = TestHooksExecution()
    report1 = await suite1.run(verbose=True)

    # Verify execution order
    expected_order = [
        "setup_class",
        "setup_method",
        "test_first",
        "teardown_method",
        "setup_method",
        "test_second",
        "teardown_method",
        "teardown_class",
    ]

    print("\nExecution log:", suite1.execution_log)
    print("Expected order:", expected_order)

    assert suite1.execution_log == expected_order, f"Unexpected execution order: {suite1.execution_log}"
    assert report1.summary.passed == 2, f"Expected 2 passed, got {report1.summary.passed}"

    print("\n=== Testing Sync Hooks ===")
    suite2 = TestSyncHooks()
    report2 = await suite2.run(verbose=True)

    expected_sync_order = [
        "sync_setup_class",
        "sync_setup_method",
        "test_sync_hooks",
        "sync_teardown_method",
        "sync_teardown_class",
    ]

    print("\nSync execution log:", suite2.sync_log)
    print("Expected sync order:", expected_sync_order)

    assert suite2.sync_log == expected_sync_order, f"Unexpected sync execution order: {suite2.sync_log}"
    assert report2.summary.passed == 1, f"Expected 1 passed, got {report2.summary.passed}"

    print("\n=== Testing Hook Errors ===")
    suite3 = TestHookErrors()
    report3 = await suite3.run(verbose=True)

    print("\nError handling log:", suite3.error_log)
    assert report3.summary.passed == 1, f"Expected 1 passed, got {report3.summary.passed}"

    print("\n=== Testing Mixed Sync/Async Hooks ===")
    suite4 = TestMixedSyncAsync()
    report4 = await suite4.run(verbose=True)

    print("\nMixed execution log:", suite4.mixed_log)
    assert report4.summary.passed == 1, f"Expected 1 passed, got {report4.summary.passed}"

    print("\n=== ALL HOOK TESTS PASSED ===")


if __name__ == "__main__":
    asyncio.run(main())
