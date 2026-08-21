# await on a coroutine chain
import asyncio

async def double(x):
    return x * 2

async def quad(x):
    a = await double(x)
    b = await double(a)
    return b

print(asyncio.run(quad(5)))
