"""
Integration tests for Many-to-Many relationship functionality.

Tests M2M operations with real PostgreSQL database.
"""
from cclab.probe import TestSuite, expect, fixture, test
try:
    from cclab._pg import titan as _engine
except ImportError:
    _engine = None

@fixture
async def m2m_tables():
    """Create posts and tags tables with a join table for M2M testing."""
    from cclab.titan import execute, insert_one
    await execute('\n        CREATE TABLE IF NOT EXISTS posts (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255) NOT NULL,\n            content TEXT\n        )\n    ')
    await execute('\n        CREATE TABLE IF NOT EXISTS tags (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(100) NOT NULL UNIQUE\n        )\n    ')
    await execute('\n        CREATE TABLE IF NOT EXISTS post_tags (\n            post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,\n            tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,\n            PRIMARY KEY (post_id, tag_id)\n        )\n    ')
    posts = [{'title': 'First Post', 'content': 'Content 1'}, {'title': 'Second Post', 'content': 'Content 2'}, {'title': 'Third Post', 'content': 'Content 3'}]
    post_ids = []
    for post in posts:
        result = await insert_one('posts', post)
        post_ids.append(result['id'])
    tags = [{'name': 'python'}, {'name': 'rust'}, {'name': 'database'}, {'name': 'orm'}]
    tag_ids = []
    for tag in tags:
        result = await insert_one('tags', tag)
        tag_ids.append(result['id'])
    yield {'post_ids': post_ids, 'tag_ids': tag_ids}

class TestM2MAddRelation(TestSuite):
    """Test adding M2M relations."""

    @test
    async def test_add_single_relation(self, m2m_tables):
        """Test adding a single M2M relation."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_id = m2m_tables['tag_ids'][0]
        await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        exists = await _engine.m2m_has_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        expect(exists).to_be(True)

    @test
    async def test_add_multiple_relations(self, m2m_tables):
        """Test adding multiple M2M relations."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_ids = m2m_tables['tag_ids'][:3]
        for tag_id in tag_ids:
            await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        count = await _engine.m2m_count_related('post_tags', 'post_id', 'tag_id', 'tags', post_id, 'id', 'id')
        expect(count).to_equal(3)

    @test
    async def test_add_duplicate_relation_idempotent(self, m2m_tables):
        """Test that adding duplicate relation is idempotent."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_id = m2m_tables['tag_ids'][0]
        await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        count = await _engine.m2m_count_related('post_tags', 'post_id', 'tag_id', 'tags', post_id, 'id', 'id')
        expect(count).to_equal(1)

class TestM2MRemoveRelation(TestSuite):
    """Test removing M2M relations."""

    @test
    async def test_remove_single_relation(self, m2m_tables):
        """Test removing a single M2M relation."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_id = m2m_tables['tag_ids'][0]
        await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        affected = await _engine.m2m_remove_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        expect(affected).to_equal(1)
        exists = await _engine.m2m_has_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        expect(exists).to_be(False)

    @test
    async def test_remove_nonexistent_relation(self, m2m_tables):
        """Test removing a nonexistent relation returns 0."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_id = m2m_tables['tag_ids'][0]
        affected = await _engine.m2m_remove_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        expect(affected).to_equal(0)

class TestM2MClearRelations(TestSuite):
    """Test clearing all M2M relations."""

    @test
    async def test_clear_all_relations(self, m2m_tables):
        """Test clearing all relations for a source."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_ids = m2m_tables['tag_ids']
        for tag_id in tag_ids:
            await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        affected = await _engine.m2m_clear_relations('post_tags', 'post_id', 'tag_id', 'tags', post_id, 'id', 'id')
        expect(affected).to_equal(len(tag_ids))
        count = await _engine.m2m_count_related('post_tags', 'post_id', 'tag_id', 'tags', post_id, 'id', 'id')
        expect(count).to_equal(0)

class TestM2MFetchRelated(TestSuite):
    """Test fetching M2M related records."""

    @test
    async def test_fetch_all_related(self, m2m_tables):
        """Test fetching all related records."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_ids = m2m_tables['tag_ids'][:2]
        for tag_id in tag_ids:
            await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        results = await _engine.m2m_fetch_related('post_tags', 'post_id', 'tag_id', 'tags', post_id, None, None, None, 'id', 'id')
        expect(len(results)).to_equal(2)
        names = [r['name'] for r in results]
        expect('python').to_be_in(names)
        expect('rust').to_be_in(names)

    @test
    async def test_fetch_with_select_columns(self, m2m_tables):
        """Test fetching with specific columns."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_id = m2m_tables['tag_ids'][0]
        await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        results = await _engine.m2m_fetch_related('post_tags', 'post_id', 'tag_id', 'tags', post_id, ['name'], None, None, 'id', 'id')
        expect(len(results)).to_equal(1)
        expect('name').to_be_in(results[0])

    @test
    async def test_fetch_with_order_by(self, m2m_tables):
        """Test fetching with ordering."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_ids = m2m_tables['tag_ids']
        for tag_id in tag_ids:
            await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        results = await _engine.m2m_fetch_related('post_tags', 'post_id', 'tag_id', 'tags', post_id, None, [('name', 'asc')], None, 'id', 'id')
        names = [r['name'] for r in results]
        expect(names).to_equal(sorted(names))

    @test
    async def test_fetch_with_limit(self, m2m_tables):
        """Test fetching with limit."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_ids = m2m_tables['tag_ids']
        for tag_id in tag_ids:
            await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        results = await _engine.m2m_fetch_related('post_tags', 'post_id', 'tag_id', 'tags', post_id, None, None, 2, 'id', 'id')
        expect(len(results)).to_equal(2)

