"""
Example: Raw SQL Execution with data-bridge-postgres

This example demonstrates how to execute raw SQL queries using the execute() function.
This is useful for power users who need features not available in the ORM, such as:
- Complex joins
- Window functions
- Custom aggregations
- DDL operations
- Database-specific functions

IMPORTANT: Always use parameterized queries ($1, $2, etc.) to prevent SQL injection!
"""

import asyncio
import os
from cclab.titan import init, close, execute


async def main():
    # Initialize PostgreSQL connection
    # IMPORTANT: Use environment variables for credentials in production
    await init(
        host=os.environ.get("PG_HOST", "localhost"),
        port=int(os.environ.get("PG_PORT", "5432")),
        database=os.environ.get("PG_DATABASE", "mydb"),
        username=os.environ.get("PG_USER", "user"),
        password=os.environ.get("PG_PASSWORD", "password")  # Default for local dev only
    )

    try:
        # Example 1: SELECT query with parameters
        print("=== Example 1: SELECT Query ===")
        users = await execute(
            "SELECT * FROM users WHERE age > $1 ORDER BY name LIMIT $2",
            [25, 10]
        )
        print(f"Found {len(users)} users")
        for user in users:
            print(f"  - {user['name']}: {user['age']} years old")

        # Example 2: INSERT query
        print("\n=== Example 2: INSERT Query ===")
        count = await execute(
            "INSERT INTO users (name, email, age) VALUES ($1, $2, $3)",
            ["Alice", "alice@example.com", 30]
        )
        print(f"Inserted {count} row(s)")

        # Example 3: UPDATE query
        print("\n=== Example 3: UPDATE Query ===")
        count = await execute(
            "UPDATE users SET age = age + 1 WHERE name = $1",
            ["Alice"]
        )
        print(f"Updated {count} row(s)")

        # Example 4: Complex query with joins
        print("\n=== Example 4: Complex JOIN Query ===")
        results = await execute("""
            SELECT u.name, u.age, o.product_name, o.quantity
            FROM users u
            INNER JOIN orders o ON u.id = o.user_id
            WHERE u.age > $1 AND o.quantity > $2
            ORDER BY o.created_at DESC
            LIMIT $3
        """, [18, 1, 20])
        print(f"Found {len(results)} orders")
        for row in results:
            print(f"  - {row['name']} ordered {row['quantity']} x {row['product_name']}")

        # Example 5: Aggregate query
        print("\n=== Example 5: Aggregate Query ===")
        stats = await execute("""
            SELECT
                COUNT(*) as total_users,
                AVG(age) as avg_age,
                MIN(age) as min_age,
                MAX(age) as max_age
            FROM users
            WHERE age > $1
        """, [0])
        if stats:
            print(f"  Total users: {stats[0]['total_users']}")
            print(f"  Average age: {stats[0]['avg_age']:.1f}")
            print(f"  Age range: {stats[0]['min_age']} - {stats[0]['max_age']}")

        # Example 6: DELETE query
        print("\n=== Example 6: DELETE Query ===")
        count = await execute(
            "DELETE FROM users WHERE age < $1",
            [18]
        )
        print(f"Deleted {count} row(s)")

        # Example 7: Window function (PostgreSQL-specific)
        print("\n=== Example 7: Window Function ===")
        ranked = await execute("""
            SELECT
                name,
                age,
                RANK() OVER (ORDER BY age DESC) as age_rank,
                ROW_NUMBER() OVER (PARTITION BY city ORDER BY age DESC) as city_rank
            FROM users
            WHERE age > $1
            LIMIT $2
        """, [20, 10])
        for row in ranked:
            print(f"  - {row['name']}: Age rank #{row['age_rank']}, City rank #{row['city_rank']}")

        # Example 8: DDL operation (create index)
        print("\n=== Example 8: DDL Operation ===")
        result = await execute("CREATE INDEX IF NOT EXISTS idx_users_age ON users(age)")
        print(f"Index created (result: {result})")

        # Example 9: DDL operation (add column)
        print("\n=== Example 9: Alter Table ===")
        result = await execute(
            "ALTER TABLE users ADD COLUMN IF NOT EXISTS last_login TIMESTAMP"
        )
        print(f"Column added (result: {result})")

        # Example 10: Using BETWEEN and IN operators
        print("\n=== Example 10: BETWEEN and IN Operators ===")
        users = await execute("""
            SELECT name, age, city
            FROM users
            WHERE age BETWEEN $1 AND $2
              AND city = ANY($3::text[])
            ORDER BY age
        """, [25, 35, ["NYC", "SF", "LA"]])
        print(f"Found {len(users)} users in target cities and age range")

        # Example 11: WITH (CTE) query
        print("\n=== Example 11: Common Table Expression (CTE) ===")
        results = await execute("""
            WITH recent_orders AS (
                SELECT user_id, COUNT(*) as order_count
                FROM orders
                WHERE created_at > NOW() - INTERVAL '30 days'
                GROUP BY user_id
            )
            SELECT u.name, u.email, COALESCE(ro.order_count, 0) as recent_orders
            FROM users u
            LEFT JOIN recent_orders ro ON u.id = ro.user_id
            WHERE u.age > $1
            ORDER BY recent_orders DESC
            LIMIT $2
        """, [21, 10])
        for row in results:
            print(f"  - {row['name']}: {row['recent_orders']} recent orders")

        # Example 12: Query with NULL handling
        print("\n=== Example 12: NULL Handling ===")
        count = await execute(
            "INSERT INTO users (name, email, age, city) VALUES ($1, $2, $3, $4)",
            ["Bob", None, 28, "LA"]  # NULL email
        )
        print(f"Inserted {count} row(s) with NULL email")

    finally:
        # Always close the connection when done
        await close()


if __name__ == "__main__":
    asyncio.run(main())
