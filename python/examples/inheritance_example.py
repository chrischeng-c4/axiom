"""Example demonstrating SQLAlchemy-style table inheritance patterns.

This file shows how to use the three inheritance strategies:
1. Single Table Inheritance (SINGLE_TABLE)
2. Joined Table Inheritance (JOINED)
3. Concrete Table Inheritance (CONCRETE)
"""

from cclab.titan import (
    Table,
    Column,
    InheritanceType,
    inheritance,
    SingleTableInheritance,
    JoinedTableInheritance,
    ConcreteTableInheritance,
    PolymorphicQueryMixin,
    get_inheritance_type,
    get_discriminator_column,
    get_discriminator_value,
    get_polymorphic_map,
)


# =============================================================================
# Example 1: Single Table Inheritance
# =============================================================================
# All employee types share one table with a 'type' column to distinguish them


@inheritance(type=InheritanceType.SINGLE_TABLE, discriminator="employee_type")
class Employee(Table, SingleTableInheritance, PolymorphicQueryMixin):
    """Base employee class using single table inheritance."""

    name: str
    email: str
    employee_type: str  # Discriminator column

    class Settings:
        table_name = "employees"


class Manager(Employee):
    """Manager subclass - stored in employees table with employee_type='manager'."""

    __discriminator_value__ = "manager"
    department: str
    budget: int


class Engineer(Employee):
    """Engineer subclass - stored in employees table with employee_type='engineer'."""

    __discriminator_value__ = "engineer"
    programming_language: str
    seniority_level: str


class Contractor(Employee):
    """Contractor subclass - stored in employees table with employee_type='contractor'."""

    __discriminator_value__ = "contractor"
    contract_end_date: str
    hourly_rate: float


# =============================================================================
# Example 2: Joined Table Inheritance
# =============================================================================
# Each class has its own table, joined via foreign key


@inheritance(type=InheritanceType.JOINED)
class Vehicle(Table, JoinedTableInheritance):
    """Base vehicle class using joined table inheritance."""

    make: str
    model: str
    year: int

    class Settings:
        table_name = "vehicles"


class Car(Vehicle):
    """Car subclass - has its own 'cars' table with FK to vehicles."""

    num_doors: int
    trunk_size: float

    class Settings:
        table_name = "cars"


class Truck(Vehicle):
    """Truck subclass - has its own 'trucks' table with FK to vehicles."""

    bed_length: float
    towing_capacity: int

    class Settings:
        table_name = "trucks"


class Motorcycle(Vehicle):
    """Motorcycle subclass - has its own 'motorcycles' table with FK to vehicles."""

    engine_cc: int
    has_sidecar: bool

    class Settings:
        table_name = "motorcycles"


# =============================================================================
# Example 3: Concrete Table Inheritance
# =============================================================================
# Each class has a complete standalone table


@inheritance(type=InheritanceType.CONCRETE)
class Animal(Table, ConcreteTableInheritance):
    """Base animal class using concrete table inheritance."""

    name: str
    age: int

    class Settings:
        table_name = "animals"


class Dog(Animal):
    """Dog subclass - complete standalone 'dogs' table."""

    name: str  # Duplicated from parent
    age: int  # Duplicated from parent
    breed: str
    is_good_boy: bool = True

    class Settings:
        table_name = "dogs"


class Cat(Animal):
    """Cat subclass - complete standalone 'cats' table."""

    name: str  # Duplicated from parent
    age: int  # Duplicated from parent
    fur_color: str
    indoor_only: bool

    class Settings:
        table_name = "cats"


class Bird(Animal):
    """Bird subclass - complete standalone 'birds' table."""

    name: str  # Duplicated from parent
    age: int  # Duplicated from parent
    species: str
    can_fly: bool

    class Settings:
        table_name = "birds"


# =============================================================================
# Demonstration of Helper Functions
# =============================================================================