class TestM2MSetRelations(TestSuite):
    """Test setting M2M relations atomically."""

    @test
    async def test_set_replaces_existing(self, m2m_tables):
        """Test that set replaces all existing relations."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        old_tags = m2m_tables['tag_ids'][:2]
        new_tags = m2m_tables['tag_ids'][2:]
        for tag_id in old_tags:
            await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        await _engine.m2m_set_relations('post_tags', 'post_id', 'tag_id', 'tags', post_id, new_tags, 'id', 'id')
        for tag_id in old_tags:
            exists = await _engine.m2m_has_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
            expect(exists).to_be(False)
        for tag_id in new_tags:
            exists = await _engine.m2m_has_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
            expect(exists).to_be(True)

    @test
    async def test_set_empty_clears_all(self, m2m_tables):
        """Test that setting empty list clears all relations."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_ids = m2m_tables['tag_ids']
        for tag_id in tag_ids:
            await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        await _engine.m2m_set_relations('post_tags', 'post_id', 'tag_id', 'tags', post_id, [], 'id', 'id')
        count = await _engine.m2m_count_related('post_tags', 'post_id', 'tag_id', 'tags', post_id, 'id', 'id')
        expect(count).to_equal(0)

class TestM2MCreateJoinTable(TestSuite):
    """Test automatic join table creation."""

    @test
    async def test_create_join_table(self):
        """Test creating a join table automatically."""
        if _engine is None:
            return  # Skip: Rust engine not available
        from cclab.titan import execute
        await execute('\n            CREATE TABLE IF NOT EXISTS users_test (\n                id SERIAL PRIMARY KEY,\n                name VARCHAR(100)\n            )\n        ')
        await execute('\n            CREATE TABLE IF NOT EXISTS groups_test (\n                id SERIAL PRIMARY KEY,\n                name VARCHAR(100)\n            )\n        ')
        await _engine.m2m_create_join_table('user_groups_test', 'user_id', 'group_id', 'users_test', 'groups_test', 'id', 'id')
        await execute("INSERT INTO users_test (name) VALUES ('Alice')")
        await execute("INSERT INTO groups_test (name) VALUES ('Admins')")
        await execute('INSERT INTO user_groups_test (user_id, group_id) VALUES (1, 1)')
        result = await execute('SELECT * FROM user_groups_test')
        expect(len(result)).to_equal(1)

class TestM2MCountRelated(TestSuite):
    """Test counting M2M relations."""

    @test
    async def test_count_zero_relations(self, m2m_tables):
        """Test counting when there are no relations."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        count = await _engine.m2m_count_related('post_tags', 'post_id', 'tag_id', 'tags', post_id, 'id', 'id')
        expect(count).to_equal(0)

    @test
    async def test_count_multiple_relations(self, m2m_tables):
        """Test counting multiple relations."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_ids = m2m_tables['tag_ids']
        for tag_id in tag_ids:
            await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        count = await _engine.m2m_count_related('post_tags', 'post_id', 'tag_id', 'tags', post_id, 'id', 'id')
        expect(count).to_equal(len(tag_ids))

class TestM2MHasRelation(TestSuite):
    """Test checking existence of M2M relations."""

    @test
    async def test_has_existing_relation(self, m2m_tables):
        """Test checking for existing relation."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_id = m2m_tables['tag_ids'][0]
        await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        exists = await _engine.m2m_has_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        expect(exists).to_be(True)

    @test
    async def test_has_nonexistent_relation(self, m2m_tables):
        """Test checking for nonexistent relation."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_id = m2m_tables['tag_ids'][0]
        exists = await _engine.m2m_has_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        expect(exists).to_be(False)

class TestM2MBidirectional(TestSuite):
    """Test bidirectional M2M relationships."""

    @test
    async def test_bidirectional_relations(self, m2m_tables):
        """Test that relations work in both directions."""
        if _engine is None:
            return  # Skip: Rust engine not available
        post_id = m2m_tables['post_ids'][0]
        tag_id = m2m_tables['tag_ids'][0]
        await _engine.m2m_add_relation('post_tags', 'post_id', 'tag_id', 'tags', post_id, tag_id, 'id', 'id')
        results = await _engine.m2m_fetch_related('post_tags', 'tag_id', 'post_id', 'posts', tag_id, None, None, None, 'id', 'id')
        expect(len(results)).to_equal(1)
        expect(results[0]['id']).to_equal(post_id)
        expect(results[0]['title']).to_equal('First Post')