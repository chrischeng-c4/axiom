"""
Example: Using aggregate functions with data-bridge PostgreSQL.

This example demonstrates how to use the query_aggregate function
to perform various aggregate operations on PostgreSQL tables.
"""
import asyncio
from cclab._engine import postgres


async def main():
    """Run aggregate query examples."""

    # Initialize connection
    await postgres.init("postgresql://user:password@localhost:5432/mydb")

    # Example 1: Simple COUNT(*)
    print("Example 1: Count all orders")
    results = await postgres.query_aggregate(
        "orders",
        [("count", None, "total_orders")],
        group_by=None,
        where_conditions=None,
        order_by=None,
        limit=None
    )
    print(f"Total orders: {results[0]['total_orders']}\n")

    # Example 2: SUM with WHERE clause
    print("Example 2: Sum of completed order amounts")
    results = await postgres.query_aggregate(
        "orders",
        [("sum", "amount", "total_revenue")],
        group_by=None,
        where_conditions=[("status", "eq", "completed")],
        order_by=None,
        limit=None
    )
    print(f"Total revenue: ${results[0]['total_revenue']}\n")

    # Example 3: GROUP BY with multiple aggregates
    print("Example 3: Order statistics by user")
    results = await postgres.query_aggregate(
        "orders",
        [
            ("count", None, "order_count"),
            ("sum", "amount", "total_spent"),
            ("avg", "amount", "avg_order_value"),
            ("min", "amount", "min_order"),
            ("max", "amount", "max_order"),
        ],
        group_by=["user_id"],
        where_conditions=None,
        order_by=[("total_spent", "desc")],
        limit=10
    )

    for row in results:
        print(f"User {row['user_id']}:")
        print(f"  Orders: {row['order_count']}")
        print(f"  Total spent: ${row['total_spent']:.2f}")
        print(f"  Average order: ${row['avg_order_value']:.2f}")
        print(f"  Min/Max: ${row['min_order']:.2f} / ${row['max_order']:.2f}")
        print()

    # Example 4: COUNT DISTINCT
    print("Example 4: Count unique customers")
    results = await postgres.query_aggregate(
        "orders",
        [("count_distinct", "user_id", "unique_customers")],
        group_by=None,
        where_conditions=None,
        order_by=None,
        limit=None
    )
    print(f"Unique customers: {results[0]['unique_customers']}\n")

    # Example 5: Complex query with multiple conditions
    print("Example 5: Revenue by category for large orders in 2024")
    results = await postgres.query_aggregate(
        "orders",
        [
            ("count", None, "order_count"),
            ("sum", "amount", "revenue"),
            ("avg", "amount", "avg_amount"),
        ],
        group_by=["category"],
        where_conditions=[
            ("amount", "gte", 100),
            ("created_at", "gte", "2024-01-01"),
            ("status", "eq", "completed"),
        ],
        order_by=[("revenue", "desc")],
        limit=5
    )

    print("Top 5 categories by revenue (orders >= $100 in 2024):")
    for row in results:
        print(f"{row['category']}: ${row['revenue']:.2f} ({row['order_count']} orders)")
    print()

    # Example 6: Using different operators
    print("Example 6: Orders with amount between $50 and $200")
    results = await postgres.query_aggregate(
        "orders",
        [("count", None, "count")],
        group_by=None,
        where_conditions=[
            ("amount", "gte", 50),
            ("amount", "lte", 200),
        ],
        order_by=None,
        limit=None
    )
    print(f"Orders in range: {results[0]['count']}\n")

    # Close connection
    await postgres.close()


if __name__ == "__main__":
    asyncio.run(main())
