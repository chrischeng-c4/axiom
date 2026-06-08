"""
Tests for constraint validation in data-bridge.

Tests cover:
- String constraints (min_length, max_length)
- Numeric constraints (min, max)
- Format validation (email, url)
- Edge cases (optional with constraints, null handling, boundary values)
"""

from typing import Annotated, Optional

from cclab import (
    MinLen,
    MaxLen,
    Min,
    Max,
    Email,
    Url,
    Constraint,
)
from cclab.nebula.type_extraction import (
    python_type_to_bson_type,
    extract_constraints,
    is_annotated_type,
    unwrap_annotated_type,
)
from cclab.probe import TestSuite, test, expect

from tests.base import CommonTestSuite


class TestConstraintClasses(CommonTestSuite):
    """Test the constraint class implementations."""

    @test(tags=["unit", "constraints"])
    async def test_minlen_instantiation(self):
        """MinLen stores min_length correctly."""
        c = MinLen(3)
        expect(c.min_length).to_equal(3)
        expect(c.to_dict()).to_equal({"min_length": 3})

    @test(tags=["unit", "constraints"])
    async def test_minlen_negative_raises(self):
        """MinLen rejects negative values."""
        try:
            MinLen(-1)
            expect(False).to_be_true()  # Should not reach here
        except ValueError as e:
            expect(str(e)).to_contain("non-negative")

    @test(tags=["unit", "constraints"])
    async def test_maxlen_instantiation(self):
        """MaxLen stores max_length correctly."""
        c = MaxLen(50)
        expect(c.max_length).to_equal(50)
        expect(c.to_dict()).to_equal({"max_length": 50})

    @test(tags=["unit", "constraints"])
    async def test_maxlen_negative_raises(self):
        """MaxLen rejects negative values."""
        try:
            MaxLen(-1)
            expect(False).to_be_true()  # Should not reach here
        except ValueError as e:
            expect(str(e)).to_contain("non-negative")

    @test(tags=["unit", "constraints"])
    async def test_min_instantiation(self):
        """Min stores min value correctly."""
        c = Min(0)
        expect(c.min).to_equal(0)
        expect(c.to_dict()).to_equal({"min": 0})

    @test(tags=["unit", "constraints"])
    async def test_min_with_float(self):
        """Min works with float values."""
        c = Min(0.5)
        expect(c.min).to_equal(0.5)
        expect(c.to_dict()).to_equal({"min": 0.5})

    @test(tags=["unit", "constraints"])
    async def test_max_instantiation(self):
        """Max stores max value correctly."""
        c = Max(100)
        expect(c.max).to_equal(100)
        expect(c.to_dict()).to_equal({"max": 100})

    @test(tags=["unit", "constraints"])
    async def test_max_with_float(self):
        """Max works with float values."""
        c = Max(99.9)
        expect(c.max).to_equal(99.9)
        expect(c.to_dict()).to_equal({"max": 99.9})

    @test(tags=["unit", "constraints"])
    async def test_email_instantiation(self):
        """Email constraint stores format correctly."""
        c = Email()
        expect(c.format).to_equal("email")
        expect(c.to_dict()).to_equal({"format": "email"})

    @test(tags=["unit", "constraints"])
    async def test_url_instantiation(self):
        """Url constraint stores format correctly."""
        c = Url()
        expect(c.format).to_equal("url")
        expect(c.to_dict()).to_equal({"format": "url"})

    @test(tags=["unit", "constraints"])
    async def test_constraint_repr(self):
        """Constraint __repr__ is informative."""
        expect(repr(MinLen(3))).to_equal("MinLen(3)")
        expect(repr(MaxLen(50))).to_equal("MaxLen(50)")
        expect(repr(Min(0))).to_equal("Min(0)")
        expect(repr(Max(100))).to_equal("Max(100)")
        expect(repr(Email())).to_equal("Email()")
        expect(repr(Url())).to_equal("Url()")


