"""
Document relations for data-bridge.

This module provides Beanie-compatible document linking:
- Link[T]: Reference to another document (fetched on demand)
- BackLink: Reverse relationship
- fetch_links: Recursively fetch linked documents

Example:
    >>> from cclab.mongo import Document, Link, BackLink
    >>>
    >>> class User(Document):
    ...     name: str
    ...     posts: BackLink["Post"] = []
    ...
    >>> class Post(Document):
    ...     title: str
    ...     author: Link[User]
    >>>
    >>> # Create linked documents
    >>> user = await User(name="Alice").save()
    >>> post = await Post(title="Hello", author=user).save()
    >>>
    >>> # Fetch with links resolved
    >>> post = await Post.find_one(Post.id == post.id, fetch_links=True)
    >>> print(post.author.name)  # "Alice"
"""

from __future__ import annotations

from enum import Enum
from typing import (
    Any,
    Generic,
    List,
    Optional,
    Type,
    TypeVar,
    Union,
    get_args,
    get_origin,
    ForwardRef,
    TYPE_CHECKING,
)
# Import ObjectId from parent cclab module (which handles Rust extension)
from .. import ObjectId


class WriteRules(str, Enum):
    """
    Rules for handling linked documents when saving.

    DO_NOTHING: Only save the current document, ignore linked documents
    WRITE: Cascade save to all linked documents before saving this one

    Example:
        >>> user = User(name="Alice")
        >>> post = Post(title="Hello", author=user)
        >>> # Save post AND user (because author is a Link)
        >>> await post.save(link_rule=WriteRules.WRITE)
    """
    DO_NOTHING = "do_nothing"
    WRITE = "write"


class DeleteRules(str, Enum):
    """
    Rules for handling linked documents when deleting.

    DO_NOTHING: Only delete the current document, leave linked documents
    DELETE_LINKS: Delete all documents that link to this one (via BackLink)

    Example:
        >>> user = await User.find_one(User.id == user_id)
        >>> # Delete user AND all posts that reference this user
        >>> await user.delete(link_rule=DeleteRules.DELETE_LINKS)
    """
    DO_NOTHING = "do_nothing"
    DELETE_LINKS = "delete_links"

if TYPE_CHECKING:
    from .document import Document

# Pydantic v2 compatibility
try:
    from pydantic import GetCoreSchemaHandler
    from pydantic_core import CoreSchema, core_schema

    PYDANTIC_V2 = True
except ImportError:
    PYDANTIC_V2 = False


T = TypeVar("T", bound="Document")


