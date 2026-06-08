"""
Tests for connection string building and initialization.

Tests for:
- _build_connection_string function variants
- Connection initialization
- Connection status checks

Migrated from test_coverage.py and focused for maintainability.
"""
from cclab.nebula.connection import _build_connection_string
from cclab.probe import test, expect
from tests.base import CommonTestSuite


class TestConnectionStringBuilding(CommonTestSuite):
    """Test _build_connection_string function."""

    @test(tags=["unit", "connection"])
    async def test_build_basic(self):
        """Test basic connection string building."""
        result = _build_connection_string(
            host="localhost",
            port=27017,
            database="mydb"
        )
        expect(result).to_equal("mongodb://localhost:27017/mydb")

    @test(tags=["unit", "connection"])
    async def test_build_with_username_password(self):
        """Test connection string with credentials."""
        result = _build_connection_string(
            host="localhost",
            port=27017,
            database="mydb",
            username="user",
            password="pass"
        )
        expect(result).to_equal("mongodb://user:pass@localhost:27017/mydb")

    @test(tags=["unit", "connection"])
    async def test_build_with_username_only(self):
        """Test connection string with username only."""
        result = _build_connection_string(
            host="localhost",
            port=27017,
            database="mydb",
            username="user"
        )
        expect(result).to_equal("mongodb://user@localhost:27017/mydb")

    @test(tags=["unit", "connection"])
    async def test_build_with_special_chars(self):
        """Test connection string with special characters in password."""
        result = _build_connection_string(
            host="localhost",
            port=27017,
            database="mydb",
            username="user",
            password="p@ss:word/test"
        )
        # Special chars should be URL encoded
        expect("user:" in result).to_be_true()
        expect("@localhost" in result).to_be_true()
        # At least one special char should be encoded
        has_encoded = "%40" in result or "%3A" in result or "%2F" in result
        expect(has_encoded).to_be_true()

    @test(tags=["unit", "connection"])
    async def test_build_with_auth_source(self):
        """Test connection string with authSource."""
        result = _build_connection_string(
            host="localhost",
            port=27017,
            database="mydb",
            auth_source="admin"
        )
        expect(result).to_equal("mongodb://localhost:27017/mydb?authSource=admin")

    @test(tags=["unit", "connection"])
    async def test_build_with_replica_set(self):
        """Test connection string with replica set."""
        result = _build_connection_string(
            host="localhost",
            port=27017,
            database="mydb",
            replica_set="rs0"
        )
        expect(result).to_equal("mongodb://localhost:27017/mydb?replicaSet=rs0")

    @test(tags=["unit", "connection"])
    async def test_build_with_multiple_options(self):
        """Test connection string with multiple options."""
        result = _build_connection_string(
            host="localhost",
            port=27017,
            database="mydb",
            auth_source="admin",
            replica_set="rs0",
            readPreference="secondary"
        )
        expect("authSource=admin" in result).to_be_true()
        expect("replicaSet=rs0" in result).to_be_true()
        expect("readPreference=secondary" in result).to_be_true()

    @test(tags=["unit", "connection"])
    async def test_build_full_connection(self):
        """Test full connection string with all options."""
        result = _build_connection_string(
            host="mongo.example.com",
            port=27018,
            database="production",
            username="admin",
            password="secret123",
            auth_source="admin",
            replica_set="prod-rs"
        )
        expect("admin:secret123@" in result).to_be_true()
        expect("mongo.example.com:27018" in result).to_be_true()
        expect("/production?" in result).to_be_true()
        expect("authSource=admin" in result).to_be_true()
        expect("replicaSet=prod-rs" in result).to_be_true()


class TestConnectionInit(CommonTestSuite):
    """Test init function variants."""

    @test(tags=["unit", "connection"])
    async def test_init_without_params_raises(self):
        """Test init raises without connection_string or database."""
        from cclab.nebula.connection import init

        error_caught = False
        try:
            await init()
        except ValueError as e:
            error_caught = True
            expect("Either connection_string or database" in str(e)).to_be_true()

        expect(error_caught).to_be_true()

    @test(tags=["unit", "connection"])
    async def test_is_connected_returns_bool(self):
        """Test is_connected returns boolean."""
        from cclab.nebula.connection import is_connected

        result = is_connected()
        expect(isinstance(result, bool)).to_be_true()


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestConnectionStringBuilding,
        TestConnectionInit,
    ], verbose=True)
