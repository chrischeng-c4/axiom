# RUN: parse
# CPython 3.12 test_async: async context managers

# Async context manager protocol
class AsyncResource:
    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        pass

# Async with statement
async def use_resource():
    async with AsyncResource() as r:
        pass

# Multiple async with
async def multi_resource():
    async with AsyncResource() as a, AsyncResource() as b:
        pass

# Nested async with
async def nested():
    async with AsyncResource() as outer:
        async with AsyncResource() as inner:
            pass
