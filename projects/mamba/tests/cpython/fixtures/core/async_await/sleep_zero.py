# asyncio.sleep(0) yields control but completes immediately
import asyncio

async def tick(n):
    await asyncio.sleep(0)
    return n + 1

async def main():
    a = await tick(10)
    b = await tick(a)
    return b

print(asyncio.run(main()))