class Link(Generic[T]):
    """
    A reference to another document.

    Link[T] stores either:
    - An ObjectId reference (lazy, unfetched)
    - A Document instance (fetched)

    When stored to MongoDB, only the ObjectId is persisted.
    When fetched with fetch_links=True, the full document is loaded.

    Example:
        >>> class Post(Document):
        ...     title: str
        ...     author: Link[User]
        ...
        >>> # Create with reference
        >>> post = Post(title="Hello", author=user)
        >>> await post.save()
        >>>
        >>> # Fetch with link resolved
        >>> post = await Post.find_one(Post.id == id, fetch_links=True)
        >>> print(post.author.name)  # Full User object
        >>>
        >>> # Fetch without resolving (default)
        >>> post = await Post.find_one(Post.id == id)
        >>> print(post.author)  # Link with just ObjectId
    """

    __slots__ = ("_ref", "_document", "_document_class")

    def __init__(
        self,
        ref: Union[str, ObjectId, "Document", "Link[T]", None] = None,
        document_class: Optional[Type[T]] = None,
    ) -> None:
        """
        Initialize a Link.

        Args:
            ref: Can be:
                - ObjectId or hex string: Store as reference
                - Document instance: Store both ref and instance
                - Link: Copy from another Link
                - None: Unset reference
            document_class: The target Document class (for type checking)
        """
        self._document: Optional[T] = None
        self._document_class: Optional[Type[T]] = document_class

        if ref is None:
            self._ref: Optional[str] = None
        elif isinstance(ref, Link):
            self._ref = ref._ref
            self._document = ref._document
            self._document_class = ref._document_class or document_class
        elif isinstance(ref, ObjectId):
            self._ref = str(ref)
        elif isinstance(ref, str):
            # Validate it's a valid ObjectId
            if ObjectId.is_valid(ref):
                self._ref = ref
            else:
                raise ValueError(f"Invalid ObjectId: {ref}")
        else:
            # Assume it's a Document instance
            from .document import Document

            if isinstance(ref, Document):
                doc_id = ref.id
                if doc_id is None:
                    raise ValueError("Cannot link to unsaved document (no id)")
                self._ref = str(doc_id)
                self._document = ref  # type: ignore
            else:
                raise TypeError(f"Cannot create Link from {type(ref).__name__}")

    @property
    def ref(self) -> Optional[str]:
        """Get the ObjectId reference as a hex string."""
        return self._ref

    @property
    def id(self) -> Optional[str]:
        """Alias for ref - get the ObjectId reference."""
        return self._ref

    @property
    def is_fetched(self) -> bool:
        """Check if the linked document has been fetched."""
        return self._document is not None

    def __getattr__(self, name: str) -> Any:
        """
        Delegate attribute access to the fetched document.

        Raises:
            ValueError: If the document hasn't been fetched yet
        """
        if name.startswith("_"):
            raise AttributeError(f"'{type(self).__name__}' has no attribute '{name}'")

        if self._document is None:
            raise ValueError(
                f"Document not fetched. Use fetch_links=True or await link.fetch()"
            )
        return getattr(self._document, name)

    def __repr__(self) -> str:
        if self._document is not None:
            return f"Link({self._document!r})"
        elif self._ref:
            return f"Link(ref={self._ref!r})"
        else:
            return "Link(None)"

    def __eq__(self, other: object) -> bool:
        if isinstance(other, Link):
            return self._ref == other._ref
        elif isinstance(other, (str, ObjectId)):
            return self._ref == str(other)
        return False

    def __hash__(self) -> int:
        return hash(self._ref)

    def __bool__(self) -> bool:
        return self._ref is not None

    async def fetch(self, document_class: Optional[Type[T]] = None) -> Optional[T]:
        """
        Fetch the linked document from the database.

        Args:
            document_class: The Document class to fetch (if not set on init)

        Returns:
            The fetched document or None if not found

        Example:
            >>> post = await Post.find_one(Post.id == id)
            >>> author = await post.author.fetch()
            >>> print(author.name)
        """
        if self._document is not None:
            return self._document

        if self._ref is None:
            return None

        doc_cls = document_class or self._document_class
        if doc_cls is None:
            raise ValueError("Document class not specified. Pass it to fetch()")

        self._document = await doc_cls.find_one({"_id": self._ref})
        return self._document

    def to_ref(self) -> Optional[str]:
        """Convert to reference for database storage."""
        return self._ref

    def to_dict(self) -> Optional[str]:
        """Convert to dict format for serialization."""
        return self._ref

    @classmethod
    def __get_pydantic_core_schema__(
        cls, source_type: Any, handler: GetCoreSchemaHandler
    ) -> CoreSchema:
        """Define Pydantic v2 schema for validation and serialization."""
        return core_schema.union_schema(
            [
                # Accept Link directly
                core_schema.is_instance_schema(cls),
                # Accept ObjectId
                core_schema.is_instance_schema(ObjectId, cls=ObjectId),
                # Accept string (ObjectId hex)
                core_schema.str_schema(),
                # Accept None
                core_schema.none_schema(),
            ],
            serialization=core_schema.plain_serializer_function_ser_schema(
                lambda x: x.to_ref() if isinstance(x, Link) else str(x) if x else None,
                info_arg=False,
                return_schema=core_schema.str_schema(),
            ),
        )


