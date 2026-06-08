"""
Tests for document inheritance support.

Document inheritance allows:
- Storing multiple document types in a single collection
- Polymorphic loading (loading correct subclass based on _class_id)
- Filtering by class type using with_children()

Migrated from pytest to cclab.probe framework.
"""
from cclab import Document
from cclab.probe import test, expect
from tests.base import MongoTestSuite, CommonTestSuite


# =====================
# Module-level test classes for CRUD tests
# =====================

class Vehicle(Document):
    """Root class for vehicle hierarchy."""
    name: str
    wheels: int = 0

    class Settings:
        name = "test_vehicles"
        is_root = True


class Car(Vehicle):
    """Car subclass."""
    doors: int = 4


class Motorcycle(Vehicle):
    """Motorcycle subclass."""
    has_sidecar: bool = False


class TestInheritanceSetup(CommonTestSuite):
    """Tests for setting up inheritance hierarchy."""

    @test(tags=["unit", "inheritance"])
    async def test_is_root_flag(self):
        """Test that is_root=True is recognized in Settings."""
        class Ship(Document):
            name: str

            class Settings:
                name = "test_ships_root"
                is_root = True

        expect(Ship._is_root).to_be_true()
        expect(Ship._class_id).to_equal("Ship")
        expect("Ship" in Ship._child_classes).to_be_true()

    @test(tags=["unit", "inheritance"])
    async def test_child_class_registration(self):
        """Test that child classes are registered with parent."""
        class Animal(Document):
            name: str

            class Settings:
                name = "test_animals"
                is_root = True

        class Dog(Animal):
            breed: str = ""

        class Cat(Animal):
            indoor: bool = True

        expect("Animal" in Animal._child_classes).to_be_true()
        expect("Dog" in Animal._child_classes).to_be_true()
        expect("Cat" in Animal._child_classes).to_be_true()

        expect(Dog._class_id).to_equal("Dog")
        expect(Cat._class_id).to_equal("Cat")

        expect(Dog._collection_name).to_equal("test_animals")
        expect(Cat._collection_name).to_equal("test_animals")

    @test(tags=["unit", "inheritance"])
    async def test_grandchild_inheritance(self):
        """Test multi-level inheritance."""
        class Entity(Document):
            name: str

            class Settings:
                name = "test_entities"
                is_root = True

        class LivingThing(Entity):
            alive: bool = True

        class Person(LivingThing):
            age: int = 0

        expect(Entity._collection_name).to_equal("test_entities")
        expect(LivingThing._collection_name).to_equal("test_entities")
        expect(Person._collection_name).to_equal("test_entities")

        expect("Entity" in Entity._child_classes).to_be_true()
        expect("LivingThing" in Entity._child_classes).to_be_true()
        expect("Person" in Entity._child_classes).to_be_true()

        expect(LivingThing._class_id).to_equal("LivingThing")
        expect(Person._class_id).to_equal("Person")

    @test(tags=["unit", "inheritance"])
    async def test_non_inheritance_document(self):
        """Test that regular documents work normally."""
        class SimpleDoc(Document):
            value: int

            class Settings:
                name = "test_simple_docs"

        expect(SimpleDoc._is_root).to_be_false()
        expect(SimpleDoc._class_id).to_be_none()
        expect(SimpleDoc._root_class).to_be_none()


