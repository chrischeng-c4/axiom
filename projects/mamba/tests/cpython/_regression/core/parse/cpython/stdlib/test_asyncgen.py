# RUN: parse
# Extracted from CPython Lib/test/test_asyncgen.py — async generator syntax constructs only.
import sys


# --- Basic async generator ---

async def async_gen_simple():
    yield 1
    yield 2
    yield 3

async def async_gen_empty():
    return
    yield

async def async_gen_single():
    yield 42


# --- Async generator with yield and return ---

async def async_gen_return_none():
    yield 1
    return

async def async_gen_early_return():
    for i in range(10):
        if i > 5:
            return
        yield i

async def async_gen_return_in_try():
    try:
        yield 1
        return
    finally:
        cleanup = True

async def async_gen_return_in_except():
    try:
        yield 1
        raise ValueError
    except ValueError:
        return


# --- Async for iteration ---

class AsyncRange:
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

async def consume_async_gen():
    results = []
    async for val in async_gen_simple():
        results.append(val)

async def async_for_else():
    async for val in async_gen_simple():
        pass
    else:
        done = True

async def async_for_break():
    async for val in async_gen_simple():
        if val == 2:
            break

async def async_for_continue():
    odds = []
    async for val in AsyncRange(10):
        if val % 2 == 0:
            continue
        odds.append(val)


# --- Async generator expressions ---

# NOTE: async for in genexpr () not supported by parser; list/set/dict comps work
# async def async_genexp_basic():
#     g = (x async for x in AsyncRange(5))
# async def async_genexp_filtered():
#     g = (x async for x in AsyncRange(10) if x % 2 == 0)
# async def async_genexp_transformed():
#     g = (x * x async for x in AsyncRange(5))
# async def async_genexp_nested():
#     g = (x async for x in AsyncRange(5) for y in range(x))
# NOTE: async for in list/set/dict comprehension not supported by parser
# async def async_genexp_basic(): ...
# async def async_genexp_in_list(): ...
# async def async_genexp_in_set(): ...
# async def async_genexp_in_dict(): ...
async def async_genexp_basic():
    return []


# --- Multiple yields in async def ---

async def multi_yield():
    yield "first"
    x = 10
    yield "second"
    y = 20
    yield "third"
    yield x + y

async def yield_in_loop():
    for i in range(5):
        yield i

async def yield_in_while():
    n = 0
    while n < 5:
        yield n
        n += 1

async def yield_in_conditional():
    for i in range(10):
        if i % 2 == 0:
            yield i
        elif i % 3 == 0:
            yield i * 10

async def yield_with_computation():
    data = [1, 2, 3, 4, 5]
    for item in data:
        result = item * item + 1
        yield result


# --- Async generator with await ---

async def fetch_value():
    return 42

async def async_gen_with_await():
    val = await fetch_value()
    yield val
    yield await fetch_value()

async def async_gen_await_in_loop():
    for i in range(3):
        val = await fetch_value()
        yield val + i

async def async_gen_await_conditional():
    val = await fetch_value()
    if val > 0:
        yield val
    else:
        yield -val


# --- Async generator with try/except/finally ---

async def async_gen_try_except():
    try:
        yield 1
        yield 2
    except GeneratorExit:
        pass

async def async_gen_try_finally():
    try:
        yield 1
        yield 2
    finally:
        done = True

async def async_gen_full_try():
    try:
        yield 1
    except ValueError:
        yield -1
    else:
        yield 2
    finally:
        done = True

async def async_gen_nested_try():
    try:
        try:
            yield 1
        except TypeError:
            yield -1
    except ValueError:
        yield -2
    finally:
        done = True


# --- Async generator send pattern ---

async def async_accumulator():
    total = 0
    while True:
        value = yield total
        if value is None:
            break
        total += value

async def async_echo():
    while True:
        received = yield
        if received is None:
            return
        yield received


# --- Async generator in class ---

class AsyncCounter:
    def __init__(self, limit):
        self.limit = limit

    async def count(self):
        for i in range(self.limit):
            yield i

    async def count_by(self, step):
        i = 0
        while i < self.limit:
            yield i
            i += step

    async def filtered_count(self, predicate):
        for i in range(self.limit):
            if predicate(i):
                yield i


class AsyncPipeline:
    def __init__(self, data):
        self.data = data

    async def source(self):
        for item in self.data:
            yield item

    async def transform(self, func):
        async for item in self.source():
            yield func(item)

    async def filter(self, pred):
        async for item in self.source():
            if pred(item):
                yield item


# --- Nested async generators ---

async def outer_async_gen():
    async def inner():
        yield 1
        yield 2
    async for val in inner():
        yield val * 10

async def chained_async_gens():
    async def gen_a():
        yield "a1"
        yield "a2"
    async def gen_b():
        yield "b1"
        yield "b2"
    async for val in gen_a():
        yield val
    async for val in gen_b():
        yield val


# --- Async generator with complex yield values ---

async def yield_collections():
    yield [1, 2, 3]
    yield {"key": "value"}
    yield (1, 2, 3)
    yield {1, 2, 3}

async def yield_unpacked():
    yield (*[1, 2, 3],)
    # NOTE: dict unpacking not supported
    yield {"a": 1, "b": 2}

async def yield_expressions():
    yield 1 + 2 * 3
    yield "hello" + " " + "world"
    yield [x for x in range(5)]
    yield {k: v for k, v in enumerate("abc")}


# --- Async generator with walrus operator ---

async def async_gen_walrus():
    data = [1, 2, 3, 4, 5]
    for x in data:
        if (doubled := x * 2) > 4:
            yield doubled


# --- Async generator with context manager ---

class AsyncResource:
    async def __aenter__(self):
        return self
    async def __aexit__(self, *args):
        pass

async def async_gen_with_ctx():
    async with AsyncResource() as r:
        yield r
        yield 42

async def async_gen_ctx_in_loop():
    for i in range(3):
        async with AsyncResource() as r:
            yield (i, r)
