"""Example demonstrating PostgreSQL migrations in data-bridge.

This example shows how to:
1. Initialize the migration system
2. Check migration status
3. Apply pending migrations
4. Rollback migrations
5. Create new migration files

Make sure PostgreSQL is running and accessible before running this example.
"""

import asyncio
from cclab import postgres


async def main():
    # Connect to PostgreSQL
    print("Connecting to PostgreSQL...")

    # Get connection string from environment or use default
    import os
    connection_string = os.environ.get(
        "POSTGRES_URI",
        "postgresql://rstn:rstn@localhost:5432/cclab_test"
    )

    await postgres.init(
        connection_string,
        max_connections=5
    )

    # Initialize migration system (create _migrations table)
    print("\n1. Initializing migration system...")
    await postgres.migration_init()
    print("   Migration system initialized!")

    # Check migration status
    print("\n2. Checking migration status...")
    status = await postgres.migration_status("examples/postgres_migrations")
    print(f"   Applied migrations: {status['applied']}")
    print(f"   Pending migrations: {status['pending']}")

    # Apply pending migrations
    if status['pending']:
        print("\n3. Applying pending migrations...")
        applied = await postgres.migration_apply("examples/postgres_migrations")
        print(f"   Applied: {applied}")
    else:
        print("\n3. No pending migrations to apply")

    # Check status again
    print("\n4. Checking status after migration...")
    status = await postgres.migration_status("examples/postgres_migrations")
    print(f"   Applied migrations: {status['applied']}")
    print(f"   Pending migrations: {status['pending']}")

    # Verify tables were created
    print("\n5. Verifying created tables...")
    tables = await postgres.list_tables()
    print(f"   Tables: {tables}")

    # Insert test data
    print("\n6. Inserting test data...")
    user = await postgres.insert_one(
        "users",
        {
            "email": "alice@example.com",
            "name": "Alice",
            "status": "active"
        }
    )
    print(f"   Created user: {user}")

    post = await postgres.insert_one(
        "posts",
        {
            "user_id": user["id"],
            "title": "My First Post",
            "content": "Hello, world!",
            "published": True
        }
    )
    print(f"   Created post: {post}")

    # Query data
    print("\n7. Querying data...")
    users = await postgres.fetch_all("users", {})
    print(f"   Users: {users}")

    posts = await postgres.fetch_all("posts", {})
    print(f"   Posts: {posts}")

    # Rollback last migration (posts table)
    print("\n8. Rolling back last migration...")
    reverted = await postgres.migration_rollback("examples/postgres_migrations", steps=1)
    print(f"   Reverted: {reverted}")

    # Check tables again
    print("\n9. Verifying tables after rollback...")
    tables = await postgres.list_tables()
    print(f"   Tables: {tables}")

    # Re-apply the migration
    print("\n10. Re-applying migrations...")
    applied = await postgres.migration_apply("examples/postgres_migrations")
    print(f"   Applied: {applied}")

    # Create a new migration file
    print("\n11. Creating new migration...")
    filepath = postgres.migration_create(
        "add user profile",
        "examples/postgres_migrations"
    )
    print(f"   Created migration file: {filepath}")
    print("\n   Edit the file to add your migration SQL, then run:")
    print("   await postgres.migration_apply('examples/postgres_migrations')")

    # Close connection
    print("\n12. Closing connection...")
    await postgres.close()
    print("   Done!")


if __name__ == "__main__":
    asyncio.run(main())
