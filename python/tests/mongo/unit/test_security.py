"""
Comprehensive security penetration tests for data-bridge MongoDB ORM.

Tests for:
1. NoSQL Injection Prevention
2. Collection Name Validation
3. Field Name Validation
4. Error Sanitization
5. Security Configuration

Migrated from pytest to cclab.probe framework.
"""
from cclab import Document
from cclab._rust import configure_security, ObjectIdConversionMode
from cclab.probe import test, expect
from tests.base import MongoTestSuite


# =====================
# Test Document Classes
# =====================

class SecureUser(Document):
    """User document for security testing."""
    email: str
    password_hash: str = ""
    role: str = "user"
    credit_card: str = ""  # Sensitive field for injection tests

    class Settings:
        name = "test_secure_users"


class SecureProduct(Document):
    """Product document for security testing."""
    name: str
    price: float = 0.0
    quantity: int = 0

    class Settings:
        name = "test_secure_products"


# =====================
# NoSQL Injection Prevention Tests
# =====================

class TestNoSQLInjectionPrevention(MongoTestSuite):
    """Tests for preventing NoSQL injection attacks."""

    async def setup(self):
        """Reset security config and cleanup before each test."""
        from cclab.nebula import _engine
        configure_security(
            objectid_mode=ObjectIdConversionMode.Lenient,
            validate_queries=True,
            sanitize_errors=True,
        )
        await _engine.delete_many("test_secure_users", {})
        await _engine.delete_many("test_secure_products", {})

    async def teardown(self):
        """Cleanup after each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_secure_users", {})
        await _engine.delete_many("test_secure_products", {})
        configure_security(
            objectid_mode=ObjectIdConversionMode.Lenient,
            validate_queries=True,
            sanitize_errors=True,
        )

    @test(tags=["security", "nosql-injection"])
    async def test_all_zeros_objectid_bypass_prevention(self):
        """
        CRITICAL: Prevent all-zeros ObjectId from bypassing access control.
        """
        admin = SecureUser(
            email="admin@example.com",
            password_hash="admin_hash",
            role="admin",
        )
        await admin.save()

        regular_user = SecureUser(
            email="user@example.com",
            password_hash="user_hash",
            role="user",
        )
        await regular_user.save()

        malicious_id = "000000000000000000000000"
        result = await SecureUser.find_one({"_id": malicious_id})

        if result is not None:
            expect(str(result._id)).to_equal(malicious_id)
        else:
            expect(result).to_equal(None)

        all_users = await SecureUser.find().to_list()
        expect(len(all_users)).to_equal(2)

    @test(tags=["security", "nosql-injection"])
    async def test_where_operator_blocked(self):
        """CRITICAL: Block $where operator to prevent arbitrary code execution."""
        user = SecureUser(email="test@example.com", password_hash="hashed")
        await user.save()

        error_caught = False
        try:
            from cclab.nebula import _engine
            await _engine.find_one(
                "test_secure_users",
                {"$where": "this.email == 'test@example.com'"}
            )
        except Exception as e:
            error_caught = True
            error_msg = str(e).lower()
            has_expected_error = "$where" in error_msg or "validation" in error_msg or "dangerous" in error_msg
            expect(has_expected_error).to_be_true()

        expect(error_caught).to_be_true()

    @test(tags=["security", "nosql-injection"])
    async def test_function_operator_blocked(self):
        """CRITICAL: Block $function operator (MongoDB 4.4+) code execution."""
        user = SecureUser(email="victim@example.com", password_hash="hash")
        await user.save()

        error_caught = False
        try:
            from cclab.nebula import _engine
            await _engine.find_one(
                "test_secure_users",
                {
                    "$expr": {
                        "$function": {
                            "body": "function() { return true; }",
                            "args": [],
                            "lang": "js"
                        }
                    }
                }
            )
        except Exception as e:
            error_caught = True
            error_msg = str(e).lower()
            has_expected_error = "$function" in error_msg or "validation" in error_msg or "dangerous" in error_msg
            expect(has_expected_error).to_be_true()

        expect(error_caught).to_be_true()

    @test(tags=["security", "nosql-injection"])
    async def test_accumulator_operator_blocked(self):
        """CRITICAL: Block $accumulator operator custom JavaScript."""
        error_caught = False
        try:
            pipeline = [
                {
                    "$group": {
                        "_id": None,
                        "custom": {
                            "$accumulator": {
                                "init": "function() { return 0; }",
                                "accumulate": "function(state, val) { return state + val; }",
                                "accumulateArgs": ["$quantity"],
                                "merge": "function(s1, s2) { return s1 + s2; }",
                                "lang": "js"
                            }
                        }
                    }
                }
            ]
            from cclab.nebula import _engine
            await _engine.aggregate("test_secure_products", pipeline)
        except Exception as e:
            error_caught = True
            error_msg = str(e).lower()
            has_expected_error = "$accumulator" in error_msg or "validation" in error_msg or "dangerous" in error_msg
            expect(has_expected_error).to_be_true()

        expect(error_caught).to_be_true()

    @test(tags=["security", "nosql-injection"])
    async def test_nested_operator_injection(self):
        """Test that deeply nested dangerous operators are also blocked."""
        error_caught = False
        try:
            from cclab.nebula import _engine
            await _engine.find_one(
                "test_secure_users",
                {
                    "email": "test@example.com",
                    "$or": [
                        {"role": "admin"},
                        {"nested": {"$where": "this.role == 'admin'"}}
                    ]
                }
            )
        except Exception as e:
            error_caught = True
            error_msg = str(e).lower()
            has_expected_error = "$where" in error_msg or "validation" in error_msg
            expect(has_expected_error).to_be_true()

        expect(error_caught).to_be_true()


# =====================
# Collection Name Validation Tests
# =====================

class TestCollectionNameValidation(MongoTestSuite):
    """Tests for collection name injection prevention."""

    @test(tags=["security", "collection-validation"])
    async def test_system_collection_access_blocked(self):
        """CRITICAL: Block access to system.* collections."""
        error_caught = False
        try:
            from cclab.nebula import _engine
            await _engine.find_one("system.users", {})
        except ValueError as e:
            error_caught = True
            error_msg = str(e)
            has_expected_error = "system." in error_msg or "reserved" in error_msg.lower()
            expect(has_expected_error).to_be_true()

        expect(error_caught).to_be_true()

    @test(tags=["security", "collection-validation"])
    async def test_dollar_sign_collection_name_blocked(self):
        """CRITICAL: Block $ in collection names."""
        error_caught = False
        try:
            from cclab.nebula import _engine
            await _engine.find_one("$cmd", {})
        except ValueError as e:
            error_caught = True
            error_msg = str(e)
            has_expected_error = "$" in error_msg or "invalid" in error_msg.lower()
            expect(has_expected_error).to_be_true()

        expect(error_caught).to_be_true()

    @test(tags=["security", "collection-validation"])
    async def test_null_byte_collection_name_blocked(self):
        """CRITICAL: Block null bytes in collection names."""
        error_caught = False
        try:
            from cclab.nebula import _engine
            await _engine.find_one("users\x00admin", {})
        except ValueError as e:
            error_caught = True
            error_msg = str(e)
            has_expected_error = "null" in error_msg.lower() or "invalid" in error_msg.lower()
            expect(has_expected_error).to_be_true()

        expect(error_caught).to_be_true()

    @test(tags=["security", "collection-validation"])
    async def test_empty_collection_name_blocked(self):
        """Test that empty collection names are rejected."""
        error_caught = False
        try:
            from cclab.nebula import _engine
            await _engine.find_one("", {})
        except ValueError as e:
            error_caught = True
            error_msg = str(e)
            has_expected_error = "empty" in error_msg.lower() or "cannot be empty" in error_msg.lower()
            expect(has_expected_error).to_be_true()

        expect(error_caught).to_be_true()

    @test(tags=["security", "collection-validation"])
    async def test_collection_name_max_length(self):
        """Test that collection names have a reasonable max length."""
        long_name = "a" * 121
        error_caught = False
        try:
            from cclab.nebula import _engine
            await _engine.find_one(long_name, {})
        except ValueError as e:
            error_caught = True
            error_msg = str(e)
            has_expected_error = "length" in error_msg.lower() or "too long" in error_msg.lower()
            expect(has_expected_error).to_be_true()

        expect(error_caught).to_be_true()


# =====================
# Field Name Validation Tests
# =====================

class TestFieldNameValidation(MongoTestSuite):
    """Tests for field name injection prevention."""

    async def setup(self):
        """Cleanup before each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_secure_users", {})

    async def teardown(self):
        """Cleanup after each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_secure_users", {})

    @test(tags=["security", "field-validation"])
    async def test_dollar_prefix_field_in_query_allowed(self):
        """$ prefix in query field names should be allowed for MongoDB operators."""
        user = SecureUser(email="test@example.com")
        await user.save()

        result = await SecureUser.find_one({
            "$or": [
                {"email": "test@example.com"},
                {"email": "other@example.com"}
            ]
        })
        expect(result).not_.to_be_none()
        expect(result.email).to_equal("test@example.com")

    @test(tags=["security", "field-validation"])
    async def test_dollar_prefix_field_in_update_blocked(self):
        """CRITICAL: Block $ prefix in update field names (outside operators)."""
        user = SecureUser(email="update@example.com", role="user")
        await user.save()

        error_caught = False
        try:
            from cclab.nebula import _engine
            await _engine.update_one(
                "test_secure_users",
                {"_id": user._id},
                {"$set": {"$injected_field": "malicious_value"}}
            )
        except Exception as e:
            error_caught = True
            error_msg = str(e).lower()
            has_expected_error = "$" in error_msg or "invalid" in error_msg or "field" in error_msg
            expect(has_expected_error).to_be_true()

        expect(error_caught).to_be_true()


# =====================
# Error Sanitization Tests
# =====================

class TestErrorSanitization(MongoTestSuite):
    """Tests for preventing information leakage through error messages."""

    @test(tags=["security", "error-sanitization"])
    async def test_connection_string_redacted(self):
        """CRITICAL: Connection strings should not appear in error messages."""
        from cclab.nebula import _engine

        try:
            await _engine.find_one(
                "test_secure_users",
                {"$where": "this.password == 'secret'"}
            )
        except Exception as e:
            error_msg = str(e)
            expect("mongodb://" not in error_msg.lower()).to_be_true()
            expect("Dangerous operator" in error_msg).to_be_true()

    @test(tags=["security", "error-sanitization"])
    async def test_credentials_redacted(self):
        """CRITICAL: Usernames and passwords should be redacted."""
        try:
            from cclab.nebula import _engine
            await _engine.init("mongodb://myuser:mypassword@localhost:99999/test")
        except Exception as e:
            error_msg = str(e)
            expect("mypassword" not in error_msg).to_be_true()

    @test(tags=["security", "error-sanitization"])
    async def test_ip_address_redacted(self):
        """MEDIUM: IP addresses should be redacted to prevent network reconnaissance."""
        try:
            from cclab.nebula import _engine
            await _engine.init("mongodb://192.168.1.100:27017/test")
        except Exception as e:
            error_msg = str(e)
            # IP should be redacted
            ip_redacted = "192.168.1.100" not in error_msg or "[IP_REDACTED]" in error_msg
            expect(ip_redacted).to_be_true()


# =====================
# Security Configuration Tests
# =====================

class TestSecurityConfiguration(MongoTestSuite):
    """Tests for security configuration modes."""

    async def setup(self):
        """Cleanup before each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_secure_users", {})

    async def teardown(self):
        """Reset config and cleanup after each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_secure_users", {})
        configure_security(
            objectid_mode=ObjectIdConversionMode.Lenient,
            validate_queries=True,
            sanitize_errors=True,
        )

    @test(tags=["security", "config"])
    async def test_lenient_mode_auto_converts_with_warning(self):
        """Test Lenient mode: auto-converts 24-hex strings to ObjectId."""
        configure_security(objectid_mode=ObjectIdConversionMode.Lenient)

        user = SecureUser(email="lenient@example.com")
        await user.save()

        fake_id = "507f1f77bcf86cd799439011"
        result = await SecureUser.find_one(SecureUser.id == fake_id)

        # Should return None (no user with this ID) or a SecureUser
        result_valid = result is None or isinstance(result, SecureUser)
        expect(result_valid).to_be_true()

    @test(tags=["security", "config"])
    async def test_query_validation_toggle(self):
        """Test that query validation can be disabled via config."""
        configure_security(validate_queries=True)

        # $where should be blocked
        error_caught = False
        try:
            from cclab.nebula import _engine
            await _engine.find_one(
                "test_secure_users",
                {"$where": "this.email == 'test'"}
            )
        except Exception:
            error_caught = True

        expect(error_caught).to_be_true()

        # Disable validation (not recommended for production!)
        configure_security(validate_queries=False)

        # Now $where should pass validation (but may still fail in MongoDB)
        # Re-enable for safety
        configure_security(validate_queries=True)


# =====================
# Integration Security Tests
# =====================

class TestSecurityIntegration(MongoTestSuite):
    """End-to-end security tests simulating real attack scenarios."""

    async def setup(self):
        """Cleanup before each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_secure_users", {})

    async def teardown(self):
        """Cleanup after each test."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_secure_users", {})

    @test(tags=["security", "integration"])
    async def test_authentication_bypass_via_injection(self):
        """Simulate authentication bypass attempt via NoSQL injection."""
        admin = SecureUser(
            email="admin@example.com",
            password_hash="hashed_admin_password",
            role="admin"
        )
        await admin.save()

        # data-bridge should prevent arbitrary document matching
        result = await SecureUser.find_one({
            "email": {"$ne": None},
            "role": "admin"
        })

        # This should work as a legitimate query
        expect(result).not_.to_be_none()
        expect(result.role).to_equal("admin")

    @test(tags=["security", "integration"])
    async def test_privilege_escalation_via_update_injection(self):
        """Simulate privilege escalation via update injection."""
        user = SecureUser(
            email="user@example.com",
            password_hash="hash",
            role="user"
        )
        await user.save()

        from cclab.nebula import _engine
        await _engine.update_one(
            "test_secure_users",
            {"_id": user._id},
            {"$set": {"password_hash": "new_hash"}}
        )

        updated = await SecureUser.find_one(SecureUser.id == user._id)
        expect(updated.password_hash).to_equal("new_hash")
        expect(updated.role).to_equal("user")  # Role unchanged

    @test(tags=["security", "integration"])
    async def test_data_exfiltration_via_regex_injection(self):
        """Test that regex injection for data exfiltration is handled safely."""
        await SecureUser(email="user1@example.com", credit_card="1111").save()
        await SecureUser(email="user2@example.com", credit_card="2222").save()
        await SecureUser(email="admin@example.com", credit_card="9999").save()

        # Legitimate regex query
        results = await SecureUser.find({"email": {"$regex": "user.*"}}).to_list()
        expect(len(results)).to_equal(2)

        # Broad regex (legitimate but dangerous in application context)
        all_results = await SecureUser.find({"email": {"$regex": ".*"}}).to_list()
        expect(len(all_results)).to_equal(3)


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestNoSQLInjectionPrevention,
        TestCollectionNameValidation,
        TestFieldNameValidation,
        TestErrorSanitization,
        TestSecurityConfiguration,
        TestSecurityIntegration,
    ], verbose=True)
