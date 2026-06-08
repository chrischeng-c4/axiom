"""Example demonstrating computed attributes in data-bridge PostgreSQL ORM (no DB required).

This example shows:
1. @hybrid_property - Works as Python property AND generates SQL
2. @hybrid_method - Like hybrid_property but with arguments
3. column_property() - Read-only computed column from SQL
4. Computed - PostgreSQL GENERATED ALWAYS AS columns
5. default_factory() - Column defaults from callables
"""

from datetime import datetime
from cclab.titan import Table, Column
from cclab.titan.computed import (
    hybrid_property,
    hybrid_method,
    column_property,
    Computed,
    default_factory,
)


class User(Table):
    """User table with hybrid properties and methods."""

    first_name: str
    last_name: str
    age: int
    created_at: datetime = Column(
        default_factory=default_factory(datetime.utcnow)
    )

    @hybrid_property
    def full_name(self):
        """Full name property that works in Python and SQL."""
        return f"{self.first_name} {self.last_name}"

    @full_name.setter
    def full_name(self, value):
        """Setter to split full name into first and last."""
        parts = value.split(" ", 1)
        self.first_name = parts[0]
        self.last_name = parts[1] if len(parts) > 1 else ""

    @full_name.expression
    def full_name(cls):
        """SQL expression for full_name."""
        from cclab.titan.columns import SqlExpr
        # For queries, this would generate: first_name || ' ' || last_name
        return SqlExpr("first_name || ' ' || last_name", "RAW", None)

    @hybrid_method
    def is_older_than(self, min_age):
        """Check if user is older than specified age (works in Python and SQL)."""
        return self.age > min_age

    @is_older_than.expression
    def is_older_than(cls, min_age):
        """SQL expression for is_older_than."""
        from cclab.titan.columns import SqlExpr
        return SqlExpr("age", ">", min_age)

    @hybrid_property
    def is_adult(self):
        """Check if user is an adult."""
        return self.age >= 18


class Product(Table):
    """Product table with computed columns."""

    name: str
    price: float
    quantity: int
    tax_rate: float = 0.1

    # Stored computed column - computed once and stored on disk
    total_value = Computed("price * quantity", stored=True)

    # Virtual computed column - computed on every read
    tax_amount = Computed("price * tax_rate", stored=False)


class Order(Table):
    """Order table with column_property."""

    amount: float
    tax_rate: float = 0.1
    discount_rate: float = 0.0

    # Read-only computed column (computed at SQL level when loaded)
    # Note: This is just a descriptor, actual SQL computation would need
    # to be implemented in the query layer
    subtotal = column_property("amount * (1 - discount_rate)")
    tax = column_property("amount * (1 - discount_rate) * tax_rate")
    total = column_property("amount * (1 - discount_rate) * (1 + tax_rate)")


