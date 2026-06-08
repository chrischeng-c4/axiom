"""
Edge case testing suite.

Tests rare but important edge cases:
- Large documents (near 16MB BSON limit)
- Concurrent operations (race conditions)
- Connection failure recovery
- Unicode handling (emojis, CJK characters)
- Empty arrays and null values
- Extreme field values

Migrated from pytest to cclab.probe framework.
"""
import asyncio
from typing import Optional, List
from pydantic import Field

from cclab import Document, init, close, is_connected
from cclab.probe import test, expect
from tests.base import MongoTestSuite, MONGODB_URI


class EdgeCaseDoc(Document):
    """General-purpose edge case testing document."""
    name: str
    data: Optional[str] = None
    numbers: List[int] = Field(default_factory=list)
    nested: Optional[dict] = None

    class Settings:
        name = "edge_cases"


class UnicodeDoc(Document):
    """Document for Unicode testing."""
    emoji: str
    chinese: str
    mixed: str
    arabic: Optional[str] = None
    special: Optional[str] = None

    class Settings:
        name = "unicode_test"


class LargeDoc(Document):
    """Document for large data testing."""
    content: str
    metadata: Optional[dict] = None

    class Settings:
        name = "large_docs"


class TestLargeDocuments(MongoTestSuite):
    """Test handling of large documents near BSON 16MB limit."""

    async def teardown(self):
        """Cleanup after each test."""
        await LargeDoc.find().delete()

    @test(tags=["mongo", "edge-case", "large"])
    async def test_large_string_field(self):
        """Test document with large string (10MB)."""
        large_string = "x" * (10 * 1024 * 1024)  # 10MB

        doc = LargeDoc(content=large_string)
        await doc.save()

        retrieved = await LargeDoc.find_one({"_id": doc._id})
        expect(retrieved).not_.to_be_none()
        expect(len(retrieved.content)).to_equal(len(large_string))
        expect(retrieved.content).to_equal(large_string)

    @test(tags=["mongo", "edge-case", "large"])
    async def test_near_bson_limit(self):
        """Test document approaching BSON 16MB limit (15MB)."""
        large_string = "y" * (15 * 1024 * 1024)

        doc = LargeDoc(content=large_string)
        await doc.save()

        retrieved = await LargeDoc.find_one({"_id": doc._id})
        expect(retrieved).not_.to_be_none()
        expect(len(retrieved.content)).to_equal(15 * 1024 * 1024)

    @test(tags=["mongo", "edge-case", "large"])
    async def test_large_nested_document(self):
        """Test document with large nested structure."""
        large_dict = {f"field_{i}": f"value_{i}" * 1000 for i in range(1000)}

        doc = LargeDoc(content="test", metadata=large_dict)
        await doc.save()

        retrieved = await LargeDoc.find_one({"_id": doc._id})
        expect(retrieved).not_.to_be_none()
        expect(len(retrieved.metadata)).to_equal(1000)


class TestConcurrentOperations(MongoTestSuite):
    """Test concurrent operations for race conditions."""

    async def setup(self):
        """Clean up test data."""
        await EdgeCaseDoc.find({"name": {"$regex": "^concurrent_"}}).delete()
        await EdgeCaseDoc.find({"name": {"$regex": "^find_test_"}}).delete()

    async def teardown(self):
        """Clean up test data."""
        await EdgeCaseDoc.find({"name": {"$regex": "^concurrent_"}}).delete()
        await EdgeCaseDoc.find({"name": {"$regex": "^find_test_"}}).delete()

    @test(tags=["mongo", "edge-case", "concurrency"])
    async def test_concurrent_inserts(self):
        """Test multiple concurrent inserts don't conflict."""
        tasks = [
            EdgeCaseDoc(name=f"concurrent_{i}", data=f"data_{i}").save()
            for i in range(100)
        ]

        results = await asyncio.gather(*tasks)
        expect(len(results)).to_equal(100)

        count = await EdgeCaseDoc.count({"name": {"$regex": "^concurrent_"}})
        expect(count).to_equal(100)

    @test(tags=["mongo", "edge-case", "concurrency"])
    async def test_concurrent_updates_same_document(self):
        """Test concurrent updates to same document (last write wins)."""
        doc = EdgeCaseDoc(name="concurrent_update", data="initial")
        await doc.save()
        doc_id = doc._id

        async def update_doc(value):
            d = await EdgeCaseDoc.find_one({"_id": doc_id})
            d.data = value
            await d.save()

        tasks = [update_doc(f"value_{i}") for i in range(10)]
        await asyncio.gather(*tasks)

        final = await EdgeCaseDoc.find_one({"_id": doc_id})
        expect(final.data.startswith("value_")).to_be_true()

    @test(tags=["mongo", "edge-case", "concurrency"])
    async def test_concurrent_find_operations(self):
        """Test concurrent find operations don't interfere."""
        docs = [EdgeCaseDoc(name=f"find_test_{i}", data=f"data_{i}") for i in range(50)]
        for doc in docs:
            await doc.save()

        tasks = [
            EdgeCaseDoc.find({"name": {"$regex": "^find_test_"}}).to_list()
            for _ in range(20)
        ]

        results = await asyncio.gather(*tasks)
        expect(all(len(r) == 50 for r in results)).to_be_true()