class TestInheritanceCRUD(MongoTestSuite):
    """Tests for CRUD operations with inheritance."""

    async def setup(self):
        """Clean up test collection."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_vehicles", {})

    async def teardown(self):
        """Clean up test collection."""
        from cclab.nebula import _engine
        await _engine.delete_many("test_vehicles", {})

    @test(tags=["mongo", "inheritance", "crud"])
    async def test_save_with_class_id(self):
        """Test that saving includes _class_id."""
        car = Car(name="Tesla Model S", wheels=4, doors=4)
        await car.save()

        from cclab.nebula import _engine
        doc = await _engine.find_one("test_vehicles", {"_id": car._id})
        expect(doc["_class_id"]).to_equal("Car")
        expect(doc["name"]).to_equal("Tesla Model S")

    @test(tags=["mongo", "inheritance", "crud"])
    async def test_polymorphic_loading(self):
        """Test that find returns correct subclass instances."""
        car = Car(name="Honda Civic", wheels=4, doors=4)
        await car.save()

        motorcycle = Motorcycle(name="Harley Davidson", wheels=2, has_sidecar=True)
        await motorcycle.save()

        vehicle = Vehicle(name="Generic Vehicle", wheels=3)
        await vehicle.save()

        all_vehicles = await Vehicle.find().to_list()
        expect(len(all_vehicles)).to_equal(3)

        types = {type(v).__name__ for v in all_vehicles}
        expect(types).to_equal({"Vehicle", "Car", "Motorcycle"})

        found_car = await Vehicle.find_one(Vehicle.name == "Honda Civic")
        expect(isinstance(found_car, Car)).to_be_true()
        expect(found_car.doors).to_equal(4)

        found_moto = await Vehicle.find_one(Vehicle.name == "Harley Davidson")
        expect(isinstance(found_moto, Motorcycle)).to_be_true()
        expect(found_moto.has_sidecar).to_be_true()

    @test(tags=["mongo", "inheritance", "crud"])
    async def test_with_children_false(self):
        """Test filtering to only exact class type."""
        await Car(name="Car 1", wheels=4, doors=2).save()
        await Car(name="Car 2", wheels=4, doors=4).save()
        await Motorcycle(name="Moto 1", wheels=2).save()
        await Vehicle(name="Vehicle 1", wheels=6).save()

        only_vehicles = await Vehicle.find().with_children(False).to_list()
        expect(len(only_vehicles)).to_equal(1)
        expect(only_vehicles[0].name).to_equal("Vehicle 1")
        expect(type(only_vehicles[0]).__name__).to_equal("Vehicle")

    @test(tags=["mongo", "inheritance", "crud"])
    async def test_with_children_true_default(self):
        """Test that with_children=True is the default."""
        await Car(name="Car A", wheels=4, doors=4).save()
        await Motorcycle(name="Moto A", wheels=2).save()
        await Vehicle(name="Vehicle A", wheels=1).save()

        all_vehicles = await Vehicle.find().to_list()
        expect(len(all_vehicles)).to_equal(3)

        all_vehicles_explicit = await Vehicle.find().with_children(True).to_list()
        expect(len(all_vehicles_explicit)).to_equal(3)

    @test(tags=["mongo", "inheritance", "crud"])
    async def test_query_specific_child_class(self):
        """Test querying directly from child class."""
        await Car(name="Car X", wheels=4, doors=2).save()
        await Car(name="Car Y", wheels=4, doors=4).save()
        await Motorcycle(name="Moto X", wheels=2).save()

        cars = await Car.find().to_list()
        expect(len(cars)).to_equal(2)
        expect(all(isinstance(c, Car) for c in cars)).to_be_true()

        motos = await Motorcycle.find().to_list()
        expect(len(motos)).to_equal(1)
        expect(isinstance(motos[0], Motorcycle)).to_be_true()

    @test(tags=["mongo", "inheritance", "crud"])
    async def test_count_with_inheritance(self):
        """Test count operations with inheritance."""
        await Car(name="C1", wheels=4).save()
        await Car(name="C2", wheels=4).save()
        await Motorcycle(name="M1", wheels=2).save()
        await Vehicle(name="V1", wheels=3).save()

        total = await Vehicle.find().count()
        expect(total).to_equal(4)

        vehicle_only = await Vehicle.find().with_children(False).count()
        expect(vehicle_only).to_equal(1)

        car_count = await Car.find().count()
        expect(car_count).to_equal(2)

    @test(tags=["mongo", "inheritance", "crud"])
    async def test_delete_with_inheritance(self):
        """Test delete operations with inheritance."""
        await Car(name="D1", wheels=4).save()
        await Car(name="D2", wheels=4).save()
        await Motorcycle(name="D3", wheels=2).save()

        deleted = await Motorcycle.find().delete()
        expect(deleted).to_equal(1)

        remaining = await Vehicle.find().to_list()
        expect(len(remaining)).to_equal(2)
        expect(all(isinstance(v, Car) for v in remaining)).to_be_true()


class TestInheritanceFields(MongoTestSuite):
    """Tests for field handling in inheritance."""

    @test(tags=["unit", "inheritance"])
    async def test_child_inherits_parent_fields(self):
        """Test that child classes inherit parent fields."""
        class BaseDoc(Document):
            base_field: str

            class Settings:
                name = "test_base_docs"
                is_root = True

        class ChildDoc(BaseDoc):
            child_field: int = 0

        expect("base_field" in ChildDoc._fields).to_be_true()
        expect("child_field" in ChildDoc._fields).to_be_true()

    @test(tags=["mongo", "inheritance"])
    async def test_save_and_load_inherited_fields(self):
        """Test saving and loading with inherited fields."""
        class BaseProduct(Document):
            name: str
            price: float = 0.0

            class Settings:
                name = "test_products_inherit"
                is_root = True

        class DigitalProduct(BaseProduct):
            download_url: str = ""
            file_size: int = 0

        from cclab.nebula import _engine
        await _engine.delete_many("test_products_inherit", {})

        product = DigitalProduct(
            name="E-Book",
            price=9.99,
            download_url="https://example.com/book.pdf",
            file_size=1024,
        )
        await product.save()

        loaded = await BaseProduct.find_one(BaseProduct.name == "E-Book")
        expect(isinstance(loaded, DigitalProduct)).to_be_true()
        expect(loaded.name).to_equal("E-Book")
        expect(loaded.price).to_equal(9.99)
        expect(loaded.download_url).to_equal("https://example.com/book.pdf")
        expect(loaded.file_size).to_equal(1024)

        await _engine.delete_many("test_products_inherit", {})


class TestInheritanceEdgeCases(CommonTestSuite):
    """Tests for edge cases in inheritance."""

    @test(tags=["unit", "inheritance", "edge-case"])
    async def test_child_without_own_settings(self):
        """Test child class without its own Settings class."""
        class Parent(Document):
            value: int

            class Settings:
                name = "test_parent_only_settings"
                is_root = True

        class Child(Parent):
            extra: str = ""

        expect(Child._collection_name).to_equal("test_parent_only_settings")
        expect(Child._class_id).to_equal("Child")

    @test(tags=["unit", "inheritance", "edge-case"])
    async def test_child_with_override_settings(self):
        """Test that child can't override collection name."""
        class Root(Document):
            x: int

            class Settings:
                name = "test_root_collection"
                is_root = True

        class Child(Root):
            y: int = 0

            class Settings:
                name = "different_collection"

        expect(Child._collection_name).to_equal("test_root_collection")

    @test(tags=["unit", "inheritance", "edge-case"])
    async def test_sibling_isolation(self):
        """Test that sibling classes don't interfere with each other."""
        class TreeNode(Document):
            label: str

            class Settings:
                name = "test_tree_nodes"
                is_root = True

        class LeafNode(TreeNode):
            data: str = ""

        class BranchNode(TreeNode):
            children_count: int = 0

        expect(LeafNode._class_id).to_equal("LeafNode")
        expect(BranchNode._class_id).to_equal("BranchNode")

        expect("LeafNode" in TreeNode._child_classes).to_be_true()
        expect("BranchNode" in TreeNode._child_classes).to_be_true()


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestInheritanceSetup,
        TestInheritanceCRUD,
        TestInheritanceFields,
        TestInheritanceEdgeCases,
    ], verbose=True)