class BackLink(Generic[T]):
    """
    Reverse relationship to documents that link to this one.

    BackLink allows querying documents that reference the current document.
    It's not stored in the database but computed on fetch.

    Example:
        >>> class User(Document):
        ...     name: str
        ...     # Posts that reference this user
        ...     posts: BackLink["Post"] = BackLink(document_class=Post, link_field="author")
        ...
        >>> class Post(Document):
        ...     title: str
        ...     author: Link[User]
        >>>
        >>> user = await User.find_one(User.id == id, fetch_links=True)
        >>> for post in user.posts:
        ...     print(post.title)
    """

    __slots__ = ("_documents", "_document_class", "_link_field", "_original_field")

    def __init__(
        self,
        document_class: Optional[Type[T]] = None,
        link_field: Optional[str] = None,
        original_field: Optional[str] = None,
    ) -> None:
        """
        Initialize a BackLink.

        Args:
            document_class: The Document class that links to this one
            link_field: Name of the Link field in the other document
            original_field: Alias for link_field (Beanie compatibility)
        """
        self._documents: list[T] = []
        self._document_class = document_class
        self._link_field = link_field or original_field
        self._original_field = original_field or link_field

    @property
    def documents(self) -> list[T]:
        """Get the list of linked documents."""
        return self._documents

    def __iter__(self):
        """Iterate over linked documents."""
        return iter(self._documents)

    def __len__(self) -> int:
        """Get the number of linked documents."""
        return len(self._documents)

    def __getitem__(self, index: int) -> T:
        """Get a linked document by index."""
        return self._documents[index]

    def __repr__(self) -> str:
        return f"BackLink({len(self._documents)} documents)"

    def __bool__(self) -> bool:
        return len(self._documents) > 0

    async def fetch(
        self, parent_id: str, document_class: Optional[Type[T]] = None
    ) -> list[T]:
        """
        Fetch documents that link to the parent.

        Args:
            parent_id: The ObjectId of the parent document
            document_class: The Document class to query

        Returns:
            List of documents that link to the parent
        """
        doc_cls = document_class or self._document_class
        if doc_cls is None:
            raise ValueError("Document class not specified")
        if self._link_field is None:
            raise ValueError("Link field not specified")

        self._documents = await doc_cls.find(
            {self._link_field: parent_id}
        ).to_list()
        return self._documents


def resolve_forward_ref(ref: Any, globalns: Optional[dict] = None) -> Optional[type]:
    """
    Resolve a forward reference to its actual type.

    Args:
        ref: The forward reference (ForwardRef or string)
        globalns: Global namespace for resolution

    Returns:
        The resolved type or None if not resolvable
    """
    if isinstance(ref, str):
        ref = ForwardRef(ref)

    if isinstance(ref, ForwardRef):
        try:
            # Python 3.9+ uses evaluate with localns
            return ref._evaluate(globalns or {}, None)
        except Exception:
            return None

    return ref


def get_link_fields(cls: type) -> dict[str, tuple[str, type]]:
    """
    Extract Link and BackLink fields from a class's type annotations.

    Args:
        cls: A class with type annotations

    Returns:
        Dict mapping field name to (link_type, target_class)
        link_type is either "Link" or "BackLink"
    """
    link_fields = {}

    # Get annotations from the class and its bases
    annotations = {}
    for base in reversed(cls.__mro__):
        if hasattr(base, "__annotations__"):
            annotations.update(base.__annotations__)

    for field_name, annotation in annotations.items():
        origin = get_origin(annotation)

        if origin is Link:
            args = get_args(annotation)
            target_type = args[0] if args else None
            link_fields[field_name] = ("Link", target_type)

        elif origin is BackLink:
            args = get_args(annotation)
            target_type = args[0] if args else None
            link_fields[field_name] = ("BackLink", target_type)

    return link_fields


__all__ = [
    "Link",
    "BackLink",
    "WriteRules",
    "DeleteRules",
    "get_link_fields",
    "resolve_forward_ref",
]
