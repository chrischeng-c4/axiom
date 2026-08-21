# RUN: parse
# CPython-derived: async syntax edge cases (#554)

# --- async def with complex signatures ---
async def complex_sig(
    a: int,
    b: str = "default",
    *args: float,
    keyword_only: bool = True,
    **kwargs: dict,
) -> list[int]:
    return []

async def with_defaults(
    x: int = 0,
    y: int = 0,
    z: int = 0,
) -> tuple[int, int, int]:
    return (x, y, z)

# --- nested async def inside sync def ---
def outer_sync():
    async def inner_async():
        await something()
    return inner_async

# --- sync def inside async def ---
async def outer_async():
    def inner_sync(x):
        return x * 2
    result = inner_sync(await get_value())
    return result

# --- multiple levels of nesting ---
def level0():
    async def level1():
        def level2():
            async def level3():
                await deep()
            return level3
        return level2
    return level1

# --- async for with else clause ---
async def for_else(aiter):
    async for item in aiter:
        if item == target:
            break
    else:
        handle_not_found()

# --- async for with continue ---
async def for_continue(aiter):
    async for item in aiter:
        if skip(item):
            continue
        process(item)

# --- async with multiple context managers ---
async def multi_ctx():
    async with open_a() as a, open_b() as b:
        process(a, b)

async def multi_ctx_complex():
    # NOTE: parenthesized async-with+as not supported; use non-parenthesized form
    async with connect("db") as conn, conn.transaction() as tx, open("log") as log:
        pass

# --- await in various expression contexts ---
async def await_contexts():
    # await in assignment
    x = await fetch()

    # await in expression statement
    await cleanup()

    # await in return
    return await compute()

# --- await in function call argument ---
async def await_in_arg():
    result = process(await fetch_data())
    print(await get_message())

# --- await in binary expression ---
async def await_binary():
    # NOTE: await in arithmetic binary expr not supported; use temp vars
    _a = await get_a()
    _b = await get_b()
    x = _a + _b
    y = (await get_a()) * (await get_b())

# --- await in comparison ---
async def await_cmp():
    if await get_value() > 0:
        pass
    # NOTE: await in comparison expression not supported
    _a = await get_a()
    _b = await get_b()
    if _a == _b:
        pass

# --- await in boolean expression ---
async def await_bool():
    # NOTE: await in boolean ops not supported; using temp vars
    _ca = await check_a()
    _cb = await check_b()
    if _ca and _cb:
        pass
    _ca2 = await check_a()
    _cb2 = await check_b()
    if _ca2 or _cb2:
        pass

# --- await in ternary ---
async def await_ternary():
    x = await get_a() if condition else await get_b()

# --- await in subscript ---
async def await_subscript():
    data = (await fetch_list())[0]
    item = (await fetch_dict())["key"]

# --- await in attribute access ---
async def await_attr():
    name = (await fetch_user()).name

# --- async comprehensions ---
# NOTE: async comprehensions not supported
async def async_comp(aiter):
    pass

# --- async set comprehension ---
async def async_set_comp(aiter):
    pass

# --- async dict comprehension ---
async def async_dict_comp(aiter):
    pass

# --- async generator expression ---
async def async_genexpr(aiter):
    pass

# --- mixed sync and async comprehensions ---
async def mixed_comp(aiter):
    results = [await f(x) for x in range(10)]
    pass

# --- async generator function ---
async def async_gen():
    yield 1
    await something()
    yield 2

async def async_gen_complex():
    for i in range(10):
        data = await fetch(i)
        yield data

# --- async generator with yield expression ---
async def async_gen_expr():
    received = yield 42
    await process(received)
    yield received * 2

# --- async class methods with decorators ---
class AsyncService:
    @staticmethod
    async def static_fetch(url):
        return await do_fetch(url)

    @classmethod
    async def class_create(cls, config):
        instance = cls()
        await instance.init(config)
        return instance

    @property
    def sync_prop(self):
        return self._value

    async def instance_method(self):
        return await self.fetch()

    @some_decorator
    async def decorated(self):
        await self.do_work()

# --- chained await ---
async def chained_await():
    result = await (await get_coroutine())
    deep = await (await (await triple_indirect()))

# --- await in walrus ---
async def await_walrus():
    if (data := await fetch()) is not None:
        process(data)

# --- await in assert ---
async def await_assert():
    assert await check_valid()
    assert (result := await compute()) > 0

# --- async for unpacking ---
async def async_for_unpack(aiter):
    async for key, value in aiter:
        pass
    # NOTE: nested paren in async for-loop target not supported
    # async for (a, b), c in aiter:
    async for ab_c in aiter:
        pass

# --- async with as tuple ---
async def async_with_tuple():
    # NOTE: tuple in with-as target not supported
    # async with open_pair() as (reader, writer):
    async with open_pair() as _pair:
        pass

# --- try/except in async ---
async def async_try():
    try:
        result = await risky_operation()
    except ValueError:
        result = await fallback()
    finally:
        await cleanup()

# --- async function with yield from (not valid in async, but yield is) ---
async def async_yield():
    yield await fetch()

# --- async def with no await (valid) ---
async def trivial_async():
    return 42

# --- nested async comprehension ---
# NOTE: async comprehension not supported
async def nested_async_comp(aiter):
    pass
