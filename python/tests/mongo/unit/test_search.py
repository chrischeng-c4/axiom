"""
Tests for text search and geospatial queries.

Tests for:
- Regex escaping utility
- Text search (basic, phrase, language options)
- Geospatial queries ($near, $geoWithin, $geoIntersects)
- Text search with MongoDB index
- Geo queries with MongoDB index

Migrated from test_comprehensive.py and split for maintainability.
"""
from typing import Annotated

from pydantic import Field

from cclab import Document, text_search, escape_regex
from cclab.probe import test, expect
from tests.base import MongoTestSuite, CommonTestSuite


# =====================
# Test Document Classes
# =====================

class GeoPlace(Document):
    """Document for geo query tests."""
    name: str
    location: Annotated[dict, Field(default_factory=dict)]

    class Settings:
        name = "test_geo_places"


class GeoLocation(Document):
    """Document for geo location tests."""
    name: str
    coords: Annotated[list, Field(default_factory=list)]

    class Settings:
        name = "test_geo_locations"


class SearchArticle(Document):
    """Document for text search tests."""
    title: str
    content: str = ""

    class Settings:
        name = "test_search_articles"


# =====================
# Regex Escape Tests (Unit)
# =====================

class TestEscapeRegex(CommonTestSuite):
    """Tests for regex escaping security helper."""

    @test(tags=["unit", "search", "regex"])
    async def test_escape_regex_dot(self):
        """Test escaping dots."""
        result = escape_regex("user@example.com")
        expect(result).to_equal(r"user@example\.com")

    @test(tags=["unit", "search", "regex"])
    async def test_escape_regex_special_chars(self):
        """Test escaping all special characters."""
        input_str = ".^$*+?{}\\|[]()"
        result = escape_regex(input_str)
        expect(result).to_equal(r"\.\^\$\*\+\?\{\}\\\|\[\]\(\)")

    @test(tags=["unit", "search", "regex"])
    async def test_escape_regex_mixed(self):
        """Test mixed content with special chars."""
        result = escape_regex("file (1).txt")
        expect(result).to_equal(r"file \(1\)\.txt")

    @test(tags=["unit", "search", "regex"])
    async def test_escape_regex_no_special(self):
        """Test string with no special chars."""
        result = escape_regex("hello world")
        expect(result).to_equal("hello world")

    @test(tags=["unit", "search", "regex"])
    async def test_escape_regex_empty(self):
        """Test empty string."""
        result = escape_regex("")
        expect(result).to_equal("")


# =====================
# Text Search Tests (Unit)
# =====================

class TestTextSearchUnit(CommonTestSuite):
    """Unit tests for text search filter creation."""

    @test(tags=["unit", "search", "text"])
    async def test_text_search_basic(self):
        """Test basic text search creation."""
        ts = text_search("python rust")
        filter_doc = ts.to_filter()

        expect("$text" in filter_doc).to_be_true()
        expect(filter_doc["$text"]["$search"]).to_equal("python rust")

    @test(tags=["unit", "search", "text"])
    async def test_text_search_with_language(self):
        """Test text search with language."""
        ts = text_search("hello world", language="english")
        filter_doc = ts.to_filter()

        expect(filter_doc["$text"]["$language"]).to_equal("english")

    @test(tags=["unit", "search", "text"])
    async def test_text_search_case_sensitive(self):
        """Test case-sensitive text search."""
        ts = text_search("Python", case_sensitive=True)
        filter_doc = ts.to_filter()

        expect(filter_doc["$text"]["$caseSensitive"]).to_be_true()

    @test(tags=["unit", "search", "text"])
    async def test_text_search_diacritic_sensitive(self):
        """Test diacritic-sensitive text search."""
        ts = text_search("cafe", diacritic_sensitive=True)
        filter_doc = ts.to_filter()

        expect(filter_doc["$text"]["$diacriticSensitive"]).to_be_true()

    @test(tags=["unit", "search", "text"])
    async def test_text_search_repr(self):
        """Test TextSearch string representation."""
        ts = text_search("hello")
        repr_str = repr(ts)
        expect("TextSearch" in repr_str).to_be_true()
        expect("hello" in repr_str).to_be_true()


# =====================
# Geo Queries Tests (Unit)
# =====================

