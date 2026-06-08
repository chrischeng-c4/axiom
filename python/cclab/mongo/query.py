"""
Query builder for chainable MongoDB queries.

This module provides a Beanie-compatible query builder that supports:
- Fluent/chainable API: .sort().skip().limit().to_list()
- Async execution with Rust backend
- Type-safe query expressions

Example:
    >>> users = await User.find(User.age > 25) \\
    ...     .sort(-User.created_at) \\
    ...     .skip(10) \\
    ...     .limit(20) \\
    ...     .to_list()
"""

from __future__ import annotations

from typing import Any, Generic, List, Optional, Type, TypeVar, TYPE_CHECKING, Union

from .fields import QueryExpr, merge_filters

if TYPE_CHECKING:
    from .document import Document
    from .fields import FieldProxy

T = TypeVar("T", bound="Document")


class QueryBuilder(Generic[T]):
    """
    Chainable query builder for MongoDB operations.

    Provides a fluent API for building and executing queries.
    All terminal operations (to_list, first, count, delete) are async.

    Example:
        >>> # Find all active users over 25, sorted by creation date
        >>> users = await User.find(User.active == True, User.age > 25) \\
        ...     .sort(-User.created_at) \\
        ...     .skip(0) \\
        ...     .limit(100) \\
        ...     .to_list()

        >>> # Count matching documents
        >>> count = await User.find(User.status == "active").count()

        >>> # Delete matching documents
        >>> deleted = await User.find(User.status == "deleted").delete()
    """

    def __init__(
        self,
        model: Type[T],
        filters: tuple,
        _sort: Optional[List[tuple]] = None,
        _skip: int = 0,
        _limit: int = 0,
        _projection: Optional[dict] = None,
        _with_children: bool = True,
        _fetch_links: bool = False,
        _fetch_links_depth: int = 1,
    ) -> None:
        """
        Initialize query builder.

        Args:
            model: Document model class
            filters: Tuple of QueryExpr or dict filters
            _sort: Sort specification [(field, direction), ...]
            _skip: Number of documents to skip
            _limit: Maximum documents to return (0 = no limit)
            _projection: Fields to include/exclude
            _with_children: Include child class documents (for inheritance)
            _fetch_links: Whether to fetch linked documents
            _fetch_links_depth: How deep to fetch nested links
        """
        self._model = model
        self._filters = filters
        self._sort_spec = _sort or []
        self._skip_val = _skip
        self._limit_val = _limit
        self._projection = _projection
        self._with_children_val = _with_children
        self._fetch_links_val = _fetch_links
        self._fetch_links_depth_val = _fetch_links_depth

    def _clone(self, **kwargs: Any) -> "QueryBuilder[T]":
        """Create a copy of this builder with updated values."""
        return QueryBuilder(
            model=kwargs.get("model", self._model),
            filters=kwargs.get("filters", self._filters),
            _sort=kwargs.get("_sort", self._sort_spec.copy()),
            _skip=kwargs.get("_skip", self._skip_val),
            _limit=kwargs.get("_limit", self._limit_val),
            _projection=kwargs.get("_projection", self._projection),
            _with_children=kwargs.get("_with_children", self._with_children_val),
            _fetch_links=kwargs.get("_fetch_links", self._fetch_links_val),
            _fetch_links_depth=kwargs.get("_fetch_links_depth", self._fetch_links_depth_val),
        )

    def with_children(self, include: bool = True) -> "QueryBuilder[T]":
        """
        Control whether child class documents are included in results.

        When querying a root document class in an inheritance hierarchy,
        by default all child class documents are included. Use this method
        to filter to only the exact class type.

        Args:
            include: If True (default), include all child class documents.
                    If False, only return documents of the exact class type.

        Returns:
            New QueryBuilder with the with_children setting

        Example:
            >>> # Vehicle is root, Car and Truck are children
            >>> all_vehicles = await Vehicle.find().to_list()  # Cars + Trucks + Vehicles
            >>> only_vehicles = await Vehicle.find().with_children(False).to_list()  # Just Vehicles
        """
        return self._clone(_with_children=include)

    def fetch_links(self, fetch: bool = True, depth: int = 1) -> "QueryBuilder[T]":
        """
        Configure automatic fetching of linked documents.

        When fetch_links is enabled, Link[T] fields will be automatically
        resolved by fetching the referenced documents from the database.

        Args:
            fetch: If True, fetch linked documents after query
            depth: How many levels of nested links to fetch (default 1)

        Returns:
            New QueryBuilder with fetch_links setting

        Example:
            >>> class Post(Document):
            ...     title: str
            ...     author: Link[User]
            ...
            >>> # Without fetch_links - author is just an ObjectId reference
            >>> post = await Post.find_one(Post.id == id)
            >>> print(post.author)  # Link(ref="507f1f77...")
            >>>
            >>> # With fetch_links - author is the full User document
            >>> post = await Post.find_one(Post.id == id, fetch_links=True)
            >>> print(post.author.name)  # "Alice"
            >>>
            >>> # Or using the method:
            >>> post = await Post.find(Post.id == id).fetch_links().first()
        """
        return self._clone(_fetch_links=fetch, _fetch_links_depth=depth)

    def sort(self, *fields: Union[tuple, str]) -> "QueryBuilder[T]":
        """
        Add sort specification.

        Args:
            *fields: Sort fields as tuples (field, direction) or FieldProxy with unary operators

        Returns:
            New QueryBuilder with sort applied

        Example:
            >>> # Sort by created_at descending
            >>> User.find().sort(-User.created_at)

            >>> # Sort by multiple fields
            >>> User.find().sort(-User.created_at, +User.name)

            >>> # Using tuples
            >>> User.find().sort(("created_at", -1), ("name", 1))
        """
        sort_spec = list(self._sort_spec)

        for field in fields:
            if isinstance(field, tuple):
                sort_spec.append(field)
            elif isinstance(field, str):
                # Handle -field for descending, +field or field for ascending
                if field.startswith("-"):
                    sort_spec.append((field[1:], -1))
                elif field.startswith("+"):
                    sort_spec.append((field[1:], 1))
                else:
                    sort_spec.append((field, 1))
            else:
                # Assume it's a FieldProxy with __neg__ or __pos__
                # This shouldn't happen since __neg__ returns tuple
                sort_spec.append((str(field), 1))

        return self._clone(_sort=sort_spec)

    def skip(self, n: int) -> "QueryBuilder[T]":
        """
        Skip n documents.

        Args:
            n: Number of documents to skip

        Returns:
            New QueryBuilder with skip applied

        Example:
            >>> # Skip first 10 results
            >>> User.find().skip(10).to_list()
        """
        return self._clone(_skip=n)

    def limit(self, n: int) -> "QueryBuilder[T]":
        """
        Limit results to n documents.

        Args:
            n: Maximum documents to return

        Returns:
            New QueryBuilder with limit applied

        Example:
            >>> # Get at most 20 users
            >>> User.find().limit(20).to_list()
        """
        return self._clone(_limit=n)

    def project(self, **fields: int) -> "QueryBuilder[T]":
        """
        Set field projection.

        Args:
            **fields: Field names with 1 (include) or 0 (exclude)

        Returns:
            New QueryBuilder with projection applied

        Example:
            >>> # Only return email and name fields
            >>> User.find().project(email=1, name=1).to_list()

            >>> # Exclude password field
            >>> User.find().project(password=0).to_list()
        """
        return self._clone(_projection=fields)

    def _build_filter(self) -> dict:
        """Build the MongoDB filter document.

        Handles inheritance filtering:
        - If with_children=False and the model has a _class_id, adds _class_id filter
        - If querying from a child class (not root), always filter by _class_id
        """
        base_filter = merge_filters(self._filters)

        # Handle inheritance filtering
        class_id_filter = None

        if self._model._class_id is not None:
            # Check if this is a root class or a child class
            if self._model._is_root:
                # Root class: only filter if with_children=False
                if not self._with_children_val:
                    class_id_filter = {"_class_id": self._model._class_id}
            else:
                # Child class: always filter by its own _class_id
                class_id_filter = {"_class_id": self._model._class_id}

        if class_id_filter:
            if base_filter:
                return {"$and": [base_filter, class_id_filter]}
            else:
                return class_id_filter

        return base_filter

    def _build_sort(self) -> Optional[dict]:
        """Build the MongoDB sort document."""
        if not self._sort_spec:
            return None
        return {field: direction for field, direction in self._sort_spec}

    async def to_list(self) -> List[T]:
        """
        Execute query and return all matching documents as a list.

        Returns:
            List of document instances

        Example:
            >>> users = await User.find(User.active == True).to_list()
        """
        from . import _engine

        collection_name = self._model.__collection_name__()
        filter_doc = self._build_filter()
        sort_doc = self._build_sort()

        # Use optimized path: Rust creates Python objects directly
        results = await _engine.find_as_documents(
            collection_name,
            self._model,
            filter_doc,
            sort=sort_doc,
            skip=self._skip_val if self._skip_val > 0 else None,
            limit=self._limit_val if self._limit_val > 0 else None,
        )

        # Fetch linked documents if requested (Week 4-5 optimization: batched!)
        if self._fetch_links_val and results:
            await self._batch_fetch_links_for_list(results, depth=self._fetch_links_depth_val)

        return results

    @staticmethod
    async def _batch_fetch_links_for_list(docs: List, depth: int = 1) -> None:
        """
        Batch fetch links for a list of documents.

        Delegates forward Link fetching to Rust via PyO3 for efficient batched
        queries. Python builds LinkField metadata and hydrates Link objects
        from results. BackLinks continue to use Python path.

        Performance:
            - Before: 100 docs × 3 links = 300 queries
            - After: 100 docs × 3 links = 3 queries (handled in Rust)
            - Improvement: 100x reduction in queries!

        Args:
            docs: List of documents to fetch links for
            depth: How deep to fetch nested links
        """
        if not docs or depth <= 0:
            return

        from . import _engine
        from .links import Link, BackLink, get_link_fields

        # Phase 1: Build LinkField metadata for Rust
        # Collect all forward Link fields and BackLink fields separately
        link_field_metadata = []
        backlink_fields = []
        target_class_map = {}  # collection_name -> target_cls for hydration

        # Get link fields from the first document (all should be same type)
        sample_doc = docs[0]
        link_fields_info = get_link_fields(type(sample_doc))

        # Track is_list per field for hydration
        field_is_list = {}

        for field_name, (link_type, target_type) in link_fields_info.items():
            target_cls = sample_doc._resolve_document_class(target_type)
            if target_cls is None:
                continue

            target_collection = target_cls.__collection_name__()

            if link_type == "Link":
                # Determine if it's a list Link by checking ALL documents
                # This avoids false detection when first doc has None/empty
                is_list = False
                for doc in docs:
                    value = doc._data.get(field_name)
                    if isinstance(value, list):
                        is_list = True
                        break
                    elif value is not None and not isinstance(value, (str, Link)):
                        # Non-null, non-string, non-Link could indicate list
                        is_list = True
                        break

                field_is_list[field_name] = is_list

                # Create PyLinkField for Rust (if available)
                if hasattr(_engine._rust, "LinkField"):
                    link_field = _engine._rust.LinkField.link_list(
                        field_name, target_collection
                    ) if is_list else _engine._rust.LinkField.link(
                        field_name, target_collection
                    )
                    link_field_metadata.append(link_field)
                    target_class_map[target_collection] = target_cls
                else:
                    # Fallback: use legacy path
                    link_field_metadata.append({
                        "field_name": field_name,
                        "target_collection": target_collection,
                        "is_list": is_list,
                    })
                    target_class_map[target_collection] = target_cls

            elif link_type == "BackLink":
                backlink_fields.append((field_name, target_type))

        # Phase 2: Delegate forward Link fetching to Rust if available
        if link_field_metadata and hasattr(_engine._rust, "fetch_links_batched"):
            # Convert docs to dicts for Rust
            doc_dicts = [doc._data for doc in docs]

            # Call Rust batched fetch
            updated_dicts = await _engine.fetch_links_batched(
                doc_dicts, link_field_metadata, depth
            )

            # Validate result length matches input length
            if len(updated_dicts) != len(docs):
                raise RuntimeError(
                    f"Rust fetch_links_batched returned {len(updated_dicts)} docs "
                    f"but expected {len(docs)}"
                )

            # Phase 3: Hydrate Link objects from Rust results
            for doc, updated_dict in zip(docs, updated_dicts):
                for field_name, (link_type, target_type) in link_fields_info.items():
                    if link_type != "Link":
                        continue

                    target_cls = sample_doc._resolve_document_class(target_type)
                    if target_cls is None:
                        continue

                    value = updated_dict.get(field_name)
                    is_list = field_is_list.get(field_name, False)

                    # Handle None/missing values
                    if value is None:
                        if is_list:
                            # Empty list for list link fields (parity with old behavior)
                            doc._data[field_name] = []
                        # For single links, keep original value (None or ref)
                        continue

                    # Hydrate to Link object
                    if isinstance(value, dict) and "_id" in value:
                        # Rust returned the linked document as dict
                        linked_doc = target_cls._from_dict(value)
                        doc._data[field_name] = Link(linked_doc, document_class=target_cls)
                    elif isinstance(value, list):
                        # List of linked documents - always set even if empty
                        links = []
                        for item in value:
                            if isinstance(item, dict) and "_id" in item:
                                linked_doc = target_cls._from_dict(item)
                                links.append(Link(linked_doc, document_class=target_cls))
                        doc._data[field_name] = links
        else:
            # Fallback: Python path for forward links
            await QueryBuilder._batch_fetch_links_python(docs, depth)

        # Phase 4: Handle BackLinks (Python path only - not in Rust yet)
        for doc in docs:
            for field_name, target_type in backlink_fields:
                await doc._fetch_backlink_field(field_name, target_type, depth)

    @staticmethod
    async def _batch_fetch_links_python(docs: List, depth: int = 1) -> None:
        """
        Legacy Python path for batch fetching links.

        Used as fallback when Rust fetch_links_batched is not available.
        """
        if not docs or depth <= 0:
            return

        from collections import defaultdict
        from .links import Link, get_link_fields

        all_link_refs = defaultdict(lambda: defaultdict(list))

        for doc in docs:
            link_fields = get_link_fields(type(doc))

            for field_name, (link_type, target_type) in link_fields.items():
                if link_type != "Link":
                    continue

                value = doc._data.get(field_name)
                if value is None:
                    continue

                target_cls = doc._resolve_document_class(target_type)
                if target_cls is None:
                    continue

                if isinstance(value, Link):
                    if value._document is not None:
                        continue
                    ref_id = value._ref
                elif isinstance(value, str):
                    ref_id = value
                else:
                    continue

                if ref_id is not None:
                    all_link_refs[target_cls][ref_id].append((doc, field_name))

        for target_cls, refs_map in all_link_refs.items():
            if not refs_map:
                continue

            ref_ids = list(refs_map.keys())
            linked_docs = await target_cls.find({"_id": {"$in": ref_ids}}).to_list()
            docs_by_id = {str(doc._id): doc for doc in linked_docs}

            for ref_id, doc_field_pairs in refs_map.items():
                linked_doc = docs_by_id.get(str(ref_id))
                if linked_doc is None:
                    continue

                for doc, field_name in doc_field_pairs:
                    link = Link(linked_doc, document_class=target_cls)
                    doc._data[field_name] = link

            if depth > 1 and linked_docs:
                await QueryBuilder._batch_fetch_links_python(linked_docs, depth=depth - 1)

    async def first(self) -> Optional[T]:
        """
        Return the first matching document.

        Returns:
            First document or None if no match

        Example:
            >>> user = await User.find(User.email == "alice@example.com").first()
        """
        results = await self.limit(1).to_list()
        return results[0] if results else None

    async def first_or_none(self) -> Optional[T]:
        """Alias for first()."""
        return await self.first()

    async def count(self) -> int:
        """
        Count matching documents.

        Returns:
            Number of matching documents

        Example:
            >>> count = await User.find(User.active == True).count()
        """
        from . import _engine

        collection_name = self._model.__collection_name__()
        filter_doc = self._build_filter()

        return await _engine.count(collection_name, filter_doc)

    async def exists(self) -> bool:
        """
        Check if any documents match the query.

        Returns:
            True if at least one document matches

        Example:
            >>> if await User.find(User.email == email).exists():
            ...     raise ValueError("Email already taken")
        """
        count = await self.limit(1).count()
        return count > 0

    async def delete(self) -> int:
        """
        Delete all matching documents.

        Returns:
            Number of deleted documents

        Example:
            >>> deleted = await User.find(User.status == "deleted").delete()
            >>> print(f"Deleted {deleted} users")
        """
        from . import _engine

        collection_name = self._model.__collection_name__()
        filter_doc = self._build_filter()

        return await _engine.delete_many(collection_name, filter_doc)

    async def update(self, update_doc: dict, upsert: bool = False) -> int:
        """
        Update all matching documents.

        Args:
            update_doc: Update operations (e.g., {"$set": {"status": "active"}})
            upsert: If True, insert a new document if no match

        Returns:
            Number of modified documents

        Example:
            >>> # Set all users to inactive
            >>> modified = await User.find(User.last_login < old_date).update(
            ...     {"$set": {"status": "inactive"}}
            ... )
        """
        from . import _engine

        collection_name = self._model.__collection_name__()
        filter_doc = self._build_filter()

        if upsert:
            result = await _engine.update_many_with_options(
                collection_name, filter_doc, update_doc, upsert=True
            )
            return result["modified_count"]
        return await _engine.update_many(collection_name, filter_doc, update_doc)

    async def upsert(self, update_doc: dict) -> dict:
        """
        Update matching documents or insert if none match.

        Args:
            update_doc: Update operations

        Returns:
            Dict with matched_count, modified_count, upserted_id

        Example:
            >>> result = await User.find(User.email == "alice@example.com").upsert(
            ...     {"$set": {"name": "Alice", "email": "alice@example.com"}}
            ... )
            >>> if result.get("upserted_id"):
            ...     print("New user created")
        """
        from . import _engine

        collection_name = self._model.__collection_name__()
        filter_doc = self._build_filter()

        return await _engine.update_one_with_options(
            collection_name, filter_doc, update_doc, upsert=True
        )

    # ===================
    # Fluent Update Operators
    # ===================

    async def set(self, fields: dict) -> int:
        """
        Set field values (Beanie-style fluent update).

        Args:
            fields: Dict of field names to values (can use FieldProxy as keys)

        Returns:
            Number of modified documents

        Example:
            >>> await User.find(User.id == user_id).set({User.name: "Alice", User.age: 30})
            >>> await User.find(User.status == "pending").set({"status": "active"})
        """
        # Convert FieldProxy keys to string names
        update_fields = {}
        for key, value in fields.items():
            if hasattr(key, "name"):
                update_fields[key.name] = value
            else:
                update_fields[str(key)] = value

        return await self.update({"$set": update_fields})

    async def inc(self, fields: dict) -> int:
        """
        Increment field values.

        Args:
            fields: Dict of field names to increment values

        Returns:
            Number of modified documents

        Example:
            >>> await User.find(User.id == user_id).inc({User.score: 10})
            >>> await User.find(User.id == user_id).inc({"login_count": 1})
        """
        update_fields = {}
        for key, value in fields.items():
            if hasattr(key, "name"):
                update_fields[key.name] = value
            else:
                update_fields[str(key)] = value

        return await self.update({"$inc": update_fields})

    async def push(self, fields: dict) -> int:
        """
        Push values to array fields.

        Args:
            fields: Dict of array field names to values to push

        Returns:
            Number of modified documents

        Example:
            >>> await User.find(User.id == user_id).push({User.tags: "new_tag"})
            >>> await User.find(User.id == user_id).push({"tags": {"$each": ["a", "b"]}})
        """
        update_fields = {}
        for key, value in fields.items():
            if hasattr(key, "name"):
                update_fields[key.name] = value
            else:
                update_fields[str(key)] = value

        return await self.update({"$push": update_fields})

    async def pull(self, fields: dict) -> int:
        """
        Remove values from array fields.

        Args:
            fields: Dict of array field names to values to pull

        Returns:
            Number of modified documents

        Example:
            >>> await User.find(User.id == user_id).pull({User.tags: "old_tag"})
        """
        update_fields = {}
        for key, value in fields.items():
            if hasattr(key, "name"):
                update_fields[key.name] = value
            else:
                update_fields[str(key)] = value

        return await self.update({"$pull": update_fields})

    async def add_to_set(self, fields: dict) -> int:
        """
        Add values to array fields only if not already present.

        Args:
            fields: Dict of array field names to values to add

        Returns:
            Number of modified documents

        Example:
            >>> await User.find(User.id == user_id).add_to_set({User.roles: "admin"})
        """
        update_fields = {}
        for key, value in fields.items():
            if hasattr(key, "name"):
                update_fields[key.name] = value
            else:
                update_fields[str(key)] = value

        return await self.update({"$addToSet": update_fields})

    async def unset(self, *fields: Any) -> int:
        """
        Remove fields from documents.

        Args:
            *fields: Field names or FieldProxy objects to unset

        Returns:
            Number of modified documents

        Example:
            >>> await User.find(User.id == user_id).unset(User.temp_field)
            >>> await User.find(User.status == "cleaned").unset("legacy_data", "old_field")
        """
        unset_fields = {}
        for field in fields:
            if hasattr(field, "name"):
                unset_fields[field.name] = ""
            else:
                unset_fields[str(field)] = ""

        return await self.update({"$unset": unset_fields})

    # ===================
    # Aggregation Helpers (Beanie-compatible)
    # ===================

    async def avg(self, field: Union["FieldProxy", str]) -> Optional[float]:
        """
        Calculate average of a field across matching documents.

        Args:
            field: Field name or FieldProxy to average

        Returns:
            Average value or None if no documents match

        Example:
            >>> avg_age = await User.find(User.active == True).avg(User.age)
            >>> avg_price = await Product.find().avg("price")
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        return await self._aggregate_single("$avg", field_name)

    async def sum(self, field: Union["FieldProxy", str]) -> Optional[float]:
        """
        Calculate sum of a field across matching documents.

        Args:
            field: Field name or FieldProxy to sum

        Returns:
            Sum value or None if no documents match

        Example:
            >>> total_sales = await Order.find(Order.status == "completed").sum(Order.amount)
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        return await self._aggregate_single("$sum", field_name)

    async def max(self, field: Union["FieldProxy", str]) -> Optional[Any]:
        """
        Find maximum value of a field across matching documents.

        Args:
            field: Field name or FieldProxy

        Returns:
            Maximum value or None if no documents match

        Example:
            >>> max_score = await Score.find().max(Score.value)
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        return await self._aggregate_single("$max", field_name)

    async def min(self, field: Union["FieldProxy", str]) -> Optional[Any]:
        """
        Find minimum value of a field across matching documents.

        Args:
            field: Field name or FieldProxy

        Returns:
            Minimum value or None if no documents match

        Example:
            >>> min_price = await Product.find(Product.in_stock == True).min(Product.price)
        """
        field_name = field.name if hasattr(field, "name") else str(field)
        return await self._aggregate_single("$min", field_name)

    async def _aggregate_single(self, operator: str, field_name: str) -> Optional[Any]:
        """
        Internal helper for single-value aggregation operations.

        Delegates to Rust execute_aggregation for pipeline construction and execution.
        Python only passes parameters; no pipeline construction happens here.

        Args:
            operator: MongoDB aggregation operator ($avg, $sum, $max, $min)
            field_name: Field to aggregate

        Returns:
            Aggregated value or None
        """
        from . import _engine

        # Map MongoDB operator to operation name for Rust
        op_map = {
            "$avg": "avg",
            "$sum": "sum",
            "$min": "min",
            "$max": "max",
        }
        op = op_map.get(operator)
        if not op:
            raise ValueError(f"Unsupported aggregation operator: {operator}")

        collection_name = self._model.__collection_name__()
        filter_doc = self._build_filter() or None

        return await _engine.execute_aggregation(collection_name, op, field_name, filter_doc)

    def __repr__(self) -> str:
        return (
            f"QueryBuilder({self._model.__name__}, "
            f"filter={self._build_filter()}, "
            f"sort={self._sort_spec}, "
            f"skip={self._skip_val}, "
            f"limit={self._limit_val})"
        )


class AggregationBuilder(Generic[T]):
    """
    Builder for MongoDB aggregation pipelines.

    Example:
        >>> results = await User.aggregate([
        ...     {"$match": {"active": True}},
        ...     {"$group": {"_id": "$department", "count": {"$sum": 1}}},
        ... ]).to_list()
    """

    def __init__(self, model: Type[T], pipeline: List[dict]) -> None:
        self._model = model
        self._pipeline = pipeline

    async def to_list(self) -> List[dict]:
        """
        Execute aggregation and return results.

        Returns:
            List of aggregation result documents
        """
        from . import _engine

        collection_name = self._model.__collection_name__()
        return await _engine.aggregate(collection_name, self._pipeline)

    def __repr__(self) -> str:
        return f"AggregationBuilder({self._model.__name__}, pipeline={self._pipeline})"
