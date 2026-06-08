"""Integration tests for cascade-aware delete operations.

This module tests cascade delete operations that handle foreign key relationships
and cascade rules via the PostgreSQL database.
"""
from cclab.titan import execute
from cclab.probe import expect, fixture, test
from tests.postgres.base import PostgresSuite
class TestCascadeDelete(PostgresSuite):

    @test
    async def test_delete_with_cascade_basic(self, setup_cascade_tables):
        """Test basic cascade delete functionality."""
        await execute("\n        INSERT INTO cascade_users (name, email)\n        VALUES ('Alice', 'alice@example.com')\n    ")
        user_result = await execute("SELECT id FROM cascade_users WHERE name = 'Alice'")
        user_id = user_result[0]['id']
        await execute(f"\n        INSERT INTO cascade_posts (user_id, title, content)\n        VALUES ({user_id}, 'My First Post', 'Hello world!')\n    ")
        post_result = await execute('SELECT id FROM cascade_posts WHERE user_id = $1', [user_id])
        post_id = post_result[0]['id']
        found_user = await execute('SELECT * FROM cascade_users WHERE id = $1', [user_id])
        expect(len(found_user)).to_equal(1)
        found_post = await execute('SELECT * FROM cascade_posts WHERE id = $1', [post_id])
        expect(len(found_post)).to_equal(1)
        await execute('DELETE FROM cascade_users WHERE id = $1', [user_id])
        found_user = await execute('SELECT * FROM cascade_users WHERE id = $1', [user_id])
        expect(len(found_user)).to_equal(0)
        found_post = await execute('SELECT * FROM cascade_posts WHERE id = $1', [post_id])
        expect(len(found_post)).to_equal(0)

    @test
    async def test_delete_with_cascade_restrict_violation(self, setup_cascade_tables):
        """Test that RESTRICT constraint prevents deletion."""
        await execute("\n        INSERT INTO cascade_users (name, email)\n        VALUES ('Bob', 'bob@example.com')\n    ")
        user_result = await execute("SELECT id FROM cascade_users WHERE name = 'Bob'")
        user_id = user_result[0]['id']
        await execute(f"\n        INSERT INTO cascade_posts (user_id, title, content)\n        VALUES ({user_id}, 'Bob''s Post', 'Some content')\n    ")
        post_result = await execute('SELECT id FROM cascade_posts WHERE user_id = $1', [user_id])
        post_id = post_result[0]['id']
        await execute(f"\n        INSERT INTO cascade_comments (post_id, user_id, text)\n        VALUES ({post_id}, {user_id}, 'Great post!')\n    ")
        try:
            await execute('DELETE FROM cascade_posts WHERE id = $1', [post_id])
            raise AssertionError('Expected exception')
        except Exception as e:
            expect('violate' in str(e).lower() or 'restrict' in str(e).lower()).to_be_true()
        found_post = await execute('SELECT * FROM cascade_posts WHERE id = $1', [post_id])
        expect(len(found_post)).to_equal(1)

    @test
    async def test_delete_with_cascade_set_null(self, setup_cascade_tables):
        """Test SET NULL cascade rule."""
        await execute("\n        INSERT INTO cascade_users (name, email)\n        VALUES ('Charlie', 'charlie@example.com'), ('Dave', 'dave@example.com')\n    ")
        user_result = await execute("SELECT id FROM cascade_users WHERE name = 'Charlie'")
        user_id = user_result[0]['id']
        other_user_result = await execute("SELECT id FROM cascade_users WHERE name = 'Dave'")
        other_user_id = other_user_result[0]['id']
        await execute(f"\n        INSERT INTO cascade_posts (user_id, title, content)\n        VALUES ({other_user_id}, 'Post by Dave', 'Content')\n    ")
        post_result = await execute('SELECT id FROM cascade_posts WHERE user_id = $1', [other_user_id])
        post_id = post_result[0]['id']
        await execute(f"\n        INSERT INTO cascade_comments (post_id, user_id, text)\n        VALUES ({post_id}, {user_id}, 'Nice post!')\n    ")
        comment_result = await execute('SELECT id FROM cascade_comments WHERE user_id = $1', [user_id])
        comment_id = comment_result[0]['id']
        await execute('DELETE FROM cascade_users WHERE id = $1', [user_id])
        found_comment = await execute('SELECT * FROM cascade_comments WHERE id = $1', [comment_id])
        expect(len(found_comment)).to_equal(1)
        expect(found_comment[0]['user_id']).to_be_none()
        expect(found_comment[0]['text']).to_equal('Nice post!')

    @test
    async def test_delete_checked_allows_cascade(self, setup_cascade_tables):
        """Test cascade delete allows CASCADE rule."""
        await execute("\n        INSERT INTO cascade_users (name, email)\n        VALUES ('Eve', 'eve@example.com')\n    ")
        user_result = await execute("SELECT id FROM cascade_users WHERE name = 'Eve'")
        user_id = user_result[0]['id']
        await execute(f"\n        INSERT INTO cascade_posts (user_id, title, content)\n        VALUES ({user_id}, 'Eve''s Post', 'Content')\n    ")
        post_result = await execute('SELECT id FROM cascade_posts WHERE user_id = $1', [user_id])
        post_id = post_result[0]['id']
        await execute('DELETE FROM cascade_users WHERE id = $1', [user_id])
        found_user = await execute('SELECT * FROM cascade_users WHERE id = $1', [user_id])
        expect(len(found_user)).to_equal(0)
        found_post = await execute('SELECT * FROM cascade_posts WHERE id = $1', [post_id])
        expect(len(found_post)).to_equal(0)

    @test
    async def test_delete_checked_blocks_restrict(self, setup_cascade_tables):
        """Test that RESTRICT blocks deletion."""
        await execute("\n        INSERT INTO cascade_users (name, email)\n        VALUES ('Frank', 'frank@example.com')\n    ")
        user_result = await execute("SELECT id FROM cascade_users WHERE name = 'Frank'")
        user_id = user_result[0]['id']
        await execute(f"\n        INSERT INTO cascade_posts (user_id, title, content)\n        VALUES ({user_id}, 'Frank''s Post', 'Content')\n    ")
        post_result = await execute('SELECT id FROM cascade_posts WHERE user_id = $1', [user_id])
        post_id = post_result[0]['id']
        await execute(f"\n        INSERT INTO cascade_comments (post_id, user_id, text)\n        VALUES ({post_id}, {user_id}, 'Comment')\n    ")
        try:
            await execute('DELETE FROM cascade_posts WHERE id = $1', [post_id])
            raise AssertionError('Expected exception')
        except Exception as e:
            expect('violate' in str(e).lower() or 'restrict' in str(e).lower()).to_be_true()
        found_post = await execute('SELECT * FROM cascade_posts WHERE id = $1', [post_id])
        expect(len(found_post)).to_equal(1)

    @test
    async def test_delete_with_cascade_no_children(self, setup_cascade_tables):
        """Test cascade delete when there are no child records."""
        await execute("\n        INSERT INTO cascade_users (name, email)\n        VALUES ('Grace', 'grace@example.com')\n    ")
        user_result = await execute("SELECT id FROM cascade_users WHERE name = 'Grace'")
        user_id = user_result[0]['id']
        await execute('DELETE FROM cascade_users WHERE id = $1', [user_id])
        found_user = await execute('SELECT * FROM cascade_users WHERE id = $1', [user_id])
        expect(len(found_user)).to_equal(0)

    @test
    async def test_delete_with_cascade_multiple_children(self, setup_cascade_tables):
        """Test cascade delete with multiple child records."""
        await execute("\n        INSERT INTO cascade_users (name, email)\n        VALUES ('Henry', 'henry@example.com')\n    ")
        user_result = await execute("SELECT id FROM cascade_users WHERE name = 'Henry'")
        user_id = user_result[0]['id']
        post_ids = []
        for i in range(3):
            await execute(f"\n            INSERT INTO cascade_posts (user_id, title, content)\n            VALUES ({user_id}, 'Post {i + 1}', 'Content {i + 1}')\n        ")
        post_results = await execute('SELECT id FROM cascade_posts WHERE user_id = $1', [user_id])
        post_ids = [row['id'] for row in post_results]
        await execute('DELETE FROM cascade_users WHERE id = $1', [user_id])
        found_user = await execute('SELECT * FROM cascade_users WHERE id = $1', [user_id])
        expect(len(found_user)).to_equal(0)
        for post_id in post_ids:
            found_post = await execute('SELECT * FROM cascade_posts WHERE id = $1', [post_id])
            expect(len(found_post)).to_equal(0)

    @test
    async def test_delete_with_cascade_transaction_rollback(self, setup_cascade_tables):
        """Test that failed cascade delete prevents deletion (RESTRICT constraint)."""
        await execute("\n        INSERT INTO cascade_users (name, email)\n        VALUES ('Ivy', 'ivy@example.com')\n    ")
        user_result = await execute("SELECT id FROM cascade_users WHERE name = 'Ivy'")
        user_id = user_result[0]['id']
        await execute(f"\n        INSERT INTO cascade_posts (user_id, title, content)\n        VALUES ({user_id}, 'Ivy''s Post', 'Content')\n    ")
        post_result = await execute('SELECT id FROM cascade_posts WHERE user_id = $1', [user_id])
        post_id = post_result[0]['id']
        await execute(f"\n        INSERT INTO cascade_comments (post_id, user_id, text)\n        VALUES ({post_id}, {user_id}, 'Comment')\n    ")
        comment_result = await execute('SELECT id FROM cascade_comments WHERE post_id = $1', [post_id])
        comment_id = comment_result[0]['id']
        try:
            await execute('DELETE FROM cascade_posts WHERE id = $1', [post_id])
            raise AssertionError('Expected exception')
        except Exception:
            pass
        found_user = await execute('SELECT * FROM cascade_users WHERE id = $1', [user_id])
        expect(len(found_user)).to_equal(1)
        found_post = await execute('SELECT * FROM cascade_posts WHERE id = $1', [post_id])
        expect(len(found_post)).to_equal(1)
        found_comment = await execute('SELECT * FROM cascade_comments WHERE id = $1', [comment_id])
        expect(len(found_comment)).to_equal(1)

