"""
Tests for full relations support.

Tests for:
- WriteRules (cascade save)
- DeleteRules (cascade delete)
- fetch_links parameter on queries
- fetch_all_links() method
- Link and BackLink classes

Migrated from pytest to cclab.probe framework.
"""
from typing import Optional
from bson import ObjectId

from cclab import Document, Link, BackLink, WriteRules, DeleteRules
from cclab.probe import test, expect
from tests.base import MongoTestSuite


# =====================
# Test Document Classes
# =====================

class Author(Document):
    """Author document for testing relations."""
    name: str
    bio: str = ""
    posts: BackLink["Article"] = BackLink(document_class=None, link_field="author")

    class Settings:
        name = "test_authors_relations"


class Article(Document):
    """Article document with link to author."""
    title: str
    content: str = ""
    author: Link[Author] = None

    class Settings:
        name = "test_articles_relations"


class Category(Document):
    """Category for testing multiple links."""
    name: str

    class Settings:
        name = "test_categories_relations"


class Tag(Document):
    """Tag document for testing."""
    name: str

    class Settings:
        name = "test_tags_relations"


class ArticleOptionalAuthor(Document):
    """Article document with truly optional author field (no link)."""
    title: str
    content: str = ""

    class Settings:
        name = "test_articles_optional_relations"


class BlogPost(Document):
    """Blog post with multiple relationships."""
    title: str
    author: Link[Author] = None
    category: Link[Category] = None

    class Settings:
        name = "test_blog_posts_relations"


# =====================
# WriteRules Tests
# =====================

