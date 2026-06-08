"""
Field descriptors and query expressions for Beanie-compatible API.

This module provides:
- FieldProxy: Enables User.email == "x" syntax for type-safe queries
- QueryExpr: Represents a single query condition
- Field: Pydantic-style field descriptor with defaults

Example:
    >>> class User(Document):
    ...     email: str
    ...     age: int
    ...
    >>> # These create QueryExpr objects
    >>> User.email == "alice@example.com"
    >>> User.age > 25
    >>> User.name.in_(["Alice", "Bob"])
"""

from __future__ import annotations

import re
from typing import Any, List, Optional, TYPE_CHECKING

if TYPE_CHECKING:
    from .document import Document


# Regex special characters that need escaping
_REGEX_SPECIAL_CHARS = re.compile(r'([.^$*+?{}\\|\[\]()])')


def escape_regex(pattern: str) -> str:
    """
    Escape special regex characters in a string for safe use in MongoDB regex queries.

    This should be used when user input is used in regex patterns to prevent
    regex injection attacks.

    Args:
        pattern: The string to escape

    Returns:
        String with all regex special characters escaped

    Example:
        >>> from cclab.mongo import escape_regex
        >>>
        >>> user_input = "user@example.com"
        >>> safe_pattern = escape_regex(user_input)
        >>> # safe_pattern = "user@example\\.com"
        >>>
        >>> users = await User.find(User.email.regex(safe_pattern)).to_list()
    """
    return _REGEX_SPECIAL_CHARS.sub(r'\\\1', pattern)


class QueryExpr:
    """
    Represents a single query condition.

    Converts to MongoDB filter syntax when building queries.

    Example:
        >>> expr = QueryExpr("email", "$eq", "alice@example.com")
        >>> expr.to_filter()
        {"email": {"$eq": "alice@example.com"}}

        >>> expr = QueryExpr("age", "$gt", 25)
        >>> expr.to_filter()
        {"age": {"$gt": 25}}
    """

    def __init__(self, field: str, op: str, value: Any) -> None:
        self.field = field
        self.op = op
        self.value = value

    def to_filter(self) -> dict:
        """Convert to MongoDB filter document."""
        # Special case: $eq can be simplified
        if self.op == "$eq":
            return {self.field: self.value}
        return {self.field: {self.op: self.value}}

    def __repr__(self) -> str:
        return f"QueryExpr({self.field!r}, {self.op!r}, {self.value!r})"

    def __and__(self, other: "QueryExpr") -> "QueryExpr":
        """Combine two expressions with $and."""
        if isinstance(other, QueryExpr):
            return QueryExpr("$and", "$and", [self.to_filter(), other.to_filter()])
        raise TypeError(f"Cannot combine QueryExpr with {type(other)}")

    def __or__(self, other: "QueryExpr") -> "QueryExpr":
        """Combine two expressions with $or."""
        if isinstance(other, QueryExpr):
            return QueryExpr("$or", "$or", [self.to_filter(), other.to_filter()])
        raise TypeError(f"Cannot combine QueryExpr with {type(other)}")


