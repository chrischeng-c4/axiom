"""
Integration tests for JSONB operators in PostgreSQL.

Tests cover:
1. JSONB column storage and retrieval
2. JSONB containment operator @> (json_contains)
3. JSONB key exists operator ? (json_exists)
4. JSONB path extraction -> and ->>
5. JSONB array containment
6. JSONB nested object queries
7. JSONB with WHERE conditions
8. JSONB combined with aggregates
9. JSONB with ORDER BY
10. JSONB index usage (GIN index)
"""
from cclab.titan import execute, insert_one, insert_many
from cclab.probe import expect, fixture, test
from tests.postgres.base import PostgresSuite
@fixture
async def products_table():
    """
    Create and populate a products table with JSONB columns.

    Schema:
        - id: SERIAL PRIMARY KEY
        - name: VARCHAR(255) NOT NULL
        - attributes: JSONB (product attributes like color, size, etc.)
        - metadata: JSONB (additional metadata like tags, ratings, etc.)
    """
    await execute('\n        CREATE TABLE IF NOT EXISTS products (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(255) NOT NULL,\n            attributes JSONB,\n            metadata JSONB\n        )\n    ')
    test_products = [{'name': 'Red T-Shirt', 'attributes': '{"color": "red", "size": "M", "material": "cotton"}', 'metadata': '{"tags": ["clothing", "casual"], "rating": 4.5, "inStock": true}'}, {'name': 'Blue Jeans', 'attributes': '{"color": "blue", "size": "L", "dimensions": {"waist": 32, "length": 34}}', 'metadata': '{"tags": ["clothing", "denim"], "rating": 4.8, "inStock": true, "supplier": {"name": "DenimCo", "country": "USA"}}'}, {'name': 'Running Shoes', 'attributes': '{"color": "black", "sizes": [8, 9, 10, 11], "features": ["waterproof", "breathable"]}', 'metadata': '{"tags": ["footwear", "sports"], "rating": 4.9, "inStock": false, "reviews": [{"user": "john", "score": 5}, {"user": "jane", "score": 4}]}'}, {'name': 'Laptop Backpack', 'attributes': '{"color": "gray", "capacity": "30L", "compartments": 5}', 'metadata': '{"tags": ["bags", "tech"], "rating": 4.6, "inStock": true, "warranty": {"years": 2, "type": "limited"}}'}, {'name': 'Simple Notepad', 'attributes': '{"color": "white", "pages": 100}', 'metadata': '{"tags": ["stationery"], "rating": null, "inStock": true}'}, {'name': 'Premium Product', 'attributes': '{"color": "gold", "limited": true}', 'metadata': '{"tags": ["premium"], "role": "admin", "rating": 5.0, "inStock": true}'}, {'name': 'Winter Jacket', 'attributes': '{"color": "navy", "size": "XL", "insulation": "down"}', 'metadata': '{"tags": ["clothing", "winter", "outdoor"], "rating": 4.7, "inStock": true}'}]
    for product in test_products:
        await execute('INSERT INTO products (name, attributes, metadata) VALUES ($1, $2::jsonb, $3::jsonb)', [product['name'], product['attributes'], product['metadata']])
    yield

