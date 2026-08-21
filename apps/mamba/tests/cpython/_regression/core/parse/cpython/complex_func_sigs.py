# RUN: parse
# Complex function signature syntax fixture (#575)

# --- simple function ---
def simple():
    pass

# --- with return value ---
def returns_int():
    return 42

# --- positional args ---
def positional(a, b, c):
    pass

# --- default values ---
def defaults(a, b=10, c="hello", d=None):
    pass

# --- *args ---
def var_args(*args):
    pass

# --- **kwargs ---
def var_kwargs(**kwargs):
    pass

# --- *args and **kwargs ---
def both(*args, **kwargs):
    pass

# --- keyword-only args (after *) ---
def keyword_only(*, key1, key2):
    pass

# --- keyword-only with defaults ---
def keyword_only_defaults(*, key1, key2="default"):
    pass

# --- positional + keyword-only ---
def mixed(a, b, *, key1, key2):
    pass

# --- positional + *args + keyword-only ---
def mixed_star(a, *args, key1, key2):
    pass

# --- all parameter types ---
def everything(a, b, c=3, *args, key1, key2="x", **kwargs):
    pass

# --- positional-only (PEP 570) ---
def pos_only(a, b, /):
    pass

# --- positional-only + regular ---
def pos_only_mixed(a, b, /, c, d):
    pass

# --- positional-only + keyword-only ---
def pos_and_kw(a, b, /, *, key):
    pass

# --- all three zones ---
def all_zones(a, b, /, c, d, *, e, f):
    pass

# --- all zones with defaults ---
def all_zones_defaults(a, b=1, /, c=2, d=3, *, e=4, f=5):
    pass

# --- full signature with all features ---
def full_sig(a, b, /, c, d="x", *args, e, f="y", **kwargs):
    pass

# --- type annotations ---
def annotated(x: int, y: str) -> bool:
    pass

# --- complex type annotations ---
def complex_types(
    items: list[int],
    mapping: dict[str, list[int]],
    optional: int | None = None,
) -> tuple[int, str]:
    pass

# --- generic function (PEP 695) ---
def generic[T](x: T) -> T:
    return x

def multi_generic[T, U](x: T, y: U) -> tuple[T, U]:
    return (x, y)

# --- decorators ---
@staticmethod
def static_func():
    pass

@classmethod
def class_func(cls):
    pass

def decorator(func):
    return func

@decorator
def decorated():
    pass

# --- nested decorators ---
def d1(func):
    return func
def d2(func):
    return func
def d3(func):
    return func

@d1
@d2
@d3
def triple_decorated():
    pass

# --- decorator with arguments ---
def with_args(n):
    def dec(func):
        return func
    return dec

@with_args(42)
def decorated_with_args():
    pass

# --- lambda variations ---
f = lambda: None
f = lambda x: x
f = lambda x, y: x + y
f = lambda x, y=10: x + y
f = lambda *args: args
f = lambda **kwargs: kwargs
f = lambda *args, **kwargs: (args, kwargs)
# NOTE: pos-only params (/) in lambda not supported
# f = lambda x, /, y, *, z: (x, y, z)
f = lambda x, y, z: (x, y, z)

# --- async function ---
async def async_func():
    pass

async def async_with_args(x: int, y: str) -> bool:
    pass

async def async_generator():
    yield 1
    yield 2

# --- nested functions ---
def outer():
    def inner():
        def innermost():
            return 42
        return innermost
    return inner

# --- closures ---
def make_adder(n):
    def adder(x):
        return x + n
    return adder

def make_counter():
    count = 0
    def increment():
        nonlocal count
        count += 1
        return count
    return increment

# --- generator functions ---
def simple_gen():
    yield 1
    yield 2
    yield 3

def gen_with_return():
    yield 1
    return "done"

def gen_with_yield_from():
    yield from range(10)

def gen_with_send():
    value = yield
    yield value * 2

# --- complex default values ---
def mutable_default(items=[]):
    pass

def computed_default(x=len("hello")):
    pass

def none_default(x: int | None = None):
    pass

# --- very long signature (multi-line) ---
def long_signature(
    first_param: int,
    second_param: str,
    third_param: float = 3.14,
    fourth_param: bool = True,
    *extra_args: int,
    keyword_one: str = "default",
    keyword_two: list[int] | None = None,
    **extra_kwargs: str,
) -> dict[str, int]:
    pass

# --- function with docstring ---
def documented(x, y):
    """This function does something.

    Args:
        x: First argument
        y: Second argument
    """
    pass

# --- recursive type hint ---
def tree_depth(node: dict | list | None) -> int:
    pass