class FieldProxy:
    """
    Field proxy that enables attribute-based query expressions.

    When accessing a field on a Document class (e.g., User.email),
    a FieldProxy is returned. This proxy overloads comparison operators
    to create QueryExpr objects.

    When accessed on an instance, returns the actual value from _data.

    Example:
        >>> class User(Document):
        ...     email: str
        ...     age: int
        ...
        >>> User.email  # Returns FieldProxy("email", User)
        >>> User.email == "alice@example.com"  # Returns QueryExpr
        >>> User.age > 25  # Returns QueryExpr
        >>> User.name.in_(["Alice", "Bob"])  # Returns QueryExpr
        >>>
        >>> user = User(email="alice@example.com")
        >>> user.email  # Returns "alice@example.com"
    """

    def __init__(self, name: str, model: Optional[type] = None) -> None:
        self.name = name
        self.model = model

    def __hash__(self) -> int:
        """Make FieldProxy hashable so it can be used as dict key."""
        return hash((self.name, id(self.model)))

    def __get__(self, obj: Any, objtype: Optional[type] = None) -> Any:
        """
        Descriptor protocol: return value on instance access, self on class access.
        """
        if obj is None:
            # Class access: User.email -> FieldProxy
            return self
        # Instance access: user.email -> value from _data
        if hasattr(obj, "_data") and self.name in obj._data:
            return obj._data[self.name]
        # Fall back to checking __dict__ or returning None
        return obj.__dict__.get(self.name)

    def __set__(self, obj: Any, value: Any) -> None:
        """
        Descriptor protocol: set value on instance.
        """
        if hasattr(obj, "_data"):
            obj._data[self.name] = value
        else:
            obj.__dict__[self.name] = value

    def __repr__(self) -> str:
        model_name = self.model.__name__ if self.model else "None"
        return f"FieldProxy({self.name!r}, {model_name})"

    # Comparison operators
    def __eq__(self, other: Any) -> QueryExpr:  # type: ignore[override]
        """Create equality query: User.email == "x" """
        return QueryExpr(self.name, "$eq", other)

    def __ne__(self, other: Any) -> QueryExpr:  # type: ignore[override]
        """Create not-equal query: User.status != "deleted" """
        return QueryExpr(self.name, "$ne", other)

    def __gt__(self, other: Any) -> QueryExpr:
        """Create greater-than query: User.age > 25"""
        return QueryExpr(self.name, "$gt", other)

    def __ge__(self, other: Any) -> QueryExpr:
        """Create greater-than-or-equal query: User.age >= 25"""
        return QueryExpr(self.name, "$gte", other)

    def __lt__(self, other: Any) -> QueryExpr:
        """Create less-than query: User.age < 25"""
        return QueryExpr(self.name, "$lt", other)

    def __le__(self, other: Any) -> QueryExpr:
        """Create less-than-or-equal query: User.age <= 25"""
        return QueryExpr(self.name, "$lte", other)

    # Array/set operators
    def in_(self, values: List[Any]) -> QueryExpr:
        """Create in-array query: User.status.in_(["active", "pending"])"""
        return QueryExpr(self.name, "$in", values)

    def not_in(self, values: List[Any]) -> QueryExpr:
        """Create not-in-array query: User.status.not_in(["deleted", "banned"])"""
        return QueryExpr(self.name, "$nin", values)

    # Existence operators
    def exists(self, value: bool = True) -> QueryExpr:
        """Check field existence: User.middle_name.exists(True)"""
        return QueryExpr(self.name, "$exists", value)

    # String operators
    def regex(self, pattern: str, options: str = "") -> QueryExpr:
        """Regex match: User.email.regex(r".*@example\\.com")"""
        if options:
            return QueryExpr(self.name, "$regex", {"$regex": pattern, "$options": options})
        return QueryExpr(self.name, "$regex", pattern)

    # Array operators
    def all(self, values: List[Any]) -> QueryExpr:
        """All elements match: User.tags.all(["python", "rust"])"""
        return QueryExpr(self.name, "$all", values)

    def size(self, length: int) -> QueryExpr:
        """Array size: User.groups.size(3)"""
        return QueryExpr(self.name, "$size", length)

    def elem_match(self, query: dict) -> QueryExpr:
        """Element match: User.scores.elem_match({"value": {"$gt": 90}})"""
        return QueryExpr(self.name, "$elemMatch", query)

    # Type operator
    def type_(self, bson_type: int | str) -> QueryExpr:
        """Type check: User.age.type_("int")"""
        return QueryExpr(self.name, "$type", bson_type)

    # Geospatial operators
    def near(
        self,
        coordinates: List[float],
        max_distance: Optional[float] = None,
        min_distance: Optional[float] = None,
    ) -> QueryExpr:
        """
        Find documents near a point (requires 2dsphere index).

        Args:
            coordinates: [longitude, latitude] GeoJSON point
            max_distance: Maximum distance in meters
            min_distance: Minimum distance in meters

        Example:
            >>> User.location.near([-73.97, 40.77], max_distance=1000)
        """
        geometry = {
            "$geometry": {"type": "Point", "coordinates": coordinates}
        }
        if max_distance is not None:
            geometry["$maxDistance"] = max_distance
        if min_distance is not None:
            geometry["$minDistance"] = min_distance
        return QueryExpr(self.name, "$near", geometry)

    def geo_within_box(
        self, bottom_left: List[float], top_right: List[float]
    ) -> QueryExpr:
        """
        Find documents within a bounding box.

        Args:
            bottom_left: [longitude, latitude] of bottom-left corner
            top_right: [longitude, latitude] of top-right corner

        Example:
            >>> User.location.geo_within_box([-74.0, 40.0], [-73.0, 41.0])
        """
        box = {"$box": [bottom_left, top_right]}
        return QueryExpr(self.name, "$geoWithin", box)

    def geo_within_polygon(self, coordinates: List[List[float]]) -> QueryExpr:
        """
        Find documents within a polygon.

        Args:
            coordinates: List of [longitude, latitude] points forming polygon

        Example:
            >>> User.location.geo_within_polygon([
            ...     [-74.0, 40.0], [-73.0, 40.0], [-73.0, 41.0], [-74.0, 41.0], [-74.0, 40.0]
            ... ])
        """
        geometry = {
            "$geometry": {"type": "Polygon", "coordinates": [coordinates]}
        }
        return QueryExpr(self.name, "$geoWithin", geometry)

    def geo_within_center_sphere(
        self, center: List[float], radius_radians: float
    ) -> QueryExpr:
        """
        Find documents within a spherical circle.

        Args:
            center: [longitude, latitude] of circle center
            radius_radians: Radius in radians (divide miles by 3963.2 or km by 6378.1)

        Example:
            >>> # 10 mile radius
            >>> User.location.geo_within_center_sphere([-73.97, 40.77], 10 / 3963.2)
        """
        return QueryExpr(
            self.name, "$geoWithin", {"$centerSphere": [center, radius_radians]}
        )

    def geo_intersects(self, geometry: dict) -> QueryExpr:
        """
        Find documents that intersect with a GeoJSON geometry.

        Args:
            geometry: GeoJSON geometry object

        Example:
            >>> User.location.geo_intersects({
            ...     "type": "Polygon",
            ...     "coordinates": [[[-74, 40], [-73, 40], [-73, 41], [-74, 41], [-74, 40]]]
            ... })
        """
        return QueryExpr(self.name, "$geoIntersects", {"$geometry": geometry})

    # Negation for sorting
    def __neg__(self) -> tuple:
        """Descending sort: -User.created_at"""
        return (self.name, -1)

    def __pos__(self) -> tuple:
        """Ascending sort: +User.created_at"""
        return (self.name, 1)

    # Allow attribute chaining for nested fields
    def __getattr__(self, name: str) -> "FieldProxy":
        """Access nested field: User.address.city"""
        return FieldProxy(f"{self.name}.{name}", self.model)


