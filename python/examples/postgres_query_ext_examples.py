"""
Examples demonstrating SQLAlchemy-style query enhancements.

This module shows how to use the query_ext utilities for building
complex, composable queries with a SQLAlchemy-like API.
"""

import asyncio
from datetime import datetime, timedelta
from cclab.titan import (
    Table, Column, init, close,
    filter_by, and_, or_, not_,
    QueryFragment, aliased,
    active_filter, date_range_filter, in_list_filter, null_check_filter
)


# Define example models
class User(Table):
    """User model with various fields."""
    name: str
    email: str
    age: int
    status: str
    role: str
    verified: bool
    created_at: datetime

    class Settings:
        table_name = "users"


class Post(Table):
    """Post model."""
    title: str
    content: str
    user_id: int
    views: int
    status: str
    created_at: datetime

    class Settings:
        table_name = "posts"


async def example_filter_by():
    """Example: Using filter_by() for simple equality filters."""
    print("\n=== Example: filter_by() ===")

    # Simple equality filters
    users = await User.find(*filter_by(status="active")).to_list()
    print(f"Active users: {len(users)}")

    # Multiple conditions
    users = await User.find(*filter_by(
        status="active",
        role="admin",
        verified=True
    )).to_list()
    print(f"Active admin users: {len(users)}")

    # Combine with other filters
    users = await User.find(
        *filter_by(status="active"),
        User.age > 25
    ).to_list()
    print(f"Active users over 25: {len(users)}")


async def example_boolean_combinators():
    """Example: Using and_(), or_(), not_() for boolean logic."""
    print("\n=== Example: Boolean Combinators ===")

    # AND condition
    users = await User.find(
        and_(User.age > 18, User.status == "active")
    ).to_list()
    print(f"Adult active users: {len(users)}")

    # OR condition
    users = await User.find(
        or_(User.role == "admin", User.role == "moderator")
    ).to_list()
    print(f"Admins or moderators: {len(users)}")

    # NOT condition
    users = await User.find(
        not_(User.status == "deleted")
    ).to_list()
    print(f"Non-deleted users: {len(users)}")

    # Complex nested logic: (age > 18) AND ((role = admin) OR (role = moderator))
    users = await User.find(
        and_(
            User.age > 18,
            or_(User.role == "admin", User.role == "moderator")
        )
    ).to_list()
    print(f"Adult admins/moderators: {len(users)}")


async def example_query_fragments():
    """Example: Using QueryFragment for reusable conditions."""
    print("\n=== Example: QueryFragment ===")

    # Define reusable fragments
    active = QueryFragment(status="active")
    verified = QueryFragment(verified=True)
    adult = QueryFragment(age__gt=18)

    # Use individually
    users = await User.find(*active).to_list()
    print(f"Active users: {len(users)}")

    # Combine fragments with &
    users = await User.find(active & verified).to_list()
    print(f"Active verified users: {len(users)}")

    # Combine multiple fragments
    users = await User.find(active & verified & adult).to_list()
    print(f"Active verified adults: {len(users)}")

    # Negate a fragment
    not_deleted = ~QueryFragment(status="deleted")
    users = await User.find(not_deleted).to_list()
    print(f"Non-deleted users: {len(users)}")


async def example_django_style_lookups():
    """Example: Django-style field lookups in QueryFragment."""
    print("\n=== Example: Django-style Lookups ===")

    # Greater than
    fragment = QueryFragment(age__gt=18)
    users = await User.find(*fragment).to_list()
    print(f"Users over 18: {len(users)}")

    # Greater than or equal
    fragment = QueryFragment(age__gte=21)
    users = await User.find(*fragment).to_list()
    print(f"Users 21+: {len(users)}")

    # Less than
    fragment = QueryFragment(age__lt=65)
    users = await User.find(*fragment).to_list()
    print(f"Users under 65: {len(users)}")

    # IN lookup
    fragment = QueryFragment(role__in=["admin", "moderator", "editor"])
    users = await User.find(*fragment).to_list()
    print(f"Staff users: {len(users)}")

    # IS NULL
    fragment = QueryFragment(deleted_at__isnull=True)
    users = await User.find(*fragment).to_list()
    print(f"Non-deleted users: {len(users)}")

    # LIKE
    fragment = QueryFragment(email__like="%@example.com")
    users = await User.find(*fragment).to_list()
    print(f"Example.com users: {len(users)}")


async def example_convenience_filters():
    """Example: Using convenience filter functions."""
    print("\n=== Example: Convenience Filters ===")

    # Active filter
    users = await User.find(*active_filter()).to_list()
    print(f"Active users: {len(users)}")

    # Custom active filter
    users = await User.find(*active_filter(column="state", value="enabled")).to_list()
    print(f"Enabled users: {len(users)}")

    # Date range filter
    now = datetime.now()
    week_ago = now - timedelta(days=7)
    posts = await Post.find(*date_range_filter("created_at", week_ago, now)).to_list()
    print(f"Posts from last week: {len(posts)}")

    # IN list filter
    users = await User.find(*in_list_filter("role", ["admin", "moderator"])).to_list()
    print(f"Admins and moderators: {len(users)}")

    # NULL check filter
    users = await User.find(*null_check_filter("deleted_at", is_null=True)).to_list()
    print(f"Non-deleted users: {len(users)}")