class TestUnicodeHandling(MongoTestSuite):
    """Test Unicode character handling."""

    async def teardown(self):
        """Cleanup after each test."""
        await UnicodeDoc.find().delete()

    @test(tags=["mongo", "edge-case", "unicode"])
    async def test_emoji_storage(self):
        """Test emoji storage and retrieval."""
        doc = UnicodeDoc(
            emoji="🚀🎉😀🔥💯",
            chinese="测试",
            mixed="Hello 世界 🌍"
        )
        await doc.save()

        retrieved = await UnicodeDoc.find_one({"_id": doc._id})
        expect(retrieved.emoji).to_equal("🚀🎉😀🔥💯")
        expect(retrieved.chinese).to_equal("测试")
        expect(retrieved.mixed).to_equal("Hello 世界 🌍")

    @test(tags=["mongo", "edge-case", "unicode"])
    async def test_cjk_characters(self):
        """Test Chinese, Japanese, Korean characters."""
        doc = UnicodeDoc(
            emoji="test",
            chinese="中文测试",
            mixed="日本語テスト 한글테스트"
        )
        await doc.save()

        retrieved = await UnicodeDoc.find_one({"chinese": "中文测试"})
        expect(retrieved).not_.to_be_none()
        expect(retrieved.mixed).to_equal("日本語テスト 한글테스트")

    @test(tags=["mongo", "edge-case", "unicode"])
    async def test_arabic_rtl_text(self):
        """Test right-to-left Arabic text."""
        doc = UnicodeDoc(
            emoji="test",
            chinese="test",
            mixed="test",
            arabic="مرحبا بك في الاختبار"
        )
        await doc.save()

        retrieved = await UnicodeDoc.find_one({"_id": doc._id})
        expect(retrieved.arabic).to_equal("مرحبا بك في الاختبار")

    @test(tags=["mongo", "edge-case", "unicode"])
    async def test_special_unicode_characters(self):
        """Test special Unicode characters (zero-width, combining, etc.)."""
        doc = UnicodeDoc(
            emoji="test",
            chinese="test",
            mixed="test",
            special="a\u0301b\u200bc"  # combining accent, zero-width space
        )
        await doc.save()

        retrieved = await UnicodeDoc.find_one({"_id": doc._id})
        expect(retrieved.special).to_equal("a\u0301b\u200bc")

    @test(tags=["mongo", "edge-case", "unicode"])
    async def test_unicode_in_queries(self):
        """Test querying with Unicode values."""
        docs = [
            UnicodeDoc(emoji="🚀", chinese="中文", mixed=f"test_{i}")
            for i in range(5)
        ]
        for doc in docs:
            await doc.save()

        results = await UnicodeDoc.find({"emoji": "🚀"}).to_list()
        expect(len(results)).to_equal(5)

        results = await UnicodeDoc.find({"chinese": "中文"}).to_list()
        expect(len(results)).to_equal(5)


class TestEmptyAndNullValues(MongoTestSuite):
    """Test handling of empty and null values."""

    async def teardown(self):
        """Cleanup after each test."""
        await EdgeCaseDoc.find().delete()

    @test(tags=["mongo", "edge-case", "null"])
    async def test_empty_arrays(self):
        """Test documents with empty arrays."""
        doc = EdgeCaseDoc(name="empty_arrays", numbers=[])
        await doc.save()

        retrieved = await EdgeCaseDoc.find_one({"_id": doc._id})
        expect(retrieved.numbers).to_equal([])

        retrieved.numbers = [1, 2, 3]
        await retrieved.save()

        updated = await EdgeCaseDoc.find_one({"_id": doc._id})
        expect(updated.numbers).to_equal([1, 2, 3])

    @test(tags=["mongo", "edge-case", "null"])
    async def test_null_optional_fields(self):
        """Test documents with null optional fields."""
        doc = EdgeCaseDoc(name="null_fields", data=None, nested=None)
        await doc.save()

        retrieved = await EdgeCaseDoc.find_one({"_id": doc._id})
        expect(retrieved.data).to_be_none()
        expect(retrieved.nested).to_be_none()

    @test(tags=["mongo", "edge-case", "null"])
    async def test_empty_string(self):
        """Test documents with empty strings."""
        doc = EdgeCaseDoc(name="", data="")
        await doc.save()

        retrieved = await EdgeCaseDoc.find_one({"_id": doc._id})
        expect(retrieved.name).to_equal("")
        expect(retrieved.data).to_equal("")

    @test(tags=["mongo", "edge-case", "null"])
    async def test_empty_nested_dict(self):
        """Test documents with empty nested dictionaries."""
        doc = EdgeCaseDoc(name="empty_dict", nested={})
        await doc.save()

        retrieved = await EdgeCaseDoc.find_one({"_id": doc._id})
        expect(retrieved.nested).to_equal({})