def Field(
    default: Any = ...,
    default_factory: Any = None,
    alias: Optional[str] = None,
    title: Optional[str] = None,
    description: Optional[str] = None,
    gt: Optional[float] = None,
    ge: Optional[float] = None,
    lt: Optional[float] = None,
    le: Optional[float] = None,
    min_length: Optional[int] = None,
    max_length: Optional[int] = None,
    pattern: Optional[str] = None,
    **extra: Any,
) -> Any:
    """
    Pydantic-compatible Field descriptor for Document fields.

    This provides metadata for document fields, similar to Pydantic's Field.
    The actual validation is delegated to Pydantic when the model is used.

    Example:
        >>> class User(Document):
        ...     email: str = Field(description="User's email address")
        ...     age: int = Field(default=0, ge=0, le=150)
        ...     groups: list[str] = Field(default_factory=list)

    Args:
        default: Default value for the field
        default_factory: Factory function for default value (for mutable defaults)
        alias: Alternative name for the field in MongoDB
        title: Human-readable title
        description: Field description
        gt: Greater than constraint (for numbers)
        ge: Greater than or equal constraint
        lt: Less than constraint
        le: Less than or equal constraint
        min_length: Minimum string/list length
        max_length: Maximum string/list length
        pattern: Regex pattern for string validation
        **extra: Additional Pydantic field options
    """
    # Build field info dict
    field_info = {
        "alias": alias,
        "title": title,
        "description": description,
        "gt": gt,
        "ge": ge,
        "lt": lt,
        "le": le,
        "min_length": min_length,
        "max_length": max_length,
        "pattern": pattern,
        **extra,
    }

    # Remove None values
    field_info = {k: v for k, v in field_info.items() if v is not None}

    # Create a FieldInfo-like object
    class FieldInfo:
        def __init__(self, default: Any, default_factory: Any, **kwargs: Any):
            self.default = default
            self.default_factory = default_factory
            self.extra = kwargs

        def __repr__(self) -> str:
            return f"Field(default={self.default!r}, ...)"

    return FieldInfo(default, default_factory, **field_info)