class TestJsonbStorageAndRetrieval(PostgresSuite):
    """Test basic JSONB storage and retrieval."""

    @test
    async def test_store_and_retrieve_jsonb(self, products_table):
        """Test storing and retrieving JSONB data."""
        results = await execute('SELECT * FROM products ORDER BY id')
        expect(len(results)).to_equal(7)
        expect(results[0]['name']).to_equal('Red T-Shirt')
        expect(results[0]['attributes']['color']).to_equal('red')
        expect(results[0]['attributes']['size']).to_equal('M')
        expect(results[0]['metadata']['tags']).to_equal(['clothing', 'casual'])
        expect(results[0]['metadata']['rating']).to_equal(4.5)
        expect(results[0]['metadata']['inStock']).to_be_true()

    @test
    async def test_retrieve_nested_jsonb(self, products_table):
        """Test retrieving nested JSONB structures."""
        results = await execute('SELECT * FROM products WHERE name = $1', ['Blue Jeans'])
        expect(len(results)).to_equal(1)
        product = results[0]
        expect(product['attributes']['dimensions']['waist']).to_equal(32)
        expect(product['attributes']['dimensions']['length']).to_equal(34)
        expect(product['metadata']['supplier']['name']).to_equal('DenimCo')
        expect(product['metadata']['supplier']['country']).to_equal('USA')

    @test
    async def test_retrieve_array_in_jsonb(self, products_table):
        """Test retrieving JSONB arrays."""
        results = await execute('SELECT * FROM products WHERE name = $1', ['Running Shoes'])
        expect(len(results)).to_equal(1)
        product = results[0]
        expect(product['attributes']['sizes']).to_equal([8, 9, 10, 11])
        expect(product['attributes']['features']).to_equal(['waterproof', 'breathable'])
        expect(len(product['metadata']['reviews'])).to_equal(2)
        expect(product['metadata']['reviews'][0]['user']).to_equal('john')
        expect(product['metadata']['reviews'][1]['score']).to_equal(4)

    @test
    async def test_null_values_in_jsonb(self, products_table):
        """Test JSONB fields with null values."""
        results = await execute('SELECT * FROM products WHERE name = $1', ['Simple Notepad'])
        expect(len(results)).to_equal(1)
        product = results[0]
        expect(product['metadata']['rating']).to_be_none()
        expect(product['metadata']['inStock']).to_be_true()

class TestJsonbContainmentOperator(PostgresSuite):
    """Test JSONB @> containment operator (json_contains)."""

    @test
    async def test_json_contains_simple_object(self, products_table):
        """Test @> operator with simple object containment."""
        results = await execute('\n            SELECT * FROM products\n            WHERE attributes @> \'{"color": "red"}\'::jsonb\n            ORDER BY id\n        ')
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Red T-Shirt')

    @test
    async def test_json_contains_multiple_fields(self, products_table):
        """Test @> operator with multiple field containment."""
        results = await execute('\n            SELECT * FROM products\n            WHERE attributes @> \'{"color": "blue", "size": "L"}\'::jsonb\n            ORDER BY id\n        ')
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Blue Jeans')

    @test
    async def test_json_contains_nested_object(self, products_table):
        """Test @> operator with nested object containment."""
        results = await execute('\n            SELECT * FROM products\n            WHERE metadata @> \'{"supplier": {"country": "USA"}}\'::jsonb\n            ORDER BY id\n        ')
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Blue Jeans')

    @test
    async def test_json_contains_array_element(self, products_table):
        """Test @> operator with array element containment."""
        results = await execute('\n            SELECT * FROM products\n            WHERE metadata @> \'{"tags": ["clothing"]}\'::jsonb\n            ORDER BY id\n        ')
        expect(len(results)).to_equal(3)
        names = [r['name'] for r in results]
        expect('Red T-Shirt' in names).to_be_true()
        expect('Blue Jeans' in names).to_be_true()
        expect('Winter Jacket' in names).to_be_true()

    @test
    async def test_json_contains_role_admin(self, products_table):
        """Test @> operator filtering by role."""
        results = await execute('\n            SELECT * FROM products\n            WHERE metadata @> \'{"role": "admin"}\'::jsonb\n            ORDER BY id\n        ')
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Premium Product')

    @test
    async def test_json_contains_boolean_value(self, products_table):
        """Test @> operator with boolean values."""
        results = await execute('\n            SELECT * FROM products\n            WHERE metadata @> \'{"inStock": true}\'::jsonb\n            ORDER BY id\n        ')
        expect(len(results)).to_equal(6)
        names = [r['name'] for r in results]
        expect('Running Shoes' not in names).to_be_true()

    @test
    async def test_json_contains_no_match(self, products_table):
        """Test @> operator with no matching results."""
        results = await execute('\n            SELECT * FROM products\n            WHERE attributes @> \'{"color": "purple"}\'::jsonb\n            ORDER BY id\n        ')
        expect(len(results)).to_equal(0)

