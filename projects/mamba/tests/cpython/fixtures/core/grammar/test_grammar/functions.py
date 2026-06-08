# RUN: parse
# CPython 3.12 test_grammar: function definitions

# Simple function
def f():
    pass

# With parameters
def f(a, b, c):
    return a + b + c

# With defaults
def f(a, b=1, c=2):
    return a + b + c

# With type annotations
def f(a: int, b: str = "hello") -> bool:
    return True

# *args and **kwargs
def f(*args, **kwargs):
    pass

# Keyword-only arguments
def f(a, *, b, c=1):
    pass

# Positional-only arguments (Python 3.8+)
def f(a, b, /, c, d):
    pass

# Combined
def f(pos_only, /, normal, *, kw_only):
    pass

# Lambda
add = lambda x, y: x + y
identity = lambda x: x

# Nested functions
def outer(x):
    def inner(y):
        return x + y
    return inner

# Recursive function
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

# Generator function
def gen():
    yield 1
    yield 2
    yield 3

# Async function
async def async_fn():
    pass

async def async_with_await():
    pass