def demonstrate_inheritance_helpers():
    """Show how to use inheritance helper functions."""
    print("=" * 70)
    print("INHERITANCE PATTERN INSPECTION")
    print("=" * 70)

    # Check inheritance types
    print("\n1. Inheritance Types:")
    print(f"   Employee: {get_inheritance_type(Employee)}")
    print(f"   Manager: {get_inheritance_type(Manager)}")
    print(f"   Vehicle: {get_inheritance_type(Vehicle)}")
    print(f"   Animal: {get_inheritance_type(Animal)}")

    # Check discriminator columns
    print("\n2. Discriminator Columns:")
    print(f"   Employee: {get_discriminator_column(Employee)}")
    print(f"   Manager: {get_discriminator_column(Manager)}")

    # Check discriminator values
    print("\n3. Discriminator Values:")
    print(f"   Employee: {get_discriminator_value(Employee)}")
    print(f"   Manager: {get_discriminator_value(Manager)}")
    print(f"   Engineer: {get_discriminator_value(Engineer)}")
    print(f"   Contractor: {get_discriminator_value(Contractor)}")

    # Check polymorphic map
    print("\n4. Polymorphic Class Map:")
    poly_map = get_polymorphic_map(Employee)
    for disc_value, cls in poly_map.items():
        print(f"   '{disc_value}' -> {cls.__name__}")

    # Check subclasses
    print("\n5. Registered Subclasses:")
    if hasattr(Employee, "get_subclasses"):
        subclasses = Employee.get_subclasses()
        print(f"   Employee subclasses: {[cls.__name__ for cls in subclasses]}")

    # Check discriminator filters
    print("\n6. Discriminator Filters:")
    if hasattr(Manager, "_get_discriminator_filter"):
        filter_info = Manager._get_discriminator_filter()
        print(f"   Manager filter: {filter_info}")

    # Check JOIN config
    print("\n7. JOIN Configuration:")
    if hasattr(Car, "_get_join_config"):
        join_config = Car._get_join_config()
        if join_config:
            print(f"   Car JOIN config: {join_config}")

    print("\n" + "=" * 70)


# =============================================================================
# Usage Examples (would require database connection)
# =============================================================================


async def usage_examples():
    """
    Example usage patterns for inheritance.

    Note: These examples require a PostgreSQL database connection.
    """

    # Single Table Inheritance Usage
    # -------------------------------
    # Create instances
    manager = Manager(
        name="Alice Smith",
        email="alice@example.com",
        employee_type="manager",
        department="Engineering",
        budget=500000,
    )
    # await manager.save()

    engineer = Engineer(
        name="Bob Jones",
        email="bob@example.com",
        employee_type="engineer",
        programming_language="Python",
        seniority_level="Senior",
    )
    # await engineer.save()

    # Query - automatically filters by discriminator
    # managers = await Manager.find().to_list()
    # This generates: SELECT * FROM employees WHERE employee_type = 'manager'

    # Polymorphic query - returns correct subclass instances
    # all_employees = await Employee.fetch_polymorphic()
    # for emp in all_employees:
    #     if isinstance(emp, Manager):
    #         print(f"Manager: {emp.name}, Dept: {emp.department}")
    #     elif isinstance(emp, Engineer):
    #         print(f"Engineer: {emp.name}, Lang: {emp.programming_language}")

    # Joined Table Inheritance Usage
    # -------------------------------
    car = Car(
        make="Toyota",
        model="Camry",
        year=2023,
        num_doors=4,
        trunk_size=15.1,
    )
    # await car.save()
    # This creates records in both 'vehicles' and 'cars' tables

    # Query - automatically JOINs parent table
    # cars = await Car.find().to_list()
    # This generates: SELECT c.*, v.* FROM cars c JOIN vehicles v ON c.id = v.id

    # Concrete Table Inheritance Usage
    # ---------------------------------
    dog = Dog(
        name="Buddy",
        age=5,
        breed="Golden Retriever",
        is_good_boy=True,
    )
    # await dog.save()
    # This only inserts into 'dogs' table (no parent table)

    # Query - only queries the specific table
    # dogs = await Dog.find().to_list()
    # This generates: SELECT * FROM dogs

    # Polymorphic query - uses UNION
    # all_animals = await Animal.find_polymorphic().to_list()
    # This generates:
    #   SELECT *, 'dog' as _type FROM dogs
    #   UNION ALL
    #   SELECT *, 'cat' as _type FROM cats
    #   UNION ALL
    #   SELECT *, 'bird' as _type FROM birds


if __name__ == "__main__":
    # Run helper function demonstration
    demonstrate_inheritance_helpers()

    # Async examples would need to be run with asyncio.run()
    # import asyncio
    # asyncio.run(usage_examples())