class TestAnnotatedTypeDetection(CommonTestSuite):
    """Test detection and extraction of Annotated types."""

    @test(tags=["unit", "constraints"])
    async def test_is_annotated_type_true(self):
        """is_annotated_type returns True for Annotated types."""
        expect(is_annotated_type(Annotated[str, MinLen(3)])).to_be_true()
        expect(is_annotated_type(Annotated[int, Min(0)])).to_be_true()

    @test(tags=["unit", "constraints"])
    async def test_is_annotated_type_false(self):
        """is_annotated_type returns False for non-Annotated types."""
        expect(is_annotated_type(str)).to_be_false()
        expect(is_annotated_type(int)).to_be_false()
        expect(is_annotated_type(Optional[str])).to_be_false()

    @test(tags=["unit", "constraints"])
    async def test_unwrap_annotated_type(self):
        """unwrap_annotated_type extracts base type and metadata."""
        base, metadata = unwrap_annotated_type(Annotated[str, MinLen(3)])
        expect(base is str).to_be_true()
        expect(len(metadata)).to_equal(1)
        expect(isinstance(metadata[0], MinLen)).to_be_true()

    @test(tags=["unit", "constraints"])
    async def test_unwrap_annotated_type_multiple_metadata(self):
        """unwrap_annotated_type handles multiple metadata items."""
        base, metadata = unwrap_annotated_type(Annotated[str, MinLen(3), MaxLen(50)])
        expect(base is str).to_be_true()
        expect(len(metadata)).to_equal(2)
        expect(isinstance(metadata[0], MinLen)).to_be_true()
        expect(isinstance(metadata[1], MaxLen)).to_be_true()

    @test(tags=["unit", "constraints"])
    async def test_unwrap_non_annotated_type(self):
        """unwrap_annotated_type returns original type with empty metadata."""
        base, metadata = unwrap_annotated_type(str)
        expect(base is str).to_be_true()
        expect(metadata).to_equal(())


class TestConstraintExtraction(CommonTestSuite):
    """Test constraint extraction from Annotated types."""

    @test(tags=["unit", "constraints"])
    async def test_extract_single_constraint(self):
        """extract_constraints extracts single constraint."""
        constraints = extract_constraints(Annotated[str, MinLen(3)])
        expect(constraints).to_equal({"min_length": 3})

    @test(tags=["unit", "constraints"])
    async def test_extract_multiple_constraints(self):
        """extract_constraints merges multiple constraints."""
        constraints = extract_constraints(Annotated[str, MinLen(3), MaxLen(50)])
        expect(constraints).to_equal({"min_length": 3, "max_length": 50})

    @test(tags=["unit", "constraints"])
    async def test_extract_numeric_constraints(self):
        """extract_constraints extracts numeric constraints."""
        constraints = extract_constraints(Annotated[int, Min(0), Max(100)])
        expect(constraints).to_equal({"min": 0, "max": 100})

    @test(tags=["unit", "constraints"])
    async def test_extract_format_constraint(self):
        """extract_constraints extracts format constraints."""
        constraints = extract_constraints(Annotated[str, Email()])
        expect(constraints).to_equal({"format": "email"})

    @test(tags=["unit", "constraints"])
    async def test_extract_no_constraints(self):
        """extract_constraints returns empty dict for non-Annotated types."""
        constraints = extract_constraints(str)
        expect(constraints).to_equal({})

    @test(tags=["unit", "constraints"])
    async def test_extract_ignores_non_constraint_metadata(self):
        """extract_constraints ignores non-Constraint metadata."""
        constraints = extract_constraints(Annotated[str, "some_metadata"])
        expect(constraints).to_equal({})


