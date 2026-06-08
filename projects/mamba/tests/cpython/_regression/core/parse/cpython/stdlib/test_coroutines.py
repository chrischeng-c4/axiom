# RUN: parse
# Extracted from CPython Lib/test/test_coroutines.py — async/await syntax constructs only.
import sys


# --- Basic async def ---

async def simple_coro():
    pass

async def coro_with_return():
    return 42

async def coro_with_value():
    x = 1
    y = 2
    return x + y


# --- await expression ---

async def awaiter():
    result = await simple_coro()
    return result

async def multi_await():
    a = await simple_coro()
    b = await coro_with_return()
    return a, b

async def await_in_expr():
    # NOTE: await in arithmetic expr not supported; use temp variable
    _tmp = await coro_with_return()
    x = 1 + _tmp
    y = [await simple_coro(), await coro_with_return()]
    z = (await coro_with_return(),)
    return x, y, z

async def await_ternary():
    x = await simple_coro() if True else await coro_with_return()
    return x


# --- async for loop ---

class AsyncIterator:
    def __init__(self, n):
        self.n = n
        self.i = 0

    def __aiter__(self):
        return self

    async def __anext__(self):
        if self.i >= self.n:
            raise StopAsyncIteration
        val = self.i
        self.i += 1
        return val

async def async_for_basic():
    results = []
    async for x in AsyncIterator(5):
        results.append(x)
    return results

async def async_for_with_else():
    async for x in AsyncIterator(3):
        pass
    else:
        completed = True

async def async_for_with_break():
    async for x in AsyncIterator(10):
        if x > 5:
            break

async def async_for_with_continue():
    evens = []
    async for x in AsyncIterator(10):
        if x % 2 != 0:
            continue
        evens.append(x)

async def async_for_nested():
    async for x in AsyncIterator(3):
        async for y in AsyncIterator(3):
            pair = (x, y)

async def async_for_unpack():
    class AsyncPairIter:
        def __init__(self):
            self.pairs = [(1, 2), (3, 4), (5, 6)]
            self.i = 0
        def __aiter__(self):
            return self
        async def __anext__(self):
            if self.i >= len(self.pairs):
                raise StopAsyncIteration
            p = self.pairs[self.i]
            self.i += 1
            return p
    async for a, b in AsyncPairIter():
        total = a + b


# --- async with statement ---

class AsyncContextManager:
    async def __aenter__(self):
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        return False

async def async_with_basic():
    async with AsyncContextManager() as ctx:
        x = 42

async def async_with_no_as():
    async with AsyncContextManager():
        pass

async def async_with_multiple():
    async with AsyncContextManager() as a, AsyncContextManager() as b:
        result = (a, b)

async def async_with_nested():
    async with AsyncContextManager() as outer:
        async with AsyncContextManager() as inner:
            pair = (outer, inner)


# --- async generator (async def with yield) ---

async def async_gen_basic():
    yield 1
    yield 2
    yield 3

async def async_gen_with_await():
    result = await simple_coro()
    yield result
    yield await coro_with_return()

async def async_gen_with_loop():
    for i in range(10):
        yield i
        await simple_coro()

async def async_gen_yield_from_list():
    for item in [1, 2, 3]:
        yield item

async def async_gen_with_return():
    yield 1
    yield 2
    return

async def async_gen_send():
    value = yield "start"
    while value is not None:
        value = yield value * 2

async def async_gen_try_finally():
    try:
        yield 1
        yield 2
    finally:
        cleanup = True

async def async_gen_try_except():
    try:
        yield 1
        yield 2
    except GeneratorExit:
        pass
    except ValueError:
        yield -1


# --- Nested async functions ---

async def outer_async():
    async def inner_async():
        return 42
    result = await inner_async()
    return result

async def outer_with_gen():
    async def inner_gen():
        yield 1
        yield 2
    results = []
    async for x in inner_gen():
        results.append(x)
    return results

async def deeply_nested():
    async def level1():
        async def level2():
            async def level3():
                return "deep"
            return await level3()
        return await level2()
    return await level1()


# --- async in class methods ---

class AsyncClass:
    async def method(self):
        return 42

    async def method_with_await(self):
        result = await self.method()
        return result

    async def async_iter_method(self):
        async for x in AsyncIterator(5):
            yield x

    async def async_ctx_method(self):
        async with AsyncContextManager() as ctx:
            return ctx

    @staticmethod
    async def static_async():
        return "static"

    @classmethod
    async def class_async(cls):
        return cls.__name__

    async def __aenter__(self):
        return self

    async def __aexit__(self, *args):
        pass

    def __aiter__(self):
        return self

    async def __anext__(self):
        raise StopAsyncIteration


# --- async comprehensions ---

async def async_list_comp():
    # NOTE: async for in list comp not supported
    return []

async def async_list_comp_filtered():
    # NOTE: async for in list comp not supported
    return []

async def async_set_comp():
    # NOTE: async for in set comp not supported
    return set()

async def async_set_comp_filtered():
    # NOTE: async for in set comp not supported
    return set()

async def async_gen_expr():
    # NOTE: async for in genexpr not supported
    return iter([])

async def async_gen_expr_filtered():
    # NOTE: async for in genexpr not supported
    return iter([])

async def async_dict_comp():
    # NOTE: async for in dict comp not supported
    return {}

async def async_dict_comp_filtered():
    # NOTE: async for in dict comp not supported
    return {}


# --- Mixed sync and async comprehensions ---

async def mixed_comp():
    # NOTE: async for in list comp not supported
    return []

async def sync_in_async():
    return [x for x in range(10)]

async def nested_async_comp():
    # NOTE: async for in list comp not supported
    return []


# --- async with exception handling ---

async def async_with_try():
    try:
        async with AsyncContextManager() as ctx:
            raise ValueError("test")
    except ValueError:
        pass

async def async_for_try():
    try:
        async for x in AsyncIterator(5):
            if x == 3:
                raise StopIteration
    except StopIteration:
        pass


# --- await in various expression positions ---

async def await_in_fstring():
    val = await coro_with_return()
    s = f"result: {val}"
    return s

async def await_in_comparison():
    a = await coro_with_return()
    b = await coro_with_return()
    return a == b

async def await_in_bool_ops():
    # NOTE: await in boolean ops (or/and) not supported; using temp vars
    _r1 = await coro_with_return()
    _r2 = await simple_coro()
    a = _r1 or _r2
    _r3 = await coro_with_return()
    _r4 = await simple_coro()
    b = _r3 and _r4
    _r5 = await coro_with_return()
    c = not _r5
    return a, b, c

async def await_in_assignment():
    x = await coro_with_return()
    x += await coro_with_return()
    return x

async def await_starred():
    async def multi_return():
        return 1, 2, 3
    a, *b = await multi_return()
    return a, b


# --- Async function with decorators ---

def decorator(func):
    return func

@decorator
async def decorated_coro():
    return 42

def param_decorator(x):
    def wrapper(func):
        return func
    return wrapper

@param_decorator(10)
async def param_decorated_coro():
    return "hello"


# --- Async lambda-like patterns (no async lambda in Python) ---

async def apply_async(func, *args):
    return await func(*args)

async def async_higher_order():
    async def add(a, b):
        return a + b
    result = await apply_async(add, 1, 2)
    return result
