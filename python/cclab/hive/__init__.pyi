from typing import Optional

class HiveConnection:
    async def execute(self, query: str) -> ResultBatch: ...
    def idle_count(self) -> int: ...
    def active_count(self) -> int: ...

class PrestoConnection:
    async def execute(self, query: str) -> ResultBatch: ...

class ResultBatch:
    @property
    def columns(self) -> list[str]: ...
    @property
    def column_types(self) -> list[str]: ...
    @property
    def has_more(self) -> bool: ...
    def __len__(self) -> int: ...

def connect(
    url: str,
    auth: str = "NOSASL",
    user: Optional[str] = None,
    password: Optional[str] = None,
    database: Optional[str] = None,
    pool_size: int = 10,
) -> HiveConnection: ...

def presto(
    url: str,
    catalog: str = "hive",
    schema: str = "default",
    user: Optional[str] = None,
    password: Optional[str] = None,
) -> PrestoConnection: ...
