"""Delete benchmark (includes insert since delete needs data)."""

from cclab.probe import BenchmarkGroup, register_group
from tests.mongo.benchmarks.models import DBUser, BeanieUser

DATA_1000 = [{"name": f"Del{i}", "email": f"del{i}@test.com", "age": 20 + i % 50} for i in range(1000)]

group = BenchmarkGroup("Delete Many (insert+delete)")


@group.add("Beanie")
async def beanie_delete_many():
    await BeanieUser.insert_many([BeanieUser(**d) for d in DATA_1000])
    await BeanieUser.find({"age": {"$gte": 30}}).delete()


@group.add("data-bridge")
async def db_delete_many():
    await DBUser.insert_many([DBUser(**d) for d in DATA_1000])
    await DBUser.find(DBUser.age >= 30).delete()


register_group(group)