class TestWriteRules(MongoTestSuite):
    """Tests for WriteRules cascade save."""

    async def setup(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_authors_relations", {})
        await _engine.delete_many("test_articles_relations", {})
        await _engine.delete_many("test_categories_relations", {})
        await _engine.delete_many("test_blog_posts_relations", {})

    async def teardown(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_authors_relations", {})
        await _engine.delete_many("test_articles_relations", {})
        await _engine.delete_many("test_categories_relations", {})
        await _engine.delete_many("test_blog_posts_relations", {})

    @test(tags=["mongo", "relations", "write-rules"])
    async def test_write_rule_do_nothing(self):
        """Test that DO_NOTHING only saves the current document."""
        author = Author(name="Alice", bio="Writer")
        await author.save()

        article = Article(title="Hello World", author=Link(author))
        await article.save(link_rule=WriteRules.DO_NOTHING)

        expect(article._id).not_.to_be_none()
        found = await Article.find_one(Article.id == article._id)
        expect(found).not_.to_be_none()
        expect(found.title).to_equal("Hello World")

    @test(tags=["mongo", "relations", "write-rules"])
    async def test_write_rule_write_saves_linked(self):
        """Test that WRITE cascades to linked documents."""
        author = Author(name="Bob", bio="Blogger")

        article = Article(title="My First Post", content="Content here", author=author)
        await article.save(link_rule=WriteRules.WRITE)

        expect(article._id).not_.to_be_none()
        expect(author._id).not_.to_be_none()

        found_author = await Author.find_one(Author.id == author._id)
        expect(found_author).not_.to_be_none()
        expect(found_author.name).to_equal("Bob")

    @test(tags=["mongo", "relations", "write-rules"])
    async def test_write_rule_with_saved_link(self):
        """Test WRITE rule when field is a Link to a saved document."""
        author = Author(name="Carol", bio="Tech writer")
        await author.save()

        article = Article(title="Tech Guide", author=Link(author))
        await article.save(link_rule=WriteRules.WRITE)

        expect(author._id).not_.to_be_none()
        expect(article._id).not_.to_be_none()

        found = await Article.find_one(Article.id == article._id)
        expect(found.author._ref).to_equal(author._id)

    @test(tags=["mongo", "relations", "write-rules"])
    async def test_write_rule_nested_cascade(self):
        """Test that WRITE rule cascades through nested links."""
        author = Author(name="Dave", bio="Philosopher")
        category = Category(name="Philosophy")

        blog = BlogPost(title="Deep Thoughts", author=author, category=category)
        await blog.save(link_rule=WriteRules.WRITE)

        expect(author._id).not_.to_be_none()
        expect(category._id).not_.to_be_none()
        expect(blog._id).not_.to_be_none()


# =====================
# DeleteRules Tests
# =====================

class TestDeleteRules(MongoTestSuite):
    """Tests for DeleteRules cascade delete."""

    async def setup(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_authors_relations", {})
        await _engine.delete_many("test_articles_relations", {})

    async def teardown(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_authors_relations", {})
        await _engine.delete_many("test_articles_relations", {})

    @test(tags=["mongo", "relations", "delete-rules"])
    async def test_delete_rule_do_nothing(self):
        """Test that DO_NOTHING only deletes the current document."""
        author = Author(name="Eve", bio="Editor")
        await author.save()

        article = Article(title="Eve's Article", author=Link(author))
        await article.save()

        await author.delete(link_rule=DeleteRules.DO_NOTHING)

        found_author = await Author.find_one(Author.id == author._id)
        expect(found_author).to_be_none()

        found_article = await Article.find_one(Article.id == article._id)
        expect(found_article).not_.to_be_none()

    @test(tags=["mongo", "relations", "delete-rules"])
    async def test_delete_rule_delete_links(self):
        """Test that DELETE_LINKS removes documents that link to this one."""
        author = Author(name="Frank", bio="Fiction writer")
        await author.save()

        article1 = Article(title="Story 1", author=Link(author))
        article2 = Article(title="Story 2", author=Link(author))
        await article1.save()
        await article2.save()

        expect(await Article.count()).to_equal(2)

        await author.delete(link_rule=DeleteRules.DELETE_LINKS)

        found_author = await Author.find_one(Author.id == author._id)
        expect(found_author).to_be_none()

        expect(await Article.count()).to_equal(0)


# =====================
# fetch_links Tests
# =====================

class TestFetchLinks(MongoTestSuite):
    """Tests for fetch_links parameter and fetch_all_links method."""

    async def setup(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_authors_relations", {})
        await _engine.delete_many("test_articles_relations", {})
        await _engine.delete_many("test_categories_relations", {})
        await _engine.delete_many("test_blog_posts_relations", {})
        await _engine.delete_many("test_articles_optional_relations", {})

    async def teardown(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_authors_relations", {})
        await _engine.delete_many("test_articles_relations", {})
        await _engine.delete_many("test_categories_relations", {})
        await _engine.delete_many("test_blog_posts_relations", {})
        await _engine.delete_many("test_articles_optional_relations", {})

    @test(tags=["mongo", "relations", "fetch-links"])
    async def test_find_one_without_fetch_links(self):
        """Test that find_one without fetch_links returns unresolved links."""
        author = Author(name="Grace", bio="Novelist")
        await author.save()

        article = Article(title="The Novel", author=Link(author))
        await article.save()

        found = await Article.find_one(Article.id == article._id)
        expect(found).not_.to_be_none()

        expect(isinstance(found.author, Link)).to_be_true()
        expect(found.author._ref).to_equal(author._id)
        expect(found.author._document).to_be_none()

    @test(tags=["mongo", "relations", "fetch-links"])
    async def test_find_one_with_fetch_links(self):
        """Test that find_one with fetch_links=True resolves links."""
        author = Author(name="Henry", bio="Historian")
        await author.save()

        article = Article(title="History Book", author=Link(author))
        await article.save()

        found = await Article.find_one(Article.id == article._id, fetch_links=True)
        expect(found).not_.to_be_none()

        expect(isinstance(found.author, Link)).to_be_true()
        expect(found.author._document).not_.to_be_none()
        expect(found.author.name).to_equal("Henry")

    @test(tags=["mongo", "relations", "fetch-links"])
    async def test_fetch_all_links_method(self):
        """Test the fetch_all_links instance method."""
        author = Author(name="Iris", bio="Journalist")
        await author.save()

        article = Article(title="News Article", author=Link(author))
        await article.save()

        found = await Article.find_one(Article.id == article._id)
        expect(found.author._document).to_be_none()

        await found.fetch_all_links()

        expect(found.author._document).not_.to_be_none()
        expect(found.author.name).to_equal("Iris")

    @test(tags=["mongo", "relations", "fetch-links"])
    async def test_query_builder_fetch_links(self):
        """Test fetch_links method on QueryBuilder."""
        author = Author(name="Jack", bio="Poet")
        await author.save()

        article1 = Article(title="Poem 1", author=Link(author))
        article2 = Article(title="Poem 2", author=Link(author))
        await article1.save()
        await article2.save()

        articles = await Article.find().fetch_links().to_list()

        expect(len(articles)).to_equal(2)
        for article in articles:
            expect(article.author._document).not_.to_be_none()
            expect(article.author.name).to_equal("Jack")

    @test(tags=["mongo", "relations", "fetch-links"])
    async def test_fetch_links_depth(self):
        """Test fetching nested links with depth parameter."""
        author = Author(name="Kate", bio="Blogger")
        await author.save()

        category = Category(name="Tech")
        await category.save()

        blog = BlogPost(title="Tech Post", author=Link(author), category=Link(category))
        await blog.save()

        found = await BlogPost.find_one(BlogPost.id == blog._id, fetch_links=True)
        expect(found.author._document).not_.to_be_none()
        expect(found.category._document).not_.to_be_none()

    @test(tags=["mongo", "relations", "fetch-links"])
    async def test_fetch_links_no_link_field(self):
        """Test that fetch_links handles documents without link fields gracefully."""
        # Use a document class that doesn't have any Link fields
        article = ArticleOptionalAuthor(title="Simple Article")
        await article.save()

        # Should not raise even with no link fields
        found = await ArticleOptionalAuthor.find_one(
            ArticleOptionalAuthor.id == article._id, fetch_links=True
        )
        expect(found).not_.to_be_none()
        expect(found.title).to_equal("Simple Article")

    @test(tags=["mongo", "relations", "fetch-links"])
    async def test_fetch_links_missing_document(self):
        """Test fetch_links when referenced document doesn't exist."""
        author = Author(name="Leo", bio="Lost")
        await author.save()
        author_id = author._id

        article = Article(title="Orphaned", author=Link(author))
        await article.save()

        await author.delete()

        # Fetch article - should handle missing author gracefully
        found = await Article.find_one(Article.id == article._id, fetch_links=True)
        expect(found).not_.to_be_none()

    @test(tags=["mongo", "relations", "fetch-links"])
    async def test_fetch_backlink(self):
        """Test fetching BackLink relationships."""
        author = Author(name="Mike", bio="Prolific")
        await author.save()

        article1 = Article(title="Post 1", author=Link(author))
        article2 = Article(title="Post 2", author=Link(author))
        await article1.save()
        await article2.save()

        found = await Author.find_one(Author.id == author._id, fetch_links=True)
        expect(found).not_.to_be_none()

        posts = found.posts
        expect(isinstance(posts, BackLink)).to_be_true()
        expect(len(posts)).to_equal(2)


# =====================
# Link Class Tests
# =====================

class TestLinkClass(MongoTestSuite):
    """Tests for the Link class functionality."""

    async def setup(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_authors_relations", {})

    async def teardown(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_authors_relations", {})

    @test(tags=["mongo", "relations", "link"])
    async def test_link_from_document(self):
        """Test creating Link from a Document instance."""
        author = Author(name="Nancy", bio="Writer")
        await author.save()

        link = Link(author)
        expect(link._ref).to_equal(author._id)
        expect(link._document is author).to_be_true()

    @test(tags=["mongo", "relations", "link"])
    async def test_link_from_object_id(self):
        """Test creating Link from ObjectId string."""
        oid = str(ObjectId())

        link = Link(oid)
        expect(link._ref).to_equal(oid)
        expect(link._document).to_be_none()

    @test(tags=["mongo", "relations", "link"])
    async def test_link_attribute_access(self):
        """Test accessing attributes through Link."""
        author = Author(name="Oscar", bio="Blogger")
        await author.save()

        link = Link(author)
        expect(link.name).to_equal("Oscar")
        expect(link.bio).to_equal("Blogger")

    @test(tags=["mongo", "relations", "link"])
    async def test_link_attribute_access_not_fetched(self):
        """Test that accessing attributes on unfetched Link raises error."""
        link = Link(str(ObjectId()))

        error_caught = False
        try:
            _ = link.name
        except ValueError as e:
            error_caught = True
            expect("Document not fetched" in str(e)).to_be_true()

        expect(error_caught).to_be_true()

    @test(tags=["mongo", "relations", "link"])
    async def test_link_is_fetched(self):
        """Test is_fetched property."""
        author = Author(name="Paul", bio="Author")
        await author.save()

        link_fetched = Link(author)
        expect(link_fetched.is_fetched).to_be_true()

        link_unfetched = Link(str(ObjectId()))
        expect(link_unfetched.is_fetched).to_be_false()

    @test(tags=["mongo", "relations", "link"])
    async def test_link_equality(self):
        """Test Link equality comparison."""
        oid = str(ObjectId())

        link1 = Link(oid)
        link2 = Link(oid)
        link3 = Link(str(ObjectId()))

        expect(link1 == link2).to_be_true()
        expect(link1 == link3).to_be_false()
        expect(link1 == oid).to_be_true()


# =====================
# BackLink Class Tests
# =====================

class TestBackLinkClass(MongoTestSuite):
    """Tests for the BackLink class functionality."""

    async def setup(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_authors_relations", {})
        await _engine.delete_many("test_articles_relations", {})

    async def teardown(self):
        """Clean up test collections."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_authors_relations", {})
        await _engine.delete_many("test_articles_relations", {})

    @test(tags=["mongo", "relations", "backlink"])
    async def test_backlink_iteration(self):
        """Test iterating over BackLink documents."""
        author = Author(name="Quinn", bio="Writer")
        await author.save()

        article1 = Article(title="A1", author=Link(author))
        article2 = Article(title="A2", author=Link(author))
        await article1.save()
        await article2.save()

        backlink = BackLink(document_class=Article, link_field="author")
        await backlink.fetch(author._id)

        titles = [a.title for a in backlink]
        expect("A1" in titles).to_be_true()
        expect("A2" in titles).to_be_true()

    @test(tags=["mongo", "relations", "backlink"])
    async def test_backlink_length(self):
        """Test len() on BackLink."""
        author = Author(name="Rachel", bio="Writer")
        await author.save()

        article = Article(title="R1", author=Link(author))
        await article.save()

        backlink = BackLink(document_class=Article, link_field="author")
        await backlink.fetch(author._id)

        expect(len(backlink)).to_equal(1)

    @test(tags=["mongo", "relations", "backlink"])
    async def test_backlink_indexing(self):
        """Test indexing BackLink."""
        author = Author(name="Steve", bio="Writer")
        await author.save()

        article = Article(title="S1", author=Link(author))
        await article.save()

        backlink = BackLink(document_class=Article, link_field="author")
        await backlink.fetch(author._id)

        expect(backlink[0].title).to_equal("S1")

    @test(tags=["mongo", "relations", "backlink"])
    async def test_backlink_empty(self):
        """Test BackLink with no linked documents."""
        author = Author(name="Tina", bio="Writer")
        await author.save()

        backlink = BackLink(document_class=Article, link_field="author")
        await backlink.fetch(author._id)

        expect(len(backlink)).to_equal(0)
        expect(bool(backlink)).to_be_false()


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestWriteRules,
        TestDeleteRules,
        TestFetchLinks,
        TestLinkClass,
        TestBackLinkClass,
    ], verbose=True)
