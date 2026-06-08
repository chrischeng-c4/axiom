# Guard: common async/await shapes that work. Str-concat return inside
# await-chain still SIGSEGVs (tracked but not yet fixed) — avoid it here.

import asyncio

async def plain():
    return 42

async def with_param(x):
    return x * 2

async def chained_int():
    a = await plain()
    b = await with_param(a)
    return a + b

async def main():
    print(await plain())
    print(await with_param(10))
    print(await chained_int())

asyncio.run(main())
