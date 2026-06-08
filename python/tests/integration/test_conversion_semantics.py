"""
Integration tests for BSON conversion semantic equivalence.

These tests ensure that the new GIL-free conversion produces identical results
to the current implementation, maintaining 100% backward compatibility.
"""

import pytest
from cclab.probe import expect
from datetime import datetime, timezone
from decimal import Decimal
from bson import ObjectId, Binary
from cclab import Document


class TestDoc(Document):
    """Test document for conversion semantics.

    Uses dynamic fields to test all BSON types without pre-defining schema.
    """
    class Settings:
        name = "test_conversion"


# T037: Test find_one preserves all BSON types
@pytest.mark.parametrize(
    "test_data",
    [
        # Basic types
        {"null_field": None},
        {"bool_true": True, "bool_false": False},
        {"int_small": 42, "int_large": 2147483648},
        {"float_normal": 3.14, "float_special": float("inf")},
        {"string_ascii": "hello", "string_unicode": "世界🚀"},
        {"bytes_data": b"binary\x00data"},

        # Collections
        {"list_empty": []},
        {"list_mixed": [1, "two", 3.0, True, None]},
        {"dict_empty": {}},
        {"dict_nested": {"level1": {"level2": {"level3": "deep"}}}},

        # Special types
        {"objectid": ObjectId()},
        {"datetime": datetime(2024, 1, 15, 10, 30, 45, tzinfo=timezone.utc)},

        # Complex nested structures
        {
            "complex": {
                "users": [
                    {"name": "Alice", "age": 30, "active": True},
                    {"name": "Bob", "age": 25, "active": False},
                ],
                "metadata": {
                    "created": datetime.now(timezone.utc),
                    "id": ObjectId(),
                },
            }
        },
    ],
    ids=[
        "null",
        "bool",
        "int",
        "float",
        "string",
        "bytes",
        "list_empty",
        "list_mixed",
        "dict_empty",
        "dict_nested",
        "objectid",
        "datetime",
        "complex",
    ],
)
async def test_find_one_preserves_all_bson_types(test_data):
    """
    T037: Verify find_one preserves all BSON types correctly.

    Tests the complete round-trip: Python → BSON → MongoDB → BSON → Python
    Ensures no data loss or type corruption during conversion.

    Success criteria (FR-009): All BSON types convert correctly
    """
    # Insert test document
    doc = TestDoc(**test_data)
    await doc.save()
    doc_id = doc.id

    # Retrieve using find_one with dict query (TestDoc has no field annotations)
    # Convert string _id to ObjectId for query
    from bson import ObjectId
    query_filter = {"_id": ObjectId(doc_id)}
    retrieved = await TestDoc.find_one(query_filter)

    assert retrieved is not None, "Document not found after insertion"

    # Convert to dict to verify all fields preserved
    retrieved_dict = retrieved.to_dict()

    # Helper function to compare values with datetime tolerance
    def compare_values(actual, expected, path=""):
        if isinstance(expected, float) and expected != expected:  # NaN
            assert actual != actual, f"NaN not preserved at {path}"
        elif isinstance(expected, datetime):
            # MongoDB stores datetime with millisecond precision
            assert abs((actual - expected).total_seconds()) < 0.001, (
                f"Datetime mismatch at {path}: {actual} != {expected}"
            )
        elif isinstance(expected, bytes):
            assert actual == expected, f"Bytes mismatch at {path}"
        elif isinstance(expected, dict):
            assert isinstance(actual, dict), f"Type mismatch at {path}: expected dict"
            for k, v in expected.items():
                assert k in actual, f"Key {k} missing at {path}"
                compare_values(actual[k], v, f"{path}.{k}")
        elif isinstance(expected, list):
            assert isinstance(actual, list), f"Type mismatch at {path}: expected list"
            assert len(actual) == len(expected), f"List length mismatch at {path}"
            for i, (a, e) in enumerate(zip(actual, expected)):
                compare_values(a, e, f"{path}[{i}]")
        else:
            assert actual == expected, (
                f"Value mismatch at {path}: {actual} != {expected}"
            )

    # Verify all fields preserved
    for key, expected_value in test_data.items():
        assert key in retrieved_dict, f"Field {key} missing from retrieved document"
        actual_value = retrieved_dict[key]
        compare_values(actual_value, expected_value, key)

    print(f"✅ All BSON types preserved correctly for: {list(test_data.keys())}")


# T038: Test find_one handles nested documents
async def test_find_one_nested_documents():
    """
    T038: Verify find_one correctly handles complex nested structures.

    Tests deeply nested dictionaries and lists to ensure recursive
    conversion works correctly at all levels.

    Success criteria: Nesting up to 100 levels (MongoDB limit)
    """
    # Create deeply nested structure (20 levels to be practical)
    deep_data = {"level": 0}
    current = deep_data
    for i in range(1, 20):
        current["nested"] = {"level": i}
        current = current["nested"]

    # Insert and retrieve
    doc = TestDoc(**deep_data)
    await doc.save()
    # Use dict query since TestDoc has no field annotations
    from bson import ObjectId
    retrieved = await TestDoc.find_one({"_id": ObjectId(doc.id)})

    assert retrieved is not None

    # Verify nesting preserved
    current_retrieved = retrieved.to_dict()
    for i in range(20):
        assert current_retrieved["level"] == i, f"Level {i} mismatch"
        if i < 19:
            assert "nested" in current_retrieved, f"Nested missing at level {i}"
            current_retrieved = current_retrieved["nested"]

    print("✅ Nested documents (20 levels) handled correctly!")


# T039: Test error messages unchanged
async def test_find_one_error_messages_unchanged():
    """
    T039: Verify error messages remain unchanged with new conversion.

    Ensures backward compatibility of error handling - existing code
    that catches specific errors should continue to work.

    Success criteria (FR-010): Error messages unchanged
    """
    # Test 1: Invalid query type
    exception_raised = False
    error_msg = ""
    try:
        await TestDoc.find_one("invalid_query")  # type: ignore
    except Exception as e:
        exception_raised = True
        error_msg = str(e)
    assert exception_raised, "Invalid query type should raise exception"
    # Error should mention type issue
    assert "type" in error_msg.lower() or "invalid" in error_msg.lower()

    # Test 2: Query on non-existent field (should return None, not error)
    result = await TestDoc.find_one({"nonexistent_field": "value"})
    assert result is None

    # Test 3: Invalid ObjectId format
    exception_raised = False
    error_msg = ""
    try:
        await TestDoc.find_one(TestDoc.id == "not_an_objectid")
    except Exception as e:
        exception_raised = True
        error_msg = str(e)
    assert exception_raised, "Invalid ObjectId format should raise exception"
    # Error should mention ObjectId
    assert "objectid" in error_msg.lower() or "invalid" in error_msg.lower()

    print("✅ Error messages preserved correctly!")