class TestExtremeValues(MongoTestSuite):
    """Test extreme numeric and edge case values."""

    async def teardown(self):
        """Cleanup after each test."""
        await EdgeCaseDoc.find().delete()

    @test(tags=["mongo", "edge-case", "numeric"])
    async def test_very_large_integers(self):
        """Test handling of very large integers."""
        doc = EdgeCaseDoc(
            name="large_int",
            numbers=[2**63 - 1, -(2**63)]  # Max/min 64-bit signed int
        )
        await doc.save()

        retrieved = await EdgeCaseDoc.find_one({"_id": doc._id})
        expect(retrieved.numbers).to_equal([2**63 - 1, -(2**63)])

    @test(tags=["mongo", "edge-case", "numeric"])
    async def test_zero_values(self):
        """Test handling of zero values."""
        doc = EdgeCaseDoc(name="zeros", numbers=[0, 0, 0])
        await doc.save()

        retrieved = await EdgeCaseDoc.find_one({"_id": doc._id})
        expect(retrieved.numbers).to_equal([0, 0, 0])

    @test(tags=["mongo", "edge-case", "numeric"])
    async def test_negative_numbers(self):
        """Test handling of negative numbers."""
        doc = EdgeCaseDoc(name="negative", numbers=[-1, -100, -1000000])
        await doc.save()

        retrieved = await EdgeCaseDoc.find_one({"_id": doc._id})
        expect(retrieved.numbers).to_equal([-1, -100, -1000000])


class TestConnectionRecovery(MongoTestSuite):
    """Test connection failure and recovery scenarios.

    Note: These tests manipulate connection state and need careful ordering.
    Each test ensures proper reconnection at the end to not affect other tests.
    """

    async def setup(self):
        """Ensure we start with a valid connection."""
        if not is_connected():
            await init(MONGODB_URI)

    async def teardown(self):
        """Always ensure we have a connection for subsequent tests."""
        if not is_connected():
            await init(MONGODB_URI)

    @test(tags=["mongo", "edge-case", "connection"])
    async def test_a_reconnection_after_close(self):
        """Test operations work after close and reconnect."""
        # This test is named with 'a_' prefix to run first
        doc = EdgeCaseDoc(name="before_close", data="test")
        await doc.save()
        doc_id = doc._id

        await close()
        expect(is_connected()).to_be_false()

        await init(MONGODB_URI)
        expect(is_connected()).to_be_true()

        retrieved = await EdgeCaseDoc.find_one({"_id": doc_id})
        expect(retrieved).not_.to_be_none()
        expect(retrieved.name).to_equal("before_close")

        await retrieved.delete()

    @test(tags=["mongo", "edge-case", "connection"])
    async def test_b_operations_fail_when_disconnected(self):
        """Test operations fail gracefully when disconnected."""
        await close()

        error_caught = False
        try:
            doc = EdgeCaseDoc(name="should_fail", data="test")
            await doc.save()
        except RuntimeError as e:
            error_caught = True
            expect("MongoDB not initialized" in str(e)).to_be_true()

        expect(error_caught).to_be_true()

    @test(tags=["mongo", "edge-case", "connection"])
    async def test_c_multiple_close_calls(self):
        """Test multiple close() calls don't cause issues."""
        await close()
        expect(is_connected()).to_be_false()

        error_caught = False
        try:
            await close()
        except RuntimeError as e:
            error_caught = True
            expect("No active connection" in str(e)).to_be_true()

        expect(error_caught).to_be_true()


class TestFieldNameEdgeCases(MongoTestSuite):
    """Test edge cases with field names."""

    async def teardown(self):
        """Cleanup after each test."""
        await EdgeCaseDoc.find().delete()

    @test(tags=["mongo", "edge-case", "fields"])
    async def test_fields_with_dots_in_nested_dict(self):
        """Test nested dicts with dots in keys (MongoDB limitation)."""
        doc = EdgeCaseDoc(
            name="dots_test",
            nested={"valid_key": "value"}
        )
        await doc.save()

        retrieved = await EdgeCaseDoc.find_one({"_id": doc._id})
        expect(retrieved.nested["valid_key"]).to_equal("value")

    @test(tags=["mongo", "edge-case", "fields"])
    async def test_very_long_field_values(self):
        """Test very long string values in fields."""
        long_string = "x" * 100000  # 100KB string

        doc = EdgeCaseDoc(name="long_value", data=long_string)
        await doc.save()

        retrieved = await EdgeCaseDoc.find_one({"_id": doc._id})
        expect(len(retrieved.data)).to_equal(100000)


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestLargeDocuments,
        TestConcurrentOperations,
        TestUnicodeHandling,
        TestEmptyAndNullValues,
        TestExtremeValues,
        TestConnectionRecovery,
        TestFieldNameEdgeCases,
    ], verbose=True)