class TestGeoQueriesUnit(CommonTestSuite):
    """Unit tests for geospatial query operators."""

    @test(tags=["unit", "search", "geo"])
    async def test_near_basic(self):
        """Test basic $near query."""
        expr = GeoPlace.location.near([-73.97, 40.77])
        filter_doc = expr.to_filter()

        expect("location" in filter_doc).to_be_true()
        expect("$near" in filter_doc["location"]).to_be_true()
        expect(filter_doc["location"]["$near"]["$geometry"]["type"]).to_equal("Point")
        expect(filter_doc["location"]["$near"]["$geometry"]["coordinates"]).to_equal([-73.97, 40.77])

    @test(tags=["unit", "search", "geo"])
    async def test_near_with_max_distance(self):
        """Test $near with max distance."""
        expr = GeoPlace.location.near([-73.97, 40.77], max_distance=1000)
        filter_doc = expr.to_filter()

        expect(filter_doc["location"]["$near"]["$maxDistance"]).to_equal(1000)

    @test(tags=["unit", "search", "geo"])
    async def test_near_with_min_distance(self):
        """Test $near with min distance."""
        expr = GeoPlace.location.near([-73.97, 40.77], min_distance=100)
        filter_doc = expr.to_filter()

        expect(filter_doc["location"]["$near"]["$minDistance"]).to_equal(100)

    @test(tags=["unit", "search", "geo"])
    async def test_geo_within_box(self):
        """Test $geoWithin with $box."""
        expr = GeoPlace.location.geo_within_box([-74.0, 40.0], [-73.0, 41.0])
        filter_doc = expr.to_filter()

        expect("location" in filter_doc).to_be_true()
        expect("$geoWithin" in filter_doc["location"]).to_be_true()
        expect("$box" in filter_doc["location"]["$geoWithin"]).to_be_true()
        expect(filter_doc["location"]["$geoWithin"]["$box"]).to_equal([[-74.0, 40.0], [-73.0, 41.0]])

    @test(tags=["unit", "search", "geo"])
    async def test_geo_within_polygon(self):
        """Test $geoWithin with polygon."""
        polygon = [[-74.0, 40.0], [-73.0, 40.0], [-73.0, 41.0], [-74.0, 41.0], [-74.0, 40.0]]
        expr = GeoPlace.location.geo_within_polygon(polygon)
        filter_doc = expr.to_filter()

        expect("$geoWithin" in filter_doc["location"]).to_be_true()
        expect("$geometry" in filter_doc["location"]["$geoWithin"]).to_be_true()
        expect(filter_doc["location"]["$geoWithin"]["$geometry"]["type"]).to_equal("Polygon")

    @test(tags=["unit", "search", "geo"])
    async def test_geo_within_center_sphere(self):
        """Test $geoWithin with $centerSphere."""
        # 10 miles in radians
        radius = 10 / 3963.2
        expr = GeoPlace.location.geo_within_center_sphere([-73.97, 40.77], radius)
        filter_doc = expr.to_filter()

        expect("$geoWithin" in filter_doc["location"]).to_be_true()
        expect("$centerSphere" in filter_doc["location"]["$geoWithin"]).to_be_true()

    @test(tags=["unit", "search", "geo"])
    async def test_geo_intersects(self):
        """Test $geoIntersects query."""
        geometry = {
            "type": "Polygon",
            "coordinates": [[[-74, 40], [-73, 40], [-73, 41], [-74, 41], [-74, 40]]]
        }
        expr = GeoPlace.location.geo_intersects(geometry)
        filter_doc = expr.to_filter()

        expect("$geoIntersects" in filter_doc["location"]).to_be_true()
        expect("$geometry" in filter_doc["location"]["$geoIntersects"]).to_be_true()


# =====================
# Text Search Integration Tests (MongoDB)
# =====================

