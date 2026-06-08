"""Find Many benchmark."""

from cclab.probe import BenchmarkGroup, register_group
from tests.mongo.benchmarks.models import DBUser, BeanieUser

group = BenchmarkGroup("Find Many (100)")


@group.add("Beanie")
async def beanie_find_many():
    return await BeanieUser.find({"age": {"$gte": 25}}).limit(100).to_list()


@group.add("data-bridge")
async def db_find_many():
    return await DBUser.find(DBUser.age >= 25).limit(100).to_list()


register_group(group)
