# asyncio.gather collects multiple coroutine results
import asyncio

async def square(n):
    return n * n

async def main():
    results = await asyncio.gather(square(2), square(3), square(4))
    return sum(results)

print(asyncio.run(main()))
