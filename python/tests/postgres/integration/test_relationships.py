"""
Integration tests for PostgreSQL foreign key relationships.

These tests verify:
- Foreign key schema introspection (get_foreign_keys)
- Querying by foreign key (find_by_foreign_key)
- ForeignKeyProxy functionality
"""
from cclab.titan import execute, get_foreign_keys, find_by_foreign_key, Column, ForeignKeyProxy
from cclab.probe import expect, test
from tests.postgres.base import PostgresSuite

class TestRelationships(PostgresSuite):

    @test
    async def test_get_foreign_keys_basic(self):
        """Test retrieving foreign key information from schema."""
        await execute('\n        CREATE TABLE users (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(255) NOT NULL\n        )\n    ')
        await execute('\n        CREATE TABLE posts (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255) NOT NULL,\n            author_id INTEGER REFERENCES users(id) ON DELETE CASCADE\n        )\n    ')
        foreign_keys = await get_foreign_keys('posts')
        expect(len(foreign_keys)).to_equal(1)
        fk = foreign_keys[0]
        expect('author_id' in fk['columns']).to_be_true()
        expect(fk['referenced_table']).to_equal('users')
        expect(fk['on_delete']).to_equal('CASCADE')

    @test
    async def test_get_foreign_keys_multiple(self):
        """Test retrieving multiple foreign keys from a table."""
        await execute('CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255))')
        await execute('CREATE TABLE categories (id SERIAL PRIMARY KEY, title VARCHAR(255))')
        await execute('\n        CREATE TABLE posts (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255),\n            author_id INTEGER REFERENCES users(id) ON DELETE CASCADE,\n            category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL\n        )\n    ')
        foreign_keys = await get_foreign_keys('posts')
        expect(len(foreign_keys)).to_equal(2)
        fk_tables = {fk['referenced_table'] for fk in foreign_keys}
        expect('users' in fk_tables).to_be_true()
        expect('categories' in fk_tables).to_be_true()
        for fk in foreign_keys:
            if fk['referenced_table'] == 'users':
                expect(fk['on_delete']).to_equal('CASCADE')
            elif fk['referenced_table'] == 'categories':
                expect(fk['on_delete']).to_equal('SET NULL')

    @test
    async def test_get_foreign_keys_no_foreign_keys(self):
        """Test get_foreign_keys on table without foreign keys."""
        await execute('\n        CREATE TABLE users (\n            id SERIAL PRIMARY KEY,\n            name VARCHAR(255)\n        )\n    ')
        foreign_keys = await get_foreign_keys('users')
        expect(len(foreign_keys)).to_equal(0)

    @test
    async def test_find_by_foreign_key(self):
        """Test querying related objects via foreign key."""
        await execute('CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255))')
        await execute('\n        CREATE TABLE posts (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255),\n            author_id INTEGER REFERENCES users(id)\n        )\n    ')
        await execute('INSERT INTO users (id, name) VALUES (1, $1)', ['Alice'])
        user_id = 1
        await execute('INSERT INTO posts (title, author_id) VALUES ($1, $2)', ['First Post', user_id])
        author = await find_by_foreign_key('users', 'id', user_id)
        expect(author).not_to_be_none()
        expect(author['name']).to_equal('Alice')
        expect(author['id']).to_equal(user_id)

    @test
    async def test_find_by_foreign_key_not_found(self):
        """Test find_by_foreign_key when no row matches."""
        await execute('CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255))')
        result = await find_by_foreign_key('users', 'id', 99999)
        expect(result).to_be_none()

    @test
    async def test_column_foreign_key_parameter(self):
        """Test Column class accepts foreign_key parameter."""
        col = Column(foreign_key='users.id', nullable=False)
        expect(col.foreign_key).to_equal('users.id')
        expect(col.nullable).to_equal(False)
        repr_str = repr(col)
        expect("foreign_key='users.id'" in repr_str).to_be_true()

    @test
    async def test_foreign_key_proxy_basic(self):
        """Test ForeignKeyProxy basic functionality."""
        await execute('CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255))')
        await execute('\n        CREATE TABLE posts (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255),\n            author_id INTEGER REFERENCES users(id)\n        )\n    ')
        await execute('INSERT INTO users (id, name) VALUES (2, $1)', ['Bob'])
        user_id = 2
        proxy = ForeignKeyProxy('users', 'id', user_id)
        expect(proxy.ref).to_equal(user_id)
        expect(proxy.id).to_equal(user_id)
        expect(proxy.column_value).to_equal(user_id)
        expect(proxy.is_fetched).to_be_false()
        author = await proxy.fetch()
        expect(author).not_to_be_none()
        expect(author['name']).to_equal('Bob')
        expect(proxy.is_fetched).to_be_true()
        author2 = await proxy.fetch()
        expect(author2).to_equal(author)

    @test
    async def test_foreign_key_proxy_not_found(self):
        """Test ForeignKeyProxy when referenced row doesn't exist."""
        await execute('CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255))')
        proxy = ForeignKeyProxy('users', 'id', 99999)
        result = await proxy.fetch()
        expect(result).to_be_none()
        expect(proxy.is_fetched).to_be_true()

    @test
    async def test_foreign_key_cascade_delete(self):
        """Test ON DELETE CASCADE behavior."""
        await execute('CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255))')
        await execute('\n        CREATE TABLE posts (\n            id SERIAL PRIMARY KEY,\n            title VARCHAR(255),\n            author_id INTEGER REFERENCES users(id) ON DELETE CASCADE\n        )\n    ')
        await execute('INSERT INTO users (id, name) VALUES (3, $1)', ['Charlie'])
        user_id = 3
        await execute('INSERT INTO posts (title, author_id) VALUES ($1, $2)', ['Post 1', user_id])
        await execute('INSERT INTO posts (title, author_id) VALUES ($1, $2)', ['Post 2', user_id])
        posts_before = await execute('SELECT * FROM posts WHERE author_id = $1', [user_id])
        expect(len(posts_before)).to_equal(2)
        await execute('DELETE FROM users WHERE id = $1', [user_id])
        posts_after = await execute('SELECT * FROM posts WHERE author_id = $1', [user_id])
        expect(len(posts_after)).to_equal(0)

    @test
    async def test_composite_foreign_key(self):
        """Test composite foreign keys (multiple columns)."""
        await execute('\n        CREATE TABLE compound_keys (\n            key1 INTEGER,\n            key2 INTEGER,\n            value VARCHAR(255),\n            PRIMARY KEY (key1, key2)\n        )\n    ')
        await execute('\n        CREATE TABLE ref_table (\n            id SERIAL PRIMARY KEY,\n            ref_key1 INTEGER,\n            ref_key2 INTEGER,\n            FOREIGN KEY (ref_key1, ref_key2) REFERENCES compound_keys(key1, key2)\n        )\n    ')
        foreign_keys = await get_foreign_keys('ref_table')
        expect(len(foreign_keys)).to_equal(1)
        fk = foreign_keys[0]
        expect(len(fk['columns'])).to_equal(2)
        expect('ref_key1' in fk['columns']).to_be_true()
        expect('ref_key2' in fk['columns']).to_be_true()
        expect(fk['referenced_table']).to_equal('compound_keys')
        expect(len(fk['referenced_columns'])).to_equal(2)