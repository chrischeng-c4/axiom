# RUN: parse
# CPython 3.12 test_async: async for loops

# Async iterator protocol
class AsyncRange:
    def __init__(self, n):
        self.n = n
        self.i = 0

    def __aiter__(self):
        return self

    async def __anext__(self):
        if self.i >= self.n:
            raise StopAsyncIteration
        self.i += 1
        return self.i - 1

# Async for loop
async def consume():
    async for item in AsyncRange(10):
        pass

# Async for with else
async def with_else():
    async for item in AsyncRange(5):
        pass
    else:
        pass

