"""
Shared models for benchmarks.

Comparing: data-bridge (Rust async) vs Beanie (Motor async)
"""

from cclab import Document
from beanie import Document as BeanieDoc


# data-bridge models
class DBUser(Document):
    name: str
    email: str
    age: int

    class Settings:
        name = "bench_db"


# Beanie models
class BeanieUser(BeanieDoc):
    name: str
    email: str
    age: int

    class Settings:
        name = "bench_beanie"


# List of all Beanie models for init_beanie()
BEANIE_MODELS = [BeanieUser]