def merge_filters(filters: tuple) -> dict:
    """
    Merge multiple QueryExpr objects into a single MongoDB filter document.

    Args:
        filters: Tuple of QueryExpr objects

    Returns:
        Combined filter document

    Example:
        >>> filters = (User.email == "alice", User.age > 25)
        >>> merge_filters(filters)
        {"email": "alice", "age": {"$gt": 25}}
    """
    if not filters:
        return {}

    result = {}
    and_conditions = []

    for f in filters:
        if isinstance(f, QueryExpr):
            filter_doc = f.to_filter()

            # Handle $and and $or at top level
            if f.field in ("$and", "$or"):
                and_conditions.append(filter_doc)
            else:
                # Merge into result, handling conflicts
                for key, value in filter_doc.items():
                    if key in result:
                        # Conflict - need to use $and
                        and_conditions.append({key: result[key]})
                        and_conditions.append({key: value})
                        del result[key]
                    else:
                        result[key] = value
        elif hasattr(f, "to_filter"):
            # TextSearch or other objects with to_filter method
            filter_doc = f.to_filter()
            result.update(filter_doc)
        elif isinstance(f, dict):
            # Raw dict filter
            result.update(f)

    # If we have $and conditions, merge them
    if and_conditions:
        if result:
            and_conditions.append(result)
        return {"$and": and_conditions} if len(and_conditions) > 1 else and_conditions[0]

    return result


class TextSearch:
    """
    MongoDB text search query builder.

    Text search works differently from regular field queries - it operates
    on text indexes and returns documents matching the search terms.

    Example:
        >>> # Create text index first
        >>> await Article.create_index([("content", "text")])
        >>>
        >>> # Search for documents containing "python" and "rust"
        >>> articles = await Article.find(text_search("python rust")).to_list()
        >>>
        >>> # Exclude terms with minus
        >>> articles = await Article.find(text_search("python -java")).to_list()
        >>>
        >>> # Phrase search with quotes
        >>> articles = await Article.find(text_search('"machine learning"')).to_list()
    """

    def __init__(
        self,
        search: str,
        language: Optional[str] = None,
        case_sensitive: bool = False,
        diacritic_sensitive: bool = False,
    ) -> None:
        """
        Create a text search query.

        Args:
            search: Text to search for. Supports:
                - Terms: "python rust" (matches either)
                - Phrases: '"machine learning"' (exact phrase)
                - Negation: "-java" (excludes term)
            language: Language for stemming (e.g., "english", "spanish")
            case_sensitive: Whether to match case
            diacritic_sensitive: Whether to match diacritics
        """
        self.search = search
        self.language = language
        self.case_sensitive = case_sensitive
        self.diacritic_sensitive = diacritic_sensitive

    def to_filter(self) -> dict:
        """Convert to MongoDB $text query."""
        text_query: dict = {"$search": self.search}

        if self.language:
            text_query["$language"] = self.language
        if self.case_sensitive:
            text_query["$caseSensitive"] = True
        if self.diacritic_sensitive:
            text_query["$diacriticSensitive"] = True

        return {"$text": text_query}

    def __repr__(self) -> str:
        return f"TextSearch({self.search!r})"


def text_search(
    search: str,
    language: Optional[str] = None,
    case_sensitive: bool = False,
    diacritic_sensitive: bool = False,
) -> TextSearch:
    """
    Create a text search query.

    Requires a text index on the collection. Text search matches documents
    containing the specified terms.

    Args:
        search: Text to search for. Supports:
            - Terms: "python rust" (matches either)
            - Phrases: '"machine learning"' (exact phrase)
            - Negation: "-java" (excludes term)
        language: Language for stemming (e.g., "english", "spanish")
        case_sensitive: Whether to match case
        diacritic_sensitive: Whether to match diacritics

    Returns:
        TextSearch object for use in queries

    Example:
        >>> from cclab.mongo import text_search
        >>>
        >>> # Simple text search
        >>> articles = await Article.find(text_search("python")).to_list()
        >>>
        >>> # Case-sensitive search
        >>> articles = await Article.find(
        ...     text_search("Python", case_sensitive=True)
        ... ).to_list()
    """
    return TextSearch(
        search=search,
        language=language,
        case_sensitive=case_sensitive,
        diacritic_sensitive=diacritic_sensitive,
    )