class TestTypeDescriptorWithConstraints(CommonTestSuite):
    """Test python_type_to_bson_type produces constraints in output."""

    @test(tags=["unit", "constraints"])
    async def test_string_with_min_length(self):
        """String with MinLen produces constraints in descriptor."""
        result = python_type_to_bson_type(Annotated[str, MinLen(3)])
        expect(result).to_equal({"type": "string", "constraints": {"min_length": 3}})

    @test(tags=["unit", "constraints"])
    async def test_string_with_max_length(self):
        """String with MaxLen produces constraints in descriptor."""
        result = python_type_to_bson_type(Annotated[str, MaxLen(50)])
        expect(result).to_equal({"type": "string", "constraints": {"max_length": 50}})

    @test(tags=["unit", "constraints"])
    async def test_string_with_both_lengths(self):
        """String with both MinLen and MaxLen produces combined constraints."""
        result = python_type_to_bson_type(Annotated[str, MinLen(3), MaxLen(50)])
        expect(result).to_equal({
            "type": "string",
            "constraints": {"min_length": 3, "max_length": 50},
        })

    @test(tags=["unit", "constraints"])
    async def test_string_with_email(self):
        """String with Email produces format constraint."""
        result = python_type_to_bson_type(Annotated[str, Email()])
        expect(result).to_equal({"type": "string", "constraints": {"format": "email"}})

    @test(tags=["unit", "constraints"])
    async def test_string_with_url(self):
        """String with Url produces format constraint."""
        result = python_type_to_bson_type(Annotated[str, Url()])
        expect(result).to_equal({"type": "string", "constraints": {"format": "url"}})

    @test(tags=["unit", "constraints"])
    async def test_int_with_min(self):
        """Int with Min produces numeric constraint."""
        result = python_type_to_bson_type(Annotated[int, Min(0)])
        expect(result).to_equal({"type": "int64", "constraints": {"min": 0}})

    @test(tags=["unit", "constraints"])
    async def test_int_with_max(self):
        """Int with Max produces numeric constraint."""
        result = python_type_to_bson_type(Annotated[int, Max(100)])
        expect(result).to_equal({"type": "int64", "constraints": {"max": 100}})

    @test(tags=["unit", "constraints"])
    async def test_int_with_both(self):
        """Int with both Min and Max produces combined constraints."""
        result = python_type_to_bson_type(Annotated[int, Min(0), Max(100)])
        expect(result).to_equal({"type": "int64", "constraints": {"min": 0, "max": 100}})

    @test(tags=["unit", "constraints"])
    async def test_float_with_constraints(self):
        """Float with numeric constraints."""
        result = python_type_to_bson_type(Annotated[float, Min(0.0), Max(1.0)])
        expect(result).to_equal({"type": "double", "constraints": {"min": 0.0, "max": 1.0}})

    @test(tags=["unit", "constraints"])
    async def test_plain_type_no_constraints(self):
        """Plain type has no constraints key."""
        result = python_type_to_bson_type(str)
        expect(result).to_equal({"type": "string"})
        expect("constraints" in result).to_be_false()

    @test(tags=["unit", "constraints"])
    async def test_plain_int_no_constraints(self):
        """Plain int has no constraints key."""
        result = python_type_to_bson_type(int)
        expect(result).to_equal({"type": "int64"})
        expect("constraints" in result).to_be_false()


class TestStringConstraintValidation(CommonTestSuite):
    """Test string constraint validation through the type extraction system."""

    @test(tags=["unit", "constraints"])
    async def test_min_length_descriptor_structure(self):
        """Verify min_length constraint in type descriptor."""
        result = python_type_to_bson_type(Annotated[str, MinLen(5)])
        expect(result["type"]).to_equal("string")
        expect(result["constraints"]["min_length"]).to_equal(5)

    @test(tags=["unit", "constraints"])
    async def test_max_length_descriptor_structure(self):
        """Verify max_length constraint in type descriptor."""
        result = python_type_to_bson_type(Annotated[str, MaxLen(100)])
        expect(result["type"]).to_equal("string")
        expect(result["constraints"]["max_length"]).to_equal(100)

    @test(tags=["unit", "constraints"])
    async def test_combined_length_constraints(self):
        """Both min and max length can be combined."""
        result = python_type_to_bson_type(Annotated[str, MinLen(2), MaxLen(10)])
        expect(result["constraints"]["min_length"]).to_equal(2)
        expect(result["constraints"]["max_length"]).to_equal(10)


