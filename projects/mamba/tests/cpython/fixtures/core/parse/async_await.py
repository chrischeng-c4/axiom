# RUN: parse

async def fetch(url: str) -> str:
    result: str = await get(url)
    return result
