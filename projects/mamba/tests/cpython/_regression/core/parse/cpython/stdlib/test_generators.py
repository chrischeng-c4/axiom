# RUN: parse
# Extracted from CPython Lib/test/test_generators.py — generator syntax constructs only.


# --- Simple generator function ---

def gen_basic():
    yield 1
    yield 2
    yield 3

g = gen_basic()
next(g)
next(g)


# --- yield with value ---

def gen_values():
    yield "hello"
    yield 42
    yield [1, 2, 3]
    yield {"key": "value"}
    yield (1, 2)
    yield None


# --- yield without value ---

def gen_bare_yield():
    yield
    x = 10
    yield
    y = 20
    yield


# --- yield in expressions ---

def gen_yield_expr():
    x = yield 1
    y = yield 2
    z = yield x + y if x and y else 0


def gen_yield_in_parens():
    x = (yield 42)
    y = (yield)
    result = (yield x)


# --- yield from iterable ---

def gen_yield_from_list():
    yield from [1, 2, 3]

def gen_yield_from_tuple():
    yield from (10, 20, 30)

def gen_yield_from_range():
    yield from range(5)

def gen_yield_from_string():
    yield from "abc"

def gen_yield_from_dict():
    yield from {"a": 1, "b": 2}

def gen_yield_from_set():
    yield from {1, 2, 3}


# --- yield from generator ---

def inner_gen():
    yield 1
    yield 2

def outer_gen():
    yield from inner_gen()
    yield 3

def chained_yield_from():
    yield from inner_gen()
    yield from inner_gen()


# --- yield from with return value ---

def gen_with_return():
    yield 1
    yield 2
    return "done"

def delegating_gen():
    result = yield from gen_with_return()
    yield result


# --- Generator expressions ---

squares = (x * x for x in range(10))
evens = (x for x in range(20) if x % 2 == 0)
pairs = ((x, y) for x in range(3) for y in range(3))
nested_genexp = (x for xs in [[1, 2], [3, 4]] for x in xs)

sum(x for x in range(10))
list(x * 2 for x in range(5))
max(x for x in [3, 1, 4, 1, 5])
min(x for x in [3, 1, 4, 1, 5])
any(x > 3 for x in range(5))
all(x > 0 for x in range(1, 5))

tuple(x for x in range(3))
set(x % 3 for x in range(9))
dict((x, x * x) for x in range(5))


# --- Generator with send pattern ---

def accumulator():
    total = 0
    while True:
        value = yield total
        if value is None:
            break
        total += value

def echo_gen():
    while True:
        received = yield
        if received is None:
            return
        yield received

def gen_send_pattern():
    val = yield "ready"
    while val is not None:
        val = yield val * 2


# --- Generator with return value ---

def gen_return_value():
    yield 1
    yield 2
    return 42

def gen_return_none():
    yield 1
    return

def gen_return_early():
    for i in range(10):
        if i > 5:
            return "stopped early"
        yield i


# --- Nested generators ---

def outer():
    def inner():
        yield 1
        yield 2
    for val in inner():
        yield val * 10

def gen_of_gens():
    def make_gen(n):
        for i in range(n):
            yield i
    for g in [make_gen(3), make_gen(2)]:
        yield from g

def recursive_gen(n):
    if n <= 0:
        return
    yield n
    yield from recursive_gen(n - 1)


# --- Generator in class method ---

class IterableContainer:
    def __init__(self, data):
        self.data = data

    def __iter__(self):
        for item in self.data:
            yield item

    def pairs(self):
        for i in range(len(self.data)):
            for j in range(i + 1, len(self.data)):
                yield (self.data[i], self.data[j])

    def filtered(self, predicate):
        for item in self.data:
            if predicate(item):
                yield item

class Tree:
    def __init__(self, value, children=None):
        self.value = value
        self.children = children or []

    def walk(self):
        yield self.value
        for child in self.children:
            yield from child.walk()


# --- Generator with try/finally ---

def gen_with_cleanup():
    try:
        yield 1
        yield 2
    finally:
        cleanup = True

def gen_with_try_except():
    try:
        yield 1
        yield 2
    except GeneratorExit:
        pass
    except ValueError:
        yield -1

def gen_with_full_try():
    try:
        yield 1
    except Exception as e:
        yield -1
    else:
        yield 2
    finally:
        done = True


# --- Generator with conditional yields ---

def conditional_gen(flag):
    if flag:
        yield "yes"
    else:
        yield "no"
    yield "done"

def gen_with_loop():
    for i in range(10):
        if i % 2 == 0:
            yield i

def gen_with_while():
    n = 10
    while n > 0:
        yield n
        n -= 1


# --- Generator with complex yield expressions ---

def gen_complex():
    yield 1 + 2
    yield [x for x in range(5)]
    yield {k: v for k, v in enumerate("abc")}
    yield (lambda x: x * 2)(5)
    # NOTE: starred yield not supported: yield *[1, 2, 3],
    yield (1, 2, 3)

def gen_yield_star():
    data = [1, 2, 3]
    yield (*data,)
    # NOTE: ** in dict literal not supported: yield {**{"a": 1}, **{"b": 2}}
    yield {"a": 1, "b": 2}


# --- Infinite generator ---

def count(start=0, step=1):
    n = start
    while True:
        yield n
        n += step

def cycle(iterable):
    saved = []
    for element in iterable:
        yield element
        saved.append(element)
    while saved:
        for element in saved:
            yield element

def repeat(obj, times=None):
    if times is None:
        while True:
            yield obj
    else:
        for i in range(times):
            yield obj


# --- Generator with assignment expressions (walrus) ---

def gen_with_walrus():
    data = [1, 2, 3, 4, 5]
    yield from (y for x in data if (y := x * 2) > 4)

def gen_walrus_filter():
    items = ["hello", "", "world", "", "!"]
    yield from (stripped for item in items if (stripped := item.strip()))


# --- Generator used in unpacking ---

a, b, c = gen_basic()
first, *rest = (x for x in range(5))
*init, last = (x for x in range(5))


# --- StopIteration handling ---

def gen_stop():
    return 42
    yield

def gen_raise_stop():
    yield 1
    raise StopIteration("finished")