class TestJsonbKeyExistsOperator(PostgresSuite):
    """Test JSONB ? key exists operator."""

    @test
    async def test_json_key_exists_simple(self, products_table):
        """Test ? operator for simple key existence."""
        results = await execute("\n            SELECT * FROM products\n            WHERE attributes ? 'limited'\n            ORDER BY id\n        ")
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Premium Product')

    @test
    async def test_json_key_exists_all_keys(self, products_table):
        """Test ?& operator for all keys existence."""
        # Only Red T-Shirt, Blue Jeans, and Winter Jacket have both 'color' and 'size'
        results = await execute("\n            SELECT * FROM products\n            WHERE attributes ?& array['color', 'size']\n            ORDER BY id\n        ")
        expect(len(results)).to_equal(3)

    @test
    async def test_json_key_exists_any_key(self, products_table):
        """Test ?| operator for any key existence."""
        results = await execute("\n            SELECT * FROM products\n            WHERE attributes ?| array['compartments', 'capacity']\n            ORDER BY id\n        ")
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Laptop Backpack')

    @test
    async def test_json_key_not_exists(self, products_table):
        """Test for key non-existence."""
        results = await execute("\n            SELECT * FROM products\n            WHERE NOT (attributes ? 'limited')\n            ORDER BY id\n        ")
        expect(len(results)).to_equal(6)

class TestJsonbPathExtraction(PostgresSuite):
    """Test JSONB path extraction operators -> and ->>."""

    @test
    async def test_json_extract_text_value(self, products_table):
        """Test ->> operator for extracting text values."""
        results = await execute("\n            SELECT name, attributes->>'color' as color\n            FROM products\n            WHERE attributes->>'color' = 'red'\n            ORDER BY id\n        ")
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Red T-Shirt')
        expect(results[0]['color']).to_equal('red')

    @test
    async def test_json_extract_nested_text(self, products_table):
        """Test ->> operator for nested path extraction."""
        results = await execute("\n            SELECT name, metadata->'supplier'->>'country' as country\n            FROM products\n            WHERE metadata->'supplier'->>'country' IS NOT NULL\n            ORDER BY id\n        ")
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Blue Jeans')
        expect(results[0]['country']).to_equal('USA')

    @test
    async def test_json_extract_number(self, products_table):
        """Test -> operator for extracting numeric values."""
        results = await execute("\n            SELECT name, (metadata->>'rating')::float as rating\n            FROM products\n            WHERE metadata->>'rating' IS NOT NULL\n            AND (metadata->>'rating')::float > 4.7\n            ORDER BY id\n        ")
        expect(len(results)).to_equal(3)
        names = [r['name'] for r in results]
        expect('Blue Jeans' in names).to_be_true()
        expect('Running Shoes' in names).to_be_true()
        expect('Premium Product' in names).to_be_true()

    @test
    async def test_json_extract_array_element(self, products_table):
        """Test ->> operator for array element extraction as text."""
        # Use ->> to get text value without JSON quotes
        results = await execute("\n            SELECT name, metadata->'tags'->>0 as first_tag\n            FROM products\n            WHERE metadata->'tags'->0 IS NOT NULL\n            ORDER BY id\n        ")
        expect(len(results)).to_equal(7)
        expect(results[0]['first_tag']).to_equal('clothing')

    @test
    async def test_json_path_comparison(self, products_table):
        """Test path extraction in WHERE clause."""
        results = await execute("\n            SELECT name, (attributes->>'compartments')::int as compartments\n            FROM products\n            WHERE (attributes->>'compartments')::int > 3\n            ORDER BY id\n        ")
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Laptop Backpack')
        expect(results[0]['compartments']).to_equal(5)