def main():
    """Run examples of computed attributes."""
    print("=== Computed Attributes Examples (No DB Required) ===\n")

    # Example 1: hybrid_property
    print("1. Hybrid Property (full_name)")
    print("-" * 40)
    user = User(first_name="Alice", last_name="Smith", age=30)
    print(f"Instance access: user.full_name = {user.full_name}")

    # Using setter
    print(f"Before setter: first_name={user.first_name}, last_name={user.last_name}")
    user.full_name = "Bob Jones"
    print(f"After setter: first_name={user.first_name}, last_name={user.last_name}")
    print(f"Full name now: {user.full_name}")
    print()

    # Example 2: hybrid_method
    print("2. Hybrid Method (is_older_than)")
    print("-" * 40)
    user = User(first_name="Charlie", last_name="Brown", age=25)
    print(f"user.is_older_than(20) = {user.is_older_than(20)}")
    print(f"user.is_older_than(30) = {user.is_older_than(30)}")

    # Class-level access generates SQL expression
    sql_expr = User.is_older_than(25)
    print(f"\nClass-level access generates SQL expression:")
    print(f"  User.is_older_than(25) = {sql_expr}")
    print(f"  SQL: {sql_expr.column} {sql_expr.op} {sql_expr.value}")
    print()

    # Example 3: hybrid_property (boolean)
    print("3. Hybrid Property (is_adult)")
    print("-" * 40)
    adult = User(first_name="Dave", last_name="Wilson", age=21)
    minor = User(first_name="Eve", last_name="Taylor", age=16)
    print(f"Adult (age {adult.age}): is_adult = {adult.is_adult}")
    print(f"Minor (age {minor.age}): is_adult = {minor.is_adult}")
    print()

    # Example 4: Computed columns
    print("4. Computed Columns (PostgreSQL GENERATED AS)")
    print("-" * 40)
    product = Product(name="Widget", price=10.0, quantity=5, tax_rate=0.1)
    print(f"Product: {product.name}")
    print(f"Price: ${product.price}, Quantity: {product.quantity}")
    print(f"Computed columns (would be set by database):")
    print(f"  - total_value (stored): price * quantity")
    print(f"  - tax_amount (virtual): price * tax_rate")

    # Simulate database setting the computed values
    product._data["total_value"] = product.price * product.quantity
    product._data["tax_amount"] = product.price * product.tax_rate
    print(f"\nIf loaded from DB, computed values would be:")
    print(f"  - total_value: ${product.total_value}")
    print(f"  - tax_amount: ${product.tax_amount}")
    print()

    # Example 5: column_property
    print("5. Column Property (SQL computed)")
    print("-" * 40)
    order = Order(amount=100.0, tax_rate=0.2, discount_rate=0.1)
    print(f"Order: amount=${order.amount}, tax={order.tax_rate}, discount={order.discount_rate}")
    print(f"Computed properties (read-only):")
    print(f"  - subtotal: {Order.subtotal.expression}")
    print(f"  - tax: {Order.tax.expression}")
    print(f"  - total: {Order.total.expression}")

    # Simulate database computing the values
    order._data["subtotal"] = order.amount * (1 - order.discount_rate)
    order._data["tax"] = order.amount * (1 - order.discount_rate) * order.tax_rate
    order._data["total"] = order.amount * (1 - order.discount_rate) * (1 + order.tax_rate)
    print(f"\nIf loaded from DB, computed values would be:")
    print(f"  - subtotal: ${order.subtotal:.2f}")
    print(f"  - tax: ${order.tax:.2f}")
    print(f"  - total: ${order.total:.2f}")

    # Try to set a read-only property (should fail)
    print("\nTrying to set read-only property:")
    try:
        order.subtotal = 999.0
        print("  ERROR: Should have raised AttributeError!")
    except AttributeError as e:
        print(f"  ✓ Correctly raised: {e}")
    print()

    # Example 6: default_factory
    print("6. Default Factory (dynamic defaults)")
    print("-" * 40)
    import time
    user1 = User(first_name="Frank", last_name="Garcia", age=35)
    time.sleep(0.01)  # Small delay to ensure different timestamps
    user2 = User(first_name="Grace", last_name="Martinez", age=28)
    print(f"User1 created_at: {user1.created_at}")
    print(f"User2 created_at: {user2.created_at}")
    print(f"Both timestamps are different (dynamic generation): {user1.created_at != user2.created_at}")
    print()

    # Example 7: Computed column SQL DDL
    print("7. Computed Column SQL DDL")
    print("-" * 40)
    print("For PostgreSQL schema creation:")
    print(f"  total_value: {Product.total_value.to_sql('DECIMAL(10,2)')}")
    print(f"  tax_amount: {Product.tax_amount.to_sql('DECIMAL(10,2)')}")
    print()

    # Example 8: Read-only enforcement
    print("8. Read-Only Enforcement")
    print("-" * 40)
    print("Trying to set computed column (should fail):")
    try:
        product.total_value = 999.0
        print("  ERROR: Should have raised AttributeError!")
    except AttributeError as e:
        print(f"  ✓ Correctly raised: {e}")

    print("\nTrying to set hybrid property without setter (should fail):")
    try:
        user.is_adult = True
        print("  ERROR: Should have raised AttributeError!")
    except AttributeError as e:
        print(f"  ✓ Correctly raised: {e}")
    print()

    print("=== All Examples Complete ===")


if __name__ == "__main__":
    main()
