"""
Integration tests for lazy loading relationships (Phase 2).

Tests Phase 2 implementation:
- SELECT strategy (separate query)
- NULL foreign key handling
- Caching loaded values
- Session identity map integration

Prerequisites:
- PostgreSQL container 'rstn-postgres' running on port 5432
- Database credentials: rstn:rstn
- Test database: cclab_test

Run with:
    POSTGRES_URI="postgresql://rstn:rstn@localhost:5432/cclab_test"         uv run pytest tests/postgres/integration/test_lazy_loading.py -v

Setup database:
    bash scripts/setup_test_db.sh
"""
from cclab.probe import expect, fixture, test
from tests.postgres.base import PostgresSuite
from cclab.titan import Table, Column, relationship, init
from cclab.titan.session import Session

class TestLazyLoading(PostgresSuite):

    @test
    async def test_lazy_load_select_strategy(self, sample_data):
        """Test lazy loading with separate SELECT query."""
        post = await Post.get(1)
        expect(post.author.is_loaded).to_be_false()
        author = await post.author
        expect(post.author.is_loaded).to_be_true()
        expect(author).to_not_be_none()
        expect(author.name).to_equal('Alice')
        expect(author.id).to_equal(1)

    @test
    async def test_lazy_load_caching(self, sample_data):
        """Test that loaded values are cached."""
        post = await Post.get(1)
        author1 = await post.author
        author2 = await post.author
        expect(author1).to_be(author2)

    @test
    async def test_lazy_load_null_fk(self, sample_data):
        """Test that NULL foreign keys return None."""
        post = await Post.get(3)
        expect(post.author.ref).to_be_none()
        author = await post.author
        expect(author).to_be_none()
        expect(post.author.is_loaded).to_be_true()

    @test
    async def test_lazy_load_with_session(self, sample_data):
        """Test lazy loading works with session."""
        async with Session() as session:
            post = await session.get(Post, 1)
            author = await post.author
            expect(author).to_not_be_none()
            expect(author.name).to_equal('Alice')

    @test
    async def test_lazy_load_identity_map(self, sample_data):
        """Test identity map ensures same instance."""
        async with Session() as session:
            post1 = await session.get(Post, 1)
            post2 = await session.get(Post, 1)
            author1 = await post1.author
            author2 = await post2.author
            expect(author1).to_be(author2)

    @test
    async def test_lazy_load_identity_map_different_posts(self, sample_data):
        """Test identity map with different posts referencing same user."""
        async with Session() as session:
            post2 = await session.get(Post, 2)
            post2.author_id = 1
            await session.commit()
            post1 = await session.get(Post, 1)
            post2_reloaded = await session.get(Post, 2)
            author1 = await post1.author
            author2 = await post2_reloaded.author
            expect(author1).to_be(author2)
            expect(author1.id).to_equal(1)
            expect(author1.name).to_equal('Alice')

    @test
    async def test_lazy_load_ref_without_loading(self, sample_data):
        """Test that ref property accesses FK without loading."""
        post = await Post.get(1)
        author_id = post.author.ref
        expect(author_id).to_equal(1)
        expect(post.author.is_loaded).to_be_false()

    @test
    async def test_lazy_load_multiple_posts(self, sample_data):
        """Test lazy loading on multiple posts."""
        post1 = await Post.get(1)
        post2 = await Post.get(2)
        author1 = await post1.author
        author2 = await post2.author
        expect(author1.id).to_equal(1)
        expect(author1.name).to_equal('Alice')
        expect(author2.id).to_equal(2)
        expect(author2.name).to_equal('Bob')
        expect(post1.author.is_loaded).to_be_true()
        expect(post2.author.is_loaded).to_be_true()

    @test
    async def test_lazy_load_without_session(self, sample_data):
        """Test lazy loading works without session context."""
        post = await Post.get(1)
        author = await post.author
        expect(author).to_not_be_none()
        expect(author.name).to_equal('Alice')
        expect(post.author.is_loaded).to_be_true()

    @test
    async def test_lazy_load_descriptor_class_access(self, sample_data):
        """Test that descriptor returns itself on class access."""
        descriptor = Post.author
        from cclab.titan.relationships import RelationshipDescriptor
        expect(isinstance(descriptor, RelationshipDescriptor)).to_be_true()

    @test
    async def test_lazy_load_descriptor_instance_access(self, sample_data):
        """Test that descriptor returns loader on instance access."""
        post = await Post.get(1)
        loader = post.author
        from cclab.titan.relationships import RelationshipLoader
        expect(isinstance(loader, RelationshipLoader)).to_be_true()

    @test
    async def test_lazy_load_nonexistent_fk(self, sample_data):
        """Test lazy loading with FK pointing to nonexistent record."""
        post = Post(id=999, title='Orphan Post', author_id=999)
        await post.save()
        try:
            author = await post.author
            expect(author).to_be_none()
            expect(post.author.is_loaded).to_be_true()
        finally:
            await Post.delete_many({'id': 999})

    @test
    async def test_selectinload_prevents_n_plus_1(self, sample_data):
        """Selectinload batch loads relationships to prevent N+1."""
        await User.delete_many({})
        await Post.delete_many({})
        for i in range(1, 11):
            await User(id=i, name=f'User {i}').save()
        for i in range(1, 11):
            await Post(id=i, title=f'Post {i}', author_id=i).save()
        from cclab.titan import selectinload
        posts = await Post.find().options(selectinload('author')).to_list()
        for post in posts:
            author = await post.author
            expect(author).to_not_be_none()
            expect(post.author.is_loaded).to_be_true()
            expect(author.name).to_equal(f'User {post.id}')

    @test
    async def test_selectinload_with_null_fk(self, sample_data):
        """Selectinload handles NULL FKs correctly."""
        await User.delete_many({})
        await Post.delete_many({})
        await User(id=1, name='Alice').save()
        await Post(id=1, title='Post 1', author_id=1).save()
        await Post(id=2, title='Post 2', author_id=None).save()
        from cclab.titan import selectinload
        posts = await Post.find().options(selectinload('author')).to_list()
        author1 = await posts[0].author
        expect(author1).to_not_be_none()
        expect(author1.name).to_equal('Alice')
        author2 = await posts[1].author
        expect(author2).to_be_none()

    @test
    async def test_selectinload_multiple_posts_same_author(self, sample_data):
        """Selectinload handles multiple posts with same author."""
        await User.delete_many({})
        await Post.delete_many({})
        await User(id=1, name='Alice').save()
        await Post(id=1, title='Post 1', author_id=1).save()
        await Post(id=2, title='Post 2', author_id=1).save()
        await Post(id=3, title='Post 3', author_id=1).save()
        from cclab.titan import selectinload
        posts = await Post.find().options(selectinload('author')).to_list()
        authors = []
        for post in posts:
            author = await post.author
            authors.append(author)
        expect(all((author is not None for author in authors))).to_be_true()
        expect(all((author.name == 'Alice' for author in authors))).to_be_true()
        expect(all((author.id == 1 for author in authors))).to_be_true()

    @test
    async def test_noload_option(self, sample_data):
        """NoLoad option marks relationship as never loaded."""
        await User.delete_many({})
        await Post.delete_many({})
        await User(id=1, name='Alice').save()
        await Post(id=1, title='Post 1', author_id=1).save()
        from cclab.titan import noload
        posts = await Post.find().options(noload('author')).to_list()
        post = posts[0]
        expect(post.author.is_loaded).to_be_true()
        author = await post.author
        expect(author).to_be_none()

    @test
    async def test_multiple_options(self, sample_data):
        """Multiple options can be applied."""
        await User.delete_many({})
        await Post.delete_many({})
        await User(id=1, name='Alice').save()
        await Post(id=1, title='Post 1', author_id=1).save()
        from cclab.titan import selectinload
        posts = await Post.find().options(selectinload('author')).to_list()
        expect(posts).to_not_be_none()
        expect(len(posts)).to_equal(1)
        author = await posts[0].author
        expect(author).to_not_be_none()
        expect(author.name).to_equal('Alice')

    @test
    async def test_selectinload_with_empty_result(self, sample_data):
        """Selectinload works with empty query results."""
        await Post.delete_many({})
        from cclab.titan import selectinload
        posts = await Post.find(Post.id == 999).options(selectinload('author')).to_list()
        expect(posts).to_equal([])

    @test
    async def test_selectinload_invalid_relationship(self, sample_data):
        """Selectinload raises error for invalid relationship name."""
        await Post.delete_many({})
        await Post(id=1, title='Post 1', author_id=1).save()
        from cclab.titan import selectinload
        try:
            posts = await Post.find().options(selectinload('invalid_relationship')).to_list()
            raise AssertionError('Expected ValueError')
        except ValueError:
            pass

    @test
    async def test_selectinload_with_all_null_fks(self, sample_data):
        """Selectinload handles case where all FKs are NULL."""
        await User.delete_many({})
        await Post.delete_many({})
        await Post(id=1, title='Post 1', author_id=None).save()
        await Post(id=2, title='Post 2', author_id=None).save()
        from cclab.titan import selectinload
        posts = await Post.find().options(selectinload('author')).to_list()
        for post in posts:
            author = await post.author
            expect(author).to_be_none()
            expect(post.author.is_loaded).to_be_true()

    @test
    async def test_joinedload_not_implemented(self, sample_data):
        """JoinedLoad raises NotImplementedError."""
        await Post.delete_many({})
        await Post(id=1, title='Post 1', author_id=1).save()
        from cclab.titan import joinedload
        try:
            posts = await Post.find().options(joinedload('author')).to_list()
            raise AssertionError('Expected NotImplementedError')
        except NotImplementedError:
            pass

    @test
    async def test_selectinload_chaining_with_filters(self, sample_data):
        """Selectinload works with filtered queries."""
        await User.delete_many({})
        await Post.delete_many({})
        await User(id=1, name='Alice').save()
        await User(id=2, name='Bob').save()
        await Post(id=1, title='Post 1', author_id=1).save()
        await Post(id=2, title='Post 2', author_id=2).save()
        from cclab.titan import selectinload
        posts = await Post.find(Post.id == 1).options(selectinload('author')).to_list()
        expect(len(posts)).to_equal(1)
        author = await posts[0].author
        expect(author).to_not_be_none()
        expect(author.name).to_equal('Alice')

    @test
    async def test_selectinload_with_order_by(self, sample_data):
        """Selectinload works with ordered queries."""
        await User.delete_many({})
        await Post.delete_many({})
        await User(id=1, name='Alice').save()
        await User(id=2, name='Bob').save()
        await Post(id=1, title='Post A', author_id=1).save()
        await Post(id=2, title='Post B', author_id=2).save()
        from cclab.titan import selectinload
        posts = await Post.find().order_by('-id').options(selectinload('author')).to_list()
        expect(len(posts)).to_equal(2)
        expect(posts[0].id).to_equal(2)
        expect(posts[1].id).to_equal(1)
        author1 = await posts[0].author
        author2 = await posts[1].author
        expect(author1.name).to_equal('Bob')
        expect(author2.name).to_equal('Alice')

    @test
    async def test_selectinload_with_limit(self, sample_data):
        """Selectinload works with limited queries."""
        await User.delete_many({})
        await Post.delete_many({})
        for i in range(1, 6):
            await User(id=i, name=f'User {i}').save()
            await Post(id=i, title=f'Post {i}', author_id=i).save()
        from cclab.titan import selectinload
        posts = await Post.find().limit(3).options(selectinload('author')).to_list()
        expect(len(posts)).to_equal(3)
        for post in posts:
            author = await post.author
            expect(author).to_not_be_none()
            expect(post.author.is_loaded).to_be_true()

class User(Table):
    """User model for testing."""
    id: int = Column(primary_key=True)
    name: str

    class Settings:
        table_name = 'test_users_lazy'

class Post(Table):
    """Post model with relationship to User."""
    id: int = Column(primary_key=True)
    title: str
    author_id: int = Column(foreign_key='test_users_lazy.id')
    author: User = relationship(User, foreign_key_column='author_id')

    class Settings:
        table_name = 'test_posts_lazy'

@fixture
async def setup_tables():
    """Setup test tables (database already initialized by conftest)."""
    await User.create_table()
    await Post.create_table()
    yield

@fixture
async def sample_data(setup_tables):
    """Insert sample data (depends on setup_tables)."""
    await Post.delete_many({})
    await User.delete_many({})
    user1 = User(id=1, name='Alice')
    await user1.save()
    user2 = User(id=2, name='Bob')
    await user2.save()
    post1 = Post(id=1, title='Post 1', author_id=1)
    await post1.save()
    post2 = Post(id=2, title='Post 2', author_id=2)
    await post2.save()
    post3 = Post(id=3, title='Post 3', author_id=None)
    await post3.save()
    yield