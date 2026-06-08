# RUN: parse
# CPython 3.12 test_grammar: yield expressions

# Simple generator
def simple_gen():
    yield 1
    yield 2
    yield 3

# Yield with expression
def squares(n):
    for i in range(n):
        yield i ** 2

# Yield in assignment
def accumulator():
    total = 0
    while True:
        value = yield total
        if value is not None:
            total += value

# Yield from (PEP 380)
def chain(*iterables):
    for it in iterables:
        yield from it

def flatten(nested):
    for item in nested:
        if isinstance(item, list):
            yield from flatten(item)
        else:
            yield item

# Generator expression vs yield
def gen_func():
    result = yield 42
    yield result

# Yield in try/finally
def gen_with_cleanup():
    try:
        yield 1
        yield 2
    finally:
        pass