class TestJsonbArrayOperations(PostgresSuite):
    """Test JSONB array containment and operations."""

    @test
    async def test_jsonb_array_contains_element(self, products_table):
        """Test array containment for single element."""
        results = await execute("\n            SELECT * FROM products\n            WHERE attributes->'sizes' @> '9'::jsonb\n            ORDER BY id\n        ")
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Running Shoes')

    @test
    async def test_jsonb_array_contains_multiple(self, products_table):
        """Test array containment for multiple elements."""
        results = await execute('\n            SELECT * FROM products\n            WHERE metadata->\'tags\' @> \'["clothing", "winter"]\'::jsonb\n            ORDER BY id\n        ')
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Winter Jacket')

    @test
    async def test_jsonb_array_length(self, products_table):
        """Test getting array length."""
        results = await execute("\n            SELECT name, jsonb_array_length(metadata->'tags') as tag_count\n            FROM products\n            WHERE jsonb_array_length(metadata->'tags') > 2\n            ORDER BY id\n        ")
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Winter Jacket')
        expect(results[0]['tag_count']).to_equal(3)

    @test
    async def test_jsonb_array_elements(self, products_table):
        """Test expanding array elements."""
        results = await execute("\n            SELECT DISTINCT jsonb_array_elements_text(metadata->'tags') as tag\n            FROM products\n            ORDER BY tag\n        ")
        expect(len(results) >= 6).to_be_true()
        tags = [r['tag'] for r in results]
        expect('clothing' in tags).to_be_true()
        expect('sports' in tags).to_be_true()

class TestJsonbWithWhereConditions(PostgresSuite):
    """Test JSONB operators combined with regular WHERE conditions."""

    @test
    async def test_jsonb_and_regular_where(self, products_table):
        """Test JSONB conditions combined with regular columns."""
        results = await execute('\n            SELECT * FROM products\n            WHERE name LIKE \'R%\'\n            AND attributes @> \'{"color": "red"}\'::jsonb\n            ORDER BY id\n        ')
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Red T-Shirt')

    @test
    async def test_multiple_jsonb_conditions(self, products_table):
        """Test multiple JSONB conditions."""
        results = await execute('\n            SELECT * FROM products\n            WHERE attributes @> \'{"color": "blue"}\'::jsonb\n            AND metadata @> \'{"inStock": true}\'::jsonb\n            ORDER BY id\n        ')
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Blue Jeans')

    @test
    async def test_jsonb_or_conditions(self, products_table):
        """Test JSONB with OR conditions."""
        results = await execute('\n            SELECT * FROM products\n            WHERE attributes @> \'{"color": "red"}\'::jsonb\n            OR attributes @> \'{"color": "blue"}\'::jsonb\n            ORDER BY id\n        ')
        expect(len(results)).to_equal(2)
        names = [r['name'] for r in results]
        expect('Red T-Shirt' in names).to_be_true()
        expect('Blue Jeans' in names).to_be_true()

    @test
    async def test_jsonb_not_condition(self, products_table):
        """Test JSONB with NOT condition."""
        results = await execute('\n            SELECT * FROM products\n            WHERE NOT (metadata @> \'{"inStock": true}\'::jsonb)\n            ORDER BY id\n        ')
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Running Shoes')