class TestTextSearchIntegration(MongoTestSuite):
    """Integration tests for text search with MongoDB."""

    async def setup(self):
        """Clean up and prepare test data."""
        await SearchArticle.find().delete()
        # Create text index on content field
        await SearchArticle.create_index([("content", "text")])

    async def teardown(self):
        """Clean up test data."""
        await SearchArticle.find().delete()

    @test(tags=["mongo", "search", "text"])
    async def test_text_search_single_match(self):
        """Test text search finds single matching document."""
        await SearchArticle(title="Python Guide", content="Learn Python programming language").save()
        await SearchArticle(title="Rust Guide", content="Learn Rust systems programming").save()
        await SearchArticle(title="JavaScript", content="Learn JavaScript for web development").save()

        results = await SearchArticle.find(text_search("Python")).to_list()
        expect(len(results)).to_equal(1)
        expect(results[0].title).to_equal("Python Guide")

    @test(tags=["mongo", "search", "text"])
    async def test_text_search_multiple_matches(self):
        """Test text search finds multiple matching documents."""
        await SearchArticle(title="Python Guide", content="Learn Python programming language").save()
        await SearchArticle(title="Rust Guide", content="Learn Rust systems programming").save()
        await SearchArticle(title="JavaScript", content="Learn JavaScript for web development").save()

        results = await SearchArticle.find(text_search("programming")).to_list()
        expect(len(results) >= 2).to_be_true()

        titles = [r.title for r in results]
        expect("Python Guide" in titles).to_be_true()
        expect("Rust Guide" in titles).to_be_true()

    @test(tags=["mongo", "search", "text"])
    async def test_text_search_all_matches(self):
        """Test text search finds all matching documents."""
        await SearchArticle(title="A", content="Learn Python").save()
        await SearchArticle(title="B", content="Learn Rust").save()
        await SearchArticle(title="C", content="Learn JavaScript").save()

        results = await SearchArticle.find(text_search("Learn")).to_list()
        expect(len(results)).to_equal(3)


# =====================
# Geo Queries Integration Tests (MongoDB)
# =====================

class TestGeoQueriesIntegration(MongoTestSuite):
    """Integration tests for geospatial queries."""

    async def setup(self):
        """Clean up and prepare test data."""
        await GeoLocation.find().delete()
        # Create 2d index for simple geo queries
        await GeoLocation.create_index([("coords", "2d")])

    async def teardown(self):
        """Clean up test data."""
        await GeoLocation.find().delete()

    @test(tags=["mongo", "search", "geo"])
    async def test_geo_within_box(self):
        """Test $geoWithin with $box query."""
        await GeoLocation(name="NYC", coords=[-73.97, 40.77]).save()
        await GeoLocation(name="Boston", coords=[-71.06, 42.36]).save()
        await GeoLocation(name="Miami", coords=[-80.19, 25.76]).save()

        # Query within box covering NYC and Boston (not Miami)
        results = await GeoLocation.find(
            GeoLocation.coords.geo_within_box([-75.0, 40.0], [-70.0, 43.0])
        ).to_list()

        names = [r.name for r in results]
        expect("NYC" in names).to_be_true()
        expect("Boston" in names).to_be_true()
        expect("Miami" in names).to_be_false()


# =====================
# Transaction Stub Tests (Unit)
# =====================

class TestTransactionStubs(CommonTestSuite):
    """Tests for transaction stub error handling."""

    @test(tags=["unit", "transactions"])
    async def test_start_session_raises_not_implemented(self):
        """Test that start_session raises TransactionNotSupportedError."""
        from cclab import start_session, TransactionNotSupportedError

        error_caught = False
        try:
            await start_session()
        except TransactionNotSupportedError:
            error_caught = True

        expect(error_caught).to_be_true()

    @test(tags=["unit", "transactions"])
    async def test_session_raises_not_implemented(self):
        """Test that Session constructor raises."""
        from cclab import Session, TransactionNotSupportedError

        error_caught = False
        try:
            Session()
        except TransactionNotSupportedError:
            error_caught = True

        expect(error_caught).to_be_true()

    @test(tags=["unit", "transactions"])
    async def test_transaction_not_supported_error_message(self):
        """Test TransactionNotSupportedError has helpful message."""
        from cclab import TransactionNotSupportedError

        error = TransactionNotSupportedError()
        error_msg = str(error)
        expect("Rust backend" in error_msg).to_be_true()
        expect("future release" in error_msg).to_be_true()


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestEscapeRegex,
        TestTextSearchUnit,
        TestGeoQueriesUnit,
        TestTextSearchIntegration,
        TestGeoQueriesIntegration,
        TestTransactionStubs,
    ], verbose=True)
