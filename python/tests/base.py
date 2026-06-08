"""
Base test suites for data-bridge tests.

This module provides base TestSuite classes that handle common setup/teardown
for different test categories.
"""

import os
from cclab.probe import TestSuite

# Shared database URI - use environment variable or default
MONGODB_URI = os.environ.get(
    "MONGODB_URI",
    "mongodb://testuser:testpass@localhost:27017/data-bridge-test?authSource=admin"
)


class MongoTestSuite(TestSuite):
    """
    Base test suite for MongoDB integration tests.

    Handles database connection setup/teardown at suite level.

    Example:
        class TestUserCRUD(MongoTestSuite):
            @test()
            async def test_create_user(self):
                user = await User(name="test").save()
                expect(user.id).to_be_true()
    """

    async def setup_suite(self):
        """Initialize MongoDB connection for the test suite."""
        from cclab import init, is_connected

        if not is_connected():
            await init(MONGODB_URI)

    async def teardown_suite(self):
        """Close MongoDB connection after all tests."""
        from cclab import close
        await close()


class CommonTestSuite(TestSuite):
    """
    Base test suite for common/unit tests.

    No database connection - for pure Python/Rust logic tests.

    Example:
        class TestConstraints(CommonTestSuite):
            @test()
            async def test_min_len_constraint(self):
                constraint = MinLen(3)
                expect(constraint.validate("abc")).to_be_true()
    """
    pass


class HttpTestSuite(TestSuite):
    """
    Base test suite for HTTP client tests.

    Example:
        class TestHttpClient(HttpTestSuite):
            async def setup_suite(self):
                from cclab.photon import HttpClient
                self.client = HttpClient(base_url="https://httpbin.org")

            @test()
            async def test_get_request(self):
                response = await self.client.get("/get")
                expect(response.status_code).to_equal(200)
    """
    pass