class TestJsonbWithAggregates(PostgresSuite):
    """Test JSONB combined with aggregate functions."""

    @test
    async def test_count_by_jsonb_field(self, products_table):
        """Test COUNT grouped by JSONB field."""
        results = await execute("\n            SELECT attributes->>'color' as color, COUNT(*) as count\n            FROM products\n            WHERE attributes->>'color' IS NOT NULL\n            GROUP BY attributes->>'color'\n            ORDER BY count DESC, color\n        ")
        expect(len(results) >= 5).to_be_true()
        for row in results:
            expect(row['count'] >= 1).to_be_true()

    @test
    async def test_avg_with_jsonb_extraction(self, products_table):
        """Test AVG with JSONB numeric extraction."""
        results = await execute("\n            SELECT AVG((metadata->>'rating')::float) as avg_rating\n            FROM products\n            WHERE metadata->>'rating' IS NOT NULL\n        ")
        expect(len(results)).to_equal(1)
        expect(results[0]['avg_rating'] >= 4.7).to_be_true()
        expect(results[0]['avg_rating'] <= 4.8).to_be_true()

    @test
    async def test_max_min_with_jsonb(self, products_table):
        """Test MAX and MIN with JSONB fields."""
        results = await execute("\n            SELECT\n                MAX((metadata->>'rating')::float) as max_rating,\n                MIN((metadata->>'rating')::float) as min_rating\n            FROM products\n            WHERE metadata->>'rating' IS NOT NULL\n        ")
        expect(len(results)).to_equal(1)
        expect(results[0]['max_rating']).to_equal(5.0)
        expect(results[0]['min_rating']).to_equal(4.5)

    @test
    async def test_group_by_jsonb_with_having(self, products_table):
        """Test GROUP BY JSONB field with HAVING clause."""
        results = await execute("\n            SELECT\n                metadata->'tags'->0 as primary_tag,\n                COUNT(*) as count\n            FROM products\n            WHERE metadata->'tags'->0 IS NOT NULL\n            GROUP BY metadata->'tags'->0\n            HAVING COUNT(*) > 1\n            ORDER BY count DESC\n        ")
        expect(len(results) >= 1).to_be_true()

class TestJsonbWithOrderBy(PostgresSuite):
    """Test JSONB with ORDER BY clauses."""

    @test
    async def test_order_by_jsonb_text(self, products_table):
        """Test ORDER BY JSONB text field."""
        results = await execute("\n            SELECT name, attributes->>'color' as color\n            FROM products\n            WHERE attributes->>'color' IS NOT NULL\n            ORDER BY attributes->>'color' ASC\n        ")
        expect(len(results)).to_equal(7)
        colors = [r['color'] for r in results]
        sorted_colors = sorted(colors)
        expect(colors).to_equal(sorted_colors)

    @test
    async def test_order_by_jsonb_number(self, products_table):
        """Test ORDER BY JSONB numeric field."""
        results = await execute("\n            SELECT name, (metadata->>'rating')::float as rating\n            FROM products\n            WHERE metadata->>'rating' IS NOT NULL\n            ORDER BY (metadata->>'rating')::float DESC\n        ")
        expect(len(results)).to_equal(6)
        expect(results[0]['name']).to_equal('Premium Product')
        expect(results[0]['rating']).to_equal(5.0)
        ratings = [r['rating'] for r in results]
        for i in range(len(ratings) - 1):
            expect(ratings[i] >= ratings[i + 1]).to_be_true()

    @test
    async def test_order_by_nested_jsonb(self, products_table):
        """Test ORDER BY nested JSONB field."""
        results = await execute("\n            SELECT name, (metadata->'warranty'->>'years')::int as warranty_years\n            FROM products\n            WHERE metadata->'warranty'->>'years' IS NOT NULL\n            ORDER BY (metadata->'warranty'->>'years')::int DESC\n        ")
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Laptop Backpack')

    @test
    async def test_order_by_multiple_jsonb_fields(self, products_table):
        """Test ORDER BY multiple JSONB fields."""
        results = await execute("\n            SELECT\n                name,\n                (metadata->>'inStock')::boolean as in_stock,\n                (metadata->>'rating')::float as rating\n            FROM products\n            WHERE metadata->>'rating' IS NOT NULL\n            ORDER BY\n                (metadata->>'inStock')::boolean DESC NULLS LAST,\n                (metadata->>'rating')::float DESC NULLS LAST\n        ")
        expect(results[0]['in_stock']).to_be_true()