@fixture
async def setup_cascade_tables():
    """Set up test tables with various cascade rules."""
    await execute('\n        CREATE TABLE IF NOT EXISTS cascade_users (\n            id SERIAL PRIMARY KEY,\n            name TEXT NOT NULL,\n            email TEXT UNIQUE NOT NULL\n        );\n    ')
    await execute('\n        CREATE TABLE IF NOT EXISTS cascade_posts (\n            id SERIAL PRIMARY KEY,\n            user_id INTEGER NOT NULL,\n            title TEXT NOT NULL,\n            content TEXT,\n            FOREIGN KEY (user_id) REFERENCES cascade_users(id) ON DELETE CASCADE\n        );\n    ')
    await execute('\n        CREATE TABLE IF NOT EXISTS cascade_comments (\n            id SERIAL PRIMARY KEY,\n            post_id INTEGER NOT NULL,\n            user_id INTEGER,\n            text TEXT NOT NULL,\n            FOREIGN KEY (post_id) REFERENCES cascade_posts(id) ON DELETE RESTRICT,\n            FOREIGN KEY (user_id) REFERENCES cascade_users(id) ON DELETE SET NULL\n        );\n    ')
    yield

class TestNestedCascadeDelete(PostgresSuite):
    """Test nested relationship cascade delete scenarios."""

    @test
    async def test_cascade_three_levels_deep(self):
        """Test cascade delete through three levels of relationships.

        Schema: User → Post (CASCADE) → Comment (CASCADE) → Reply (CASCADE)
        Delete user, verify all 4 levels cascade.
        """
        try:
            await execute('\n                CREATE TABLE IF NOT EXISTS nested_users (\n                    id SERIAL PRIMARY KEY,\n                    name TEXT NOT NULL,\n                    email TEXT UNIQUE NOT NULL\n                );\n            ')
            await execute('\n                CREATE TABLE IF NOT EXISTS nested_posts (\n                    id SERIAL PRIMARY KEY,\n                    user_id INTEGER NOT NULL,\n                    title TEXT NOT NULL,\n                    FOREIGN KEY (user_id) REFERENCES nested_users(id) ON DELETE CASCADE\n                );\n            ')
            await execute('\n                CREATE TABLE IF NOT EXISTS nested_comments (\n                    id SERIAL PRIMARY KEY,\n                    post_id INTEGER NOT NULL,\n                    text TEXT NOT NULL,\n                    FOREIGN KEY (post_id) REFERENCES nested_posts(id) ON DELETE CASCADE\n                );\n            ')
            await execute('\n                CREATE TABLE IF NOT EXISTS nested_replies (\n                    id SERIAL PRIMARY KEY,\n                    comment_id INTEGER NOT NULL,\n                    text TEXT NOT NULL,\n                    FOREIGN KEY (comment_id) REFERENCES nested_comments(id) ON DELETE CASCADE\n                );\n            ')
            await execute("\n                INSERT INTO nested_users (name, email)\n                VALUES ('John', 'john@example.com')\n            ")
            user_result = await execute("SELECT id FROM nested_users WHERE name = 'John'")
            user_id = user_result[0]['id']
            await execute(f"\n                INSERT INTO nested_posts (user_id, title)\n                VALUES ({user_id}, 'My Post')\n            ")
            post_result = await execute('SELECT id FROM nested_posts WHERE user_id = $1', [user_id])
            post_id = post_result[0]['id']
            await execute(f"\n                INSERT INTO nested_comments (post_id, text)\n                VALUES ({post_id}, 'Great post!')\n            ")
            comment_result = await execute('SELECT id FROM nested_comments WHERE post_id = $1', [post_id])
            comment_id = comment_result[0]['id']
            await execute(f"\n                INSERT INTO nested_replies (comment_id, text)\n                VALUES ({comment_id}, 'I agree!')\n            ")
            reply_result = await execute('SELECT id FROM nested_replies WHERE comment_id = $1', [comment_id])
            reply_id = reply_result[0]['id']
            expect(len(await execute('SELECT * FROM nested_users WHERE id = $1', [user_id]))).to_equal(1)
            expect(len(await execute('SELECT * FROM nested_posts WHERE id = $1', [post_id]))).to_equal(1)
            expect(len(await execute('SELECT * FROM nested_comments WHERE id = $1', [comment_id]))).to_equal(1)
            expect(len(await execute('SELECT * FROM nested_replies WHERE id = $1', [reply_id]))).to_equal(1)
            await execute('DELETE FROM nested_users WHERE id = $1', [user_id])
            expect(len(await execute('SELECT * FROM nested_users WHERE id = $1', [user_id]))).to_equal(0)
            expect(len(await execute('SELECT * FROM nested_posts WHERE id = $1', [post_id]))).to_equal(0)
            expect(len(await execute('SELECT * FROM nested_comments WHERE id = $1', [comment_id]))).to_equal(0)
            expect(len(await execute('SELECT * FROM nested_replies WHERE id = $1', [reply_id]))).to_equal(0)
        finally:
            await execute('DROP TABLE IF EXISTS nested_replies CASCADE')
            await execute('DROP TABLE IF EXISTS nested_comments CASCADE')
            await execute('DROP TABLE IF EXISTS nested_posts CASCADE')
            await execute('DROP TABLE IF EXISTS nested_users CASCADE')

    @test
    async def test_cascade_mixed_rules_nested(self):
        """Test cascade with mixed rules in nested relationships.

        Schema: User (CASCADE) → Post (CASCADE) → Comment (RESTRICT)
        Delete user should fail because nested RESTRICT exists.
        """
        try:
            await execute('\n                CREATE TABLE IF NOT EXISTS mixed_users (\n                    id SERIAL PRIMARY KEY,\n                    name TEXT NOT NULL\n                );\n            ')
            await execute('\n                CREATE TABLE IF NOT EXISTS mixed_posts (\n                    id SERIAL PRIMARY KEY,\n                    user_id INTEGER NOT NULL,\n                    title TEXT NOT NULL,\n                    FOREIGN KEY (user_id) REFERENCES mixed_users(id) ON DELETE CASCADE\n                );\n            ')
            await execute('\n                CREATE TABLE IF NOT EXISTS mixed_comments (\n                    id SERIAL PRIMARY KEY,\n                    post_id INTEGER NOT NULL,\n                    text TEXT NOT NULL,\n                    FOREIGN KEY (post_id) REFERENCES mixed_posts(id) ON DELETE RESTRICT\n                );\n            ')
            await execute("INSERT INTO mixed_users (name) VALUES ('Alice')")
            user_result = await execute("SELECT id FROM mixed_users WHERE name = 'Alice'")
            user_id = user_result[0]['id']
            await execute(f"INSERT INTO mixed_posts (user_id, title) VALUES ({user_id}, 'Post')")
            post_result = await execute('SELECT id FROM mixed_posts WHERE user_id = $1', [user_id])
            post_id = post_result[0]['id']
            await execute(f"INSERT INTO mixed_comments (post_id, text) VALUES ({post_id}, 'Comment')")
            try:
                await execute('DELETE FROM mixed_users WHERE id = $1', [user_id])
                raise AssertionError('Expected exception')
            except Exception as e:
                expect('violate' in str(e).lower() or 'restrict' in str(e).lower()).to_be_true()
            expect(len(await execute('SELECT * FROM mixed_users WHERE id = $1', [user_id]))).to_equal(1)
            expect(len(await execute('SELECT * FROM mixed_posts WHERE id = $1', [post_id]))).to_equal(1)
        finally:
            await execute('DROP TABLE IF EXISTS mixed_comments CASCADE')
            await execute('DROP TABLE IF EXISTS mixed_posts CASCADE')
            await execute('DROP TABLE IF EXISTS mixed_users CASCADE')

    @test
    async def test_cascade_with_set_null_grandchild(self):
        """Test cascade with SET NULL on grandchild relationship.

        Schema: User (CASCADE) → Post → Comment (SET NULL on optional_user_id)
        Delete user, verify posts cascade but comments remain with NULL user_id.
        """
        try:
            await execute('\n                CREATE TABLE IF NOT EXISTS setnull_users (\n                    id SERIAL PRIMARY KEY,\n                    name TEXT NOT NULL\n                );\n            ')
            await execute('\n                CREATE TABLE IF NOT EXISTS setnull_posts (\n                    id SERIAL PRIMARY KEY,\n                    user_id INTEGER NOT NULL,\n                    title TEXT NOT NULL,\n                    FOREIGN KEY (user_id) REFERENCES setnull_users(id) ON DELETE CASCADE\n                );\n            ')
            await execute('\n                CREATE TABLE IF NOT EXISTS setnull_comments (\n                    id SERIAL PRIMARY KEY,\n                    post_id INTEGER NOT NULL,\n                    optional_user_id INTEGER,\n                    text TEXT NOT NULL,\n                    FOREIGN KEY (post_id) REFERENCES setnull_posts(id) ON DELETE CASCADE,\n                    FOREIGN KEY (optional_user_id) REFERENCES setnull_users(id) ON DELETE SET NULL\n                );\n            ')
            await execute("INSERT INTO setnull_users (name) VALUES ('Bob'), ('Charlie')")
            bob_result = await execute("SELECT id FROM setnull_users WHERE name = 'Bob'")
            bob_id = bob_result[0]['id']
            charlie_result = await execute("SELECT id FROM setnull_users WHERE name = 'Charlie'")
            charlie_id = charlie_result[0]['id']
            await execute(f"INSERT INTO setnull_posts (user_id, title) VALUES ({charlie_id}, 'Charlie Post')")
            post_result = await execute('SELECT id FROM setnull_posts WHERE user_id = $1', [charlie_id])
            post_id = post_result[0]['id']
            await execute(f"\n                INSERT INTO setnull_comments (post_id, optional_user_id, text)\n                VALUES ({post_id}, {bob_id}, 'Nice post!')\n            ")
            comment_result = await execute('SELECT id FROM setnull_comments WHERE post_id = $1', [post_id])
            comment_id = comment_result[0]['id']
            await execute('DELETE FROM setnull_users WHERE id = $1', [bob_id])
            expect(len(await execute('SELECT * FROM setnull_users WHERE id = $1', [bob_id]))).to_equal(0)
            comment = await execute('SELECT * FROM setnull_comments WHERE id = $1', [comment_id])
            expect(len(comment)).to_equal(1)
            expect(comment[0]['optional_user_id']).to_be_none()
            expect(comment[0]['text']).to_equal('Nice post!')
            await execute('DELETE FROM setnull_users WHERE id = $1', [charlie_id])
            expect(len(await execute('SELECT * FROM setnull_posts WHERE id = $1', [post_id]))).to_equal(0)
            expect(len(await execute('SELECT * FROM setnull_comments WHERE id = $1', [comment_id]))).to_equal(0)
        finally:
            await execute('DROP TABLE IF EXISTS setnull_comments CASCADE')
            await execute('DROP TABLE IF EXISTS setnull_posts CASCADE')
            await execute('DROP TABLE IF EXISTS setnull_users CASCADE')

    @test
    async def test_cascade_sibling_branches(self):
        """Test cascade delete with sibling branches.

        Schema: User (CASCADE) → Post and User (CASCADE) → Profile
        Delete user, verify both branches cascade independently.
        """
        try:
            await execute('\n                CREATE TABLE IF NOT EXISTS branch_users (\n                    id SERIAL PRIMARY KEY,\n                    name TEXT NOT NULL\n                );\n            ')
            await execute('\n                CREATE TABLE IF NOT EXISTS branch_posts (\n                    id SERIAL PRIMARY KEY,\n                    user_id INTEGER NOT NULL,\n                    title TEXT NOT NULL,\n                    FOREIGN KEY (user_id) REFERENCES branch_users(id) ON DELETE CASCADE\n                );\n            ')
            await execute('\n                CREATE TABLE IF NOT EXISTS branch_profiles (\n                    id SERIAL PRIMARY KEY,\n                    user_id INTEGER NOT NULL,\n                    bio TEXT,\n                    FOREIGN KEY (user_id) REFERENCES branch_users(id) ON DELETE CASCADE\n                );\n            ')
            await execute("INSERT INTO branch_users (name) VALUES ('Diana')")
            user_result = await execute("SELECT id FROM branch_users WHERE name = 'Diana'")
            user_id = user_result[0]['id']
            await execute(f"INSERT INTO branch_posts (user_id, title) VALUES ({user_id}, 'Post 1'), ({user_id}, 'Post 2')")
            post_results = await execute('SELECT id FROM branch_posts WHERE user_id = $1', [user_id])
            post_ids = [row['id'] for row in post_results]
            await execute(f"INSERT INTO branch_profiles (user_id, bio) VALUES ({user_id}, 'Hello world')")
            profile_result = await execute('SELECT id FROM branch_profiles WHERE user_id = $1', [user_id])
            profile_id = profile_result[0]['id']
            expect(len(await execute('SELECT * FROM branch_users WHERE id = $1', [user_id]))).to_equal(1)
            expect(len(post_results)).to_equal(2)
            expect(len(await execute('SELECT * FROM branch_profiles WHERE id = $1', [profile_id]))).to_equal(1)
            await execute('DELETE FROM branch_users WHERE id = $1', [user_id])
            expect(len(await execute('SELECT * FROM branch_users WHERE id = $1', [user_id]))).to_equal(0)
            for post_id in post_ids:
                expect(len(await execute('SELECT * FROM branch_posts WHERE id = $1', [post_id]))).to_equal(0)
            expect(len(await execute('SELECT * FROM branch_profiles WHERE id = $1', [profile_id]))).to_equal(0)
        finally:
            await execute('DROP TABLE IF EXISTS branch_profiles CASCADE')
            await execute('DROP TABLE IF EXISTS branch_posts CASCADE')
            await execute('DROP TABLE IF EXISTS branch_users CASCADE')

    @test
    async def test_cascade_diamond_relationship(self):
        """Test cascade with diamond relationship.

        Schema: User → Post (CASCADE), Post → Comment (RESTRICT)
        Deleting User triggers CASCADE to Post, which is blocked by RESTRICT on Comment.
        Note: Comment has NO direct FK to User (to ensure RESTRICT works).
        """
        try:
            await execute('''
                CREATE TABLE IF NOT EXISTS diamond_users (
                    id SERIAL PRIMARY KEY,
                    name TEXT NOT NULL
                );
            ''')
            await execute('''
                CREATE TABLE IF NOT EXISTS diamond_posts (
                    id SERIAL PRIMARY KEY,
                    user_id INTEGER NOT NULL,
                    title TEXT NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES diamond_users(id) ON DELETE CASCADE
                );
            ''')
            await execute('''
                CREATE TABLE IF NOT EXISTS diamond_comments (
                    id SERIAL PRIMARY KEY,
                    post_id INTEGER NOT NULL,
                    text TEXT NOT NULL,
                    FOREIGN KEY (post_id) REFERENCES diamond_posts(id) ON DELETE RESTRICT
                );
            ''')
            await execute("INSERT INTO diamond_users (name) VALUES ('Eve')")
            user_result = await execute("SELECT id FROM diamond_users WHERE name = 'Eve'")
            user_id = user_result[0]['id']
            await execute(f"INSERT INTO diamond_posts (user_id, title) VALUES ({user_id}, 'Eve Post')")
            post_result = await execute('SELECT id FROM diamond_posts WHERE user_id = $1', [user_id])
            post_id = post_result[0]['id']
            await execute(f"INSERT INTO diamond_comments (post_id, text) VALUES ({post_id}, 'Comment')")
            # Try to delete user - should fail because Post has Comment with RESTRICT
            delete_failed = False
            try:
                await execute('DELETE FROM diamond_users WHERE id = $1', [user_id])
            except Exception as e:
                delete_failed = True
                # PostgreSQL restrict violation should contain 'violate' or 'foreign key'
                error_msg = str(e).lower()
                expect('violate' in error_msg or 'foreign' in error_msg or 'constraint' in error_msg).to_be_true()
            expect(delete_failed).to_be_true()
            # Verify data is still intact
            expect(len(await execute('SELECT * FROM diamond_users WHERE id = $1', [user_id]))).to_equal(1)
            expect(len(await execute('SELECT * FROM diamond_posts WHERE id = $1', [post_id]))).to_equal(1)
            # Delete comment first, then cascade should work
            await execute(f'DELETE FROM diamond_comments WHERE post_id = {post_id}')
            await execute('DELETE FROM diamond_users WHERE id = $1', [user_id])
            expect(len(await execute('SELECT * FROM diamond_users WHERE id = $1', [user_id]))).to_equal(0)
            expect(len(await execute('SELECT * FROM diamond_posts WHERE id = $1', [post_id]))).to_equal(0)
        finally:
            await execute('DROP TABLE IF EXISTS diamond_comments CASCADE')
            await execute('DROP TABLE IF EXISTS diamond_posts CASCADE')
            await execute('DROP TABLE IF EXISTS diamond_users CASCADE')

    @test
    async def test_cascade_large_nested_tree(self):
        """Test cascade with large nested tree.

        Create: 1 user → 5 posts → 10 comments each = 50 comments
        Delete user, verify all cascade efficiently.
        """
        try:
            await execute('\n                CREATE TABLE IF NOT EXISTS large_users (\n                    id SERIAL PRIMARY KEY,\n                    name TEXT NOT NULL\n                );\n            ')
            await execute('\n                CREATE TABLE IF NOT EXISTS large_posts (\n                    id SERIAL PRIMARY KEY,\n                    user_id INTEGER NOT NULL,\n                    title TEXT NOT NULL,\n                    FOREIGN KEY (user_id) REFERENCES large_users(id) ON DELETE CASCADE\n                );\n            ')
            await execute('\n                CREATE TABLE IF NOT EXISTS large_comments (\n                    id SERIAL PRIMARY KEY,\n                    post_id INTEGER NOT NULL,\n                    text TEXT NOT NULL,\n                    FOREIGN KEY (post_id) REFERENCES large_posts(id) ON DELETE CASCADE\n                );\n            ')
            await execute("INSERT INTO large_users (name) VALUES ('Frank')")
            user_result = await execute("SELECT id FROM large_users WHERE name = 'Frank'")
            user_id = user_result[0]['id']
            post_ids = []
            for i in range(5):
                await execute(f"INSERT INTO large_posts (user_id, title) VALUES ({user_id}, 'Post {i + 1}')")
            post_results = await execute('SELECT id FROM large_posts WHERE user_id = $1', [user_id])
            post_ids = [row['id'] for row in post_results]
            expect(len(post_ids)).to_equal(5)
            comment_count = 0
            for post_id in post_ids:
                for j in range(10):
                    await execute(f"INSERT INTO large_comments (post_id, text) VALUES ({post_id}, 'Comment {j + 1}')")
                    comment_count += 1
            all_comments = await execute('SELECT COUNT(*) as cnt FROM large_comments')
            expect(all_comments[0]['cnt']).to_equal(50)
            await execute('DELETE FROM large_users WHERE id = $1', [user_id])
            expect(len(await execute('SELECT * FROM large_users WHERE id = $1', [user_id]))).to_equal(0)
            remaining_posts = await execute('SELECT COUNT(*) as cnt FROM large_posts WHERE user_id = $1', [user_id])
            expect(remaining_posts[0]['cnt']).to_equal(0)
            remaining_comments = await execute('SELECT COUNT(*) as cnt FROM large_comments')
            expect(remaining_comments[0]['cnt']).to_equal(0)
        finally:
            await execute('DROP TABLE IF EXISTS large_comments CASCADE')
            await execute('DROP TABLE IF EXISTS large_posts CASCADE')
            await execute('DROP TABLE IF EXISTS large_users CASCADE')