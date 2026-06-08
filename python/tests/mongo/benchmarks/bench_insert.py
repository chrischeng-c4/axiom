"""Insert benchmarks."""

from cclab.probe import BenchmarkGroup, register_group
from tests.mongo.benchmarks.models import DBUser, BeanieUser

# Insert One
insert_one = BenchmarkGroup("Insert One")


@insert_one.add("Beanie")
async def beanie_insert_one():
    await BeanieUser(name="Test", email="test@test.com", age=30).insert()


@insert_one.add("data-bridge")
async def db_insert_one():
    await DBUser(name="Test", email="test@test.com", age=30).save()


register_group(insert_one)


# Bulk Insert
DATA_1000 = [{"name": f"User{i}", "email": f"u{i}@test.com", "age": 20 + i % 50} for i in range(1000)]

bulk_insert = BenchmarkGroup("Bulk Insert (1000)")


@bulk_insert.add("Beanie")
async def beanie_bulk_insert():
    await BeanieUser.insert_many([BeanieUser(**d) for d in DATA_1000])


@bulk_insert.add("data-bridge")
async def db_bulk_insert():
    await DBUser.insert_many([DBUser(**d) for d in DATA_1000])


register_group(bulk_insert)
