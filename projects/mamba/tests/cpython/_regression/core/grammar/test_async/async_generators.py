# RUN: parse
# CPython 3.12 test_async: async generators (PEP 525)

# Basic async generator
async def async_count(n):
    for i in range(n):
        yield i

# Async generator with await
async def fetch_items(urls):
    for url in urls:
        yield url