class TestNumericConstraintValidation(CommonTestSuite):
    """Test numeric constraint validation through type extraction."""

    @test(tags=["unit", "constraints"])
    async def test_int_min_constraint(self):
        """Verify int min constraint in type descriptor."""
        result = python_type_to_bson_type(Annotated[int, Min(0)])
        expect(result["type"]).to_equal("int64")
        expect(result["constraints"]["min"]).to_equal(0)

    @test(tags=["unit", "constraints"])
    async def test_int_max_constraint(self):
        """Verify int max constraint in type descriptor."""
        result = python_type_to_bson_type(Annotated[int, Max(150)])
        expect(result["type"]).to_equal("int64")
        expect(result["constraints"]["max"]).to_equal(150)

    @test(tags=["unit", "constraints"])
    async def test_negative_min_allowed(self):
        """Negative min values are allowed."""
        result = python_type_to_bson_type(Annotated[int, Min(-100)])
        expect(result["constraints"]["min"]).to_equal(-100)

    @test(tags=["unit", "constraints"])
    async def test_float_constraints(self):
        """Float constraints work with decimal values."""
        result = python_type_to_bson_type(Annotated[float, Min(0.01), Max(99.99)])
        expect(result["type"]).to_equal("double")
        expect(result["constraints"]["min"]).to_equal(0.01)
        expect(result["constraints"]["max"]).to_equal(99.99)


class TestFormatConstraintValidation(CommonTestSuite):
    """Test format constraint validation through type extraction."""

    @test(tags=["unit", "constraints"])
    async def test_email_format_descriptor(self):
        """Verify email format in type descriptor."""
        result = python_type_to_bson_type(Annotated[str, Email()])
        expect(result["type"]).to_equal("string")
        expect(result["constraints"]["format"]).to_equal("email")

    @test(tags=["unit", "constraints"])
    async def test_url_format_descriptor(self):
        """Verify url format in type descriptor."""
        result = python_type_to_bson_type(Annotated[str, Url()])
        expect(result["type"]).to_equal("string")
        expect(result["constraints"]["format"]).to_equal("url")

    @test(tags=["unit", "constraints"])
    async def test_email_with_length_constraints(self):
        """Email format can be combined with length constraints."""
        result = python_type_to_bson_type(
            Annotated[str, MinLen(5), MaxLen(254), Email()]
        )
        expect(result["constraints"]["min_length"]).to_equal(5)
        expect(result["constraints"]["max_length"]).to_equal(254)
        expect(result["constraints"]["format"]).to_equal("email")


class TestEdgeCases(CommonTestSuite):
    """Test edge cases for constraint handling."""

    @test(tags=["unit", "constraints"])
    async def test_boundary_min_length_zero(self):
        """min_length=0 is allowed (no minimum)."""
        c = MinLen(0)
        expect(c.min_length).to_equal(0)

    @test(tags=["unit", "constraints"])
    async def test_boundary_max_length_zero(self):
        """max_length=0 means empty string only."""
        c = MaxLen(0)
        expect(c.max_length).to_equal(0)

    @test(tags=["unit", "constraints"])
    async def test_min_equals_max_numeric(self):
        """min and max can be equal (exact value required)."""
        result = python_type_to_bson_type(Annotated[int, Min(42), Max(42)])
        expect(result["constraints"]["min"]).to_equal(42)
        expect(result["constraints"]["max"]).to_equal(42)

    @test(tags=["unit", "constraints"])
    async def test_min_equals_max_length(self):
        """min_length and max_length can be equal (exact length required)."""
        result = python_type_to_bson_type(Annotated[str, MinLen(10), MaxLen(10)])
        expect(result["constraints"]["min_length"]).to_equal(10)
        expect(result["constraints"]["max_length"]).to_equal(10)

    @test(tags=["unit", "constraints"])
    async def test_constraint_base_class(self):
        """Constraint base class has correct __constraint_type__."""
        expect(Constraint.__constraint_type__).to_equal("base")

    @test(tags=["unit", "constraints"])
    async def test_minlen_constraint_type(self):
        """MinLen has correct __constraint_type__."""
        expect(MinLen.__constraint_type__).to_equal("min_length")

    @test(tags=["unit", "constraints"])
    async def test_email_constraint_type(self):
        """Email has correct __constraint_type__."""
        expect(Email.__constraint_type__).to_equal("format")


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites, ReportFormat

    suites = [
        TestConstraintClasses,
        TestAnnotatedTypeDetection,
        TestConstraintExtraction,
        TestTypeDescriptorWithConstraints,
        TestStringConstraintValidation,
        TestNumericConstraintValidation,
        TestFormatConstraintValidation,
        TestEdgeCases,
    ]

    run_suites(suites, output_format=ReportFormat.Markdown, verbose=True)
