# RUN: parse
# CPython 3.12 test_async: await expressions

# Basic await
async def fetch():
    return 42

async def basic_await():
    result = await fetch()
    return result

# Await in assignment
async def multiple_awaits():
    a = await fetch()
    b = await fetch()
    return a + b

# Await in return
async def return_await():
    return await fetch()

# Await in expression
async def await_in_expr():
    x = (await fetch()) + 1
    return x

# Await in conditional
async def await_conditional():
    if await fetch():
        return True
    return False

# Chained awaits
async def helper():
    return await fetch()

async def chained():
    result = await helper()
    return result