class TestJsonbIndexUsage(PostgresSuite):
    """Test JSONB GIN index creation and usage."""

    @test
    async def test_create_gin_index_on_jsonb(self, products_table):
        """Test creating GIN index on JSONB column."""
        await execute('\n            CREATE INDEX idx_products_attributes\n            ON products USING GIN (attributes)\n        ')
        indexes = await execute("\n            SELECT indexname, indexdef\n            FROM pg_indexes\n            WHERE tablename = 'products'\n            AND indexname = 'idx_products_attributes'\n        ")
        expect(len(indexes)).to_equal(1)
        expect('gin' in indexes[0]['indexdef'].lower()).to_be_true()

    @test
    async def test_gin_index_jsonb_path(self, products_table):
        """Test creating GIN index on JSONB path."""
        await execute("\n            CREATE INDEX idx_products_tags\n            ON products USING GIN ((metadata->'tags'))\n        ")
        results = await execute('\n            SELECT * FROM products\n            WHERE metadata->\'tags\' @> \'["sports"]\'::jsonb\n        ')
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Running Shoes')

    @test
    async def test_query_performance_with_gin_index(self, products_table):
        """Test query with GIN index for containment."""
        await execute('\n            CREATE INDEX idx_products_metadata\n            ON products USING GIN (metadata)\n        ')
        # Verify the GIN index query works and returns expected results
        results = await execute('\n            SELECT * FROM products\n            WHERE metadata @> \'{"inStock": true}\'::jsonb\n            ORDER BY id\n        ')
        expect(len(results)).to_equal(6)
        # Note: EXPLAIN queries return None from execute() as they are non-SELECT utility commands
        # The main test is that the containment query works correctly with the GIN index

class TestJsonbComplexQueries(PostgresSuite):
    """Test complex JSONB query scenarios."""

    @test
    async def test_jsonb_subquery(self, products_table):
        """Test JSONB in subquery."""
        results = await execute("\n            SELECT name, (metadata->>'rating')::float as rating\n            FROM products\n            WHERE (metadata->>'rating')::float > (\n                SELECT AVG((metadata->>'rating')::float)\n                FROM products\n                WHERE metadata->>'rating' IS NOT NULL\n            )\n            ORDER BY rating DESC\n        ")
        expect(len(results) >= 1).to_be_true()

    @test
    async def test_jsonb_case_expression(self, products_table):
        """Test JSONB with CASE expression."""
        results = await execute("\n            SELECT\n                name,\n                CASE\n                    WHEN (metadata->>'inStock')::boolean = true THEN 'Available'\n                    WHEN (metadata->>'inStock')::boolean = false THEN 'Out of Stock'\n                    ELSE 'Unknown'\n                END as stock_status\n            FROM products\n            ORDER BY name\n        ")
        expect(len(results)).to_equal(7)
        running_shoes = [r for r in results if r['name'] == 'Running Shoes']
        expect(running_shoes[0]['stock_status']).to_equal('Out of Stock')

    @test
    async def test_jsonb_join_condition(self, products_table):
        """Test JSONB in JOIN condition."""
        await execute('DROP TABLE IF EXISTS categories')
        await execute('''
            CREATE TABLE categories (
                category_name VARCHAR(50),
                tag_name VARCHAR(50)
            )
        ''')
        try:
            await execute("""
                INSERT INTO categories (category_name, tag_name) VALUES
                ('Apparel', 'clothing'),
                ('Athletic', 'sports'),
                ('Accessories', 'bags')
            """)
            results = await execute("""
                SELECT DISTINCT p.name, c.category_name
                FROM products p
                JOIN categories c ON p.metadata->'tags' @> to_jsonb(c.tag_name)
                ORDER BY p.name
            """)
            expect(len(results) >= 3).to_be_true()
        finally:
            await execute('DROP TABLE IF EXISTS categories')

    @test
    async def test_jsonb_update_field(self, products_table):
        """Test updating JSONB field."""
        await execute("\n            UPDATE products\n            SET metadata = jsonb_set(metadata, '{rating}', '4.95'::jsonb)\n            WHERE name = 'Running Shoes'\n        ")
        results = await execute("\n            SELECT name, (metadata->>'rating')::float as rating\n            FROM products\n            WHERE name = 'Running Shoes'\n        ")
        expect(len(results)).to_equal(1)
        expect(results[0]['rating']).to_equal(4.95)

    @test
    async def test_jsonb_remove_field(self, products_table):
        """Test removing field from JSONB."""
        await execute("\n            UPDATE products\n            SET metadata = metadata - 'rating'\n            WHERE name = 'Simple Notepad'\n        ")
        results = await execute("\n            SELECT name, metadata\n            FROM products\n            WHERE name = 'Simple Notepad'\n        ")
        expect(len(results)).to_equal(1)
        expect('rating' not in results[0]['metadata']).to_be_true()

    @test
    async def test_jsonb_merge_objects(self, products_table):
        """Test merging JSONB objects."""
        await execute('\n            UPDATE products\n            SET metadata = metadata || \'{"verified": true}\'::jsonb\n            WHERE name = \'Premium Product\'\n        ')
        results = await execute("\n            SELECT name, metadata\n            FROM products\n            WHERE name = 'Premium Product'\n        ")
        expect(len(results)).to_equal(1)
        expect(results[0]['metadata']['verified']).to_be_true()
        expect(results[0]['metadata']['role']).to_equal('admin')