async def example_complex_scenarios():
    """Example: Complex real-world query scenarios."""
    print("\n=== Example: Complex Scenarios ===")

    # Scenario 1: Find active, verified users over 18 who are staff
    active = active_filter()
    verified = QueryFragment(verified=True)
    adult = QueryFragment(age__gt=18)
    staff = or_(
        User.role == "admin",
        User.role == "moderator",
        User.role == "editor"
    )

    users = await User.find(active & verified & adult, staff).to_list()
    print(f"Active verified adult staff: {len(users)}")

    # Scenario 2: Find popular recent posts
    now = datetime.now()
    week_ago = now - timedelta(days=7)
    recent = date_range_filter("created_at", week_ago, now)
    popular = QueryFragment(views__gt=1000)
    published = QueryFragment(status="published")

    posts = await Post.find(recent & popular & published).to_list()
    print(f"Popular recent posts: {len(posts)}")

    # Scenario 3: Find users who need attention
    # (inactive for 30 days OR never verified) AND NOT deleted
    thirty_days_ago = datetime.now() - timedelta(days=30)
    needs_attention = or_(
        QueryFragment(last_login__lt=thirty_days_ago),
        and_(
            QueryFragment(verified=False),
            QueryFragment(created_at__lt=thirty_days_ago)
        )
    )
    not_deleted = ~QueryFragment(status="deleted")

    users = await User.find(needs_attention, not_deleted).to_list()
    print(f"Users needing attention: {len(users)}")


async def example_operator_overloading():
    """Example: Using operator overloading for cleaner syntax."""
    print("\n=== Example: Operator Overloading ===")

    # Using & for AND
    users = await User.find(
        (User.age > 18) & (User.status == "active")
    ).to_list()
    print(f"Adult active users: {len(users)}")

    # Using | for OR
    users = await User.find(
        (User.role == "admin") | (User.role == "moderator")
    ).to_list()
    print(f"Admins or moderators: {len(users)}")

    # Complex expression with operators
    condition = (
        (User.age > 18) &
        (User.verified == True) &
        ((User.role == "admin") | (User.role == "moderator"))
    )
    users = await User.find(condition).to_list()
    print(f"Verified adult staff: {len(users)}")


async def example_reusable_filters():
    """Example: Creating reusable filter libraries."""
    print("\n=== Example: Reusable Filter Library ===")

    # Define a library of reusable filters
    class UserFilters:
        """Common user filters."""

        active = active_filter()
        verified = QueryFragment(verified=True)
        adult = QueryFragment(age__gte=18)
        senior = QueryFragment(age__gte=65)
        new_users = date_range_filter("created_at", datetime.now() - timedelta(days=30))

        admin = QueryFragment(role="admin")
        moderator = QueryFragment(role="moderator")
        staff = or_(
            User.role == "admin",
            User.role == "moderator",
            User.role == "editor"
        )

        premium = QueryFragment(subscription_tier__in=["pro", "enterprise"])
        trial = QueryFragment(subscription_tier="trial")

    # Use the filter library
    users = await User.find(UserFilters.active & UserFilters.verified).to_list()
    print(f"Active verified users: {len(users)}")

    users = await User.find(UserFilters.active & UserFilters.staff).to_list()
    print(f"Active staff: {len(users)}")

    users = await User.find(UserFilters.new_users & UserFilters.premium).to_list()
    print(f"New premium users: {len(users)}")


async def example_self_joins_with_aliased():
    """Example: Self-joins using aliased()."""
    print("\n=== Example: Self-Joins with aliased() ===")

    # This is a placeholder example showing the concept
    # Full implementation requires join() support in QueryBuilder

    # Create an alias for the Employee table
    Manager = aliased(User, "manager")

    # In a real implementation, this would work like:
    # employees = await User.find() \
    #     .join(Manager, User.manager_id == Manager.id) \
    #     .select(User.name, Manager.name.label("manager_name")) \
    #     .to_list()

    print("Self-joins with aliased() require join() method implementation")
    print(f"Manager alias created: {Manager}")


async def main():
    """Run all examples."""
    # Initialize database connection
    # Note: Replace with your actual connection string
    # await init("postgresql://user:pass@localhost/dbname")

    print("=" * 60)
    print("SQLAlchemy-Style Query Extensions Examples")
    print("=" * 60)

    # Run examples (commented out as they require a real database)
    # await example_filter_by()
    # await example_boolean_combinators()
    # await example_query_fragments()
    # await example_django_style_lookups()
    # await example_convenience_filters()
    # await example_complex_scenarios()
    # await example_operator_overloading()
    # await example_reusable_filters()
    # await example_self_joins_with_aliased()

    # Demonstrate without database (just showing the API)
    print("\n=== API Demonstration (no database required) ===\n")

    # Create example filters
    active = active_filter()
    print(f"Active filter: {active}")

    verified = QueryFragment(verified=True, age__gt=18)
    print(f"Verified adult fragment: {verified}")

    combined = active & verified
    print(f"Combined fragment: {combined}")

    boolean_expr = and_(User.age > 18, or_(User.role == "admin", User.role == "moderator"))
    print(f"Boolean expression: {boolean_expr}")

    # Show SQL generation
    sql, params = boolean_expr.to_sql()
    print(f"\nGenerated SQL: {sql}")
    print(f"Parameters: {params}")

    # Close database connection
    # await close()

    print("\n" + "=" * 60)
    print("Examples complete!")
    print("=" * 60)


if __name__ == "__main__":
    asyncio.run(main())
