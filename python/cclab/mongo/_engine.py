"""
Internal module that bridges Python layer to Rust backend.

This module wraps the Rust cclab module and provides a clean
async interface for the Document class and QueryBuilder.

NOTE: This is an internal module. Do not import directly.
Use the public API from cclab instead.
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional

# Import the Rust module
try:
    from ..cclab import nebula as _rust
except ImportError as e:
    raise ImportError(
        "cclab Rust extension not found. "
        "Make sure you've built it with: maturin develop"
    ) from e


# ===================
# Connection Management
# ===================


async def init(connection_string: str) -> None:
    """Initialize MongoDB connection via Rust backend."""
    return await _rust.init(connection_string)


def is_connected() -> bool:
    """Check if MongoDB is connected."""
    return _rust.is_connected()


# ===================
# Core Operations
# ===================


async def find_one(
    collection: str,
    filter: Optional[Dict[str, Any]] = None,
) -> Optional[Dict[str, Any]]:
    """
    Find a single document.

    Args:
        collection: Collection name
        filter: Query filter

    Returns:
        Document dict or None
    """
    # T044-T045: Rust find_one now returns PyDict directly (GIL-free conversion)
    result = await _rust.Document.find_one(collection, filter or {})
    return result  # Already a dict, no .to_dict() needed


async def find(
    collection: str,
    filter: Optional[Dict[str, Any]] = None,
) -> List[Dict[str, Any]]:
    """
    Find all matching documents.

    Args:
        collection: Collection name
        filter: Query filter

    Returns:
        List of document dicts
    """
    results = await _rust.Document.find(collection, filter or {})
    return [doc.to_dict() for doc in results]


async def find_with_options(
    collection: str,
    filter: Optional[Dict[str, Any]] = None,
    sort: Optional[Dict[str, int]] = None,
    skip: Optional[int] = None,
    limit: Optional[int] = None,
    projection: Optional[Dict[str, int]] = None,
) -> List[Dict[str, Any]]:
    """
    Find documents with sorting, pagination, and projection.

    Args:
        collection: Collection name
        filter: Query filter
        sort: Sort specification {field: 1 or -1}
        skip: Number of documents to skip
        limit: Maximum documents to return
        projection: Fields to include/exclude

    Returns:
        List of document dicts
    """
    # Check if Rust has find_with_options, otherwise fall back to basic find
    if hasattr(_rust.Document, "find_with_options"):
        results = await _rust.Document.find_with_options(
            collection,
            filter or {},
            sort=sort,
            skip=skip,
            limit=limit,
            projection=projection,
        )
        return [doc.to_dict() for doc in results]
    else:
        # Fallback: basic find without options
        # TODO: Implement in Rust for full support
        results = await _rust.Document.find(collection, filter or {})
        docs = [doc.to_dict() for doc in results]

        # Apply sort in Python (temporary fallback)
        if sort:
            for field, direction in reversed(sort.items()):
                docs.sort(key=lambda x: x.get(field, ""), reverse=(direction == -1))

        # Apply skip/limit in Python
        if skip:
            docs = docs[skip:]
        if limit:
            docs = docs[:limit]

        return docs


async def count(
    collection: str,
    filter: Optional[Dict[str, Any]] = None,
) -> int:
    """
    Count matching documents.

    Args:
        collection: Collection name
        filter: Query filter

    Returns:
        Document count
    """
    return await _rust.Document.count(collection, filter or {})


# ===================
# Insert Operations
# ===================


async def insert_one(
    collection: str,
    document: Dict[str, Any],
    document_class: Optional[type] = None,
) -> str:
    """
    Insert a single document.

    Args:
        collection: Collection name
        document: Document to insert
        document_class: Optional Document class for schema validation

    Returns:
        Inserted document's ObjectId as hex string
    """
    # Create a RustDocument
    doc = _rust.Document(collection, document)

    # Check if validation is enabled in Settings
    use_validation = False
    if document_class and hasattr(document_class, 'Settings'):
        use_validation = getattr(document_class.Settings, 'use_validation', False)

    # Use validated save if document_class is provided, validation is enabled, and save_validated exists
    if document_class and use_validation and hasattr(doc, 'save_validated'):
        from .type_extraction import extract_schema
        schema = extract_schema(document_class)
        return await doc.save_validated(schema)
    else:
        # Fallback to regular save
        return await doc.save()


async def insert_many(
    collection: str,
    documents: List[Dict[str, Any]],
) -> List[str]:
    """
    Insert multiple documents.

    Args:
        collection: Collection name
        documents: List of documents to insert

    Returns:
        List of inserted ObjectIds
    """
    # Check if Rust has bulk insert
    if hasattr(_rust.Document, "insert_many"):
        return await _rust.Document.insert_many(collection, documents)
    else:
        # Fallback: insert one by one
        # TODO: Implement bulk insert in Rust
        ids = []
        for doc in documents:
            rust_doc = _rust.Document(collection, doc)
            doc_id = await rust_doc.save()
            ids.append(doc_id)
        return ids


# ===================
# Update Operations
# ===================


async def update_one(
    collection: str,
    filter: Dict[str, Any],
    update: Dict[str, Any],
    document_class: Optional[type] = None,
) -> int:
    """
    Update a single document.

    Args:
        collection: Collection name
        filter: Query filter
        update: Update operations (e.g., {"$set": {...}})
        document_class: Optional Document class for validating updated fields

    Returns:
        Number of modified documents (0 or 1)
    """
    # Handle $set wrapper
    if "$set" in update:
        update_doc = update["$set"]
    else:
        update_doc = update

    # TODO: Add validation for updated fields
    # Update validation is complex because:
    # - Need to handle various operators ($set, $inc, $push, etc.)
    # - Only validate fields being updated, not entire schema
    # - Requires Rust-side update_one_validated method
    # For now, validation is only applied on insert operations

    return await _rust.Document.update_one(collection, filter, update_doc)


async def update_many(
    collection: str,
    filter: Dict[str, Any],
    update: Dict[str, Any],
) -> int:
    """
    Update multiple documents.

    Args:
        collection: Collection name
        filter: Query filter
        update: Update operations

    Returns:
        Number of modified documents
    """
    # Check if Rust has update_many
    if hasattr(_rust.Document, "update_many"):
        return await _rust.Document.update_many(collection, filter, update)
    else:
        # Fallback: update matching documents one by one
        # This is inefficient but works as a fallback
        # TODO: Implement update_many in Rust
        docs = await find(collection, filter)
        count = 0
        for doc in docs:
            if "_id" in doc:
                await update_one(collection, {"_id": doc["_id"]}, update)
                count += 1
        return count


# ===================
# Delete Operations
# ===================


async def delete_one(
    collection: str,
    filter: Dict[str, Any],
) -> int:
    """
    Delete a single document.

    Args:
        collection: Collection name
        filter: Query filter

    Returns:
        Number of deleted documents (0 or 1)
    """
    # Check if Rust has delete_one
    if hasattr(_rust.Document, "delete_one"):
        return await _rust.Document.delete_one(collection, filter)
    else:
        # Fallback: find one and delete
        doc = await find_one(collection, filter)
        if doc and "_id" in doc:
            # Use delete_many with specific _id
            return await _rust.Document.delete_many(collection, {"_id": doc["_id"]})
        return 0


async def delete_many(
    collection: str,
    filter: Dict[str, Any],
) -> int:
    """
    Delete multiple documents.

    Args:
        collection: Collection name
        filter: Query filter

    Returns:
        Number of deleted documents
    """
    return await _rust.Document.delete_many(collection, filter)


# ===================
# Aggregation
# ===================


async def aggregate(
    collection: str,
    pipeline: List[Dict[str, Any]],
) -> List[Dict[str, Any]]:
    """
    Run an aggregation pipeline.

    Args:
        collection: Collection name
        pipeline: Aggregation pipeline stages

    Returns:
        List of result documents
    """
    # Check if Rust has aggregate
    if hasattr(_rust.Document, "aggregate"):
        results = await _rust.Document.aggregate(collection, pipeline)
        return [doc.to_dict() if hasattr(doc, "to_dict") else doc for doc in results]
    else:
        # Aggregation not implemented in Rust yet
        raise NotImplementedError(
            "Aggregation not yet implemented in Rust backend. "
            "Please use raw MongoDB operations."
        )


async def execute_aggregation(
    collection: str,
    op: str,
    field: Optional[str] = None,
    filter: Optional[Dict[str, Any]] = None,
) -> Any:
    """
    Execute a single-value aggregation via Rust.

    This is the thin wrapper entry point that delegates to Rust's
    execute_aggregation function. Python only passes parameters;
    pipeline construction happens in Rust.

    Args:
        collection: Collection name
        op: Operation type: "avg", "sum", "min", "max", or "count"
        field: Field name for aggregation (required for non-count operations)
        filter: Optional match filter

    Returns:
        Scalar result (int, float) or None if no documents match
    """
    if hasattr(_rust, "execute_aggregation"):
        return await _rust.execute_aggregation(collection, op, field, filter)
    else:
        # Per spec R3: Python must not construct aggregation pipelines
        # Raise error instead of fallback to enforce thin wrapper pattern
        raise NotImplementedError(
            f"Aggregation operation '{op}' requires Rust backend. "
            "Rebuild with: maturin develop"
        )


# ===================
# Upsert Operations
# ===================


async def update_one_with_options(
    collection: str,
    filter: Dict[str, Any],
    update: Dict[str, Any],
    upsert: bool = False,
) -> Dict[str, Any]:
    """
    Update a single document with options.

    Args:
        collection: Collection name
        filter: Query filter
        update: Update operations
        upsert: If True, insert if no match

    Returns:
        Dict with matched_count, modified_count, upserted_id
    """
    if hasattr(_rust.Document, "update_one_with_options"):
        return await _rust.Document.update_one_with_options(
            collection, filter, update, upsert
        )
    else:
        # Fallback without upsert
        count = await update_one(collection, filter, update)
        return {"matched_count": count, "modified_count": count, "upserted_id": None}


async def update_many_with_options(
    collection: str,
    filter: Dict[str, Any],
    update: Dict[str, Any],
    upsert: bool = False,
) -> Dict[str, Any]:
    """
    Update multiple documents with options.

    Args:
        collection: Collection name
        filter: Query filter
        update: Update operations
        upsert: If True, insert if no match

    Returns:
        Dict with matched_count, modified_count, upserted_id
    """
    if hasattr(_rust.Document, "update_many_with_options"):
        return await _rust.Document.update_many_with_options(
            collection, filter, update, upsert
        )
    else:
        count = await update_many(collection, filter, update)
        return {"matched_count": count, "modified_count": count, "upserted_id": None}


# ===================
# Replace Operations
# ===================


async def replace_one(
    collection: str,
    filter: Dict[str, Any],
    replacement: Dict[str, Any],
    upsert: bool = False,
) -> Dict[str, Any]:
    """
    Replace a single document.

    Args:
        collection: Collection name
        filter: Query filter
        replacement: The replacement document
        upsert: If True, insert if no match

    Returns:
        Dict with matched_count, modified_count, upserted_id
    """
    if hasattr(_rust.Document, "replace_one"):
        return await _rust.Document.replace_one(collection, filter, replacement, upsert)
    else:
        # Fallback: delete and insert (not atomic, but works)
        deleted = await delete_one(collection, filter)
        if deleted > 0:
            await insert_one(collection, replacement)
            return {"matched_count": 1, "modified_count": 1, "upserted_id": None}
        elif upsert:
            doc_id = await insert_one(collection, replacement)
            return {"matched_count": 0, "modified_count": 0, "upserted_id": doc_id}
        return {"matched_count": 0, "modified_count": 0, "upserted_id": None}


# ===================
# Distinct Operations
# ===================


async def distinct(
    collection: str,
    field: str,
    filter: Optional[Dict[str, Any]] = None,
) -> List[Any]:
    """
    Get distinct values for a field.

    Args:
        collection: Collection name
        field: Field name
        filter: Optional query filter

    Returns:
        List of distinct values
    """
    if hasattr(_rust.Document, "distinct"):
        return await _rust.Document.distinct(collection, field, filter)
    else:
        # Fallback: aggregate distinct
        pipeline = [{"$group": {"_id": f"${field}"}}]
        if filter:
            pipeline.insert(0, {"$match": filter})
        results = await aggregate(collection, pipeline)
        return [r["_id"] for r in results if r["_id"] is not None]


# ===================
# Find One and Modify
# ===================


async def find_one_and_update(
    collection: str,
    filter: Dict[str, Any],
    update: Dict[str, Any],
    return_document: str = "before",
    upsert: bool = False,
    sort: Optional[Dict[str, int]] = None,
) -> Optional[Dict[str, Any]]:
    """
    Find a document and update it atomically.

    Args:
        collection: Collection name
        filter: Query filter
        update: Update operations
        return_document: "before" or "after"
        upsert: If True, insert if no match
        sort: Sort specification

    Returns:
        The found document or None
    """
    if hasattr(_rust.Document, "find_one_and_update"):
        result = await _rust.Document.find_one_and_update(
            collection, filter, update, return_document, upsert, sort
        )
        return result.to_dict() if result else None
    else:
        # Non-atomic fallback
        doc = await find_one(collection, filter)
        if doc:
            await update_one(collection, filter, update)
            if return_document == "after":
                return await find_one(collection, filter)
            return doc
        elif upsert:
            await insert_one(collection, update.get("$set", update))
            if return_document == "after":
                return await find_one(collection, filter)
        return None


async def find_one_and_replace(
    collection: str,
    filter: Dict[str, Any],
    replacement: Dict[str, Any],
    return_document: str = "before",
    upsert: bool = False,
) -> Optional[Dict[str, Any]]:
    """
    Find a document and replace it atomically.

    Args:
        collection: Collection name
        filter: Query filter
        replacement: The replacement document
        return_document: "before" or "after"
        upsert: If True, insert if no match

    Returns:
        The found document or None
    """
    if hasattr(_rust.Document, "find_one_and_replace"):
        result = await _rust.Document.find_one_and_replace(
            collection, filter, replacement, return_document, upsert
        )
        return result.to_dict() if result else None
    else:
        # Non-atomic fallback
        doc = await find_one(collection, filter)
        if doc:
            result = await replace_one(collection, filter, replacement, upsert=False)
            if result["modified_count"] > 0 and return_document == "after":
                return await find_one(collection, {"_id": doc["_id"]})
            return doc
        elif upsert:
            await insert_one(collection, replacement)
            if return_document == "after":
                return await find_one(collection, filter)
        return None


async def find_one_and_delete(
    collection: str,
    filter: Dict[str, Any],
    sort: Optional[Dict[str, int]] = None,
) -> Optional[Dict[str, Any]]:
    """
    Find a document and delete it atomically.

    Args:
        collection: Collection name
        filter: Query filter
        sort: Sort specification

    Returns:
        The deleted document or None
    """
    if hasattr(_rust.Document, "find_one_and_delete"):
        result = await _rust.Document.find_one_and_delete(collection, filter, sort)
        return result.to_dict() if result else None
    else:
        # Non-atomic fallback
        doc = await find_one(collection, filter)
        if doc:
            await delete_one(collection, {"_id": doc["_id"]})
            return doc
        return None


# ===================
# Optimized Operations
# ===================


async def find_as_documents(
    collection: str,
    document_class: type,
    filter: Optional[Dict[str, Any]] = None,
    sort: Optional[Dict[str, int]] = None,
    skip: Optional[int] = None,
    limit: Optional[int] = None,
) -> List[Any]:
    """
    Find documents and return as typed Document instances.

    This is an optimized version that creates Python objects directly in Rust,
    avoiding intermediate dict conversion.

    For document inheritance hierarchies (root classes with children), this
    falls back to the Python path to enable polymorphic loading via _from_db().

    Args:
        collection: Collection name
        document_class: The Document subclass to instantiate
        filter: Query filter
        sort: Sort specification {field: 1 or -1}
        skip: Number of documents to skip
        limit: Maximum documents to return

    Returns:
        List of document instances (typed, may be polymorphic subclasses)
    """
    # Check if this class is part of an inheritance hierarchy with children
    # If so, we need Python's _from_db() for polymorphic loading
    has_children = (
        hasattr(document_class, "_child_classes")
        and document_class._child_classes
        and len(document_class._child_classes) > 1
    )

    # Use optimized Rust path for non-polymorphic classes
    if hasattr(_rust.Document, "find_as_documents") and not has_children:
        return await _rust.Document.find_as_documents(
            collection,
            document_class,
            filter or {},
            sort=sort,
            skip=skip,
            limit=limit,
        )
    else:
        # Fallback to Python path (enables polymorphic loading via _from_db)
        results = await find_with_options(
            collection, filter, sort, skip, limit
        )
        # Database data is already valid, skip validation for 2-3x speedup
        return [document_class._from_db(doc, validate=False) for doc in results]


# ===================
# Bulk Operations
# ===================


async def bulk_write(
    collection: str,
    operations: List[Dict[str, Any]],
    ordered: bool = True,
) -> Dict[str, Any]:
    """
    Execute bulk write operations.

    Args:
        collection: Collection name
        operations: List of operation dicts, each with:
            - op: Operation type (insert_one, update_one, update_many, delete_one, delete_many, replace_one)
            - filter: Query filter (for update/delete/replace)
            - document: Document to insert (for insert_one)
            - update: Update operations (for update_one/update_many)
            - replacement: Replacement document (for replace_one)
            - upsert: If True, insert if no match (for update/replace)
        ordered: If True, stop on first error. If False, continue with remaining ops.

    Returns:
        Dict with:
            - inserted_count: Number of inserted documents
            - matched_count: Number of matched documents
            - modified_count: Number of modified documents
            - deleted_count: Number of deleted documents
            - upserted_count: Number of upserted documents
            - upserted_ids: Dict mapping operation index to upserted _id

    Example:
        >>> from cclab.mongo import UpdateOne, InsertOne, DeleteOne
        >>> result = await User.bulk_write([
        ...     UpdateOne(User.status == "pending").set(User.status, "active"),
        ...     InsertOne(User(name="Alice")),
        ...     DeleteOne(User.expired == True),
        ... ])
    """
    if hasattr(_rust.Document, "bulk_write"):
        result = await _rust.Document.bulk_write(collection, operations, ordered)
        # Result can be dict (from IntoPy) or wrapper object
        if isinstance(result, dict):
            return result
        else:
            return {
                "inserted_count": result.inserted_count,
                "matched_count": result.matched_count,
                "modified_count": result.modified_count,
                "deleted_count": result.deleted_count,
                "upserted_count": result.upserted_count,
                "upserted_ids": result.upserted_ids,
            }
    else:
        # Fallback: execute operations one by one
        inserted_count = 0
        matched_count = 0
        modified_count = 0
        deleted_count = 0
        upserted_count = 0
        upserted_ids: Dict[int, str] = {}

        for idx, op in enumerate(operations):
            op_type = op.get("op")
            try:
                if op_type == "insert_one":
                    await insert_one(collection, op["document"])
                    inserted_count += 1
                elif op_type == "update_one":
                    result = await update_one_with_options(
                        collection, op["filter"], op["update"], op.get("upsert", False)
                    )
                    matched_count += result.get("matched_count", 0)
                    modified_count += result.get("modified_count", 0)
                    if result.get("upserted_id"):
                        upserted_count += 1
                        upserted_ids[idx] = result["upserted_id"]
                elif op_type == "update_many":
                    result = await update_many_with_options(
                        collection, op["filter"], op["update"], op.get("upsert", False)
                    )
                    matched_count += result.get("matched_count", 0)
                    modified_count += result.get("modified_count", 0)
                    if result.get("upserted_id"):
                        upserted_count += 1
                        upserted_ids[idx] = result["upserted_id"]
                elif op_type == "delete_one":
                    deleted_count += await delete_one(collection, op["filter"])
                elif op_type == "delete_many":
                    deleted_count += await delete_many(collection, op["filter"])
                elif op_type == "replace_one":
                    result = await replace_one(
                        collection, op["filter"], op["replacement"], op.get("upsert", False)
                    )
                    matched_count += result.get("matched_count", 0)
                    modified_count += result.get("modified_count", 0)
                    if result.get("upserted_id"):
                        upserted_count += 1
                        upserted_ids[idx] = result["upserted_id"]
            except Exception as e:
                if ordered:
                    raise
                # In unordered mode, continue with remaining operations
                continue

        return {
            "inserted_count": inserted_count,
            "matched_count": matched_count,
            "modified_count": modified_count,
            "deleted_count": deleted_count,
            "upserted_count": upserted_count,
            "upserted_ids": upserted_ids,
        }


# ===================
# Index Management
# ===================


async def create_index(
    collection: str,
    keys: List[tuple],
    **options: Any,
) -> str:
    """
    Create an index on a collection.

    Args:
        collection: Collection name
        keys: List of (field, direction) tuples, e.g., [("email", 1)]
        **options: Index options (unique, sparse, name, expire_after_seconds)

    Returns:
        Name of the created index
    """
    # Convert keys list to dict for Rust
    keys_dict = {k: v for k, v in keys}

    # Convert options
    opts = {}
    if options.get("unique"):
        opts["unique"] = True
    if options.get("sparse"):
        opts["sparse"] = True
    if options.get("name"):
        opts["name"] = options["name"]
    if "expire_after_seconds" in options or "expireAfterSeconds" in options:
        ttl = options.get("expire_after_seconds") or options.get("expireAfterSeconds")
        opts["expire_after_seconds"] = ttl
    if options.get("background"):
        opts["background"] = True

    return await _rust.Document.create_index(collection, keys_dict, opts if opts else None)


async def list_indexes(collection: str) -> List[Dict[str, Any]]:
    """
    List all indexes on a collection.

    Args:
        collection: Collection name

    Returns:
        List of index information dicts
    """
    return await _rust.Document.list_indexes(collection)


async def drop_index(collection: str, index_name: str) -> None:
    """
    Drop an index from a collection.

    Args:
        collection: Collection name
        index_name: Name of the index to drop
    """
    return await _rust.Document.drop_index(collection, index_name)


# ===================
# Collection Management
# ===================


async def create_collection(
    collection: str,
    options: Optional[Dict[str, Any]] = None,
) -> bool:
    """
    Create a collection with options (including time-series).

    This function creates a collection if it doesn't already exist.
    For time-series collections, the appropriate options must be provided.

    Args:
        collection: Collection name
        options: Collection options, including:
            - timeseries: Time-series configuration with:
                - timeField: Field containing timestamp
                - metaField: Field containing metadata (optional)
                - granularity: "seconds", "minutes", or "hours" (optional)
                - bucketMaxSpanSeconds: Max bucket span (MongoDB 6.3+, optional)
                - bucketRoundingSeconds: Bucket rounding (MongoDB 6.3+, optional)
            - expireAfterSeconds: TTL for automatic document deletion (optional)

    Returns:
        True if collection was created, False if it already existed

    Example:
        >>> # Create time-series collection
        >>> await create_collection("sensor_readings", {
        ...     "timeseries": {
        ...         "timeField": "timestamp",
        ...         "metaField": "sensor_id",
        ...         "granularity": "minutes",
        ...     },
        ...     "expireAfterSeconds": 86400 * 30,  # 30 days
        ... })
    """
    # Check if Rust has create_collection
    if hasattr(_rust.Document, "create_collection"):
        return await _rust.Document.create_collection(collection, options)
    else:
        # Fallback: Try to create via database command
        # This is a best-effort fallback; may not work for all options
        try:
            # Try inserting and deleting a dummy doc to create the collection
            # This won't work for time-series which require explicit creation
            if options and "timeseries" in options:
                raise NotImplementedError(
                    "Time-series collections require Rust backend support. "
                    "Rebuild with: maturin develop"
                )
            # For regular collections, just return True (they're created on first insert)
            return True
        except Exception:
            return False


# ===================
# Link Fetching
# ===================


async def fetch_links_batched(
    docs: List[Dict[str, Any]],
    link_fields: List[Any],
    max_depth: int = 1,
) -> List[Dict[str, Any]]:
    """
    Fetch linked documents in batches via Rust.

    This delegates to Rust's fetch_links_batched for efficient batched
    link resolution. Python only passes parameters; the batching logic
    is in Rust.

    Args:
        docs: List of documents (as dicts) to fetch links for
        link_fields: List of LinkField objects describing the links
        max_depth: Maximum depth for recursive link fetching (1-5)

    Returns:
        List of documents with links populated (as dicts)
    """
    if hasattr(_rust, "fetch_links_batched"):
        return await _rust.fetch_links_batched(docs, link_fields, max_depth)
    else:
        # Fallback: return docs unchanged (links not fetched)
        return docs
