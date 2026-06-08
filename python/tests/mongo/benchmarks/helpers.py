"""
Helper functions and constants for benchmark tests.
"""

from typing import List, Dict, Any


# Batch sizes for parametrized tests
BATCH_SIZES = [10, 100, 1000, 10000, 50000]


def generate_user_data(count: int) -> List[Dict[str, Any]]:
    """
    Generate test user documents with rich schema.

    Args:
        count: Number of documents to generate

    Returns:
        List of dictionaries representing user documents
    """
    return [
        {
            "name": f"User{i}",
            "email": f"user{i}@example.com",
            "age": 20 + (i % 50),
            "city": ["NYC", "LA", "SF", "Chicago", "Boston"][i % 5],
            "score": float(i * 1.5),
            "active": i % 2 == 0,
        }
        for i in range(count)
    ]


def get_collection_name(framework: str, operation: str, batch_size: int = None) -> str:
    """
    Generate unique collection name per framework/operation/batch.

    Ensures complete isolation to prevent data interference.

    Args:
        framework: Framework identifier (e.g., "cclab", "beanie")
        operation: Operation name (e.g., "insert_bulk", "find_one")
        batch_size: Optional batch size for further isolation

    Returns:
        Unique collection name
    """
    if batch_size is not None:
        return f"bench_{framework}_{operation}_{batch_size}"
    return f"bench_{framework}_{operation}"


def get_benchmark_params(batch_size: int) -> Dict[str, int]:
    """
    Calculate adaptive benchmark parameters based on batch size.

    Scales down iterations for larger batches to keep total time reasonable.

    Args:
        batch_size: Number of documents in the batch

    Returns:
        Dictionary with 'iterations' and 'rounds' keys
    """
    if batch_size <= 100:
        iterations = 50
        rounds = 5
    elif batch_size <= 1000:
        iterations = 20
        rounds = 5
    elif batch_size <= 10000:
        iterations = 10
        rounds = 3
    else:  # 50000
        iterations = 3
        rounds = 3

    return {"iterations": iterations, "rounds": rounds}
