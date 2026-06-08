# RUN: parse
# CPython 3.12 test_async: async function definitions

# Basic async function
async def hello():
    return "hello"

# Async with parameters
async def greet(name: str) -> str:
    return f"Hello, {name}"

# Async with default args
async def fetch(url, timeout=30):
    pass

# Async with *args, **kwargs
async def flexible(*args, **kwargs):
    pass

# Await expression
async def caller():
    result = await hello()
    return result

# Multiple awaits
async def multi():
    a = await hello()
    b = await greet("world")
    return a, b

# Async method
class AsyncService:
    async def process(self):
        pass

    async def fetch_data(self, key):
        return key
