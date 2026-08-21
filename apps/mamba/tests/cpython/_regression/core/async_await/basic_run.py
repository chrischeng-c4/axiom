# asyncio.run with basic coroutine returning a value
import asyncio

async def hello():
    return 42

print(asyncio.run(hello()))


async def with_arg(x):
    return x * 2

print(asyncio.run(with_arg(21)))
