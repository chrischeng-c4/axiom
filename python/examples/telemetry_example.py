"""
Example demonstrating OpenTelemetry integration with data-bridge PostgreSQL ORM.

This example shows how to:
1. Use instrumentation decorators
2. Create custom spans with database attributes
3. Track connection pool metrics
4. Handle exceptions in spans

Setup:
    pip install opentelemetry-api opentelemetry-sdk opentelemetry-exporter-jaeger

Run:
    # With tracing enabled (default)
    python examples/telemetry_example.py

    # With tracing disabled
    DATA_BRIDGE_TRACING_ENABLED=false python examples/telemetry_example.py
"""

import asyncio
from cclab.titan.telemetry import (
    is_tracing_enabled,
    create_query_span,
    create_session_span,
    create_relationship_span,
    instrument_span,
    instrument_query,
    instrument_session,
    add_exception,
    set_span_result,
    get_connection_pool_metrics,
    SpanAttributes,
)


# Example 1: Using decorators for automatic instrumentation
@instrument_query("find")
async def find_users(min_age: int):
    """Find users with age greater than min_age."""
    print(f"Finding users with age > {min_age}")
    # Simulated query execution
    await asyncio.sleep(0.01)
    return [
        {"id": 1, "name": "Alice", "age": 30},
        {"id": 2, "name": "Bob", "age": 35},
    ]


@instrument_session("flush")
async def flush_session():
    """Flush pending changes to database."""
    print("Flushing session...")
    # Simulated flush operation
    await asyncio.sleep(0.02)


@instrument_span("custom.operation", attributes={"component": "example"})
async def custom_operation():
    """Custom operation with span."""
    print("Executing custom operation")
    await asyncio.sleep(0.01)
    return "done"


# Example 2: Using span context managers for fine-grained control
async def complex_query_with_spans():
    """Example showing manual span creation with detailed attributes."""
    print("\n=== Complex Query with Manual Spans ===")

    # Query span with all attributes
    with create_query_span(
        operation="find",
        table="users",
        filters_count=3,
        limit=10,
        offset=0,
        order_by="created_at DESC",
    ) as span:
        try:
            # Simulated query execution
            await asyncio.sleep(0.01)
            result_count = 10

            # Set result information
            set_span_result(span, count=result_count)
            print(f"Query returned {result_count} results")

        except Exception as e:
            # Record exception in span
            add_exception(span, e)
            raise


# Example 3: Session span with state tracking
async def session_operation_with_spans():
    """Example showing session span with pending/dirty/deleted counts."""
    print("\n=== Session Operation with Spans ===")

    with create_session_span(
        operation="flush",
        pending_count=5,
        dirty_count=3,
        deleted_count=1,
    ) as span:
        try:
            # Simulated flush operation
            await asyncio.sleep(0.02)
            print("Session flushed successfully")

        except Exception as e:
            add_exception(span, e)
            raise


# Example 4: Relationship loading span
async def relationship_loading_with_spans():
    """Example showing relationship loading with different strategies."""
    print("\n=== Relationship Loading with Spans ===")

    # Lazy loading
    with create_relationship_span(
        name="user.posts",
        strategy="lazy",
        depth=0,
    ) as span:
        await asyncio.sleep(0.01)
        print("Loaded relationship 'user.posts' using lazy strategy")

    # Eager loading (selectin)
    with create_relationship_span(
        name="post.comments",
        strategy="selectin",
        depth=1,
    ) as span:
        await asyncio.sleep(0.015)
        set_span_result(span, count=25)
        print("Loaded relationship 'post.comments' using selectin strategy (25 items)")


# Example 5: Connection pool metrics
def connection_pool_metrics_example():
    """Example showing connection pool metrics recording."""
    print("\n=== Connection Pool Metrics ===")

    metrics = get_connection_pool_metrics()

    # Simulate connection pool state changes
    metrics.record_pool_stats(in_use=5, idle=3, max_size=10)
    print("Recorded pool stats: in_use=5, idle=3, max_size=10")

    metrics.record_pool_stats(in_use=7, idle=1, max_size=10)
    print("Recorded pool stats: in_use=7, idle=1, max_size=10")


# Example 6: Error handling with spans
async def error_handling_example():
    """Example showing exception handling in spans."""
    print("\n=== Error Handling with Spans ===")

    with create_query_span(
        operation="insert",
        table="users",
    ) as span:
        try:
            # Simulated error
            raise ValueError("Duplicate key violation")

        except ValueError as e:
            # Record exception
            add_exception(span, e)
            print(f"Caught and recorded exception: {e}")
            # Don't re-raise in this example


async def main():
    """Run all examples."""
    print("OpenTelemetry Integration Examples")
    print("=" * 50)
    print(f"Tracing enabled: {is_tracing_enabled()}\n")

    # Example 1: Decorated functions
    print("=== Using Decorators ===")
    users = await find_users(min_age=25)
    print(f"Found {len(users)} users\n")

    await flush_session()
    print()

    result = await custom_operation()
    print(f"Custom operation result: {result}\n")

    # Example 2: Manual spans
    await complex_query_with_spans()

    # Example 3: Session spans
    await session_operation_with_spans()

    # Example 4: Relationship loading
    await relationship_loading_with_spans()

    # Example 5: Connection pool metrics
    connection_pool_metrics_example()

    # Example 6: Error handling
    await error_handling_example()

    print("\n" + "=" * 50)
    print("All examples completed!")


if __name__ == "__main__":
    # Optional: Set up OpenTelemetry SDK
    # This is commented out to avoid requiring OpenTelemetry SDK installation
    # Uncomment to see actual tracing output

    """
    from opentelemetry import trace
    from opentelemetry.sdk.trace import TracerProvider
    from opentelemetry.sdk.trace.export import ConsoleSpanExporter, SimpleSpanProcessor

    # Set up tracer provider
    provider = TracerProvider()
    processor = SimpleSpanProcessor(ConsoleSpanExporter())
    provider.add_span_processor(processor)
    trace.set_tracer_provider(provider)
    """

    # Run examples
    asyncio.run(main())