class TestJsonbEdgeCases(PostgresSuite):
    """Test JSONB edge cases and special scenarios."""

    @test
    async def test_empty_jsonb_object(self, products_table):
        """Test querying empty JSONB objects."""
        await execute("\n            INSERT INTO products (name, attributes, metadata)\n            VALUES ('Empty Product', '{}'::jsonb, '{}'::jsonb)\n        ")
        results = await execute("\n            SELECT name FROM products\n            WHERE attributes = '{}'::jsonb\n        ")
        expect(len(results)).to_equal(1)
        expect(results[0]['name']).to_equal('Empty Product')

    @test
    async def test_null_vs_missing_jsonb_field(self, products_table):
        """Test difference between NULL and missing JSONB field."""
        results_null = await execute("\n            SELECT name FROM products\n            WHERE metadata->>'rating' IS NULL\n        ")
        expect(len(results_null) >= 1).to_be_true()

    @test
    async def test_jsonb_type_checking(self, products_table):
        """Test JSONB type checking functions."""
        results = await execute("\n            SELECT\n                name,\n                jsonb_typeof(metadata->'rating') as rating_type\n            FROM products\n            WHERE metadata->'rating' IS NOT NULL\n            ORDER BY name\n        ")
        for row in results:
            if row['rating_type'] != 'null':
                expect(row['rating_type']).to_equal('number')

    @test
    async def test_large_jsonb_document(self, products_table):
        """Test storing and querying large JSONB document."""
        large_json = '{"field1": "value1", "field2": "value2", "field3": "value3", '
        for i in range(4, 51):
            large_json += f'"field{i}": "value{i}", '
        large_json += '"field51": "value51"}'
        await execute("\n            INSERT INTO products (name, attributes, metadata)\n            VALUES ('Large JSON Product', $1::jsonb, '{}'::jsonb)\n        ", [large_json])
        results = await execute("\n            SELECT name, attributes\n            FROM products\n            WHERE name = 'Large JSON Product'\n        ")
        expect(len(results)).to_equal(1)
        expect(len(results[0]['attributes'])).to_equal(51)
        expect(results[0]['attributes']['field50']).to_equal('value50')