"""Integration tests for PostgreSQL extensions (Full-Text Search, PostGIS, Arrays)."""
from cclab.probe import expect, test
from tests.postgres.base import PostgresSuite
from cclab.titan import Table, Column, init, FullTextSearch, fts, Point, GeoQuery, ArrayOps, raiseload, selectinload, relationship

class TestPgExtensions(PostgresSuite):

    @test
    async def test_fulltext_to_tsvector(self):
        """Test to_tsvector generation."""
        result = FullTextSearch.to_tsvector('content')
        expect(result).to_equal("to_tsvector('english', content)")

    @test
    async def test_fulltext_to_tsvector_custom_config(self):
        """Test to_tsvector with custom language config."""
        result = FullTextSearch.to_tsvector('content', config='spanish')
        expect(result).to_equal("to_tsvector('spanish', content)")

    @test
    async def test_fulltext_to_tsquery(self):
        """Test to_tsquery generation."""
        result = FullTextSearch.to_tsquery('python & database')
        expect(result).to_equal("to_tsquery('english', 'python & database')")

    @test
    async def test_fulltext_to_tsquery_escapes_quotes(self):
        """Test to_tsquery escapes single quotes."""
        result = FullTextSearch.to_tsquery("it's working")
        expect(result).to_equal("to_tsquery('english', 'it''s working')")

    @test
    async def test_fulltext_plainto_tsquery(self):
        """Test plainto_tsquery generation."""
        result = FullTextSearch.plainto_tsquery('python database')
        expect(result).to_equal("plainto_tsquery('english', 'python database')")

    @test
    async def test_fulltext_match(self):
        """Test match expression generation."""
        result = FullTextSearch.match('content', 'python database')
        expect('to_tsvector').to_be_in(result)
        expect('plainto_tsquery').to_be_in(result)
        expect('@@').to_be_in(result)
        expect(result).to_equal("to_tsvector('english', content) @@ plainto_tsquery('english', 'python database')")

    @test
    async def test_fulltext_rank(self):
        """Test rank expression generation."""
        result = FullTextSearch.rank('content', 'python database')
        expect('ts_rank').to_be_in(result)
        expect('to_tsvector').to_be_in(result)
        expect('plainto_tsquery').to_be_in(result)

    @test
    async def test_fulltext_alias(self):
        """Test fts alias works."""
        result = fts.match('content', 'test')
        expect(result).to_equal(FullTextSearch.match('content', 'test'))

    @test
    async def test_postgis_point_creation(self):
        """Test Point creation."""
        point = Point(121.5, 25.0)
        expect(point.lng).to_equal(121.5)
        expect(point.lat).to_equal(25.0)
        expect(point.srid).to_equal(4326)

    @test
    async def test_postgis_point_custom_srid(self):
        """Test Point with custom SRID."""
        point = Point(121.5, 25.0, srid=3857)
        expect(point.srid).to_equal(3857)

    @test
    async def test_postgis_point_to_sql(self):
        """Test Point to_sql conversion."""
        point = Point(121.5, 25.0)
        sql = point.to_sql()
        expect(sql).to_equal('ST_SetSRID(ST_MakePoint(121.5, 25.0), 4326)')

    @test
    async def test_postgis_point_from_wkt(self):
        """Test Point from_wkt."""
        result = Point.from_wkt('POINT(121.5 25.0)')
        expect(result).to_equal("ST_GeomFromText('POINT(121.5 25.0)', 4326)")

    @test
    async def test_postgis_point_repr(self):
        """Test Point repr."""
        point = Point(121.5, 25.0)
        expect(repr(point)).to_equal('Point(lng=121.5, lat=25.0, srid=4326)')

    @test
    async def test_postgis_distance(self):
        """Test ST_Distance query."""
        result = GeoQuery.distance('coordinates', 'ST_MakePoint(121.5, 25.0)')
        expect(result).to_equal('ST_Distance(coordinates, ST_MakePoint(121.5, 25.0))')

    @test
    async def test_postgis_dwithin(self):
        """Test ST_DWithin query."""
        result = GeoQuery.dwithin('coordinates', 'ST_MakePoint(121.5, 25.0)', 1000)
        expect('ST_DWithin').to_be_in(result)
        expect('1000').to_be_in(result)
        expect(result).to_equal('ST_DWithin(coordinates, ST_MakePoint(121.5, 25.0), 1000)')

    @test
    async def test_postgis_contains(self):
        """Test ST_Contains query."""
        result = GeoQuery.contains('polygon', 'point')
        expect(result).to_equal('ST_Contains(polygon, point)')

    @test
    async def test_postgis_within(self):
        """Test ST_Within query."""
        result = GeoQuery.within('point', 'polygon')
        expect(result).to_equal('ST_Within(point, polygon)')

    @test
    async def test_postgis_intersects(self):
        """Test ST_Intersects query."""
        result = GeoQuery.intersects('geom1', 'geom2')
        expect(result).to_equal('ST_Intersects(geom1, geom2)')

    @test
    async def test_array_contains(self):
        """Test array contains operator."""
        result = ArrayOps.contains('tags', ['python', 'database'])
        expect('tags @>').to_be_in(result)
        expect('ARRAY[').to_be_in(result)
        expect(result).to_equal("tags @> ARRAY['python', 'database']")

    @test
    async def test_array_contained_by(self):
        """Test array contained by operator."""
        result = ArrayOps.contained_by('tags', ['python', 'rust', 'go'])
        expect('tags <@').to_be_in(result)
        expect('ARRAY[').to_be_in(result)

    @test
    async def test_array_overlap(self):
        """Test array overlap operator."""
        result = ArrayOps.overlap('tags', ['python', 'rust'])
        expect('tags &&').to_be_in(result)
        expect('ARRAY[').to_be_in(result)
        expect(result).to_equal("tags && ARRAY['python', 'rust']")

    @test
    async def test_array_any_string(self):
        """Test ANY operator with string."""
        result = ArrayOps.any('tags', 'python')
        expect(result).to_equal("'python' = ANY(tags)")

    @test
    async def test_array_any_number(self):
        """Test ANY operator with number."""
        result = ArrayOps.any('scores', 100)
        expect(result).to_equal('100 = ANY(scores)')

    @test
    async def test_array_length(self):
        """Test array_length function."""
        result = ArrayOps.length('tags')
        expect(result).to_equal('array_length(tags, 1)')

    @test
    async def test_array_format_strings(self):
        """Test array formatting with strings."""
        result = ArrayOps._format_array(['test', 'value'])
        expect(result).to_equal("ARRAY['test', 'value']")

    @test
    async def test_array_format_strings_with_quotes(self):
        """Test array formatting with strings containing quotes."""
        result = ArrayOps._format_array(["it's", 'test'])
        expect(result).to_equal("ARRAY['it''s', 'test']")

    @test
    async def test_array_format_numbers(self):
        """Test array formatting with numbers."""
        result = ArrayOps._format_array([1, 2, 3])
        expect(result).to_equal('ARRAY[1, 2, 3]')

    @test
    async def test_array_format_empty(self):
        """Test array formatting with empty list."""
        result = ArrayOps._format_array([])
        expect(result).to_equal('ARRAY[]')

    @test
    async def test_raiseload_raises_on_access(self):
        """Test that raiseload raises error when relationship is accessed."""
        await Author.create_table()
        await Book.create_table()
        try:
            author = await Author.insert(name='Test Author')
            book = await Book.insert(title='Test Book', author_id=author.id)
            books = await Book.find().options(raiseload('author')).to_list()
            expect(len(books)).to_equal(1)
            try:
                await books[0].author
                raise AssertionError('Expected RuntimeError')
            except RuntimeError as e:
                expect("Attempted to access unloaded relationship 'author'").to_be_in(str(e))
                expect('Use selectinload()').to_be_in(str(e))
        finally:
            await Book.drop_table()
            await Author.drop_table()

    @test
    async def test_raiseload_with_selectinload_works(self):
        """Test that selectinload prevents raiseload error."""
        await Author.create_table()
        await Book.create_table()
        try:
            author = await Author.insert(name='Test Author')
            book = await Book.insert(title='Test Book', author_id=author.id)
            books = await Book.find().options(selectinload('author')).to_list()
            expect(len(books)).to_equal(1)
            loaded_author = await books[0].author
            expect(loaded_author).to_not_be_none()
            expect(loaded_author.name).to_equal('Test Author')
        finally:
            await Book.drop_table()
            await Author.drop_table()

    @test
    async def test_raiseload_invalid_relationship(self):
        """Test raiseload with invalid relationship name."""
        await Author.create_table()
        await Book.create_table()
        try:
            books = await Book.find().options(raiseload('invalid_rel')).to_list()
            raise AssertionError('Expected ValueError')
        except ValueError as e:
            expect('Unknown relationship: invalid_rel').to_be_in(str(e))
        finally:
            await Book.drop_table()
            await Author.drop_table()

class Article(Table):
    id: int = Column(primary_key=True)
    title: str
    content: str

    class Settings:
        table_name = 'test_articles_fts'

class Location(Table):
    id: int = Column(primary_key=True)
    name: str

    class Settings:
        table_name = 'test_locations_postgis'

class Post(Table):
    id: int = Column(primary_key=True)
    title: str

    class Settings:
        table_name = 'test_posts_arrays'

class Author(Table):
    id: int = Column(primary_key=True)
    name: str

    class Settings:
        table_name = 'test_authors_raiseload'

class Book(Table):
    id: int = Column(primary_key=True)
    title: str
    author_id: int = Column(foreign_key='test_authors_raiseload.id')
    author: Author = relationship(Author, foreign_key_column='author_id')

    class Settings:
        table_name = 'test_books_raiseload'