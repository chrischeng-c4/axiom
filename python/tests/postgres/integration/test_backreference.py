"""
Integration tests for PostgreSQL BackReference descriptor.

These tests verify:
- BackReference.fetch_all() - Fetch all related rows
- BackReference.fetch_one() - Fetch first related row
- BackReference.count() - Count related rows
- BackReference behavior with no results
- BackReference.ref_value property
- Class vs instance access patterns
"""
from cclab.titan import execute, insert_one, BackReference, BackReferenceQuery
from cclab.probe import expect, test
from tests.postgres.base import PostgresSuite

class TestBackreference(PostgresSuite):

    @test
    async def test_backreference_fetch_all(self):
        """Test fetching all related rows via BackReference."""
        await execute('\n        CREATE TABLE br_users_1 (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(255) NOT NULL\n        )\n    ')
        await execute('\n        CREATE TABLE br_posts_1 (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255) NOT NULL,\n            user_id INTEGER REFERENCES br_users_1(id) ON DELETE CASCADE\n        )\n    ')
        await execute('INSERT INTO br_users_1 (id, name) VALUES (1, $1)', ['Alice'])
        user_id = 1
        await execute('INSERT INTO br_posts_1 (title, user_id) VALUES ($1, $2)', ['First Post', user_id])
        await execute('INSERT INTO br_posts_1 (title, user_id) VALUES ($1, $2)', ['Second Post', user_id])
        await execute('INSERT INTO br_posts_1 (title, user_id) VALUES ($1, $2)', ['Third Post', user_id])
        back_ref_query = BackReferenceQuery('br_posts_1', 'user_id', 'id', user_id)
        posts = await back_ref_query.fetch_all()
        expect(len(posts)).to_equal(3)
        titles = {post['title'] for post in posts}
        expect('First Post' in titles).to_be_true()
        expect('Second Post' in titles).to_be_true()
        expect('Third Post' in titles).to_be_true()
        for post in posts:
            expect(post['user_id']).to_equal(user_id)

    @test
    async def test_backreference_fetch_one(self):
        """Test fetching first related row via BackReference."""
        await execute('\n        CREATE TABLE br_users_2 (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(255) NOT NULL\n        )\n    ')
        await execute('\n        CREATE TABLE br_posts_2 (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255) NOT NULL,\n            user_id INTEGER REFERENCES br_users_2(id),\n            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP\n        )\n    ')
        await execute('INSERT INTO br_users_2 (id, name) VALUES (2, $1)', ['Bob'])
        user_id = 2
        await execute('INSERT INTO br_posts_2 (title, user_id) VALUES ($1, $2)', ['First Post', user_id])
        await execute('INSERT INTO br_posts_2 (title, user_id) VALUES ($1, $2)', ['Second Post', user_id])
        back_ref_query = BackReferenceQuery('br_posts_2', 'user_id', 'id', user_id)
        first_post = await back_ref_query.fetch_one()
        expect(first_post is not None).to_be_true()
        expect(first_post['user_id']).to_equal(user_id)
        expect(first_post['title'] in ['First Post', 'Second Post']).to_be_true()

    @test
    async def test_backreference_count(self):
        """Test counting related rows via BackReference."""
        await execute('\n        CREATE TABLE br_users_3 (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(255) NOT NULL\n        )\n    ')
        await execute('\n        CREATE TABLE br_posts_3 (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255) NOT NULL,\n            user_id INTEGER REFERENCES br_users_3(id)\n        )\n    ')
        await execute('INSERT INTO br_users_3 (id, name) VALUES (3, $1)', ['Charlie'])
        user_id = 3
        await execute('INSERT INTO br_posts_3 (title, user_id) VALUES ($1, $2)', ['Post 1', user_id])
        await execute('INSERT INTO br_posts_3 (title, user_id) VALUES ($1, $2)', ['Post 2', user_id])
        await execute('INSERT INTO br_posts_3 (title, user_id) VALUES ($1, $2)', ['Post 3', user_id])
        await execute('INSERT INTO br_posts_3 (title, user_id) VALUES ($1, $2)', ['Post 4', user_id])
        back_ref_query = BackReferenceQuery('br_posts_3', 'user_id', 'id', user_id)
        post_count = await back_ref_query.count()
        expect(post_count).to_equal(4)

    @test
    async def test_backreference_no_results(self):
        """Test BackReference with no related rows returns empty list."""
        await execute('\n        CREATE TABLE br_users_4 (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(255) NOT NULL\n        )\n    ')
        await execute('\n        CREATE TABLE br_posts_4 (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255) NOT NULL,\n            user_id INTEGER REFERENCES br_users_4(id)\n        )\n    ')
        await execute('INSERT INTO br_users_4 (id, name) VALUES (4, $1)', ['David'])
        user_id = 4
        back_ref_query = BackReferenceQuery('br_posts_4', 'user_id', 'id', user_id)
        posts = await back_ref_query.fetch_all()
        expect(len(posts)).to_equal(0)
        expect(posts).to_equal([])
        first_post = await back_ref_query.fetch_one()
        expect(first_post).to_be_none()
        post_count = await back_ref_query.count()
        expect(post_count).to_equal(0)

    @test
    async def test_backreference_ref_value(self):
        """Test BackReferenceQuery.ref_value property returns correct value."""
        await execute('\n        CREATE TABLE br_users_5 (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(255) NOT NULL\n        )\n    ')
        await execute('\n        CREATE TABLE br_posts_5 (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255) NOT NULL,\n            user_id INTEGER REFERENCES br_users_5(id)\n        )\n    ')
        await execute('INSERT INTO br_users_5 (id, name) VALUES (5, $1)', ['Eve'])
        user_id = 5
        back_ref_query = BackReferenceQuery('br_posts_5', 'user_id', 'id', user_id)
        expect(back_ref_query.ref_value).to_equal(user_id)
        expect(back_ref_query.ref_value).to_equal(5)

    @test
    async def test_backreference_class_access(self):
        """Test accessing BackReference descriptor on class returns descriptor itself."""

        class MockUser:
            posts = BackReference('posts', 'user_id', 'id')
        descriptor = MockUser.posts
        expect(isinstance(descriptor, BackReference)).to_be_true()
        expect(descriptor.source_table).to_equal('posts')
        expect(descriptor.source_column).to_equal('user_id')
        expect(descriptor.target_column).to_equal('id')

    @test
    async def test_backreference_instance_access(self):
        """Test accessing BackReference descriptor on instance returns BackReferenceQuery."""
        await execute('\n        CREATE TABLE br_users_6 (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(255) NOT NULL\n        )\n    ')
        await execute('\n        CREATE TABLE br_posts_6 (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255) NOT NULL,\n            user_id INTEGER REFERENCES br_users_6(id)\n        )\n    ')

        class MockUser:
            posts = BackReference('posts', 'user_id', 'id')

            def __init__(self, user_id):
                self._data = {'id': user_id}
        user = MockUser(user_id=6)
        query = user.posts
        expect(isinstance(query, BackReferenceQuery)).to_be_true()
        expect(query.source_table).to_equal('posts')
        expect(query.source_column).to_equal('user_id')
        expect(query.target_column).to_equal('id')
        expect(query.ref_value).to_equal(6)

    @test
    async def test_backreference_null_ref_value(self):
        """Test BackReference behavior when ref_value is None."""
        back_ref_query = BackReferenceQuery('posts', 'user_id', 'id', None)
        posts = await back_ref_query.fetch_all()
        expect(posts).to_equal([])
        first_post = await back_ref_query.fetch_one()
        expect(first_post).to_be_none()
        post_count = await back_ref_query.count()
        expect(post_count).to_equal(0)

    @test
    async def test_backreference_multiple_users(self):
        """Test BackReference correctly filters by different user IDs."""
        await execute('\n        CREATE TABLE br_users_7 (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(255) NOT NULL\n        )\n    ')
        await execute('\n        CREATE TABLE br_posts_7 (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255) NOT NULL,\n            user_id INTEGER REFERENCES br_users_7(id)\n        )\n    ')
        await execute('INSERT INTO br_users_7 (id, name) VALUES (7, $1)', ['User1'])
        await execute('INSERT INTO br_users_7 (id, name) VALUES (8, $1)', ['User2'])
        await execute('INSERT INTO br_posts_7 (title, user_id) VALUES ($1, $2)', ['User1 Post 1', 7])
        await execute('INSERT INTO br_posts_7 (title, user_id) VALUES ($1, $2)', ['User1 Post 2', 7])
        await execute('INSERT INTO br_posts_7 (title, user_id) VALUES ($1, $2)', ['User2 Post 1', 8])
        await execute('INSERT INTO br_posts_7 (title, user_id) VALUES ($1, $2)', ['User2 Post 2', 8])
        await execute('INSERT INTO br_posts_7 (title, user_id) VALUES ($1, $2)', ['User2 Post 3', 8])
        user7_query = BackReferenceQuery('br_posts_7', 'user_id', 'id', 7)
        user7_posts = await user7_query.fetch_all()
        expect(len(user7_posts)).to_equal(2)
        user8_query = BackReferenceQuery('br_posts_7', 'user_id', 'id', 8)
        user8_posts = await user8_query.fetch_all()
        expect(len(user8_posts)).to_equal(3)
        for post in user7_posts:
            expect(post['user_id']).to_equal(7)
            expect(post['title'].startswith('User1')).to_be_true()
        for post in user8_posts:
            expect(post['user_id']).to_equal(8)
            expect(post['title'].startswith('User2')).to_be_true()

    @test
    async def test_backreference_repr(self):
        """Test __repr__ method of BackReference and BackReferenceQuery."""
        back_ref = BackReference('posts', 'user_id', 'id')
        repr_str = repr(back_ref)
        expect('posts' in repr_str).to_be_true()
        expect('user_id' in repr_str).to_be_true()
        expect('id' in repr_str).to_be_true()
        query = BackReferenceQuery('posts', 'user_id', 'id', 123)
        query_repr = repr(query)
        expect('posts' in query_repr).to_be_true()
        expect('user_id' in query_repr).to_be_true()
        expect('123' in query_repr).to_be_true()

    @test
    async def test_backreference_custom_target_column(self):
        """Test BackReference with custom target_column (not 'id')."""
        await execute('\n        CREATE TABLE br_users_8 (\n            user_uuid VARCHAR(36) PRIMARY KEY,\n            name VARCHAR(255) NOT NULL\n        )\n    ')
        await execute('\n        CREATE TABLE br_posts_8 (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255) NOT NULL,\n            author_uuid VARCHAR(36) REFERENCES br_users_8(user_uuid)\n        )\n    ')
        user_uuid = '550e8400-e29b-41d4-a716-446655440000'
        await execute('INSERT INTO br_users_8 (user_uuid, name) VALUES ($1, $2)', [user_uuid, 'Frank'])
        await execute('INSERT INTO br_posts_8 (title, author_uuid) VALUES ($1, $2)', ['Post 1', user_uuid])
        await execute('INSERT INTO br_posts_8 (title, author_uuid) VALUES ($1, $2)', ['Post 2', user_uuid])
        back_ref_query = BackReferenceQuery('br_posts_8', 'author_uuid', 'user_uuid', user_uuid)
        posts = await back_ref_query.fetch_all()
        expect(len(posts)).to_equal(2)
        for post in posts:
            expect(post['author_uuid']).to_equal(user_uuid)
        post_count = await back_ref_query.count()
        expect(post_count).to_equal(2)